# Spec 7: Run Manifest — Declarative Run Configuration

**Date**: 2026-05-04
**Status**: Draft
**Dependencies**: Specs 1–6 (PlanManifest, three-layer compositor)
**Depended on by**: Future dashboard automation, CI/CD integration

---

## 1. Problem Statement

### 1.1 Output path inconsistency

`redist state` and `redist states` produce outputs under incompatible directory structures:

```
# redist state --state VT --year 2020 --version v1
→ outputs/v1/2020/states/vermont/

# redist states --year 2020 --output-dir outputs/my_run/2020
→ outputs/my_run/2020/states/vermont/
```

`redist state` computes the path from `--version` using the rule
`outputs/{version}/{year}/states/{state_name}/`. `redist states` requires a raw
`--output-dir` string and does NOT embed version in any structured way. The two
commands are therefore path-incompatible: outputs from `states` cannot be found
by `analyze`, `map`, `report`, or `aggregate` unless the user manually mirrors
the directory layout that `state` would have produced.

Practical consequence: a three-year official run (`redist states` × 3) produces
three trees with bespoke paths. The dashboard generator hardcodes assumptions
about where files live. If the operator changes the base directory between years,
the dashboard breaks silently.

### 1.2 No declarative run configuration

The three-layer algorithm compositor (Spec 1 extension, B.7–B.16) exposes roughly
twenty flags that must be passed identically across `state`, `states`, and `run`.
Reproducing the official federal proposal requires:

```bash
redist states \
  --year 2020 --version official_proposal \
  --output-dir outputs/official_proposal/2020 \
  --partition-mode apportion-regions \
  --weights-override county --alpha-county 2.0 \
  --search convergence --convergence-threshold 600 \
  --workers 6
```

This command must be run three times (2020, 2010, 2000). There is no checked-in
artifact that records these flags. A operator reproducing the run must know to
use identical flags for all three years. A flag mismatch — e.g., using the wrong
`--convergence-threshold` for one year — produces outputs that look structurally
correct but differ silently from the other years.

### 1.3 Dashboard generation is hardcoded

`generate_master_dashboard.py` and `deploy_docs.py` take explicit `--version` and
`--year` arguments. They have no way to discover what algorithm produced a given
output tree, what artifact types were requested, or whether national-summary
files should exist alongside per-state outputs. Every dashboard generation is
operator-scripted and undocumented.

### 1.4 Outputs are algorithmically opaque

Given a directory `outputs/official_proposal/2020/states/vermont/`, there is no
machine-readable record of which partition mode, alpha-county value, or
convergence threshold produced those outputs. The `manifest.json` written by
`redist state` records provenance for a single state plan, but does not record
run-level configuration (which states were included, which years were run, which
artifact types were generated). Auditors inspecting outputs must reconstruct this
from shell history or memory.

### 1.5 Run reproducibility is fragile

Reproducing a run on a fresh machine requires:

1. Knowing the exact CLI flags for all three years
2. Knowing the worker count and output directory structure used
3. Knowing which artifact types (`--run-analysis`, `--skip-political`, maps, etc.)
   were enabled
4. Running the commands in the correct order across years

None of this is captured in any committed artifact today.

---

## 2. Solution: The RunManifest YAML Format

A **RunManifest** is a single YAML file that completely specifies a multi-year,
multi-state redistricting run: which algorithm, which years, which output
directories, and which artifacts to generate. It is:

- **Committed to version control** alongside the code
- **Machine-readable** by `redist run-manifest` for execution
- **Human-readable** for audit and review
- **Purely declarative** — no side effects until `run-manifest` is invoked
- **Additive** — existing `state`/`states`/`run` commands are unchanged

### 2.1 Complete YAML Schema

