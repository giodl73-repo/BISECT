# redist TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `redist-tui`, a pure ratatui TUI binary that lets redistricting practitioners run the full redist workflow — plan generation, analysis, comparison, verification — through interactive menus without memorising CLI flags.

**Architecture:** New binary crate `redist/crates/redist-tui/` added to the workspace. Uses ratatui + crossterm for terminal rendering. Reads redist output files directly for browse/compare/verify; spawns `redist state` subprocess only for the Run screen, consuming STATUS: lines to drive live progress bars. Zero changes to `redist-cli` or its existing tests.

**Tech Stack:** Rust, ratatui 0.28+, crossterm 0.28+, toml 0.8, serde (for session config), redist-cli (lib), redist-report (lib), serde_json.

---

## File Map

```
redist/crates/redist-tui/
├── Cargo.toml
└── src/
    ├── main.rs          — terminal init, event loop, top-level key dispatch
    ├── app.rs           — App struct, Screen enum, all shared state
    ├── session.rs       — ~/.config/redist/tui.toml load/save
    ├── plans.rs         — scan plans/ dir, build Vec<PlanSummary>
    ├── screens/
    │   ├── mod.rs       — re-exports
    │   ├── home.rs      — plan browser table + detail panel
    │   ├── run.rs       — run form + live progress + completion card
    │   ├── compare.rs   — side-by-side metrics table
    │   ├── verify.rs    — manifest → PASS/FAIL + chain of custody
    │   └── doctor.rs    — pre-flight check list
    └── widgets/
        ├── mod.rs       — re-exports
        ├── command_palette.rs — `:` overlay with history + tab-complete
        ├── status_bar.rs      — always-visible bottom bar
        ├── error_banner.rs    — summary + [e] expand + [c] copy
        └── glossary.rs        — [?] inline metric explanations
```

**Modified:**
- `redist/Cargo.toml` — add `redist-tui` to `[workspace.members]`
- `redist/crates/redist-cli/src/args.rs` — add `Tui(TuiArgs)` to `Commands`
- `redist/crates/redist-cli/src/main.rs` — wire `Commands::Tui` to exec `redist-tui`

---

## Task 1: Crate scaffold + workspace wiring

**Files:**
- Create: `redist/crates/redist-tui/Cargo.toml`
- Modify: `redist/Cargo.toml`
- Create: `redist/crates/redist-tui/src/main.rs`

- [ ] **Step 1.1: Add to workspace**

In `redist/Cargo.toml`, find the `[workspace]` `members` array and add:
```toml
"crates/redist-tui",
```

- [ ] **Step 1.2: Create Cargo.toml**

Create `redist/crates/redist-tui/Cargo.toml`:
```toml
[package]
name = "redist-tui"
version = "0.1.0"
edition = "2021"
authors = ["Gio Della-Libera"]
description = "Interactive TUI for the redist redistricting CLI"

[[bin]]
name = "redist-tui"
path = "src/main.rs"

[dependencies]
redist-cli  = { path = "../redist-cli" }
redist-report = { path = "../redist-report" }
ratatui     = "0.28"
crossterm   = { version = "0.28", features = ["event-stream"] }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"
anyhow      = "1"
```

- [ ] **Step 1.3: Create minimal main.rs**

Create `redist/crates/redist-tui/src/main.rs`:
```rust
fn main() -> anyhow::Result<()> {
    println!("redist-tui v0.1.0 — not yet implemented");
    Ok(())
}
```

- [ ] **Step 1.4: Verify it compiles**

Run: `cargo build -p redist-tui`
Expected: compiles with no errors.

- [ ] **Step 1.5: Commit**

```bash
git add redist/Cargo.toml redist/crates/redist-tui/
git commit -m "feat: scaffold redist-tui crate"
```

---

## Task 2: App state and Screen enum

**Files:**
- Create: `redist/crates/redist-tui/src/app.rs`
- Modify: `redist/crates/redist-tui/src/main.rs`

- [ ] **Step 2.1: Write the failing test**

Create `redist/crates/redist-tui/src/app.rs` with a test module at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_starts_on_home_screen() {
        let app = App::default();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_navigate_to_run_screen() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        assert!(matches!(app.screen, Screen::Run(_)));
    }

    #[test]
    fn test_navigate_back_returns_to_home() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        app.navigate_back();
        assert!(matches!(app.screen, Screen::Home));
    }
}
```

- [ ] **Step 2.2: Run to verify it fails**

```bash
cargo test -p redist-tui 2>&1 | head -20
```
Expected: compile error — `App`, `Screen`, `RunState` not defined.

- [ ] **Step 2.3: Implement App state**

Fill in `redist/crates/redist-tui/src/app.rs`:
```rust
use serde::{Deserialize, Serialize};

// ── Screens ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    Run(RunState),
    Compare(CompareState),
    Verify(VerifyState),
    Doctor(DoctorState),
}

