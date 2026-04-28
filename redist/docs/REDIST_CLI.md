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
| `-y`, `--year` | `2020` | Census year: `2020`, `2010`, `2000` (or country-specific year for international locations) |
| `-v`, `--version` | `v1` | Version tag (used in output path) |
| `--output-dir` | `outputs/{version}` | Override output root directory |
| `-m`, `--partition-mode` | `edge-weighted` | `unweighted`, `edge-weighted`, `metis-vra` |
| `--ufactor` | `5` | METIS imbalance tolerance (5 = ±0.5%) |
| `--niter` | `100` | METIS refinement iterations |
| `--seed` | *(random)* | METIS random seed for reproducible runs |
| `--label` | `{state}_{chamber}_{year}` | Human label for this plan run |
| `--chamber` | `congressional` | Chamber type: `congressional`, `house`, `senate`, `custom` |
| `--districts` | *(auto from policy)* | Override district count |
| `--balance-tolerance` | *(from policy)* | Max deviation per district in percent (e.g., `0.5` for congressional, `5.0` for state legislative). Values are percentages — `0.5` means ±0.5%, not ±0.5 people. |
| `--population-source` | `total` | Population basis: `total`, `vap`, `cvap` |
| `--seats-per-district` | `1` | Seats per constituency for multi-member systems (Malta: `5`, Ireland: `3`–`5`) |
| `--total-seats` | *(none)* | Alternative to `--seats-per-district x --districts`; ideal_per_seat = total_pop / total_seats |
| `--adjacency` | *(auto)* | Direct path to `.adj.bin` file (bypasses manifest lookup; required for international locations) |
| `--state-name` | *(lowercase state code)* | Human-readable location name used in file paths and labels |
| `--coi-weights` | *(none)* | Path to JSON file mapping GEOID → weight (0.0–1.0) for Community of Interest preservation |
| `-r`, `--reset` | false | Delete output directory before starting |
| `-d`, `--debug` | false | Extra diagnostic output |
| `-p`, `--print-only` | false | Dry run — print actions without executing |
| `--force` | false | Overwrite existing plan without error |

**Examples**:

```bash
# Vermont (1 district) — fast smoke test
redist state --state VT --year 2020 --version V3

# Alabama with VRA mode (7 districts)
redist state --state AL --year 2020 --version V3 --partition-mode metis-vra

# Reproducible run with fixed seed
redist state --state CA --year 2020 --version V3 --seed 42

# Fresh re-run (delete previous output first)
redist state --state TX --year 2020 --version V3 --reset

# Washington state house (98 districts), labeled
redist state --state WA --year 2020 --chamber house --label wa_house_draft1

# Ireland STV (43 constituencies, 4 seats average)
redist state --state IE --year 2022 \
  --adjacency outputs/international/ie/ie_adjacency_2022.adj.bin \
  --districts 43 --seats-per-district 4 --balance-tolerance 16.0

# Malta parliament (13 districts, 5 seats each)
redist state --state MT-PARLIAMENT --year 2022 \
  --adjacency outputs/international/mt/mt_adjacency_2022.adj.bin \
  --seats-per-district 5 --total-seats 65

# With Community of Interest weights
redist state --state WA --year 2020 --coi-weights coi_weights.json
```

**Output files** (under `outputs/{version}/{year}/{state_name}/`):
- `final_assignments.json` — tract → district mapping
- `manifest.json` — full run parameters for reproducibility
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
| `-m`, `--partition-mode` | `edge-weighted` | `unweighted`, `edge-weighted`, `metis-vra` |
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
| `--type <TYPES>` | `all` | `tiger`, `redistricting`, `adjacency`, `enacted`, `geography`, `elections`, or `all` |
| `--release` | false | Download adjacency pkl files from GitHub Releases |
| `--check-only` | false | Print what would be downloaded, without downloading |
| `--force` | false | Re-download even if files already exist |
| `-w`, `--workers` | `4` | Parallel download connections |
| `--manifest <PATH>` | *(embedded)* | Override the URL manifest file |
| `--verify-downloads` | false | Verify SHA-256 of each downloaded file; deletes corrupt files on mismatch |

