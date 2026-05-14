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
| T.5 ProportionalSection | 3.8 | 3.7 | 3.5 | 3.9 | 4.0 | 3.6 | 22.5 | Golden |
| T.6 NestSection | 3.7 | 3.6 | 3.3 | 3.9 | 4.0 | 3.7 | 22.2 | Golden; constructor/maps remain future evidence |
| T.7 VRASection | 3.7 | 3.6 | 3.4 | 3.9 | 4.0 | 3.4 | 22.0 | Golden; final-plan VAP/CVAP evidence pending |
| T.8 StabilitySection | 3.7 | 3.6 | 3.2 | 3.8 | 4.0 | 3.5 | 21.8 | Strong; full CSS packages pending |
| T.9 Multi-Reapportionment Stability | 3.7 | 3.7 | 3.1 | 3.9 | 4.0 | 3.5 | 21.9 | Strong; scenario packages pending |
| T.10 Centroidal Voronoi | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.4 | 21.2 | Strong; ratio-aware splits and packages pending |
| T.11 CVD Geographic | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.5 | 21.3 | Strong; benchmark packages and ratio-aware CVD pending |
| T.12 BFS Growth | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.5 | 21.3 | Strong; benchmark packages and ratio-aware splits pending |
| T.13 Moving-Knife | 3.6 | 3.6 | 3.0 | 3.8 | 3.9 | 3.6 | 21.5 | Strong; per-node angle ledgers and exact Reock pending |
| T.14 Spectral Partitioning | 3.7 | 3.5 | 3.5 | 3.8 | 4.0 | 3.6 | 22.1 | Strong; needs richer graph/sweep visual |
| T.15 Capacity-Constrained Clustering | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 | Golden |
| T.16 Hierarchical Regionalization | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 | Golden |
| T.17 Flow-Based Construction | 3.8 | 3.6 | 3.6 | 3.9 | 4.0 | 3.6 | 22.5 | Golden |

## Search, Optimization, And Audit

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| U.0 Search Optimization Overview | 3.5 | 3.4 | 3.2 | 3.6 | 3.6 | 3.4 | 20.7 | Strong; package boundary refreshed |
| U.1 ConvergenceSweep | 3.7 | 3.6 | 3.5 | 3.9 | 4.0 | 3.6 | 22.3 | Strong; search-ledger boundary |
| U.2 Parameter Sensitivity | 3.5 | 3.4 | 3.3 | 3.5 | 3.5 | 3.4 | 20.6 | Stronger; synthetic sweep package, real 50-state sweep pending |
| U.3 Simulated Annealing | 3.3 | 3.4 | 3.1 | 3.4 | 3.4 | 3.2 | 19.8 | Serviceable-plus; heuristic boundary |
| U.4 Parallel Tempering | 3.5 | 3.5 | 3.3 | 3.5 | 3.5 | 3.4 | 20.7 | Stronger; synthetic PT package, production CLI pending |
| U.5 Adaptive Multiscale | 3.4 | 3.4 | 3.2 | 3.5 | 3.4 | 3.3 | 20.2 | Strong; bisect-multiscale boundary |
| U.6 ILP Redistricting | 3.4 | 3.5 | 3.2 | 3.6 | 3.4 | 3.3 | 20.4 | Strong; exact status language aligned |
| U.7 Pareto Redistricting | 3.5 | 3.5 | 3.3 | 3.6 | 3.5 | 3.4 | 20.8 | Strong; selected-frontier artifact aligned |
| U.8 PercentileSweep | 3.7 | 3.5 | 3.5 | 3.9 | 4.0 | 3.5 | 22.1 | Strong |
| U.9 BisectionEnsemble | 3.6 | 3.5 | 3.4 | 3.7 | 4.0 | 3.5 | 21.7 | Strong |
| U.10 bisect-ensemble | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.6 | 22.6 | Golden |
| U.11 Resolution-Aware | 3.4 | 3.3 | 3.2 | 3.5 | 3.4 | 3.3 | 20.1 | Strong; resolution package boundary |
| U.12 Algorithm-Selection Matrix | 3.8 | 3.6 | 3.6 | 3.9 | 4.0 | 3.7 | 22.6 | Golden |
| U.13 Exact-vs-Heuristic Certification | 3.7 | 3.6 | 3.5 | 3.9 | 3.6 | 3.6 | 21.9 | Strong; U.16-U.20 boundary integrated |
| U.14 Multi-Objective Selection | 3.6 | 3.5 | 3.4 | 3.7 | 3.6 | 3.5 | 21.3 | Strong; selected-frontier record aligned |
| U.15 Legal Postures for Search | 3.7 | 3.4 | 3.4 | 3.9 | 3.6 | 3.6 | 21.6 | Strong; artifact-class posture aligned |
| U.16 Branch-and-Cut | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.7 | 22.7 | Golden |
| U.17 Branch-and-Price | 3.8 | 3.8 | 3.6 | 3.9 | 4.0 | 3.7 | 22.8 | Golden |
| U.18 Large-Neighborhood Search | 3.8 | 3.8 | 3.7 | 3.9 | 4.0 | 3.8 | 23.0 | Golden |
| U.19 Evolutionary Search Comparison | 3.8 | 3.7 | 3.7 | 3.9 | 4.0 | 3.7 | 22.8 | Golden |
| U.20 Plan Audit Certificates | 3.9 | 3.7 | 3.8 | 4.0 | 4.0 | 3.7 | 23.1 | Golden; fixed-point paper |

