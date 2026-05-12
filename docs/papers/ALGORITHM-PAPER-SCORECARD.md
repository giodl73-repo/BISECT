# Algorithm Paper Scorecard

**Scoring date:** 2026-05-12

This is a triage scorecard for papers that present an algorithm, algorithmic
family, optimization/search method, metric algorithm, or implementation
contract. It uses the same six paper-quality gates as
`PAPER-QUALITY-REVIEW.md`, but most rows below are provisional: they have not
all received the full read/patch/rebuild loop.

## Scope

Included tracks:

- B: algorithm foundations and bisection-method papers
- T: plan construction algorithms
- U: search, optimization, selection, and audit algorithms
- G: ensemble, MCMC, SMC, and ReCom-family algorithms
- J: apportionment algorithms
- K: compactness metric algorithms
- L: partisan metric algorithms
- M: community-character edge-weight algorithms with indexed papers

Excluded from this pass: C/D/E/F/I/A papers whose primary role is validation,
legal application, adoption, policy, or portfolio synthesis rather than an
algorithm or metric definition. Some of those papers depend on algorithms, but
they should be scored in their own empirical/legal/synthesis passes.

## Rubric

| Gate | Meaning |
|---|---|
| P1 | Claim discipline |
| P2 | Algorithm clarity |
| P3 | Evidence path |
| P4 | Legal/statistical/audit boundary |
| P5 | Build and artifact hygiene |
| P6 | Reader experience |

Score bands:

| Band | Total | Meaning |
|---|---:|---|
| Golden | 22.5-24.0 | Strong enough to use as a pattern for the portfolio |
| Strong | 20.5-22.4 | Solid, with limited targeted gaps |
| Serviceable | 18.0-20.4 | Usable draft; needs examples, evidence, or claim tightening |
| Needs Work | 15.0-17.9 | Important but requires a real revision pass |
| Blocked | <15.0 | Known correctness/build/evidence blocker or source-only gap |

## Construction And Bisection

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| B.0 Algorithm Design Overview | 3.9 | 3.8 | 3.8 | 3.9 | 4.0 | 3.8 | 23.2 | Golden; accepted pattern |
| B.1 Recursive Bisection | 3.4 | 3.6 | 3.2 | 3.3 | 3.5 | 3.4 | 20.4 | Strong; add modern audit/package bridge |
| B.2 Edge-Weighted Bisection | 3.3 | 3.5 | 3.1 | 3.2 | 3.5 | 3.3 | 19.9 | Serviceable; tighten evidence and implementation parity |
| B.3 Multi-vs-Edge | 3.3 | 3.4 | 3.0 | 3.2 | 3.5 | 3.2 | 19.6 | Serviceable; formalize constraint-conflict mechanism |
| B.4 Adaptive Bisection | 3.1 | 3.2 | 2.9 | 3.0 | 3.4 | 3.0 | 18.6 | Serviceable; theorem/local-search claims need care |
| B.5 N-Way vs Recursive | 3.2 | 3.3 | 3.0 | 3.1 | 3.4 | 3.1 | 19.1 | Serviceable; clarify comparison protocol |
| B.6 Computational Complexity | 3.0 | 3.2 | 2.9 | 3.1 | 3.3 | 3.0 | 18.5 | Serviceable; approximation/runtime claims need audit |
| B.7 Seed Sensitivity | 3.2 | 3.1 | 3.0 | 3.0 | 3.4 | 3.2 | 18.9 | Serviceable; strengthen reproducibility path |
| T.1 GeoSection | 3.8 | 3.9 | 3.6 | 3.8 | 4.0 | 3.9 | 23.0 | Golden; visual standard bearer |
| T.2 AreaSection | 3.7 | 3.8 | 3.5 | 3.7 | 4.0 | 3.8 | 22.5 | Golden; keep evidence caveats visible |
| T.3 County-Sticky Weights | 3.7 | 3.7 | 3.5 | 3.8 | 4.0 | 3.7 | 22.4 | Strong; near-golden |
| T.4 ApportionRegions | 3.8 | 3.8 | 3.6 | 3.8 | 4.0 | 3.8 | 22.8 | Golden |
| T.5 ProportionalSection | 3.0 | 3.1 | 2.8 | 3.0 | 3.3 | 3.0 | 18.2 | Needs algorithm/evidence refresh |
| T.6 NestSection | 3.1 | 3.2 | 2.9 | 3.1 | 3.3 | 3.0 | 18.6 | Needs nesting examples and verifier story |
| T.7 VRASection | 3.0 | 3.1 | 2.8 | 2.9 | 3.3 | 3.0 | 18.1 | Legal boundary pass needed |
| T.8 StabilitySection | 3.1 | 3.0 | 3.0 | 3.0 | 3.3 | 3.0 | 18.4 | Needs clearer algorithm-vs-analysis scope |
| T.9 Multi-Reapportionment Stability | 3.1 | 3.0 | 2.9 | 3.0 | 3.3 | 3.0 | 18.3 | Needs examples and current data anchors |
| T.10 Centroidal Voronoi | 3.0 | 3.2 | 2.8 | 3.0 | 3.3 | 3.0 | 18.3 | Algorithm explanation pass needed |
| T.11 CVD Geographic | 3.0 | 3.1 | 2.8 | 3.0 | 3.3 | 3.0 | 18.2 | Needs visuals and implementation parity |
| T.12 BFS Growth | 3.0 | 3.2 | 2.8 | 3.0 | 3.3 | 3.0 | 18.3 | Needs worked growth/rejection ledger |
| T.13 Moving-Knife | 3.0 | 3.1 | 2.8 | 3.0 | 3.3 | 3.1 | 18.3 | Needs geometric visual explanation |
| T.14 Spectral Partitioning | 3.7 | 3.5 | 3.5 | 3.8 | 4.0 | 3.6 | 22.1 | Strong; needs richer graph/sweep visual |
| T.15 Capacity-Constrained Clustering | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 | Golden |
| T.16 Hierarchical Regionalization | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 | Golden |
| T.17 Flow-Based Construction | 3.8 | 3.6 | 3.6 | 3.9 | 4.0 | 3.6 | 22.5 | Golden |

