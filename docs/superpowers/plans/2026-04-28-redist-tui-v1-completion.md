# redist TUI v1 Completion Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the 3 remaining gaps before redist-tui v1 is complete: elapsed timer bug, compare screen stub, and session pre-fill for Doctor screen.

**Architecture:** Pure ratatui, single binary `redist-tui`. All changes in `redist/crates/redist-tui/src/`. See spec: `docs/superpowers/specs/2026-04-27-redist-tui-design.md`.

**Tech Stack:** ratatui 0.28, crossterm 0.28, redist-cli (lib) for PlanContext and analysis loading.

---

## File Map

```
Modified:
  src/main.rs               — fix elapsed_secs timer, Doctor screen input wiring
  src/screens/compare.rs    — full compare screen implementation
  src/screens/run.rs        — elapsed display fix
  src/integration_tests.rs  — new pipeline tests
```

---

## Task 1: Fix elapsed_secs timer bug

**Files:** `src/main.rs:167`, `src/screens/run.rs`

**The bug:** `run_state.progress.elapsed_secs = run_state.progress.elapsed_secs.saturating_add(0);`
`saturating_add(0)` adds zero — the timer never moves. Every run shows "Elapsed: 0:00".

- [ ] **Step 1.1: Read main.rs around line 124-170 to understand the tick loop**

