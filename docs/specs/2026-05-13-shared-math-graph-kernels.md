# Shared Math, Statistics, And Graph Kernel Crates

**Status:** Shared graph, statistics, math, and optimization kernels active
**Date:** 2026-05-13
**Scope:** Reusable Rust algorithm crates for BISECT, ROUTE, and future civic
evidence projects
**Review record:** [`shared-math-graph-kernels-r1_roles.md`](reviews/shared-math-graph-kernels-r1_roles.md)

## Decision

Create a small family of reusable Rust algorithm crates instead of continuing to
copy graph, statistics, and optimization kernels into each domain project.

These shared crates are reusable algorithm sets whose value comes from explicit
L0/L1/L2 coverage. Other projects can use them, but changes should stay
algorithm-driven: bug fixes, justified reuse, or coverage needed to protect a
domain algorithm.

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

Owns only shared numeric primitives that are reusable across higher-level
crates:

- small deterministic dense-matrix helpers;
- matrix/vector multiplication and transpose;
- Gauss-Jordan inversion with partial pivoting;
- exact rational arithmetic;
- stable summation;
- log-sum-exp;
- numeric tolerance policy;
- special functions needed by `rstat-core`.

Initial matrix helpers have landed because BISECT had audited WLS/HC3 matrix
math in `bisect-analysis::bloc_voting`, and ROUTE/optimization packages will
need the same small deterministic linear-algebra substrate without adopting a
large external dependency.

### `ropt-core`

Owns generic optimizer/search machinery only after specific duplication exists:

- deterministic seed streams;
- local-search transcript shape;
- simulated annealing schedule helpers;
- Pareto-front utilities;
- generic repair/reject reason records.

It does not own domain objectives or feasibility constraints.

The initial optimizer kernel has landed with generic minimization-objective
Pareto dominance, fast non-dominated sorting, and crowding distance in
`ropt-core`. `bisect-pareto` consumes these helpers while retaining
redistricting objective definitions and NSGA-II orchestration. Crowding-distance
normalization rejects overflowed objective ranges, spans, increments, and
distance sums with typed optimizer errors instead of returning non-finite
interior front distances.

Deterministic domain-separated seed derivation has also landed in `ropt-core`.
`bisect-pareto` keeps Pareto-specific domain labels (`PARETO_INIT_`,
`PARETO_CROSS_`, `PARETO_MUT_`) while delegating the SHA-256 seed kernel to the
shared crate.
The same seed kernel now supports existing SMC, multiscale, ensemble-chain, and
parallel-tempering seed helpers; those crates keep their own domain prefixes and
RNG semantics.
Seed transcripts include an explicit seed-kernel version tag, domain length,
part count, and `SeedPart` type tags so distinct domain/part boundaries cannot
collide through raw byte concatenation.

## First Wave: `rgraph-core`

The first implementation wave should create `rgraph-core` with a narrow API:

1. Minimal graph adapter trait for directed weighted graphs. **Landed in
   `rgraph-core`.**
2. `shortest_path_distance` with optional edge filter. **Landed in
   `rgraph-core`.**
3. `single_source_shortest_paths` returning distances, predecessor edges, and
   shortest-path counts. **Distance and shortest-path count overflow are
   rejected with typed graph errors before centrality can consume non-finite
   distances or path counts.**
4. `edge_betweenness` using weighted Brandes accumulation with deterministic tie
   handling. **Landed in `rgraph-core` during the first wave to preserve ROUTE
   compatibility fixtures.**
5. `reachable_nodes` for unweighted/filterable reachability. **Landed in
    `rgraph-core`; `bisect-analysis::contiguity` consumes it.**
6. Connected components over all nodes or a restricted node subset. **Landed in
   `rgraph-core`; `bisect-analysis::contiguity` now consumes restricted
   components directly.**
7. Undirected bridge detection over the directed adapter. **Landed in
   `rgraph-core`; this remains kernel-only until a contiguity-fragility or ROUTE
   redundancy consumer is added.**
8. Undirected articulation point detection over the directed adapter. **Landed in
   `rgraph-core`; this remains kernel-only until a contiguity-fragility or ROUTE
   redundancy consumer is added.**
