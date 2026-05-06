# Claude AI Guide — Redistricting Platform

**Updated**: 2026-05-06

## What this is

Congressional and state legislative redistricting via METIS recursive bisection. 50 states, 435 districts, 3 census years. Purely algorithmic. Production tool is the `redist` Rust binary (~213× faster than the archived Python pipeline).

## Key directories

```
redist/crates/          # Rust workspace (redist-cli, redist-core, redist-data, redist-analysis,
                        #   redist-map, redist-report, redist-apportion, redist-metis, redist-ensemble)
docs/                   # Human-facing docs (REDIST_CLI.md, PAPERS.md, concepts/, quickstart/, legal/)
docs/papers/            # 63+ compiled PDFs (committed)
research/               # LaTeX sources for all papers (A–H series)
scripts/                # Python: data download, elections, dashboard generation only
configs/                # YAML plan configs (configs/{label}.yml)
runs/ analysis/ reports/ # Build outputs (gitignored)
data/{year}/            # Raw census data (gitignored, ~55GB)
archive/python-pipeline-final/  # Sealed forensic reference — do not touch
```

## Core commands

```bash
# Build a plan (reads configs/{label}.yml)
redist build <label> --year 2020 --workers 8

# Analysis + report + verify SHA chain
redist label-analyze <label> --year 2020 --types all
redist label-report  <label> --year 2020 --format html json
redist label-verify  <label> --year 2020

# List / inspect
redist ls
redist show <label>

# Single state (for development)
redist state --state VT --year 2020 --version v_test

# Fetch census data
redist fetch --year 2020 --workers 8
```

## Three-layer compositor

Every run is defined by three flags:

| Flag | Controls | Key values |
|------|----------|-----------|
| `--structure` | Bisection tree shape | `standard-bisect`, `prime-factor` (ApportionRegions), `ratio-optimal` (GeoSection), `ratio-optimal-area`, `ratio-optimal-vra`, `nway` |
| `--weights-override` | METIS edge/vertex signal | `geographic` (default), `county`, `unweighted`, `vra-aligned`, `proportional` |
| `--search` | Seed selection strategy | `single`, `multi`, `convergence` (T=600 default), `percentile`, `bisection-ensemble` |

YAML config example:
```yaml
name: official_2020
algorithm:
  structure: prime-factor
  weights: county
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 8
years: ["2020", "2010", "2000"]
```

## METIS engines

- `--metis-engine c-ffi` — default, bundled C METIS (requires C compiler at build time)
- `--metis-engine redist-metis` — pure Rust, no C dependency (portable binary)
- `cargo build --no-default-features` — builds the portable pure-Rust binary

## Git rules

- ⚠️ NEVER commit: `data/`, `outputs/`, `*.{png,jpg,pdf}` (except `docs/`)
- Windows: ASCII only in console output — NO Unicode (CP1252 crashes)
- State names in code: `lowercase_underscores` (`north_carolina`, not `NC`)

## Testing

```bash
cargo test -p redist-cli --lib -- --test-threads=1  # inline unit tests (CWD-sensitive)
cargo test -p redist-ensemble                        # 61 tests: L0/L1/L2
pytest tests/unit/ -v                               # Python unit tests
```

Add tests for every new feature: L0 (inline unit), L1 (integration, synthetic data), L2 (real data, `#[ignore]`).

## Key docs

- `docs/REDIST_CLI.md` — complete CLI reference
- `docs/PAPERS.md` — all 63+ papers with PDFs, organised by question
- `docs/concepts/` — algorithm guides (three-layer compositor, section algorithms, label pipeline, ensemble methods)
- `docs/quickstart/` — persona guides (special master, researcher, state staff, algorithm explorer, federal statute)
- `docs/legal/` — model federal statute, fairness doctrine
- `README.md` — public-facing overview
