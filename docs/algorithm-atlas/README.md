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
| How do RCOUNT election audits replay statistical evidence? | [V.16 SHANGRLA Assorters](v16-shangrla-assorters.md) | [V.15 ALPHA](v15-alpha-betting-martingale.md), [V.13 Minerva/Athena](v13-minerva-athena-ballot-polling.md), [V.14 Kaplan-Markov/MACRO](v14-kaplan-markov-macro-comparison.md) |

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

## RCOUNT Audit Algorithm Family

These pages extend the atlas from RPLAN-producing algorithms into RCOUNT audit
replay. V-series pages are certification/audit algorithms. W-series pages are
exploratory analytics that can guide investigation but must not be treated as
certifying evidence.

| Algorithm | Visual Guide | What To Look For |
|---|---|---|
| V.12 BRAVO Ballot-Polling RLA | [V.12 BRAVO](v12-bravo-ballot-polling.md) | Sequential likelihood ratio, ballot observations, stop/continue boundary |
| V.13 Minerva And Athena | [V.13 Minerva/Athena](v13-minerva-athena-ballot-polling.md) | Round-level ballot-polling risk measurements and RI/Arlo-style reports |
| V.14 Kaplan-Markov / MACRO | [V.14 Kaplan-Markov/MACRO](v14-kaplan-markov-macro-comparison.md) | CVR/hand comparison, overstatement errors, running P-values |
| V.15 ALPHA And Betting Martingales | [V.15 ALPHA](v15-alpha-betting-martingale.md) | Assorter values, bets, martingale transcript, adaptive stopping |
| V.16 SHANGRLA Assorters | [V.16 SHANGRLA](v16-shangrla-assorters.md) | Outcome assertions, assorter formulas, mean tests |
| V.17 Stratified And Hybrid RLAs | [V.17 Stratified/Hybrid](v17-stratified-suite-hybrid.md) | Polling/comparison strata, combining rules, nuisance parameters |
| V.18 Batch Comparison | [V.18 Batch Comparison](v18-batch-comparison.md) | Batch hand tallies, reported batch totals, batch overstatements |
| V.19 RAIRE And AWAIRE | [V.19 RAIRE/AWAIRE](v19-raire-awaire-rcv.md) | RCV/IRV assertion sets and ranked-ballot evidence |
| V.20 Bayesian Tabulation Audits | [V.20 Bayesian Audits](v20-bayesian-tabulation-audits.md) | Priors, posterior outcome probabilities, calibration boundary |
| V.21 SOBA Observable Audits | [V.21 SOBA](v21-soba-observable-ballot-audits.md) | Commitments, privacy-preserving ballot linkage, public observability |
| W.01 Election Forensic Analytics | [W.01 Forensics](w01-election-forensic-analytics.md) | Outlier scores, residuals, digit tests, investigative boundary |

## Review Artifacts

| Artifact | Purpose |
|---|---|
| [Rubric](RUBRIC.md) | Defines the six BISECT scoring dimensions |
| [Visual Patterns](VISUAL-PATTERNS.md) | Captures reusable figure rules and anti-patterns |
| [Scorecard](reviews/atlas-scorecard-2026-05-11.md) | Records the current all-golden atlas review |
| [Role Review](reviews/rubric-r1_roles.md) | Preserves the `.roles` review pass for the rubric |

## Possible Future Pages

Future RPLAN atlas pages could cover VRASection, ProportionalSection,
NestSection, StabilitySection, Moving-Knife, compact-Polsby, CVD, BFS growth,
and simulated annealing. Future RCOUNT pages should add implementation-specific
variants only when they have a transcript shape, fixtures, and a claim boundary.

## Relationship To Other Docs

- `docs/concepts/algorithm-family-layer-cake.md` is the crate and evidence
  taxonomy.
- `docs/concepts/t-u-portfolio-dependency-map.md` maps papers, packages, and
  verifier paths.
- `docs/PAPERS.md` is the publication index.
- `docs/examples/rplan-benchmark-packages/` contains committed benchmark-tier
  packages referenced by these pages.
