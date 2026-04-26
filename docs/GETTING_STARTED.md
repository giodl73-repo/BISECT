# Getting Started

**Last Updated**: 2026-04-25

This guide walks you through setting up and running the Congressional Redistricting
System. By the end you'll have generated your first redistricting outputs.

---

## Prerequisites

### Required: METIS graph partitioning

METIS is required for all redistricting runs.

```bash
# Windows/macOS/Linux — Conda (recommended)
conda install -c conda-forge metis
gpmetis --help   # should print usage

# macOS (Homebrew)
brew install metis

# Linux (apt)
sudo apt-get install metis
```

### Required for Rust CLI (recommended path)

Install [Rust](https://rustup.rs):

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows — download rustup-init.exe from rustup.rs
```

Build the binary once:

```bash
cargo build --release --manifest-path redist/Cargo.toml
```

Add to PATH (so you can run `redist` anywhere):

```bash
# Windows PowerShell
$env:PATH += ";$PWD\redist\target\release"

# Linux/macOS (add to .bashrc/.zshrc)
export PATH="$PWD/redist/target/release:$PATH"
```

### Required for Python analysis scripts (optional)

```bash
conda create -n redistricting python=3.13
conda activate redistricting
conda install -c conda-forge geopandas pyarrow networkx matplotlib tqdm
pip install requests pymetis
```

---

## Quick Start

### Option 1: Single small state — ~30 seconds

Vermont has 1 congressional district. It's the fastest possible smoke test.

```bash
# Step 1: Clone the repo
git clone https://github.com/your-org/apportionment.git
cd apportionment

# Step 2: Build the Rust binary (one-time)
cargo build --release --manifest-path redist/Cargo.toml

# Step 3: Download Vermont census data
redist fetch --states VT --year 2020

# Step 4: Download pre-built adjacency files (requires gh auth login)
redist fetch --states VT --year 2020 --release
python scripts/data/generate_adj_bin.py --year 2020 --states VT

# Step 5: Run Vermont redistricting
redist state --state VT --year 2020 --version V3
```

**Expected output** (under `outputs/V3/2020/vermont/`):
```
[OK] VT in 512ms
```

Files created:
- `final_assignments.json` — tract → district mapping
- `vra_analysis.json` — majority-minority district analysis

### Option 2: All 50 states — ~15 seconds

```bash
# Download all 2020 data (run once)
redist fetch --year 2020
redist fetch --year 2020 --release
python scripts/data/generate_adj_bin.py --year 2020

# Run all 50 states in parallel
redist states --year 2020 --version V3 \
  --output-dir outputs/V3 --workers 8
```

**Expected output**:
```
[redist states] 50 states (8 workers)
[OK] 50/50 states complete
```

### Option 3: All 3 census years — ~45 seconds

```bash
# Download data for all years
redist fetch --year all
redist fetch --year all --release
python scripts/data/generate_adj_bin.py --year 2020
python scripts/data/generate_adj_bin.py --year 2010
python scripts/data/generate_adj_bin.py --year 2000

# Run all years
redist run --version V3 --workers 12
```

---

## Time Expectations

| Task | Time |
|------|------|
| Build Rust binary (one-time) | 1–2 min |
| Download 2020 data (50 states) | 10–30 min |
| Vermont (1 district) | ~0.5 s |
| Alabama VRA (7 districts) | ~0.65 s |
| All 50 states — 2020 | ~15 s (8 workers) |
| All 50 states × 3 years | ~45 s (12 workers) |

---

## Understanding the Output

Outputs land under `outputs/{version}/{year}/{state_name}/`:

```
outputs/V3/2020/
  ├── vermont/
  │   ├── final_assignments.json    ← tract → district map
  │   └── vra_analysis.json         ← majority-minority analysis
  ├── california/
  │   └── ...
  └── ...  (48 more states)
```

**`final_assignments.json`** — the main result:
```json
{
  "50001010100": 1,
  "50001010200": 1,
  ...
}
```
Keys are Census tract GEOIDs; values are district numbers (1-indexed).

**`vra_analysis.json`** — VRA / majority-minority analysis:
```json
{
  "mm_count": 0,
  "districts": [{"district": 1, "minority_pct": 0.063, "is_mm": false}]
}
```

### Dashboards

Interactive dashboards are at `docs/`:

| Dashboard | URL (after `git clone`) |
|-----------|------------------------|
| VRA / 2020 (V4) | `docs/dashboard_vra.html` |
| 2020 (V3) | `docs/dashboard_2020.html` |
| 2010 (V3) | `docs/dashboard_2010.html` |
| 2000 (V3) | `docs/dashboard_2000.html` |

Open any `.html` file in a browser. No server required.

To deploy updated dashboards:
```bash
python scripts/web/deploy_docs.py --version V3 --year 2020 --out docs/dashboard_2020.html
```

---

## VRA Mode (Majority-Minority Districts)

For states where the Voting Rights Act applies, use `--partition-mode metis-vra`:

```bash
redist state --state AL --year 2020 --version V3 --partition-mode metis-vra
```

VRA mode uses demographic data (`data/{year}/demographics/`) to boost edge weights
between minority-heavy tracts, encouraging majority-minority districts.

---

## Resume Interrupted Runs

`redist states` and `redist run` skip states that already have `final_assignments.json`.
If a run is interrupted, just re-run the same command — completed states are skipped.

```bash
# Resume where you left off
redist states --year 2020 --version V3 --output-dir outputs/V3 --workers 8

# Force re-run of all states
redist states --year 2020 --version V3 --output-dir outputs/V3 --reprocess
```

---

## Python Pipeline (Alternative)

The Python pipeline produces maps and analysis that the Rust CLI doesn't yet generate
(matplotlib maps, political/demographic/compactness CSVs). It's slower but more complete.

```bash
# Activate Python environment
conda activate redistricting

# Single state
python scripts/pipeline/run_state_redistricting.py \
  --state VT --year 2020 --version v1 \
  --output-dir outputs/python_test

# All 50 states, single year
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version v1 --workers 4

# Multi-year parallel (2020 + 2010 + 2000)
python scripts/pipeline/run_complete_redistricting.py --version v1
```

**Time expectations (Python)**:
| Task | Time |
|------|------|
| Single state (VT) | ~4.5 s |
| Single state (CA) | ~45 s |
| All 50 states | ~55 min |
| All 3 years | 2–4 h |

---

## Common Issues

**`gpmetis: command not found`**
: Install METIS: `conda install -c conda-forge metis`

**`error: no such file or directory: redist/target/release/redist`**
: Build the binary first: `cargo build --release --manifest-path redist/Cargo.toml`

**`ERROR: adjacency file not found`**
: Download adjacency files with `redist fetch --year 2020 --release`, then convert:
  `python scripts/data/generate_adj_bin.py --year 2020`

**`WARNING: falling back to pkl shim`**
: The `.adj.bin` file is missing for this state. Run `generate_adj_bin.py` to fix.
  The pkl shim still works — this is a warning, not an error.

**`redist fetch --release` fails (requires gh)**
: Install and authenticate: `gh auth login`. Or skip `--release` and build adjacency
  from TIGER data with the Python scripts.

**Python: `ImportError: cannot import name 'X'`**
: Check you have Python 3.13+ and the packages from the conda environment:
  `conda activate redistricting && conda list geopandas`

---

## Next Steps

- **Full CLI reference**: [`docs/REDIST_CLI.md`](REDIST_CLI.md)
- **Algorithm explanation**: [`docs/RECURSIVE_BISECTION.md`](RECURSIVE_BISECTION.md)
- **Output data fields**: [`docs/DATA_DICTIONARY.md`](DATA_DICTIONARY.md)
- **Troubleshooting**: [`docs/TROUBLESHOOTING.md`](TROUBLESHOOTING.md)
- **Contributing**: [`docs/CONTRIBUTING.md`](CONTRIBUTING.md)
- **Dependencies detail**: [`docs/DEPENDENCIES.md`](DEPENDENCIES.md)
