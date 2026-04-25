# Data Format Invariants (DF-01..DF-04)

Properties of census data formats that must be maintained. Violating these produces silent data errors — the pipeline runs but produces wrong outputs.

---

## DF-01: GEOID is exactly 11 characters for census tracts

**Invariant:** Every census tract GEOID is a string of exactly 11 decimal digits: SS (state FIPS) + CCC (county FIPS) + TTTTTT (tract code). Short GEOIDs cause adjacency join failures; long GEOIDs match nothing.

**Why it must hold:** GEOIDs are the join key between TIGER geometry, demographics, and adjacency graphs. A length mismatch causes silent inner-join failures — tracts with mismatched GEOIDs get zero population or are omitted entirely.

**When it can be violated:** Reading GEOID as int then converting back to string loses leading zeros. Reading from CSV without `str.zfill(11)`.

**Enforcement:** TIGER reader validates `geoid.len() == 11` and returns TigerError::InvalidGeoidLength if not.

**Test:** `tests/unit/test_rust_tiger.py::TestTigerReaderVermont::test_geoid_length`
`tests/unit/test_rust_tiger.py::TestTigerReaderVermont::test_geoid_state_prefix`

**Status:** ENFORCED

---

## DF-02: Edge weight keys are canonical (u < v)

**Invariant:** All edge weight dictionaries (adjacency.edge_weights, VRA edge weights) use canonical key order where the smaller index is always first: key = (min(u,v), max(u,v)).

**Why it must hold:** Non-canonical keys cause lookup failures. If the edge (3,1) is stored but the lookup uses (1,3), the weight is treated as 1.0 (the default) instead of the actual boundary length. The METIS file then has wrong edge weights, producing wrong district shapes.

**When it can be violated:** Building edge weights from adjacency without canonicalizing: `(i, j)` where j could be < i. serialize_adjacency now validates canonical order before writing.

**Enforcement:** serialize_adjacency asserts u < v for all keys. metis_format.rs validates non-canonical keys return error. TIGER reader canonicalizes on load.

**Test:** `tests/unit/test_rust_graph.py::test_edge_weights_canonical_order`
`tests/unit/test_rust_serialize.py::test_edge_weights_canonical_order_preserved`
`redist/crates/redist-core/src/metis_format.rs::test_non_canonical_edge_key_returns_error`

**Status:** ENFORCED

---

## DF-03: Adjacency is symmetric

**Invariant:** If vertex j is in the neighbor list of vertex i, then i must be in the neighbor list of j. Non-symmetric adjacency produces disconnected subgraphs or asymmetric edge weights.

**Why it must hold:** METIS requires the adjacency graph to be undirected (symmetric). A directed edge creates a graph where METIS can cut a one-way edge and get different results depending on traversal direction.

**When it can be violated:** Build adjacency from only one direction of each edge pair (filtering `j > i` at construction but failing to add both directions).

**Enforcement:** `build_adjacency_graph` adds both (i,j) and (j,i) directions when building from candidate pairs. Test verifies symmetry.

**Test:** `tests/unit/test_rust_adjacency.py::test_adjacency_is_symmetric`

**Status:** ENFORCED

---

## DF-04: Vertex weights are positive integers (population ≥ 1)

**Invariant:** Every tract's vertex weight (population) is a positive integer ≥ 1. Zero or negative weights cause METIS to produce degenerate partitions (empty districts).

**Why it must hold:** METIS uses vertex weights to balance partitions. A tract with weight 0 can be assigned to any district without cost, and a tract with weight -1 makes districts look more balanced than they are.

**Enforcement:** vertex_weights clamped to minimum 1 in split_subgraph (`vertex_weights[g].max(1)`) before writing METIS format. This is defensive — PL 94-171 should never produce 0-population tracts.

**Test:** PARTIAL — no explicit test verifies all weights ≥ 1. The max(1) clamp prevents METIS failures but could hide data quality issues.

**Status:** PARTIAL — add explicit test: `assert!(graph.vertex_weights.iter().all(|&v| v >= 1))`
