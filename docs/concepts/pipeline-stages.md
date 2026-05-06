# Pipeline Stages

## Short version

Four stages run in sequence: build → analyze → report → verify. Each stage records a SHA-256 hash of the previous stage's output, forming a tamper-evident audit chain suitable for court submission. The `redist` binary manages all stages; no Python orchestration is required.

---

## The four stages

```
[1] Build     Run METIS recursive bisection for all states; write final_assignments.json
     ↓
[2] Analyze   Compute compactness, VRA, partisan, contiguity, and splits metrics
     ↓
[3] Report    Generate HTML, JSON, and PDF reports from analysis outputs
     ↓
[4] Verify    Traverse the SHA-256 chain (config → build → analysis → report)
```

---

## Stage 1: Build

**Command**: `redist build <label>`

Reads `configs/<label>.yml`, which defines the structure mode, edge weights, search parameters, and population tolerance for the named plan. Runs METIS recursive bisection for all 50 states (or a subset specified with `--states`) and writes results to `runs/<label>/<year>/<state>/`.

Key outputs per state:

- `final_assignments.json` — tract-to-district mapping
- `bisection_tree.json` — round-by-round split hierarchy
- `population_balance.json` — deviation from ideal per district

On completion, a SHA-256 hash of the config file is recorded in `runs/<label>/index.json`. This anchors the audit chain: any subsequent change to the config file will break chain verification.

**Resumability**: `redist build` skips states that already have a valid `final_assignments.json`. Re-run a specific state without disturbing others:

```bash
redist build my_plan --states NC
```

Force a complete rebuild of all states:

```bash
redist build my_plan --force
```

**Parallel execution**: By default, `redist build` runs all states concurrently using Rayon. Set `--workers` to limit parallelism:

```bash
redist build my_plan --workers 8
```

California (~8,000 tracts, 52 districts) is the longest-running state (3–8 minutes). Wyoming (~130 tracts, 1 district) completes in seconds.

### The three-layer compositor

Before the build executes, the YAML config is resolved through three layers that control what METIS sees:

| Layer | Key | Effect |
|-------|-----|--------|
| **Structure** | `structure` | Selects the geographic partitioning mode: `default`, `geo-section`, `area-section`, or `apportion-regions`. Determines how the bisection tree is seeded and how regions are grouped before METIS runs. |
| **Weights** | `weights_override` | Overrides default edge weights. Edge weight = shared boundary length by default; overrides can scale by census tract area, population density, or custom factors. Minimizing cut weight drives compactness. |
| **Search** | `search` | Controls METIS solver parameters: `ufactor` (population imbalance tolerance), random seed, and number of restarts. Also governs post-partitioning district contiguity checks. |

Structure modes:

- `default` — standard recursive bisection, no pre-grouping
- `geo-section` (`--structure geo-section`) — groups tracts into geographic sections (e.g., coastal vs. inland) before bisection, improving regional coherence
- `area-section` (`--structure area-section`) — sections based on land area rather than population; useful for sparse western states
- `apportion-regions` (`--structure apportion-regions`) — pre-assigns tracts to named apportionment regions before running bisection within each region

Example config (`configs/nc_2020.yml`):

```yaml
year: 2020
states: [NC]
structure: geo-section
weights_override:
  boundary_scale: 1.0
  area_scale: 0.0
search:
  ufactor: 5
  seed: 42
  restarts: 3
```

---

## Stage 2: Analyze

**Command**: `redist label-analyze <label> --types all`

Reads `runs/<label>/index.json` (including its SHA-256 of the build config) and the per-state assignment files. Computes metrics across five dimensions and writes results to `analysis/<label>/<year>/`.

Metric types (pass any combination to `--types`):

| Type | Description |
|------|-------------|
| `compactness` | Polsby-Popper and Reock scores per district |
| `vra` | VRA § 2 minority opportunity districts; requires demographic data |
| `partisan` | Partisan lean per district using election data (2020 only) |
| `contiguity` | Verifies every district forms a single connected component |
| `splits` | County and municipal boundary splits per district |
| `all` | All of the above |

On completion, the SHA-256 of `runs/<label>/index.json` is recorded in `analysis/<label>/index.json`. If the build outputs were modified after the fact, this hash will not match during verification.

```bash
# Run all metric types
redist label-analyze nc_2020 --types all

# Run only compactness and contiguity
redist label-analyze nc_2020 --types compactness contiguity
```

---

## Stage 3: Report

**Command**: `redist label-report <label> --format html json pdf`

Reads `analysis/<label>/index.json` and renders reports. Outputs land in `reports/<label>/<year>/`.

Available formats:

| Format | Output | Use |
|--------|--------|-----|
| `html` | Interactive dashboard | Browser viewing, internal review |
| `json` | Machine-readable summary | Downstream tooling, CI assertions |
| `pdf` | Static typeset document | Court submission, public record |

The PDF format requires Typst to be installed; see `redist-report/typst-templates/README.md` for setup. The HTML format is always available.

```bash
# Generate all formats
redist label-report nc_2020 --format html json pdf

# HTML only (no Typst required)
redist label-report nc_2020 --format html
```

For court-submission reports, additional flags supply required metadata:

```bash
redist label-report nc_2020 --format pdf \
  --expert-name "Dr. Jane Smith" \
  --jurisdiction "North Carolina" \
  --citation-style bluebook
```

---

## Stage 4: Verify

**Command**: `redist label-verify <label>`

Traverses the full SHA-256 chain from config through build, analysis, and report. Reports pass/fail for each link:

```
config hash     OK   (sha256: a3f7...)
build index     OK   (sha256: 9c12...)
analysis index  OK   (sha256: 4e80...)
report manifest OK   (sha256: 7d3a...)
Chain: VALID
```

Any modification to pipeline outputs — even a single byte — breaks the corresponding link and produces a `TAMPERED` result. This makes the chain suitable for evidentiary use: a passing verification attests that the reported outputs are exactly those produced by the declared config.

```bash
redist label-verify nc_2020
```

---

## The audit trail

Each stage writes its output and records a hash of the previous stage's output index. The resulting chain looks like:

```
configs/nc_2020.yml
      ↓ sha256 recorded in
runs/nc_2020/index.json
      ↓ sha256 recorded in
analysis/nc_2020/index.json
      ↓ sha256 recorded in
reports/nc_2020/manifest.json
```

This design means:

- **Reproducibility**: given the config file, anyone with the same census data can reproduce the run and verify the chain matches.
- **Tamper evidence**: post-hoc edits to district assignments or metric outputs break the chain at the modified stage.
- **Court admissibility**: `redist label-verify` provides a single command that an opposing expert or special master can run to confirm outputs have not been altered since generation.

---

## Running the full pipeline

```bash
# Full pipeline for a named plan, all stages
redist build nc_2020 --workers 8
redist label-analyze nc_2020 --types all
redist label-report nc_2020 --format html json pdf
redist label-verify nc_2020
```

For rapid iteration during research, omit the report and verify steps until results are ready for submission.

---

## Further reading

- `docs/REDIST_CLI.md` — complete CLI reference (all commands, flags, env vars)
- `docs/concepts/recursive-bisection.md` — algorithm detail
- `docs/concepts/census-data.md` — data sources and structure
- `CLAUDE.md` — quick reference for all run commands
