# PlanContext Refactor Spec
**Date:** 2026-04-28  
**Status:** Approved for implementation planning  
**Depends on:** 2026-04-28-redist-cli-architecture.md

---

## Goal

Introduce `PlanContext` as the single source of truth for per-plan metadata in all Class B commands (analyze, report, map, export, compare, verify). Eliminate the 4-way metadata resolution divergence that caused the `num_districts` bug class.

**Success criterion:** After this refactor, no Class B command ever calls `load_all_states()`, `chamber_district_count()`, or `LocationRegistry` to resolve metadata about an existing plan.

---

## Scope

### In scope
- New `PlanContext` struct in `redist-cli/src/plan_context.rs`
- Update `analyze.rs` to use `PlanContext` (highest priority — just fixed the bug here)
- Update `report_cmd.rs` to use `PlanContext` (already has `ReportContext` in redist-report — consolidate)
- Update `compare.rs` to use `PlanContext` for plan metadata
- Update `verify.rs` to use `PlanContext`
- Update `map_cmd.rs` to use `PlanContext` (currently hardcodes "V3" paths)
- 3 pipeline integration tests

### Out of scope
- `StateConfig` field count reduction (separate refactor, lower priority)
- Output path centralization (follow-on task)
- `redist states` / `redist run` bulk commands (Class A, not affected)

---

## The New Type

### `PlanContext` in `redist-cli/src/plan_context.rs`

```rust
use std::path::{Path, PathBuf};
use redist_report::PlanManifest;

/// Single source of truth for all metadata about an existing redistricting plan.
/// Loaded from the plan's manifest.json — never from global registry or manifest.
pub struct PlanContext {
    pub plan_dir: PathBuf,
    pub manifest: PlanManifest,
}

impl PlanContext {
    /// Load a plan by label from the standard output tree.
    /// Returns Err with actionable message if plan directory or manifest missing.
    pub fn from_label(
        output_base: &Path,
        version: &str,
        year: &str,
        label: &str,
    ) -> anyhow::Result<Self> {
        let plan_dir = output_base
            .join(version)
            .join(year)
            .join("plans")
            .join(label);

        if !plan_dir.exists() {
            // List available plans for helpful error
            let plans_dir = output_base.join(version).join(year).join("plans");
            let available = if plans_dir.exists() {
                std::fs::read_dir(&plans_dir)
                    .ok()
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                            .take(10)
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            };

            let hint = if available.is_empty() {
                "Run 'redist state --label ...' to create a plan first.".to_string()
            } else {
                format!("Available plans: {}", available.join(", "))
            };

            anyhow::bail!(
                "Plan '{}' not found at {}\n{}",
                label,
                plan_dir.display(),
                hint
            );
        }

        let manifest_path = plan_dir.join("manifest.json");
        let manifest: PlanManifest = if manifest_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?
        } else {
            anyhow::bail!(
                "Plan '{}' exists but has no manifest.json — the plan may be corrupt.\n\
                 Delete the directory and re-run: redist state --label {}",
                label,
                label
            );
        };

        Ok(Self { plan_dir, manifest })
    }

    // ── Metadata accessors (all from manifest) ────────────────────────────────

    pub fn num_districts(&self) -> usize {
        self.manifest.num_districts
    }

    pub fn chamber(&self) -> &str {
        &self.manifest.chamber
    }

    pub fn state_code(&self) -> &str {
        &self.manifest.state_code
    }

    pub fn year(&self) -> &str {
        &self.manifest.year
    }

    pub fn label(&self) -> &str {
        &self.manifest.label
    }

    pub fn balance_tolerance_frac(&self) -> f64 {
        self.manifest.balance_tolerance_pct / 100.0
    }

    pub fn seats_per_district(&self) -> usize {
        self.manifest.seats_per_district.max(1)
    }

    // ── Derived paths ─────────────────────────────────────────────────────────

    pub fn analysis_dir(&self) -> PathBuf {
        self.plan_dir.join("analysis")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.plan_dir.join("data")
    }

    pub fn maps_dir(&self) -> PathBuf {
        self.plan_dir.join("maps")
    }

    pub fn assignments_path(&self) -> PathBuf {
        self.data_dir().join("final_assignments.json")
    }

    pub fn analysis_file(&self, name: &str) -> PathBuf {
        self.analysis_dir().join(name)
    }
}
```

---

## Migration: What Changes in Each File

### `analyze.rs` (highest priority — just hit this bug)

**Before:**
```rust
// Lines 97-130: three-way resolution for num_districts
let (state_name, num_districts) = {
    let all = load_all_states(&year).unwrap_or_default();
    if let Some((_, name, nd)) = all.iter().find(...)  { ... }  // congressional count!
    else { // manifest fallback }
};
```

