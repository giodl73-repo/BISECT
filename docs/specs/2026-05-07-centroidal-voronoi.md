# Spec: Centroidal Voronoi Districts — Geometric District Construction via Iterative Seed Placement

**Status**: Proposed  
**Date**: 2026-05-07  
**Layer**: Structure (SplitStrategy) — replaces METIS at each bisection node  
**Related paper**: B.22  
**New crate**: Not needed — pure geometry, implemented in `redist-cli/src/bisection_runner.rs`

---

## Overview

All existing structure-layer algorithms (METIS, SA, GeoSection, AreaSection) operate on graph topology — adjacency, edge cuts. Centroidal Voronoi Districts (CVD) operates on **geography**: it places k district seeds at population-weighted centroid locations and assigns each tract to its nearest seed by geographic distance. This produces geometrically compact, roughly circular districts — a fundamentally different compactness profile from edge-cut minimization.

CVD is the "packer" algorithm: it fills geographic space greedily by proximity, independent of political boundaries or demographic composition. It provides a natural baseline: "what does the most geometrically natural partition look like?"

CVD is fast: O(n × k × n_iter) where n_iter ≈ 20 typically suffices for convergence. No spanning trees, no MCMC, no randomness (deterministic given seed initialization). It parallelises perfectly across bisection nodes.

**Relationship to existing structure methods**:

| Method | Optimizes | Randomness | Data needed | Phase |
|---|---|---|---|---|
| METIS | Edge cut (graph) | Seeded | None | Done |
| SA | Edge cut (graph) | Seeded | None | Done |
| GeoSection | Direction + EC | Seeded | None | Done |
| **CVD** | **Geographic distance** | Seeded | Centroids (Geo) | **This spec** |
| BFS Growth | Population balance | Seeded | None | B.23 |

---

## 1. Voronoi metric

CVD supports two metrics, implemented as a Rust enum. The metric determines how "nearest seed" is computed at each iteration.

```rust
pub enum VoronoiMetric {
    /// Graph-distance (BFS hop count) — no new data needed. Phase 1.
    GraphDistance,
    /// Geographic Euclidean distance — requires tract centroids. Phase 2.
    Geographic { projection: ProjectionType },
}

pub enum ProjectionType {
    /// Euclidean on raw (lon, lat) — adequate for continental US states.
    Raw,
    /// EPSG:5070 Albers Equal Area Conic — required for Alaska and Hawaii.
    AlbersEqualAreaConic,
}
```

**Phase 1 (GraphDistance)**: Distance between tract i and seed j is the BFS hop count from i to j in the adjacency graph. No geographic data beyond the adjacency list is required. Fully implementable with existing pipeline data.

**Phase 2 (Geographic)**: Distance is Euclidean on projected coordinates. For continental US states, raw (lon, lat) Euclidean distance is adequate. For states spanning large latitudinal ranges — Alaska, Hawaii — use EPSG:5070 Albers Equal Area Conic projection before computing distances to avoid meridian-convergence distortion.

**New field needed in `LoadedGraph`**: `pub tract_centroids: Vec<(f64, f64)>` (longitude, latitude). Currently absent; adding this to the adjacency loader is a Phase 2 task. Centroids are available from TIGER/Line shapefiles as the centroid of each tract polygon, stored in the adjacency `.pkl` file metadata or derivable from the `_geoids.json` file.

---

## 2. Seeding

Three initialization strategies:

```rust
pub enum SeedInit {
    /// Evenly spaced seeds by BFS distance from graph center. Deterministic, no randomness.
    Regular,
    /// Distance-weighted sampling: first seed random, subsequent seeds
    /// weighted by squared distance from existing seeds (k-means++ criterion).
    KmeansPlusPlus,
    /// Uniform random placement. Reproducible given cvd_seed.
    Random,
}
```

All random seeding is derived from `cvd_seed`:

```
cvd_seed(base_seed) = SHA-256("CVD_INIT_" || base_seed:u64le) → least-significant 64 bits
```

`SeedInit::Regular` is fully deterministic (no randomness). `SeedInit::KmeansPlusPlus` and `SeedInit::Random` consume `cvd_seed` via `SmallRng::seed_from_u64(cvd_seed)`. The prefix `"CVD_INIT_"` is distinct from all other algorithm prefixes; a test asserts the prefix constant equals `"CVD_INIT_"` exactly and that `cvd_seed(0)` equals a hard-coded expected value.