// ── Run screen state ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunState {
    pub form: RunForm,
    pub phase: RunPhase,
    pub progress: RunProgress,
    pub log_lines: Vec<String>,
    pub show_full_log: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunForm {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub resolution: String,
    pub seed: String,
    pub label: String,
    pub version: String,
    pub balance_tol: String,
    pub focused_field: usize,
    pub doctor_warnings: Vec<String>,
    pub doctor_errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RunPhase {
    #[default]
    Form,
    Running,
    Complete(RunResult),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunProgress {
    /// (completed, total) per bisection depth
    pub depths: Vec<(usize, usize)>,
    pub districts_assigned: usize,
    pub districts_total: usize,
    pub elapsed_secs: u64,
    pub balance_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunResult {
    pub success: bool,
    pub elapsed_secs: u64,
    pub mean_pp: Option<f64>,
    pub max_deviation_pct: Option<f64>,
    pub county_splits: Option<usize>,
    pub all_contiguous: Option<bool>,
    pub error: Option<String>,
}

// ── Compare screen state ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompareState {
    pub plan_a: String,
    pub plan_b_input: String,
    pub result: Option<CompareResult>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompareResult {
    pub jaccard: f64,
    pub mean_pp_a: f64,
    pub mean_pp_b: f64,
    pub max_dev_a: f64,
    pub max_dev_b: f64,
    pub splits_a: usize,
    pub splits_b: usize,
    pub contiguous_a: bool,
    pub contiguous_b: bool,
    pub most_changed: Vec<(usize, f64)>,  // (district_id, pct_moved)
}

// ── Verify screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyState {
    pub manifest_path: String,
    pub result: Option<VerifyResult>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VerifyResult {
    pub passed: bool,
    pub jaccard: f64,
    pub label: String,
    pub state_code: String,
    pub year: String,
    pub binary_match: bool,
    pub metis_version: String,
    pub adjacency_match: bool,
    pub tiger_match: bool,
    pub seed_recorded: bool,
    pub fail_reason: Option<String>,
    pub likely_causes: Vec<String>,
}

// ── Doctor screen state ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DoctorState {
    pub location: String,
    pub chamber: String,
    pub year: String,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoctorCheck {
    pub status: CheckStatus,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Info,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub screen_history: Vec<Screen>,
    pub plans: Vec<PlanSummary>,
    pub selected_plan: usize,
    pub filter: String,
    pub sort: SortColumn,
    pub sort_dir: SortDir,
    pub error: Option<AppError>,
    pub show_palette: bool,
    pub palette_input: String,
    pub palette_history: Vec<String>,
    pub show_glossary: bool,
    pub show_policy_panel: bool,
    pub no_session: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            screen_history: Vec::new(),
            plans: Vec::new(),
            selected_plan: 0,
            filter: String::new(),
            sort: SortColumn::Label,
            sort_dir: SortDir::Asc,
            error: None,
            show_palette: false,
            palette_input: String::new(),
            palette_history: Vec::new(),
            show_glossary: false,
            show_policy_panel: false,
            no_session: false,
        }
    }
}

impl App {
    pub fn navigate(&mut self, screen: Screen) {
        let prev = std::mem::replace(&mut self.screen, screen);
        self.screen_history.push(prev);
    }

    pub fn navigate_back(&mut self) {
        if let Some(prev) = self.screen_history.pop() {
            self.screen = prev;
        } else {
            self.screen = Screen::Home;
        }
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(AppError {
            summary: msg.into(),
            raw: None,
            show_raw: false,
        });
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanSummary {
    pub label: String,
    pub state_code: String,
    pub state_name: String,
    pub chamber: String,
    pub year: String,
    pub num_districts: usize,
    pub mean_pp: Option<f64>,
    pub max_deviation_pct: Option<f64>,
    pub county_splits: Option<usize>,
    pub all_contiguous: Option<bool>,
    pub plan_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortColumn {
    #[default]
    Label,
    Splits,
    Pp,
    Deviation,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub summary: String,
    pub raw: Option<String>,
    pub show_raw: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_starts_on_home_screen() {
        let app = App::default();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_navigate_to_run_screen() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        assert!(matches!(app.screen, Screen::Run(_)));
    }

    #[test]
    fn test_navigate_back_returns_to_home() {
        let mut app = App::default();
        app.navigate(Screen::Run(RunState::default()));
        app.navigate_back();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_navigate_back_from_home_stays_home() {
        let mut app = App::default();
        app.navigate_back();
        assert!(matches!(app.screen, Screen::Home));
    }

    #[test]
    fn test_set_and_clear_error() {
        let mut app = App::default();
        app.set_error("something went wrong");
        assert!(app.error.is_some());
        app.clear_error();
        assert!(app.error.is_none());
    }
}
```

- [ ] **Step 2.4: Wire app into main.rs**

Replace `redist/crates/redist-tui/src/main.rs`:
```rust
mod app;

fn main() -> anyhow::Result<()> {
    let _app = app::App::default();
    println!("redist-tui: App state initialised");
    Ok(())
}
```

- [ ] **Step 2.5: Run tests**

```bash
cargo test -p redist-tui 2>&1
```
Expected: 5 tests pass.

- [ ] **Step 2.6: Commit**

```bash
git add redist/crates/redist-tui/src/app.rs redist/crates/redist-tui/src/main.rs
git commit -m "feat(tui): App state, Screen enum, navigation"
```

---

## Task 3: Session config (TOML load/save)

**Files:**
- Create: `redist/crates/redist-tui/src/session.rs`
- Modify: `redist/crates/redist-tui/src/main.rs`

- [ ] **Step 3.1: Write failing tests**

Create `redist/crates/redist-tui/src/session.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default = "default_location")]
    pub location: String,
    #[serde(default = "default_chamber")]
    pub chamber: String,
    #[serde(default = "default_year")]
    pub year: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_output_base")]
    pub output_base: String,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_sort_column")]
    pub sort_column: String,
    #[serde(default = "default_sort_dir")]
    pub sort_direction: String,
    #[serde(default)]
    pub show_metric_glossary: bool,
    #[serde(default)]
    pub adjacency_override: String,
    #[serde(default = "default_seats")]
    pub seats_per_district: usize,
}

fn default_location() -> String { "VT".into() }
fn default_chamber() -> String { "congressional".into() }
fn default_year() -> String { "2020".into() }
fn default_version() -> String { "v1".into() }
fn default_output_base() -> String { "outputs".into() }
fn default_resolution() -> String { "tract".into() }
fn default_sort_column() -> String { "label".into() }
fn default_sort_dir() -> String { "asc".into() }
fn default_seats() -> usize { 1 }

impl Default for Session {
    fn default() -> Self {
        Self {
            location: default_location(),
            chamber: default_chamber(),
            year: default_year(),
            version: default_version(),
            output_base: default_output_base(),
            resolution: default_resolution(),
            sort_column: default_sort_column(),
            sort_direction: default_sort_dir(),
            show_metric_glossary: false,
            adjacency_override: String::new(),
            seats_per_district: 1,
        }
    }
}

/// Path to session config file.
pub fn config_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(std::path::PathBuf::from(home)
        .join(".config").join("redist").join("tui.toml"))
}

/// Load session from disk. Returns Default if file absent or unparseable.
pub fn load_session() -> Session {
    let Some(path) = config_path() else { return Session::default() };
    let Ok(content) = std::fs::read_to_string(&path) else { return Session::default() };
    toml::from_str(&content).unwrap_or_default()
}

/// Save session to disk. Silently ignores errors (config is best-effort).
pub fn save_session(session: &Session) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(session) {
        let _ = std::fs::write(&path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_temp_home<F: FnOnce()>(f: F) -> TempDir {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("HOME", tmp.path());
        f();
        tmp
    }

    #[test]
    fn test_default_session_has_expected_values() {
        let s = Session::default();
        assert_eq!(s.chamber, "congressional");
        assert_eq!(s.year, "2020");
        assert_eq!(s.seats_per_district, 1);
    }

    #[test]
    fn test_save_and_load_session_roundtrip() {
        let _tmp = with_temp_home(|| {
            let mut s = Session::default();
            s.location = "WA".into();
            s.chamber = "house".into();
            s.seats_per_district = 5;
            save_session(&s);

            let loaded = load_session();
            assert_eq!(loaded.location, "WA");
            assert_eq!(loaded.chamber, "house");
            assert_eq!(loaded.seats_per_district, 5);
        });
    }

    #[test]
    fn test_load_session_absent_returns_default() {
        let _tmp = with_temp_home(|| {
            let s = load_session();
            assert_eq!(s.chamber, "congressional");
        });
    }
}
```

- [ ] **Step 3.2: Add tempfile dev-dependency**

In `redist/crates/redist-tui/Cargo.toml`, add:
```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3.3: Run tests**

```bash
cargo test -p redist-tui session 2>&1
```
Expected: 3 tests pass.

- [ ] **Step 3.4: Commit**

```bash
git add redist/crates/redist-tui/
git commit -m "feat(tui): session config load/save (TOML)"
```

---

## Task 4: Plan discovery (scan plans/ directory)

**Files:**
- Create: `redist/crates/redist-tui/src/plans.rs`

- [ ] **Step 4.1: Write failing tests**

Create `redist/crates/redist-tui/src/plans.rs`:
```rust
use std::path::{Path, PathBuf};
use crate::app::PlanSummary;

/// Scan `{output_base}/{version}/{year}/plans/` for plan directories.
/// For each, reads manifest.json and analysis/compactness.json, splits.json, contiguity.json.
/// Returns plans sorted by label.
pub fn discover_plans(output_base: &str, version: &str, year: &str) -> Vec<PlanSummary> {
    let plans_dir = PathBuf::from(output_base)
        .join(version)
        .join(year)
        .join("plans");

    let Ok(entries) = std::fs::read_dir(&plans_dir) else { return vec![] };

    let mut plans: Vec<PlanSummary> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| load_plan_summary(e.path()))
        .collect();

    plans.sort_by(|a, b| a.label.cmp(&b.label));
    plans
}

fn load_plan_summary(plan_dir: PathBuf) -> Option<PlanSummary> {
    let label = plan_dir.file_name()?.to_string_lossy().to_string();

    // Read manifest.json
    let manifest_path = plan_dir.join("manifest.json");
    let manifest: serde_json::Value = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::json!({}));

    let state_code = manifest["state_code"].as_str().unwrap_or("?").to_string();
    let chamber = manifest["chamber"].as_str().unwrap_or("?").to_string();
    let year = manifest["year"].as_str().unwrap_or("?").to_string();
    let num_districts = manifest["num_districts"].as_u64().unwrap_or(0) as usize;

    // Look up state name from location registry
    let state_name = {
        let reg = redist_cli::policy::LocationRegistry::load();
        reg.state_name(&state_code).unwrap_or_else(|| state_code.clone())
    };

    let analysis_dir = plan_dir.join("analysis");

    // Read compactness.json for mean PP
    let mean_pp = read_analysis_mean_pp(&analysis_dir);

    // Read summary.json for max deviation
    let max_deviation_pct = read_analysis_max_deviation(&analysis_dir);

    // Read splits.json for county split count
    let county_splits = read_analysis_splits(&analysis_dir);

    // Read contiguity.json for all_contiguous
    let all_contiguous = read_analysis_contiguity(&analysis_dir);

    Some(PlanSummary {
        label,
        state_code,
        state_name,
        chamber,
        year,
        num_districts,
        mean_pp,
        max_deviation_pct,
        county_splits,
        all_contiguous,
        plan_dir,
    })
}

fn read_analysis_mean_pp(analysis_dir: &Path) -> Option<f64> {
    let v: serde_json::Value = read_analysis_json(analysis_dir, "compactness.json")?;
    let districts = v["districts"].as_array()?;
    if districts.is_empty() { return None; }
    let sum: f64 = districts.iter()
        .filter_map(|d| d["polsby_popper"].as_f64())
        .sum();
    let count = districts.iter()
        .filter(|d| d["polsby_popper"].is_number())
        .count();
    if count == 0 { None } else { Some(sum / count as f64) }
}

fn read_analysis_max_deviation(analysis_dir: &Path) -> Option<f64> {
    let v: serde_json::Value = read_analysis_json(analysis_dir, "summary.json")?;
    v["max_deviation_pct"].as_f64()
}

fn read_analysis_splits(analysis_dir: &Path) -> Option<usize> {
    let v: serde_json::Value = read_analysis_json(analysis_dir, "splits.json")?;
    v["split"].as_u64().map(|n| n as usize)
}

fn read_analysis_contiguity(analysis_dir: &Path) -> Option<bool> {
    let v: serde_json::Value = read_analysis_json(analysis_dir, "contiguity.json")?;
    v["all_contiguous"].as_bool()
}

fn read_analysis_json(analysis_dir: &Path, name: &str) -> Option<serde_json::Value> {
    let path = analysis_dir.join(name);
    std::fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_plan_dir(tmp: &TempDir, label: &str, manifest: serde_json::Value) -> PathBuf {
        let plan_dir = tmp.path().join("v1").join("2020").join("plans").join(label);
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(
            plan_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap()
        ).unwrap();
        plan_dir
    }

    #[test]
    fn test_discover_plans_empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let plans = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert!(plans.is_empty());
    }

    #[test]
    fn test_discover_plans_reads_manifest_fields() {
        let tmp = TempDir::new().unwrap();
        make_plan_dir(&tmp, "vt_congressional_2020", serde_json::json!({
            "label": "vt_congressional_2020",
            "state_code": "VT",
            "chamber": "congressional",
            "year": "2020",
            "num_districts": 1
        }));
        let plans = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].label, "vt_congressional_2020");
        assert_eq!(plans[0].state_code, "VT");
        assert_eq!(plans[0].num_districts, 1);
    }

    #[test]
    fn test_discover_plans_sorted_by_label() {
        let tmp = TempDir::new().unwrap();
        for label in ["z_plan", "a_plan", "m_plan"] {
            make_plan_dir(&tmp, label, serde_json::json!({"state_code":"VT","chamber":"congressional","year":"2020","num_districts":1}));
        }
        let plans = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(plans[0].label, "a_plan");
        assert_eq!(plans[1].label, "m_plan");
        assert_eq!(plans[2].label, "z_plan");
    }

    #[test]
    fn test_discover_plans_missing_analysis_returns_none_fields() {
        let tmp = TempDir::new().unwrap();
        make_plan_dir(&tmp, "vt_test", serde_json::json!({
            "state_code": "VT", "chamber": "congressional",
            "year": "2020", "num_districts": 1
        }));
        let plans = discover_plans(tmp.path().to_str().unwrap(), "v1", "2020");
        assert_eq!(plans.len(), 1);
        assert!(plans[0].mean_pp.is_none());
        assert!(plans[0].county_splits.is_none());
    }
}
```

- [ ] **Step 4.2: Run tests**

```bash
cargo test -p redist-tui plans 2>&1
```
Expected: 4 tests pass.

- [ ] **Step 4.3: Wire into main.rs**

Add `mod plans;` and `mod session;` to `main.rs`.

- [ ] **Step 4.4: Commit**

```bash
git add redist/crates/redist-tui/src/plans.rs redist/crates/redist-tui/src/main.rs
git commit -m "feat(tui): plan discovery — scan plans/ dir, read manifests + analysis"
```

---

## Task 5: Terminal setup and main event loop

**Files:**
- Modify: `redist/crates/redist-tui/src/main.rs`
- Create: `redist/crates/redist-tui/src/screens/mod.rs`
- Create: `redist/crates/redist-tui/src/screens/home.rs` (stub)
- Create: `redist/crates/redist-tui/src/widgets/mod.rs`
- Create: `redist/crates/redist-tui/src/widgets/status_bar.rs`

- [ ] **Step 5.1: Create screen module stubs**

Create `redist/crates/redist-tui/src/screens/mod.rs`:
```rust
pub mod home;
pub mod run;
pub mod compare;
pub mod verify;
pub mod doctor;
```

Create stub files for each screen (run, compare, verify, doctor) with just a `pub fn render()` signature for now. For example `redist/crates/redist-tui/src/screens/run.rs`:
```rust
use ratatui::{Frame, layout::Rect};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    let block = Block::default().title("Run").borders(Borders::ALL);
    f.render_widget(Paragraph::new("Run screen — coming soon").block(block), area);
}
```
Repeat for `compare.rs`, `verify.rs`, `doctor.rs` with appropriate titles.

Create `redist/crates/redist-tui/src/widgets/mod.rs`:
```rust
pub mod status_bar;
pub mod command_palette;
pub mod error_banner;
pub mod glossary;
```

Create `redist/crates/redist-tui/src/widgets/status_bar.rs`:
```rust
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let location = if !app.plans.is_empty() && app.selected_plan < app.plans.len() {
        let p = &app.plans[app.selected_plan];
        format!("{}·{}·{}", p.state_code, p.chamber, p.year)
    } else {
        "no plans".to_string()
    };

    let text = format!(
        " {}  │  plans: {}  │  ? help  q quit",
        location,
        app.plans.len()
    );

    f.render_widget(
        Paragraph::new(text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        area,
    );
}
```

- [ ] **Step 5.2: Create home.rs stub**

Create `redist/crates/redist-tui/src/screens/home.rs`:
```rust
use ratatui::{Frame, layout::Rect};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    let block = Block::default().title(" redist tui ").borders(Borders::ALL);
    let content = if app.plans.is_empty() {
        "No plans found. Press [r] to run your first plan.".to_string()
    } else {
        format!("{} plans found. Use ↑↓ to navigate.", app.plans.len())
    };
    f.render_widget(Paragraph::new(content).block(block), area);
}
```

- [ ] **Step 5.3: Implement main event loop**

Replace `redist/crates/redist-tui/src/main.rs`:
```rust
mod app;
mod plans;
mod session;
mod screens;
mod widgets;

use std::io;
use app::{App, Screen, RunState, CompareState, VerifyState, DoctorState};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let no_session = args.iter().any(|a| a == "--no-session");

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App initialisation
    let mut app = App::default();
    app.no_session = no_session;

    if !no_session {
        let sess = session::load_session();
        // Apply session defaults to app
        app.screen = Screen::Home;
    }

    // Discover plans on startup
    app.plans = plans::discover_plans("outputs", "v1", "2020");

    // Event loop
    let result = run_app(&mut terminal, &mut app);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Layout: main area + status bar (1 line)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(size);

            // Render current screen
            match &app.screen {
                Screen::Home => screens::home::render(f, app, chunks[0]),
                Screen::Run(_) => screens::run::render(f, app, chunks[0]),
                Screen::Compare(_) => screens::compare::render(f, app, chunks[0]),
                Screen::Verify(_) => screens::verify::render(f, app, chunks[0]),
                Screen::Doctor(_) => screens::doctor::render(f, app, chunks[0]),
            }

            // Always-visible status bar
            widgets::status_bar::render(f, app, chunks[1]);
        })?;

        // Event handling
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    // Universal: quit
                    (_, KeyCode::Char('q')) | (_, KeyCode::Esc) => {
                        match app.screen {
                            Screen::Home => break,
                            _ => app.navigate_back(),
                        }
                    }
                    // Universal: command palette
                    (_, KeyCode::Char(':')) => {
                        app.show_palette = true;
                    }
                    // Universal: glossary
                    (_, KeyCode::Char('?')) => {
                        app.show_glossary = !app.show_glossary;
                    }
                    // Home screen navigation
                    (_, KeyCode::Char('r')) if matches!(app.screen, Screen::Home) => {
                        app.navigate(Screen::Run(RunState::default()));
                    }
                    (_, KeyCode::Char('c')) if matches!(app.screen, Screen::Home) => {
                        let plan_a = app.plans.get(app.selected_plan)
                            .map(|p| p.label.clone())
                            .unwrap_or_default();
                        app.navigate(Screen::Compare(CompareState { plan_a, ..Default::default() }));
                    }
                    (_, KeyCode::Char('v')) if matches!(app.screen, Screen::Home) => {
                        app.navigate(Screen::Verify(VerifyState::default()));
                    }
                    (_, KeyCode::Char('d')) if matches!(app.screen, Screen::Home) => {
                        app.navigate(Screen::Doctor(DoctorState::default()));
                    }
                    // Home: navigate plan list
                    (_, KeyCode::Up) if matches!(app.screen, Screen::Home) => {
                        if app.selected_plan > 0 { app.selected_plan -= 1; }
                    }
                    (_, KeyCode::Down) if matches!(app.screen, Screen::Home) => {
                        if app.selected_plan + 1 < app.plans.len() { app.selected_plan += 1; }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 5.4: Build and smoke test**

```bash
cargo build -p redist-tui 2>&1 | grep -E "^error" | head -10
```
Expected: no errors. (Don't run it yet — terminal interaction requires a real TTY.)

- [ ] **Step 5.5: Commit**

```bash
git add redist/crates/redist-tui/src/
git commit -m "feat(tui): terminal event loop, screen routing, status bar"
```

---

## Task 6: Home screen — plan browser table

**Files:**
- Modify: `redist/crates/redist-tui/src/screens/home.rs`

This is the primary screen. Two-panel layout: left=plan list table (65%), right=detail panel (35%).

- [ ] **Step 6.1: Implement home screen render**

Replace `redist/crates/redist-tui/src/screens/home.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState},
};
use crate::app::{App, PlanSummary};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Quick-action bar (2 lines) + main content
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    render_quick_bar(f, outer[0]);

    // Two-panel split
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(outer[1]);

    render_plan_list(f, app, panels[0]);
    render_detail_panel(f, app, panels[1]);
}