**Examples**:

```bash
# Check what's missing without downloading anything
redist fetch --year 2020 --check-only

# Download TIGER + redistricting data for Vermont only
redist fetch --year 2020 --states VT --type tiger redistricting

# Download all 2020 data (TIGER + redistricting)
redist fetch --year 2020 --workers 8

# Download election data for partisan analysis
redist fetch --year 2020 --type elections

# Pull pre-built adjacency pkl files from GitHub Releases
# (requires: gh auth login)
redist fetch --year 2020 --release

# Download all 3 years
redist fetch --year all --workers 8
```

**What gets downloaded**:

| Type | Destination | Description |
|------|-------------|-------------|
| `tiger` | `data/{year}/tiger/tracts/` | Census tract shapefiles |
| `redistricting` | `data/{year}/redistricting/` | PL 94-171 population data |
| `adjacency` | `outputs/V3/data/{year}/adjacency/` | Pre-built graph pkl files |
| `elections` | `data/{year}/elections/` | Presidential election results by tract |

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
redist analyze --label <LABEL> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(optional when --label set)* | Two-letter state code; read from plan manifest when `--label` is provided |
| `--label <LABEL>` | *(none)* | Plan label; resolves paths from `plans/{label}/` |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version identifier |
| `--output-base` | `outputs` | Output base directory |
| `--types <TYPES>` | `all` | Analyzers: `demographic`, `political`, `urban`, `summary`, `compactness`, `contiguity`, `splits`, `all` |
| `--force` | false | Re-run even if output exists |
| `--allow-imbalance` | false | Don't exit non-zero on population balance failure (research use) |
| `--allow-noncontiguous` | false | Don't set exit code bit 1 on non-contiguous districts (research use) |
| `--election-file <PATH>` | `data/{year}/elections/presidential_by_tract.csv` | Custom election CSV for partisan analysis |
| `--election-format` | `us-presidential` | Election data format: `us-presidential`, `ie-dail`, `de-bundestag`, `generic-party-totals` |
| `--bootstrap-samples` | `1000` | Bootstrap samples for partisan confidence interval |
| `--external-analyzer <SCRIPT>` | *(none)* | Path to external analyzer script; called as `python script.py {assignments_json} {output_dir}` and recorded in audit trail |
| `--output-dir <PATH>` | *(auto)* | Override output directory for analysis results |

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

# Using a plan label (state inferred from manifest)
redist analyze --label vt_congressional_2020 --types all

# Ireland with Dail election format
redist analyze --state IE --year 2022 --election-format ie-dail

# Run with external custom analyzer script
redist analyze --state WA --year 2020 --external-analyzer scripts/my_analyzer.py

# Custom election data file
redist analyze --state WA --year 2020 \
  --election-file data/2020/elections/wa_governor_by_tract.csv
```

**Output files** (under `outputs/{version}/states/{state}/analysis/`):

| File | Key fields |
|------|-----------|
| `demographic.json` | `total_pop`, `pct_white/black/asian/hispanic/other`, `pct_minority`, `is_majority_minority`, `pop_basis` |
| `political.json` | `dem_pct`, `rep_pct`, `margin`, `lean_dem`, `is_uncontested`, `available` |
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
| `--state <CODE>` | *(required for state scope)* | Two-letter state code |
| `--scope` | `state` | `state` or `national` |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version identifier |
| `--types <TYPES>` | `districts` | Map types: `districts`, `rounds`, `political`, `demographic`, `compactness`, `splits`, `all` |
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

# County split choropleth
redist map --state WA --year 2020 --version V3 --types splits
```

**Output files** (under `outputs/{version}/states/{state}/maps/`):