```yaml
# RunManifest v1
# Location: outputs/{name}/run.yml  OR  ~/.redist/runs/{name}.yml
# Written by: redist init-manifest  OR  manually
# Executed by: redist run-manifest <path>

# ── Identity ──────────────────────────────────────────────────────────────────
schema_version: "1"
name: official_proposal
description: >
  Federal redistricting proposal for all 50 states across three census years.
  Uses ApportionRegions structure, county-stickiness weights (alpha=2.0),
  and convergence search (threshold=600). Reference implementation for the
  proposed federal redistricting statute.
version: "1.0"
created: "2026-05-04"

# ── Algorithm (three-layer compositor) ────────────────────────────────────────
algorithm:
  # Layer 1: structure — which tree of splits?
  # Values: standard-bisect | nway | ratio-optimal | ratio-optimal-area |
  #         ratio-optimal-vra | prime-factor | compact-polsby | apportion-regions
  structure: apportion-regions

  # Layer 2: weights — what signals go into edge costs?
  # Values: unweighted | geographic | county | vra-aligned | proportional
  weights: county

  # Layer 2 tuning: county stickiness factor (B.10). Ignored if weights != county.
  alpha_county: 2.0

  # Layer 3: search — how to explore the seed space?
  # Values: single | multi | convergence
  search: convergence

  # Convergence threshold: consecutive non-improving seeds before stopping (B.7).
  # Required when search: convergence. Ignored otherwise.
  convergence_threshold: 600

  # Optional multi-seed count. Required when search: multi. Ignored otherwise.
  # seeds: 50

  # Population balance tolerance (percent). Congressional standard: 0.5.
  balance_tolerance: 0.5

  # Parallel workers per year.
  workers: 6

# ── Census years ──────────────────────────────────────────────────────────────
years: [2020, 2010, 2000]

# ── Output directory structure ────────────────────────────────────────────────
# Template variables available in all path strings:
#   {name}        → manifest name (e.g., "official_proposal")
#   {year}        → census year being processed (e.g., "2020")
#   {state_name}  → lowercase underscore state name (e.g., "north_carolina")
#   {base}        → resolved value of outputs.base
#
# All paths are relative to the repository root unless they begin with /.
outputs:
  base: "outputs/{name}"
  states: "{base}/{year}/states/{state_name}/"
  analysis: "{base}/{year}/states/{state_name}/analysis/"
  intermediate: "{base}/{year}/states/{state_name}/intermediate/"
  national: "{base}/national/"

# ── Artifact generation (opt-in; default false for all) ───────────────────────
artifacts:
  # Per-state analysis types to run after redistricting.
  # Mirrors redist analyze --types <list>.
  analysis_types: [demographic, political, compactness, summary]

  # Generate per-state district maps (PNG).
  maps: false
  map_types: [districts]          # only consulted when maps: true
  dpi: 150                        # only consulted when maps: true

  # Generate a static dashboard HTML for each year.
  dashboard: true
  dashboard_path: "{base}/dashboard_{year}.html"

  # Generate national rollup files (us_summary.json, us_demographic.json, etc.)
  national_summary: true
  national_path: "{base}/national_{year}.json"

  # Generate proportionality analysis (requires partisan data).
  proportionality: false

# ── State filter ──────────────────────────────────────────────────────────────
# Empty list = all 50 US states (default).
# Subset list: process only these states (two-letter codes).
states: []
# states: [VT, DE]  # for quick smoke tests

# ── Run metadata (auto-populated at execution time; null in committed file) ───
meta:
  git_commit: null          # filled by redist run-manifest at start
  redist_version: null      # filled by redist run-manifest at start
  started_at: null          # ISO-8601 UTC timestamp
  completed_at: null        # ISO-8601 UTC timestamp; null until run finishes
  completed_years: []       # years successfully completed
  failed_states: {}         # map of year -> [state codes that errored]
```

### 2.2 Template variable resolution

Template variables are resolved left-to-right, with each resolved value available
to subsequent expressions. Resolution is eager and eager-only: no circular
references are permitted.

| Variable | Resolved as |
|----------|-------------|
| `{name}` | `manifest.name` field |
| `{base}` | Resolved value of `outputs.base` (after substituting `{name}`) |
| `{year}` | The census year currently being processed as a string: `"2020"` |
| `{state_name}` | Lowercase underscore state name, e.g., `"north_carolina"` |

Resolution is performed by `redist_cli::manifest_runner::resolve_path(template,
ctx)` where `ctx` carries the four variable values for the current iteration.

All resolved paths are validated to be within the repository root or an absolute
path starting with `/`. Paths escaping the working directory via `..` are
rejected at load time with `[CONFIG] RunManifest: path template '{template}'
resolves outside repository root`.

### 2.3 Schema versioning and forward compatibility

`schema_version: "1"` is required. Future schema versions may add fields; a v1
runner encountering an unrecognised top-level key emits a warning and continues.
A runner encountering `schema_version: "2"` or higher exits with
`[CONFIG] RunManifest: schema_version "2" requires redist >= 2.0.0 (running 1.x)`.

---

