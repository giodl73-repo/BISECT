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

### 2.5 Cache invalidation

**Cache invalidation**: When `redist build X --year Y --force` overwrites an
existing build, the system MUST:
1. Delete `analysis/X/Y/` (analysis outputs are stale)
2. Delete `reports/X/Y/` (reports are stale)
3. Remove year Y from `analyzed` and `reported` lists in `.redist` registry
4. Write a `runs/X/Y/STALE_ANALYSIS` sentinel file explaining why analysis was cleared

This cascade prevents silent stale data. The operator is warned:
```
[WARNING] Overwriting build for official_proposal/2020.
          Clearing stale analysis (2020/wisconsin/proportionality.json + 49 others).
          Run 'redist analyze official_proposal --year 2020' to re-analyze.
```

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

### 3.1a `redist import X --from FILE` — create a label from an external plan file

```bash
redist import commission_v3 --from commission_v3.csv --year 2020
redist import minority_proposal --from minority_proposal.shp --year 2020 --format shapefile
redist import civic_geojson --from civic_input.geojson --year 2020 --format geojson
redist import partner_plan --from partner.rplan --year 2020 --format rplan
```

Creates `runs/X/` from an externally-produced plan file. Supported input formats:
- `csv`: GEOID,district columns (standard state-legislative exchange format)
- `geojson`: FeatureCollection with `district_id` property
- `shapefile`: `.shp` + `.dbf` with GEOID and DISTRICT columns
- `rplan`: native redist format (Spec 1)

The imported run has `"algorithm": {"structure": "external", "source": FILE_SHA256}`.
It appears in `redist ls` as a label with `built: [Y]` but `algorithm: external`.
All `analyze`, `report`, and `compare` commands work identically on imported plans.

```bash
redist import commission_v3 --from commission_v3.csv --year 2020
redist analyze commission_v3 --year 2020
redist compare official_proposal commission_v3 --year 2020
```

This enables the adversarial plan review workflow used by state legislative staff.

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

Machine-readable output for CI:

```bash
redist ls --json
# {"official_proposal": {"built": ["2020","2010","2000"], "analyzed": ["2020"], "reported": []}, ...}
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

Machine-readable output for CI:

```bash
redist show official_proposal --json   # full index.json content + registry entry
```

**CI/CD flag**: All `redist` commands accept `--no-interactive` to disable
confirmation prompts for use in CI/CD pipelines. `--force` skips conflict checks;
`--no-interactive` skips confirmation prompts. These flags are orthogonal:
- `--force` — bypass guards (allow overwrite, allow cascade delete)
- `--no-interactive` — suppress prompts (fail on ambiguity rather than asking)

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

`redist label X Y` is a registry-only alias: it does NOT rename directories or
update index files. Use `redist mv X Y` (below) for a full rename.

### 3.8a `redist mv X Y` — rename a label

Renames label X to Y with full filesystem and registry consistency:
1. Moves `runs/X/` → `runs/Y/` (filesystem rename)
2. Updates `runs/Y/index.json`: sets `"label": "Y"`
3. Moves `analysis/X/` → `analysis/Y/` if it exists; updates `analysis/Y/index.json`
4. Moves `reports/X/` → `reports/Y/` if it exists
5. Updates registry: removes X entry, adds Y entry with same stage data
6. Errors if Y already exists in registry (use `--force` to overwrite)

```bash
redist mv senate_draft2 senate_final
redist mv senate_draft2 senate_final --force   # overwrite if Y exists
```

Distinction:
```
redist label X Y  # alias only — does NOT rename directories or update index.json
redist mv X Y     # full rename — directories + indexes + registry
```

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

**`--out` on `redist build` is NOT supported.** Build always writes to `runs/X/`.
This is intentional: allowing non-conventional paths would break the implicit
contract that `analyze X` reads from `runs/X/`. Power users who need non-standard
paths should use `redist mv X Y` after a build to relocate outputs while
maintaining registry and index consistency.

**`--out` on `redist report` IS supported**, since reports are terminal artifacts
not read by any downstream command:

```bash
# Write report to a custom directory
redist report official_proposal --year 2020 --out /mnt/share/reports/

