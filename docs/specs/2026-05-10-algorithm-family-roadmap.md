# Spec: Algorithm Family Roadmap for T/U Refactor

**Date:** 2026-05-10  
**Status:** Draft, Round 1 role-reviewed  
**Scope:** New construction, search, and optimization algorithm families after the
Track B split into `T-plan-construction` and `U-search-optimization`.

**Review record:** [`algorithm-family-roadmap-r1_roles.md`](reviews/algorithm-family-roadmap-r1_roles.md)

## Decision

Add missing redistricting algorithm families as first-class papers and Rust
surfaces, but keep the implementation boundaries aligned with the current
workspace:

- `bisect-cli` owns user-facing flags, YAML parsing, run manifests, and the
  three-layer compositor.
- `bisect-core` owns shared graph, partition, population, and low-level validity
  types. New crates should not duplicate these.
- `bisect-apportion` owns recursive bisection tree composition and reusable
  construction traits.
- `bisect-ensemble` owns ReCom-family MCMC kernels.
- `bisect-smc`, `bisect-pareto`, `bisect-ilp`, and `bisect-multiscale` remain
  dedicated algorithm-family crates.
- New crates are warranted only when a family has a distinct solver/runtime,
  output contract, or dependency set.

## Existing Coverage

| Family | Current location | Status |
|--------|------------------|--------|
| Recursive/METIS construction | `bisect-cli`, `bisect-apportion`, `bisect-core` | implemented |
| GeoSection / AreaSection / VRASection | `bisect-cli::runner::SplitStrategy` | implemented |
| CVD / BFS / Moving-Knife | `bisect-cli::runner::SplitStrategy` | implemented |
| Simulated annealing | `bisect-cli::runner::SplitStrategy::SimulatedAnnealing` | implemented as construction refinement |
| ReCom / forest ReCom / merge-split / VRA ReCom | `bisect-ensemble` | implemented |
| SMC | `bisect-smc`, `bisect ensemble --method smc` | implemented |
| Pareto / NSGA-II | `bisect-pareto`, `bisect pareto` | implemented |
| ILP exact optimization | `bisect-ilp`, `--structure ilp` | implemented at baseline level |
| Multiscale MCMC | `bisect-multiscale`, `--search multiscale*` | implemented |

The missing work is not "add one more algorithm"; it is to round out the
taxonomy so construction, exact optimization, local search, evolutionary search,
and topology constraints have explicit homes.

## Track And Crate Map

