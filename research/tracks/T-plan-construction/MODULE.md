# Track T -- Plan Construction

**Theme**: Algorithms that construct a map or determine the structure of the
recursive redistricting tree. These papers answer: given a jurisdiction and a
target district count, how is a valid plan built?

This track owns the `--structure` side of the compositor. Search,
optimization, certification, and plan-selection methods live in
`U-search-optimization`.

## Papers

| Paper | Role |
|-------|------|
| T.1+geosection-ratio-optimal-bisection | Ratio-optimal first split and isoperimetric normalization |
| T.2+areasection-dual-population-area-constraint | Dual population/area structure |
| T.3+subdivision-respecting-redistricting | County/subdivision-respecting construction criterion |
| T.4+apportion-regions | Prime-factor bisection tree and ApportionRegions |
| T.5+proportional-section | Proportionality-aware structure variant |
| T.6+nestsection-nested-multi-chamber | Nested bicameral construction |
| T.7+vrasection-minority-opportunity-bisection | VRA-aware construction mode |
| T.8+stabilitysection-cross-census-stability | Stability-aware bisection structure |
| T.9+multi-reapportionment-stability | Construction behavior under seat gains/losses |
| T.10+centroidal-voronoi | CVD geographic construction |
| T.11+cvd-geographic | Geographic CVD extension |
| T.12+bfs-growth | BFS region-growing baseline |
| T.13+moving-knife | Reock-oriented moving-knife construction |

## Writing Queue

These construction families are implemented as audited vertical slices and now
need native manuscripts:

- T.14+spectral-partitioning -- Laplacian/Fiedler-vector graph partitioning as
  a deterministic construction baseline.
- T.15+capacity-constrained-clustering -- capacity-constrained clustering with
  population capacities, contiguity repair, and RPLAN audit sidecars.
- T.16+hierarchical-regionalization -- SKATER/Max-p style regionalization and
  deterministic agglomerative construction methods.
- T.17+flow-based-construction -- flow formulations used as constructive district
  builders rather than exact global optimizers.

Implementation boundaries, CLI surfaces, and crate placement are specified in
`docs/specs/2026-05-10-algorithm-family-roadmap.md`.
Paper-writing stages and panel-style review expectations are specified in
`docs/specs/2026-05-11-algorithm-family-paper-writing-goal.md`.

All paper sources live in subdirectories here.
Compiled PDFs are in `docs/papers/`.