## 3. CLI Interface

### 3.1 `redist run-manifest` — execute a manifest

```bash
# Run all years defined in the manifest
redist run-manifest outputs/official_proposal/run.yml

# Run a single year from the manifest (overrides manifest years list)
redist run-manifest outputs/official_proposal/run.yml --year 2020

# Run a subset of states (overrides manifest states list)
redist run-manifest outputs/official_proposal/run.yml --states VT DE

# Dry run: print what would be executed without running anything
redist run-manifest outputs/official_proposal/run.yml --dry-run

# Reprocess all states even if completion markers exist
redist run-manifest outputs/official_proposal/run.yml --reprocess

# Reset (delete) output tree before starting
redist run-manifest outputs/official_proposal/run.yml --reset

# Force overwrite of existing plans
redist run-manifest outputs/official_proposal/run.yml --force
```

**Dry-run output format** (`--dry-run`):

```
RunManifest: official_proposal (schema v1)
  Algorithm: apportion-regions / county (alpha=2.0) / convergence (t=600)
  Years: [2020, 2010, 2000]
  States: all 50
  Outputs:
    base:     outputs/official_proposal
    states:   outputs/official_proposal/{year}/states/{state_name}/
    national: outputs/official_proposal/national/
  Artifacts: analysis=[demographic political compactness summary]
             dashboard=true  national_summary=true  maps=false

Would execute:
  [2020] redist states --year 2020 --output-dir outputs/official_proposal/2020 \
           --partition-mode apportion-regions --weights-override county \
           --alpha-county 2.0 --search convergence --convergence-threshold 600 \
           --workers 6
  [2020] redist aggregate --year 2020 --version official_proposal --types all
  [2020] python scripts/web/generate_master_dashboard.py --year 2020 \
           --out outputs/official_proposal/dashboard_2020.html
  [2010] redist states --year 2010 ...
  [2010] ...
  [2000] redist states --year 2000 ...
  [2000] ...
```

### 3.2 `redist init-manifest` — generate a manifest interactively

```bash
# Generate manifest for the official federal proposal
redist init-manifest \
  --name official_proposal \
  --description "Federal redistricting statute reference implementation" \
  --structure apportion-regions \
  --weights county \
  --alpha-county 2.0 \
  --search convergence \
  --convergence-threshold 600 \
  --years 2020 2010 2000 \
  --workers 6 \
  --out outputs/official_proposal/run.yml

# Generate a quick-test manifest for Vermont only
redist init-manifest \
  --name vt_test \
  --structure apportion-regions \
  --weights county \
  --alpha-county 2.0 \
  --search convergence \
  --convergence-threshold 600 \
  --states VT \
  --years 2020 \
  --out outputs/vt_test/run.yml
```

If `--out` is omitted, the manifest is printed to stdout and not written.
If the output file already exists and `--force` is not set, exits with
`[CONFIG] init-manifest: outputs/official_proposal/run.yml already exists.
Use --force to overwrite.`

### 3.3 `redist dashboard` — generate dashboard from manifest

```bash
# Generate dashboard for all years defined in the manifest
redist dashboard outputs/official_proposal/run.yml

# Generate dashboard for a specific year only
redist dashboard outputs/official_proposal/run.yml --year 2020

# Override output path (ignores manifest dashboard_path template)
redist dashboard outputs/official_proposal/run.yml --year 2020 \
  --out web/dashboard_2020.html
```

This command reads `artifacts.dashboard_path` from the manifest, resolves the
template, and invokes the dashboard generator. It is a thin wrapper over
`generate_master_dashboard.py` that eliminates the need to manually construct
`--version`, `--year`, and `--out` flags.

### 3.4 Exit codes

| Code | Meaning |
|------|---------|
| 0 | All years and states succeeded |
| 1 | One or more states failed (partial failure); outputs for successful states are intact |
| 2 | Manifest parse or validation error |
| 3 | Prerequisite data missing for at least one year |
| 4 | All states failed for at least one year (total failure) |

Exit code 1 (partial failure) is not fatal: the manifest `meta.failed_states`
field records which states errored. A subsequent `redist run-manifest --reprocess`
will retry only the failed states.

---

## 4. Implementation Plan

### 4.1 New file: `redist/crates/redist-cli/src/manifest_runner.rs`