## Ensemble And Sampling

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| G.0 Ensemble Methodology | 3.1 | 3.1 | 3.0 | 3.2 | 3.3 | 3.1 | 18.8 | Needs modern package/diagnostic framing |
| G.1 GerryChain Congressional Comparison | 3.5 | 3.4 | 3.4 | 3.6 | 3.6 | 3.4 | 20.9 | Stronger; active synthetic package plus real-evidence gap |
| G.2 Partisan Outcome Distributions | 3.4 | 3.3 | 3.2 | 3.5 | 3.5 | 3.3 | 20.2 | Stronger; active synthetic package plus real-election gap |
| G.3 Compactness Distribution Position | 3.5 | 3.4 | 3.3 | 3.6 | 3.6 | 3.4 | 20.8 | Stronger; active synthetic package plus real-metric gap |
| G.4 Ensemble Diagnostics | 3.8 | 3.7 | 3.7 | 3.9 | 4.0 | 3.6 | 22.7 | Golden |
| G.5 Convergence Mixing Analysis | 3.4 | 3.4 | 3.2 | 3.6 | 3.4 | 3.3 | 20.3 | Strong; diagnostics scoped correctly |
| G.6 Short-Burst | 3.3 | 3.5 | 3.2 | 3.3 | 3.5 | 3.4 | 20.2 | Strong draft, accepted but evidence could deepen |
| G.7 SMC Redistricting | 3.8 | 3.8 | 3.7 | 3.8 | 4.0 | 3.7 | 22.8 | Golden |
| G.8 Flip Proposals | 3.5 | 3.6 | 3.3 | 3.4 | 3.5 | 3.5 | 20.8 | Strong |
| G.9 Forest ReCom | 3.5 | 3.6 | 3.3 | 3.4 | 3.5 | 3.5 | 20.8 | Strong |
| G.10 Merge-Split | 3.8 | 3.8 | 3.6 | 3.9 | 4.0 | 3.6 | 22.7 | Golden |
| G.11 Multiscale MCMC | 3.8 | 3.7 | 3.5 | 3.9 | 4.0 | 3.6 | 22.5 | Golden |
| G.12 Short-Burst Chains | 3.4 | 3.4 | 3.2 | 3.5 | 3.4 | 3.3 | 20.2 | Strong; short-burst diagnostic boundary |
| G.13 VRA-Aware Ensemble | 3.8 | 3.7 | 3.6 | 3.8 | 3.6 | 3.7 | 22.2 | Strong near-golden |
| G.14 Ensemble Comparison | 3.9 | 3.8 | 3.8 | 3.9 | 3.6 | 3.9 | 22.9 | Golden |
| G.15 Comprehensive Comparison | 3.5 | 3.5 | 3.4 | 3.6 | 3.4 | 3.5 | 20.9 | Strong; index status should be reconciled |

