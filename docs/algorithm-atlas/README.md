# Algorithm Atlas

The Algorithm Atlas is the visual field guide for BISECT's algorithm families.
It complements `docs/concepts/` and the research papers with small diagrams,
plain-language summaries, and links to the crates, CLI surfaces, papers, and
RPLAN packages that make each method concrete.

All current atlas pages meet the upgraded gold-standard threshold in the
[2026-05-11 scorecard](reviews/atlas-scorecard-2026-05-11.md). Pages are
reviewed against the [BISECT Algorithm Atlas Rubric](RUBRIC.md), a six-part
scorecard for behavioral mechanics, BISECT integration, spatial visual
specificity, explanatory story, claim boundaries, and traceable evidence.

The companion [Visual Patterns](VISUAL-PATTERNS.md) guide records the rule that
now drives the atlas: show the split, move, assignment, cut, repair, selection,
or sample transition directly. Do not merely label it.

## Overview

![BISECT algorithm atlas overview](assets/overview.svg)

Every family can use different internal machinery, but publication-grade plan
outputs converge on the same fixed point:

```text
algorithm output -> RPLAN -> RCTX -> audit certificate -> manifest -> verifier
```

## How BISECT Uses These Algorithms

BISECT is not only a collection of standalone algorithms. The construction
methods are ways to make the next auditable plan-building decision: choose a
balanced split, choose seeds, grow capacity-aware assignments, merge regions, or
prove that the declared construction profile cannot be satisfied.

| Role In BISECT | Algorithms That Play This Role |
|---|---|
| Choose a balanced bisection cut | T.14 spectral partitioning |
| Choose seed centers for district growth | T.15 capacity clustering, T.17 flow construction |
| Assign units under population capacity | T.15 capacity clustering, T.17 flow construction |
| Build larger connected regions from smaller ones | T.16 hierarchical regionalization |
| Emit infeasibility or repair evidence instead of hiding failure | T.15 capacity clustering, T.17 flow construction |

In other words, the algorithm is the construction engine, but the BISECT system
cares just as much about the evidence trail. A method is useful when its choices
can be replayed, summarized, packaged as RPLAN/RCTX, and checked by the verifier.

## Reading Paths

Read the atlas by the question you are trying to answer:

| Question | Start Here | Then Read |
|---|---|---|
| How does BISECT divide geography? | [GeoSection](geosection.md) | [AreaSection](areasection.md), [County-Sticky Weights](county-sticky-weights.md), [ApportionRegions](apportionregions.md) |
| How do construction algorithms build plans? | [T.14 Spectral Partitioning](t14-spectral-partitioning.md) | [T.15 Capacity Clustering](t15-capacity-clustering.md), [T.16 Hierarchical Regionalization](t16-hierarchical-regionalization.md), [T.17 Flow Construction](t17-flow-construction.md) |
| How do exact and search methods explain their claims? | [U.16 Branch-And-Cut](u16-branch-and-cut.md) | [U.17 Branch-And-Price](u17-branch-and-price.md), [U.18 Local Search](u18-local-search.md), [U.19 Evolutionary Comparison](u19-evolutionary-comparison.md) |
| How do sampling methods explain movement and diagnostics? | [ReCom Ensemble](recom-ensemble.md) | [Sequential Monte Carlo](sequential-monte-carlo.md), [Multiscale MCMC](multiscale-mcmc.md) |
| How does everything become audit evidence? | [U.20 RPLAN Audit Certificates](u20-rplan-audit-certificates.md) | [Three-Layer Compositor](three-layer-compositor.md), [Seed Search Modes](seed-search-modes.md) |

## Bisection Compositor Family

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| Three-Layer Compositor | [Three-Layer Compositor](three-layer-compositor.md) | Structure, weights, search as independent choices |
| GeoSection | [GeoSection](geosection.md) | Ratio scan, isoperimetric normalization, caterpillar avoidance |
| AreaSection | [AreaSection](areasection.md) | Population/area dual constraint and Lorenz feasibility |
| ApportionRegions | [ApportionRegions](apportionregions.md) | Prime-factor tree and reusable regional spine |
| County-Sticky Weights | [County-Sticky Weights](county-sticky-weights.md) | Intra-county edge boost and compactness/split trade-off |
| Seed Search Modes | [Seed Search Modes](seed-search-modes.md) | Single, multi, convergence, percentile, bisection-ensemble |

## Visual Grammar

The atlas uses a few repeated visual conventions so the pages can be read as a
family rather than as isolated pictures:

