# Algorithm Family Layer Cake

## Short Version

BISECT now has several algorithm families, but they are not a pile of unrelated
methods. They sit in layers:

- shared plan identity and audit crates at the bottom
- construction algorithms that make plans
- search and ensemble algorithms that move through plan space
- exact optimization algorithms that prove or bound solutions
- Pareto/evolutionary comparison algorithms that expose trade-offs
- analysis, reporting, maps, and UI at the top

The important fixed point is RPLAN: different algorithms may use very different
internal machinery, but final plan outputs converge back into RPLAN/RCTX/audit
certificate/manifest artifacts.

## Visual Map

```text
BISECT algorithm stack
======================

0. Shared substrate / neutral plan format
-----------------------------------------
rplan-core
  DistrictPlan, RplanContext, unit indexes, graph/pop/context hashes

rplan-io
  RPLAN/RCTX read-write, JSON schemas, stable serialization

rplan-audit
  audit certificates, legal profiles, contiguity/population checks,
  algorithm_lineage, certificate verification

bisect-core
  shared graph, population, partition, VRA/partisan primitives

bisect-data
  adjacency/data loading, serialization, TIGER/enacted helpers


1. Plan construction families: "make a plan"
--------------------------------------------
bisect-apportion
  recursive bisection composition
  prime-factor / apportion-regions
  spectral partitioning                       T.14
  nesting / split helpers

bisect-cli runner structures
  METIS recursive bisection
  GeoSection
  AreaSection
  VRASection
  CVD
  BFS growth
  Moving-Knife
  simulated-annealing construction refinement

bisect-clustering
  capacity-constrained clustering             T.15
  hierarchical regionalization                T.16
  repair-aware clustering outputs
  clustering/regionalization summaries

bisect-flow
  flow-based construction                      T.17
  capacity/cost/infeasibility witness baseline


2. Exact optimization families: "prove or bound a plan"
-------------------------------------------------------
bisect-ilp
  ILP formulation
  branch-and-cut / separation                  U.16
  connectivity cuts
  solve reports and audit summaries

bisect-column
  branch-and-price / column generation         U.17
  pricing problem
  master problem
  formulation-only and small exact fixture solve


3. Search / improvement families: "move through plan space"
-----------------------------------------------------------
bisect-local-search
  one-move local improvement                   U.18
  tabu/LNS scaffolding
  validity-preserving improvement summaries
  used by `bisect improve`

bisect-ensemble
  ReCom
  forest ReCom
  merge-split
  VRA ReCom
  ensemble sampling machinery

bisect-smc
  sequential Monte Carlo
  particles, proposal, resampling, SMC outputs

bisect-multiscale
  multiscale hierarchy
  coarse/fine chain machinery
  multiscale MCMC search


4. Multi-objective / evolutionary comparison
--------------------------------------------
bisect-pareto
  NSGA-II / Pareto frontier                    U.7 / U.19
  dominance + crowding distance
  crossover and mutation with validity fallback
  frontier NDJSON
  selected-frontier RPLAN/RCTX/audit packages


5. Analysis, reporting, visualization
-------------------------------------
bisect-analysis
  compactness, contiguity, demographics, partisan, VRA,
  bloc voting, splits, proportionality, diagnostics

bisect-report
  plan manifests
  report generation
  narrative/report audit identity
  comparison/civic gates
  RPLAN/report integration

bisect-map
  rendering/projection/dissolve/labels/colors

bisect-tui / bisect-web
  user interfaces over the algorithm/report layers


6. User-facing orchestration
----------------------------
bisect-cli
  `bisect state`
    --structure metis / spectral / capacity-clustering /
                regionalization / flow-construction / ilp / etc.

  `bisect improve`
    audited local-search improvement over an existing RPLAN/RCTX

  `bisect exact`
    branch-and-price reports and exact fixture packages

  `bisect pareto`
    NSGA-II frontier output
    selected frontier audit packages

  `bisect verify`
    manifest, ILP audit summary, RPLAN certificate verification

  YAML config parsing
  run manifests
  sidecars
  algorithm_lineage wiring
```

## Compact Mental Model

```text
                 reports / maps / UI
              bisect-report, map, TUI, web
                         ^
                         |
                 bisect-cli orchestration
                         ^
                         |
      +------------------+------------------+
      |                  |                  |
 construction        search/MCMC        exact/optimization
 T-series            U-series           U-series
      |                  |                  |
 apportion        ensemble / smc       ilp / column
 clustering       multiscale           branch-cut / price
 flow             local-search
                  pareto/evolution
      +------------------+------------------+
                         |
              rplan-core / rplan-io / rplan-audit
                         |
                    bisect-core / bisect-data
```

## The Main Distinction

The construction families answer: "How do we create a valid plan from the
state graph?"

The search families answer: "How do we explore or improve plans once we have a
valid starting point?"

The exact optimization families answer: "Can we prove optimality, compute
bounds, or produce solver-grade certificates for a small or staged instance?"

The Pareto/evolutionary family answers: "What trade-offs exist among competing
objectives, and which frontier plan should be exported for audit?"

The audit layer answers: "Regardless of how the plan was produced, can the
artifact be independently checked against a declared context and legal profile?"

## Crate-To-Concept Index

| Crate | Conceptual role |
|---|---|
| `rplan-core` | Neutral plan/context identity |
| `rplan-io` | RPLAN/RCTX serialization |
| `rplan-audit` | Certificates and verification |
| `bisect-core` | Shared graph/population/partition primitives |
| `bisect-data` | Census, adjacency, and enacted-plan data support |
| `bisect-apportion` | Recursive construction and apportionment-style split structures |
| `bisect-clustering` | Capacity clustering and regionalization |
| `bisect-flow` | Flow-based construction |
| `bisect-ilp` | ILP branch-and-cut exact optimization |
| `bisect-column` | Branch-and-price / column generation |
| `bisect-local-search` | Local improvement over existing plans |
| `bisect-ensemble` | ReCom-family ensemble sampling |
| `bisect-smc` | Sequential Monte Carlo sampling |
| `bisect-multiscale` | Multiscale chain/search methods |
| `bisect-pareto` | Pareto frontier and evolutionary comparison |
| `bisect-analysis` | Metrics, diagnostics, and evidence layers |
| `bisect-report` | Reports, manifests, and audit-facing summaries |
| `bisect-map` | Map rendering |
| `bisect-cli` | User-facing orchestration and sidecar emission |

## Why This Shape Matters

The fixed point is not one preferred algorithm. It is a shared artifact
contract. A flow constructor, a clustering constructor, an ILP solver, a local
search improver, and a Pareto frontier exporter can all produce different
plans, but the accepted final path is the same:

```text
algorithm output
  -> RPLAN plan
  -> RCTX context
  -> audit certificate
  -> manifest
  -> verify/report
```

That is what lets BISECT add new algorithm families without turning the
verification story into a separate bespoke process for every solver.

## Related Guides

- [three-layer-compositor.md](three-layer-compositor.md) explains the structure,
  weights, and search choices for `bisect state`.
- [section-algorithms.md](section-algorithms.md) explains the older B/T/U
  algorithm paper families.
- [ensemble-methods.md](ensemble-methods.md) explains ReCom-style ensemble
  evaluation.
- [label-pipeline.md](label-pipeline.md) explains run labels, output layout,
  and the audit chain.
