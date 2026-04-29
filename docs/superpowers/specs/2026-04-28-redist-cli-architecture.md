# redist CLI Architecture Spec
**Date:** 2026-04-28  
**Status:** Approved — basis for plan-context-refactor

---

## Problem This Spec Solves

The CLI has accumulated multiple overlapping sources of truth for plan metadata (num_districts, chamber, balance_tolerance, state_name). The `state → analyze → report` pipeline broke silently because `analyze` resolved `num_districts` from the global congressional manifest (10 for WA) instead of the plan's manifest (98 for WA house). This class of bug — a command resolving metadata from the wrong source — will recur without a canonical architecture.

---

## Core Invariant

**The plan manifest is the single source of truth for all per-plan metadata.**

Any command that operates on an existing plan (analyze, report, compare, verify, map, export, migrate) MUST read its metadata from `{plan_dir}/manifest.json`. Global lookups (LocationRegistry, load_all_states, args) are only valid when creating a plan for the first time, or in bulk operations that don't target a specific plan.

---

## The Two Contexts

### 1. `PlanContext` — for commands that operate on existing plans

Used by: `analyze`, `report`, `compare`, `verify`, `map`, `export`, `migrate`, `doctor` (when checking an existing plan)

```rust
pub struct PlanContext {
    pub plan_dir: PathBuf,
    pub manifest: PlanManifest,  // loaded from plan_dir/manifest.json
}

impl PlanContext {
    /// Load from a labeled plan in the standard output tree.
    pub fn from_label(output_base: &Path, version: &str, year: &str, label: &str) -> Result<Self>

    /// Convenience: all metadata comes from the manifest, never from global tables.
    pub fn num_districts(&self) -> usize { self.manifest.num_districts }
    pub fn chamber(&self) -> &str { &self.manifest.chamber }
    pub fn state_code(&self) -> &str { &self.manifest.state_code }
    pub fn year(&self) -> &str { &self.manifest.year }
    pub fn balance_tolerance_frac(&self) -> f64 { self.manifest.balance_tolerance_pct / 100.0 }

    /// Derived paths — always relative to plan_dir.
    pub fn analysis_dir(&self) -> PathBuf { self.plan_dir.join("analysis") }
    pub fn data_dir(&self) -> PathBuf { self.plan_dir.join("data") }
    pub fn maps_dir(&self) -> PathBuf { self.plan_dir.join("maps") }
    pub fn assignments_path(&self) -> PathBuf { self.data_dir().join("final_assignments.json") }
}
```

**What PlanContext does NOT do:**
- Does not call `load_all_states()`
- Does not call `LocationRegistry`
- Does not consult CLI args
- Does not know about versions, output_base, or other plans

### 2. `RunContext` — for commands that create a new plan

Used by: `state`, `states`, `run`, `suite draw`

`RunContext` is resolved from CLI args + LocationRegistry + manifest fallback (for international states). It produces a `PlanManifest` that gets written as `manifest.json`. After writing, the plan can be operated on via `PlanContext`.

```rust
// NOT a new struct — this is what StateConfig already does.
// The key change: StateConfig stays as-is for RUN operations;
// PlanContext is added for POST-RUN operations.
```

---

## Source of Truth Table

