# Track U -- Search and Optimization

**Theme**: Methods that search, optimize, certify, or select among valid
redistricting plans and algorithm families. These papers answer: once plan
construction methods exist, how do we choose, improve, certify, or compare
candidate plans?

This track absorbs the former `H-search` track and owns the `--search` layer
plus exact optimization, heuristic optimization, multi-objective optimization,
and practitioner algorithm selection.

## Papers

| Paper | Role |
|-------|------|
| U.1+convergence-sweep | Multi-seed convergence and statutory seed formula |
| U.2+parameter-sensitivity | Sensitivity of outcomes to tuning choices |
| U.3+simulated-annealing | Heuristic edge-cut optimization via annealing |
| U.4+parallel-tempering | Multi-chain replica-exchange optimization/search |
| U.5+adaptive-multiscale | Self-tuning multiscale search |
| U.6+ilp-redistricting | Exact optimization and optimality certificates |
| U.7+pareto-redistricting | Multi-objective NSGA-II Pareto search |
| U.8+percentile-sweep | Search percentile and legal posture selection |
| U.9+bisection-ensemble | Local ReCom search at bisection nodes |
| U.10+bisect-ensemble | High-performance Rust ensemble/search implementation |
| U.11+resolution-aware | Resolution as a first-class search/selection parameter |

## Needed Spine Papers

These should be added as native U-series papers after the physical refactor:

- U.0+search-optimization-overview
- U.12+algorithm-selection-matrix
- U.13+exact-vs-heuristic-certification
- U.14+multi-objective-selection
- U.15+legal-postures-for-search

## Candidate Additions

These are not manuscripts yet. They are the clean places to add algorithm
families that the current U-series only partially covers:

- U.16+branch-and-cut-redistricting -- exact MIP formulations with connectivity
  cuts, separation routines, and certificate reporting. U.6 covers ILP at a
  high level; this should be the branch-and-cut paper.
- U.17+branch-and-price-redistricting -- column generation / set-partitioning
  formulations where columns are feasible districts or district fragments.
- U.18+large-neighborhood-search -- ruin-and-repair, tabu search, variable
  neighborhood search, and matheuristic repair around valid plans.
- U.19+evolutionary-search-comparison -- genetic/evolutionary methods beyond
  U.7's NSGA-II Pareto framing, including crossover validity and repair.
- U.20+plan-audit-certificates -- RPLAN-compatible plan audits for contiguity,
  compactness, split limits, population, provenance, and topology constraints.
  Implementation spec: `docs/specs/2026-05-10-plan-audit-certificates.md`.
  RPLAN factoring spec: `docs/specs/2026-05-10-rplan-incubation.md`.
  RPLAN v0.2 schema: `docs/specs/2026-05-10-rplan-v0.2-schema.md`.

Implementation boundaries, CLI surfaces, and crate placement are specified in
`docs/specs/2026-05-10-algorithm-family-roadmap.md`.

All paper sources live in subdirectories here.
Compiled PDFs are in `docs/papers/`.