9. Undirected edge-cut counting over adjacency lists and assignment labels.
   **Landed in `rgraph-core`; `bisect-local-search`, `bisect-pareto`, and
   `bisect-ensemble::parallel_tempering` consume it while keeping objective and
   sampling semantics local. Follow-on slice-consumer expansion also routes
   `bisect-apportion::spectral`, `bisect-clustering`, `bisect-flow`, and the CLI
   zero-based spectral summary helper through the shared kernel. The
    closure-based `undirected_edge_cut_by` adapter now lets HashMap-shaped
    bisection-runner helpers and HashSet-shaped evidence helpers reuse the
    graph-only edge traversal while keeping missing-assignment defaults and
    side-membership semantics local. `bisect-column` also uses the same adapter
    for bitmask membership labels without adding a bitmask-specific shared API.
    Edge-cut counting now treats adjacency as an unordered boundary list: duplicate
    entries and one-way high-to-low edges are counted once per unordered edge.
    Weighted cut scoring in `bisect-cli::bisection_runner` is consolidated as a
    local helper, not promoted to `rgraph-core`, because current reuse is confined
    to one module and weight-policy boundaries are CLI-specific.**
10. Assignment-label connectivity over adjacency lists. **Landed in
    `rgraph-core`; `bisect-local-search`, `bisect-clustering`, `bisect-flow`, and
    `bisect-ilp` consume the shared kernel while retaining domain wrappers for
    population balance, repair, search, and solver semantics. Connectivity is
    evaluated on the undirected closure of the adjacency list so one-way boundary
    adapters do not falsely fragment districts.**
11. Node-subset connectivity over adjacency lists. **Landed in `rgraph-core`;
    `bisect-column`, `bisect-smc::proposal`, `bisect-pareto::mutation`, CLI
    bisection subset checks, and SA evidence checks consume it while keeping
    empty-subset policy and pricing/proposal/mutation/evidence semantics local.
    It uses the same undirected adjacency policy as assignment-label
    connectivity.**
12. Unit tests with tiny synthetic graphs covering:
   - equal shortest-path split;
   - ignored non-shortest direct edge;
  - edge-filtered disconnection;
  - zero-node and one-node graphs;
  - negative/non-finite weight rejection policy.
   **The graph kernel now has inline L0 tests, L1 integration tests under
   `crates/rgraph-core/tests/l1_paths_centrality.rs`, and ignored L2 stress tests
   under `crates/rgraph-core/tests/l2_graph_stress.rs`.**

Route can then replace its local Dijkstra/Brandes implementation by adapting
`HighwayGraph`. BISECT now consumes the same crate for graph-only contiguity
component traversal without coupling to route data.

For adjacency-list helpers whose names describe undirected redistricting
boundaries (`undirected_edge_cut`, assignment-label connectivity, node-subset
connectivity, and restricted connected components), malformed one-way adjacency
lists are handled as weak/undirected graphs rather than silently producing
orientation-dependent answers. Directed shortest paths and reachability remain
directed by design.
Chamber-adjacency evidence construction now has a typed checked path that rejects
tract/adjacency length mismatches, zero or out-of-range district labels, and
out-of-range neighbor indices before nesting or chamber-boundary evidence can
panic on malformed graph rows.
Nesting validation now rejects zero house or senate district labels before
upper/lower chamber containment maps can include invalid district 0.
Contiguity evidence construction now applies the same typed validation for
tract/adjacency length mismatches, unknown assignment GEOIDs, zero or
out-of-range district labels, and out-of-range neighbor indices before district
component evidence can silently drop rows or traverse malformed adjacency.
Disconnected-tract county-context rendering also rejects malformed UTF-8 byte
boundaries before extracting five-byte county FIPS prefixes, so contiguity
evidence cannot panic while annotating non-primary components.
County-split evidence construction now exposes a checked GEOID parsing path that
rejects malformed UTF-8 byte boundaries before extracting five-byte county FIPS
prefixes, so malformed split records return typed errors instead of panicking.
County and municipal split aggregation now also reject zero district labels
before split evidence can report invalid district 0 containment.
Urban place aggregation now rejects zero or out-of-range district labels before
place evidence can create extra district rows outside the expected plan domain.
Plan-comparison evidence now rejects zero district labels in either plan before
Jaccard and tract-count deviation summaries can include an invalid district 0.
Canonical assignment comparison in report/interchange helpers likewise rejects
district 0 before relabeling, so round-trip equality evidence cannot normalize
an invalid label into a valid canonical district.
GerryChain assignment import now rejects district 0 at parse time, preventing
external interchange inputs from materializing a zero-district RPLAN.
CLI DRA CSV and tract-level GeoJSON imports apply the same parse-time district
0 rejection before deriving imported plan district counts.
Tract-level GeoJSON import also rejects features that provide a GEOID with a
missing or non-integer district property instead of silently skipping the row.
District-polygon GeoJSON import now validates `district_id` as a positive
integer before point-in-polygon assignment or nearest-centroid fallback can
materialize an invalid imported district.
District-polygon GeoJSON import also rejects malformed Polygon/MultiPolygon
coordinate rings instead of skipping invalid geometry or defaulting bad
coordinates to `(0, 0)`.
District-polygon GeoJSON import now rejects files with no usable polygon
geometry instead of returning an empty assignment set that later appears as a
zero-district import.
Comparison report assignment loading now rejects non-integer, zero, and
oversized `final_assignments.json` district values before diff evidence is
computed.
Comparison report manifest loading now rejects missing, zero, non-integer, and
oversized `num_districts` instead of defaulting malformed plan metadata to zero.
Comparison report analysis loading still treats absent optional analysis files as
unknown, but now rejects unreadable or malformed JSON instead of silently
defaulting parsed evidence fields.
Partisan comparison analysis now rejects non-numeric or missing Dem-share entries
in recognized per-district shapes instead of filtering them out of seat counts.
VRA comparison analysis now rejects malformed or oversized `mm_count` values
instead of defaulting present-but-invalid values to zero.
Compactness comparison analysis now rejects non-numeric or missing per-district
`polsby_popper` values instead of filtering them out of the mean.