## Search, Optimization, And Audit

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| U.0 Search Optimization Overview | 3.3 | 3.2 | 3.0 | 3.4 | 3.5 | 3.2 | 19.6 | Needs architecture refresh |
| U.1 ConvergenceSweep | 3.7 | 3.5 | 3.5 | 3.8 | 4.0 | 3.5 | 22.0 | Strong |
| U.2 Parameter Sensitivity | 3.0 | 2.9 | 2.9 | 3.0 | 3.3 | 3.0 | 18.1 | Needs current evidence protocol |
| U.3 Simulated Annealing | 3.0 | 3.2 | 2.8 | 3.0 | 3.3 | 3.0 | 18.3 | Needs modern search/status framing |
| U.4 Parallel Tempering | 3.0 | 3.1 | 2.8 | 3.0 | 3.3 | 3.0 | 18.2 | Needs chain diagnostics clarity |
| U.5 Adaptive Multiscale | 3.0 | 3.1 | 2.8 | 3.0 | 3.3 | 3.0 | 18.2 | Needs implementation/evidence pass |
| U.6 ILP Redistricting | 3.0 | 3.2 | 2.9 | 3.0 | 3.3 | 3.0 | 18.4 | Superseded by U.16; align proof language |
| U.7 Pareto Redistricting | 3.1 | 3.2 | 3.0 | 3.1 | 3.4 | 3.1 | 18.9 | Align with U.19 selected-frontier artifacts |
| U.8 PercentileSweep | 3.7 | 3.5 | 3.5 | 3.9 | 4.0 | 3.5 | 22.1 | Strong |
| U.9 BisectionEnsemble | 3.6 | 3.5 | 3.4 | 3.7 | 4.0 | 3.5 | 21.7 | Strong |
| U.10 bisect-ensemble | 3.2 | 3.4 | 3.1 | 3.2 | 3.5 | 3.2 | 19.6 | Needs package/transcript bridge |
| U.11 Resolution-Aware | 3.1 | 3.0 | 3.0 | 3.1 | 3.3 | 3.0 | 18.5 | Needs algorithm-selection framing |
| U.12 Algorithm-Selection Matrix | 3.8 | 3.6 | 3.6 | 3.9 | 4.0 | 3.7 | 22.6 | Golden |
| U.13 Exact-vs-Heuristic Certification | 3.6 | 3.4 | 3.3 | 3.8 | 3.5 | 3.4 | 21.0 | Strong; update after U.16-U.20 |
| U.14 Multi-Objective Selection | 3.4 | 3.3 | 3.2 | 3.5 | 3.5 | 3.3 | 20.2 | Serviceable; needs selected-plan examples |
| U.15 Legal Postures for Search | 3.5 | 3.1 | 3.1 | 3.8 | 3.5 | 3.3 | 20.3 | Serviceable; legal posture examples needed |
| U.16 Branch-and-Cut | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.7 | 22.7 | Golden |
| U.17 Branch-and-Price | 3.8 | 3.8 | 3.6 | 3.9 | 4.0 | 3.7 | 22.8 | Golden |
| U.18 Large-Neighborhood Search | 3.8 | 3.8 | 3.7 | 3.9 | 4.0 | 3.8 | 23.0 | Golden |
| U.19 Evolutionary Search Comparison | 3.8 | 3.7 | 3.7 | 3.9 | 4.0 | 3.7 | 22.8 | Golden |
| U.20 Plan Audit Certificates | 3.9 | 3.7 | 3.8 | 4.0 | 4.0 | 3.7 | 23.1 | Golden; fixed-point paper |