| Metadata | When creating a plan | When operating on a plan |
|----------|---------------------|--------------------------|
| `num_districts` | LocationRegistry → load_all_states → --districts | `PlanContext.manifest.num_districts` |
| `chamber` | CLI `--chamber` arg | `PlanContext.manifest.chamber` |
| `year` | CLI `--year` arg (validated by registry) | `PlanContext.manifest.year` |
| `state_code` | CLI `--state` arg | `PlanContext.manifest.state_code` |
| `balance_tolerance` | LocationRegistry → CLI override | `PlanContext.manifest.balance_tolerance_pct` |
| `state_name` | Registry.state_name() → lowercase code | `PlanContext.manifest.state_code.to_lowercase()` |
| `version` | CLI `--version` arg | CLI arg (not in manifest — plans don't own their version) |
| `label` | CLI `--label` or default_label() | CLI `--label` arg (plan dir IS the label) |

**The boundary:** Once a plan exists on disk, its manifest is authoritative. CLI args for `--year`, `--state`, `--chamber` on `analyze`/`report` are only for locating the plan, not for overriding its metadata.

---

## Output Path Conventions

All output paths are relative to `{output_base}/{version}/`:

```
{output_base}/{version}/
├── {year}/
│   ├── plans/
│   │   └── {label}/          ← PlanContext.plan_dir
│   │       ├── manifest.json
│   │       ├── data/
│   │       │   └── final_assignments.json
│   │       ├── analysis/
│   │       │   ├── summary.json
│   │       │   ├── compactness.json
│   │       │   ├── contiguity.json
│   │       │   ├── splits.json
│   │       │   ├── demographic.json
│   │       │   ├── political.json
│   │       │   └── partisan.json
│   │       ├── maps/
│   │       │   ├── districts.png
│   │       │   └── compactness.png
│   │       └── intermediate/
│   └── states/                ← LEGACY only (migrate target)
│       └── {state_name}/data/
└── data/
    └── {year}/
        └── adjacency/         ← NOT inside a plan
            └── {state}_adjacency_{year}.pkl
```

**Invariants:**
- `{year}` in the path comes from the plan's manifest, not from CLI args at analysis time
- `{label}` is the only identifier needed to find a plan given `output_base` + `version`
- Legacy `states/` path is read-only (migration source); new plans always go under `plans/`
- "V3" and "V4" adjacency variants are managed by `LocationRegistry.adjacency_path()` only — no other code hardcodes "V3"

---

## Command Classification

### Class A: Plan-creating commands
Resolve metadata from args + registry. Write manifest.json.
- `redist state`, `redist states`, `redist run`, `redist suite draw`, `redist import`, `redist migrate`

**Obligation:** Write a complete, correct `manifest.json` before writing any other outputs.

### Class B: Plan-operating commands
Load `PlanContext` from manifest. Never re-resolve metadata from global sources.
- `redist analyze`, `redist report`, `redist map`, `redist export`, `redist compare`, `redist verify`

**Obligation:** Use `PlanContext.from_label()` at entry. Fail with clear error if manifest missing.

### Class C: Metadata-only commands
Don't operate on plans. Read from registry or global sources.
- `redist doctor`, `redist policy`, `redist fetch`, `redist validate`, `redist tui`

---

## StateConfig Scope (What It Is and Isn't)

`StateConfig` is the "run parameters" struct — it carries everything needed to EXECUTE a redistricting run. It is NOT a plan description struct.

**Stays in StateConfig** (algorithm + control):
```
state_code, state_name, num_districts, year, version, output_dir
partition_mode, ufactor, niter, seed, debug, reset, reprocess, force
num_districts_override, chamber, label, population_source, balance_tolerance
write_manifest, resolution, seats_per_district, total_seats
adjacency_override, coi_weights
```

**Does NOT belong in StateConfig** (should be computed from PlanContext after run):
- Any analysis results
- Any path to output files (derive from output_dir + label)

The 28-field size of StateConfig reflects the complexity of the redistricting run parameters — this is acceptable. What's NOT acceptable is using StateConfig as a source of truth for commands that operate on already-run plans.

---

## Pipeline Integration Test Requirement

Every Class B command MUST have at least one integration test that:
1. Creates a minimal plan directory with a realistic manifest.json
2. Runs the command against it
3. Asserts correct output given the manifest metadata

The `num_districts` bug was: `analyze` read 10 (from global manifest) instead of 98 (from plan manifest). The test would be:

```rust
#[test]
fn test_analyze_reads_num_districts_from_plan_manifest() {
    let tmp = TempDir::new();
    // Create a plan with num_districts=98, chamber="house"
    write_manifest(tmp, "wa_house_test", 98, "house", "WA", "2020");
    write_fake_analysis_files(tmp);
    
    let ctx = PlanContext::from_label(tmp.path(), "v1", "2020", "wa_house_test").unwrap();
    assert_eq!(ctx.num_districts(), 98);  // NOT 10 (congressional)
}
```

---

## What This Spec Does NOT Address

- The internal implementation of each analyzer (compactness, contiguity, etc.)
- The redist-tui screen layouts (covered in 2026-04-27-redist-tui-design.md)
- The RPLAN format (covered in redist-report crate)
- The adjacency binary format (covered in redist-data crate)