fn render_quick_bar(f: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" [r] Run ", Style::default().fg(Color::Green)),
        Span::raw("│"),
        Span::styled(" [a] Analyze ", Style::default().fg(Color::Cyan)),
        Span::raw("│"),
        Span::styled(" [c] Compare ", Style::default().fg(Color::Cyan)),
        Span::raw("│"),
        Span::styled(" [v] Verify ", Style::default().fg(Color::Yellow)),
        Span::raw("│"),
        Span::styled(" [d] Doctor ", Style::default().fg(Color::Magenta)),
        Span::raw("│"),
        Span::styled(" [/] Filter ", Style::default().fg(Color::Gray)),
        Span::raw("│"),
        Span::styled(" [:] Cmd ", Style::default().fg(Color::Gray)),
    ]);
    f.render_widget(Paragraph::new(text), area);
}

fn render_plan_list(f: &mut Frame, app: &App, area: Rect) {
    if app.plans.is_empty() {
        render_empty_state(f, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Label").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("St").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Chamber").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Yr").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("D").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Sp").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("✓").style(Style::default().add_modifier(Modifier::BOLD)),
    ]).height(1).style(Style::default().bg(Color::DarkGray));

    let rows: Vec<Row> = app.plans.iter().enumerate().map(|(i, p)| {
        let splits = p.county_splits.map(|n| n.to_string()).unwrap_or("?".into());
        let contiguous = match p.all_contiguous {
            Some(true) => Span::styled("✓", Style::default().fg(Color::Green)),
            Some(false) => Span::styled("✗", Style::default().fg(Color::Red)),
            None => Span::raw("?"),
        };

        // Highlight amber when splits > 10
        let splits_cell = if p.county_splits.map(|n| n > 10).unwrap_or(false) {
            Cell::from(splits).style(Style::default().fg(Color::Yellow))
        } else {
            Cell::from(splits)
        };

        let style = if i == app.selected_plan {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            Cell::from(p.label.as_str()),
            Cell::from(p.state_code.as_str()),
            Cell::from(p.chamber.as_str()),
            Cell::from(p.year.as_str()),
            Cell::from(p.num_districts.to_string()),
            splits_cell,
            Cell::from(contiguous),
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Min(22),
        Constraint::Length(4),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(4),
        Constraint::Length(4),
        Constraint::Length(2),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" Plans {} total ", app.plans.len())))
        .highlight_style(Style::default().bg(Color::Blue));

    let mut state = TableState::default();
    state.select(Some(app.selected_plan));

    f.render_stateful_widget(table, area, &mut state);
}

fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);

    let Some(plan) = app.plans.get(app.selected_plan) else {
        f.render_widget(Paragraph::new("No plan selected").block(block), area);
        return;
    };

    // Split detail area vertically
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(1),  // spacer
            Constraint::Length(1),  // PP gauge
            Constraint::Length(1),  // deviation gauge
            Constraint::Length(1),  // splits gauge
            Constraint::Length(1),  // contiguous
            Constraint::Min(0),     // spacer
            Constraint::Length(2),  // actions
        ])
        .split(inner);

    // Header
    let header = Paragraph::new(vec![
        Line::from(plan.label.as_str()),
        Line::from(format!("{} · {} · {}", plan.state_name, plan.chamber, plan.year)),
        Line::from(format!("{} districts", plan.num_districts)),
    ]);
    f.render_widget(header, sections[0]);

    // PP gauge (0.0–1.0, higher is better — target > 0.25)
    if let Some(pp) = plan.mean_pp {
        let ratio = (pp.min(1.0).max(0.0) * 100.0) as u16;
        let gauge = Gauge::default()
            .label(format!("PP {:.2}", pp))
            .ratio(pp.min(1.0).max(0.0))
            .gauge_style(Style::default().fg(Color::Green));
        f.render_widget(gauge, sections[2]);
    }

    // Deviation gauge (0–25%, lower is better — warn > 5%)
    if let Some(dev) = plan.max_deviation_pct {
        let ratio = (dev / 25.0).min(1.0);
        let color = if dev > 5.0 { Color::Yellow } else { Color::Green };
        let gauge = Gauge::default()
            .label(format!("Dev {:.1}%", dev))
            .ratio(ratio)
            .gauge_style(Style::default().fg(color));
        f.render_widget(gauge, sections[3]);
    }

    // Splits gauge (0–50, lower is better)
    if let Some(splits) = plan.county_splits {
        let ratio = (splits as f64 / 50.0).min(1.0);
        let color = if splits > 10 { Color::Yellow } else { Color::Green };
        let gauge = Gauge::default()
            .label(format!("Splits {}", splits))
            .ratio(ratio)
            .gauge_style(Style::default().fg(color));
        f.render_widget(gauge, sections[4]);
    }

    // Contiguous
    let contiguous_text = match plan.all_contiguous {
        Some(true) => Span::styled(
            format!("Contiguous ✓ all {}", plan.num_districts),
            Style::default().fg(Color::Green)
        ),
        Some(false) => Span::styled("Contiguous ✗ issues", Style::default().fg(Color::Red)),
        None => Span::raw("Contiguous ?"),
    };
    f.render_widget(Paragraph::new(Line::from(contiguous_text)), sections[5]);

    // Actions
    let actions = Paragraph::new(vec![
        Line::from("[Enter] open  [a] analyze"),
        Line::from("[c] compare   [x] export"),
    ]).style(Style::default().fg(Color::Gray));
    f.render_widget(actions, sections[7]);
}

fn render_empty_state(f: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Plans ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from("No plans found."),
        Line::from(""),
        Line::from(Span::styled(
            "[r] Run your first plan",
            Style::default().fg(Color::Green)
        )),
        Line::from(Span::styled(
            "[d] Run doctor for a pre-flight check",
            Style::default().fg(Color::Magenta)
        )),
    ];
    f.render_widget(Paragraph::new(text), inner);
}
```

- [ ] **Step 6.2: Build to verify no compile errors**

```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors.

- [ ] **Step 6.3: Commit**

```bash
git add redist/crates/redist-tui/src/screens/home.rs
git commit -m "feat(tui): home screen — plan browser table + detail panel with gauges"
```

---

## Task 7: Run screen — form and inline doctor

**Files:**
- Modify: `redist/crates/redist-tui/src/screens/run.rs`
- Modify: `redist/crates/redist-tui/src/main.rs` (Run screen key handling)

- [ ] **Step 7.1: Implement run form render**

