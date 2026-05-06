# The Label-Based Pipeline

## Short version

A label is a short name — like `official_2020` or `nc_areasection` — that
resolves to all build, analysis, and report paths by convention. You never
specify output paths manually. You name a run, write a config, and every
command that touches that run uses the label. The label also anchors the
SHA-256 audit chain: each stage hashes the previous stage's output and records
it, so tampering at any stage breaks the chain.

---

## What is a label?

A label is a human-readable identifier for a complete redistricting run. It
captures the full pipeline from algorithm configuration through final reports.
Labels are stored in the `.redist` registry file at the root of the project.

Labels follow no enforced naming convention, but the project uses these
patterns by practice:

| Pattern | Example | Meaning |
|---|---|---|
| `official_{year}` | `official_2020` | Production run for a census year |
| `{state}_{algorithm}` | `nc_areasection` | State-specific algorithm experiment |
| `{paper}_{variant}` | `b11_prime_factor` | Paper-specific variant run |
| `civic_{source}` | `civic_charlotte_2024` | Imported civic counter-proposal |

---

## Directory layout

Every label maps to a fixed directory structure under the project root. No
path arguments are required or accepted in label-based commands.

```
configs/{label}.yml              ← algorithm config (human-edited, committed)

runs/{label}/{year}/{state}/     ← build outputs
  final_assignments.json         ← tract-to-district mapping
  index.json                     ← build metadata + SHA chain link

analysis/{label}/{year}/         ← analysis outputs
  {state}_summary.json
  {state}_compactness.json
  {state}_splits.json
  ...

reports/{label}/{year}/          ← report outputs
  {state}_report.html
  {state}_report.json
  manifest.json                  ← report-level SHA chain terminus
```

For multi-year runs (the default), each census year gets its own subdirectory
under `runs/{label}/` and `analysis/{label}/`. The `reports/{label}/` output
summarises all years together.

---

## The SHA-256 audit chain

The label pipeline implements a four-stage audit chain. Each stage hashes the
primary output of the previous stage and embeds that hash in its own output
file. Verifying the chain requires only the final manifest; the verifier can
reconstruct every intermediate hash.

```
configs/{label}.yml
       |
       | SHA-256 of config file contents
       v
runs/{label}/{year}/{state}/index.json
  "config_sha256": "a3f7..."
       |
       | SHA-256 of final_assignments.json
       v
analysis/{label}/{year}/{state}_summary.json
  "assignments_sha256": "b9c2..."
       |
       | SHA-256 of analysis output
       v
reports/{label}/{year}/manifest.json
  "analysis_sha256": "d4e1..."
```

If any file is modified after the fact — a common concern in election
litigation — the hash comparison fails at the corresponding stage. The chain
provides a tamper-evident record from algorithm configuration to final report.

The `redist label-verify` command traverses the chain automatically and reports
the status of each link.

---

## Core verbs

### `redist build`

```bash
redist build official_2020
redist build official_2020 --year 2020
redist build official_2020 --year 2020 --states NC TX GA
```

Draws districts according to `configs/official_2020.yml`. Writes
`final_assignments.json` (tract-to-district mapping) and `index.json`
(build metadata including config SHA, build timestamp, and binary version) to
`runs/official_2020/{year}/{state}/`.

If `--states` is not specified, all 50 states are built in parallel using the
worker count from the config.

### `redist label-analyze`

```bash
redist label-analyze official_2020
redist label-analyze official_2020 --year 2020 --types compactness,splits
```

Runs the specified analysis types on all built states for the label. Reads
`final_assignments.json` from `runs/`, writes analysis JSON files to
`analysis/official_2020/{year}/`.

Available analysis types: `demographic`, `political`, `compactness`,
`contiguity`, `splits`, `summary`, `all`.

Each analysis output file embeds the SHA-256 of the `final_assignments.json`
it was computed from. This is the assignments link in the audit chain.

### `redist label-report`

```bash
redist label-report official_2020
redist label-report official_2020 --format html
redist label-report official_2020 --format pdf --expert-name "Jane Smith"
```

Generates HTML, JSON, or PDF reports from the analysis outputs. Reads from
`analysis/official_2020/{year}/`, writes to `reports/official_2020/{year}/`.

The `--format pdf` path requires Typst; see
`redist/docs/typst-templates/README.md` for installation. For court submissions,
the PDF format produces a report that satisfies the requirements of the
`redist-report` crate's civic gate (BD-R1).

### `redist label-verify`

```bash
redist label-verify official_2020
redist label-verify official_2020 --year 2020
```