## Ensemble And Sampling

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| G.0 Ensemble Methodology | 3.1 | 3.1 | 3.0 | 3.2 | 3.3 | 3.1 | 18.8 | Needs modern package/diagnostic framing |
| G.1 GerryChain Congressional Comparison | 3.0 | 2.9 | 3.0 | 3.0 | 3.3 | 3.0 | 18.2 | Evidence/data validation pass |
| G.2 Partisan Outcome Distributions | 2.6 | 2.8 | 2.6 | 2.8 | 3.2 | 2.8 | 16.8 | Known data-validation issues |
| G.3 Compactness Distribution Position | 3.0 | 2.9 | 2.9 | 3.0 | 3.3 | 3.0 | 18.1 | Needs current ensemble anchors |
| G.4 Ensemble Diagnostics | 3.1 | 3.3 | 3.1 | 3.2 | 3.3 | 3.2 | 19.2 | Batch 4 target |
| G.5 Convergence Mixing Analysis | 3.0 | 3.1 | 2.9 | 3.1 | 3.3 | 3.0 | 18.4 | Needs diagnostic evidence pass |
| G.6 Short-Burst | 3.3 | 3.5 | 3.2 | 3.3 | 3.5 | 3.4 | 20.2 | Strong draft, accepted but evidence could deepen |
| G.7 SMC Redistricting | 3.2 | 3.5 | 3.2 | 3.3 | 3.5 | 3.3 | 20.0 | Batch 4 target; crate implemented |
| G.8 Flip Proposals | 3.5 | 3.6 | 3.3 | 3.4 | 3.5 | 3.5 | 20.8 | Strong |
| G.9 Forest ReCom | 3.5 | 3.6 | 3.3 | 3.4 | 3.5 | 3.5 | 20.8 | Strong |
| G.10 Merge-Split | 3.4 | 3.5 | 3.2 | 3.4 | 3.5 | 3.4 | 20.4 | Batch 4 target |
| G.11 Multiscale MCMC | 3.3 | 3.5 | 3.1 | 3.3 | 3.5 | 3.3 | 20.0 | Batch 4 target |
| G.12 Short-Burst Chains | 3.2 | 3.3 | 3.0 | 3.2 | 3.2 | 3.1 | 19.0 | Spec/paper alignment needed |
| G.13 VRA-Aware Ensemble | 3.8 | 3.7 | 3.6 | 3.8 | 3.6 | 3.7 | 22.2 | Strong near-golden |
| G.14 Ensemble Comparison | 3.9 | 3.8 | 3.8 | 3.9 | 3.6 | 3.9 | 22.9 | Golden |
| G.15 Comprehensive Comparison | 3.5 | 3.5 | 3.4 | 3.6 | 3.4 | 3.5 | 20.9 | Strong; index status should be reconciled |

