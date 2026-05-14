# Shared Math, Statistics, And Graph Kernel Crates

**Status:** First graph and statistics kernel waves complete
**Date:** 2026-05-13
**Scope:** Reusable Rust algorithm crates for BISECT, ROUTE, and future civic
evidence projects
**Review record:** [`shared-math-graph-kernels-r1_roles.md`](reviews/shared-math-graph-kernels-r1_roles.md)

## Decision

Create a small family of reusable Rust algorithm crates instead of continuing to
copy graph, statistics, and optimization kernels into each domain project.

The immediate need is not an R-language dependency. The need is a pure-Rust
substrate for deterministic algorithms that can be reused by redistricting,
route-network analysis, election audits, and later civic evidence packages.

The first crate should be graph-first:

```text
rgraph-core
```

The second crate should be statistics-first:

```text
rstat-core
```

Additional math and optimization crates should land only when duplicated code or
clear consumers justify them.

## Why This Exists

Two active repositories already need overlapping algorithmic kernels:

- `C:\src\apportionment` has graph partitioning, contiguity, spanning-tree
  sampling, compactness analysis, permutation tests, ensemble diagnostics,
  audit statistics, local search, simulated annealing, SMC, flow, ILP, and
  column-generation scaffolding.
- `C:\src\route` has highway graph construction, weighted shortest paths,
  Brandes betweenness centrality, redundancy/connectivity analysis, Wardrop
  assignment, incident simulation, and corridor scoring.

Those are not domain-specific at the kernel level. A weighted Dijkstra with
stable tie handling, connected-component check, edge betweenness routine,
quantile summary, or reproducible bootstrap should not be reimplemented for
every package family.

## Naming

Use short `r*` crate names only when the name means reusable/reproducible Rust,
not the R programming language.

Recommended names:

| Crate | Owns | First consumers |
|---|---|---|
| `rgraph-core` | Graph traversal, connectivity, centrality, cuts, spanning-tree helpers, deterministic graph summaries | `route-network`, `bisect-core`, `bisect-ensemble`, `bisect-analysis` |
| `rstat-core` | Deterministic statistical summaries, permutation/bootstrap helpers, MCMC diagnostics, exact/probability utilities | `rcount-stats`, `bisect-analysis`, `bisect-ensemble`, future RSTAT |
| `rmath-core` | Low-level numeric utilities only after reuse pressure appears | `rstat-core`, `rcount-stats` |
| `ropt-core` | General optimization/search kernels only after graph/stat foundations stabilize | `bisect-local-search`, `bisect-pareto`, `route-sim` |

Avoid a monolithic `algorithms` crate. Crates should stay small enough that a
domain project can depend on graph kernels without pulling statistical audit
math, or depend on statistics without pulling graph libraries.

## Non-Goals

- Do not create a new domain model for redistricting, routing, or election
  audits.
- Do not move package formats, manifests, or civic evidence records into these
  crates.
- Do not depend on R, Python, GEOS, METIS, or solver binaries from core kernels.
- Do not promise fastest-possible HPC performance before the API and invariants
  are stable.
- Do not extract every existing implementation in the first wave.

## Boundary Rules

### `rgraph-core`

Owns:

- graph traits/adapters over simple index-based graphs;
- deterministic weighted shortest paths;
- edge-filtered shortest paths;
- connected components and reachability;
- Brandes node/edge betweenness centrality;
- spanning-tree helpers needed by ReCom-style samplers;
- cut and boundary summaries when they are graph-only.

Does not own:

- Census/TIGER geography;
- highway corridor records;
- district assignments;
- METIS bindings;
- route scoring rules;
- legal or civic interpretations of graph results.

### `rstat-core`

Owns:

- stable descriptive summaries: mean, variance, quantiles, min/max, weighted
  summaries;
- deterministic bootstrap/permutation scaffolds;
- MCMC diagnostics such as autocorrelation, ESS, and R-hat;
- binomial, beta, and tail-probability utilities needed by multiple consumers;
- effect-size and interval helpers when formulas are generic.

Does not own:

- RCOUNT package verification;
- RLA method semantics;
- redistricting-specific fairness claims;
- certification or legal pass/fail outcomes;
- privacy/disclosure policy.

### `rmath-core`

Owns only shared numeric primitives that are reused by at least two higher-level
crates:

