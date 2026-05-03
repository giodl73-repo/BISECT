# `redist` CLI Reference

The `redist` binary is a Rust rewrite of the Python redistricting pipeline. It is
~200× faster than the Python equivalent and has no Python runtime dependency for
the core redistricting work.

**Built from**: `redist/crates/redist-cli/`

---

## Installation

```bash
# Build the release binary (one-time)
cargo build --release --manifest-path redist/Cargo.toml

# The binary lands at:
#   redist/target/release/redist        (Linux/macOS)
#   redist/target/release/redist.exe    (Windows)

# Optional: add to PATH so you can run `redist` from anywhere
# Windows PowerShell:
$env:PATH += ";$PWD\redist\target\release"
```

**Prerequisites**: `gpmetis` must be on `PATH`. METIS ships via conda:
```bash
conda install -c conda-forge metis
```

---

## Quick Start

```bash
# Download Vermont data (small state, good for testing)
redist fetch --states VT --year 2020

# Run Vermont redistricting
redist state --state VT --year 2020 --version V3

# Run all 50 states (8 workers, ~15 seconds with .adj.bin files)
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8
```

---

## Commands

### `redist state` — Single state

Runs redistricting for one state and writes outputs to `outputs/{version}/`.

```
redist state --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | Two-letter state code: `VT`, `CA`, `TX`, … |
| `-y`, `--year` | `2020` | Census year: `2020`, `2010`, `2000` |
| `-v`, `--version` | `v1` | Version tag (used in output path) |
| `--output-dir` | `outputs/{version}` | Override output root directory |
| `-m`, `--partition-mode` | `edge-weighted` | `unweighted`, `edge-weighted`, `metis-vra`, `partisan-weighted`, `geosection`, `areasection`, `compact-bisect`, `prime-factor` |
| `--partisan-shares` | *(none)* | TSV file with `geoid<TAB>dem_share`. Required for `partisan-weighted` mode. **Mutually exclusive with `metis-vra`** (Callais p.36 disentanglement). |
| `--dem-threshold` | `0.55` | dem_share ≥ this → unit is "strong-D". Partisan-weighted mode only. |
| `--rep-threshold` | `0.45` | dem_share ≤ this → unit is "strong-R". Partisan-weighted mode only. |
| `--ufactor` | `5` | METIS imbalance tolerance (5 = ±0.5%) |
| `--niter` | `100` | METIS refinement iterations |
| `--seed` | *(random)* | METIS random seed for reproducible runs |
| `--geosection-seeds` | `50` | Seeds per ratio for GeoSection (B.8) and AreaSection (B.9). Each ratio 1:k-1 through k/2:k/2 is tried with this many seeds; the minimum-normalised-EC ratio wins. Typical: 50–200. |
| `--area-swing` | `1.10` | **AreaSection only.** METIS `ubvec[1]` for the land-area balance constraint: `1.10` = ±10% swing from 50/50. Empirically validated range: `1.05`–`1.15`. Below `1.05` forces near-equal splits for concentrated states (GA → 7:7). Above `1.15` allows urban peeling to return (WI → 2:6 at `1.20`). |
| `--balance-tolerance` | `0.5` | Max deviation per district in percent (0.5 = ±0.5%, the congressional one-person-one-vote standard). Pass as percent, not fraction. |
| `-r`, `--reset` | false | Delete output directory before starting |
| `-d`, `--debug` | false | Extra diagnostic output |
| `-p`, `--print-only` | false | Dry run — print actions without executing |

**Examples**:

```bash
# Vermont (1 district) — fast smoke test
redist state --state VT --year 2020 --version V3

# Alabama with VRA mode (7 districts)
redist state --state AL --year 2020 --version V3 --partition-mode metis-vra

# Partisan-weighted bisection (Plan 03; Callais 2026-04-29)
# Preserves D-D and R-R clusters via edge-weighting; uses recursive bisection.
redist state --state LA --year 2020 --version V3 \
    --partition-mode partisan-weighted \
    --partisan-shares outputs/data/2020/partisan/la/dem_shares.tsv

# Reproducible run with fixed seed
redist state --state CA --year 2020 --version V3 --seed 42

# Fresh re-run (delete previous output first)
redist state --state TX --year 2020 --version V3 --reset
```

**Output files** (under `outputs/{version}/{year}/{state_name}/`):
- `final_assignments.json` — tract → district mapping
- `vra_analysis.json` — majority-minority district analysis (VRA mode only)

---

### `redist states` — All states in parallel

Runs all 50 states (or a subset) using Rayon for parallelism.

```
redist states --year <YEAR> --version <VERSION> --output-dir <DIR> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-y`, `--year` | *(required)* | Census year: `2020`, `2010`, `2000` |
| `-v`, `--version` | *(required)* | Version tag |
| `--output-dir` | *(required)* | Output root directory |
| `-w`, `--workers` | `4` | Parallel workers (match CPU core count) |
| `--states <CODES>` | *(all 50)* | Space-separated state codes to include |
| `-m`, `--partition-mode` | `edge-weighted` | `unweighted`, `edge-weighted`, `metis-vra`, `geosection`, `areasection`, `compact-bisect` |
| `--geosection-seeds` | `50` | Seeds per ratio for GeoSection/AreaSection |
| `--area-swing` | `1.10` | **AreaSection only.** Area imbalance tolerance `ubvec[1]`. See `redist state --help`. |
| `--balance-tolerance` | `0.5` | Max deviation per district in percent |
| `--reprocess` | false | Re-run states that already have outputs |
| `-d`, `--debug` | false | Extra diagnostic output |

**Examples**:

```bash
# All 50 states, 8 workers (~15 seconds with .adj.bin files)
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8

# Small states only (fast validation)
redist states --year 2020 --version V3 --output-dir outputs/V3 \
  --states VT DE AK WY

# Resume interrupted run (skips completed states automatically)
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8

