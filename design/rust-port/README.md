# Rust CLI Port Plan

**Status**: Planning  
**Author**: Gio Della-Libera  
**Date**: 2026-04-24  
**Scope**: Progressive replacement of the Python redistricting pipeline with a Rust CLI

---

## Why Rust

The Python pipeline works. The Rust port is not a rewrite-for-the-sake-of-it; it is targeted at three specific pain points:

1. **METIS startup latency.** Each state redistricting run spawns a subprocess (`gpmetis`). At 50 states × 3 years × multiple rounds, the subprocess overhead is measurable. A native Rust wrapper calling METIS as a linked library (or reimplementing the graph partitioning kernel) eliminates it.

2. **Adjacency graph build time.** `adjacency.py` (661 lines) does spatial joins in Python/GeoPandas. For 50 states it takes ~20 minutes. The inner loop — finding shared tract boundaries — is a candidate for Rust with SIMD-friendly geometry operations.

3. **Deployment portability.** The current pipeline requires Python 3.13, METIS binary, GeoPandas, and a complex dependency chain. A single `redist` binary with embedded logic would make court-submission reproducibility straightforward: copy one file, run it.

PyO3 bindings mean the Python pipeline continues working throughout the migration. No big-bang rewrite.

---

## Architecture: 5-Crate Workspace

```
redist/                          # Cargo workspace root
├── Cargo.toml
├── crates/
│   ├── redist-core/             # Algorithm kernel (no I/O)
│   ├── redist-data/             # Census data loading and adjacency
│   ├── redist-cli/              # Binary: `redist` command
│   ├── redist-analysis/         # Compactness, demographics, partisan metrics
│   └── redist-web/              # Static dashboard generation
└── python/
    └── redist_py/               # PyO3 bindings (wraps redist-core)
```

### Crate responsibilities

| Crate | Python equivalent | I/O | Key types |
|---|---|---|---|
| `redist-core` | `src/apportionment/partition/` | None | `Graph`, `Partition`, `EdgeWeights`, `BisectionTree` |
| `redist-data` | `src/apportionment/data/` + TIGER shapefiles | Read-only | `TractGraph`, `Demographics`, `Adjacency` |
| `redist-cli` | `scripts/pipeline/run_*.py` | Full | `RedistArgs`, `PipelineConfig` |
| `redist-analysis` | `scripts/pipeline/analyze_*.py` + compactness/ | Read/Write | `CompactnessReport`, `VraAnalysis` |
| `redist-web` | `scripts/web/` | Write | `DashboardBuilder` |

---

## Phase Plan

### Phase 0 — Scaffold (1 week)
**Goal**: Cargo workspace boots, PyO3 bindings compile, Python tests pass against Rust core.

- [ ] `cargo init` workspace with 5 crate stubs
- [ ] `redist-core`: `Graph` struct (adjacency list + vertex weights), `Partition` struct (tract→district map), round-trip serialize to/from Python dict
- [ ] `redist_py`: PyO3 module exposing `Graph.from_dict()` and `Partition.to_dict()`
- [ ] CI: add `cargo test` + `maturin develop` to `.github/workflows/`
- [ ] Python: replace `RecursiveBisection.__init__` graph-loading with `redist_py.Graph.from_dict()` — no behavioral change, just proves the binding works

**Exit criterion**: `pytest tests/unit/test_vra_edge_weighting.py` passes with Rust graph loading.

---

### Phase 1 — Core Algorithm in Rust (3 weeks)
**Goal**: `redist-core` contains the full recursive bisection + edge-weighting logic. Python calls it via PyO3. METIS still invoked as subprocess.

#### 1a. Edge weight construction
Port `run_state_redistricting.py:220-260` (the VRA edge weight loop) to Rust:
```rust
// redist-core/src/vra.rs
pub fn build_vra_edge_weights(
    graph: &Graph,
    minority_fracs: &[f64],
    threshold: f64,
) -> EdgeWeights {
    let minority_frac_state = minority_fracs.iter()
        .filter(|&&f| f >= threshold)
        .count() as f64 / minority_fracs.len() as f64;
    let alpha = (3.0_f64).max(10.0 * (1.0 - 0.7 * minority_frac_state));
    // ...
}
```
This is the adaptive boost formula (Eq. adaptive_boost in D.0). Rust gives us a single ground-truth implementation that both the CLI and the Python pipeline call.

#### 1b. Bisection tree
Port `recursive_bisection.py:RecursiveBisection` tree-building logic. The bisection tree determines the split order; it is pure computation with no I/O.