| File | Description |
|------|-------------|
| `districts.png` | Categorical district map — each district a distinct color |
| `political.png` | Partisan choropleth — red (Rep) → white (tossup) → blue (Dem) |
| `demographic.png` | Minority % choropleth — yellow (low) → brown (high) |
| `compactness.png` | Polsby-Popper choropleth — green sequential |
| `splits.png` | County split choropleth — highlight split counties |
| `maps/rounds/round_00.png` … | Bisection progression — one PNG per round |

**Labeling**: labels are sized adaptively based on polygon area. Large regions show `"1 (26)"` (district number + how many final districts this region will contain). Small regions show just `"1"`. Analytical maps show `"D+12%"` or `"42% min"` below the district number.

---

## `redist compare` — Compare two plans

Computes pairwise metrics (Jaccard similarity, population balance delta, compactness delta, county split delta, partisan shift) between two redistricting plans.

```
redist compare --plan-a <LABEL_OR_PATH> [--plan-b <LABEL_OR_PATH>] [OPTIONS]
redist compare --labels <LABEL1,LABEL2,...> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--plan-a <LABEL>` | *(required)* | First plan — label or path to plan directory |
| `--plan-b <LABEL>` | *(none)* | Second plan — label or path |
| `--enacted` | false | Use currently enacted districts as plan B |
| `-y`, `--year` | `2020` | Census year for label resolution |
| `-v`, `--version` | *(none)* | Version directory for label resolution |
| `--metrics <TYPES>` | `all` | `population`, `compactness`, `splits`, `partisan`, `all` |
| `--format` | `table` | Output format: `table`, `json`, `csv` |
| `--out <PATH>` | *(stdout)* | Write output to file |
| `--allow-cross-year` | false | Suppress warning for intentional cross-census-year comparisons |
| `--labels <LIST>` | *(empty)* | Comma-separated list of N plan labels for an N-plan summary table (alternative to `--plan-a`/`--plan-b`) |
| `--output-dir <PATH>` | *(auto)* | Override output directory for comparison results |

**Examples**:

```bash
# Compare two labeled plans
redist compare --plan-a wa_house_plan_a --plan-b wa_house_plan_b

# Compare against enacted districts
redist compare --plan-a vt_congressional_2020 --enacted

# Export comparison as JSON
redist compare --plan-a plan1 --plan-b plan2 --format json --out compare.json

# N-plan summary table (more than two plans)
redist compare --labels wa_house_plan_a,wa_house_plan_b,wa_house_plan_c

# Cross-year comparison (suppress warning)
redist compare --plan-a wa_2010 --plan-b wa_2020 --allow-cross-year
```

**Expected output** (table format):

```
Metric              Plan A       Plan B       Delta
----------------------------------------------------------
Max pop deviation   0.48%        0.41%        -0.07%
Mean PP score       0.312        0.328        +0.016
County splits       12           10           -2
Jaccard similarity  -            -            0.87
```

---

## `redist report` — Commission report

Generates a self-contained HTML or machine-readable JSON report from analysis outputs, suitable for public commission review or court submission.

```
redist report --label <LABEL> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--label <LABEL>` | *(required)* | Plan label (e.g., `vt_congressional_2020`) |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version directory |
| `--format <FORMATS>` | `html` | Space-separated output formats: `html`, `json`, `pdf` (pdf not yet available) |
| `--out <DIR>` | `reports/{label}/` | Output directory for report files |
| `--audit-only` | false | Write `audit.json` only (chain-of-custody; skips HTML/JSON report) |
| `--audit-with-report` | false | Write full report AND `audit.json` (court submission mode) |
| `--output-base` | `outputs` | Base output directory |

**Examples**:

```bash
# HTML report for a labeled plan
redist report --label vt_congressional_2020 --format html

# JSON report
redist report --label wa_house_draft1 --format json

# Both HTML and JSON
redist report --label tx_congressional_2020 --format html json

# Court submission: full report + audit trail
redist report --label wa_congressional_2020 --audit-with-report

# Chain-of-custody only (no full report)
redist report --label wa_congressional_2020 --audit-only

# Custom output directory
redist report --label vt_congressional_2020 --out /tmp/commission_reports/
```