## Second Wave: `rstat-core`

The statistics wave should begin with extraction, not invention.

Candidate first API:

1. `SummaryStats` for count, mean, variance, standard deviation, min, max.
   **Landed in `rstat-core::summary`. Summary aggregate overflow is rejected
   with typed errors instead of returning non-finite evidence fields.**
2. Deterministic quantiles with an explicit interpolation rule. **Landed as R-7
   quantiles and percentile intervals in `rstat-core::summary`. Quantile
   interpolation overflow is rejected with typed summary errors rather than
   emitting non-finite interval bounds.**
3. Autocorrelation, effective sample size, and R-hat helpers now living in
   `bisect-analysis`. **Landed in `rstat-core`; `bisect-analysis` keeps public
   record wrappers.**
4. Permutation-test helpers now living in `bisect-analysis`. **The beta CDF
   probability kernel used by the permutation report now lives in `rstat-core`.**
5. Exact or stable probability utilities that can later support `rcount-stats`,
    without moving RCOUNT method semantics out of `rcount-stats`. **Initial
    incomplete-beta and Lanczos-gamma helpers landed; the beta CDF now rejects
    non-finite `x` values and non-positive/non-finite shape parameters with typed
    probability errors. It also rejects overflowed shape-window and continued
    fraction intermediates before permutation or hypothesis evidence can consume
    non-finite beta-CDF values. Audit method semantics stay in `rcount-stats`.**
6. Seeded bootstrap resampling helpers. **Landed in `rstat-core::resampling`;
   `bisect-analysis::partisan` consumes them for bootstrap CIs. Partisan CLI
   ingestion and the fallible metrics wrapper reject non-finite, negative, or
   overflowed vote totals before bootstrap evidence can panic or emit non-finite
   intervals.**
7. Empirical p-values and multiple-testing corrections. **Landed in
   `rstat-core::hypothesis`; `bisect-analysis::permutation` and
   `bisect-analysis::bloc_voting` consume them. Named Holm correction rejects
   duplicate test names rather than silently collapsing evidence rows in the
   returned map.**
8. Weighted descriptive summaries. **Landed in `rstat-core::summary`;
   `bisect-analysis::bloc_voting` and `bisect-analysis::compactness` consume
   them. Weighted aggregate overflow is rejected with typed errors before
   weighted means or variances can become non-finite.**
9. Normal CDF approximation. **Landed in `rstat-core::probability`;
   `bisect-analysis::bloc_voting` consumes it for HC3 normal-approximation
   p-values. The helper rejects non-finite z-scores with a typed probability
   error so malformed regression evidence cannot silently propagate `NaN`
   p-values.**
10. Bootstrap percentile interval reuse. **`bisect-analysis::bloc_voting`
     cluster-bootstrap intervals now consume the tested R-7 percentile interval
     helper in `rstat-core::summary`. Percentile interval helpers reject reversed
     quantiles (`low_q > high_q`) with a typed `SummaryError` rather than
     returning `(high, low)` evidence bounds, and bloc-voting propagates summary
     errors instead of substituting `(NaN, NaN)` cluster intervals.**