# Force re-run of all states
redist states --year 2020 --version V3 --output-dir outputs/V3 --reprocess
```

**Resume behavior**: `redist states` checks for `final_assignments.json` before
running each state. States with existing outputs are skipped. Use `--reprocess`
to override.

---

### `redist run` — Multi-year orchestrator

Runs one or all three census years sequentially, each with full 50-state parallel
processing.

```
redist run [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-y`, `--year` | `all` | `2020`, `2010`, `2000`, or `all` |
| `-v`, `--version` | `v1` | Version tag |
| `--output-dir` | `outputs/{version}` | Output root directory |
| `-w`, `--workers` | `12` | Parallel workers per year |
| `--states <CODES>` | *(all 50)* | Limit to specific states |
| `-m`, `--partition-mode` | `edge-weighted` | Partition mode |
| `--reprocess` | false | Ignore completion markers |
| `-r`, `--reset` | false | Delete outputs before starting |
| `-d`, `--debug` | false | Debug mode |

**Examples**:

```bash
# All 50 states, all 3 years (production run)
redist run --version V3 --workers 12

# Single year
redist run --year 2020 --version V3

# Two states, one year (quick test)
redist run --year 2020 --version test --states VT DE --workers 2
```

---

### `redist fetch` — Download census data

Downloads TIGER shapefiles and PL 94-171 redistricting data from Census.gov, and
optionally pulls pre-built adjacency files from GitHub Releases.

```
redist fetch [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-y`, `--year` | `2020` | `2020`, `2010`, `2000`, or `all` |
| `--states <CODES>` | *(all 50)* | Limit download to specific states |
| `--type <TYPES>` | `all` | `tiger`, `redistricting`, `adjacency`, or `all`. ⚠️ `elections` / `enacted` / `geography` are declared but not yet implemented — see "Election data" below. |
| `--release` | false | Download adjacency pkl files from GitHub Releases |
| `--check-only` | false | Print what would be downloaded, without downloading |
| `--force` | false | Re-download even if files already exist |
| `-w`, `--workers` | `4` | Parallel download connections |
| `--manifest <PATH>` | *(embedded)* | Override the URL manifest file |

**Examples**:

```bash
# Check what's missing without downloading anything
redist fetch --year 2020 --check-only

# Download TIGER + redistricting data for Vermont only
redist fetch --year 2020 --states VT --type tiger redistricting

# Download all 2020 data (TIGER + redistricting)
redist fetch --year 2020 --workers 8

# Pull pre-built adjacency pkl files from GitHub Releases
# (requires: gh auth login)
redist fetch --year 2020 --release
```

#### Election data (separate from `redist fetch`)

`redist fetch --type elections` is declared as a CLI option but not yet implemented in `fetch.rs` — running it emits a WARNING. Election sources live in their own registry under `scripts/data/elections/sources.json` and are fetched via a Python dispatcher:

```bash
# List all known election-data sources
python scripts/data/elections/fetch_elections.py list

# Filter to tract-resolution sources
python scripts/data/elections/fetch_elections.py list --resolution tract

# Show full details of a specific source
python scripts/data/elections/fetch_elections.py show harvard-fekrazad-2020

# Fetch from the default tract-level source for a year
python scripts/data/elections/fetch_elections.py fetch --year 2020

# Fetch from a specific source by id
python scripts/data/elections/fetch_elections.py fetch --source harvard-fekrazad-2020
```

Sources currently in the registry:

| ID | Resolution | Auto-fetch | License |
|---|---|---|---|
| `harvard-fekrazad-2020` | tract | yes | CC0-1.0 |
| `mggg-openprecincts` | precinct | no (manual per-state) | varied |
| `mit-edsl-1976-2022` | county / district | no | CC-BY-4.0 |
| `dave-redistricting-app` | precinct / tract | no (web export) | proprietary |
| `daily-kos-elections` | district | no (manual) | editorial |

Adding a new source is a JSON edit to `sources.json`. Sources whose `fetcher` is null print their URL and any manual-fetch instructions.

**Why a registry:** the new Gingles 2/3 standard from *Louisiana v. Callais* requires plaintiffs to control for partisan affiliation when proving racial bloc voting. That makes high-resolution election data load-bearing for §2 evidence in a way it wasn't pre-Callais. The registry is the place to add precinct-level / primary-election sources as they become available.

The canonical 3-step partisan-lean flow:

```bash
# 1. Pick + fetch an election-data source (one-time per cycle)
python scripts/data/elections/fetch_elections.py fetch --year 2020

# 2. Run redistricting (Rust)
redist state --state VT --year 2020 --label vt_test

# 3. Compute partisan metrics + render colored map (Rust)
redist analyze --label vt_test --types political
redist map --label vt_test --types political

# Download all 3 years
redist fetch --year all --workers 8
```

**What gets downloaded**:

| Type | Destination | Description |
|------|-------------|-------------|
| `tiger` | `data/{year}/tiger/tracts/` | Census tract shapefiles |
| `redistricting` | `data/{year}/redistricting/` | PL 94-171 population data |
| `adjacency` | `outputs/V3/data/{year}/adjacency/` | Pre-built graph pkl files |

**After downloading adjacency pkls**, convert them to the fast native format:

```bash
python scripts/data/generate_adj_bin.py --year 2020
```

This produces `.adj.bin` files that the Rust CLI loads directly without any Python
subprocess (see [Native Adjacency Loading](#native-adjacency-loading) below).

---

## Native Adjacency Loading

The CLI loads adjacency data in two ways, tried in order:

1. **`.adj.bin` (preferred)** — Binary format read entirely in Rust. Zero Python
   subprocess. ~10% faster than pkl shim.
2. **`.pkl` shim (fallback)** — Python subprocess converts pkl to JSON on the fly.
   Prints a warning. Used when `.adj.bin` is absent.

To convert all 50 state pkls to `.adj.bin`:

```bash
python scripts/data/generate_adj_bin.py --year 2020
# Produces: outputs/V3/data/2020/adjacency/{state}_adjacency_2020.adj.bin
#           outputs/V3/data/2020/adjacency/{state}_adjacency_2020_geoids.json
```

---

## `redist aggregate` — Merge state analysis into national datasets

Walks all present states in `outputs/{version}/states/*/analysis/` and merges per-type JSON files into national datasets at `outputs/{version}/national/`.

```
redist aggregate [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-v`, `--version` | `v1` | Version identifier |
| `--types <TYPES>` | `all` | Analyzer types to aggregate: `demographic`, `political`, `urban`, `compactness`, `summary`, `all` |
| `--csv` | false | Write CSV alongside each JSON output |
| `--force` | false | Re-aggregate even if output exists |

**Examples**:
```bash
# Aggregate all analyzer types
redist aggregate --version V3 --types all --csv

# Just demographic national dataset
redist aggregate --version V3 --types demographic

# After running all 50 states + analyze:
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8
# (then for each state: redist analyze --state XX --types all)
redist aggregate --version V3 --types all --csv
```

**Output files** (under `outputs/{version}/national/`):

| File | Description |
|------|-------------|
| `us_demographic.json` | All districts across all states with demographic metrics |
| `us_political.json` | Partisan lean per district, all states |
| `us_compactness.json` | PP/Reock/CHR per district, all states |
| `us_summary.json` | Full summary with balance checks, all states |
| `us_demographic.csv` | CSV export of the JSON (with `--csv`) |
| `us_summary.csv` | CSV export (with `--csv`) |

All JSON files include `state_count`, `district_count`, `scope: "national"`, and a `"state"` field on every district record.

---

## `redist map --scope national` — National map with AK/HI insets

Renders all present states onto one canvas using an inset projection: continental US in top 75%, Alaska inset bottom-left, Hawaii inset next to Alaska.

Add `--scope national` to any `redist map` call (omit `--state`):

```bash
redist map --scope national --version V3 --types districts
redist map --scope national --version V3 --types political --dpi 300
redist map --scope national --version V3 --types districts political demographic compactness
```

**Output files** (under `outputs/{version}/national/maps/`):

| File | Description |
|------|-------------|
| `districts.png` | All 435 districts, categorical color scheme |
| `political.png` | National partisan choropleth (red→white→blue) |
| `demographic.png` | National minority % choropleth (yellow→brown) |
| `compactness.png` | National PP choropleth (green sequential) |

**Projection**: equirectangular display projection only (not equal-area). Alaska and Hawaii are scaled and translated into insets — not geographically accurate for metric computation but correct for visual display.

**Note**: States without TIGER shapefiles or assignment data are skipped. A warning lists any omitted states: `WARNING: N states omitted from national districts map`.

---

## `redist analyze` — Per-district analytics

Computes analytics for each district and writes JSON to `outputs/{version}/states/{state}/analysis/`.

```
redist analyze --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | Two-letter state code |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version identifier |
| `--output-base` | `outputs` | Output base directory |
| `--types <TYPES>` | `all` | Analyzers: `demographic`, `political`, `proportionality`, `urban`, `summary`, `compactness`, `bloc-voting`, `all` |
| `--force` | false | Re-run even if output exists |
| `--allow-imbalance` | false | Don't exit non-zero on population balance failure (research use) |

**Requires**: `redist state` must have been run first to produce `final_assignments.json`.

**Examples**:

```bash
# All analyzers for Vermont
redist analyze --state VT --year 2020 --version V3 --types all

# Just demographic + political
redist analyze --state AL --year 2020 --version V3 --types demographic political

# Compactness metrics (PP, Reock, CHR)
redist analyze --state TX --year 2020 --version V3 --types compactness

# Allow imbalanced results without exiting non-zero
redist analyze --state VT --year 2020 --version V3 --allow-imbalance
```

**Output files** (under `outputs/{version}/states/{state}/analysis/`):

| File | Key fields |
|------|-----------|
| `demographic.json` | `total_pop`, `pct_white/black/asian/hispanic/other`, `pct_minority`, `is_majority_minority`, `pop_basis` |
| `political.json` | `dem_pct`, `rep_pct`, `margin`, `lean_dem`, `is_uncontested`, `available` |
| `proportionality.json` | `dem_vote_share_statewide`, `dem_seat_share`, `proportionality_gap_pp` (signed: positive = D over-represented), `n_uncontested`, `per_district_dem_share_sorted` |
| `urban.json` | `largest_city`, `largest_city_pop`, `num_places`, `available` |
| `compactness.json` | `polsby_popper`, `reock`, `convex_hull_ratio`, `crs_note` |
| `summary.json` | All of the above merged + `ideal_pop`, `pop_deviation_pct`, `pop_balance_ok`, `population_balance_valid` |

**Notes:**
- `demographic.json` uses **total population** (not VAP). VRA district identification uses separate VAP-based logic in `vra_analysis.json`. Field `pop_basis: "total_population"` is always present to document this.
- `political.json` is `available: false` for years without election data (2000, 2010).
- `compactness.json` metrics are computed on WGS84 coordinates (display-only CRS). Scores for east-west-elongated states (Montana, Nevada) are systematically compressed. See `crs_note` in output.
- `summary.json` exits with code 2 if any district fails ±0.5% population balance; suppress with `--allow-imbalance`.

---

## `redist map` — PNG map rendering

Renders district maps to PNG using the native Rust SVG→PNG pipeline (no Python, no matplotlib).

```
redist map --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | Two-letter state code |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version identifier |
| `--types <TYPES>` | `districts` | Map types: `districts`, `rounds`, `political`, `demographic`, `compactness`, `all` |
| `--dpi` | `150` | Output DPI: `72`, `100`, `150`, `200`, `300` |
| `--force` | false | Re-render even if output exists |

**Requires**: `redist state` for all map types. `redist analyze` for choropleth types (political, demographic, compactness).

**Examples**:

```bash
# Categorical district map (colored regions)
redist map --state VT --year 2020 --version V3 --types districts

# Partisan choropleth (red-white-blue by Dem %)
redist map --state AL --year 2020 --version V3 --types political

# Minority % choropleth (yellow-brown sequential)
redist map --state TX --year 2020 --version V3 --types demographic

# All map types, publication quality
redist map --state CA --year 2020 --version V3 --types all --dpi 300

# Bisection round progression maps
redist map --state AL --year 2020 --version V3 --types rounds
```

**Output files** (under `outputs/{version}/states/{state}/maps/`):

| File | Description |
|------|-------------|
| `districts.png` | Categorical district map — each district a distinct color |
| `political.png` | Partisan choropleth — red (Rep) → white (tossup) → blue (Dem) |
| `demographic.png` | Minority % choropleth — yellow (low) → brown (high) |
| `compactness.png` | Polsby-Popper choropleth — green sequential |
| `maps/rounds/round_00.png` … | Bisection progression — one PNG per round |

**Labeling**: labels are sized adaptively based on polygon area. Large regions show `"1 (26)"` (district number + how many final districts this region will contain). Small regions show just `"1"`. Analytical maps show `"D+12%"` or `"42% min"` below the district number.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `REDIST_PYTHON` | Python binary to use for the pkl shim (default: `python`). Set if `python` on `PATH` doesn't have the right packages. |
| `REDIST_GH` | `gh` binary for `--release` downloads (default: `gh`). Useful for testing with a fake gh script. |
| `REDIST_MANIFEST` | Path to a custom manifest JSON file (overrides the embedded manifest). |

**Example** — point to a specific Python environment:
```bash
REDIST_PYTHON="C:/miniconda3/envs/redist/python.exe" redist state --state VT --year 2020 --version V3
```

---

## Partition Modes

| Mode | Flag | Description |
|------|------|-------------|
| Unweighted | `unweighted` | Pure graph cut, no edge weights |
| Edge-weighted | `edge-weighted` | Weights edges by shared boundary length (default) |
| VRA | `metis-vra` | Boosts edges between minority-heavy tracts to encourage majority-minority districts; uses n-way partitioning |
| Partisan-weighted | `partisan-weighted` | Boosts D-D and R-R edges to preserve partisan clusters; uses recursive bisection |

`metis-vra` requires demographic CSV files in `data/{year}/demographics/`.

`partisan-weighted` requires `--partisan-shares <PATH>` pointing at a per-tract D-share TSV (see `docs/file-formats/partisan-shares.md` once written; format is header `geoid<TAB>dem_share`, GEOIDs are 11-char TIGER FIPS, share is float in `[0.0, 1.0]`).

**Disentanglement guard:** `--partition-mode metis-vra` and `--partisan-shares` cannot be used in the same `redist state` invocation. The runner returns an error if both are specified. This is structural enforcement of *Louisiana v. Callais* p.36 (race-conscious and partisan signals must not be mixed in production map runs). See `docs/legal/CALLAIS_REFERENCE.md`.

---

## Provenance and `redist doctor --verify-manifest`

Every `redist state` run writes a `provenance.json` sidecar in the output `data/` directory containing the binary's `redist_version`, `redist_build_commit` (with `-dirty` suffix when the working tree had uncommitted changes), `redist_build_date`, and `rustc_version`. This is written atomically alongside `final_assignments.json` regardless of whether `--manifest` is set, so every plan has an audit trail to the binary that produced it.

When `--manifest` is set, `manifest.json` additionally records the binary version, input adjacency hash, and other run parameters. The `redist doctor --verify-manifest <PATH>` subcommand reads the manifest and:

- Prints the running binary's `redist_version`, `redist_build_commit` (with `-dirty` suffix if the working tree had uncommitted changes), `redist_build_date`, and `rustc_version`
- Prints the recorded `binary_version`, `binary_sha256`, `adjacency_sha256`, and `created_at`
- Asserts that the recorded version matches the running binary (FAIL if not)
- Reports the adjacency file referenced (informational; does not re-hash)
- Warns if the running binary was built outside a git checkout or from a dirty tree

**Exit codes:**
- `0` — version matches (or manifest had no version field, in which case a WARN is emitted)
- `1` — version mismatch
- `2` — manifest cannot be read or parsed

Example:

```bash
redist doctor --verify-manifest outputs/v1/states/vermont/data/manifest.json
```

Use this when independently verifying that a court-submitted plan was produced by a published `redist` binary. Combine with `sha256sum` on the adjacency file to attest to the input data: the manifest records `adjacency_sha256` (when available) and `tiger_source_url` for upstream Census provenance.

## Researcher Toolkit (partial — diagnostics + notebooks + paper-mode renderer wired)

The Researcher Toolkit plan is partial. Today (2026-04-30):

**Shipped:**
- **`redist-analysis::ensemble_diagnostics`** (Task 7 / S-03): pure-Rust math for the three load-bearing convergence diagnostics — Gelman-Rubin R-hat across ≥4 chains, Effective Sample Size (Geyer 1992 initial monotone sequence) on summary statistics, Hamming-distance autocorrelation on the partition trajectory with integrated `tau_int`. JSON-shape structs mirror the spec's `rhat.json` / `ess.json` / `hamming_autocorr.json`. 21 L0 tests.
- **`notebooks/`** (Task 1): 5 notebook stubs (`01_quickstart`, `02_parameter_sweep`, `03_callais_evidence`, `04_gerrychain_interop`, `05_mcmc_ensemble`) with cell-1 metadata declaring `runtime_budget_secs` (60 / 120 / 300 / 120 / 1800), cell-2 kernel-state attestation header against compatible RANGES (Task 2 / B-06: `redist_py >=0.4,<0.5`, `gerrychain >=0.3.2,<0.4`), and final-cell completion sentinel. Notebook bodies are TODO; scaffolding + conventions shipped.
- **`scripts/research/paper_mode_template/REPRODUCE.sh`** (Task 8 / D-05): AEA-compliant replication-script template with platform check (target `linux-x86_64-glibc-2.35`), Cargo.lock + rust-toolchain.toml + requirements.lock SHA verification, locked cargo build, output-checksum verification via jq, cross-platform reviewer note (use WSL / Docker `ubuntu:22.04`).
- **`redist analyze --paper-mode`** (Task 8.1) — emits a `paper_mode/` subdirectory containing 9 files: `REPRODUCE.sh` (template substituted with the recorded `redist_build_commit` + lockfile SHAs + verbatim analyze invocation), `inputs.sha256.json` + `expected_outputs.sha256.json` (deterministic file-walks + canonical BTreeMap key ordering), `environment.json` (rustc + target_platform + OS), `seeds.json`, `CITATION.bib` + `CITATION.apa.txt` + `CITATION.chicago.txt` (per `docs/file-formats/citation-strings.md` §3.3), and a `README.md` walkthrough. `--paper-mode-citation-style {bluebook|apa|chicago}` defaults to `apa` per citation-strings.md §1. Implementation in `redist-cli/src/paper_mode.rs`; 8 module unit tests + 4 args-parse tests.

- **`redist research validate-ensemble`** (Task 5 / M-02): consumes per-chain JSONL files (one chain per file, lines are `{step, metrics: {name: value}}`), computes Gelman-Rubin R-hat + pooled Effective Sample Size per metric using `redist-analysis::ensemble_diagnostics`. Emits `validate-ensemble v1` JSON report with per-metric pass/fail against `--rhat-threshold` (default 1.05) + `--ess-min` (default 100). Optional `--enacted <metrics.json>` records per-metric `enacted_percentile_rank` against the pooled ensemble distribution. `--strict` propagates convergence failures as non-zero exit. Input validation rejects fewer than 4 chains (S-03), unequal chain lengths, and missing-metric-in-some-chain with `[INPUT]` actionable errors. 8 unit tests including well-mixed pass / unmixed flag-failure / enacted percentile-rank position / `*.jsonl` extension filter.

**Deferred (next session pickup):**
- **Notebook BODY content** — beyond cell-1 metadata + cell-2 attestation; depends on the `redist_py` PyO3 binding stabilizing.
- **`redist research check-compat`** (Task 3) — actual round-trip property test against installed GerryChain.
- **`scripts/research/mcmc_ensemble.py`** (Task 6) — GerryChain wrapper.
- **`.github/workflows/notebooks.yml`** (Task 1.5) — runtime-budget-enforcing CI.
- **L0 paper-mode acceptance test** running `bash REPRODUCE.sh` from a clean ubuntu:22.04 container + asserting byte-identical `expected_outputs.sha256.json` matches (Task 8.4).
- **Conformance lint** against the `social-science-data-editors/template-readme` rubric (Task 8.5).
- **Per-step seed derivation** — today only `master_seed` is recorded; per-step seeds will land when the broader seed-derivation system ships.

## Deposition Prep (partial — whitelist DAG + log writer + verifier; daemon deferred)

The Deposition Prep plan is partial. Today (2026-04-30):

**Shipped (`redist-cli/src/depo.rs` + `data/whitelist_dependencies.json` + `docs/parameters/whitelist-dependencies.md`):**
- **Whitelist DAG (S-01)**: 8 parameters with explicit invalidation edges + narrative guardrails + warning rules. Markdown spec + machine-readable JSON; CI gate (when wired) will assert agreement.
- **`whitelist_deps()`** + **`lookup_param()`** + **`whitelist_compat_sha256()`**: parses the embedded JSON; the SHA goes into every `whatif-manifest v1` output so a future reader knows which compat ranges were active.
- **`parse_param_kv()`**: validates `--param KEY=VALUE` against the whitelist. Float/range/enum validation; error messages list allowed params + point at the doc.
- **`overrides_hash()`**: SHA-256 of the canonical-JSON serialization of an override map. Order-independent (BTreeMap-keyed); deterministic.
- **`canonicalize_json()`**: sorted-keys, no-whitespace JSON serialization for the log + manifest formats.
- **`DepoLogWriter`** (Task 5 / C-01): canonical JSONL log with `prev_sha256` hash chain (first entry references `"GENESIS"`), per-entry fsync, sidecar `deposition_log_{date}.manifest.json` (schema `depo-log v1`) updated atomically on every append, `close()` writes `final_sha256` of the entire log file. Recovers `(next_seq, prev_sha)` on reopen so a daemon restart appends cleanly.
- **`verify_log_file()`** + **`verify_log_bytes()`** (Task 6): walks the JSONL line-by-line, recomputes each `prev_sha256`, asserts seq monotonicity. Returns the first divergent seq + byte offset on tamper. Detects single-byte mutations AND seq-skips.
- **`whatif-manifest v1`** struct: bind parent_plan_label + parent_plan_manifest_sha256 + parent_report_pdf_sha256 + overrides + overrides_hash + override_path_relative (M-04 portable) + redist_version/build_commit/short/rustc + whitelist_compat_sha256 + generated_at + note.
- **`REDIST_BUILD_COMMIT_OVERRIDE`** env (B-07): build script honors it; verbatim, no `-dirty` suffix; emits `cargo:warning=using REDIST_BUILD_COMMIT_OVERRIDE=...` so it's visible in build logs. Used by reproducible-build packagers.
- **28 new L0 tests**: whitelist coverage (3), parse_param_kv (8 cases), overrides_hash (3), canonical JSON (3), log writer (5: GENESIS, chain, recovery, sidecar update, close+final_sha), verify-log (4: valid, single-byte tamper detection, seq-skip detection, empty), bytes-direct (1), build-commit env override.

**Deferred (next session pickup):**
- **CLI dispatch** (`redist depo recompute`, `redist depo eval`, `redist depo verify-log`, etc.) — the depo module is library-only today; no `Commands::Depo` variant in `args.rs`. The dispatch ships with the daemon (Task 4) since the daemon needs the log writer + recompute logic anyway.
- **Task 3** (IPC abstraction): Unix domain socket vs Windows named pipe (PP-26).
- **Task 4** (`redist deposition-server` daemon): warm in-memory plan, accept loop, signal handling, two-phase shutdown drain (PP-24).
- **Task 7** (`--enforce-build-commit` + `--case-mode` defaults, BD-N2): build-commit binding when starting the daemon against an existing report.
- **Task 9** (p99 benchmark methodology, B-03): N=5 warm-up + N=50 measured; pinned runner.
- **Task 10** (`examples/deposition-checklist.ipynb`): pre-deposition sweep + bookmark notebook.

## Civic Bidirectional Input (partial — ingest, validation, conflict detection)

The Civic Bidirectional Input plan is partial. Today (2026-04-30):

**Shipped (`redist-cli/src/civic.rs`):**
- `redist civic` subcommand group with five subcommands: `ingest`, `add-candidate-race`, `list`, `show`, `conflicts`. Wired into `main.rs::Commands::Civic`.
- `civic-coi v1` `CivicManifest` schema (matches `docs/file-formats/manifests.md` §3.8).
- **BOM-tolerant CSV reader + canonicalization** (Task 2 / PP-27 / DATUM): UTF-8 BOM stripped silently; UTF-16 rejected with documented remediation; LF line endings + sorted by `(geoid, comment_id)` + GEOIDs always quoted + `# civic-coi-csv v1` schema header line. Re-running the canonicalizer on the same input produces byte-identical output.
- **GEOID typo + leading-zero detection** (Task 3 / PP-28): length-9-or-10 GEOIDs trigger the Excel "leading zero stripped — re-export with column-format = Text" message; length-11-not-in-tract-set triggers a typo hint; non-numeric / wrong-length errors are categorized.
- **URL validator** (Task 4 / PP-29): rejects `mailto`/`file`/`data`/`javascript` schemes, parsed-IP loopback (127.x, ::1, ::ffff:127.0.0.1), private (RFC 1918), link-local (169.254/16), unspecified (0.0.0.0 / ::), and the literal `localhost` string. Predicate-named errors per spec.
- **Conflict detection** (Task 7 / B-08): `redist civic conflicts --label A --label B [...]` reads each label's `normalized.csv`, joins on GEOID, and emits a `ConflictsReport` with three categories — `coi_overlap` (same GEOID, different comment_ids), `coi_label_mismatch` (same comment_id, different labels), `candidate_race_disagreement` (reserved for the candidate-race CLI). `race_conflict_robustness_violated()` implements the B-08 threshold semantics: `(n_disputed / n_total) >= threshold` (default 0.10) → downstream Callais sets `robust=false`.
- **`ValidateMode`** parsing for `--validate {strict|lenient|advisory}`. Strict surfaces the first error; lenient/advisory record warnings in the manifest.
- **`run_ingest`** end-to-end happy path: reads CSV, canonicalizes, validates GEOIDs + URLs, writes `outputs/{version}/civic_inputs/{label}/{original.csv, normalized.csv, validation_log.txt, manifest.json}`. Atomic via direct `std::fs::write` (PlanDirGuard integration is the next session's pickup).
- **URL snapshot** (Task 5 / PP-30 / C-02): `redist civic ingest --snapshot-urls` archives every unique non-empty `source_url` into `snapshots/<sha8>.body` + `<sha8>.headers.txt` under the civic input dir. Bounded-fetch policy: 5 MB max body, 10 s timeout, ≤3 redirects, `User-Agent: redist-civic-snapshot/<version>`. Each URL gets a `UrlSnapshotRecord` in `manifest.url_snapshots` (status, content_type, length, truncated, body SHA-256, snapshot path). Network failures and non-2xx statuses produce `[WARN]` entries in `validation_log.txt` rather than aborting (link rot is data, not a fatal error). 5 unit tests against a localhost `TcpListener` test server.
- **Candidate-race CLI integration** (Task 8): `redist civic add-candidate-race <CSV> --year --state --submitter --attestation-doc [--label] [--output-base] [--version]` calls `redist_analysis::race_of_candidate::parse_race_of_candidate_csv` (which validates schema + BD-R2 attestation_doc_format + computes per-row attestation SHA-256s), copies the CSV + master attestation document into `outputs/{version}/civic_inputs/{label}/`, and writes `annotations.json` + `provenance.json` + `manifest.json` with the new `candidate-race v1` schema (carries n_annotations, n_curators, n_disputes, annotations_independently_verified, all SHA-256 chain links). Default label is `candidate_race_<year>_<state-lowercased>`. 5 unit tests cover full output + default-label derivation + missing-CSV / missing-attestation rejection + unverified-caveat propagation.
- **56 L0 tests** including BOM stripping, UTF-16 rejection, byte-stable round-trip, every URL predicate, every GEOID error case, B-08 boundary policy (below / at / above threshold), conflict-detection coverage, URL snapshot success/truncation/404/unreachable, candidate-race full-output / default-label / missing-CSV / missing-attestation / unverified-caveat.

**Deferred (next session pickup):**
- Task 5 follow-ups: WARC format option (today only `headers-body`); robots.txt respect or explicit allowlist.
- Task 8 (E2E civic-counter-proposal example): depends on PCN CLI dispatch shipping first.
- Task 9 (Sheets template + `docs/civic/HOWTO.md` for non-technical users).
- Task 10 (sanitized LA 2024 fixture under `tests/fixtures/civic/la_2024_round/`).
- Task 11 (dogfood test report — requires real-world neighborhood-association partner).
- PlanDirGuard integration into `run_ingest` (the SSI primitive is shipped + tested; integrating here is mechanical).

## Plan Comparison & Narrative (CLI dispatch wired; remaining items deferred)

The Plan Comparison & Narrative plan is partial. Today (2026-04-30):

**Shipped (data layer + renderer + manifest writer + CLI dispatch):**
- `redist-report::moe` (S-04 / Task 4): margin-of-error suppression with two metric monotonicities. Monotone (Dem seats, mean PP, population): suppression fires when sign of (a-b) flips inside CI overlap. Non-monotone (MM count): suppression fires when CIs overlap; per-district indeterminacy when BVAP CI straddles 50%. Canonical text constant (`"within margin of error; see numerical table."`). 13 L0 tests.
- `redist-report::comparison` (Task 2): `ComparisonReport` + `PlanSide` + `DiffSummary` data structures with both `from_loaded()` constructor AND from-disk assembler (`load_plan_side_from_dir`, `diff_from_assignments`). Flexible serde_json::Value parsing handles multiple JSON shapes (per_district_dem_share array vs nested districts; mean_polsby_popper field vs computed mean from districts). `AssembleError` is a `thiserror`-derived enum with `[INPUT]`-prefixed messages.
- `redist-report::narrative` (Tasks 3+5): direct-Rust narrative renderer. Civic-friendly framing first, `[DRAFT]` gate per paragraph, `--approved-by` sign-off, civic-counter-proposal framing label, threshold disclosure, close-call flagging, MoE suppression integration. ASCII-only output (PP-34). 16 L0 tests including all 4 value-correctness anchors.
- `redist-report::narrative_manifest` (M-04 + PP-31 + COVENANT C-3 / Task 9): `narrative-manifest v1` schema with plan SHAs (NOT just labels), template SHA, threshold values, MoE inputs, approved_by + approved_at, civic-counter-proposal attribution, baseline plan reference. Reproducible via `build_narrative_manifest_with_clock()` (env-free, parallel-test-safe). BTreeMap canonical key ordering. 13 L0 tests.
- **CLI dispatch (Task 11)**: `redist compare --format narrative` and `--format both` write `narrative.md` + `narrative_manifest.json` end-to-end. `--format both` also prints the legacy table to stdout. Output dir defaults to `outputs/{version}/comparisons/{plan_a}_vs_{plan_b}/` and is overridable via `--report-dir`. `template_sha256` records the SHA of the embedded `narrative.rs` source (the renderer IS the template). `SOURCE_DATE_EPOCH` propagates to `approved_at` for byte-stable manifests. 4 L1 integration tests in `compare_narrative_l1.rs` cover DRAFT-mode default, approved-mode prefix removal, `--format both` dual-output, and `--enacted`-only rejection.
- **HTML side-by-side (Task 7)**: `redist compare --format html` writes `comparison.html` alongside the narrative + manifest. Self-contained HTML5 with embedded CSS — no external resources. Layout: civic-counter-proposal banner when applicable, DRAFT/APPROVED badge header, side-by-side metrics table (Districts, Dem-leaning seats, MM count, mean Polsby-Popper, total population with thousands separators), diff scope, inline narrative as paragraphs, chain-of-custody footer with plan-A/plan-B/template SHAs + reproducibility command. All user-controlled fields escaped via `escape_html()`. Print stylesheet preserves color on paper. 16 module unit tests + 1 L1 integration test.
- `redist compare --leaning-threshold` / `--close-call-band` / `--approved-by` / `--report-dir` CLI flags wired into `args.rs`.

**Deferred (next pickup):**
- Diff PNG visualization (Task 6): requires extending `redist-map` with a diff-renderer (third map, color-coded by destination district).
- Civic summary card PNG with watermark (Task 8 / BD-N3): 1200×675 social-media share preview with diagonal watermark on civic-counter-proposal plans.
- `--comments-label` overlay (Task 10): consumes `redist civic ingest` outputs; depends on Civic Bidirectional plan shipping first.
- Tera-based override-template path (`--narrative-template <PATH>`).
- `--enacted` plan B handling for narrative dispatch (today the dispatcher rejects with an actionable error pointing at the .rplan / DRA CSV / GeoJSON-import alternatives).
- Embedded base64 PNG maps inside the HTML report (today the HTML is text-only; the diff PNG renderer in Task 6 will provide the image data).

## State Staff Interop (partial — atomic import + Callais gate + civic bypass)

The State Staff Interop plan is partial. Today (2026-04-30):

**Shipped:**
- `redist-report::manifest::PlanDirGuard` (PP-22): atomic-import infrastructure. Builds plans into `{label}.tmp/`, renames to `{label}/` on `commit()`, deletes tmp on drop without commit. Refuses to overwrite without `force=true` (label-collision check). 6 L0 tests covering commit/drop/collision/force/stale-cleanup/mid-run-race.
- `redist-report::manifest::callais_preflight` (BOUNDARY): inspects a `PlanManifest` for the simultaneous presence of VRA-aware (`metis-vra` OR `cvap` population source) AND partisan-weighted markers; returns `[BOUNDARY]` error when both. Wired into `redist analyze` so any plan whose manifest carries both markers refuses analysis. Wired into `redist import` as a forward guard. 5 L0 tests covering clean/VRA-only/partisan-only/blocked/error-message.
- `redist-report::canonical` (spec §6 round-trip equality): `canonicalize_assignments()` re-numbers districts by ascending min-GEOID; `diff_assignments()` returns a structured `AssignmentDiff`; `assert_canonical_equal()` returns `[INPUT]`-categorized error with the offending GEOIDs. 10 L0 tests including label-permutation collapse, three-way permutation, distinguishes-different-partitions.
- `redist import --as-civic-counter-proposal --submitted-by "<org>"` (Task 7, COMMONS): tags the imported plan's manifest with `submission_type = "civic_counter_proposal"`. `--submitted-by` required. `--submitted-at` defaults to import time. Downstream comparison reports surface the civic framing instead of treating it as authoritative.
- `PlanManifest` extended with: `submission_type` (default `"authoritative"`), `submitted_by`, `submitted_at`, `source_tool`, `source_tool_version`, `source_format_fingerprint`, `import_compat_sha256`. All optional / serde-default for backward compat with legacy manifests.

**Deferred:**
- Full schema-version handshake (Task 5: `import_compat.json` compile-time embed + multi-attribute fingerprint for Districtr + DRA column-set fingerprint). The manifest fields are wired for it; the lookup table + matching code is the next session's pickup.
- Atomic-import refactor of `run_import` itself (Task 1.2: integrate `PlanDirGuard` into the import flow). The guard is shipped + tested; integrating it requires a careful refactor of the existing 90-line `run_import` body.
- Native shapefile import (Task 4): existing `is_shapefile_extension` guard still emits the `ogr2ogr` guidance; native read via the `shapefile` crate is the next session's work.
- L1 round-trip property tests for Districtr/DRA (Task 8.4-8.7).

## Court Submission Reports (partial — Typst integration scaffolded, not executable)

The Court Submission Reports plan is partially implemented. Today (2026-04-30):

**Shipped:**
- `docs/file-formats/manifests.md` — canonical manifest field inventory + naming + path-portability + cross-manifest hash-link convention.
- `docs/file-formats/citation-strings.md` — Bluebook / APA / Chicago templates per source class with worked examples.
- `redist report --format pdf` CLI surface flags: `--expert-name`, `--expert-credentials`, `--expert-affiliation`, `--case-caption-file`, `--jurisdiction`, `--citation-style`, `--expert-config`, `--allow-non-strict-civic`, `--draft`. (Wired into `args.rs`; the legacy HTML→PDF path doesn't consume them yet.)
- `redist-report::civic_gate` (BD-R1): when court-mode render detects civic inputs ingested under non-strict validation (`--validate {lenient,advisory}`), refuses unless `--allow-non-strict-civic` is set. 10 L0 tests cover all paths.
- `redist/crates/redist-report/typst-templates/.typst-version` (`0.12.0`) and `.verapdf-version` (`1.26.2`) version pins.
- `redist/crates/redist-report/typst-templates/README.md` documents the integration path the next session will execute.

**Deferred (requires Typst + verapdf installed in dev + CI):**
- The actual Typst document templates (`expert_report.typ` + per-section partials).
- The Rust `typst_render` module that shells out to Typst and gates on verapdf (PP-32).
- PDF/A-2b determinism (`SOURCE_DATE_EPOCH`, sorted-name tar+gzip with fixed mtime, zeroed PDF metadata — D-04).
- Removal of the legacy `try_generate_pdf` wkhtmltopdf/pandoc fallback.
- L1 + L2 acceptance tests (B-01 P0 PDF-text-extraction, B-10 section-header ordering, the deliberately-malformed-Typst negative test).
- Reproducibility-zip generator (`reproducibility_package.zip` with deterministic byte output).

**Today's `redist report --format pdf` behavior:** falls back to the legacy `try_generate_pdf` path (wkhtmltopdf/pandoc with a `[CONFIG]` error if neither is installed). The new flags are accepted but advisory.

## Within-Party Bloc Voting (Callais Evidence Layer)

`redist analyze --types bloc-voting` runs a per-precinct WLS regression of candidate share on racial composition AND a partisan baseline, producing the disentangled bloc-voting evidence required by *Louisiana v. Callais* (608 U.S. ___, 2026-04-29) p.36. It is opt-in (NOT included in `--types all`) because it requires a curator-attested race-of-candidate annotation file.

### What it does

For every analyzed candidate × variant (primary baseline + 3 robustness baselines + N leave-one-out variants from civic conflict-resolution), the analyzer:

1. Fits WLS weighted by precinct vote count: β̂ = (XᵀWX)⁻¹ XᵀWy.
2. Computes HC3 robust standard errors (Long & Ervin 2000).
3. Computes Variance Inflation Factor for `pct_minority` against `pct_dem_baseline`. VIF > 5 sets `vif_underpowered_flag`.
4. Cluster-bootstraps by county (B configurable, default 10 000) for confidence intervals; flags `ci_naive_vs_cluster_diverged` when the cluster CI materially exceeds the naive precinct-level CI.
5. Holm-Bonferroni step-down across the **joint family** of all (candidate, variant) tests. Family size m = n_candidates × (1 primary + 3 robustness + n_loo_variants) per the SCALE-block-lifting bargain in v2 spec.
6. Per-candidate roll-up: `race_coefficient_significant_under_all` is `true` iff Holm-corrected p < α under every variant.
7. Emits the verbatim ecology caveat in every output: precinct-level association ≠ individual-voter behavior. Required.
8. When the race-of-candidate provenance has any row with `independently_verified=false`, prepends `[CAVEAT — annotations not independently verified]` to the draft interpretation (B-02 anchor 4).

### CLI surface

```bash
redist analyze \
    --label la_2020_callais --year 2020 --version v1 \
    --types bloc-voting \
    --candidate-race-csv data/elections/race_of_candidate/la_2020_dem_primary.csv \
    --partisan-baseline data/elections/precincts/la_2020_dem_primary.tsv \
    --party DEM \
    --election presidential-primary \
    --minority-group black \
    --bootstrap-samples 10000 \
    --ci-level 0.95 \
    --alpha 0.05 \
    --min-precincts 50 \
    --method wls
```

Required flags for `--types bloc-voting`:

- `--candidate-race-csv <PATH>` — schema in `docs/file-formats/race-of-candidate.md`. Every annotation must come with a curator + signed attestation document; the parser computes SHA-256 of the CSV and every attestation doc.
- `--partisan-baseline <PATH>` — per-precinct TSV with columns `candidate_name, precinct_id, county_fips, total_votes, candidate_share, pct_minority, pct_dem_baseline`. Produced via `scripts/data/political/build_dem_shares.py` plus a state-specific precinct loader (LA/AL/GA fetchers in `scripts/data/elections/`).

Optional flags:

- `--method wls` (default; `rxc` returns not-yet-implemented per spec)
- `--minority-group {black|hispanic|asian}` (default `black`)
- `--alpha <FLOAT>` (default `0.05`)
- `--ci-level <FLOAT>` (default `0.95`)
- `--bootstrap-samples <N>` (default `10000`; consider `2000` for development on slow hardware)
- `--min-precincts <N>` (default `50`; analyses below this exit `[INPUT]` with a clear message)

### Output artifacts

Written to the plan's `analysis/` directory:

- `bloc_voting.json` — schema `bloc-voting v1`, validates against `redist-analysis/schemas/bloc_voting.schema.json`. Top-level fields: `analyzer`, `state`, `year`, `election`, `party`, `method`, `ecology`, `candidates[]` (one per analyzed candidate, with `regression`, `robustness_check`, `ecology_caveat`, `draft_interpretation`), `race_of_candidate_provenance` (sha256 chain), `_family_detail[]` (per-variant breakdown when robustness/LOO variants are present), `provenance` (build commit, rustc).
- `bloc_voting_summary.md` — plain-English `[DRAFT — expert witness should rewrite]` summary including verbatim ecology caveat, robustness table, and curator attestation summary.
- `analysis/bloc_voting/race_of_candidate.csv` — staged copy of the input CSV (Task 5; reproducibility-zip will pick it up).
- `analysis/bloc_voting/attestations/<sha256>.<ext>` — every unique attestation document, content-addressed.

### Error categories (`docs/error-conventions.md`)

- `[INPUT]` — missing flag, bad row in race-of-candidate CSV, attestation doc not found, format mismatch, fewer precincts than `--min-precincts`.
- `[INTERNAL]` — singular design matrix (predictors collinear), numerical edge case in HC3 sandwich.
- Method `rxc` returns a clear "not yet implemented per spec" message pointing at `docs/legal/CALLAIS_REFERENCE.md` for the deferral rationale.

### Why is this safe to claim as Callais evidence?

Every SCALE-block-lifting receipt is enforced by a named L0 test:

- `test_b02_anchor1_ols_coefficient_within_002` — WLS recovers β within ±0.02 on synthetic ground truth.
- `test_b02_anchor2_holm_dominates_raw_on_30test_family` — Holm correction is provably conservative.
- `test_b02_anchor3_vif_above_5_sets_underpowered_flag` — collinearity catches the underpowered case.
- `test_b02_anchor4_independently_verified_false_injects_caveat` — un-verified annotations cannot ship without the caveat.

See `redist/crates/redist-analysis/src/bloc_voting.rs` and `bloc_voting_writer.rs`, plus `docs/file-formats/race-of-candidate.md` for the curator-attestation chain.

---

### `redist doctor --check-tutorial-data`

Validates that a tutorial walkthrough's pinned data + expected outputs match their `checksums.json`. Catches upstream-data drift (Census re-publishes a TIGER vintage, Fekrazad publishes a new file revision, etc.) before the user wastes a debugging session chasing a phantom bug.

```bash
redist doctor --check-tutorial-data --tutorial vermont-2020
```

Reads `examples/{tutorial}-walkthrough/checksums.json` (schema: `tutorial-checksums v1`), hashes each pinned input + expected output that exists locally, and reports per-row `[PASS]` / `[FAIL]` / `[MISSING]`.

**Exit codes:**
- `0` — every row PASS, OR rows are MISSING (file not yet fetched/run is not an error)
- `1` — at least one row FAIL (file present but hash differs from pin)
- `2` — checksums.json missing or malformed

The Vermont 2020 walkthrough fixture ships with `PIN_ON_FIRST_RUN` placeholder SHAs; a maintainer runs `bash examples/vermont-2020-walkthrough/pin.sh` after a successful clean run to replace them with real hashes. Until then, expect FAIL on populated files. See `examples/vermont-2020-walkthrough/README.md`.

---

## First-time setup: `bootstrap.sh` / `bootstrap.bat`

For a clean machine, the project ships one-shot bootstrap scripts at the repo root:

```bash
bash bootstrap.sh                            # Linux/macOS
bash bootstrap.sh --with-python              # also build redist_py PyO3 wheel
bash bootstrap.sh --with-api-key             # prompt + validate Dataverse key
bootstrap.bat                                # Windows mirror
```

Steps: rustup install if missing → `cargo build --release --locked` → PATH preflight (PP-18: verify binary at expected path before mutating PATH) → optional Python wheel + API-key round-trip validation (PP-19: validate before write) → real smoke test (PP-20: not `--print-only`; runs `redist state --state VT --year 2020`).

Target wall-clock: ≤ 10 minutes on a clean Ubuntu 22.04 container or Windows 11 VM.

See `docs/quickstart/quickstart-{persona}.md` for persona-specific next steps.

---

## Performance

Benchmarks on Windows 11, 8 workers, 2020 census:

| Workload | Python | Rust CLI |
|----------|--------|----------|
| Vermont (1 district) | ~4.5 s | ~0.5 s |
| Alabama VRA (7 districts) | ~1.6 s | ~0.65 s |
| All 50 states | ~55 min | ~15.5 s |

**~213× faster** than Python for the full 50-state run.

---

## Comparing Rust vs Python Outputs

After running both pipelines, use the comparison script to validate:

```bash
python scripts/pipeline/compare_rust_vs_python.py \
  --rust-version V4 --python-version V3 --year 2020
```

Checks per state: assignment file exists, district count matches config, tract
count matches adjacency graph, population balance ≤ 0.5%.

---

## Troubleshooting

**`ERROR: unknown state 'XX'`**
: The state code is not recognized. Use two-letter USPS codes: `VT`, `CA`, `TX`.

**`ERROR: adjacency file not found`**
: Run `redist fetch --release` to download adjacency files, then
  `python scripts/data/generate_adj_bin.py` to convert them.

**`gpmetis: command not found`**
: Install METIS: `conda install -c conda-forge metis`

**`WARNING: falling back to pkl shim`**
: The `.adj.bin` file is missing. Run `generate_adj_bin.py` to create it, or
  ignore the warning — the pkl shim works correctly, just ~10% slower.

**`ERROR: unsupported year 'X'`**
: Valid years are `2020`, `2010`, `2000`, and (for `run`/`fetch`) `all`.