**Expected output** — `reports/{label}/index.html` — a self-contained HTML page with:
- District population balance table
- Compactness metrics (PP, Reock, CHR) per district
- Partisan lean summary (if election data available)
- County split count
- Demographic breakdown per district

`audit.json` contains the full chain-of-custody: manifest hash, binary SHA-256, environment, timestamp, and METIS parameters.

---

## `redist export` — Export plan

Exports a redistricting plan to external formats for use with GerryChain, GIS tools, or court submission.

```
redist export --label <LABEL> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--label <LABEL>` | *(required)* | Plan label |
| `-y`, `--year` | `2020` | Census year |
| `-v`, `--version` | `v1` | Version directory |
| `--format <FORMATS>` | `geojson` | Space-separated formats: `geojson`, `gerrychain`, `csv`, `reproducibility-package` |
| `--out <DIR>` | `exports/{label}/` | Output directory |
| `--output-base` | `outputs` | Base output directory |

**Format details**:

| Format | Description |
|--------|-------------|
| `geojson` | RFC 7946 GeoJSON FeatureCollection with district assignment per tract |
| `gerrychain` | GerryChain v2.3 format (uses `"assignment"` singular field) |
| `csv` | GEOID,district CSV — one row per census tract |
| `reproducibility-package` | Court-submission directory: manifest, binary hash, assignments, analysis, audit trail |

**Examples**:

```bash
# GeoJSON export
redist export --label vt_congressional_2020 --format geojson

# GerryChain format for chain-of-thought analysis
redist export --label wa_house_draft1 --format gerrychain

# Multiple formats at once
redist export --label tx_congressional_2020 --format geojson csv

# Court-submission reproducibility package
redist export --label wa_congressional_2020 --format reproducibility-package

# Custom output path
redist export --label vt_congressional_2020 --out /tmp/exports/
```

**Expected output** (geojson):

```
exports/vt_congressional_2020/
  vt_congressional_2020.geojson     # FeatureCollection, one feature per tract
  vt_congressional_2020.csv         # GEOID,district (with --format csv)
```

**Expected output** (reproducibility-package):

```
exports/vt_congressional_2020_repro/
  manifest.json          # Full run parameters + environment
  assignments.json       # Tract → district mapping
  binary_hash.sha256     # SHA-256 of the redist binary used
  audit.json             # Chain-of-custody record
  analysis/              # All analysis outputs
```

---

## `redist doctor` — Pre-flight check

Verifies that all required data files are present and valid for a given location, year, chamber, and resolution. Reports warnings for resolution mismatches and errors for missing adjacency or population files.

```
redist doctor --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | Location code (e.g., `WA`, `IE`, `MT-PARLIAMENT`, `_TEST_EL`) |
| `-y`, `--year` | `2020` | Census year to check |
| `--chamber` | `congressional` | Chamber to check: `congressional`, `house`, `senate` |
| `--resolution` | `tract` | Resolution to check: `tract`, `block_group`, `block` |
| `--output-base` | `outputs` | Base directory for adjacency path check |

**Examples**:

```bash
# Check Vermont 2020 congressional
redist doctor --state VT --year 2020

# Check Washington state house at block group resolution
redist doctor --state WA --year 2020 --chamber house --resolution block_group

# Check Ireland 2022 (international location)
redist doctor --state IE --year 2022