Traverses the SHA-256 chain for the label. For each state and year, checks:

1. The config file matches the SHA in `index.json`.
2. The `final_assignments.json` matches the SHA in the analysis output.
3. The analysis output matches the SHA in the report manifest.

Reports one of three statuses per link:

| Status | Meaning |
|---|---|
| `VERIFIED` | Hash matches. File was not modified after the stage ran. |
| `MISMATCH` | Hash does not match. File was modified after the stage ran. |
| `MISSING` | Expected file does not exist. Stage has not been run. |

Any `MISMATCH` result is a tamper-evidence failure and should be investigated
before the plan is used in a legal proceeding.

### `redist ls`

```bash
redist ls
redist ls --year 2020
```

Lists all labels registered in `.redist`, with a one-line status summary
for each: which stages have been completed, how many states are built, and
whether any MISMATCH conditions exist.

```
LABEL               CONFIG  BUILD         ANALYSIS      REPORTS
official_2020       OK      50/50 states  50/50 states  50/50 states
nc_areasection      OK      1/1 states    1/1 states    -
b11_pilot           OK      12/50 states  -             -
```

### `redist show`

```bash
redist show official_2020
redist show official_2020 --year 2020
```

Detailed status for one label. Shows the config algorithm parameters, resolved
paths for each year, per-state build status, SHA chain integrity summary, and
timing information.

### `redist mv`

```bash
redist mv nc_draft nc_official
```

Atomically renames a label. Updates:
- The `.redist` registry entry
- All filesystem directories under `runs/`, `analysis/`, and `reports/`
- The `name` field in `configs/{label}.yml`
- The `label` field in every `index.json` under the old label path

This is an atomic operation with rollback: if any step fails, the rename
is reversed.

### `redist label-import`

```bash
redist label-import nc_enacted --from nc_2022_enacted.csv --year 2020 \
  --format csv --submitted-by "NC Legislature"
```

Imports an external plan (CSV, GeoJSON, or Shapefile) into the label-based
directory layout without running the redistricting algorithm. The imported
plan receives the same `index.json` structure as a built plan, with a
`submission_type: civic-counter-proposal` field.

The import command runs the same contiguity and population-balance checks
as the build command. A plan that fails contiguity or violates the 0.5%
population tolerance will be rejected at import time.

### `redist label-compare`

```bash
redist label-compare official_2020 nc_enacted --year 2020
redist label-compare official_2020 nc_enacted --year 2020 --json
```

Side-by-side diff of two label-based plans. Compares:
- Jaccard similarity between districts (overlap of tract assignments)
- Mean and median Polsby-Popper for each plan
- Number of county splits for each plan
- Partisan lean distribution (if analysis has been run)

The `--json` flag writes machine-readable output suitable for piping into
analysis scripts.

---

## Config YAML format

Every label requires a `configs/{label}.yml` file. This is the human-edited,
version-controlled source of truth for the algorithm configuration. See
[three-layer-compositor.md](three-layer-compositor.md) for a complete field
reference.

```yaml
# configs/official_2020.yml
name: official_2020
description: "Production run — ApportionRegions, geographic weights, convergence T=600"

algorithm:
  structure: prime-factor        # Layer 1: bisection tree topology
  weights: geographic            # Layer 2: METIS edge signal
  search: convergence            # Layer 3: seed search strategy
  convergence_threshold: 600     # required when search: convergence
  balance_tolerance: 0.5         # population tolerance in percent
  engine: c-ffi                  # METIS backend

workers: 6                       # parallel workers per census year
years: ["2020", "2010", "2000"]  # census years to include in the build
analysis_types:
  - compactness
  - splits
  - summary
```

The `name` field must match the filename: `configs/official_2020.yml` must
have `name: official_2020`. The `redist config` command validates this and
will reject a config with a mismatched name.

---

## Creating a new label

```bash
# 1. Create and validate the config
redist config create my_plan --structure ratio-optimal --weights geographic \
  --search convergence --convergence-threshold 600

# 2. Build for all years
redist build my_plan

# 3. Analyze
redist label-analyze my_plan --types compactness,splits,summary

# 4. Generate reports
redist label-report my_plan --format html

# 5. Verify the audit chain
redist label-verify my_plan
```

---

## Further reading

- [three-layer-compositor.md](three-layer-compositor.md) — algorithm configuration
- [section-algorithms.md](section-algorithms.md) — algorithm family reference
- `docs/REDIST_CLI.md` — complete CLI flag reference
- `docs/superpowers/specs/` — label pipeline design specifications