```rust
//! RunManifest executor — parses YAML, resolves templates, dispatches to
//! existing state/states/aggregate commands.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Schema ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct RunManifest {
    pub schema_version: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub created: String,
    pub algorithm: AlgorithmSpec,
    pub years: Vec<u16>,
    pub outputs: OutputSpec,
    pub artifacts: ArtifactSpec,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub meta: RunMeta,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlgorithmSpec {
    pub structure: String,
    pub weights: String,
    #[serde(default)]
    pub alpha_county: f64,
    pub search: String,
    #[serde(default = "default_convergence_threshold")]
    pub convergence_threshold: u32,
    pub seeds: Option<usize>,
    #[serde(default = "default_balance_tolerance")]
    pub balance_tolerance: f64,
    #[serde(default = "default_workers")]
    pub workers: usize,
}

fn default_convergence_threshold() -> u32 { 600 }
fn default_balance_tolerance() -> f64 { 0.5 }
fn default_workers() -> usize { 6 }

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputSpec {
    pub base: String,
    pub states: String,
    pub analysis: String,
    #[serde(default)]
    pub intermediate: Option<String>,
    #[serde(default)]
    pub national: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ArtifactSpec {
    #[serde(default)]
    pub analysis_types: Vec<String>,
    #[serde(default)]
    pub maps: bool,
    #[serde(default)]
    pub map_types: Vec<String>,
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    #[serde(default)]
    pub dashboard: bool,
    pub dashboard_path: Option<String>,
    #[serde(default)]
    pub national_summary: bool,
    pub national_path: Option<String>,
    #[serde(default)]
    pub proportionality: bool,
}

fn default_dpi() -> u32 { 150 }

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RunMeta {
    pub git_commit: Option<String>,
    pub redist_version: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default)]
    pub completed_years: Vec<u16>,
    #[serde(default)]
    pub failed_states: HashMap<String, Vec<String>>,
}

// ── Template resolution ───────────────────────────────────────────────────────

pub struct TemplateCtx<'a> {
    pub name: &'a str,
    pub year: &'a str,
    pub state_name: &'a str,
    pub base: &'a str,
}

/// Resolve a path template by substituting {name}, {year}, {state_name}, {base}.
/// Returns Err if the resolved path escapes the repository root via '..'.
pub fn resolve_path(template: &str, ctx: &TemplateCtx) -> Result<PathBuf, String> {
    let s = template
        .replace("{name}", ctx.name)
        .replace("{base}", ctx.base)
        .replace("{year}", ctx.year)
        .replace("{state_name}", ctx.state_name);
    let p = PathBuf::from(&s);
    // Reject path traversal via ..
    for component in p.components() {
        if component.as_os_str() == ".." {
            return Err(format!(
                "[CONFIG] RunManifest: path template '{}' resolves to '{}' which contains '..'",
                template, s
            ));
        }
    }
    Ok(p)
}

// ── Load and validate ─────────────────────────────────────────────────────────

pub fn load_manifest(path: &Path) -> Result<RunManifest, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("[CONFIG] RunManifest: cannot read '{}': {}", path.display(), e))?;
    let manifest: RunManifest = serde_yaml::from_str(&text)
        .map_err(|e| format!("[CONFIG] RunManifest: YAML parse error in '{}': {}", path.display(), e))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(m: &RunManifest) -> Result<(), String> {
    if m.schema_version != "1" {
        return Err(format!(
            "[CONFIG] RunManifest: schema_version \"{}\" requires a newer redist (running 1.x)",
            m.schema_version
        ));
    }
    if m.years.is_empty() {
        return Err("[CONFIG] RunManifest: years list must not be empty".to_string());
    }
    let valid_structures = ["standard-bisect","nway","ratio-optimal","ratio-optimal-area",
        "ratio-optimal-vra","prime-factor","compact-polsby","apportion-regions"];
    if !valid_structures.contains(&m.algorithm.structure.as_str()) {
        return Err(format!(
            "[CONFIG] RunManifest: unknown algorithm.structure '{}'", m.algorithm.structure
        ));
    }
    let valid_weights = ["unweighted","geographic","county","vra-aligned","proportional"];
    if !valid_weights.contains(&m.algorithm.weights.as_str()) {
        return Err(format!(
            "[CONFIG] RunManifest: unknown algorithm.weights '{}'", m.algorithm.weights
        ));
    }
    let valid_search = ["single","multi","convergence"];
    if !valid_search.contains(&m.algorithm.search.as_str()) {
        return Err(format!(
            "[CONFIG] RunManifest: unknown algorithm.search '{}'", m.algorithm.search
        ));
    }
    Ok(())
}
```

