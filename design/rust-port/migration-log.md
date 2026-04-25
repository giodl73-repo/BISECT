# Migration Log

## Phase 0 — Complete (2026-04-24)
Cargo workspace scaffold. PyO3 bindings compile. `Graph.from_csr()`, `Partition`,
`build_vra_edge_weights()` all exposed. 7/7 Python tests pass.

## Phase 1 — Complete (2026-04-24)
VRA formula moved to Rust (single source). Bisection tree schedule. METIS format
writer/parser. Balance checker wired into Python pipeline.
- `vra.rs`: adaptive boost formula — Python copy deleted from run_state_redistricting.py
- `bisection.rs`: BisectionTree split schedule, ufactor_for_depth
- `metis_format.rs`: write_metis_graph, parse_metis_partition (native Rust)
- `partition.rs`: assert_balanced wired after every final partition

## Phase 2 — Complete (2026-04-24)
TIGER shapefile reader, parallel adjacency builder, island bridging, .adj.bin
serialization. All data modules in Rust, exposed via PyO3.
- `tiger.rs`: pure Rust ESRI .shp reader (no GDAL), WKB output
- `adjacency.rs`: R-tree candidate detection + Rayon parallel intersection
- `bridge.rs`: Union-Find connected components + nearest-county bridging
- `serialize.rs`: RADJ binary format with magic header and validation

## Phase 3 — Complete (2026-04-25)
Full CLI binary. `redist state VT` and `redist state AL --partition-mode metis-vra`
both pass end-to-end acceptance tests. Python subprocess calls eliminated.

### Phase 3 key implementation fixes (bugs found, not in plan):
- `REDIST_PYTHON` env var: Rust binary uses caller's Python (PP-04 pitfall)
- Adjacency path separate from output_dir (PP-05 pitfall)
- State directory uses `state_name` not `state_code.lower()` (DP-03 pitfall)
- Error propagation from par_iter: `filter_map(.ok()?)` → `collect::<Result>()` (PP-06 pitfall)
- ufactor_for_depth() returns decimal (1.001..1.005), not integer (PP-07 pitfall)
- Target partition weights (tpwgts): k_left/k + k_right/k for METIS (AP-06 pitfall)

### Phase 3d benchmarks (2026-04-25, Windows 11, 12-core):

| State | Mode | Python (s) | Rust CLI (s) | Speedup |
|-------|------|------------|--------------|---------|
| VT    | edge-weighted (1 district) | 4.48 | 0.52 | **8.6×** |
| AL    | metis-vra (7 districts)    | 1.63 | 0.65 | **2.5×** |

Notes:
- Python VT includes adjacency loading from pkl, METIS via pymetis, writing pkl
- Rust VT includes adjacency loading via Python shim (pkl→JSON), METIS via subprocess, writing JSON
- AL VRA uses n-way partition (single gpmetis call) instead of 7 recursive 2-way calls
- Main remaining overhead: adjacency pkl loading via Python shim (Phase 2 target: native .adj.bin)

### `redist fetch` (Phase 3d bonus):
- `manifest.json` embedded in binary: 50 states, TIGER + PL 94-171 URLs
- `reqwest::blocking` for HTTP downloads (no Python subprocess)
- `zip` crate for extraction
- `REDIST_MANIFEST` env var for local manifest override
- 22 integration tests with stdlib HTTP fixture server (no external deps)
- TIGER fixture files committed: VT (1 district, 193 tracts) + RI (2 districts, 250 tracts)

## Phase 4 — Complete (2026-04-25)
Compactness metrics (PP, Reock, CHR) and VRA analysis in `redist-analysis` crate.
Exposed via PyO3. Role review caught: MM threshold `>` not `>=` (VA-01 invariant),
perimeter excludes holes, deterministic FP aggregation, array length validation.

## Phase 5 — Not started (optional)
Decision gate: profile subprocess overhead after 50-state run. If >10% of
wall time, link METIS as C library via bindgen. Otherwise skip.
Current assessment: METIS subprocess is fast (ms-scale per call); the main
bottleneck is adjacency pkl loading via Python shim. Resolve Phase 2 (.adj.bin)
before evaluating Phase 5.