Adjacent election-evidence consumers also enforce the same invariant before
calling or comparing shared statistical outputs: partisan, political, and
proportionality ingestion reject non-finite, negative, or overflowed vote totals
with typed errors instead of emitting non-finite vote shares, gaps, margins, or
bootstrap intervals.
Party-list D'Hondt allocation and Gallagher disproportionality helpers likewise
expose fallible typed-error paths so malformed vote maps cannot silently compare
NaN quotients or produce non-finite disproportionality scores.
Demographic aggregation now rejects negative counts, subgroup totals above
`total_pop`, and overflowing district sums before demographic percentages can
become impossible evidence fields.
VRA majority-minority aggregation now exposes a fallible checked path that
rejects invalid thresholds, negative tract totals, non-finite population
components, and component totals above tract population before district VRA
percentages are emitted.
Summary population-balance merging now rejects invalid balance tolerances and
negative population inputs before summary rows can emit invalid deviation or
balance flags.
Compactness geometry and population-weighted compactness consumers now reject
non-finite polygon coordinates, non-finite centroids, population length
mismatches, population overflows, and non-finite compactness intermediates before
compactness evidence can emit `NaN`/`inf` values or panic on malformed rows.
Race-of-candidate provenance parsing now rejects malformed curator attestation
dates unless they match `YYYY-MM-DD` with calendar-valid month/day fields
including leap-year handling, so annotation provenance cannot carry free-form or
impossible attestation dates.
It also rejects absolute, rooted, or parent-traversal attestation document paths
before hashing, preserving the relative-path evidence package boundary.

`rstat-core::mcmc` rejects malformed diagnostics inputs before producing evidence
metrics: R-hat rejects non-finite chain values with a typed diagnostics error,
ESS rejects non-finite trace values, integrated autocorrelation time rejects
non-finite or out-of-range lag values, and `hamming_autocorrelation` rejects
empty partition vectors so trajectory diagnostics cannot emit non-finite values
for malformed records. R-hat and ESS also reject overflowed aggregate arithmetic
with typed diagnostics errors before convergence records can contain non-finite
summary metrics.

RCOUNT should continue to own election-audit method replay. `rstat-core` owns
only reusable math.

The statistics kernel now has a three-level test ladder: inline L0 unit tests,
L1 integration tests under `crates/rstat-core/tests/l1_*`, and ignored L2 numeric
stress tests under `crates/rstat-core/tests/l2_numeric_stress.rs`.

## Third Wave: `rmath-core`

The numeric kernel starts with small dense linear algebra, extracted from
`bisect-analysis::bloc_voting`:

1. Row-major dense matrix type. **Landed in `rmath-core::DenseMatrix`.**
2. Matrix multiplication, matrix-vector multiplication, and transpose. **Landed
   in `rmath-core`; `bisect-analysis::bloc_voting` consumes matrix multiply and
   matrix-vector multiply. Matrix product overflow is rejected with a typed
   numeric error instead of returning non-finite regression matrices.**
3. Gauss-Jordan inverse with partial pivoting and typed errors. **Landed in
   `rmath-core`; bloc-voting WLS and HC3 now use it. Inversion rejects
   overflowed row normalization, row elimination, and result entries with typed
   numeric errors before callers can consume non-finite inverse matrices.**
4. Test ladder. **Inline L0 tests, L1 WLS normal-equation/pivoting tests, and an
   ignored L2 Hilbert-like inverse stress test landed under
   `crates/rmath-core/tests/`.**
5. Dot products, L2 norms, centering, in-place normalization, and centered
   normalization. **Landed in `rmath-core`; `bisect-apportion::spectral` consumes
   centered normalization and `bisect-data::fiedler` consumes dot, centering, and
   normalization helpers. Normalization thresholds must be finite and
   non-negative; negative thresholds are rejected with a typed error to avoid
   zero-vector division into non-finite values. Centering sum/mean overflow and
   dot/L2 norm overflow are also rejected before normalization can turn finite
   vectors into non-finite or silently collapsed outputs.**
6. Closed-form symmetric 2x2 eigensystem. **Landed in `rmath-core`;
   `bisect-cli::geosection_orientation` consumes the minor eigenvector for PCA
   minor-axis orientation. The eigensystem rejects non-finite intermediate
   results, including overflowed eigenvalues and eigenvector norms, instead of
   returning invalid orientation evidence.**

`rmath-core` remains numeric-only. It must not own regression semantics,
statistical inference, optimization objectives, or redistricting/route domain
interpretation.

The remaining possible `rmath-core` candidate is a graph-Laplacian
power-iteration helper if another consumer appears beyond the current
`bisect-data::fiedler` certificate.

## API Invariants

- Deterministic outputs for deterministic inputs.
- Stable tie handling documented in every shortest-path or ranking routine.
- No hidden global RNG. Randomized helpers must take an explicit seed or RNG.
- Seed transcript helpers must tag domain boundaries, part boundaries, and part
  types; domain prefixes alone are not enough to prevent cross-shape collisions.
- No panics for malformed public inputs; return typed errors.
- No silent dropping of invalid weights unless the function name and docs say so.
- Public interval helpers must reject reversed bounds rather than swapping or
  returning evidence intervals with `low > high`.
- Undirected adjacency-list helpers must be orientation-invariant for duplicated
  or one-way boundary entries.
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

