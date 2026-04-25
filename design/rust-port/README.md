# Rust CLI Port Plan

**Status**: Planning — reviewed by cartographic roles 2026-04-24  
**Author**: Gio Della-Libera  
**Scope**: Progressive replacement of the Python redistricting pipeline with a Rust CLI

---

## Why Rust

The Python pipeline works. The Rust port targets three specific pain points:

1. **Adjacency build time.** `adjacency.py` (661 lines) builds shared-boundary graphs in Python/GeoPandas. The bottleneck is the per-edge Shapely `intersection()` call — O(E) calls, each expensive. For 50 states this takes ~20 minutes. Rust parallelizes this across cores with no GIL.

2. **Deployment portability.** The current pipeline requires Python 3.13, METIS binary, GeoPandas, and a complex dependency chain. A single `redist` binary makes court-submission reproducibility straightforward.

3. **Algorithm ground truth.** The adaptive boost formula (`max(3.0, 10.0*(1-0.7*f))`) currently lives only in `run_state_redistricting.py`. Moving it to `redist-core` makes it a single authoritative implementation shared by the Python pipeline (via PyO3) and the Rust CLI — eliminating the paper/code drift that the D-track review found.

Note: METIS startup latency is *not* a primary pain point. Each `gpmetis` subprocess is fast; 50-state parallelism already limits wall time to the slowest state. Phase 5 (native METIS) is optional.

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

### PyO3 Binding Rules

1. **No Python objects cross the boundary except dicts and numpy arrays.** No GeoPandas DataFrames, no Shapely geometries — convert to plain types before calling Rust.
2. **Rust functions are pure for computation; `redist-cli` owns I/O.** PyO3-callable functions have no file I/O side effects. File writes happen in the CLI binary or in Python, never through PyO3.
3. **Error propagation via `PyErr`.** All Rust errors convert to typed Python exceptions with messages matching current Python error strings.
4. **VRA analysis written by `redist-cli`, not PyO3.** To achieve atomicity (see Migration Invariants), the CLI writes both `final_assignments` and `vra_analysis.json` together. PyO3 returns computation results; the binary handles the atomic write.

---

## Phase Plan

### Phase 0 — Scaffold (1 week)
**Goal**: Cargo workspace boots, PyO3 bindings compile, a new test verifies graph round-trip.

- [ ] `cargo init` workspace with 5 crate stubs
- [ ] `redist-core`: `Graph` struct (CSR adjacency list + vertex weights), `Partition` struct (tract→district map), round-trip serialize to/from Python dict
- [ ] `redist_py`: PyO3 module exposing `Graph.from_csr()` and `Partition.to_dict()`
- [ ] **Write new test** `tests/unit/test_rust_graph.py`: construct `redist_py.Graph` from a known 5-vertex CSR dict, assert `n_vertices()==5`, `n_edges()==correct`. This is the Phase 0 exit criterion test.
- [ ] CI: add `rust-tests` job to `.github/workflows/` (see CI section below)
- [ ] Set `REDIST_NO_RUST=1` in the **existing** `test` CI job so it keeps passing without Rust compiled
- [ ] Python: replace `RecursiveBisection.__init__` graph-loading with `redist_py.Graph.from_csr()` only when `REDIST_NO_RUST` is unset

**Exit criterion**: `pytest tests/unit/test_rust_graph.py` passes (new test). Existing `tests/unit/` suite passes in both `REDIST_NO_RUST=1` mode (CI `test` job) and Rust-compiled mode (CI `rust-tests` job).

---

### Phase 1 — Core Algorithm in Rust (3 weeks)
**Goal**: `redist-core` contains edge-weight construction and bisection tree logic. Python calls it via PyO3. METIS still invoked as subprocess.

#### 1a. Edge weight construction
Port the VRA edge weight loop from `run_state_redistricting.py:226-260` to Rust. The full loop — not just the scalar formula — is required:

```rust
// redist-core/src/vra.rs
pub fn build_vra_edge_weights(
    edges: &[(usize, usize)],
    minority_fracs: &[f64],   // per-tract fractions; len == n_vertices
    threshold: f64,
) -> HashMap<(usize, usize), f64> {
    // Step 1: compute state-level minority tract fraction
    let n_minority = minority_fracs.iter().filter(|&&f| f >= threshold).count();
    let minority_frac_state = n_minority as f64 / minority_fracs.len() as f64;
    let alpha = (3.0_f64).max(10.0 * (1.0 - 0.7 * minority_frac_state));

    // Step 2: boost only minority-to-minority edges (both endpoints above threshold)
    edges.iter()
        .filter(|&&(u, v)| minority_fracs[u] >= threshold && minority_fracs[v] >= threshold)
        .map(|&(u, v)| ((u, v), alpha))
        .collect()
}
```