#### 1c. METIS wrapper
Keep `gpmetis` as a subprocess call for now (Phase 2 replaces it). Port the wrapper to Rust so it constructs the `.graph` file, invokes `gpmetis`, and parses the `.part` output — all without Python overhead.

#### 1d. Population balance checker
Port `_check_balance` from `test_vra_pipeline_balance.py` into `redist-core` as a first-class function (not just a test). Every partition produced by the pipeline asserts its own balance before returning.

**Exit criterion**: `pytest tests/unit/ tests/integration/test_vra_edge_weighting.py` passes. `run_state_redistricting.py` still works, now delegating to `redist_py`.

---

### Phase 2 — Data Layer in Rust (2 weeks)
**Goal**: `redist-data` replaces GeoPandas adjacency build. The Python pipeline loads adjacency pkl files that were built by Rust.

#### 2a. TIGER shapefile reader
Read TIGER/Line `.shp` / GeoParquet (whichever format the pipeline uses) using the `geo` crate. No GeoPandas dependency needed.

#### 2b. Shared-boundary detection
The hot loop in `adjacency.py:build_adjacency_graph()` — finding which tract pairs share a boundary segment — is O(n²) in Python but can be O(n log n) with a sweep-line or R-tree index. Port to Rust using `rstar` (R-tree) for spatial indexing.

Minimum boundary length filter (default 10m, current default in pipeline) becomes a compile-time-checked constant with a runtime override flag.

#### 2c. Water adjacency / island bridging
Port `adjacency_county_bridge.py` logic. This handles Hawaii, Alaska island components — the county-bridge heuristic that finds nearest mainland tract.

#### 2d. Serialization
Write adjacency as both `.pkl` (for Python backward compat) and `.bin` (fast binary format for Rust-to-Rust round trips). The `.pkl` writer can be a thin Python shim.

**Exit criterion**: Adjacency graphs produced by Rust are byte-identical (within floating-point tolerance) to current Python-produced graphs. Validated by running `pytest tests/integration/test_output_completeness.py` on both sets.

---

### Phase 3 — CLI Binary (2 weeks)
**Goal**: `redist run --state AL --year 2020 --version V3` works end-to-end from the `redist` binary. Python pipeline remains fully functional in parallel.

#### 3a. Argument parsing
Mirror the current flag surface exactly:
```
redist run [--state <STATE>] [--year <YEAR>] [--version <VERSION>]
           [--partition-mode <MODE>] [--states <LIST>]
           [--reprocess] [--skip-analysis]
```
Use `clap` with derive macros. All flags documented with the same semantics as the Python `argparse` equivalents.

#### 3b. Progress reporting
Implement the STATUS protocol (`STATUS:{pos}:{msg}`) in Rust so the Rust CLI integrates with the existing Python progress coordinator without changes.

#### 3c. Multi-state parallelism
Replace `run_states_parallel.py` (Python multiprocessing) with Rayon parallel iterator across states. Each state runs in its own thread with independent METIS invocations.

#### 3d. Output format parity
Rust CLI writes the same directory structure and file formats as the Python pipeline:
- `states/{state}/data/district_summary.csv`
- `states/{state}/data/final_assignments.pkl` (via PyO3 pickle shim)
- `states/{state}/data/vra_analysis.json`

**Exit criterion**: `redist run --state VT --year 2020 --version V4 --partition-mode metis-vra` produces outputs that pass `pytest tests/integration/test_vra_pipeline_balance.py::TestVRACodePathIntegrity`.

---

### Phase 4 — Analysis and Dashboard (2 weeks)
**Goal**: `redist analyze` and `redist dashboard` work from the binary. Python scripts deprecated but not deleted.

#### 4a. Compactness metrics
Port Polsby-Popper and Reock computation to `redist-analysis`. These are pure geometry — area and perimeter from dissolved district polygons. Use `geo` crate's area/perimeter implementations.

#### 4b. VRA analysis
Port `vra_utils.py:analyze_mm_districts()` to Rust. The `vra_analysis.json` file is now written by `redist-core`, not by a Python post-processing step — eliminating the vra_mode premature-clear pitfall entirely (the analysis runs atomically with the partition).

#### 4c. Static dashboard
Port `scripts/web/deploy_docs.py` HTML generation to `redist-web`. The dashboard is a self-contained HTML file with base64-embedded maps; Rust can generate this deterministically with no JavaScript build step.

**Exit criterion**: `redist analyze --state AL --version V4` produces `vra_analysis.json` matching current V4 output. `redist dashboard --version V4 --year 2020` produces a dashboard that passes visual review.

---