## Apportionment And Metrics

| Paper | P1 | P2 | P3 | P4 | P5 | P6 | Total | Priority |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| J.0 Apportionment Overview | 3.4 | 3.5 | 3.2 | 3.4 | 3.4 | 3.4 | 20.3 | Stronger; 2020 Census/SHA package added |
| J.1 Huntington-Hill | 3.5 | 3.6 | 3.4 | 3.5 | 3.5 | 3.5 | 21.0 | Strong; 2020 Census/SHA package verifies official seats |
| J.2 Webster | 3.3 | 3.4 | 3.1 | 3.3 | 3.3 | 3.3 | 19.7 | Serviceable-plus; current API boundary |
| J.3 Adams | 3.3 | 3.4 | 3.1 | 3.3 | 3.3 | 3.3 | 19.7 | Serviceable-plus; current API boundary |
| J.4 Jefferson/D'Hondt | 3.3 | 3.4 | 3.1 | 3.3 | 3.3 | 3.3 | 19.7 | Serviceable-plus; current API boundary |
| J.5 Apportionment Paradoxes | 3.3 | 3.4 | 3.1 | 3.4 | 3.3 | 3.3 | 19.8 | Serviceable-plus; synthetic-check boundary |
| J.6 bisect-apportion Implementation | 3.5 | 3.6 | 3.4 | 3.6 | 3.6 | 3.5 | 21.2 | Strong; 2020 Census/SHA verifier package added |
| K.0 Compactness Overview | 3.6 | 3.5 | 3.4 | 3.6 | 3.5 | 3.5 | 21.1 | Strong; exact-MBC smoke package added |
| K.1 Polsby-Popper | 3.5 | 3.6 | 3.3 | 3.4 | 3.4 | 3.5 | 20.7 | Strong |
| K.2 Reock | 3.7 | 3.7 | 3.5 | 3.7 | 3.7 | 3.6 | 21.9 | Strong; exact-MBC smoke package and proxy boundary |
| K.3 Convex Hull | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.4 Schwartzberg | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.5 Length-Width | 3.2 | 3.3 | 3.0 | 3.2 | 3.3 | 3.2 | 19.2 | Minor revision |
| K.6 Population-Weighted Compactness | 3.5 | 3.5 | 3.3 | 3.4 | 3.4 | 3.5 | 20.6 | Strong |
| K.7 Composite/Court Guide | 3.5 | 3.5 | 3.3 | 3.6 | 3.5 | 3.5 | 20.9 | Stronger; exact/proxy Reock disclosure package |
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

1. Metric follow-through: K.2/K.0/K.7 now have exact-MBC smoke evidence, but
   real district polygon exact-MBC packages remain future evidence lifts.
2. Package evidence follow-through: U.2/U.4 now have synthetic package evidence,
   but still need real archived sweep and production CLI packages before they can
   score as implementation papers.
3. G-track package follow-through: G.1-G.3 now have active synthetic package
   evidence, but still need archived real external traces and election/metric
   packages before their percentile claims can score as empirical findings.

## Track Summary

| Track family | Included rows | Current center of gravity | Main gap |
|---|---:|---|---|
| B/T construction foundations | 26 | Strong for reviewed/new slices, serviceable for older slices; T.5-T.7 golden and T.8/T.9 strong | legacy visuals and audit/package bridges |
| U search/optimization/audit | 21 | Strong after synthetic U.2/U.4 package and U.10/U.16-U.20 golden slices | real archived sweep/CLI packages for U.2/U.4 |
| G ensemble/sampling | 16 | Strong after active synthetic G.1-G.3 package; G.4/G.7/G.10/G.11 golden | archived real external traces for G.1-G.3 |
| J apportionment | 7 | Serviceable-plus after implementation-boundary pass | Census/SHA fixtures and Hamilton public API remain future work |
| K compactness metrics | 8 | Strong/serviceable-plus after exact-MBC smoke package | real district exact-MBC packages and K.5 minor revision |
| L partisan metrics | 7 | Strong | cross-link to G/A synthesis |
| M community weights | 5 | Strong where PDFs exist | source-only papers need completion |