# Check a test/synthetic location
redist doctor --state _TEST_EL
```

**Expected output**:

```
[OK]  Location code 'VT' found in policy registry
[OK]  Year 2020 is valid for VT
[OK]  Chamber 'congressional' — 1 district
[OK]  Adjacency file: outputs/v1/data/2020/adjacency/vt_adjacency_2020.adj.bin
[OK]  Population data: data/2020/redistricting/vt_pop_2020.csv
[WARN] Resolution 'block_group' not yet validated for VT — tract recommended
```

Exit codes: `0` = all checks pass, `1` = warnings only, `2` = missing required files.

---

## `redist verify` — Reproduce from manifest

Reads a `manifest.json`, re-runs redistricting with the exact same parameters (seed, state, year, METIS settings), then computes Jaccard similarity between the original and reproduced plan. Used to confirm bit-for-bit reproducibility and court-submission verification.

```
redist verify --manifest <PATH> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--manifest <PATH>` | *(required)* | Path to `manifest.json` to reproduce and verify |
| `--min-similarity` | `0.99` | Minimum Jaccard similarity for a PASS result (0.0–1.0) |
| `--verify-label` | `{original_label}_verify` | Label for the re-run verification plan |
| `--output-base` | `outputs` | Base output directory for verification outputs |
| `--dry-run` | false | Print the equivalent CLI command without executing |
| `--skip-binary-check` | false | Skip SHA-256 binary hash check (for source-built or different-release binaries) |

**Examples**:

```bash
# Verify Vermont plan from manifest
redist verify --manifest outputs/v1/states/vermont/manifest.json

# Verify with custom threshold (research use)
redist verify --manifest plans/wa_congressional/manifest.json --min-similarity 0.95

# See what command would be run without executing
redist verify --manifest plans/vt_test/manifest.json --dry-run

# Skip binary hash check (building from source)
redist verify --manifest plans/vt_test/manifest.json --skip-binary-check
```

**Expected output**:

```
Reproducing: vt_congressional_2020 (seed=42, state=VT, year=2020)
Re-run complete.
Jaccard similarity: 0.9983
Binary hash match: YES
RESULT: PASS (threshold: 0.99)
```

Exit codes: `0` = PASS (similarity >= threshold), `1` = FAIL (below threshold), `2` = manifest unreadable or re-run failed.

---

## `redist sweep` — N-seed optimization

Runs N redistricting seeds for a state and keeps the top-K plans ranked by a chosen metric. Useful for finding the most compact, least split, or most balanced plan across a distribution of random seeds.

```
redist sweep --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | State code |
| `-y`, `--year` | `2020` | Census year |
| `--chamber` | `congressional` | Chamber type |
| `--seeds` | `10` | Number of seeds to run |
| `--seed-start` | `1` | Starting seed value (seeds `N` through `N+seeds-1` are used) |
| `--keep-top` | `3` | Keep top-K plans by the optimization metric |
| `--optimize-by` | `compactness` | Metric to rank by: `compactness`, `splits`, `deviation` |
| `-v`, `--version` | `sweep` | Version identifier for output paths |
| `--output-base` | `outputs` | Output base directory |
| `--run` | false | Auto-execute the generated commands (v2 feature; currently prints commands only) |

**Examples**:

```bash
# Run 10 seeds for Vermont, keep top 3 by compactness
redist sweep --state VT

# 20 seeds for Washington, keep top 5 by split minimization
redist sweep --state WA --seeds 20 --keep-top 5 --optimize-by splits

# Start from seed 100 (to extend a prior sweep)
redist sweep --state CA --seeds 50 --seed-start 100 --optimize-by compactness

# Custom version label
redist sweep --state TX --seeds 30 --version sweep_v2
```

**Expected output**:

```
Sweep: VT congressional 2020, 10 seeds (1..10), optimize by compactness
Generating redist state commands...

redist state --state VT --year 2020 --seed 1 --label vt_sweep_seed1 --version sweep
redist state --state VT --year 2020 --seed 2 --label vt_sweep_seed2 --version sweep
...

Top 3 by compactness (run after all seeds complete):
  redist compare --labels vt_sweep_seed3,vt_sweep_seed7,vt_sweep_seed9
```

Note: `--run` is a planned v2 feature; currently `sweep` prints the shell commands to run. Pipe to a shell script or execute manually.

---

## `redist tui` — Interactive terminal UI

Launches a full-screen interactive terminal UI for exploring plans, running redistricting, and reviewing analysis results without using the command line.

```
redist tui [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--no-session` | false | Start with a clean session (ignore saved config from previous runs) |

**Example**:

```bash
# Launch TUI with saved session
redist tui

# Start fresh (no previous session state)
redist tui --no-session
```