### Phase 5 — Native METIS (stretch goal, 4+ weeks)
**Goal**: Replace `gpmetis` subprocess with a Rust implementation of multilevel k-way graph partitioning.

This is optional and separate from the main port. The `gpmetis` subprocess works fine; this is purely a performance and portability improvement.

Options:
- **Link METIS as a C library** via `bindgen` — simplest, keeps METIS algorithms, eliminates subprocess overhead
- **Port METIS kernels to Rust** — full control, no C dependency, but significant work (coarsening, refinement, FM passes)
- **Use an existing Rust graph partitioning crate** — if one exists at sufficient quality

Decision gate: after Phase 4 is complete, profile actual subprocess overhead across a 50-state run. If it's <5% of total runtime, skip Phase 5.

---

## PyO3 Binding Strategy

The Python pipeline calls Rust throughout the migration. The binding surface is intentionally narrow:

```python
# python/redist_py/__init__.py (generated by maturin)
from .redist_py import (
    Graph,          # adjacency graph
    Partition,      # district assignments
    build_vra_edge_weights,   # Phase 1
    build_adjacency_graph,    # Phase 2
    analyze_mm_districts,     # Phase 4
)
```

Rules:
1. **No Python objects cross the boundary except dicts and numpy arrays.** No GeoPandas DataFrames, no Shapely geometries — convert to plain types before calling Rust.
2. **Rust functions are pure.** No side effects, no file I/O through PyO3. The Python side handles I/O; Rust handles computation.
3. **Error propagation via `PyErr`.** All Rust errors convert to typed Python exceptions (`ValueError`, `RuntimeError`) with messages matching the current Python error strings so existing error handling works.

---

## CI Integration

Add to `.github/workflows/tests.yml`:

```yaml
rust-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --workspace
    - run: pip install maturin && maturin develop
    - run: pytest tests/unit/test_vra_edge_weighting.py tests/unit/test_pipeline_flag_propagation.py
```

The Rust tests and Python-via-PyO3 tests run in the same CI job. If Rust compiles but PyO3 breaks Python tests, the job fails.

---

## Migration Invariants (BOUNDARY role)

These must hold at every phase boundary:

1. **Population balance**: Every partition produced by Rust must pass `±0.5%` check before writing output. Enforced in `redist-core::Partition::assert_balanced()`.
2. **VRA analysis atomicity**: `vra_analysis.json` is written in the same transaction as `final_assignments.pkl`. If the process dies after writing one but not the other, the state directory is marked corrupt and must be reprocessed. No more `vra_mode` premature-clear class of bugs.
3. **Output format parity**: Python and Rust pipelines must produce byte-compatible CSVs (within floating-point string formatting tolerance). Validated by the integration test suite.
4. **No silent fallback**: If Rust code is unavailable (e.g., PyO3 module not compiled), the Python pipeline raises `ImportError` immediately rather than silently falling back to slower Python. Opt-out via `REDIST_NO_RUST=1` env var.

---

## File Layout

```
design/rust-port/
├── README.md             ← this file
├── api.md                ← PyO3 API surface (written during Phase 0)
├── benchmarks.md         ← performance targets and measurements
└── migration-log.md      ← per-phase completion notes
```

---

## Dependencies

| Crate | Purpose | Alternative considered |
|---|---|---|
| `clap` (derive) | CLI argument parsing | `argh` — rejected, less ergonomic |
| `rayon` | Data-parallel state processing | `tokio` — rejected, CPU-bound not I/O-bound |
| `geo` | Geometry types, area/perimeter | `geos` (C bindings) — rejected, portability |
| `rstar` | R-tree spatial index | `kdtree` — rejected, no bounding-box queries |
| `serde` + `serde_json` | JSON serialization | — |
| `csv` | CSV read/write | — |
| `pyo3` | Python bindings | `cffi` — rejected, no type safety |
| `maturin` | Build PyO3 wheel | `setuptools-rust` — rejected, less ergonomic |
| `numpy` (via `numpy` crate) | numpy array interop in PyO3 | — |

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| METIS C library linking fails on Windows | Medium | Fallback to subprocess in Phase 1-4; Phase 5 is optional |
| PyO3 ABI breaks on Python 3.13 | Low | Pin to stable ABI (`abi3`); test in CI |
| Rust adjacency output diverges from Python on edge cases (water, islands) | Medium | Byte-comparison integration test across all 50 states before retiring Python |
| Polsby-Popper values diverge due to floating-point differences | Low | Accept tolerance of 1e-6; flag any deviation >1e-4 |
| Parallel METIS invocations hit gpmetis process limit | Low | Rayon thread pool capped at `min(workers, 8)` |
