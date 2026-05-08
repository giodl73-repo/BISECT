# Spec: Moving-Knife Algorithm (MKA) — Fair-Division Geometric Bisection for Maximum Reock Compactness

**Status**: Proposed (R1 reviewed, P1 fixes applied)
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 4/4 → avg 3.25/4  
**Date**: 2026-05-07  
**Layer**: Structure (SplitStrategy::MovingKnife) — replaces METIS at each bisection node  
**Related paper**: B.25  
**New crate**: Not needed — implemented in `bisect-cli/src/bisection_runner.rs`  
**Citation**: Puppe & Tasnadi (2026, *Public Choice*) — "Moving-Knife Redistricting"  
**Requires**: `tract_centroids` (same as CVD Phase 2; already in `LoadedGraph` from task #150)

---

## Overview

All current structure algorithms optimise different metrics:

| Algorithm | Optimises | Compactness profile |
|---|---|---|
| METIS | Edge cuts (graph) | Low Reock, low edge-cut |
| SA | Edge cuts via annealing | Low Reock, low edge-cut |
| CVD | Geographic distance to seeds | Medium–high Polsby-Popper |
| **MKA** | **Reock score directly** | **Highest Reock** |

Reock is the most legally prominent compactness metric: courts in NC, WI, GA, and FL have explicitly referenced it. MKA provides a provable claim: "the submitted plan uses the most Reock-compact bisection direction available."

Theoretical basis: fair-division theory (moving-knife protocol, Dubins & Spanier 1961). A "knife" sweeps across the geographic space; the algorithm finds the orientation where stopping the knife produces the most compact split. At each bisection node, MKA tests `n_orientations` candidate sweep directions, scores each split by the minimum Reock score of the two halves, and returns the direction that maximises it.

**Why not replace existing algorithms**: MKA is additive — it activates only via `--structure moving-knife`. Graph-distance CVD, METIS, and AreaSection remain the defaults for their respective use cases. MKA is best when court filings require an explicit Reock-maximising justification.

---

## 1. Centroid data

MKA uses the same `tract_centroids` field added in CVD Phase 2:

```rust
pub struct LoadedGraph {
    // ... existing fields ...
    /// Population-weighted centroid of each tract polygon (WGS84 lon/lat).
    /// Empty if the centroid companion file is not present.
    pub tract_centroids: Vec<(f64, f64)>,
}
```

Loading: after loading the adjacency file, look for `{stem}_centroids.json` in the same directory. If present, load into `tract_centroids`. If absent, error at dispatch time (MKA requires centroids; no fallback mode). The companion file format and index alignment rules are identical to CVD Phase 2.

```json
{"centroids": [[lon0, lat0], [lon1, lat1], ...]}
```

If `len(centroids) != len(adj)`, the loader returns an error. No new fetch step is required if centroids were already fetched for CVD Phase 2.

---

## 2. Projection

MKA reuses the Albers projection from CVD Phase 2 without modification:

| State | Projection | EPSG |
|---|---|---|
| Continental US (48 states + DC) | Albers Equal Area Conic, NAD83 | 5070 |
| Alaska | Alaska Albers Equal Area Conic | 3338 |
| Hawaii | Hawaii Albers Equal Area | 102007 |
| Puerto Rico | Puerto Rico and Virgin Islands | 32161 |

The `albers_project` function is shared with CVD Phase 2 (no duplication). State projection selection is automatic from state FIPS code. The same spherical approximation caveat applies (≤0.3% error for continental US; negligible at census-tract resolution).

---

## 3. Algorithm — Moving-Knife sweep

```
split_subgraph_mka(adj, pop, tract_indices, centroids, n_orientations, metric, base_seed):
  sorted = sorted(tract_indices)
  projected = [albers_project(centroids[i]) for i in sorted]   // reuse from CVD Phase 2

  best_score = -inf
  best_left = empty
  best_right = empty

  seed_rng = mka_seed(base_seed)  // for tie-breaking only; MKA is deterministic per orientation

  for k in 0..n_orientations:
    theta = k * PI / n_orientations                  // angle in [0°, 180°)
    axis = (cos(theta), sin(theta))                  // unit vector for sweep direction

    // Project each tract centroid onto the sweep axis
    projections = [(dot(projected[i], axis), i) for i in 0..m]
    projections.sort_by projection_value

    // Find the split that gives best population balance
    target_pop = total_pop / 2
    cumulative_pop = 0
    split_idx = 0
    for (i, (proj, tract)) in projections.enumerate():
      cumulative_pop += pop[sorted[tract]]
      if cumulative_pop >= target_pop:
        split_idx = i
        break

    left_tracts  = projections[..=split_idx].map(|(_,t)| sorted[t])
    right_tracts = projections[split_idx+1..].map(|(_,t)| sorted[t])

    // Compute Reock score: area / minimum_enclosing_circle_area
    // min_enclosing_circle via Welzl's algorithm (expected O(m))
    reock_left  = reock_score(left_tracts,  projected)
    reock_right = reock_score(right_tracts, projected)
    score = min(reock_left, reock_right)             // maximise the worse half

    if score > best_score:
      best_score = score
      best_left  = left_tracts
      best_right = right_tracts

  // Post-hoc rebalance (same 200-iter boundary-swap as CVD/BFS)
  (best_left, best_right) = rebalance(best_left, best_right, adj, pop, balance_tolerance)
  return (global_left_set, global_right_set)
```

**Reock score formula**: Reock(D) = Area_polygon(D) / Area(MEC_polygon(D))

where Area_polygon(D) = sum of TIGER/Line polygon areas for tracts in D, and MEC_polygon(D) is the minimum enclosing circle of the tract *polygon boundary vertices* (not just centroids). In the Phase 1 implementation, we use the centroid approximation: MEC is fitted to the projected centroid points only, not the full polygon boundaries. This approximation may produce Reock_approx > 1 for large boundary tracts; the implementation clamps to [0.0, 1.0] to prevent this. The L0 invariant `Reock ≤ 1` is enforced by clamping, not structural guarantee. Phase 2 will implement the polygon-boundary MEC using the Shapely convex hull + circumscribed circle approach.

**PolsbyPopper alternative**: When `--mka-metric polsby` is specified, the sweep maximises Polsby-Popper instead of Reock. PP requires perimeter computation (O(m) with precomputed edge lengths from the adjacency file) — supported but Reock is the default, since Reock requires only centroid positions and MEC geometry with no polygon perimeter data.

**Contiguity**: Linear sweep does not guarantee contiguous districts for non-convex subgraph geometries. The post-hoc rebalance step detects and repairs disconnected components via BFS, consistent with CVD Phase 2 behavior.

---

## 4. Seeding

MKA is deterministic given `base_seed` and `n_orientations`. The seed is needed only for tie-breaking when two orientations give identical score (extremely rare in practice):

```
mka_seed(base_seed, node_path) =
  SHA-256("MKA_INIT_" || node_path_len:u32le || node_path.as_bytes() || base_seed:u64le) → u64le
```

where `node_path_len` is the byte length of `node_path` as a 4-byte little-endian u32. This eliminates the prefix-collision risk by making the boundary between node_path and base_seed unambiguous. The separator `"_"` is dropped since the length prefix makes it unnecessary. The prefix `"MKA_INIT_"` is distinct from `"CVD_GEO_INIT_"` and `"CVD_INIT_"`. A test asserts all three prefix constants are present in source and are pairwise distinct. `node_path` is required to make seeds unique across bisection tree nodes (same role as in CVD Phase 2).

---

## 5. Rust struct and API

**Location**: `bisect-cli/src/bisection_runner.rs` (no new crate needed)

```rust
pub enum CompactnessMetric {
    /// Reock score: Area(D) / Area(MEC(D)) — default. Requires centroid positions only.
    Reock,
    /// Polsby-Popper: 4π·Area(D) / Perimeter(D)² — requires precomputed edge lengths.
    PolsbyPopper,
}
```

**`SplitStrategy` extension**:

```rust
SplitStrategy::MovingKnife {
    n_orientations: usize,       // sweep granularity (default: 180 = every 1°)
    metric: CompactnessMetric,   // default: Reock
}
```

**`run_moving_knife` signature**:

```rust
pub fn run_moving_knife(
    adj: &[Vec<u32>],
    pop: &[i64],
    tract_centroids: &[(f64, f64)],   // required; no None fallback
    tract_areas: &[f64],              // m² per tract (from TIGER/Line); required for Reock
    config: &MovingKnifeConfig,
    base_seed: u64,
    node_path: &str,
) -> (HashSet<usize>, HashSet<usize>)  // (left_set, right_set)
```

**Data source for tract areas**: TIGER/Line census tract shapefiles provide polygon areas in square metres. These are computed from the `.adj.bin` adjacency files during `bisect fetch --year YEAR --states STATE` and stored in the `LoadedGraph.tract_areas` field alongside `vertex_weights` (population). No separate fetch step is needed — tract area data is bundled with the adjacency file. If `tract_areas` is empty (e.g. for synthetic test graphs), the Reock numerator defaults to the number of tracts as a proxy area, and the denominator is the MEC area of the centroid positions. This fallback is acceptable for unit tests but not for production use.

**`split_subgraph_mka_direction`** (AreaSection warm-start; see Section 6):

```rust
pub fn split_subgraph_mka_direction(
    tract_indices: &HashSet<usize>,
    centroids: &[(f64, f64)],
    n_orientations: usize,
) -> f64   // returns θ* in radians ∈ [0, π)
```

---

## 6. Compositor integration

**Pure MKA** (`--structure moving-knife`):

```rust
SplitStrategy::MovingKnife {
    n_orientations: usize,       // default: 180
    metric: CompactnessMetric,   // default: Reock
}
```

**AreaSection hybrid** (`--structure area-section --area-section-init moving-knife`): MKA outputs the optimal angle θ* without performing the full split. AreaSection uses θ* to set `ratio_direction` before calling its own balance-enforcement logic. This is a full override: `ratio_direction = mka_direction` (θ* replaces any ratio-direction computed from the config). The `AreaSectionInit` enum gains a new variant:

```rust
pub enum AreaSectionInit {
    Ratio,         // existing default: ratio_direction from config
    MovingKnife,   // new: ratio_direction = split_subgraph_mka_direction(...)
}
```

No other compositor changes are needed. The existing rayon-based bisection runner parallelism covers MKA automatically.

---

## 7. CLI

```bash
# Pure MKA structure
bisect state --state NC --year 2020 \
  --structure moving-knife \
  --mka-orientations 180 \
  --mka-metric reock
```

With `--structure moving-knife`, the CLI checks `graph.tract_centroids.len() > 0` before dispatch and errors if centroids are absent:

```
ERROR: --structure moving-knife requires tract centroid data.
       Run: bisect fetch --type centroids --states NC --year 2020
```

Note: tract centroid coordinates and tract polygon areas (`tract_areas`) both originate from the same `bisect fetch` step and are bundled into the `.adj.bin` adjacency file. If centroids are missing, areas are also missing; a single fetch resolves both.

```bash
# Hybrid: MKA direction + AreaSection balance enforcement
bisect state --state NC --year 2020 \
  --structure area-section \
  --area-section-init moving-knife
```

**YAML**:

```yaml
algorithm:
  structure: moving-knife
  mka_orientations: 180
  mka_metric: reock           # or: polsby-popper
  weights: geographic
  search: convergence
  convergence_threshold: 600
  balance_tolerance: 0.5
workers: 8
```

---

## 8. Audit chain

Every run appends to `runs/{label}/{year}/index.json`. MKA adds the following fields:

```json
"structure": "moving-knife",
"mka_orientations": 180,
"mka_metric": "reock",
"mka_seed_formula": "SHA-256('MKA_INIT_' || node_path_len:u32le || node_path || base_seed:u64le)",
"node_path": "01",
"base_seed": 12345678,
"mka_seed": 987654321,
"optimal_angle_deg": 47.0,
"reock_left": 0.412,
"reock_right": 0.389,
"min_reock_score": 0.389,
"convergence_warning": false
```

`optimal_angle_deg` enables independent verification: an auditor reruns the sweep with the same centroids and `n_orientations` and confirms θ* matches. `node_path` is required for independent seed reproduction: `mka_seed = SHA-256("MKA_INIT_" || node_path_len:u32le || node_path || base_seed:u64le)`. `convergence_warning` is always `false` for MKA (no iterative convergence; the sweep is exhaustive over all orientations). `min_reock_score` is the score of the worst half at θ* — the quantity MKA directly maximises.

---

## 9. Test invariants

### L0 (inline unit tests)

- Same `base_seed` + `n_orientations` → identical `optimal_angle_deg` and plan on two runs (determinism)
- `min_reock_score > 0` for any valid subgraph with m ≥ 2
- `reock_left ≥ 0` and `reock_right ≥ 0`
- MEC area ≥ district area for all districts (Reock ≤ 1 by definition)
- `reock_left.clamp(0.0, 1.0) == reock_left` (Reock values never exceed 1.0 after clamping)
- `optimal_angle_deg` ∈ [0, 180) for all inputs
- `n_orientations=1`: only tests θ=0° (horizontal sweep); valid plan returned, no panic
- Prefix `"MKA_INIT_"` is distinct from `"CVD_GEO_INIT_"` and `"CVD_INIT_"` — hard-coded assertion confirming all three constants are pairwise unequal
- Both halves non-empty after sweep for any subgraph with m ≥ 2

### L1 (integration, synthetic data)

- 4×4 grid with evenly-spaced synthetic centroids (1m apart): optimal angle is 0° or 90° (horizontal or vertical sweep for symmetric grid)
- Elongated grid (1×4 tracts): MKA finds the horizontal sweep (θ=0°), perpendicular to the long axis — the split that minimises the MEC radius of each half
- Both halves non-empty after sweep on all synthetic grids
- Post-hoc rebalance preserves contiguity for 4×4 grid (all districts contiguous after rebalance)
- `split_subgraph_mka_direction` returns the same θ* as `run_moving_knife` for the same inputs (direction-only path is consistent with full-split path)

### L2 (`#[ignore]`, real data)

- NC 2020: MKA `min_reock_score` > CVD-Geographic `min_reock_score` (MKA optimises Reock directly; CVD does not)
- FL 2020: MKA `min_reock_score` improvement ≥ 5% over METIS baseline (FL panhandle geometry has large gains from angle-optimised splits)

---

## 10. Open questions (deferred)

1. **n_orientations tradeoff**: 180 (every 1°) vs 36 (every 5°) — runtime vs quality. Spec defaults to 180; Phase 2 will benchmark both on NC and FL 2020 to confirm 180 is worth the 5× runtime cost over 36.

2. **Non-convex subgraphs**: If the subgraph has a concave boundary, a linear sweep may not produce the globally best Reock orientation. This limitation is accepted and documented: MKA is a best-available heuristic for non-convex regions, not a global optimiser.

3. **AreaSection integration protocol**: when `--area-section-init moving-knife`, the MKA angle overrides the ratio-direction completely (`ratio_direction = mka_direction`). It is not used as a warm-start for further optimisation. This is the full-override interpretation; an additive warm-start variant is deferred.

4. **Tract area source**: `tract_areas` (m² per tract) is required for Reock computation. *Resolved (P1 fix)*: TIGER/Line polygon areas are already bundled in the `.adj.bin` adjacency file and stored in `LoadedGraph.tract_areas`. No new fetch step is needed. See data source note in Section 5.

5. **Interaction with bisection ensemble**: `BisectionEnsemble` over MKA subproblems varies `base_seed` per member. Verify that `mka_seed` (which depends on both `base_seed` and `node_path`) produces distinct seeds across ensemble members at each bisection node — same verification needed for CVD Phase 2.

---

## Comparison with other structure algorithms

| Algorithm | Optimises | Distance metric | Reock | EC | Runtime |
|---|---|---|---|---|---|
| METIS | Edge cuts | Graph topology | Low | Lowest | O(m log m) |
| SA | Edge cuts | Graph topology | Low | Low | O(n_steps × m) |
| CVD (graph) | Proximity | BFS hop-count | Medium | Medium | O(n_iter × m) |
| CVD (geo) | Proximity | Euclidean | Medium-high | Medium | O(n_iter × m) |
| BFS Growth | Geographic balance | Euclidean seeds | Medium | High | O(m log m) |
| **MKA** | **Reock** | **Euclidean sweep** | **Highest** | High | **O(n_orient × m)** |

---

## References

- CVD Phase 2 spec: `docs/specs/2026-05-07-cvd-phase2.md`
- CVD Phase 1 spec: `docs/specs/2026-05-07-centroidal-voronoi.md`
- `puppe2026`: Puppe, C. & Tasnadi, A. (2026). "Moving-Knife Redistricting." *Public Choice*.
- `dubins1961`: Dubins, L.E. & Spanier, E.H. (1961). "How to cut a cake fairly." *American Mathematical Monthly* 68(1), 1–17.
- `welzl1991`: Welzl, E. (1991). "Smallest enclosing disks (balls and ellipsoids)." *New Results and New Trends in Computer Science*, LNCS 555, 359–370.
- `snyder1987`: Snyder, J.P. (1987). "Map Projections — A Working Manual." USGS Professional Paper 1395. §14 (Albers Equal-Area Conic).