| Shape | Meaning |
|---|---|
| Graph nodes and edges | Units, precincts, blocks, or intermediate regions |
| Thick divider | A candidate cut, cluster boundary, or assignment boundary |
| Colored region | A district, cluster, flow bin, or merged regional unit |
| Table-like witness | Data recorded so the result can be replayed or audited |
| Red or amber path | Constraint pressure, repair, or infeasibility behavior |
| Blue package rail | The fixed point from algorithm output to RPLAN/RCTX/certificate |

Each algorithm page is organized around four questions:

1. What object does the algorithm operate on?
2. What choice does it make at each step?
3. What evidence is emitted so another tool can replay or reject the result?
4. Where is the claim boundary, especially when a result is infeasible or only
   benchmark-tier?

## Gold-Standard Checklist

New or revised atlas pages should satisfy the same test before being treated as
excellent:

- Show the candidate objects, not just the final result.
- Show why those candidates exist: adjacency, ratio scan, seed rule, pricing
  round, particle proposal, or solver incumbent.
- Show the decision rule in the visual: accepted, rejected, skipped, repaired,
  resampled, selected, or proven by status.
- Show the downstream consequence: recursive workload, package sidecar,
  verifier status, merge log, solve report, frontier entry, or diagnostics.
- State the claim boundary so readers know what the method does not prove.
- Name the evidence fields a reviewer can inspect in RPLAN/RCTX, summaries,
  transcripts, reports, packages, or NDJSON output.

## Construction Family

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| T.14 Spectral Partitioning | [T.14 Spectral Partitioning](t14-spectral-partitioning.md) | Fiedler ordering, sweep cuts, deterministic construction |
| T.15 Capacity-Constrained Clustering | [T.15 Capacity Clustering](t15-capacity-clustering.md) | Seeds, capacity-aware assignment, repair/status lineage |
| T.16 Hierarchical Regionalization | [T.16 Hierarchical Regionalization](t16-hierarchical-regionalization.md) | Adjacent merges, merge log, hierarchy depth |
| T.17 Flow-Based Construction | [T.17 Flow Construction](t17-flow-construction.md) | Seeds, capacities, flow-style assignment, infeasibility witness |

## Search, Optimization, And Audit Family

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| U.16 Branch-And-Cut | [U.16 Branch-And-Cut](u16-branch-and-cut.md) | ILP model, connectivity cuts, solver status, bounds/gaps |
| U.17 Branch-And-Price | [U.17 Branch-And-Price](u17-branch-and-price.md) | District columns, pricing, master problem, formulation status |
| U.18 Local Search | [U.18 Local Search](u18-local-search.md) | Boundary moves, validity-preserving improvement, search summary |
| U.19 Evolutionary Comparison | [U.19 Evolutionary Comparison](u19-evolutionary-comparison.md) | Crossover/mutation, Pareto frontier, selected-frontier package |
| U.20 RPLAN Audit Certificates | [U.20 RPLAN Audit Certificates](u20-rplan-audit-certificates.md) | Fixed point, hashes, certificate verification, failure reasons |

## Sampling And Ensemble Family

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| ReCom Ensemble | [ReCom Ensemble](recom-ensemble.md) | Merge adjacent districts, sample spanning tree, cut balanced edge |
| Sequential Monte Carlo | [Sequential Monte Carlo](sequential-monte-carlo.md) | Particles, staged district proposals, weights, ESS resampling |
| Multiscale MCMC | [Multiscale MCMC](multiscale-mcmc.md) | Coarse/fine hierarchy, tract moves, block-group moves, rebalance |

## Review Artifacts

| Artifact | Purpose |
|---|---|
| [Rubric](RUBRIC.md) | Defines the six BISECT scoring dimensions |
| [Visual Patterns](VISUAL-PATTERNS.md) | Captures reusable figure rules and anti-patterns |
| [Scorecard](reviews/atlas-scorecard-2026-05-11.md) | Records the current all-golden atlas review |
| [Role Review](reviews/rubric-r1_roles.md) | Preserves the `.roles` review pass for the rubric |

## Possible Future Pages

Future atlas pages could cover VRASection, ProportionalSection, NestSection,
StabilitySection, Moving-Knife, compact-Polsby, CVD, BFS growth, and simulated
annealing. Add them only when the page can meet the gold-standard checklist
above.

## Relationship To Other Docs

- `docs/concepts/algorithm-family-layer-cake.md` is the crate and evidence
  taxonomy.
- `docs/concepts/t-u-portfolio-dependency-map.md` maps papers, packages, and
  verifier paths.
- `docs/PAPERS.md` is the publication index.
- `docs/examples/rplan-benchmark-packages/` contains committed benchmark-tier
  packages referenced by these pages.