## Apportionment And Metrics

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| J.0 Apportionment Overview | 3.2 | 3.3 | 3.0 | 3.2 | 3.3 | 3.2 | 19.2 | Minor-revision family pass |
| J.1 Huntington-Hill | 3.3 | 3.5 | 3.1 | 3.3 | 3.3 | 3.3 | 19.8 | Needs verified implementation bridge |
| J.2 Webster | 3.2 | 3.4 | 3.0 | 3.2 | 3.3 | 3.2 | 19.3 | Serviceable |
| J.3 Adams | 3.2 | 3.4 | 3.0 | 3.2 | 3.3 | 3.2 | 19.3 | Serviceable |
| J.4 Jefferson/D'Hondt | 3.2 | 3.4 | 3.0 | 3.2 | 3.3 | 3.2 | 19.3 | Serviceable |
| J.5 Apportionment Paradoxes | 3.2 | 3.3 | 3.0 | 3.3 | 3.3 | 3.2 | 19.3 | Serviceable |
| J.6 bisect-apportion Implementation | 2.6 | 3.0 | 2.6 | 2.8 | 3.2 | 2.8 | 17.0 | Needs Work; known stale implementation claims |
| K.0 Compactness Overview | 3.4 | 3.4 | 3.2 | 3.3 | 3.4 | 3.4 | 20.1 | Strong but inherits K.2 method caveat |
| K.1 Polsby-Popper | 3.5 | 3.6 | 3.3 | 3.4 | 3.4 | 3.5 | 20.7 | Strong |
| K.2 Reock | 2.8 | 3.1 | 2.6 | 3.0 | 3.3 | 3.0 | 17.8 | Needs Work; known Welzl/implementation mismatch |
| K.3 Convex Hull | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.4 Schwartzberg | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.5 Length-Width | 3.2 | 3.3 | 3.0 | 3.2 | 3.3 | 3.2 | 19.2 | Minor revision |
| K.6 Population-Weighted Compactness | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.7 Composite/Court Guide | 3.2 | 3.3 | 3.0 | 3.3 | 3.3 | 3.2 | 19.3 | Minor revision; K.2 caveat propagates |
| L.0 Partisan Fairness Overview | 3.6 | 3.4 | 3.4 | 3.8 | 3.4 | 3.5 | 21.1 | Strong |
| L.1 Efficiency Gap | 3.7 | 3.5 | 3.4 | 3.8 | 3.4 | 3.6 | 21.4 | Strong |
| L.2 Mean-Median | 3.7 | 3.5 | 3.4 | 3.8 | 3.4 | 3.6 | 21.4 | Strong |
| L.3 Partisan Bias | 3.7 | 3.5 | 3.4 | 3.8 | 3.4 | 3.6 | 21.4 | Strong |
| L.4 Declination | 3.6 | 3.5 | 3.4 | 3.8 | 3.4 | 3.5 | 21.2 | Strong |
| L.5 Seats-Votes | 3.7 | 3.5 | 3.4 | 3.8 | 3.4 | 3.6 | 21.4 | Strong |
| L.6 Proportionality vs Majoritarianism | 3.7 | 3.4 | 3.4 | 3.8 | 3.4 | 3.6 | 21.3 | Strong |
| M.0 Community Character Overview | 3.5 | 3.4 | 3.3 | 3.6 | 3.4 | 3.5 | 20.7 | Strong |
| M.1 Economic Character via LODES | n/a | n/a | n/a | n/a | n/a | n/a | Source-only | Post-write pending |
| M.3 Housing Character via ACS | n/a | n/a | n/a | n/a | n/a | n/a | Source-only | First draft/source gap |
| M.6 Administrative Zone Membership | 3.5 | 3.5 | 3.3 | 3.6 | 3.4 | 3.5 | 20.8 | Strong |
| M.9 Economic Character Weights | 3.5 | 3.5 | 3.3 | 3.6 | 3.4 | 3.5 | 20.8 | Strong |

## Highest-Priority Lifts

1. Batch 4: G.4, G.7, G.10, and G.11. These are the ensemble/sampling papers
   most directly tied to implemented crates and current atlas concepts.
2. Legacy construction pass: T.5-T.13. These need the same "show the split,
   show the candidate, show the rejection" standard that made T.1 and U.18 much
   clearer.
3. Metric correctness pass: K.2 first, then K.0/K.7 propagation. The Reock
   implementation mismatch is the most concrete known blocker in the metric
   algorithm papers.
4. Apportionment implementation pass: J.6, then J.0-J.5. The mathematical
   method papers are serviceable, but the implementation paper has known stale
   claims.

## Track Summary

| Track family | Included rows | Current center of gravity | Main gap |
|---|---:|---|---|
| B/T construction foundations | 26 | Strong for reviewed/new slices, serviceable for older slices | legacy visuals and audit/package bridges |
| U search/optimization/audit | 21 | Strong, with U.16-U.20 golden | refresh older U.2-U.7 and U.13-U.15 |
| G ensemble/sampling | 16 | Strong but uneven | diagnostics/package evidence and data-validation cleanup |
| J apportionment | 7 | Serviceable | J.6 stale implementation claims |
| K compactness metrics | 8 | Strong except K.2 | Reock implementation mismatch |
| L partisan metrics | 7 | Strong | cross-link to G/A synthesis |
| M community weights | 5 | Strong where PDFs exist | source-only papers need completion |