**After:**
```rust
// Two lines replace the entire block
let ctx = PlanContext::from_label(&output_root, version, &year, label.as_deref().unwrap_or(""))?;
let (state_name, num_districts) = (ctx.state_code().to_lowercase(), ctx.num_districts());
```

Remove: `use crate::runner::load_all_states;` from analyze.rs imports.

### `report_cmd.rs`

**Before:** builds `ReportContext` from `PlanManifest` (already does this right)

**After:** replace with `PlanContext::from_label()` and pass to `assemble_report()`. The `ReportContext` type in `redist-report` should accept a `PlanContext` or be consolidated with it.

Note: `redist-report::ReportContext` already does what `PlanContext` does — the two should be merged or `PlanContext` should delegate to `ReportContext`. Simplest: make `PlanContext` wrap `ReportContext` or vice versa.

### `compare.rs`

**Before:** `load_plan_assignments()` reconstructs plan paths from label + version + year args.

**After:** when `--plan-a` is a label (not a file path), create `PlanContext` and use `ctx.assignments_path()`. The year/version fallback chain in `load_plan_assignments` can be simplified.

### `verify.rs`

**Before:** reads manifest manually at lines 70-90.

**After:** `PlanContext::from_label()` handles manifest loading and provides `balance_tolerance_frac()`, `num_districts()` etc.

Add: `ctx.plan_dir` existence check replaces the manual `output_base_path.exists()` check.

### `map_cmd.rs`

**Before:** hardcodes "V3" and "V4" at lines 60-90 for geometry path resolution.

**After:** geometry path comes from `PlanContext` (or `LocationRegistry.adjacency_path()` for adjacency). Remove hardcoded "V3"/"V4" strings.

---

## Integration Tests to Add

### Test 1: `PlanContext` resolves from manifest, not global

```rust
// In plan_context.rs tests:
#[test]
fn test_plan_context_reads_num_districts_from_manifest_not_registry() {
    let tmp = TempDir::new().unwrap();
    // WA house plan: 98 districts (NOT 10 congressional)
    let manifest = PlanManifest {
        label: "wa_house_test".into(),
        state_code: "WA".into(),
        chamber: "house".into(),
        year: "2020".into(),
        num_districts: 98,
        ..Default::default()
    };
    let plan_dir = tmp.path().join("v1").join("2020").join("plans").join("wa_house_test");
    std::fs::create_dir_all(&plan_dir).unwrap();
    std::fs::write(plan_dir.join("manifest.json"),
        serde_json::to_string(&manifest).unwrap()).unwrap();

    let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "wa_house_test").unwrap();
    assert_eq!(ctx.num_districts(), 98);  // Must be 98, NOT 10
    assert_eq!(ctx.chamber(), "house");
}
```

### Test 2: `analyze` uses PlanContext num_districts

```rust
// In analyze.rs tests:
#[test]
fn test_analyze_summary_has_correct_district_count_for_house_plan() {
    // Create a plan with 3 districts (not congressional)
    // Run the summary analysis on it
    // Assert summary.json has 3 districts, not some other count
}
```

### Test 3: Pipeline — `state` manifest feeds `analyze` correctly

```rust
// In integration_tests.rs (new file):
#[test]
fn test_plan_context_feeds_correctly_to_analyze() {
    // 1. Create a fake plan with manifest num_districts=5, chamber="house"
    // 2. Create fake final_assignments.json with 5 districts
    // 3. Run the analysis context loading
    // 4. Assert num_districts=5 is used for summary computation
    // 5. Assert balance is computed against 5 districts, not congressional count
}
```

---

## What This Enables

After this refactor:

1. **Adding a new Class B command** is simple: `let ctx = PlanContext::from_label(...)?;` — done. No resolution logic to write.

2. **International state analysis works** without any special-casing: the manifest records `state_code: "IE"`, `num_districts: 43` — PlanContext reads those directly.

3. **The bug class is impossible**: there's no `load_all_states()` in Class B commands, so congressional counts can never leak into legislative analysis.

4. **`--state` and `--year` become optional** on Class B commands when `--label` is provided: the label alone is sufficient to locate the plan and read its metadata.

---

## Migration Safety

**Phase 1:** Add `PlanContext` struct (no existing code changes). Add tests. Green.

**Phase 2:** Update `analyze.rs` first (most critical, already partially fixed). Tests must pass.

**Phase 3:** Update `report_cmd.rs`, `compare.rs`, `verify.rs`, `map_cmd.rs` one at a time.

**Phase 4:** Remove dead `load_all_states()` calls from Class B commands. Remove duplicate resolution logic.

Each phase is independently mergeable and testable.

---

## Non-goals of This Refactor

- **Not** changing `StateConfig` fields (separate refactor)
- **Not** centralizing output path constants (follow-on)
- **Not** changing the manifest format or PlanManifest struct
- **Not** changing how Class A commands (state, states, run) work