| Paper | Algorithm family | Track | Rust home | CLI/YAML surface |
|-------|------------------|-------|-----------|------------------|
| T.14 | Spectral partitioning | Plan construction | new module in `bisect-apportion` first; crate only if it grows | `--structure spectral`, YAML `structure: spectral` |
| T.15 | Capacity-constrained clustering | Plan construction | new crate `bisect-clustering` | `--structure capacity-clustering`, `bisect cluster` optional later |
| T.16 | Hierarchical regionalization | Plan construction | `bisect-clustering` | `--structure regionalization` |
| T.17 | Flow-based construction | Plan construction | new crate `bisect-flow` | `--structure flow-construction` |
| U.16 | Branch-and-cut exact optimization | Search/optimization | extend `bisect-ilp` | `--structure ilp --ilp-method branch-and-cut` |
| U.17 | Branch-and-price / column generation | Search/optimization | new crate `bisect-column` | `bisect exact --method branch-and-price` |
| U.18 | Large-neighborhood / tabu / matheuristics | Search/optimization | new crate `bisect-local-search` | `--search lns`, `--search tabu`, `bisect improve` |
| U.19 | Evolutionary search comparison | Search/optimization | extend `bisect-pareto`; optional `bisect-evolution` only if single-objective GA grows | `bisect pareto --method nsga2`, `--method evolutionary` |
| U.20 | Plan audit certificates | Search/optimization / audit | new generic crates `rplan-core`, `rplan-io`, `rplan-audit` | `rplan audit`, used by all solvers; spec: [`2026-05-10-plan-audit-certificates.md`](2026-05-10-plan-audit-certificates.md); RPLAN v0.2 schema: [`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md); factoring: [`2026-05-10-rplan-incubation.md`](2026-05-10-rplan-incubation.md) |

## Shared Abstractions To Add First

Before adding the family crates, add shared interfaces so every family can plug
into the compositor without one-off glue.

### `rplan-core`

Add a common plan object that is independent of bisect:

```rust
pub struct PlanUnitIndex {
    pub unit_ids: Vec<String>,
    pub unit_kind: UnitKind,
    pub state: String,
    pub year: u16,
}

pub struct DistrictPlan {
    pub units: PlanUnitIndex,
    pub assignment: Vec<u32>,
    pub k: usize,
}

pub struct PlanStats {
    pub populations: Vec<u64>,
    pub population_deviation_ppm: Vec<i64>,
    pub edge_cut: u64,
    pub cut_fraction: f64,
    pub contiguous_by_district: Vec<bool>,
    pub split_counts: SplitCounts,
}
```

This should replace ad hoc `Vec<usize>` / `Vec<u32>` plan passing where new
crates need stable interfaces. The unit index is mandatory: every plan must
carry enough information to map assignment entries back to tract/block-group/
block GEOIDs without relying on external ordering conventions.

### `bisect-core::objective`

Add reusable objective traits:

```rust
pub trait PlanObjective {
    fn name(&self) -> &'static str;
    fn evaluate(&self, ctx: &PlanContext, plan: &DistrictPlan) -> f64;
    fn direction(&self) -> ObjectiveDirection;
}
```

Initial objectives: edge cut, population deviation, county splits, compactness
proxy, VRA opportunity deficit, partisan seat deviation.

`PlanContext` must carry the graph, population vector, optional geometries,
subdivision membership, VAP/demographic tables, and election tables. Objective
implementations must return structured "missing input" errors rather than
silently degrading when required data are absent.

### `rplan-audit`

Create a generic audit crate that every algorithm can call:

- contiguity check and disconnected-component diagnosis
- population tolerance check, parameterized by jurisdiction, chamber, and year
- split-count constraints, parameterized by state legal rule rather than a
  universal split definition
- VRA feasibility hooks
- geometry/topology checks when polygons are available
- audit-certificate JSON for why a plan is valid or invalid, including schema version,
  source-data hashes, binary/version metadata, and district-level witnesses

This crate should depend on `rplan-core`, `rplan-io`, and optionally
`rplan-geo`; bisect and algorithm crates depend on it, not the other way
around.

## Cross-Cutting Requirements From Role Review

These requirements apply to every new family below.

### Legal Parameterization

Do not hardcode congressional assumptions into shared validators. Congressional
plans use strict equal-population requirements; state legislative plans require
state- and chamber-specific tolerances and subdivision rules. The shared
constraint API must accept:

- state
- chamber type
- year
- ideal population and tolerance rule
- subdivision preservation rule
- VRA data availability and threshold policy

The default CLI path may use congressional defaults, but certificates must state
which legal profile was applied.

### Data Provenance

Every algorithm that uses geography, demographic data, election data, or
subdivision membership must declare required inputs and their source vintages.
At minimum, manifests and certificates must record:

- PL 94-171 source hash for population inputs
- TIGER/Line vintage and source hash for geometry/adjacency
- GEOID format and unit kind
- ACS/election source hashes when used
- projection used for Euclidean distances or compactness proxies

### Audit And Reproducibility

Every mode must emit enough metadata for independent replay:

- crate and binary version
- git commit or release tag
- solver name/version for external solvers
- all random seeds and sub-seed derivation tags
- full parameter set, including defaults
- fallback path, if any, with reason
- final certificate hash

External solver or subprocess use must be disclosed in the certificate.

### Interoperability

New outputs must be versioned from day one:

- plan assignment JSON / RPLAN schema version
- certificate JSON schema version
- optional GeoJSON export profile with RFC 7946 assumptions
- GerryChain/DRA/PlanScore export compatibility notes when supported

No crate should invent an unversioned output format.

## T-Series Construction Additions

### T.14 Spectral Partitioning

**Purpose:** deterministic graph-partitioning baseline using Laplacian spectral
cuts.

**Rust location:** start as `bisect-apportion::spectral`. Do not create a crate
unless we add eigensolver dependencies that should stay isolated.

**Implementation:**

- Build weighted graph Laplacian from `Graph` adjacency and edge weights.
- Compute approximate Fiedler vector with deterministic Lanczos/power iteration.
- Sort vertices by Fiedler value.
- Sweep cut positions satisfying population balance and pick minimum cut.
- Recurse through existing bisection compositor.
- Record eigensolver tolerance, iteration count, and convergence status in the
  manifest. If the eigensolver fails to converge, return a structured error or
  explicit fallback; do not silently return a partial vector.

**CLI:**

- `--structure spectral`
- `--spectral-iters N` default 200
- `--spectral-sweep population-balanced|min-cut` default `population-balanced`

**Tests:**

- path graph splits near midpoint
- two-clique bridge graph cuts the bridge
- deterministic same seed/input
- manifest records spectral parameters

### T.15 Capacity-Constrained Clustering

**Purpose:** k-means/k-medoids/regionalization with hard population capacities
and contiguity repair.

**Rust location:** new crate `bisect-clustering`.

**Modules:**

- `seeds.rs`: k-means++, farthest-point, county-aware seed placement
- `assign.rs`: capacity-constrained assignment
- `repair.rs`: contiguity and population repair
- `metrics.rs`: within-cluster distance, cut fraction, compactness proxy
- `output.rs`: audit metadata

**CLI:**

- `--structure capacity-clustering`
- `--cluster-method kmedoids|balanced-kmeans`
- `--cluster-seeds farthest|kmeans-plus-plus|county`
- `--cluster-repair bfs|flow`
- YAML `structure: capacity-clustering`

**Tests:**

- grid graph produces k connected districts
- impossible population capacity returns `[CONFIG]`/`[ALGO]` error
- repair never worsens population balance beyond tolerance

### T.16 Hierarchical Regionalization

**Purpose:** SKATER/Max-p style regionalization and agglomerative construction.

**Rust location:** same `bisect-clustering` crate.

**Implementation:**

- minimum spanning tree over tract graph
- edge removal under population capacity
- agglomerative merge mode for small units
- optional county/subdivision boundary penalty from existing weight layer

**CLI:**

- `--structure regionalization`
- `--regionalization-method skater|max-p|agglomerative`
- `--regionalization-min-pop-ratio F`

**Tests:**

- MST cut count is `k - 1`
- all output regions contiguous
- county penalty changes cut choice on synthetic graph

### T.17 Flow-Based Construction

**Purpose:** use min-cost flow / balanced flow as a constructive district builder,
not as a global exact optimizer.

**Rust location:** new crate `bisect-flow`.

**Implementation:**

- Declare centroid/projection requirements up front.
- source/sink or multi-commodity approximation for assigning tracts to seeds
- capacities enforce population bounds
- cost combines distance, edge boundary, and optional county penalty
- repair stage calls `rplan-audit`

**CLI:**

- `--structure flow-construction`
- `--flow-seeds farthest|existing`
- `--flow-cost distance|edge-cut|hybrid`
- `--flow-repair none|bfs`

**Tests:**

- capacity bounds respected on small graphs
- infeasible capacity detected
- deterministic output for fixed seed

## U-Series Search And Optimization Additions

### U.16 Branch-And-Cut Redistricting

**Purpose:** make exact optimization credible by separating connectivity cuts
instead of relying on a static ILP formulation.

**Rust location:** extend `bisect-ilp`.

**Modules to add:**

- `connectivity_cuts.rs`
- `separation.rs`
- `certificates.rs`
- `mps.rs` if not already complete enough

**CLI/YAML:**

- `--structure ilp --ilp-method branch-and-cut`
- YAML `structure: ilp`, `ilp_method: branch-and-cut`
- keep current fallback behavior behind `--ilp-fallback metis|error`

The spec must distinguish true lazy-callback branch-and-cut from iterative
solve-separate-resolve branch-and-cut. Solver support varies. If the selected
solver does not support lazy cuts, the manifest must record the iterative
separation mode and its stopping criteria.

**Tests:**

- disconnected incumbent solution triggers a cut
- connected solution emits zero violated connectivity cuts
- certificate reports lower bound, incumbent, gap, and cut count

### U.17 Branch-And-Price / Column Generation

**Purpose:** set-partitioning formulation where columns are feasible districts or
district fragments.

**Rust location:** new crate `bisect-column`.

**Modules:**

- `pricing.rs`: generate negative-reduced-cost district columns
- `master.rs`: restricted set-partitioning master problem
- `pool.rs`: district column pool and dominance pruning
- `branching.rs`: Ryan-Foster or pairwise branching
- `output.rs`: certificate metadata

**CLI:**

- Add `bisect exact --method branch-and-price`
- Inputs: state/year, k, population tolerance, optional column pool path
- Output: JSON certificate plus optional plan assignment

This family is not a `--search` mode. It has a different lifecycle from ordinary
state runs: column pool generation, restricted master solve, pricing iterations,
branching, and certificate emission. Keep it behind `bisect exact` until the
certificate format is stable.

**Tests:**

- tiny graph solves same optimum as ILP
- column pool serialization round trip
- infeasible master returns structured status, not panic

### U.18 Large-Neighborhood Search / Tabu / Matheuristics

**Purpose:** practical improvement around a valid starting plan using
ruin-and-repair, tabu search, variable-neighborhood search, and optional exact
repair.

**Rust location:** new crate `bisect-local-search`.

**Modules:**

- `neighborhood.rs`: boundary flips, district-pair recombination, ruin sets
- `tabu.rs`: tabu tenure and aspiration criteria
- `lns.rs`: destroy/repair loop
- `repair.rs`: BFS, flow, or ILP repair hooks
- `accept.rs`: best-improvement, simulated-annealing, threshold accepting

**CLI:**

- `bisect improve --input plan.json --method lns|tabu|vns`
- `--search lns` for inline compositor use
- `--lns-iters`, `--lns-destroy-frac`, `--tabu-tenure`

**Tests:**

- valid input plan remains valid after every accepted move
- objective is non-increasing in best-improvement mode
- tabu list prevents immediate reversal on synthetic graph

### U.19 Evolutionary Search Comparison

**Purpose:** separate the general evolutionary-search family from U.7's
multi-objective Pareto framing.

**Rust location:** extend `bisect-pareto` first. Create `bisect-evolution` only
if single-objective GA, memetic algorithms, or island models become substantial.

**Implementation:**

- expose GA operators independently of NSGA-II
- add selectable methods: `nsga2`, `single-objective-ga`, `smc-pareto`
- make repair/validity calls go through `rplan-audit`
- report invalid offspring rate and repair success rate; these are required to
  compare evolutionary methods honestly rather than only reporting the final
  frontier.

**CLI:**

- `bisect pareto --method nsga2|smc-pareto`
- optional later: `bisect evolve --objective edge-cut|compactness|custom`

**Tests:**

- crossover plus repair preserves contiguity
- mutation preserves assignment length and district labels
- fixed seed gives identical frontier metadata

### U.20 Plan Audit Certificates

**Purpose:** make contiguity, splits, compactness, and topology constraints
auditable across every algorithm family.

**Rust location:** generic crates `rplan-core`, `rplan-io`, and
`rplan-audit`, incubated in this repo under the RPLAN factoring spec.

**CLI:**

- `rplan audit --plan PATH --state ST --year YYYY`
- `--constraints contiguity,population,splits,vra,compactness`
- JSON output usable by `bisect report`

**Integration:**

- `bisect-cli::runner` calls the audit layer before writing final run manifest.
- `bisect-ilp`, `bisect-pareto`, `bisect-smc`, `bisect-local-search`,
  `bisect-clustering`, and `bisect-flow` use the same validators.

**Tests:**

- disconnected district produces component witness
- population failure reports offending district ids
- valid plan has stable JSON certificate schema

## CLI Integration Plan

### `PartitionMode` / `StructureMode`

Add variants:

- `Spectral`
- `CapacityClustering`
- `Regionalization`
- `FlowConstruction`

Map them to new `SplitStrategy` variants in `bisect-cli::runner`:

- `SplitStrategy::Spectral { iters, sweep }`
- `SplitStrategy::CapacityClustering { method, seeds, repair }`
- `SplitStrategy::Regionalization { method, min_pop_ratio }`
- `SplitStrategy::FlowConstruction { seed_mode, cost, repair }`

### `SearchMode`

Add variants:

- `Lns`
- `Tabu`
- optional `Evolutionary` only if used inline rather than via `bisect pareto`

Exact branch-and-price should not be a `SearchMode`; it should be a subcommand
or ILP method because it has a different output contract and solver lifecycle.

### Subcommands

Add:

- `bisect exact` for exact optimization workflows that are not ordinary
  state-run structure modes.
- `bisect improve` for local search from an existing plan.
- `rplan audit` for topology and compliance audit certificates.

Keep `bisect pareto` for Pareto/evolutionary frontiers and `bisect ensemble` for
distributional ensemble generation.

## Workspace Changes

Add these members to root `Cargo.toml` when implementation starts:

```toml
"crates/rplan-core",
"crates/rplan-io",
"crates/rplan-audit",
"crates/bisect-clustering",
"crates/bisect-flow",
"crates/bisect-local-search",
"crates/bisect-column",
```

Recommended dependency direction:

```text
bisect-cli -> algorithm crates
bisect-cli -> rplan-audit
bisect-cli -> rplan-io

algorithm crates -> bisect-core
algorithm crates -> rplan-core
algorithm crates -> rplan-audit

rplan-io -> rplan-core
rplan-audit -> rplan-core
rplan-audit -> rplan-io
rplan-audit -> rplan-geo / geo (optional)
```

Avoid dependencies from `bisect-core` to any algorithm crate.
Avoid dependencies from `rplan-*` to any `bisect-*` or algorithm crate.

## Paper/Spec Creation Order

1. RPLAN incubation / `rplan-core`, `rplan-io`, `rplan-audit`: shared audit
   foundation first; see
   [`2026-05-10-rplan-incubation.md`](2026-05-10-rplan-incubation.md) and
   [`2026-05-10-rplan-v0.2-schema.md`](2026-05-10-rplan-v0.2-schema.md) and
   [`2026-05-10-plan-audit-certificates.md`](2026-05-10-plan-audit-certificates.md).
2. U.16 / `bisect-ilp` branch-and-cut: upgrades existing exact optimization.
3. T.15 / `bisect-clustering`: covers the largest missing construction family.
4. U.18 / `bisect-local-search`: gives practical improvement methods and repair
   infrastructure.
5. T.14 / spectral: cheap deterministic construction baseline.
6. T.16 / regionalization: builds on clustering infrastructure.
7. U.17 / `bisect-column`: heavier exact optimization after certificates exist.
8. T.17 / `bisect-flow`: constructive flow once repair/certification is stable.
9. U.19 / evolutionary comparison: extend Pareto once repair is shared.

## Acceptance Criteria

For each new family:

- a track paper slot exists in T or U
- a `docs/specs/YYYY-MM-DD-*.md` implementation spec exists
- the crate/module boundary is listed in this roadmap
- CLI help, YAML schema, and run manifest include the new mode
- output/certificate schema versions are declared
- data inputs and source hashes are declared
- L0 synthetic tests cover validity and determinism
- at least one L1 integration test exercises the `bisect-cli` surface
- all final plans pass `rplan-audit`
- at least one role-review pass is complete before implementation begins
