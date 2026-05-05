# Spec 7: Label-Based Run Management

**Date**: 2026-05-04
**Status**: Draft
**Dependencies**: Specs 1–6 (PlanManifest, three-layer compositor, analysis pipeline)
**Depended on by**: Future dashboard automation, CI/CD integration

---

## 1. Problem Statement

### 1.1 Paths are the current API — and they break

Every `redist` command today requires the user to manage paths explicitly:

```bash
redist states --year 2020 --output-dir outputs/official_proposal/2020 \
  --partition-mode apportion-regions --weights-override county \
  --alpha-county 2.0 --search convergence --convergence-threshold 600

redist analyze --version official_proposal --year 2020 --types proportionality splits

redist report --version official_proposal --year 2020 \
  --out outputs/official_proposal/reports/
```

The output directory for `states` and the `--version` argument for `analyze` are
manually kept in sync by the operator. There is no enforcement. A mismatch — e.g.,
running `states` with `--output-dir outputs/op_2020` but then calling `analyze
--version official_proposal` — produces no error, no output, and no diagnosis.

Practical consequence: the three-year run (`states` × 3) produces three trees with
bespoke paths. The dashboard generator hardcodes assumptions about where files live.
If the operator changes the base directory between years, the dashboard breaks
silently. Operators reproduce runs from shell history, not from committed artifacts.

### 1.2 Flag duplication across invocations

The three-layer algorithm compositor exposes roughly twenty flags that must be
passed identically across `state`, `states`, `analyze`, `report`, and `compare`.
There is no committed artifact that records these flags. A flag mismatch — e.g.,
the wrong `--convergence-threshold` for one year — produces outputs that look
structurally correct but differ silently.

### 1.3 Outputs are algorithmically opaque

Given a directory `outputs/official_proposal/2020/states/vermont/`, there is no
machine-readable record of which partition mode, alpha-county value, or convergence
threshold produced those outputs. The per-state `manifest.json` (Spec 1) records
state-level provenance but not run-level configuration. Auditors must reconstruct
this from shell history or memory.

### 1.4 The core insight

The problem is not that paths are wrong — it is that paths are the API. Users
should name things, not locate them. The system should resolve names to paths
through convention, not require the user to thread path strings through every
subcommand.

---

## 2. Solution: Label-Based Run Management

**Core principle: names are the API; paths are the implementation.**

A **label** is a short human name for a run (`official_proposal`, `senate_draft2`,
`vt_test`). The user names runs once; every `redist` command resolves the label to
the correct directory through fixed convention. There are no path arguments in
normal use.

```bash
redist build   official_proposal --config configs/statute.yml --year 2020
redist analyze official_proposal --types proportionality splits
redist report  official_proposal
redist compare official_proposal senate_draft2 --year 2020
redist ls
redist show    official_proposal
```

### 2.1 Convention-based directory resolution

Every command resolves its input and output directories from the label alone:

| Command | Reads from | Writes to |
|---------|-----------|-----------|
| `build X` | — | `runs/X/` |
| `analyze X` | `runs/X/` | `analysis/X/` |
| `report X` | `analysis/X/` | `reports/X/` |
| `compare A B` | `analysis/A/`, `analysis/B/` | — (stdout or `--out`) |

No path arguments. No `--version` flags. No `--output-dir` strings. The label is
the only identifier the user ever types.

### 2.2 The `.redist` registry

A single JSON file at the repository root tracks what has been done to each label.
It is never edited by the user — only written by `redist` commands.

```json
{
  "official_proposal": {
    "built":    ["2020", "2010", "2000"],
    "analyzed": ["2020"],
    "reported": []
  },
  "senate_draft2": {
    "built":    ["2020"],
    "analyzed": ["2020"],
    "reported": ["2020"]
  }
}
```

The registry is git-ignored and auto-managed. Every `build`/`analyze`/`report`
call updates it. It is the authoritative answer to "what has been done."

### 2.3 One `index.json` per stage per label

Each stage writes a single `index.json` at the root of its output directory.
This is the machine-readable record of what happened — the algorithm, the year,
the state-by-state results.

```
runs/official_proposal/index.json          ← build index
analysis/official_proposal/index.json      ← analysis index (references run)
reports/official_proposal/index.json       ← report index (references analysis)
```

### 2.4 Separation of concerns: config vs. registry

The **config file** (`configs/official_proposal.yml`) declares WHAT TO DO:
algorithm parameters, years, workers. The user edits this before running.

The **registry** (`.redist`) records WHAT WAS DONE: which years were built,
analyzed, reported. The system writes this; the user never edits it.

The **index files** (`runs/X/index.json`, `analysis/X/index.json`) record HOW IT
WENT: state-by-state status, timestamps, version. The system writes these; the
user reads them for diagnosis.

---

## 3. Verb Inventory

### 3.1 `redist build X` — run redistricting for a label

```bash
# Full build (all years in config)
redist build official_proposal --config configs/statute.yml

# Single year
redist build official_proposal --config configs/statute.yml --year 2020

# Subset of states
redist build official_proposal --config configs/statute.yml --year 2020 \
  --states VT DE

# Override worker count
redist build official_proposal --config configs/statute.yml --workers 4

# Dry run — print what would run without executing
redist build official_proposal --config configs/statute.yml --dry-run

# Use explicit label flag (equivalent to first positional argument)
redist build --label official_proposal --config configs/statute.yml
```

`build` writes outputs to `runs/official_proposal/` and updates `.redist`
with the years successfully built.

### 3.2 `redist analyze X` — run analysis for a label

```bash
# All analysis types (default)
redist analyze official_proposal

# Specific types only
redist analyze official_proposal --types proportionality splits

# Single year
redist analyze official_proposal --types proportionality --year 2020

# Subset of states
redist analyze official_proposal --types compactness --year 2020 --states CA TX NY
```