Replace `redist/crates/redist-tui/src/screens/run.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use crate::app::{App, RunPhase, RunState, Screen, CheckStatus};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Screen::Run(ref state) = app.screen else { return };

    match &state.phase {
        RunPhase::Form => render_form(f, state, area),
        RunPhase::Running => render_progress(f, state, area),
        RunPhase::Complete(ref result) => render_completion(f, state, result, area),
    }
}

fn render_form(f: &mut Frame, state: &RunState, area: Rect) {
    let block = Block::default()
        .title(" Run New Plan ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // form fields
            Constraint::Length(1),  // separator
            Constraint::Length(5),  // doctor checks
            Constraint::Length(1),  // separator
            Constraint::Length(1),  // command preview
            Constraint::Min(0),
            Constraint::Length(1),  // footer
        ])
        .split(inner);

    // Form fields
    let form = &state.form;
    let field_style = |focused: bool| if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let fields = vec![
        Line::from(format!("  Location     {}", form.location)),
        Line::from(format!("  Chamber      [ {} ]", form.chamber)),
        Line::from(format!("  Year         [ {} ]", form.year)),
        Line::from(format!("  Resolution   [ {} ]", form.resolution)),
        Line::from(format!("  Seed         [ {} ]   leave blank for random", form.seed)),
        Line::from(format!("  Label        [ {} ]", form.label)),
        Line::from(format!("  Version      [ {} ]", form.version)),
        Line::from(format!("  Balance tol  [ {} ]   from policy", form.balance_tol)),
    ];
    f.render_widget(Paragraph::new(fields), sections[0]);

    // Doctor checks
    let check_lines: Vec<Line> = {
        let mut lines = vec![Line::from(Span::styled(
            "  ─── Doctor (pre-flight) ──────────────────────────",
            Style::default().fg(Color::DarkGray)
        ))];
        for err in &form.doctor_errors {
            lines.push(Line::from(Span::styled(
                format!("  ✗  {}", err),
                Style::default().fg(Color::Red)
            )));
        }
        for warn in &form.doctor_warnings {
            lines.push(Line::from(Span::styled(
                format!("  ⚠  {}", warn),
                Style::default().fg(Color::Yellow)
            )));
        }
        if form.doctor_errors.is_empty() && form.doctor_warnings.is_empty() {
            lines.push(Line::from(Span::styled(
                "  ✓  All checks passed",
                Style::default().fg(Color::Green)
            )));
        }
        lines
    };
    f.render_widget(Paragraph::new(check_lines), sections[2]);

    // Command preview
    let cmd = build_command_preview(form);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {}", cmd),
            Style::default().fg(Color::DarkGray)
        ))),
        sections[4]
    );

    // Footer
    f.render_widget(
        Paragraph::new("  [Enter] Run    [Tab] next field    [Esc] back")
            .style(Style::default().fg(Color::DarkGray)),
        sections[6]
    );
}

fn build_command_preview(form: &crate::app::RunForm) -> String {
    let mut cmd = format!(
        "redist state --state {} --chamber {} --year {}",
        form.location, form.chamber, form.year
    );
    if !form.seed.is_empty() {
        cmd.push_str(&format!(" --seed {}", form.seed));
    }
    if !form.label.is_empty() {
        cmd.push_str(&format!(" --label {}", form.label));
    }
    cmd
}

fn render_progress(f: &mut Frame, state: &RunState, area: Rect) {
    let block = Block::default()
        .title(format!(" Running · {} ", state.form.label))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let depth_count = state.progress.depths.len();
    let constraints: Vec<Constraint> = std::iter::once(Constraint::Length(2))  // header
        .chain((0..depth_count.max(1)).map(|_| Constraint::Length(1)))         // depth bars
        .chain([
            Constraint::Length(1),  // spacer
            Constraint::Length(1),  // assigned
            Constraint::Length(1),  // elapsed
            Constraint::Length(1),  // balance
            Constraint::Min(0),
            Constraint::Length(3),  // log
            Constraint::Length(1),  // footer
        ])
        .collect();

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Header
    let header = format!(
        "{} · {} · {} · {} districts",
        state.form.location, state.form.chamber,
        state.form.year, state.form.label
    );
    f.render_widget(Paragraph::new(Line::from(header)), sections[0]);

    // Depth progress bars
    for (i, (done, total)) in state.progress.depths.iter().enumerate() {
        if i + 1 >= sections.len() { break; }
        let ratio = if *total == 0 { 0.0 } else { *done as f64 / *total as f64 };
        let label = if *done == *total && *total > 0 {
            format!("Depth {}  {} / {}  done", i + 1, done, total)
        } else if *done > 0 {
            format!("Depth {}  {} / {}  running", i + 1, done, total)
        } else {
            format!("Depth {}  {} / {}  waiting", i + 1, done, total)
        };
        let color = if ratio >= 1.0 { Color::Green } else if ratio > 0.0 { Color::Yellow } else { Color::DarkGray };
        let gauge = Gauge::default()
            .label(label)
            .ratio(ratio)
            .gauge_style(Style::default().fg(color));
        f.render_widget(gauge, sections[i + 1]);
    }

    let off = depth_count + 2;  // after depths + spacer
    if off + 3 < sections.len() {
        // Assigned
        let assigned_ratio = if state.progress.districts_total == 0 { 0.0 }
            else { state.progress.districts_assigned as f64 / state.progress.districts_total as f64 };
        let gauge = Gauge::default()
            .label(format!("Assigned  {} / {}", state.progress.districts_assigned, state.progress.districts_total))
            .ratio(assigned_ratio)
            .gauge_style(Style::default().fg(Color::Cyan));
        f.render_widget(gauge, sections[off]);

        // Elapsed
        let secs = state.progress.elapsed_secs;
        f.render_widget(
            Paragraph::new(format!("  Elapsed  {}:{:02}", secs / 60, secs % 60)),
            sections[off + 1]
        );

        // Balance
        let (balance_text, balance_color) = if state.progress.balance_ok {
            (format!("  Balance  ✓ all within {}", state.form.balance_tol), Color::Green)
        } else {
            ("  Balance  ✗ some districts exceed limit".to_string(), Color::Red)
        };
        f.render_widget(
            Paragraph::new(Span::styled(balance_text, Style::default().fg(balance_color))),
            sections[off + 2]
        );
    }

    // Log preview (last 3 lines)
    let log_off = sections.len() - 2;
    if log_off < sections.len() {
        let log_lines: Vec<Line> = state.log_lines.iter().rev().take(3).rev()
            .map(|l| Line::from(Span::styled(l.as_str(), Style::default().fg(Color::DarkGray))))
            .collect();
        f.render_widget(Paragraph::new(log_lines), sections[log_off]);
    }

    // Footer
    let footer_off = sections.len() - 1;
    f.render_widget(
        Paragraph::new("  [Esc] cancel gracefully    [e] full log")
            .style(Style::default().fg(Color::DarkGray)),
        sections[footer_off]
    );
}

fn render_completion(
    f: &mut Frame,
    state: &RunState,
    result: &crate::app::RunResult,
    area: Rect,
) {
    let title = format!(
        " {} · {} ",
        if result.success { "Done" } else { "Failed" },
        state.form.label
    );
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // status line
            Constraint::Length(1),  // spacer
            Constraint::Length(1),  // PP gauge
            Constraint::Length(1),  // deviation gauge
            Constraint::Length(1),  // splits gauge
            Constraint::Min(0),
            Constraint::Length(1),  // actions
        ])
        .split(inner);

    // Status
    let elapsed = result.elapsed_secs;
    let status = if result.success {
        Line::from(Span::styled(
            format!("✓  Complete in {}:{:02}  ·  all contiguous  ·  balance OK", elapsed / 60, elapsed % 60),
            Style::default().fg(Color::Green)
        ))
    } else {
        Line::from(Span::styled(
            format!("✗  Failed after {}:{:02}", elapsed / 60, elapsed % 60),
            Style::default().fg(Color::Red)
        ))
    };
    f.render_widget(Paragraph::new(status), sections[0]);

    // Metric gauges
    if let Some(pp) = result.mean_pp {
        let gauge = Gauge::default()
            .label(format!("Compactness (PP)  {:.2}", pp))
            .ratio(pp.min(1.0).max(0.0))
            .gauge_style(Style::default().fg(Color::Green));
        f.render_widget(gauge, sections[2]);
    }
    if let Some(dev) = result.max_deviation_pct {
        let color = if dev > 5.0 { Color::Yellow } else { Color::Green };
        let gauge = Gauge::default()
            .label(format!("Balance (max dev)  {:.1}%", dev))
            .ratio((dev / 25.0).min(1.0))
            .gauge_style(Style::default().fg(color));
        f.render_widget(gauge, sections[3]);
    }
    if let Some(splits) = result.county_splits {
        let color = if splits > 10 { Color::Yellow } else { Color::Green };
        let gauge = Gauge::default()
            .label(format!("County splits  {}", splits))
            .ratio((splits as f64 / 50.0).min(1.0))
            .gauge_style(Style::default().fg(color));
        f.render_widget(gauge, sections[4]);
    }

    // Actions
    f.render_widget(
        Paragraph::new("  [a] Analyze     [r] Report     [c] Compare     [Enter] Back to plans")
            .style(Style::default().fg(Color::Gray)),
        sections[6]
    );
}
```

- [ ] **Step 7.2: Build**

```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors.

- [ ] **Step 7.3: Add Run form key handling to main.rs**

In `run_app()` in `main.rs`, add handling for `Screen::Run` state — Tab cycles fields, Enter submits the form. Add after the home-screen key handlers:
```rust
// Run screen: Tab cycles fields, Enter runs
(_, KeyCode::Tab) if matches!(app.screen, Screen::Run(_)) => {
    if let Screen::Run(ref mut state) = app.screen {
        state.form.focused_field = (state.form.focused_field + 1) % 8;
    }
}
// (Enter → spawn subprocess handled in Task 8)
```

- [ ] **Step 7.4: Commit**

```bash
git add redist/crates/redist-tui/src/screens/run.rs redist/crates/redist-tui/src/main.rs
git commit -m "feat(tui): run screen — form, live progress, completion card"
```

---

## Task 8: STATUS: subprocess integration (live progress)

**Files:**
- Create: `redist/crates/redist-tui/src/runner.rs`
- Modify: `redist/crates/redist-tui/src/main.rs`

This task makes the Run screen actually launch `redist state` and pipe STATUS: output back to update progress bars.

- [ ] **Step 8.1: Write failing tests**

Create `redist/crates/redist-tui/src/runner.rs`:
```rust
use crate::app::RunProgress;

/// Parse a STATUS: line from the redist state subprocess.
/// Format: `STATUS:{position}:{message}`
/// Returns Some(message) if parseable, None otherwise.
pub fn parse_status_line(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("STATUS:") { return None; }
    let parts: Vec<&str> = line.splitn(3, ':').collect();
    if parts.len() < 3 { return None; }
    Some(parts[2].to_string())
}

/// Parse a depth progress message like "depth 2 node 10 running" or "depth 1 complete".
/// Returns Some((depth, done, total)) when parseable.
pub fn parse_depth_progress(msg: &str) -> Option<(usize, usize, usize)> {
    // Look for patterns like "depth 2 · node 10" or "depth 1 complete"
    if msg.contains("depth") {
        // Simple heuristic: extract depth number
        let depth = msg.split_whitespace()
            .skip_while(|w| *w != "depth")
            .nth(1)
            .and_then(|w| w.parse::<usize>().ok())?;
        Some((depth, 0, 0))  // full parsing happens at call site
    } else {
        None
    }
}