**Navigation**: Arrow keys or `hjkl` to move, `Enter` to select, `q` or `Esc` to go back, `?` for help. The TUI provides:

- **Plans screen** — browse existing plans and their status
- **Run screen** — configure and launch redistricting for a state or all states
- **Analyze screen** — run analysis on an existing plan
- **Report screen** — generate commission reports
- **Log screen** — live output from running commands

The TUI preserves session state (selected state, year, version, last command) across invocations unless `--no-session` is passed.

---

## `redist policy` — Location policy lookup

Shows redistricting rules for a state or international location: subdivision terminology, balance tolerances, VRA applicability, commission type, nesting requirements, and multi-member seat counts.

```
redist policy --state <CODE> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--state <CODE>` | *(required)* | State or location code (e.g., `WA`, `IE`, `MT-PARLIAMENT`, `_TEST_EL`) |
| `--format` | `table` | Output format: `table` or `json` |

**Examples**:

```bash
# Show Washington state policy as table
redist policy --state WA

# Ireland policy (international STV system)
redist policy --state IE

# Malta parliament policy
redist policy --state MT-PARLIAMENT

# Machine-readable JSON (for scripting)
redist policy --state CA --format json

# Test/synthetic location
redist policy --state _TEST_EL
```

**Expected output** (table format, Washington):

```
Location: Washington (WA)
Congressional districts (2020): 10
House districts: 98
Senate districts: 49
Subdivision term: county / counties
Balance tolerance (congressional): 0.5%
Balance tolerance (house/senate):  5.0%
Population basis: total
VRA Section 2 applies: yes
Commission type: bipartisan commission
Nesting requirement: house within senate (2:1 ratio)
```

**Expected output** (table format, Ireland):

```
Location: Ireland (Dail Eireann) (IE)
Constituencies (2022): 43
Total seats: 174
Seats per constituency: 3-5
Electoral system: single_transferable_vote
Balance tolerance: 16.0%
Balance formula: population_per_seat
Commission type: electoral_commission
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `REDIST_PYTHON` | Python binary to use for the pkl shim (default: `python`). Set if `python` on `PATH` doesn't have the right packages. |
| `REDIST_GH` | `gh` binary for `--release` downloads (default: `gh`). Useful for testing with a fake gh script. |
| `REDIST_MANIFEST` | Path to a custom manifest JSON file (overrides the embedded manifest). |
| `REDIST_LOCATION_POLICY` | Path to a custom `location_policy.json` (overrides the embedded policy). |

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
| VRA | `metis-vra` | Boosts edges between minority-heavy tracts to encourage majority-minority districts |

VRA mode requires demographic CSV files in `data/{year}/demographics/`.

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
: The state code is not recognized. Use two-letter USPS codes: `VT`, `CA`, `TX`. For international locations, use codes from `location_policy.json` (e.g., `IE`, `MT-PARLIAMENT`).

**`ERROR: adjacency file not found`**
: Run `redist fetch --release` to download adjacency files, then
  `python scripts/data/generate_adj_bin.py` to convert them. For international locations, use `--adjacency` to provide the path directly.

**`gpmetis: command not found`**
: Install METIS: `conda install -c conda-forge metis`

**`WARNING: falling back to pkl shim`**
: The `.adj.bin` file is missing. Run `generate_adj_bin.py` to create it, or
  ignore the warning — the pkl shim works correctly, just ~10% slower.

**`ERROR: unsupported year 'X'`**
: Valid years are `2020`, `2010`, `2000`, and (for `run`/`fetch`) `all`. International locations use country-specific years (e.g., `2022` for Ireland).

**`redist verify` FAIL**
: Jaccard similarity below threshold. Possible causes: different `gpmetis` binary version, different OS, or non-deterministic METIS behavior. Try `--skip-binary-check` to isolate binary differences from algorithmic differences.

**`redist sweep` shows commands but doesn't run them**
: The `--run` flag is a planned v2 feature. Copy the printed commands and run them manually, or pipe to bash: `redist sweep --state VT | bash`