`analyze` reads from `runs/official_proposal/` and writes to
`analysis/official_proposal/`. Requires the label to appear in `.redist` with at
least one built year. If `--year Y` is requested but `Y` is not in `.redist`
`built` list, exits with:

```
[CONFIG] analyze: 'official_proposal' has not been built for year 2010.
Run: redist build official_proposal --year 2010
```

### 3.3 `redist report X` — generate reports for a label

```bash
# HTML dashboard (default)
redist report official_proposal

# Single year
redist report official_proposal --year 2020

# Format selection
redist report official_proposal --year 2020 --format json
redist report official_proposal --year 2020 --format html

# Expert-witness mode (for court submission)
redist report official_proposal --year 2020 --format pdf \
  --expert-name "Dr. Jane Smith" --jurisdiction "Wisconsin"
```

`report` reads from `analysis/official_proposal/` and writes to
`reports/official_proposal/`. Requires the label to have been analyzed for the
requested year.

### 3.4 `redist compare A B` — compare two labels

```bash
# Compare two runs for a year
redist compare official_proposal senate_draft2 --year 2020

# Specific metrics
redist compare official_proposal senate_draft2 --year 2020 \
  --metrics compactness splits proportionality

# Output to file
redist compare official_proposal senate_draft2 --year 2020 \
  --out comparison_2020.json

# Compare against enacted districts (downloads if absent)
redist compare official_proposal --enacted --year 2020
```

`compare` reads from `analysis/A/` and `analysis/B/`. Both labels must appear
in `.redist` with the requested year analyzed.

### 3.5 `redist ls` — list all labels and their stage completion

```
$ redist ls

Label                Built           Analyzed        Reported
official_proposal    2020 2010 2000  2020            —
senate_draft2        2020            2020            2020
vt_test              2020            —               —
```

### 3.6 `redist show X` — show details for a label

```
$ redist show official_proposal

Label:     official_proposal
Config:    configs/statute.yml
Algorithm: apportion-regions / county (alpha=2.0) / convergence (t=600)
Workers:   6

Built:
  2020  runs/official_proposal/2020/  50/50 states OK
  2010  runs/official_proposal/2010/  50/50 states OK
  2000  runs/official_proposal/2000/  50/50 states OK

Analyzed:
  2020  analysis/official_proposal/2020/  [proportionality, splits, compactness, summary]

Reported:
  (none)

Paths:
  runs/     runs/official_proposal/
  analysis/ analysis/official_proposal/
  reports/  reports/official_proposal/
```

### 3.7 `redist rm X` — delete a stage

```bash
# Delete the report stage only
redist rm official_proposal --stage report

# Delete the analysis stage (also deletes report if it exists)
redist rm official_proposal --stage analyze

# Delete everything (build + analysis + report)
redist rm official_proposal

# Delete a specific year's build outputs only
redist rm official_proposal --stage build --year 2010
```

`rm` updates `.redist` to reflect the deleted stages. Prompts for confirmation
unless `--force` is set.

### 3.8 `redist label X Y` — copy a label

```bash
# Point label Y at the same runs as X (does not copy files — updates registry)
redist label official_proposal official_proposal_v1
```

Creates registry entry `official_proposal_v1` pointing to the same run directories
as `official_proposal`. Useful for tagging a completed run before starting a new
build under the same label.

### 3.9 `redist plan [X]` — interactive TUI for the full pipeline

`redist plan` is the interactive terminal frontend for the entire label system.
It replaces `redist tui` with a label-aware, action-oriented interface.

```bash
redist plan                      # TUI — pick a label from registry
redist plan official_proposal    # TUI — pre-scoped to this label
redist plan official_proposal --configure  # TUI — edit algorithm config
```

**Entry screen (no label specified):**
```
┌─ redist ─────────────────────────────────────────────────────┐
│ Labels                          Built    Analyzed   Reported  │
│ ──────                          ──────   ────────   ──────── │
│ official_proposal               2020–00  2020       —        │
│ bakeoff_geo                     2020     —          —        │
│                                                              │
│ [n] New label  [Enter] Select  [q] Quit                      │
└──────────────────────────────────────────────────────────────┘
```

**Label screen (`redist plan official_proposal`):**
```
┌─ official_proposal ──────────────────────────────────────────┐
│ Algorithm: ApportionRegions + County(α=2.0) + Convergence(600)│
│                                                              │
│ Year   Built     Analyzed   Reported                        │
│ ────   ──────    ────────   ────────                        │
│ 2020   ✓ 50/50  ✓ 50/50   —                               │
│ 2010   ✓ 50/50  —          —                               │
│ 2000   ✓ 49/50  —          —                               │
│                                                              │
│ [b] Build  [a] Analyze  [r] Report  [c] Configure  [q] Back  │
└──────────────────────────────────────────────────────────────┘
```

**Configure screen (`redist plan official_proposal --configure`):**

Interactive three-layer compositor wizard:
```
┌─ Configure: official_proposal ───────────────────────────────┐
│                                                              │
│ Layer 1 — Structure                                         │
│   ● apportion-regions   HH prime-factor tree (B.11)        │
│   ○ ratio-optimal       GeoSection (B.8)                   │
│   ○ ratio-optimal-area  AreaSection (B.9)                  │
│                                                              │
│ Layer 2 — Weights                                           │
│   ● county  α=[ 2.0 ]   County-sticky (B.10)              │
│   ○ geographic           TIGER boundary lengths             │
│   ○ vra-aligned          Minority alignment (B.14)         │
│                                                              │
│ Layer 3 — Search                                            │
│   ● convergence  T=[ 600 ]   Seed buster (B.16)           │
│   ○ multi        N=[ 50  ]   Fixed seeds                   │
│   ○ single                   Content-derived seed only     │
│                                                              │
│ [↑↓] Select  [←→] Adjust values  [s] Save  [q] Cancel      │
└──────────────────────────────────────────────────────────────┘
```