# Show all resolved paths without running
redist show official_proposal
```

The `BuildArgs` struct (§8.6) retains the `--out` field declaration for forward
compatibility but the build dispatcher rejects it at runtime with:
```
[CONFIG] build: --out is not supported on 'redist build'. Build always writes
         to runs/{label}/. Use 'redist mv X Y' to relocate outputs after building.
```

### 3.11 `redist verify X` — traverse the SHA chain for a label

```bash
redist verify official_proposal --year 2020
```

Output (all links intact):
```
Config:   configs/official_proposal.yml [sha256: abc123] OK MATCH
Build:    runs/official_proposal/2020/index.json [config_sha256: abc123] OK CHAIN OK
Analysis: analysis/official_proposal/2020/index.json [run_sha256: def456] OK CHAIN OK
Report:   reports/official_proposal/2020/index.json [analysis_sha256: xyz789] OK CHAIN OK

Verdict: official_proposal/2020 is VERIFIED -- unbroken SHA chain from config to report.
```

Output (broken chain):
```
Build: runs/official_proposal/2020/index.json [config_sha256: abc123] FAIL MISMATCH
       (stored: abc123, actual: fed321)
Verdict: FAILED -- config may have been modified after build.
```

`redist verify` is the mechanical tool for courts, special masters, and auditors
to confirm the algorithm was run as specified without post-hoc modification. It
traverses the four-link chain: config file → build index → analysis index → report
index, verifying each SHA reference.

Note: `redist verify` uses ASCII-only output (`OK`/`FAIL`) to comply with the
Windows CP1252 console policy (PP-34).

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

**Concurrency invariant**: All mutations to `.redist` MUST acquire an advisory
file lock on `.redist.lock` before loading the registry, and release it after the
atomic rename. The `registry.rs` implementation uses `fs2::FileExt::lock_exclusive`
(cross-platform) before every load-modify-save cycle. Read-only operations
(`ls`, `show`) use `lock_shared`. This prevents the lost-update race when two
parallel year builds both call `mark_built` — without locking, the second writer's
rename would silently discard the first writer's mark.

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
  registry.rs         ← .redist read/write, atomic update, file locking, invariant enforcement
  build_cmd.rs        ← `redist build` dispatcher (replaces/wraps states runner)
  import_cmd.rs       ← `redist import` — ingest external plan files into label system
  mv_cmd.rs           ← `redist mv` — atomic rename of label (dirs + indexes + registry)
  verify_cmd.rs       ← `redist verify` — traverse and validate SHA chain
  config_cmd.rs       ← `redist config new/validate`
  ls_cmd.rs           ← `redist ls` and `redist show` (with --json support)
  rm_cmd.rs           ← `redist rm`

configs/
  official_proposal.yml   ← committed algorithm config (this spec's reference)
```

### 8.2 Modified files

```
redist/crates/redist-cli/src/
  args.rs     ← add Build, Ls, Show, Rm, Label, Mv, Import, Verify, ConfigNew, ConfigValidate variants
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
/// List all labels and their stage completion (--json for machine-readable output)
Ls(LsArgs),
/// Show details for a label (--json for machine-readable output)
Show(ShowArgs),
/// Delete a stage for a label
Rm(RmArgs),
/// Copy a label (registry alias only — does not rename directories)
Label(LabelArgs),
/// Rename a label atomically (directories + indexes + registry)
Mv(MvArgs),
/// Import an external plan file into the label system
Import(ImportArgs),
/// Traverse and verify the SHA chain for a label
Verify(VerifyArgs),
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

All mutating functions MUST acquire an exclusive advisory lock on `.redist.lock`
before loading the registry and release it after the atomic rename. Read-only
functions acquire a shared lock. Uses `fs2::FileExt` (cross-platform).

```rust
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;

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
/// Acquires shared lock on .redist.lock for the duration of the read.
pub fn load() -> Result<Registry, String> { ... }