### 4.2 New `Commands` variants in `args.rs`

```rust
/// Execute a RunManifest YAML file (Spec 7)
RunManifest(RunManifestArgs),
/// Generate a RunManifest YAML file interactively (Spec 7)
InitManifest(InitManifestArgs),
/// Generate dashboards from a RunManifest (Spec 7)
Dashboard(DashboardArgs),
```

```rust
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct RunManifestArgs {
    /// Path to RunManifest YAML file
    pub manifest: std::path::PathBuf,

    /// Process only this census year (overrides manifest years list)
    #[arg(short = 'y', long)]
    pub year: Option<String>,

    /// Process only these states (overrides manifest states list)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Print commands without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Reprocess all states (ignore completion markers)
    #[arg(long)]
    pub reprocess: bool,

    /// Delete output tree before starting
    #[arg(long)]
    pub reset: bool,

    /// Force overwrite of existing plan outputs
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct InitManifestArgs {
    /// Manifest name (becomes {name} in path templates)
    #[arg(long)]
    pub name: String,

    /// Human-readable description
    #[arg(long)]
    pub description: Option<String>,

    /// Layer 1: structure (default: apportion-regions)
    #[arg(long, default_value = "apportion-regions")]
    pub structure: String,

    /// Layer 2: weights (default: county)
    #[arg(long, default_value = "county")]
    pub weights: String,

    /// County stickiness alpha (default: 2.0)
    #[arg(long, default_value_t = 2.0)]
    pub alpha_county: f64,

    /// Layer 3: search (default: convergence)
    #[arg(long, default_value = "convergence")]
    pub search: String,

    /// Convergence threshold (default: 600)
    #[arg(long, default_value_t = 600)]
    pub convergence_threshold: u32,

    /// Census years to include (default: 2020 2010 2000)
    #[arg(long = "years", num_args = 1.., value_delimiter = ' ', default_values = ["2020","2010","2000"])]
    pub years: Vec<u16>,

    /// Parallel workers (default: 6)
    #[arg(short = 'w', long, default_value_t = 6)]
    pub workers: usize,

    /// State codes to include (default: all 50)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Output path for generated YAML (default: stdout)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,

    /// Overwrite existing manifest file
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct DashboardArgs {
    /// Path to RunManifest YAML file
    pub manifest: std::path::PathBuf,

    /// Generate dashboard for this year only (default: all manifest years)
    #[arg(short = 'y', long)]
    pub year: Option<String>,

    /// Override dashboard output path (ignores manifest dashboard_path template)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,
}
```

### 4.3 Dispatcher in `main.rs`

```rust
Commands::RunManifest(args) => {
    let manifest = manifest_runner::load_manifest(&args.manifest)
        .unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(2); });
    std::process::exit(manifest_runner::execute(&manifest, &args)?);
}
Commands::InitManifest(args) => {
    manifest_runner::init_manifest(&args)
        .unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(2); });
}
Commands::Dashboard(args) => {
    let manifest = manifest_runner::load_manifest(&args.manifest)
        .unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(2); });
    manifest_runner::generate_dashboards(&manifest, &args)
        .unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
}
```

### 4.4 Execution logic: how `run-manifest` calls existing commands

`manifest_runner::execute()` translates a `RunManifest` into a sequence of
`AlgorithmConfig + StatesArgs` invocations by constructing the same data
structures that `main.rs` would build from CLI flags, then calling the existing
runner functions directly — no subprocess spawning, no shell escaping.

For each `year` in `manifest.years` (in order):

1. **Resolve paths**: call `resolve_path()` for `outputs.base` with the current
   year substituted. The resolved base becomes `--output-dir` for the internal
   `states` invocation.
2. **Build `AlgorithmConfig`**: translate `algorithm.structure`, `algorithm.weights`,
   `algorithm.search` → `SplitStrategy`, `WeightSpec`, `SeedCompositor` using the
   same mapping that `StateArgs` uses in `main.rs`.
3. **Determine state list**: use `args.states` override if non-empty, else
   `manifest.states` if non-empty, else all 50 US states from the registry.
4. **Call `runner::run_states_parallel()`** with the constructed config.
5. **Run analysis** for each state if `artifacts.analysis_types` is non-empty.
6. **Run aggregate** if `artifacts.national_summary` is true.
7. **Generate dashboard** if `artifacts.dashboard` is true.
8. **Update `meta`** in the manifest YAML on disk: add the completed year to
   `meta.completed_years`, record any `meta.failed_states`, update
   `meta.completed_at`.