Pressing `[s]` writes the selected configuration to `configs/official_proposal.yml`.
The user then runs `redist build official_proposal` (or presses `[b]` from the
label screen) to execute.

The configure screen is the interactive frontend for the three-layer compositor —
it exposes the full `--structure` / `--weights-override` / `--search` flags
through menus rather than CLI flags, making the algorithm choices visible and
explorable without memorising flag names.

**Implementation note:** `redist plan` calls the existing `redist-tui` binary
(already in the workspace) with a new `--label` flag. The TUI crate handles
terminal rendering; `redist plan` is a thin dispatcher. This makes `redist tui`
an alias for `redist plan` with no label specified.

### 3.10 Escape hatches for power users

```bash
# Override output directory (build writes here instead of runs/X/)
redist build official_proposal --out /mnt/results/

# Show all resolved paths without running
redist show official_proposal

# Read analysis from a non-standard location
redist report official_proposal --analysis-dir /mnt/results/analysis/
```

---

## 4. Directory Structure

```
.redist                                    ← registry JSON (git-ignored, auto-managed)

configs/
  official_proposal.yml                   ← algorithm config (git-tracked, user-edited)
  senate_draft2.yml

runs/
  official_proposal/
    index.json                             ← build index (what was built)
    2020/
      wisconsin/
        assignments.json                   ← tract → district map
        provenance.json                    ← per-state audit chain
      california/
        assignments.json
        provenance.json
      ...
    2010/
      ...
    2000/
      ...
  senate_draft2/
    index.json
    2020/
      ...

analysis/
  official_proposal/
    index.json                             ← analysis index (what was analyzed)
    2020/
      wisconsin/
        proportionality.json
        splits.json
        compactness.json
        summary.json
      california/
        ...
    2010/
      ...
  senate_draft2/
    index.json
    2020/
      ...

reports/
  official_proposal/
    index.json                             ← report index (what was reported)
    dashboard_2020.html
    dashboard_2010.html
    dashboard_2000.html
  senate_draft2/
    index.json
    dashboard_2020.html
```

All three top-level directories (`runs/`, `analysis/`, `reports/`) are
git-ignored. The only committed artifacts are the config files in `configs/`
and the spec documents in `docs/specs/`.

---

## 5. Index.json Schemas

### 5.1 Build index (`runs/X/index.json`)

Written by `redist build` after each year completes. Updated atomically
(write to `.tmp`, rename).

```json
{
  "label": "official_proposal",
  "schema_version": "1",
  "algorithm": {
    "structure": "apportion-regions",
    "weights": "county",
    "alpha_county": 2.0,
    "search": "convergence",
    "convergence_threshold": 600,
    "balance_tolerance": 0.5
  },
  "config_file": "configs/official_proposal.yml",
  "config_sha256": "a3f8bc2d...",
  "created": "2026-05-04T18:30:00Z",
  "redist_version": "0.1.0",
  "git_commit": "f263823a...",
  "years": {
    "2020": {
      "started":   "2026-05-04T18:30:00Z",
      "completed": "2026-05-04T19:45:00Z",
      "workers": 6,
      "succeeded": 50,
      "failed": 0,
      "states": {
        "wisconsin":      { "status": "ok",   "districts": 8  },
        "california":     { "status": "ok",   "districts": 52 },
        "vermont":        { "status": "ok",   "districts": 1  }
      }
    },
    "2010": {
      "started":   "2026-05-04T19:46:00Z",
      "completed": "2026-05-04T21:02:00Z",
      "workers": 6,
      "succeeded": 50,
      "failed": 0,
      "states": {}
    }
  }
}
```

### 5.2 Analysis index (`analysis/X/index.json`)

Written by `redist analyze` after each year completes. References the run
by label (not by path — the path is derived from the label by convention).

```json
{
  "label": "official_proposal",
  "schema_version": "1",
  "run": "official_proposal",
  "run_index_sha256": "d4e7a1c9...",
  "types": ["proportionality", "splits", "compactness", "summary"],
  "created": "2026-05-04T19:00:00Z",
  "redist_version": "0.1.0",
  "git_commit": "f263823a...",
  "years": {
    "2020": {
      "started":   "2026-05-04T19:00:00Z",
      "completed": "2026-05-04T19:15:00Z",
      "states": {
        "wisconsin": {
          "proportionality": "ok",
          "splits":          "ok",
          "compactness":     "ok",
          "summary":         "ok"
        },
        "california": {
          "proportionality": "ok",
          "splits":          "ok",
          "compactness":     "ok",
          "summary":         "ok"
        }
      }
    }
  }
}
```

The `run` field references by NAME not by path. The path to the run outputs
is always `runs/{run}/` — this is the convention. An auditor reading the
analysis index knows exactly where to find the corresponding run outputs.

### 5.3 Report index (`reports/X/index.json`)

Written by `redist report` after generation completes.

```json
{
  "label": "official_proposal",
  "schema_version": "1",
  "analysis": "official_proposal",
  "analysis_index_sha256": "c9b2e3f1...",
  "created": "2026-05-04T20:00:00Z",
  "redist_version": "0.1.0",
  "git_commit": "f263823a...",
  "years": {
    "2020": {
      "format": "html",
      "output": "reports/official_proposal/dashboard_2020.html",
      "output_sha256": "8f2a1c7b..."
    }
  }
}
```

---

## 6. The `.redist` Registry

### 6.1 Format