Once this function exists in `redist-core`, **delete** `run_state_redistricting.py:226-260` and replace with `redist_py.build_vra_edge_weights(...)`. The formula then lives in exactly one place.

#### 1b. Bisection tree
Port `RecursiveBisection` tree-building logic. Important: the current Python implementation is **level-parallel** — it processes all nodes at a given depth simultaneously via `ProcessPoolExecutor`. The Rust port must also be level-parallel, not depth-first recursive. Use Rayon for parallel splits at each level. A depth-first port would differ in split order and produce different (not just slower) results.

#### 1c. METIS wrapper
Port `metis_wrapper.py` to Rust: construct the METIS `.graph` file, invoke `gpmetis` subprocess, parse `.part` output. Note: the current primary path uses `pymetis` (a Python library), which does **not** pass `-contig` or `-minconn` flags. The subprocess `gpmetis` path does. The Rust wrapper uses the subprocess path. This is a **behavioral change**: Rust will use `-contig` by default while the Python/pymetis path does not. Document this explicitly.

#### 1d. Population balance checker
Add `Partition::assert_balanced(vertex_weights, n_districts, tolerance)` to `redist-core`. Call it only on **final** leaf partitions (not intermediate bisection nodes, which use their own tighter `ufactor` tolerance). Called from both PyO3 (when Python pipeline finalizes a state) and from `redist-cli` (Phase 3).

**Exit criterion**: 
- `pytest tests/unit/` passes with `REDIST_NO_RUST` unset (Rust formula called)
- `run_state_redistricting.py` still works end-to-end for VT and AL
- `run_state_redistricting.py:226-260` deleted (only one copy of formula exists)

---

### Phase 2 — Data Layer in Rust (2 weeks)
**Goal**: `redist-data` replaces GeoPandas for adjacency computation. The bottleneck is parallelizing per-edge intersection calls, not replacing the STR-tree spatial index (libpysal already uses one).

#### 2a. TIGER shapefile reader
Read TIGER/Line `.shp` or GeoParquet using the `shapefile` crate (pure Rust, no GDAL dependency). Output: array of WKB-encoded polygons + GEOID strings.

Note: until Phase 2a is complete, Python still reads shapefiles via GeoPandas and passes WKB to Rust via `build_adjacency_graph()`. GeoPandas cannot be removed as a dependency until 2a is complete.

#### 2b. Boundary detection and intersection
Port `adjacency.py:_compute_boundary_lengths()`. The actual bottleneck is the per-edge Shapely `intersection()` call (not spatial index construction, which libpysal already handles with STR-tree). The Rust approach:
1. Use `rstar` to find candidate neighbor pairs (equivalent to libpysal Queen contiguity)
2. Parallelize the `geo::BooleanOps::intersection()` calls across edges using Rayon
3. Filter by `min_boundary_length` (default 10m)

#### 2c. Water adjacency / island bridging
Port `adjacency_county_bridge.py` — nearest-tract connection for disconnected components (Hawaii, Alaska islands).

#### 2d. Serialization and backward compat
Write adjacency as `.bin` (fast binary, Rust-to-Rust). For Python backward compat during migration, provide a separate Python script `scripts/data/convert_adj_bin_to_pkl.py` that reads `.bin` and writes `.pkl`. This is **not** a PyO3 function — it is a standalone conversion script, keeping Rule 2 clean.

**New test required** (write before marking Phase 2 complete):  
`tests/integration/test_adjacency_parity.py` — build adjacency for a small state (VT or DE) in both Python and Rust, compare edge sets and boundary lengths with tolerance 1e-4 m.

**Exit criterion**: `pytest tests/integration/test_adjacency_parity.py` passes for VT and DE. Existing integration tests pass with Rust-produced adjacency.

---

### Phase 3 — CLI Binary (2 weeks)
**Goal**: `redist run --state AL --year 2020 --version V3` works end-to-end from the `redist` binary.

#### 3a. Argument parsing
Mirror the current flag surface exactly using `clap` derive macros:
```
redist run [--state <STATE>] [--year <YEAR>] [--version <VERSION>]
           [--partition-mode <MODE>] [--states <LIST>]
           [--reprocess] [--skip-analysis]
```