/// Write .redist atomically (write to .redist.tmp, rename).
/// Caller must hold exclusive lock (acquired via with_exclusive_lock).
pub fn save(registry: &Registry) -> Result<(), String> { ... }

/// Acquire exclusive lock, run f(registry) -> modify -> save atomically.
/// All mutating public functions below call this internally.
pub fn with_exclusive_lock<F>(f: F) -> Result<(), String>
where
    F: FnOnce(&mut Registry) -> Result<(), String>,
{
    let lock_file = OpenOptions::new().create(true).write(true)
        .open(".redist.lock")?;
    lock_file.lock_exclusive()?;
    let mut reg = load_unlocked()?;
    f(&mut reg)?;
    save(&reg)?;
    lock_file.unlock()?;
    Ok(())
}

/// Mark a year as built for a label. Acquires exclusive lock. Enforces invariants.
pub fn mark_built(label: &str, year: &str) -> Result<(), String> {
    with_exclusive_lock(|reg| { /* add year to built list */ })
}

/// Mark a year as analyzed. Acquires exclusive lock. Returns Err if year not in built.
pub fn mark_analyzed(label: &str, year: &str) -> Result<(), String> {
    with_exclusive_lock(|reg| { /* verify built, add to analyzed */ })
}

/// Mark a year as reported. Acquires exclusive lock. Returns Err if year not in analyzed.
pub fn mark_reported(label: &str, year: &str) -> Result<(), String> {
    with_exclusive_lock(|reg| { /* verify analyzed, add to reported */ })
}

/// Require that a year has been built. Used by analyze for pre-flight checks.
/// Acquires shared lock.
pub fn require_built(label: &str, year: &str) -> Result<(), String> { ... }

/// Require that a year has been analyzed. Used by report for pre-flight checks.
/// Acquires shared lock.
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
| `test_registry_concurrent_mark_built_no_lost_update` | two concurrent `mark_built` calls both appear in registry |
| `test_registry_exclusive_lock_blocks_concurrent_write` | second writer waits for first to release lock |
| `test_mv_renames_dirs_and_updates_index_label` | `mv X Y` → `runs/Y/index.json` has `"label": "Y"` |
| `test_mv_errors_if_target_exists` | `mv X Y` where Y exists → `[CONFIG]` error |
| `test_mv_force_overwrites_target` | `mv X Y --force` succeeds when Y exists |
| `test_import_csv_creates_runs_dir` | `import X --from f.csv` → `runs/X/index.json` with `algorithm.structure: external` |
| `test_import_sets_source_sha256` | imported plan records SHA of source file in `algorithm.source` |
| `test_import_appears_in_ls_as_external` | `redist ls` shows imported label with `algorithm: external` |
| `test_verify_unbroken_chain_prints_ok` | all SHAs match → prints `VERIFIED` |
| `test_verify_broken_config_sha_prints_fail` | config modified after build → `FAILED` |
| `test_verify_missing_report_reports_missing` | report not generated → `MISSING` for report link |
| `test_ls_json_output_is_valid_json` | `redist ls --json` → parseable JSON matching registry |
| `test_show_json_output_contains_index` | `redist show X --json` → includes `index.json` content |
| `test_build_force_clears_stale_analysis` | `build X --force` deletes `analysis/X/Y/` and removes from registry |
| `test_build_force_writes_stale_sentinel` | `runs/X/Y/STALE_ANALYSIS` file written on forced rebuild |
| `test_build_out_flag_rejected` | `build X --out /tmp/` → `[CONFIG]` error |

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
| **State Staff Interop** | `redist import X --from FILE` is the ingestion path for externally-submitted plans; integrates with `redist compare` for adversarial review |
| **Court Submission Reports** | `redist verify X` provides the mechanical SHA-chain traversal cited in expert witness reports |
| **Civic Bidirectional Input** | `redist import X --from FILE --format csv` is the CSV ingestion path compatible with civic ingest |
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