The drain loop runs every time `event::poll()` returns (50ms timeout). We need wall-clock tracking, not tick counting (ticks aren't guaranteed to be exactly 50ms).

- [ ] **Step 1.2: Add start_time to RunState in app.rs**

In `src/app.rs`, add to `RunState`:
```rust
/// Wall-clock start time for elapsed display (set when phase transitions to Running)
pub run_started_at: Option<std::time::Instant>,
```

Update `RunState::default()` to set `run_started_at: None`.

- [ ] **Step 1.3: Set start_time when run begins in main.rs**

In main.rs, in the Enter key handler for Run form that transitions to Running, add:
```rust
state.run_started_at = Some(std::time::Instant::now());
```

- [ ] **Step 1.4: Update elapsed_secs from wall clock in the drain loop**

Replace:
```rust
run_state.progress.elapsed_secs = run_state.progress.elapsed_secs.saturating_add(0);
```
With:
```rust
if let Some(started) = run_state.run_started_at {
    run_state.progress.elapsed_secs = started.elapsed().as_secs();
}
```

- [ ] **Step 1.5: Verify the fix compiles**
```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -5
```
Expected: no errors.

- [ ] **Step 1.6: Add test**

In `src/screens/run.rs` tests:
```rust
#[test]
fn test_run_progress_elapsed_from_instant() {
    use crate::app::RunProgress;
    let start = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = start.elapsed().as_secs();
    // elapsed should be 0 (< 1 second) — just verify no panic
    assert!(elapsed < 60, "elapsed must be reasonable");
}
```

- [ ] **Step 1.7: Commit**
```bash
git add redist/crates/redist-tui/src/
git commit -m "fix(tui): elapsed timer uses wall clock not saturating_add(0)"
```

---

## Task 2: Compare screen — full implementation

**Files:** `src/screens/compare.rs`

Current state: `compare.rs` returns a "Compare screen — coming soon" paragraph.

Per the spec (section "Compare" in design doc): Plan A label (static) + Plan B text input + Jaccard Gauge + metrics table (PP, deviation, splits, contiguous, Δ column) + most-changed districts list + export CSV option.

The data comes from reading analysis JSON files from two plans via PlanContext.

- [ ] **Step 2.1: Add data loading functions**

Add to `compare.rs`:
```rust
use crate::plan_context::PlanContext;
use std::path::PathBuf;

/// Load comparison metrics for a plan by label.
fn load_plan_metrics(
    label: &str,
    output_base: &str,
    version: &str,
    year: &str,
) -> Option<PlanMetrics> {
    let ctx = PlanContext::from_label(
        PathBuf::from(output_base).join(version).as_path(),
        version, year, label
    ).ok()?;
    
    let mean_pp = load_mean_pp(&ctx);
    let max_dev = load_max_dev(&ctx);
    let splits = load_splits(&ctx);
    let contiguous = load_contiguous(&ctx);
    
    Some(PlanMetrics { label: label.to_string(), mean_pp, max_dev, splits, contiguous })
}

#[derive(Debug, Clone, Default)]
pub struct PlanMetrics {
    pub label: String,
    pub mean_pp: Option<f64>,
    pub max_dev: Option<f64>,
    pub splits: Option<usize>,
    pub contiguous: Option<bool>,
}

fn load_mean_pp(ctx: &PlanContext) -> Option<f64> {
    let path = ctx.analysis_file("compactness.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    let d = v["districts"].as_array()?;
    let vals: Vec<f64> = d.iter().filter_map(|x| x["polsby_popper"].as_f64()).collect();
    if vals.is_empty() { None } else { Some(vals.iter().sum::<f64>() / vals.len() as f64) }
}

fn load_max_dev(ctx: &PlanContext) -> Option<f64> {
    let path = ctx.analysis_file("summary.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    v["max_deviation_pct"].as_f64()
}

fn load_splits(ctx: &PlanContext) -> Option<usize> {
    let path = ctx.analysis_file("splits.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    v["split"].as_u64().map(|n| n as usize)
}

fn load_contiguous(ctx: &PlanContext) -> Option<bool> {
    let path = ctx.analysis_file("contiguity.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
    v["all_contiguous"].as_bool()
}
```

- [ ] **Step 2.2: Add CompareResult to app.rs**

`CompareResult` already exists in app.rs (was added during the initial TUI build). Verify it has the right fields. If not, update.

- [ ] **Step 2.3: Implement compare screen render**

Replace the stub in `compare.rs` with the full implementation from the spec:

```rust
pub fn render(f: &mut Frame, area: Rect, state: &CompareState) {
    let block = Block::default().title(" Compare Plans ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // plan inputs
            Constraint::Length(2),   // jaccard
            Constraint::Length(8),   // metrics table
            Constraint::Min(0),      // most-changed / empty
            Constraint::Length(1),   // footer
        ])
        .split(inner);

    // Plan inputs
    f.render_widget(
        Paragraph::new(vec![
            Line::from(format!("  Plan A  {}", state.plan_a)),
            Line::from(vec![
                Span::raw("  Plan B  [ "),
                Span::styled(&state.plan_b_input, Style::default().fg(Color::Yellow)),
                Span::styled("█", Style::default().fg(Color::Yellow)),
                Span::raw(" ]  [Enter] load"),
            ]),
        ]),
        sections[0]
    );

    let Some(ref result) = state.result else {
        f.render_widget(
            Paragraph::new("  Enter Plan B label and press Enter to compare")
                .style(Style::default().fg(Color::DarkGray)),
            sections[1]
        );
        return;
    };

    // Jaccard gauge
    let jcolor = if result.jaccard >= 0.95 { Color::Green }
        else if result.jaccard >= 0.80 { Color::Yellow }
        else { Color::Red };
    let gauge = Gauge::default()
        .label(format!("Similarity  Jaccard {:.3}  [?] what is Jaccard",
            result.jaccard))
        .ratio(result.jaccard.min(1.0).max(0.0))
        .gauge_style(Style::default().fg(jcolor));
    f.render_widget(gauge, sections[1]);

    // Metrics table
    let header = Row::new(vec![
        Cell::from("Metric").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Plan A").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Plan B").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Δ").style(Style::default().add_modifier(Modifier::BOLD)),
    ]).height(1).style(Style::default().bg(Color::DarkGray));

    let pp_delta = result.mean_pp_a - result.mean_pp_b;
    let dev_delta = result.max_dev_a - result.max_dev_b;
    let splits_delta = result.splits_a as i64 - result.splits_b as i64;

    let rows = vec![
        metric_row("Compactness (PP)",
            &format!("{:.2}", result.mean_pp_a),
            &format!("{:.2}", result.mean_pp_b),
            pp_delta, true),
        metric_row("Balance (max dev%)",
            &format!("{:.1}%", result.max_dev_a),
            &format!("{:.1}%", result.max_dev_b),
            -dev_delta, true),
        metric_row("County splits",
            &result.splits_a.to_string(),
            &result.splits_b.to_string(),
            -splits_delta as f64, true),
        Row::new(vec![
            Cell::from("Contiguous"),
            Cell::from(if result.contiguous_a { "✓" } else { "✗" }),
            Cell::from(if result.contiguous_b { "✓" } else { "✗" }),
            Cell::from(if result.contiguous_a == result.contiguous_b { "── same" }
                else if result.contiguous_a { "▲" } else { "▼" }),
        ]),
    ];

    let table = Table::new(rows, [
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ]).header(header);
    f.render_widget(table, sections[2]);

    // Footer
    f.render_widget(
        Paragraph::new("  [Esc] back    [?] Jaccard glossary")
            .style(Style::default().fg(Color::DarkGray)),
        sections[4]
    );
}

fn metric_row<'a>(name: &'a str, a: &'a str, b: &'a str, delta: f64, higher_better: bool) -> Row<'a> {
    let (text, color) = if delta.abs() < 0.001 {
        ("── same".to_string(), Color::Gray)
    } else if (delta > 0.0) == higher_better {
        (format!("{:+.2} ▲", delta), Color::Green)
    } else {
        (format!("{:+.2} ▼", delta), Color::Red)
    };
    Row::new(vec![
        Cell::from(name),
        Cell::from(a),
        Cell::from(b),
        Cell::from(text).style(Style::default().fg(color)),
    ])
}
```

- [ ] **Step 2.4: Wire Plan B input into main.rs key handling**

In `main.rs`, in the Compare screen key handler:
- `KeyCode::Char(c)` → append to `state.plan_b_input`
- `KeyCode::Backspace` → pop from `state.plan_b_input`
- `KeyCode::Enter` → trigger load: call `load_plan_metrics()` to build `CompareResult`, set `state.result`

The `CompareResult` fields (jaccard, mean_pp_a/b, max_dev_a/b, splits_a/b, contiguous_a/b) come from loading both plans' analysis JSON files via PlanContext.

For Jaccard: load both plans' `final_assignments.json` files via PlanContext, compute overlap.

- [ ] **Step 2.5: Add tests**

In `src/screens/compare.rs` tests:
```rust
fn render_compare_to_string(state: &CompareState) -> String {
    let backend = ratatui::backend::TestBackend::new(120, 30);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f, f.area(), state)).unwrap();
    terminal.backend().buffer().content().iter()
        .map(|c| c.symbol().to_string()).collect()
}

#[test]
fn test_compare_shows_plan_a_label() {
    let state = CompareState {
        plan_a: "wa_house_test".into(),
        ..Default::default()
    };
    let content = render_compare_to_string(&state);
    assert!(content.contains("wa_house_test") || content.contains("Plan A"));
}

#[test]
fn test_compare_shows_jaccard_when_result_present() {
    let state = CompareState {
        plan_a: "plan_a".into(),
        result: Some(CompareResult {
            jaccard: 0.847,
            mean_pp_a: 0.31, mean_pp_b: 0.28,
            max_dev_a: 3.2, max_dev_b: 4.1,
            splits_a: 8, splits_b: 12,
            contiguous_a: true, contiguous_b: false,
            most_changed: vec![],
        }),
        ..Default::default()
    };
    let content = render_compare_to_string(&state);
    assert!(content.contains("0.847") || content.contains("Jaccard"),
        "content: {}", &content[..200.min(content.len())]);
}

#[test]
fn test_load_mean_pp_returns_none_when_no_file() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // Create a fake plan with no compactness.json
    let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("test");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"), serde_json::json!({
        "label":"test","state_code":"VT","year":"2020","chamber":"congressional",
        "num_districts":1,"population_source":"total","partition_mode":"edge-weighted",
        "seed":null,"binary_version":"0.1.0","binary_sha256":"","binary_download_url":"",
        "adjacency_file":"","adjacency_sha256":"","adjacency_build_command":"",
        "adjacency_build_version":"0.1.0","tiger_source_url":"","tiger_sha256":null,
        "created_at":"2026-04-28T00:00:00Z","balance_tolerance_pct":0.5,
        "population_balance_valid":true,"seats_per_district":1,"total_seats":1,
        "electoral_system":"single_member","gpmetis_version":"unknown"
    }).to_string()).unwrap();
    
    let ctx = crate::plan_context::PlanContext::from_label(
        tmp.path(), "v1", "2020", "test"
    ).unwrap();
    // No compactness.json exists
    assert!(load_mean_pp(&ctx).is_none(), "must return None when file absent");
}
```

- [ ] **Step 2.7: Commit**
```bash
git add redist/crates/redist-tui/src/
git commit -m "feat(tui): compare screen — Jaccard gauge, metrics table, plan metrics loading"
```

---

## Task 3: Session pre-fill — Doctor screen location input

**Files:** `src/main.rs`, `src/screens/doctor.rs`

Current state: When user opens Doctor screen (`[d]` on Home), it's pre-populated from `default_run_form.location` and `default_run_form.chamber`. But the Doctor screen doesn't have text input — the `DoctorState.location` is set at navigation time and never editable in the TUI.

Fix: Allow the user to type a new location code on the Doctor screen (like the Run form allows typing).

- [ ] **Step 3.1: Add input field to DoctorState in app.rs**

`DoctorState` already has `location: String`. In the Doctor screen render, show it as an editable input similar to the Verify screen's manifest path input.

- [ ] **Step 3.2: Wire character input in main.rs Doctor key handler**

In main.rs, Screen::Doctor key handling, add:
```rust
(KeyCode::Char(c), _) if matches!(app.screen, Screen::Doctor(_)) => {
    if let Screen::Doctor(ref mut state) = app.screen {
        state.location.push(c.to_uppercase().next().unwrap_or(c));
    }
}
(KeyCode::Backspace, _) if matches!(app.screen, Screen::Doctor(_)) => {
    if let Screen::Doctor(ref mut state) = app.screen {
        state.location.pop();
    }
}
```

- [ ] **Step 3.3: Update doctor.rs render to show input cursor**

In `doctor.rs` render, show the location as an editable input with cursor when empty:
```rust
let location_display = if state.location.is_empty() {
    "  Enter location code (e.g., WA, IE, _TEST_EL): █".to_string()
} else {
    format!("  Location: {}█", state.location)
};
```

- [ ] **Step 3.4: Trigger re-run of checks when location changes**

In `doctor.rs render()`, when `state.location` is non-empty, always run the live checks (this already happens via `run_checks(&state)` in render). The checks automatically re-run because render is called on every keypress.

- [ ] **Step 3.5: Add test**

```rust
#[test]
fn test_doctor_screen_empty_location_shows_prompt() {
    use crate::app::DoctorState;
    let state = DoctorState::default();
    let backend = ratatui::backend::TestBackend::new(120, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f, f.area(), &state)).unwrap();
    let content: String = terminal.backend().buffer().content().iter()
        .map(|c| c.symbol().to_string()).collect();
    assert!(
        content.contains("location") || content.contains("Location") || content.contains("WA"),
        "empty doctor must show location prompt: {}", &content[..200.min(content.len())]
    );
}
```

- [ ] **Step 3.6: Commit**
```bash
git add redist/crates/redist-tui/src/
git commit -m "feat(tui): doctor screen location is now editable — type to change"
```

---

## Task 4: Integration test for the full TUI workflow

**Files:** `src/integration_tests.rs`

- [ ] **Step 4.1: Add compare data-loading integration test**

```rust
#[test]
fn test_load_plan_metrics_from_real_analysis() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    // Create plan with compactness.json
    let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("wa_test");
    let analysis_dir = plan_dir.join("analysis");
    std::fs::create_dir_all(&analysis_dir).unwrap();
    
    // Minimal manifest
    std::fs::write(plan_dir.join("manifest.json"), /* ... */ ).unwrap();
    
    // Compactness file with known PP
    std::fs::write(analysis_dir.join("compactness.json"), serde_json::json!({
        "districts": [{"district": 1, "polsby_popper": 0.42}]
    }).to_string()).unwrap();
    
    let metrics = crate::screens::compare::load_plan_metrics(
        "wa_test",
        tmp.path().to_str().unwrap(),
        "v1", "2020"
    );
    
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert!((m.mean_pp.unwrap_or(0.0) - 0.42).abs() < 0.001,
        "mean PP must match compactness.json: {:?}", m.mean_pp);
}
```

- [ ] **Step 4.2: Run full test suite**
```bash
cargo test --workspace --lib 2>&1 | grep "test result"
```
Expected: all pass. Target: 911 + ~8 new = ~919 tests.

- [ ] **Step 4.3: Final commit + push**
```bash
git add redist/crates/redist-tui/src/
git commit -m "feat(tui): v1 complete — timer fix, compare screen, doctor input"
git push origin master:main
```

---

## Self-Review

**Spec coverage:**

| Spec requirement | Task |
|-----------------|------|
| Live elapsed timer | Task 1 |
| Compare screen (Jaccard, metrics table) | Task 2 |
| Doctor screen location editable | Task 3 |
| Integration tests for compare data loading | Task 4 |

**No placeholders** — all code shown. **Type consistency** — `CompareState`, `CompareResult` from `app.rs` used throughout. `PlanContext` from `plan_context.rs` used in data loading.

**Out of scope for v1:**
- Real-time map rendering
- Multi-pane tiling
- Most-changed districts list (deferred to v2 — data loading complex)
- Mouse support