- exact rational arithmetic;
- stable summation;
- log-sum-exp;
- numeric tolerance policy;
- special functions needed by `rstat-core`.

If `rmath-core` would be a dump of helpers with one consumer, defer it.

### `ropt-core`

Owns generic optimizer/search machinery only after specific duplication exists:

- deterministic seed streams;
- local-search transcript shape;
- simulated annealing schedule helpers;
- Pareto-front utilities;
- generic repair/reject reason records.

It does not own domain objectives or feasibility constraints.

## First Wave: `rgraph-core`

The first implementation wave should create `rgraph-core` with a narrow API:

1. Minimal graph adapter trait for directed weighted graphs. **Landed in
   `rgraph-core`.**
2. `shortest_path_distance` with optional edge filter. **Landed in
   `rgraph-core`.**
3. `single_source_shortest_paths` returning distances, predecessor edges, and
   shortest-path counts.
4. `edge_betweenness` using weighted Brandes accumulation with deterministic tie
   handling. **Landed in `rgraph-core` during the first wave to preserve ROUTE
   compatibility fixtures.**
5. `reachable_nodes` for unweighted/filterable reachability. **Landed in
   `rgraph-core`; `bisect-analysis::contiguity` consumes it.**
6. Unit tests with tiny synthetic graphs covering:
   - equal shortest-path split;
   - ignored non-shortest direct edge;
   - edge-filtered disconnection;
   - zero-node and one-node graphs;
   - negative/non-finite weight rejection policy.

Route can then replace its local Dijkstra/Brandes implementation by adapting
`HighwayGraph`. BISECT now consumes the same crate for graph-only contiguity
component traversal without coupling to route data.

## Second Wave: `rstat-core`

The statistics wave should begin with extraction, not invention.

Candidate first API:

1. `SummaryStats` for count, mean, variance, standard deviation, min, max.
2. Deterministic quantiles with an explicit interpolation rule.
3. Autocorrelation, effective sample size, and R-hat helpers now living in
   `bisect-analysis`. **Landed in `rstat-core`; `bisect-analysis` keeps public
   record wrappers.**
4. Permutation-test helpers now living in `bisect-analysis`. **The beta CDF
   probability kernel used by the permutation report now lives in `rstat-core`.**
5. Exact or stable probability utilities that can later support `rcount-stats`,
   without moving RCOUNT method semantics out of `rcount-stats`. **Initial
   incomplete-beta and Lanczos-gamma helpers landed; audit method semantics stay
   in `rcount-stats`.**

RCOUNT should continue to own election-audit method replay. `rstat-core` owns
only reusable math.

## API Invariants

- Deterministic outputs for deterministic inputs.
- Stable tie handling documented in every shortest-path or ranking routine.
- No hidden global RNG. Randomized helpers must take an explicit seed or RNG.
- No panics for malformed public inputs; return typed errors.
- No silent dropping of invalid weights unless the function name and docs say so.
- Domain-separated transcript/hash helpers belong in domain crates, not here.
- Tests must include both positive and negative cases for every public function.

## Repository Relationship

The clean long-term home is a small standalone workspace or Git dependency that
both `apportionment` and `route` can consume. During incubation, it is acceptable
to create the first crate inside one repo and keep the API intentionally
portable, but the wave should record the extraction path.

Recommended path:

1. Incubate `rgraph-core` in `C:\src\apportionment\crates\rgraph-core` because
   the current wave machinery and role review live here. **Current decision:
   keep incubated until ROUTE can consume a portable shared dependency.**
2. Keep dependencies minimal: `thiserror`, `ordered-float`, and optionally
   `petgraph` only behind an adapter feature.
3. Add route consumption in `C:\src\route` after the API passes tiny graph tests.
   **Deferred: a direct local path dependency from ROUTE to apportionment would
   make ROUTE non-portable.**
4. Move to a shared repository only after two real consumers compile against it.

## Wave Acceptance Criteria

A wave around this spec is complete only when:

- `rgraph-core` exists with focused tests;
- one apportionment consumer or fixture uses it;
- one route consumer or compatibility adapter uses it, or a follow-up issue
  records the exact blocker;
- local duplicate graph code is removed or marked as a compatibility wrapper;
- `cargo fmt` and focused tests pass in every touched workspace;
- the spec is updated with any API decisions made during implementation.

`rstat-core` should be a later wave unless the first wave uncovers a statistics
dependency that must be solved immediately.

