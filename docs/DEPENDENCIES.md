# Dependencies

**Last Updated**: 2026-04-25

Installation guide for all project dependencies.

---

## Two paths

| Path | Who | What you need |
|------|-----|---------------|
| **Rust CLI** (recommended) | Running redistricting | Rust toolchain + METIS |
| **Python pipeline** | Development, analysis, post-processing | Python 3.13+ + geospatial stack |

Most users only need the Rust CLI path.

---

## Path 1: Rust CLI (Recommended)

### 1. Rust toolchain

Install from [rustup.rs](https://rustup.rs):

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows — download and run rustup-init.exe from rustup.rs
# Then restart your shell.
```

Verify:
```bash
rustc --version   # should be 1.75+ 
cargo --version
```

### 2. METIS graph partitioning library

METIS is required for redistricting (the binary invokes `gpmetis`).

**Windows/macOS/Linux — Conda (easiest)**:
```bash
conda install -c conda-forge metis

# Verify
gpmetis --help
```

**Linux (apt)**:
```bash
sudo apt-get install metis
```

**macOS (Homebrew)**:
```bash
brew install metis
```

### 3. Build the binary

```bash
cargo build --release --manifest-path redist/Cargo.toml
# Binary: redist/target/release/redist.exe (Windows) or redist/target/release/redist (Linux/macOS)
```

Add to PATH so `redist` works anywhere:
```bash
# Windows PowerShell (current session)
$env:PATH += ";$PWD\redist\target\release"

# Linux/macOS (add to ~/.bashrc or ~/.zshrc)
export PATH="$PWD/redist/target/release:$PATH"
```

### 4. Verify

```bash
redist --version
gpmetis --help | head -1
```

### 5. Download data and run

```bash
redist fetch --year 2020                              # Download census data
redist fetch --year 2020 --release                   # + adjacency files from GitHub Releases
python scripts/data/generate_adj_bin.py --year 2020  # Convert to fast native format

redist state --state VT --year 2020 --version V3     # Single state smoke test
redist states --year 2020 --version V3 \
  --output-dir outputs/V3 --workers 8                # All 50 states
```

See [`docs/REDIST_CLI.md`](REDIST_CLI.md) for full command reference.

---

## Path 2: Python Pipeline (Development / Analysis)

Required if you are:
- Developing or modifying the Python pipeline
- Running post-processing analysis scripts (`scripts/political/`, `scripts/demographic/`, `scripts/compactness/`)
- Building adjacency graphs from raw TIGER shapefiles

### System requirements

| Spec | Minimum | Recommended |
|------|---------|-------------|
| Python | 3.13 | 3.13 |
| RAM | 8 GB | 16 GB |
| Disk | 30 GB | 60 GB |
| CPU cores | 2 | 8+ |

### Install Python dependencies

**Conda (recommended on Windows — avoids GDAL/GEOS issues)**:

```bash
conda create -n redistricting python=3.13
conda activate redistricting

# Geospatial stack
conda install -c conda-forge geopandas pyarrow networkx matplotlib tqdm

# METIS bindings (in addition to the conda-forge METIS binary above)
conda install -c conda-forge metis
pip install pymetis

# HTTP + other utilities
pip install requests
```

**pip only (Linux/macOS)**:

```bash
python -m venv venv
source venv/bin/activate

pip install geopandas pyarrow networkx matplotlib tqdm requests pymetis
```

### Core Python packages

| Package | Version | Purpose |
|---------|---------|---------|
| `geopandas` | ≥0.14 | Tract geometries, adjacency calculation |
| `pandas` | ≥2.0 | Dataframes |
| `numpy` | ≥1.26 | Numerical operations |
| `pyarrow` | ≥14.0 | Parquet I/O |
| `networkx` | ≥3.0 | Adjacency graphs |
| `matplotlib` | ≥3.8 | Map generation |
| `tqdm` | ≥4.66 | Progress bars |
| `requests` | ≥2.31 | Census API / HTTP downloads |
| `pymetis` | ≥2023.1 | Python METIS bindings (optional, used by legacy path) |

### Verify Python installation

```bash
python -c "
import geopandas, pandas, numpy, pyarrow, networkx, matplotlib, tqdm, requests
print('All core packages OK')
import shutil, sys
gpmetis = shutil.which('gpmetis')
print(f'gpmetis: {gpmetis or \"NOT FOUND\"}')
try:
    import pymetis; print('pymetis: OK')
except ImportError:
    print('pymetis: not installed (optional)')
"
```

---

## PyO3 Bindings (Rust ↔ Python, Developers Only)

Required only if you are modifying the `redist_py` Python bindings or working on
the `redist-core`/`redist-data` crates.

```bash
pip install maturin

cd redist/python/redist_py
maturin develop          # Build and install in the current Python env
python -c "import redist_py; print('PyO3 bindings OK')"
```

---

## Census API Key (Optional)

Required only for downloading demographics data via the Census API.
`redist fetch` handles TIGER and PL 94-171 data without an API key.

```bash
# Get key: https://api.census.gov/data/key_signup.html

# Configure (Linux/macOS)
export CENSUS_API_KEY="your_key_here"

# Configure (Windows PowerShell)
$env:CENSUS_API_KEY = "your_key_here"
```

---

## Troubleshooting

**`gpmetis: command not found`**
: Install with `conda install -c conda-forge metis` or `brew install metis`.

**`error[E0463]: can't find crate` during `cargo build`**
: Run `rustup update stable` to ensure you have a recent Rust compiler.

**`OSError: Could not find libgdal`** (Windows)
: Use `conda install -c conda-forge geopandas` instead of pip.

**`ImportError: cannot import name 'X' from geopandas`**
: Upgrade: `conda update -c conda-forge geopandas`.

**`MemoryError` during large state processing**
: Process states with fewer workers: `redist states --workers 2`.

---

## See Also

- [`REDIST_CLI.md`](REDIST_CLI.md) — CLI commands and options
- [`GETTING_STARTED.md`](GETTING_STARTED.md) — First-run walkthrough
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — Development workflow