#### 3b. Progress reporting
Implement the STATUS protocol (`STATUS:{pos}:{msg}`, ASCII only) in Rust.

#### 3c. Multi-state parallelism
Replace `run_states_parallel.py` with Rayon parallel iterator across states. Cap at `min(workers, 12)`.

#### 3d. Output format parity
Rust CLI writes:
- `states/{state}/data/district_summary.csv`
- `states/{state}/data/final_assignments.pkl` (via `pickle` crate or Python shim)
- `states/{state}/data/vra_analysis.json`

**New test required** (write before marking Phase 3 complete):  
`tests/integration/test_cli_parity.py` — run `redist run --state VT ...` as a subprocess, check that `final_assignments.pkl` and `vra_analysis.json` exist, and that population balance passes. Do **not** reuse `TestVRACodePathIntegrity` (which invokes the Python script). New test must invoke the `redist` binary directly.

**Exit criterion**: `pytest tests/integration/test_cli_parity.py` passes for VT (1 district, trivial) and AL (7 districts, VRA target).

---

### Phase 4 — Analysis and Dashboard (2 weeks)
**Goal**: `redist analyze` and `redist dashboard` work from the binary.

#### 4a. Compactness metrics
Port Polsby-Popper and Reock to `redist-analysis`. **Critical requirement**: district polygons must be projected to an equal-area CRS (EPSG:5070 Albers Equal Area for CONUS, EPSG:3338 for Alaska, EPSG:6364 for Hawaii) before computing area/perimeter. The `geo` crate computes planar area from coordinate values — WGS84 degrees give area in square degrees, not square meters. Polsby-Popper is meaningless without projected coordinates.

Implementation: accept projected WKB as input to the compactness function. The caller (CLI or Python) projects before calling.

**New test required**:  
`tests/unit/test_rust_compactness.py` — compute Polsby-Popper for a known projected polygon (e.g., unit square in meters) in both Python and Rust; assert values match within 1e-6.

#### 4b. VRA analysis (atomic write)
Port `vra_utils.py:analyze_mm_districts()` to Rust in `redist-analysis`. The `redist-cli` binary writes both `final_assignments` and `vra_analysis.json` inside a single function:

```rust
// redist-cli: write both outputs atomically
fn write_state_outputs(state_dir: &Path, partition: &Partition, vra: &VraAnalysis) -> Result<()> {
    let tmp_assignments = state_dir.join("data/.final_assignments.tmp.pkl");
    let tmp_vra = state_dir.join("data/.vra_analysis.tmp.json");
    write_assignments(&tmp_assignments, partition)?;
    write_vra_analysis(&tmp_vra, vra)?;
    // Both writes succeeded; now rename atomically
    fs::rename(&tmp_assignments, state_dir.join("data/final_assignments.pkl"))?;
    fs::rename(&tmp_vra, state_dir.join("data/vra_analysis.json"))?;
    Ok(())
}
```

This eliminates the `vra_mode` premature-clear class of bugs: analysis and partition are computed together and written atomically. Partial-write state (one file but not the other) is detected on restart by the presence of `.tmp.*` files.

#### 4c. Static dashboard
Port `scripts/web/deploy_docs.py` to `redist-web`.

**Exit criterion**: `redist analyze --state AL --version V4` produces `vra_analysis.json` with `mm_count==2`. `redist dashboard --version V4 --year 2020` matches current `docs/dashboard_vra.html` visually.

---

### Phase 5 — Native METIS (stretch goal, 4+ weeks)
**Goal**: Replace `gpmetis` subprocess with linked C library (via `bindgen`) or Rust partitioning kernel.

**Decision gate**: profile subprocess overhead after Phase 4. If subprocess is <10% of total runtime, skip this phase entirely — the subprocess is architecturally acceptable long-term.

**Windows note**: linking METIS as a C library via `bindgen` requires MSVC or MinGW-w64. Development on Windows 11 works for Phases 1-4 (pure Rust + PyO3 with no C linking); Phase 5 requires Linux/WSL or a MinGW build environment.

**Performance impact**: This is where the 55 min → 10 min full-run target becomes achievable. Phases 1-4 realistically achieve ~20-25 min (adjacency savings ~15 min + increased parallelism). Native METIS eliminates per-state subprocess overhead and enables tighter bisection loop integration.

---

## CI Integration