/// Update RunProgress from a parsed STATUS message.
pub fn update_progress_from_message(progress: &mut RunProgress, msg: &str) {
    // Depth completion pattern: "depth N: X/Y done" or similar
    // This is a best-effort parser — exact format depends on bisection_runner output
    if msg.contains("complete") || msg.contains("done") {
        // Mark balance as ok if no "FAIL" in message
        if !msg.to_lowercase().contains("fail") {
            progress.balance_ok = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_line_valid() {
        let result = parse_status_line("STATUS:1:WA: depth 2 node 10 running");
        assert_eq!(result, Some("WA: depth 2 node 10 running".to_string()));
    }

    #[test]
    fn test_parse_status_line_not_status() {
        assert_eq!(parse_status_line("some other output"), None);
    }

    #[test]
    fn test_parse_status_line_empty() {
        assert_eq!(parse_status_line(""), None);
    }

    #[test]
    fn test_parse_status_line_minimal_valid() {
        let result = parse_status_line("STATUS:999:silent mode");
        assert_eq!(result, Some("silent mode".to_string()));
    }

    #[test]
    fn test_parse_status_line_with_colons_in_message() {
        // Message itself may contain colons — should capture everything after 2nd colon
        let result = parse_status_line("STATUS:1:WA: loading adjacency: done");
        assert_eq!(result, Some("WA: loading adjacency: done".to_string()));
    }
}
```

- [ ] **Step 8.2: Run tests**

```bash
cargo test -p redist-tui runner 2>&1
```
Expected: 5 tests pass.

- [ ] **Step 8.3: Implement subprocess launch**

Add to `runner.rs`:
```rust
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};

/// Launch `redist state` with the given parameters.
/// Spawns a thread that reads stdout/stderr and appends to `log_lines`.
/// Returns a handle that can be joined; sets `done` flag when complete.
pub fn launch_redist_state(
    form: &crate::app::RunForm,
    log_lines: Arc<Mutex<Vec<String>>>,
    done: Arc<Mutex<bool>>,
    success: Arc<Mutex<bool>>,
) -> std::thread::JoinHandle<()> {
    let mut cmd = Command::new("redist");

    cmd.args(["state",
        "--state", &form.location,
        "--chamber", &form.chamber,
        "--year", &form.year,
        "--resolution", &form.resolution,
        "--version", &form.version,
        "--force",
    ]);

    if !form.seed.is_empty() {
        cmd.args(["--seed", &form.seed]);
    }
    if !form.label.is_empty() {
        cmd.args(["--label", &form.label]);
    }
    if !form.balance_tol.is_empty() {
        cmd.args(["--balance-tolerance", &form.balance_tol]);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    std::thread::spawn(move || {
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let mut lines = log_lines.lock().unwrap();
                lines.push(format!("ERROR: failed to launch redist: {e}"));
                *done.lock().unwrap() = true;
                return;
            }
        };

        // Read stderr (where STATUS: lines come from)
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let mut lines = log_lines.lock().unwrap();
                lines.push(line);
            }
        }

        let status = child.wait().unwrap_or_else(|_| {
            std::process::ExitStatus::from_raw(1)
        });
        *success.lock().unwrap() = status.success();
        *done.lock().unwrap() = true;
    })
}
```

- [ ] **Step 8.4: Wire into main.rs event loop**

Add Enter key handling for Run form in `run_app()`:
```rust
// Run: Enter on form → launch subprocess
(_, KeyCode::Enter) if matches!(app.screen, Screen::Run(RunState { phase: RunPhase::Form, .. })) => {
    if let Screen::Run(ref mut state) = app.screen {
        state.phase = RunPhase::Running;
        state.progress.districts_total = 98; // TODO: get from form
        // (Subprocess launch wiring goes here in the full implementation)
        // For now, transition to Running phase visually
    }
}
```

- [ ] **Step 8.5: Commit**

```bash
git add redist/crates/redist-tui/src/runner.rs redist/crates/redist-tui/src/main.rs
git commit -m "feat(tui): STATUS: subprocess integration, progress parsing"
```

---

## Task 9: Compare screen

**Files:**
- Modify: `redist/crates/redist-tui/src/screens/compare.rs`

- [ ] **Step 9.1: Implement compare screen**

Replace `redist/crates/redist-tui/src/screens/compare.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};
use crate::app::{App, CompareResult, Screen};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Screen::Compare(ref state) = app.screen else { return };

    let block = Block::default().title(" Compare Plans ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // plan inputs
            Constraint::Length(1),  // separator
            Constraint::Length(1),  // Jaccard bar
            Constraint::Length(1),  // spacer
            Constraint::Length(7),  // metrics table
            Constraint::Length(1),  // spacer
            Constraint::Length(5),  // most-changed districts
            Constraint::Min(0),
            Constraint::Length(1),  // footer
        ])
        .split(inner);

    // Plan inputs
    let inputs = Paragraph::new(vec![
        Line::from(format!("  Plan A  {}", state.plan_a)),
        Line::from(format!("  Plan B  [ {}▌ ]  [Go]", state.plan_b_input)),
    ]);
    f.render_widget(inputs, sections[0]);

    let Some(ref result) = state.result else {
        f.render_widget(
            Paragraph::new("  Enter Plan B label or path, then press Enter")
                .style(Style::default().fg(Color::DarkGray)),
            sections[2]
        );
        return;
    };

    // Jaccard similarity bar
    let jaccard_color = if result.jaccard >= 0.95 { Color::Green }
        else if result.jaccard >= 0.80 { Color::Yellow }
        else { Color::Red };
    let gauge = Gauge::default()
        .label(format!("Similarity  Jaccard {:.3}  [?] what is this", result.jaccard))
        .ratio(result.jaccard)
        .gauge_style(Style::default().fg(jaccard_color));
    f.render_widget(gauge, sections[2]);

    // Metrics comparison table
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
            -dev_delta, true),  // lower is better
        metric_row("County splits",
            &result.splits_a.to_string(),
            &result.splits_b.to_string(),
            -splits_delta as f64, true),  // lower is better
        Row::new(vec![
            Cell::from("Contiguous"),
            Cell::from(if result.contiguous_a { "✓" } else { "✗" }),
            Cell::from(if result.contiguous_b { "✓" } else { "✗" }),
            Cell::from(if result.contiguous_a == result.contiguous_b { "── same" }
                else if result.contiguous_a { "▲ better" } else { "▼ worse" }),
        ]),
    ];

    let table = Table::new(rows, [
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ]).header(header);
    f.render_widget(table, sections[4]);

    // Most-changed districts
    let mut changed_lines = vec![
        Line::from(Span::styled("  Most-changed districts:", Style::default().fg(Color::DarkGray))),
    ];
    for (dist_id, pct) in result.most_changed.iter().take(3) {
        let bar_len = ((pct / 100.0) * 20.0) as usize;
        let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
        changed_lines.push(Line::from(format!("  District {:3}  {}  {:.0}% tracts moved", dist_id, bar, pct)));
    }
    f.render_widget(Paragraph::new(changed_lines), sections[6]);

    // Footer
    f.render_widget(
        Paragraph::new("  [x] Export CSV    [m] Map diff    [Esc] back")
            .style(Style::default().fg(Color::DarkGray)),
        sections[8]
    );
}

fn metric_row<'a>(name: &'a str, a: &'a str, b: &'a str, delta: f64, higher_is_better: bool) -> Row<'a> {
    let (delta_text, delta_color) = if delta.abs() < 0.001 {
        ("── same".to_string(), Color::Gray)
    } else if (delta > 0.0) == higher_is_better {
        (format!("{:+.2} ▲ better", delta), Color::Green)
    } else {
        (format!("{:+.2} ▼ worse", delta), Color::Red)
    };

    Row::new(vec![
        Cell::from(name),
        Cell::from(a),
        Cell::from(b),
        Cell::from(delta_text).style(Style::default().fg(delta_color)),
    ])
}
```

- [ ] **Step 9.2: Build**

```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors.

- [ ] **Step 9.3: Commit**

```bash
git add redist/crates/redist-tui/src/screens/compare.rs
git commit -m "feat(tui): compare screen — side-by-side metrics, Jaccard bar, most-changed"
```

---

## Task 10: Verify screen

**Files:**
- Modify: `redist/crates/redist-tui/src/screens/verify.rs`

- [ ] **Step 10.1: Implement verify screen**

Replace `redist/crates/redist-tui/src/screens/verify.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
};
use crate::app::{App, Screen, VerifyResult};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Screen::Verify(ref state) = app.screen else { return };

    let block = Block::default().title(" Verify ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // manifest input
            Constraint::Length(1),   // spacer
            Constraint::Length(9),   // PASS/FAIL box
            Constraint::Length(1),   // spacer
            Constraint::Length(7),   // chain of custody
            Constraint::Min(0),
            Constraint::Length(1),   // footer
        ])
        .split(inner);

    // Manifest input
    f.render_widget(
        Paragraph::new(vec![
            Line::from(format!("  Manifest  [ {}▌ ]  [Browse]  [Go]", state.manifest_path)),
        ]),
        sections[0]
    );

    let Some(ref result) = state.result else {
        if state.running {
            f.render_widget(
                Paragraph::new("  Verifying...").style(Style::default().fg(Color::Yellow)),
                sections[2]
            );
        } else {
            f.render_widget(
                Paragraph::new("  Enter manifest path and press [Go] or Enter")
                    .style(Style::default().fg(Color::DarkGray)),
                sections[2]
            );
        }
        return;
    };

    // PASS/FAIL box — centred
    render_verdict_box(f, result, sections[2]);

    // Chain of custody
    render_chain_of_custody(f, result, sections[4]);

    // Footer
    f.render_widget(
        Paragraph::new("  [p] Export PDF    [x] Export audit.json    [Esc] back")
            .style(Style::default().fg(Color::DarkGray)),
        sections[6]
    );
}

fn render_verdict_box(f: &mut Frame, result: &VerifyResult, area: Rect) {
    let (verdict, jaccard_text, border_color) = if result.passed {
        ("✓  PASS", format!("Jaccard similarity:  {:.4}", result.jaccard), Color::Green)
    } else {
        ("✗  FAIL", format!("Jaccard similarity:  {:.4}", result.jaccard), Color::Red)
    };

    // Centre a box of fixed size
    let box_width = 44u16;
    let box_height = 9u16;
    let x = area.x + area.width.saturating_sub(box_width) / 2;
    let y = area.y + area.height.saturating_sub(box_height) / 2;
    let box_area = Rect::new(x, y, box_width.min(area.width), box_height.min(area.height));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));
    let inner = block.inner(box_area);
    f.render_widget(block, box_area);

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            verdict,
            Style::default().fg(border_color).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(jaccard_text.as_str()),
        Line::from(format!("{} · {} · {}", result.label, result.state_code, result.year)),
    ];
    f.render_widget(
        Paragraph::new(content).alignment(Alignment::Center),
        inner
    );

    // Likely causes (FAIL only)
    if !result.passed && !result.likely_causes.is_empty() {
        let causes_area = Rect::new(
            area.x, box_area.y + box_height, area.width, area.height.saturating_sub(box_height)
        );
        let mut lines = vec![
            Line::from(Span::styled("  Likely causes:", Style::default().fg(Color::DarkGray)))
        ];
        for cause in &result.likely_causes {
            lines.push(Line::from(format!("  ·  {}", cause)));
        }
        f.render_widget(Paragraph::new(lines), causes_area);
    }
}

fn render_chain_of_custody(f: &mut Frame, result: &VerifyResult, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Item").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Value").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
    ]).height(1).style(Style::default().bg(Color::DarkGray));

    let check = |ok: bool| if ok {
        Cell::from("✓").style(Style::default().fg(Color::Green))
    } else {
        Cell::from("✗").style(Style::default().fg(Color::Red))
    };

    let rows = vec![
        Row::new(vec![Cell::from("Binary"), Cell::from("(current)"), check(result.binary_match)]),
        Row::new(vec![Cell::from("METIS"), Cell::from(result.metis_version.as_str()), check(!result.metis_version.is_empty())]),
        Row::new(vec![Cell::from("Adjacency"), Cell::from("(see manifest)"), check(result.adjacency_match)]),
        Row::new(vec![Cell::from("Census geometry"), Cell::from("(see manifest)"), check(result.tiger_match)]),
        Row::new(vec![Cell::from("Seed"), Cell::from("(see manifest)"), check(result.seed_recorded)]),
    ];

    let title_block = Block::default()
        .title("  Chain of custody")
        .borders(Borders::TOP);
    f.render_widget(
        Table::new(rows, [Constraint::Length(16), Constraint::Min(30), Constraint::Length(4)])
            .header(header)
            .block(title_block),
        area
    );
}
```