```json
{
  "official_proposal": {
    "built":    ["2020", "2010", "2000"],
    "analyzed": ["2020"],
    "reported": []
  },
  "senate_draft2": {
    "built":    ["2020"],
    "analyzed": ["2020"],
    "reported": ["2020"]
  },
  "vt_test": {
    "built":    ["2020"],
    "analyzed": [],
    "reported": []
  }
}
```

### 6.2 Invariants

- Every label in the registry has exactly three lists: `built`, `analyzed`, `reported`.
- A year appears in `analyzed` only if it also appears in `built`.
- A year appears in `reported` only if it also appears in `analyzed`.
- The registry is updated atomically after each stage completes for a year.
- The registry is never read for path resolution — paths are always derived from
  the label by convention. The registry is read only to validate that prerequisites
  are met before a command runs.

### 6.3 Location and gitignore

`.redist` lives at the repository root. It is git-ignored:

```
# .gitignore addition
.redist
runs/
analysis/
reports/
```

If `.redist` does not exist, `redist ls` prints an empty table and any command
that reads the registry (e.g., `analyze`) treats all labels as having no prior work.

---

## 7. Algorithm Config File

The config file is a YAML document the user edits and commits. It declares what to
do when `redist build` is called with this label. It is separate from the registry
(which records what was done) and separate from the index (which records how it went).

### 7.1 Schema

```yaml
# configs/official_proposal.yml
# User-editable. Commit this. Do not confuse with .redist (auto-managed).

name: official_proposal
description: >
  Federal redistricting proposal for all 50 states across three census years.
  Uses ApportionRegions structure (B.11), county-stickiness weights (alpha=2.0,
  B.10), and convergence search (threshold=600, B.7/B.16). Reference implementation
  for the proposed federal redistricting statute.

algorithm:
  # Layer 1: bisection structure
  # Values: standard-bisect | nway | ratio-optimal | ratio-optimal-area |
  #         ratio-optimal-vra | prime-factor | compact-polsby | apportion-regions
  structure: apportion-regions

  # Layer 2: edge weights
  # Values: unweighted | geographic | county | vra-aligned | proportional
  weights: county

  # County stickiness factor (B.10). Ignored if weights != county.
  alpha_county: 2.0

  # Layer 3: seed search strategy
  # Values: single | multi | convergence
  search: convergence

  # Consecutive non-improving seeds before stopping (B.7/B.16).
  # Required when search: convergence.
  convergence_threshold: 600

  # Population balance tolerance (percent). Congressional standard: 0.5.
  balance_tolerance: 0.5

# Parallel workers per year.
workers: 6

# Census years to build. All three is standard for a federal proposal.
years: [2020, 2010, 2000]

# Analysis types to run after each build year.
# Values: demographic | political | compactness | contiguity | splits | summary
analysis_types: [demographic, political, compactness, contiguity, splits, summary]
```

### 7.2 Rules

- `name` must match the label used in `redist build` commands. If they differ,
  `build` exits with `[CONFIG] build: config name 'X' does not match label 'Y'`.
- All `algorithm` fields are required except `convergence_threshold` (required only
  when `search: convergence`) and `alpha_county` (required only when
  `weights: county`).
- `workers` defaults to 4 if omitted.
- `years` defaults to `[2020, 2010, 2000]` if omitted.
- `analysis_types` defaults to `[compactness, splits, summary]` if omitted.
- Unknown keys cause a parse error (`[CONFIG] config: unknown field 'weigths'`).

### 7.3 Creating a config

```bash
# Generate a config interactively
redist config new official_proposal \
  --structure apportion-regions \
  --weights county \
  --alpha-county 2.0 \
  --search convergence \
  --convergence-threshold 600 \
  --years 2020 2010 2000 \
  --workers 6 \
  --out configs/official_proposal.yml

# Print to stdout without writing
redist config new official_proposal ... --dry-run

# Validate an existing config
redist config validate configs/official_proposal.yml
```

---

## 8. Implementation Plan

### 8.1 New files

```
redist/crates/redist-cli/src/
  label.rs            ← label resolution: label → path convention
  registry.rs         ← .redist read/write, atomic update, invariant enforcement
  build_cmd.rs        ← `redist build` dispatcher (replaces/wraps states runner)
  config_cmd.rs       ← `redist config new/validate`
  ls_cmd.rs           ← `redist ls` and `redist show`
  rm_cmd.rs           ← `redist rm`

configs/
  official_proposal.yml   ← committed algorithm config (this spec's reference)
```

### 8.2 Modified files

```
redist/crates/redist-cli/src/
  args.rs     ← add Build, Ls, Show, Rm, Label, ConfigNew, ConfigValidate variants
  main.rs     ← dispatch new Commands variants
```

### 8.3 New `Commands` variants

```rust
/// Run redistricting for a label (reads config, writes runs/X/)
Build(BuildArgs),
/// Run analysis for a label (reads runs/X/, writes analysis/X/)
Analyze(AnalyzeArgs),       // replaces existing analyze with label-aware version
/// Generate reports for a label (reads analysis/X/, writes reports/X/)
Report(ReportArgs),         // replaces existing report with label-aware version
/// Compare two labels (reads analysis/A/, analysis/B/)
Compare(CompareArgs),       // replaces existing compare with label-aware version
/// List all labels and their stage completion
Ls(LsArgs),
/// Show details for a label
Show(ShowArgs),
/// Delete a stage for a label
Rm(RmArgs),
/// Copy a label (update registry only)
Label(LabelArgs),
/// Config subcommand group (new, validate)
Config(ConfigArgs),
/// Interactive TUI for the full pipeline — label-aware frontend
/// Replaces `redist tui`; `redist tui` becomes an alias for `redist plan`
Plan(PlanArgs),
```