Step 8 writes the manifest file atomically (write to `.tmp`, rename). This
means the manifest on disk always reflects the last completed state — a partial
run can be resumed with `--reprocess`.

### 4.5 Provenance capture

At step 1 of each year, before any computation:

```rust
meta.git_commit = Some(env!("REDIST_BUILD_COMMIT_OVERRIDE")
    .to_string()
    .or_else(|| /* git rev-parse HEAD */));
meta.redist_version = Some(env!("CARGO_PKG_VERSION").to_string());
meta.started_at = Some(chrono::Utc::now().to_rfc3339());
```

`REDIST_BUILD_COMMIT_OVERRIDE` is already set by the build system (per the
Deposition Prep plan, `depo::build_commit()`). No new environment variable is
introduced.

### 4.6 Meta update atomicity

The manifest is written back to disk after each year completes (not after each
state, to avoid write amplification). If the process is killed mid-year,
`meta.completed_years` will not include the interrupted year. A subsequent
`run-manifest --reprocess` will re-run that year from scratch.

---

## 5. Migration Path

### 5.1 Existing commands are unchanged

`redist state`, `redist states`, and `redist run` accept exactly the same flags
as before. No deprecations are introduced. Existing shell scripts and CI
configurations continue to work.

### 5.2 The manifest is pure sugar

`redist run-manifest` is a thin coordinator that calls the same internal
functions as `redist states` and `redist run`. It does not introduce new
algorithm logic or new output formats. Every output produced by
`redist run-manifest` could also be produced by a shell script calling
`redist states` three times with the appropriate flags.

### 5.3 Incremental adoption path

Teams can adopt the manifest at their own pace:

| Stage | Adoption |
|-------|---------|
| 0 | Existing shell scripts unchanged |
| 1 | `redist init-manifest` generates a run.yml; team reviews and commits it |
| 2 | Shell scripts replaced by `redist run-manifest run.yml` |
| 3 | `--dry-run` used to audit what each run will execute before committing |
| 4 | CI/CD invokes `redist run-manifest` directly |

### 5.4 Path unification

The manifest output path templates enforce the same structure that `redist state`
produces: `{base}/{year}/states/{state_name}/`. This means outputs from a
manifest run are immediately consumable by `redist analyze --version {name}`,
`redist map --version {name}`, and `redist aggregate --version {name}` —
because the version-keyed directory tree is identical.

The inconsistency between `state` and `states` is resolved by the manifest
always generating a `--output-dir` that embeds the year, making the output
tree isomorphic to what `state` would have produced.

---

## 6. The Official Proposal Manifest

The following YAML should be committed at `outputs/official_proposal/run.yml`.
This is the reference implementation for the proposed federal redistricting
statute. It is not generated — it is authored once and committed.

```yaml
schema_version: "1"
name: official_proposal
description: >
  Reference implementation for the proposed federal redistricting statute.
  Algorithm: ApportionRegions (B.11) structure, county-stickiness weights
  alpha=2.0 (B.10), convergence search threshold=600 (B.7/B.16).
  All 50 states, three census years: 2020, 2010, 2000.
  This manifest is the citable artifact that defines the "official proposal" run.
  Any run claiming to reproduce these results must use this manifest unmodified.
version: "1.0"
created: "2026-05-04"

algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 2.0
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5
  workers: 6

years: [2020, 2010, 2000]

outputs:
  base: "outputs/{name}"
  states: "{base}/{year}/states/{state_name}/"
  analysis: "{base}/{year}/states/{state_name}/analysis/"
  intermediate: "{base}/{year}/states/{state_name}/intermediate/"
  national: "{base}/national/"

artifacts:
  analysis_types: [demographic, political, compactness, summary]
  maps: false
  map_types: [districts]
  dpi: 150
  dashboard: true
  dashboard_path: "{base}/dashboard_{year}.html"
  national_summary: true
  national_path: "{base}/national_{year}.json"
  proportionality: true

states: []

meta:
  git_commit: null
  redist_version: null
  started_at: null
  completed_at: null
  completed_years: []
  failed_states: {}
```

**To execute the official proposal:**

```bash
redist run-manifest outputs/official_proposal/run.yml
```