```yaml
# Add to .github/workflows/tests.yml

# Existing 'test' job: add env var so Python tests pass without Rust compiled
test:
  env:
    REDIST_NO_RUST: "1"
  # ... rest of existing job unchanged

# New job
rust-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: sudo apt-get install -y metis
    - run: cargo test --workspace
    - run: pip install maturin && maturin develop
    - run: pytest tests/unit/test_rust_graph.py    # Phase 0
    # Add more tests as phases complete:
    # - run: pytest tests/integration/test_adjacency_parity.py   # Phase 2
    # - run: pytest tests/integration/test_cli_parity.py         # Phase 3
    # - run: pytest tests/unit/test_rust_compactness.py          # Phase 4
```

---

## Migration Invariants (BOUNDARY role)

1. **Population balance**: Every **final** partition (leaf nodes only, not intermediate bisections) asserts ±0.5% before output. Enforced in `redist-core::Partition::assert_balanced()`. Intermediate bisection nodes use their own `ufactor` tolerance.

2. **VRA analysis atomicity**: `final_assignments` and `vra_analysis.json` are written via rename from temp files in `redist-cli::write_state_outputs()`. Partial-write states are detectable and restartable. This is only enforceable from the CLI binary — not through PyO3 (Python controls I/O when calling via bindings).

3. **Formula ground truth**: After Phase 1a, the adaptive boost formula exists only in `redist-core/src/vra.rs`. `run_state_redistricting.py:226-260` is deleted. Both Python (via `redist_py.build_vra_edge_weights`) and the CLI use the same implementation. Any future change to the formula requires one edit in one place.

4. **No silent fallback**: When `REDIST_NO_RUST` is unset and `redist_py` is not compiled, the Python pipeline raises `ImportError` immediately. Set `REDIST_NO_RUST=1` to use pure-Python path (slower, same results).

5. **Output format parity**: Python and Rust pipelines produce byte-compatible CSVs (±1e-6 float tolerance). Verified by `test_adjacency_parity.py` (Phase 2) and `test_cli_parity.py` (Phase 3) before each Python code path is deprecated.

---

## Per-Phase Test Requirements

Every phase **must** have a new test written before the phase is marked complete. Existing tests do not count.

| Phase | Required new test | What it proves |
|---|---|---|
| 0 | `tests/unit/test_rust_graph.py` | `Graph.from_csr()` round-trips correctly |
| 1 | extend `test_vra_edge_weighting.py` to call `redist_py.build_vra_edge_weights` | Rust formula matches Python formula output |
| 2 | `tests/integration/test_adjacency_parity.py` | Rust adjacency byte-matches Python for VT + DE |
| 3 | `tests/integration/test_cli_parity.py` | `redist` binary produces valid VRA outputs |
| 4 | `tests/unit/test_rust_compactness.py` | PP values match Python within 1e-6 (projected input) |

---

## Dependencies

| Crate | Purpose | Notes |
|---|---|---|
| `clap` (derive) | CLI argument parsing | |
| `rayon` | Data-parallel processing | CPU-bound; preferred over tokio |
| `geo` | Geometry types, area/perimeter | Requires projected coords for compactness |
| `geo-types` | Shared geometry primitives | |
| `rstar` | R-tree for candidate neighbor pairs | |
| `shapefile` | Pure-Rust ESRI shapefile reader | Phase 2a; avoids GDAL dependency |
| `serde` + `serde_json` | JSON serialization | |
| `csv` | CSV read/write | |
| `pyo3` | Python bindings | Pin to stable ABI (`abi3`) |
| `maturin` | Build PyO3 wheel | |
| `numpy` (pyo3-numpy) | numpy array interop | Pin version; test with Python 3.13 |

Removed from original plan: `geos` (C bindings, portability issues), `kdtree` (no bounding-box queries).

---

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| METIS C linking fails on Windows (Phase 5) | Medium | Phase 5 is optional; Linux/WSL for Phase 5 dev |
| `geo` crate area wrong due to WGS84 input | **Eliminated** | Plan now requires projected input |
| PyO3 ABI breaks on Python 3.13 | Low | Pin `abi3`; test in CI |
| Rust adjacency diverges on edge cases (islands, water) | Medium | `test_adjacency_parity.py` covers all 50 states |
| Adaptive boost formula drift during migration | **Eliminated** | Python copy deleted at end of Phase 1a |
| `REDIST_NO_RUST` breaks existing CI | **Eliminated** | Existing CI job sets it explicitly |
| 55→10 min target not achievable without Phase 5 | Known | Realistic Phases 1-4 target: ~20-25 min |
| METIS version unpinned (apt-get) | Low | Pin version in CI after Phase 5 decision |
