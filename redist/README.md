# redist — Algorithmic Redistricting CLI

Compact, population-balanced congressional districts for all 50 US states — 839 tests, ~15 seconds for a full 50-state run, no Python required for core redistricting.

## What it does

`redist` draws congressional districts using METIS recursive bisection. It enforces ±0.5% population balance, contiguity, and compactness for every district — with no political or racial data in the partitioning step. Outputs include assignment files, district maps (PNG), and per-district analytics (demographic, partisan, compactness). The tool also supports international electoral systems (Ireland STV, Malta STV, Germany Bundestag) and multi-chamber plans (house + senate + congressional).

Research papers covering the algorithm and validation live in [`research/`](../research/), including multi-member extensions (E.1), international applications (E.6), and MAUP sensitivity analysis (C.1).

## Quickstart (5 commands)

```bash
# 1. Build
cargo build --release -p redist-cli -p redist-tui

# 2. Download adjacency data for Vermont (small state, fast)
./target/release/redist fetch --type adjacency --states VT --year 2020

# 3. Run redistricting
./target/release/redist state --state VT --year 2020 --label vt_test

# 4. Analyze + report
./target/release/redist analyze --label vt_test --types all
./target/release/redist report --label vt_test --format html

# 5. Interactive TUI
./target/release/redist tui
```

## Key commands

| Command | What it does |
|---------|-------------|
| `redist state` | Redistrict a single state |
| `redist states` | All 50 states in parallel (~15 s) |
| `redist run` | Multi-year orchestrator (2000/2010/2020) |
| `redist fetch` | Download census data and adjacency files |
| `redist analyze` | Per-district analytics (demographic, partisan, compactness) |
| `redist compare` | Compare two plans: Jaccard, population, compactness |
| `redist verify` | Reproduce a plan from its manifest and confirm it matches |
| `redist doctor` | Pre-flight check: data files, year validity, resolution warnings |
| `redist sweep` | Run N seeds, keep top-K plans by a chosen metric |
| `redist report` | Generate HTML/JSON commission report |
| `redist export` | Export plan as GeoJSON, GerryChain v2.3, CSV, or reproducibility package |
| `redist policy` | Show redistricting rules for a state (tolerances, VRA, commission type) |
| `redist tui` | Interactive terminal UI |
| `redist map` | Render district maps to PNG |
| `redist suite` | Draw and validate multi-chamber nested plans |
| `redist aggregate` | Merge all state analysis into national datasets |
| `redist validate` | Validate a `.rplan` file for format correctness |
| `redist import` | Import a GeoJSON plan into RPLAN format |
| `redist migrate` | Copy a legacy state plan into the `plans/{label}/` tree |

## Data requirements

Pre-built adjacency files are required to run redistricting. Download them with:

```bash
# Adjacency graphs (required)
redist fetch --type adjacency --year 2020

# Census demographics (required for analyze)
redist fetch --type redistricting --year 2020

# Election data (optional, enables partisan analysis)
redist fetch --type elections --year 2020
```

After downloading adjacency `.pkl` files, convert to the native binary format for best performance:

```bash
python scripts/data/generate_adj_bin.py --year 2020
```

## International locations

The following non-US locations are configured in `data/location_policy.json` and work with `redist state`, `redist doctor`, and `redist policy`:

| Code | Location | System |
|------|----------|--------|
| `IE` | Ireland (Dail Eireann) | STV, 3-5 seats per constituency |
| `MT-PARLIAMENT` | Malta (House of Representatives) | STV, 5 seats per district |
| `DE` | Germany | Mixed-member proportional |

Acquisition scripts for international adjacency data: `scripts/data/international/`.

Example — Ireland (use `--seats-per-district 4` for average seat count):

```bash
redist state --state IE --year 2022 --adjacency outputs/international/ie/ie_adjacency_2022.adj.bin \
  --districts 43 --seats-per-district 4 --balance-tolerance 16.0
```

## Requirements

- **Rust 1.75+** — `cargo build --release`
- **METIS** (`gpmetis` on PATH) — `conda install -c conda-forge metis`
- **Python 3.10+** — optional; needed only for `.pkl` adjacency fallback and data download scripts

## Test suite

839 tests across 8 crates:

```bash
cargo test --workspace --lib
```

Acceptance tests (require data files):

```bash
pytest tests/acceptance/ -v
```

## Research

The `research/` directory contains papers organized by topic:

- **B series** — Algorithm design: recursive bisection, edge-weighted partitioning, complexity analysis
- **C series** — Validation: MAUP sensitivity (C.1), cross-census validation (C.2), temporal stability (C.3)
- **D series** — VRA compliance: threshold analysis, compactness tradeoffs, legal implementation
- **E series** — Extensions: multi-member districts (E.1), international applications (E.6)

## License

MIT