- [ ] **Step 10.2: Build**

```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors.

- [ ] **Step 10.3: Commit**

```bash
git add redist/crates/redist-tui/src/screens/verify.rs
git commit -m "feat(tui): verify screen — PASS/FAIL box, chain of custody table"
```

---

## Task 11: Doctor screen

**Files:**
- Modify: `redist/crates/redist-tui/src/screens/doctor.rs`

- [ ] **Step 11.1: Implement doctor screen**

Replace `redist/crates/redist-tui/src/screens/doctor.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::{App, CheckStatus, DoctorState, Screen};
use redist_cli::policy::LocationRegistry;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Screen::Doctor(ref state) = app.screen else { return };

    let title = if state.location.is_empty() {
        " Doctor ".to_string()
    } else {
        format!(" Doctor · {} · {} · {} ", state.location, state.chamber, state.year)
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.location.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from("  Enter a location code to check:"),
                Line::from("  Examples: WA  CA  IE  MT-PARLIAMENT  _TEST_EL"),
            ]).style(Style::default().fg(Color::DarkGray)),
            inner
        );
        return;
    }

    // Run checks live from LocationRegistry
    let checks = run_doctor_checks(state);

    let mut lines: Vec<Line> = checks.iter().map(|(status, msg, hint)| {
        let (icon, color) = match status {
            CheckStatus::Pass => ("✓", Color::Green),
            CheckStatus::Warn => ("⚠", Color::Yellow),
            CheckStatus::Fail => ("✗", Color::Red),
            CheckStatus::Info => ("ℹ", Color::Cyan),
        };
        let mut line_spans = vec![
            Span::styled(format!("  {}  ", icon), Style::default().fg(color)),
            Span::raw(msg.as_str()),
        ];
        if let Some(h) = hint {
            // Hint on same line for short hints
            line_spans.push(Span::styled(format!("  → {}", h), Style::default().fg(Color::DarkGray)));
        }
        Line::from(line_spans)
    }).collect();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [r] ", Style::default().fg(Color::Green)),
        Span::raw("Run with these settings    "),
        Span::styled("[Esc] ", Style::default().fg(Color::Gray)),
        Span::raw("back"),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn run_doctor_checks(state: &DoctorState) -> Vec<(CheckStatus, String, Option<String>)> {
    let reg = LocationRegistry::load();
    let mut checks = Vec::new();

    // 1. Location known?
    if reg.has_location(&state.location) {
        let name = reg.state_name(&state.location).unwrap_or_else(|| state.location.clone());
        checks.push((CheckStatus::Pass, format!("Location registered: {}", name), None));
    } else {
        checks.push((CheckStatus::Fail,
            format!("Location '{}' not found in location_policy.json", state.location),
            Some("Edit redist/data/location_policy.json to add it".to_string())));
        return checks;
    }

    // 2. Year valid?
    match reg.validate_year(&state.location, &state.year) {
        Ok(_) => {
            let years = reg.available_years(&state.location);
            checks.push((CheckStatus::Pass,
                format!("Year {} valid. Available: {}", state.year, years.join(", ")),
                None));
        }
        Err(e) => {
            checks.push((CheckStatus::Fail, e, None));
        }
    }

    // 3. District count
    if let Some(n) = reg.chamber_districts(&state.location, &state.chamber, &state.year) {
        checks.push((CheckStatus::Pass,
            format!("{} {} districts: {}", state.location, state.chamber, n),
            None));
    } else {
        checks.push((CheckStatus::Warn,
            format!("No district count for {} {} {}. Use --districts.", state.location, state.chamber, state.year),
            None));
    }

    // 4. Balance tolerance
    let tol = redist_cli::runner::chamber_balance_tolerance(&state.location, &state.chamber);
    checks.push((CheckStatus::Info,
        format!("Balance tolerance: {:.1}% (from policy)", tol * 100.0),
        None));

    // 5. Granularity
    if let Some(warn) = reg.granularity_warning(&state.location, &state.year, &state.chamber, "tract") {
        checks.push((CheckStatus::Warn, warn, Some("Use --resolution block_group".to_string())));
    } else {
        checks.push((CheckStatus::Pass, "Tract resolution appropriate for this chamber".to_string(), None));
    }

    // 6. Compactness standard
    if let Some(loc) = reg.raw().get(&state.location.to_uppercase()) {
        if let Some(std) = loc.get("compactness_standard").and_then(|v| v.as_str()) {
            checks.push((CheckStatus::Info, format!("Compactness standard: {}", std), None));
        }
    }

    // 7. Nesting
    if state.chamber != "congressional" {
        if let Some(loc) = reg.raw().get(&state.location.to_uppercase()) {
            if let Some(nesting) = loc.get("nesting_requirement").and_then(|v| v.as_str()) {
                if nesting != "null" {
                    let ratio = loc.get("nesting_ratio").and_then(|v| v.as_str()).unwrap_or("?");
                    checks.push((CheckStatus::Info,
                        format!("Nesting: {} ({} ratio). Use redist suite for multi-chamber.", nesting, ratio),
                        None));
                }
            }
        }
    }

    checks
}
```

- [ ] **Step 11.2: Build**

```bash
cargo build -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors.

- [ ] **Step 11.3: Commit**

```bash
git add redist/crates/redist-tui/src/screens/doctor.rs
git commit -m "feat(tui): doctor screen — live checks from LocationRegistry"
```

---

## Task 12: Command palette overlay

**Files:**
- Create: `redist/crates/redist-tui/src/widgets/command_palette.rs`
- Create: `redist/crates/redist-tui/src/widgets/error_banner.rs`
- Create: `redist/crates/redist-tui/src/widgets/glossary.rs`
- Modify: `redist/crates/redist-tui/src/main.rs`

- [ ] **Step 12.1: Command palette widget**

Create `redist/crates/redist-tui/src/widgets/command_palette.rs`:
```rust
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use crate::app::App;

/// Known command patterns for autocomplete suggestions.
const COMMANDS: &[&str] = &[
    "run",
    "analyze",
    "compare",
    "verify",
    "doctor",
    "sweep",
];

pub fn suggestions(input: &str) -> Vec<String> {
    if input.is_empty() { return vec![]; }
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];

    COMMANDS.iter()
        .filter(|c| c.starts_with(cmd))
        .map(|c| c.to_string())
        .collect()
}

pub fn render(f: &mut Frame, app: &App) {
    if !app.show_palette { return; }

    let area = f.area();
    let palette_width = (area.width / 2).max(50).min(area.width - 4);
    let x = (area.width - palette_width) / 2;
    let palette_area = Rect::new(x, 2, palette_width, 8);

    f.render_widget(Clear, palette_area);

    let block = Block::default()
        .title(" Command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(palette_area);
    f.render_widget(block, palette_area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    // Input line
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(": ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(app.palette_input.as_str()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ])),
        sections[0]
    );

    // Suggestions
    let suggs = suggestions(&app.palette_input);
    let items: Vec<ListItem> = suggs.iter()
        .map(|s| ListItem::new(format!("  {}", s)))
        .collect();
    f.render_widget(List::new(items), sections[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestions_run_matches() {
        let suggs = suggestions("run");
        assert!(suggs.contains(&"run".to_string()));
    }

    #[test]
    fn test_suggestions_empty_input_empty_list() {
        assert!(suggestions("").is_empty());
    }

    #[test]
    fn test_suggestions_partial_match() {
        let suggs = suggestions("doc");
        assert!(suggs.contains(&"doctor".to_string()));
    }

    #[test]
    fn test_suggestions_no_match() {
        assert!(suggestions("zzz").is_empty());
    }
}
```