**To reproduce a single year for verification:**

```bash
redist run-manifest outputs/official_proposal/run.yml --year 2020
```

**To perform a smoke test with Vermont only before committing to a full run:**

```bash
redist run-manifest outputs/official_proposal/run.yml \
  --year 2020 --states VT --dry-run
```

---

## 7. Tests

### L0 (unit — no pipeline data required)

| Test | What it checks |
|------|---------------|
| `test_resolve_path_substitutes_all_variables` | `{base}/{year}/states/{state_name}/` → correct expansion |
| `test_resolve_path_rejects_dotdot_traversal` | `../../etc/passwd` exits with `[CONFIG]` error |
| `test_resolve_path_ignores_unknown_variables` | unknown `{foo}` preserved literally |
| `test_validate_manifest_unknown_structure` | unknown `algorithm.structure` returns Err |
| `test_validate_manifest_unknown_weights` | unknown `algorithm.weights` returns Err |
| `test_validate_manifest_empty_years` | empty `years` returns Err |
| `test_validate_manifest_schema_v2_rejected` | `schema_version: "2"` returns Err with message |
| `test_load_manifest_round_trips_yaml` | serialize → deserialize → serialize produces identical YAML |
| `test_init_manifest_produces_valid_yaml` | `init_manifest()` output passes `validate_manifest()` |
| `test_dry_run_output_contains_all_years` | dry-run output mentions all three years |
| `test_meta_update_writes_completed_year` | after year completes, `meta.completed_years` includes it |
| `test_meta_update_is_atomic` | temp file + rename pattern used (no partial write) |
| `test_state_override_wins_over_manifest` | `--states VT` overrides `states: [CA, TX]` in manifest |
| `test_official_proposal_manifest_is_valid` | `load_manifest("outputs/official_proposal/run.yml")` passes |

### L2 acceptance (requires VT or DE adjacency data)

| Test | What it checks |
|------|---------------|
| `test_run_manifest_vt_2020_produces_outputs` | single-state manifest run produces `final_assignments.json` |
| `test_run_manifest_partial_failure_records_failed_states` | intentional bad state → `meta.failed_states` populated |
| `test_run_manifest_reprocess_reruns_failed_states` | `--reprocess` on partial run retries failed states |
| `test_dashboard_command_resolves_template_path` | `redist dashboard run.yml --year 2020` resolves dashboard_path |

---

## 8. Alignment with Existing Specs and Plans

| Spec/Plan | Alignment |
|-----------|-----------|
| **Spec 1 (Custom Parameters)** | `RunManifest.algorithm` maps 1:1 to `StateArgs` compositor flags |
| **Spec 2 (Plan Comparison)** | Manifest outputs use standard `{version}/{year}/states/` tree — compatible with `redist compare --version {name}` |
| **Spec 6 (Reports)** | `PlanManifest` written per-state is unchanged; `RunManifest` records run-level provenance above it |
| **Deposition Prep** | `meta.git_commit` uses `REDIST_BUILD_COMMIT_OVERRIDE` (already defined); `DepoLogWriter` can be pointed at the manifest file |
| **B.10 (county weights)** | `algorithm.alpha_county` maps directly to `WeightSpec.alpha_county` |
| **B.11 (ApportionRegions)** | `algorithm.structure: apportion-regions` maps to `SplitStrategy::ApportionRegions` |
| **B.16 (convergence)** | `algorithm.search: convergence` + `algorithm.convergence_threshold` maps to `SeedCompositor::ConvergenceSweep` |

---

## 9. Open Questions (deferred)

1. **Per-year worker overrides**: Should `workers` be specifiable per year
   (e.g., 2020 uses 6, 2000 uses 4 because data is smaller)? Deferred: flat
   `workers` is sufficient for v1.

2. **Manifest registry**: Should `redist` maintain a registry of named manifests
   in `~/.redist/runs/` accessible by name rather than path?
   (`redist run-manifest official_proposal` vs. `redist run-manifest outputs/.../run.yml`)
   Deferred: path-based is simpler and more explicit for v1.

3. **Resume semantics**: Should `run-manifest` resume from the last completed
   state within a year (not just the last completed year)? Deferred: year-level
   granularity is sufficient for v1; state-level resume can be added when needed.

4. **Manifest signing**: Should the committed `run.yml` carry a GPG signature
   so that any modification is detectable? Deferred: `meta.git_commit` provides
   implicit tamper detection through git history.