---

## 3. Algorithm

```
CentroidalVoronoi(tract_coords, tract_pop, tract_adj, k, n_iter, metric, seed_init, balance_tolerance, base_seed):
  // tract_coords[i] = (lon, lat) of tract centroid [Phase 2 only]
  // tract_pop[i]    = population of tract i
  // tract_adj       = adjacency list (always available)

  // Step 1: Initialise k seeds
  cvd_seed = SHA-256("CVD_INIT_" || base_seed:u64le) → u64le
  seeds = initialize_seeds(tract_coords OR tract_adj, tract_pop, k, seed_init, cvd_seed)

  for iter in 0..n_iter:
    // Step 2: Assign each tract to nearest seed (Voronoi assignment)
    // Phase 1 (GraphDistance): BFS hop count from each seed; multi-source BFS is O(n)
    // Phase 2 (Geographic):    Euclidean distance on projected coordinates; O(n × k)
    assignment = argmin_j(distance(tract[i], seed[j])) for each i

    // Step 3: Rebalance to enforce population balance
    // Boundary tracts move from overweight to underweight districts
    // Uses same BFS-boundary rebalance as run_multiscale
    assignment = rebalance(assignment, tract_adj, tract_pop, balance_tolerance)

    // Step 4: Update seeds to population-weighted centroid of each district
    for j in 0..k:
      tracts_j = [i : assignment[i] == j]
      total_pop = sum(tract_pop[t] for t in tracts_j)
      // Phase 1: seed is tract in tracts_j minimizing mean BFS distance to all others in tracts_j
      // Phase 2: seed = (sum(lon[t] * pop[t]) / total_pop, sum(lat[t] * pop[t]) / total_pop)
      seeds[j] = centroid_of(tracts_j, metric)

    // Step 5: Check convergence (seeds unchanged from previous iteration)
    if seeds == previous_seeds: break

  return (assignment, iter)  // assignment = k-district plan; iter = iterations_to_convergence
```

**Graph-distance centroid (Phase 1)**: The "centroid" of a district in graph space is the tract minimizing the sum of BFS hop counts to all other tracts in the district — the graph-distance medoid. Computing the exact medoid is O(n^2) per district per iteration; in practice, approximate via the tract minimizing mean distance to a random sample of 50 tracts in the district (O(50n) per iteration, adequate for convergence).

**Contiguity note**: Voronoi assignment does not guarantee contiguous districts for non-convex geographies. The rebalance step (Step 3) uses BFS to detect and repair disconnected components, consistent with existing `run_multiscale` behavior.

---

## 4. Rust struct and API

**Location**: `redist-cli/src/bisection_runner.rs` (no new crate needed)

```rust
pub struct CentroidalVoronoiConfig {
    pub n_iter: usize,            // max CVD iterations (default: 20)
    pub metric: VoronoiMetric,    // default: GraphDistance
    pub seed_init: SeedInit,      // default: KmeansPlusPlus
    pub balance_tolerance: f64,   // passed from SplitStrategy context
}

/// Run CVD on a subgraph (one bisection node).
/// Returns (assignment, iterations_to_convergence).
pub fn run_centroidal_voronoi(
    adj: &[Vec<u32>],
    pop: &[i64],
    tract_centroids: Option<&[(f64, f64)]>,  // None for Phase 1 / GraphDistance
    k: u32,
    config: &CentroidalVoronoiConfig,
    base_seed: u64,
) -> (Vec<u32>, usize)
```

---

## 5. Compositor integration

```rust
SplitStrategy::CentroidalVoronoi {
    n_iter: usize,          // max CVD iterations (default: 20)
    metric: VoronoiMetric,  // default: GraphDistance
    seed_init: SeedInit,    // default: KmeansPlusPlus
}
```

The compositor passes `balance_tolerance` from the run configuration to `run_centroidal_voronoi`. Parallelism across bisection nodes is already provided by the existing `rayon`-based bisection runner; no additional parallelism machinery is needed.

---

## 6. CLI

```bash
--structure centroidal-voronoi --cvd-iters 20
```

With explicit metric and seeding:

```bash
--structure centroidal-voronoi --cvd-iters 20 --cvd-metric graph-distance --cvd-seed-init kmeans++
```

**YAML**:

```yaml
algorithm:
  structure: centroidal-voronoi
  weights: geographic
  search: convergence
  cvd_iters: 20
  cvd_metric: graph-distance        # or: geographic
  cvd_seed_init: kmeans++           # or: regular, random
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 8
```

---

## 7. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"structure": "centroidal-voronoi",
"n_iter": 20,
"metric": "graph-distance",
"seed_init": "kmeans++",
"base_seed": 12345678,
"cvd_seed": 987654321,
"iterations_to_convergence": 14,
"final_ec": 2847,
"final_pp_mean": 0.241
```

`iterations_to_convergence` records how many iterations were needed before seeds stabilized. If `iterations_to_convergence == n_iter`, the algorithm did not converge within the budget; this is a warning condition (not an error) and should be flagged in the report.

`cvd_seed` is the derived seed used for all random initialization; an auditor who knows `base_seed` can recompute `cvd_seed = SHA-256("CVD_INIT_" || base_seed:u64le)` and verify determinism. `final_ec` and `final_pp_mean` (mean Polsby-Popper across k districts) characterize the output compactness.

---

## 8. Test invariants

### L0 (inline unit tests)

- Graph-distance Voronoi: every tract assigned to a district in 0..k-1; no district empty
- Same `base_seed` -> identical assignment on two runs (determinism)
- `iterations_to_convergence <= n_iter` (no infinite loop)
- Population balance: all districts within `balance_tolerance` of ideal population after rebalance
- With `SeedInit::Regular`, k seeds are evenly spaced across the graph by BFS distance from graph center
- `cvd_seed` prefix constant in source equals `"CVD_INIT_"` exactly; `cvd_seed(0)` equals hard-coded expected value
- Convergence stability: after convergence, no tract would reduce its district's population imbalance by moving to an adjacent district's seed (local stability under rebalance)
- `n_iter=0`: returns the initial Voronoi assignment with 0 iterations (no crash, valid assignment)
- k=1: degenerate case — all tracts assigned to district 0, 1 iteration, no panic

### L1 (integration, synthetic data)

- 4x4 grid k=4, GraphDistance: produces a valid 4-partition in ≤ 20 iterations
- Convergence verification: running for 100 iterations gives the same result as 20 (confirmed stable)
- Determinism: two runs with the same `base_seed` give identical assignments and `iterations_to_convergence`
- Contiguity: all k districts are contiguous in the 4x4 grid (rebalance step enforces connectivity)
- `SeedInit::Regular` produces the same assignment regardless of `base_seed` (no randomness consumed)

### L2 (`#[ignore]`, real data)

- NC 2020 k=14, GraphDistance: mean Polsby-Popper compactness compared to a single METIS call
- Expected: CVD produces higher PP than standard METIS (geographic proximity maximizes geometric compactness) but lower than `BisectionEnsemble(p=0.0)` (which optimizes edge cuts directly)
- `iterations_to_convergence < n_iter` for NC 2020 (algorithm converges within budget under default settings)

---

## 9. Open questions (deferred)

1. **Metric choice**: Graph-distance Voronoi is fully implementable with existing data (Phase 1). Geographic Euclidean Voronoi requires tract centroids from TIGER/Line (Phase 2). Both are specced; implement graph-distance first, add geographic as a Phase 2 option once `tract_centroids` field is added to `LoadedGraph`.

2. **Contiguity guarantee**: Voronoi regions can be disconnected for non-convex state geometries. Contiguity must be enforced as a post-processing step within the rebalance routine via BFS component detection. The current spec delegates this to the existing `run_multiscale` rebalance; verify this is sufficient for all 50 states.

3. **Alaska and Hawaii projection**: Phase 2 geographic CVD should use EPSG:5070 Albers Equal Area Conic for Alaska and Hawaii to avoid distortion from raw (lon, lat) Euclidean distance. Confirm projection constants match those used in `redist-map` for consistency.

4. **Graph-distance medoid approximation**: The Phase 1 centroid update uses a sampled medoid (50 random tracts per district). Evaluate whether this approximation is sufficient for convergence or whether exact medoid (Floyd-Warshall per subgraph) is needed for small subgraphs (k=2, n<50).

5. **Interaction with search layer**: CVD is a structure-layer algorithm; search-layer strategies (convergence, percentile, bisection-ensemble) operate on the output of the structure layer. No changes to the search layer are anticipated, but verify that `BisectionEnsemble` over CVD subproblems behaves correctly when `iterations_to_convergence` varies across ensemble members.