### 8.4 `label.rs` — path convention

```rust
/// Resolve the runs directory for a label.
/// Convention: runs/{label}/
pub fn runs_dir(label: &str) -> PathBuf {
    PathBuf::from("runs").join(label)
}

/// Resolve the analysis directory for a label.
/// Convention: analysis/{label}/
pub fn analysis_dir(label: &str) -> PathBuf {
    PathBuf::from("analysis").join(label)
}

/// Resolve the reports directory for a label.
/// Convention: reports/{label}/
pub fn reports_dir(label: &str) -> PathBuf {
    PathBuf::from("reports").join(label)
}

/// Resolve the build index path for a label.
pub fn build_index_path(label: &str) -> PathBuf {
    runs_dir(label).join("index.json")
}

/// Resolve the analysis index path for a label.
pub fn analysis_index_path(label: &str) -> PathBuf {
    analysis_dir(label).join("index.json")
}

/// Resolve the report index path for a label.
pub fn report_index_path(label: &str) -> PathBuf {
    reports_dir(label).join("index.json")
}
```

### 8.5 `registry.rs` — `.redist` read/write

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Registry {
    #[serde(flatten)]
    pub labels: HashMap<String, LabelRecord>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LabelRecord {
    pub built:    Vec<String>,
    pub analyzed: Vec<String>,
    pub reported: Vec<String>,
}

/// Load .redist from the repository root. Returns empty registry if absent.
pub fn load() -> Result<Registry, String> { ... }

/// Write .redist atomically (write to .redist.tmp, rename).
pub fn save(registry: &Registry) -> Result<(), String> { ... }

/// Mark a year as built for a label. Enforces invariants.
pub fn mark_built(label: &str, year: &str) -> Result<(), String> { ... }

/// Mark a year as analyzed. Returns Err if year not in built.
pub fn mark_analyzed(label: &str, year: &str) -> Result<(), String> { ... }

/// Mark a year as reported. Returns Err if year not in analyzed.
pub fn mark_reported(label: &str, year: &str) -> Result<(), String> { ... }

/// Require that a year has been built. Used by analyze for pre-flight checks.
pub fn require_built(label: &str, year: &str) -> Result<(), String> { ... }

/// Require that a year has been analyzed. Used by report for pre-flight checks.
pub fn require_analyzed(label: &str, year: &str) -> Result<(), String> { ... }
```

### 8.6 `BuildArgs` struct

```rust
#[derive(Debug, Parser)]
pub struct BuildArgs {
    /// Label for this run (first positional argument or --label)
    #[arg(group = "label_source")]
    pub label: Option<String>,

    #[arg(long = "label", group = "label_source")]
    pub label_flag: Option<String>,

    /// Path to algorithm config YAML
    #[arg(long)]
    pub config: std::path::PathBuf,

    /// Build only this census year (overrides config years list)
    #[arg(short = 'y', long)]
    pub year: Option<String>,

    /// Build only these states (overrides config years list)
    #[arg(long = "states", num_args = 0.., value_delimiter = ' ')]
    pub states: Vec<String>,

    /// Parallel workers (overrides config workers)
    #[arg(short = 'w', long)]
    pub workers: Option<usize>,

    /// Print commands without executing
    #[arg(long)]
    pub dry_run: bool,

    /// Force overwrite of existing outputs
    #[arg(long)]
    pub force: bool,

    /// Override output directory (default: runs/{label}/)
    #[arg(long)]
    pub out: Option<std::path::PathBuf>,
}
```

### 8.7 Migration from existing commands

The existing `redist state`, `redist states`, and `redist run` commands are
**unchanged**. No deprecations are introduced in this spec. Label-based commands
are new top-level verbs that call the same internal runner functions:

| Label command | Delegates to |
|---------------|-------------|
| `redist build X` | `runner::run_states_parallel()` with resolved output dir |
| `redist analyze X` | existing analyze pipeline with resolved input/output dirs |
| `redist report X` | existing report pipeline with resolved paths |
| `redist compare A B` | existing compare with resolved analysis dirs |

The resolved output directory for `build` follows the same tree that `redist states`
produces: `runs/{label}/{year}/states/{state_name}/`. This means build outputs are
immediately consumable by the existing per-state tooling.

### 8.8 Index.json writing

After each year completes in `build`:

1. Load existing `runs/{label}/index.json` if it exists.
2. Merge the new year's results into the `years` map.
3. Write back atomically (`.tmp` + rename).
4. Call `registry::mark_built(label, year)`.

After each year completes in `analyze`:

1. Load existing `analysis/{label}/index.json` if it exists.
2. Merge results. Write back atomically.
3. Call `registry::mark_analyzed(label, year)`.

After `report` completes:

1. Write `reports/{label}/index.json` with output paths and SHAs.
2. Call `registry::mark_reported(label, year)`.

---

## 9. The Official Proposal End-to-End

### 9.1 Config file (committed)

```yaml
# configs/official_proposal.yml
name: official_proposal
description: >
  Reference implementation for the proposed federal redistricting statute.
  Algorithm: ApportionRegions (B.11), county weights alpha=2.0 (B.10),
  convergence search threshold=600 (B.7/B.16). All 50 states, 2020/2010/2000.
  This config is the citable artifact defining the official proposal parameters.
  Any run claiming to reproduce these results must use this config unmodified.

algorithm:
  structure: apportion-regions
  weights: county
  alpha_county: 2.0
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5

workers: 6
years: [2020, 2010, 2000]
analysis_types: [demographic, political, compactness, contiguity, splits, summary]
```

### 9.2 Full workflow

```bash
# Step 1: Smoke test with Vermont before committing to a full run
redist build official_proposal --config configs/official_proposal.yml \
  --year 2020 --states VT --dry-run

# Step 2: Build all 50 states, all three years (~2–4h)
redist build official_proposal --config configs/official_proposal.yml

# Step 3: Check what was built
redist ls
# official_proposal    2020 2010 2000    —    —

# Step 4: Run analysis for 2020
redist analyze official_proposal --year 2020

# Step 5: Generate report
redist report official_proposal --year 2020 --format html

# Step 6: Inspect status
redist show official_proposal
# Label:     official_proposal
# Built:     2020 2010 2000
# Analyzed:  2020
# Reported:  2020

# Step 7: Compare against an alternative plan
redist build senate_draft2 --config configs/senate_draft2.yml --year 2020
redist analyze senate_draft2 --year 2020
redist compare official_proposal senate_draft2 --year 2020 \
  --metrics compactness splits proportionality
```

### 9.3 Error scenarios

```bash
# Attempt to analyze before building
$ redist analyze official_proposal --year 2010
[CONFIG] analyze: 'official_proposal' has not been built for year 2010.
Run: redist build official_proposal --year 2010

# Attempt to report before analyzing
$ redist report official_proposal --year 2000
[CONFIG] report: 'official_proposal' has not been analyzed for year 2000.
Run: redist analyze official_proposal --year 2000

# Config name mismatch
$ redist build senate_draft2 --config configs/official_proposal.yml
[CONFIG] build: config name 'official_proposal' does not match label 'senate_draft2'.

# Label collision without --force
$ redist build official_proposal --config configs/official_proposal.yml --year 2020
[CONFIG] build: 'official_proposal' year 2020 already exists (built 2026-05-04T18:30Z).
Use --force to overwrite.
```

### 9.3 Interactive workflow via `redist plan`

For operators who prefer a menu-driven interface rather than CLI flags:

```bash
# Open the interactive planner for this label
redist plan official_proposal
```

This opens the TUI showing current build/analyze/report status.
From inside the TUI:
- Press `[c]` → opens the three-layer compositor wizard to view or modify `configs/official_proposal.yml`
- Press `[b]` for year 2010 → runs `redist build official_proposal --year 2010` in the background
- Press `[a]` for year 2020 → runs `redist analyze official_proposal --year 2020`
- Press `[r]` for year 2020 → runs `redist report official_proposal --year 2020`
- Navigate state table → drill into per-state metrics and maps

The plan TUI is the operator's primary interface. CLI flags are the power-user
and scripting interface. Both produce identical outputs — `plan` is a thin
dispatcher over the same underlying commands.

**Configure screen** (accessible via `[c]` or `redist plan X --configure`):
```
┌─ Configure: official_proposal ───────────────────────────────┐
│                                                              │
│ Layer 1 — Structure                                         │
│   ● apportion-regions   HH prime-factor tree (B.11)        │
│   ○ ratio-optimal       GeoSection (B.8)                   │
│   ○ ratio-optimal-area  AreaSection (B.9)                  │
│                                                              │
│ Layer 2 — Weights                                           │
│   ● county  α=[ 2.0 ]   County-sticky (B.10)              │
│   ○ geographic           TIGER boundary lengths (default)   │
│   ○ vra-aligned  w=[ 0.40 ]  Minority alignment (B.14)    │
│                                                              │
│ Layer 3 — Search                                            │
│   ● convergence  T=[ 600 ]   Certified optimal (B.16)     │
│   ○ multi        N=[ 50  ]   Fixed seed count              │
│   ○ single                   Content-derived seed only     │
│                                                              │
│ Balance tolerance: [ 3.0 ]%    Workers: [ 6 ]             │
│                                                              │
│ [↑↓] Select layer  [←→] Adjust values  [s] Save  [q] Cancel │
└──────────────────────────────────────────────────────────────┘
```

`[s]` writes to `configs/official_proposal.yml`. The in-TUI description
("`HH prime-factor tree (B.11)`", "`Certified optimal (B.16)`") links each
choice to its research paper, making the legal justification visible at the
point of decision.

---

## 10. Tests

### L0 (unit — no pipeline data required)

| Test | What it checks |
|------|---------------|
| `test_label_runs_dir` | `label::runs_dir("official_proposal")` == `runs/official_proposal` |
| `test_label_analysis_dir` | convention consistent with runs_dir |
| `test_label_reports_dir` | convention consistent with runs_dir |
| `test_registry_mark_built_updates_built_list` | year appears in `built` after mark |
| `test_registry_mark_analyzed_requires_built` | year not in `built` → Err |
| `test_registry_mark_reported_requires_analyzed` | year not in `analyzed` → Err |
| `test_registry_write_is_atomic` | tmp file renamed, no partial write |
| `test_registry_load_missing_returns_empty` | absent `.redist` → empty registry |
| `test_registry_invariants_built_superset_of_analyzed` | invariant enforced |
| `test_registry_invariants_analyzed_superset_of_reported` | invariant enforced |
| `test_build_index_round_trip` | serialize → deserialize → serialize → identical JSON |
| `test_analysis_index_references_run_by_name` | `"run": "official_proposal"` not a path |
| `test_report_index_records_sha256` | output file SHA in report index |
| `test_config_name_mismatch_exits_error` | config `name: X` + label `Y` → `[CONFIG]` |
| `test_config_missing_required_field_exits_error` | missing `structure` → `[CONFIG]` |
| `test_config_unknown_field_exits_error` | typo `weigths` → `[CONFIG]` |
| `test_require_built_exits_with_actionable_message` | error message contains fix |
| `test_require_analyzed_exits_with_actionable_message` | error message contains fix |
| `test_label_collision_exits_with_timestamp` | error message shows `built` timestamp |
| `test_dry_run_prints_all_years` | dry-run output lists all config years |
| `test_ls_empty_registry_prints_empty_table` | no crash when `.redist` absent |
| `test_official_proposal_config_is_valid` | `load_config("configs/official_proposal.yml")` passes |

### L2 acceptance (requires VT adjacency data)

| Test | What it checks |
|------|---------------|
| `test_build_vt_2020_produces_index_json` | `runs/vt_test/index.json` written after build |
| `test_build_updates_registry_built_list` | `.redist` contains `vt_test.built: ["2020"]` |
| `test_analyze_vt_2020_produces_analysis_index` | `analysis/vt_test/index.json` written |
| `test_analyze_updates_registry_analyzed_list` | `.redist` updated after analyze |
| `test_report_vt_2020_produces_dashboard` | `reports/vt_test/dashboard_2020.html` exists |
| `test_show_vt_test_displays_correct_paths` | `redist show vt_test` output matches actual paths |
| `test_rm_stage_report_deletes_reports_dir` | `reports/vt_test/` removed, `.redist` updated |
| `test_compare_two_labels_produces_output` | `redist compare A B` exits 0 with metric table |

---

## 11. Alignment with Existing Specs and Plans

| Spec/Plan | Alignment |
|-----------|-----------|
| **Spec 1 (Custom Parameters)** | `BuildArgs` passes `--balance-tolerance`, `--population-source` through to `StateConfig` unchanged |
| **Spec 2 (Plan Comparison)** | `redist compare A B` is the label-aware replacement for `redist compare --plan-a --plan-b` |
| **Spec 3 (Constraint Analysis)** | Analysis types in config file map to `--types` flag |
| **Spec 4 (Partisan Metrics)** | `political` in `analysis_types` triggers partisan analysis |
| **Spec 5 (Multi-chamber)** | `redist build X --config configs/wa_house.yml` — label and config independent |
| **Spec 6 (Reports)** | `PlanManifest` per-state is unchanged; `reports/X/index.json` is the new run-level audit anchor |
| **Deposition Prep** | `git_commit` in every index uses `REDIST_BUILD_COMMIT_OVERRIDE`; `DepoLogWriter` can reference index files |
| **B.10 (county weights)** | `algorithm.alpha_county` in config maps to `WeightSpec.alpha_county` |
| **B.11 (ApportionRegions)** | `algorithm.structure: apportion-regions` maps to `SplitStrategy::ApportionRegions` |
| **B.16 (convergence)** | `algorithm.search: convergence` + `convergence_threshold` maps to `SeedCompositor::ConvergenceSweep` |

---

## 12. Open Questions (deferred)

1. **Per-year worker overrides**: Should `workers` be specifiable per year in the
   config? Deferred: flat `workers` is sufficient for v1.

2. **Global config search path**: Should `redist build X` search for `configs/X.yml`
   automatically, making `--config` optional? Deferred: explicit `--config` is safer
   for v1; auto-discovery can be added when the pattern proves stable.

3. **Label namespacing**: Should labels be namespaced (e.g., `federal/official_proposal`
   vs. `wa/senate_draft2`)? Deferred: flat namespace is simpler for v1. Subdirectory
   support in `runs/`, `analysis/`, `reports/` can be added transparently.

4. **Registry signing**: Should `.redist` carry a signature so that tampering
   (e.g., manually marking a year as built) is detectable? Deferred: git history
   provides implicit tamper detection. Explicit signing can be added for deposition
   workflows.

5. **Remote registry**: Should `.redist` be sharable (e.g., stored in S3 alongside
   outputs) so a team member can run `redist ls` against a shared output tree without
   running `build` locally? Deferred: local-only for v1.

---

---

## Panel Reviews

**Date**: 2026-05-04
**Spec**: `2026-05-04-spec7-run-manifest.md` (Label-Based Run Management rewrite)
**Round**: 1

---

### Reviewer 1: Ce Zhang (ETH Zurich — Systems, ML Infrastructure)

**Score**: 9/10 — The design is substantially cleaner than the previous version.

Replacing path-as-API with label-as-API is the correct abstraction. The three-directory convention (`runs/`, `analysis/`, `reports/`) is fixed, predictable, and machine-verifiable. Every command that touches files can validate its inputs by checking the convention rather than trusting a user-supplied string. This is a meaningful reliability improvement over the manifest-runner approach.

The registry invariants (§6.2) are well-specified. Enforcing that `analyzed` is a subset of `built` at write time — not at read time — means the invariant cannot be violated by concurrent writes, provided writes are atomic. The atomic write protocol (`.tmp` + rename) in §8.5 is correct for single-process use. For multi-process use (e.g., a CI system running two `analyze` commands concurrently), the rename is not sufficient. Add a note that `.redist` writes use advisory file locking (`flock` on Linux, `LockFile` on Windows) to serialize concurrent updates.

The `test_registry_write_is_atomic` test is listed but its assertion is underspecified. "No partial write" is not a testable predicate without injecting a failure between the write and the rename. The test should simulate a kill between those steps (by wrapping the write in a closure that panics after the `.tmp` write) and verify that the pre-existing `.redist` is intact.

One gap: the spec does not address the case where `runs/official_proposal/` exists on disk but `official_proposal` is not in `.redist` (e.g., the registry was deleted or the user ran `redist states` directly). `redist ls` should detect this and offer a recovery path: `[WARN] Found runs/official_proposal/ not in registry. Run: redist registry sync to import.`

---

### Reviewer 2: Nadia Polikarpova (UC San Diego — Formal Methods, Program Synthesis)

**Score**: 8.5/10 — Much stronger than the prior draft. Two schema concerns remain.

The separation of config (what to do), registry (what was done), and index (how it went) is the right decomposition. The previous spec's dual-use of a single YAML file as both specification and execution state was a correctness hazard; this spec eliminates it.

The config schema (§7.1) now specifies `deny_unknown_fields` behavior via the rule "unknown keys cause a parse error" — this closes the silent-misconfiguration bug I flagged previously. The required-vs-optional field rules in §7.2 are stated normatively and testable. Both are improvements.

Two remaining concerns. First, the analysis index (§5.2) has `"run": "official_proposal"` — a name reference to the build label. The spec says "the path is derived from the label by convention." This is fine when the label is the same for both run and analyze. But `redist analyze X --out /mnt/custom/` (the escape hatch in §3.9) breaks the convention: the analysis is now in a non-standard location but the index still says `"run": "official_proposal"` with no path override recorded. Either the escape hatch must also write a `run_dir` field to the index, or the escape hatch should be restricted to `report` (where no downstream commands read the output directory from the index). Leaving this inconsistency makes the index schema unreliable for auditors.

Second, the `BuildArgs` struct (§8.6) has a `group = "label_source"` that accepts the label as either a positional argument or `--label`. This pattern is underspecified: what happens when both are supplied? Clap's `group` makes them mutually exclusive, which is correct, but the error message will be Clap's default rather than a user-friendly `[CONFIG]` message. Add a test for this case.

---

### Reviewer 3: Percy Liang (Stanford — Empirical ML, Reproducibility)

**Score**: 9.5/10 — This is the right reproducibility primitive for redistricting work.

The previous spec's `meta.git_commit` field was good but buried in the YAML execution state. This spec puts `git_commit` in every index — build, analysis, report — which means any index file is independently verifiable against the source repository. That is the correct design for a system where outputs are shared with auditors who did not run the pipeline themselves.

The `config_sha256` field in the build index (§5.1) is the critical addition I would have requested. It means an auditor can take `configs/official_proposal.yml` from the git repository, hash it, and verify against the build index that the committed config was the one actually used. This closes the "was the config modified between commit and run?" gap.

One improvement: the analysis index records `run_index_sha256` (the SHA of `runs/X/index.json`). But `runs/X/index.json` itself references `config_sha256`. A full audit chain is therefore: report index → analysis index → build index → config SHA → committed config. This chain should be documented explicitly in §5 as the "audit chain" so special masters know how to traverse it. Without documentation, auditors will not know to follow the chain.

The `analysis_types` field in the official proposal config (§9.1) now includes `contiguity` and `splits` — a concrete improvement over the previous draft that omitted these legally significant outputs. The inclusion of both in the reference config is the right default.

---

### Reviewer 4: Moon Duchin (Tufts — Mathematics, Redistricting Practice)

**Score**: 9/10 — Practitioner workflow is now first-class.

The label system is the right mental model for redistricting practitioners. In every engagement I work on, plans have names — `draft1`, `commission_proposal`, `court_submission`. The previous system required translating names into paths; this system treats names as the fundamental unit and derives everything else. The cognitive load reduction is real and significant.

The error messages in §9.3 are exemplary. "You have not built year 2010. Run: redist build official_proposal --year 2010" is exactly what a practitioner needs. It is actionable, specific, and does not require reading documentation. The label-collision message showing the timestamp of the existing build is equally good.

One gap: the `redist ls` output (§3.5) shows which years are in each stage but not how many states succeeded. A commission director checking in on a long run wants to know "50 states done or 47?" before looking at the full `redist show`. Suggest adding a state count column to `redist ls`, or at minimum a "partial" indicator when `succeeded < total states expected`:

```
Label                Built               Analyzed   Reported
official_proposal    2020(50) 2010(50)   2020        —
senate_draft2        2020(47/50 partial) —           —
```

The `redist rm` command (§3.7) correctly cascades — deleting `analyze` also deletes `report`. The cascade direction (removing a stage also removes all dependent stages) should be stated as a rule in §3.7, not just implied by the examples. Practitioners will want to know: does `redist rm official_proposal --stage build` also delete the analysis and report, or just the build outputs? The spec implies yes but does not state it.

---

### Reviewer 5: Dana Hendricks (Wisconsin Legislative Technology Office — State GIS Director)

**Score**: 8/10 — Clear improvement. Three workflow gaps from actual use.

I manage redistricting data for the Wisconsin Legislature. We receive proposed plans from multiple parties — the commission, minority caucus, advocacy groups — and need to analyze each one and compare them side-by-side. The label system directly matches how we think about plans. "Commission_v3 versus minority_proposal_2" is how we talk in meetings, not "outputs/v3/2020 versus outputs/minority/2020."

Three gaps from actual use. First: we often receive a plan as a shapefile or CSV from an outside party, not as a set of tract assignments produced by `redist build`. We need `redist import minority_proposal --from minority_proposal.csv --year 2020` that creates `runs/minority_proposal/` from an external file. The current spec assumes all runs are produced by `redist build`. Adding an `import` verb to the label system would make it usable for the adversarial-plan-review workflow, which is the most common redistricting scenario in state legislatures.

Second: we share outputs with legal staff and commissioners who do not run the pipeline themselves. They need to browse `reports/official_proposal/` and find a dashboard they can open in a browser. The current directory structure has `reports/official_proposal/dashboard_2020.html` — good — but there is no `reports/official_proposal/index.html` that links to all years. `redist report X` should generate a top-level index page alongside the per-year dashboards.

Third: the `redist rm` cascade (deleting build deletes analysis and report) is dangerous without an explicit confirmation step. Add `--dry-run` to `redist rm` so staff can preview what will be deleted before confirming. The `--force` flag that skips confirmation should require typing the label name, not just passing a flag: `redist rm official_proposal --force official_proposal`. This pattern (confirming by retyping the label) is used by Heroku and prevents the most common accidental-deletion scenario.
