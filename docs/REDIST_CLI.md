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
| `-m`, `--partition-mode` | `edge-weighted` | `unweighted`, `edge-weighted`, `metis-vra` |
| `--ufactor` | `5` | METIS imbalance tolerance (5 = ±0.5%) |
| `--niter` | `100` | METIS refinement iterations |
| `--seed` | *(random)* | METIS random seed for reproducible runs |
| `-r`, `--reset` | false | Delete output directory before starting |
| `-d`, `--debug` | false | Extra diagnostic output |
| `-p`, `--print-only` | false | Dry run — print actions without executing |

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
| `--type <TYPES>` | `all` | `tiger`, `redistricting`, `adjacency`, or `all` |
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