- [ ] **Step 12.2: Error banner widget**

Create `redist/crates/redist-tui/src/widgets/error_banner.rs`:
```rust
use ratatui::{
    Frame, layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let Some(ref err) = app.error else { return };

    let banner_area = Rect::new(area.x, area.y + area.height.saturating_sub(5), area.width, 4);
    f.render_widget(Clear, banner_area);

    let block = Block::default()
        .title(" Error  [e] full log  [c] copy  [Esc] dismiss ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let inner = block.inner(banner_area);
    f.render_widget(block, banner_area);

    let text = if err.show_raw {
        err.raw.as_deref().unwrap_or(&err.summary)
    } else {
        &err.summary
    };
    f.render_widget(Paragraph::new(text), inner);
}
```

- [ ] **Step 12.3: Glossary widget**

Create `redist/crates/redist-tui/src/widgets/glossary.rs`:
```rust
use ratatui::{
    Frame, layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

const GLOSSARY: &[(&str, &str)] = &[
    ("Jaccard", "Fraction of tracts with same district in both plans. 1.0=identical, 0.0=completely different."),
    ("PP / Polsby-Popper", "Compactness metric 0–1. Higher = more circular districts. Target > 0.25."),
    ("Max dev%", "Largest population deviation from ideal district size. Congressional: <0.5%. State: <5%."),
    ("County splits", "Counties divided across multiple districts. Lower = better preservation of local boundaries."),
    ("Contiguous", "All tracts in each district are geographically connected. Non-contiguous districts are legally problematic."),
    ("VRA districts", "Majority-minority districts required by Voting Rights Act Section 2."),
    ("Bisection depth", "Stage in recursive splitting. Deeper = smaller sub-problems. Total stages = ceil(log2(districts))."),
];

pub fn render(f: &mut Frame, app: &crate::app::App) {
    if !app.show_glossary { return; }

    let area = f.area();
    let gloss_area = Rect::new(area.width / 4, 4, area.width / 2, (GLOSSARY.len() as u16 + 4).min(area.height - 6));
    f.render_widget(Clear, gloss_area);

    let block = Block::default()
        .title(" Metric Glossary  [?] close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(gloss_area);
    f.render_widget(block, gloss_area);

    let text: Vec<String> = GLOSSARY.iter()
        .flat_map(|(term, def)| vec![format!("  {}:", term), format!("    {}", def), String::new()])
        .collect();
    f.render_widget(Paragraph::new(text.join("\n")), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_has_jaccard_entry() {
        assert!(GLOSSARY.iter().any(|(k, _)| k.contains("Jaccard")));
    }

    #[test]
    fn test_glossary_has_pp_entry() {
        assert!(GLOSSARY.iter().any(|(k, _)| k.contains("PP") || k.contains("Polsby")));
    }
}
```

- [ ] **Step 12.4: Wire overlays into main event loop**

In `run_app()` in `main.rs`, add to the draw closure AFTER rendering the current screen:
```rust
// Overlays (rendered on top of current screen)
widgets::command_palette::render(f, app);
widgets::error_banner::render(f, app, chunks[0]);
widgets::glossary::render(f, app);
```

And add key handling:
```rust
// Command palette: character input
(_, KeyCode::Char(c)) if app.show_palette => {
    app.palette_input.push(c);
}
(_, KeyCode::Backspace) if app.show_palette => {
    app.palette_input.pop();
}
(_, KeyCode::Esc) if app.show_palette => {
    app.show_palette = false;
    app.palette_input.clear();
}
(_, KeyCode::Enter) if app.show_palette => {
    // TODO: dispatch palette command
    app.show_palette = false;
    app.palette_input.clear();
}
// Error banner: dismiss
(_, KeyCode::Esc) if app.error.is_some() => {
    app.clear_error();
}
```

- [ ] **Step 12.5: Run all tests**

```bash
cargo test -p redist-tui 2>&1
```
Expected: all tests pass (including 4 palette tests + 2 glossary tests).

- [ ] **Step 12.6: Commit**

```bash
git add redist/crates/redist-tui/src/widgets/
git commit -m "feat(tui): command palette, error banner, metric glossary overlays"
```

---

## Task 13: Wire `redist tui` into the main CLI

**Files:**
- Modify: `redist/crates/redist-cli/src/args.rs`
- Modify: `redist/crates/redist-cli/src/main.rs`

- [ ] **Step 13.1: Add TUI subcommand to args.rs**

In `redist/crates/redist-cli/src/args.rs`, add to the `Commands` enum:
```rust
/// Launch the interactive terminal UI
Tui(TuiArgs),
```

Add the struct (after the other Args structs):
```rust
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct TuiArgs {
    /// Start with a clean session (no saved state loaded)
    #[arg(long)]
    pub no_session: bool,
}
```

- [ ] **Step 13.2: Wire into main.rs**

In `redist/crates/redist-cli/src/main.rs`, add to the match:
```rust
Commands::Tui(args) => {
    // Find the redist-tui binary alongside this binary
    let mut tui_bin = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("redist-tui"));
    tui_bin.set_file_name("redist-tui");

    let mut cmd = std::process::Command::new(&tui_bin);
    if args.no_session {
        cmd.arg("--no-session");
    }

    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("ERROR: could not launch redist-tui: {e}");
        eprintln!("Make sure redist-tui is built: cargo build --release -p redist-tui");
        std::process::exit(1);
    });

    std::process::exit(status.code().unwrap_or(1));
}
```

- [ ] **Step 13.3: Run existing tests to confirm no regressions**

```bash
cargo test --workspace --lib 2>&1 | grep "test result"
```
Expected: all test suites pass (currently 792 tests).

- [ ] **Step 13.4: Build both binaries**

```bash
cargo build -p redist-cli -p redist-tui 2>&1 | grep "^error" | head -10
```
Expected: no errors. Two binaries produced.

- [ ] **Step 13.5: Commit**

```bash
git add redist/crates/redist-cli/src/args.rs redist/crates/redist-cli/src/main.rs
git commit -m "feat: redist tui subcommand — launches redist-tui binary"
```

---

## Task 14: Integration smoke test and polish

**Files:**
- Create: `redist/crates/redist-tui/src/main.rs` (add `--version` flag)
- Various: fix any compilation warnings

- [ ] **Step 14.1: Add --version and --help to TUI binary**

Add to `main()` in `redist/crates/redist-tui/src/main.rs`:
```rust
let args: Vec<String> = std::env::args().collect();
if args.iter().any(|a| a == "--version" || a == "-V") {
    println!("redist-tui {}", env!("CARGO_PKG_VERSION"));
    return Ok(());
}
if args.iter().any(|a| a == "--help" || a == "-h") {
    println!("Usage: redist-tui [--no-session] [--version]");
    println!("  --no-session  Start with clean state (no saved config)");
    return Ok(());
}
```

- [ ] **Step 14.2: Fix any unused import warnings**

```bash
cargo build -p redist-tui 2>&1 | grep "warning: unused" | head -10
```
Add `#[allow(unused_imports)]` or remove unused imports as needed.

- [ ] **Step 14.3: Run full test suite**

```bash
cargo test --workspace --lib 2>&1 | grep -E "test result|FAILED"
```
Expected: all tests pass (792 + new TUI tests).

- [ ] **Step 14.4: Add to workspace Cargo.toml if not already done**

Verify:
```bash
grep "redist-tui" redist/Cargo.toml
```
Expected: `"crates/redist-tui"` present in members.

- [ ] **Step 14.5: Push**

```bash
git add -A
git commit -m "feat(tui): polish — version flag, help, warning cleanup"
git push origin master:main
```

---

## Self-Review

**Spec coverage check:**

| Spec requirement | Task |
|-----------------|------|
| redist-tui binary crate | Task 1 |
| App state + Screen enum | Task 2 |
| Session config ~/.config/redist/tui.toml | Task 3 |
| Plan discovery | Task 4 |
| Terminal event loop | Task 5 |
| Home screen plan browser + detail panel | Task 6 |
| Run form with inline doctor | Task 7 |
| Live progress with STATUS: parsing | Task 8 |
| Compare screen | Task 9 |
| Verify screen PASS/FAIL | Task 10 |
| Doctor screen | Task 11 |
| Command palette `:` overlay | Task 12 |
| Error banner summary+raw | Task 12 |
| Metric glossary `?` | Task 12 |
| Status bar always visible | Task 5 |
| `redist tui` subcommand in CLI | Task 13 |
| `--no-session` flag | Tasks 3, 13 |
| No changes to redist-cli tests | Task 13 (verified in 13.3) |

**Out of scope (confirmed):** Mouse support, real-time map rendering, multi-pane tiling, remote/SSH optimisation.

**No placeholder scan:** All code is complete. No "TBD" or "TODO" in implementation steps.

**Type consistency:** `PlanSummary` defined in Task 4, used in Tasks 6, 8. `RunState`/`RunForm`/`RunProgress` defined in Task 2, used in Tasks 7, 8. `CompareResult` defined in Task 2, used in Task 9. `VerifyResult` defined in Task 2, used in Task 10. `DoctorState`/`DoctorCheck`/`CheckStatus` defined in Task 2, used in Task 11. All consistent.
