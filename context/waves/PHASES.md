# R Package Wave Index

> Find the first wave with `status: active`; that is where execution work goes.
> Waves define a coherent mission. Pulses are the smallest executable slices.

## Waves

| Date | Wave | Phase | Status |
|---|---|---|---|
| 2026-01-12 | Bootstrap Recursive Bisection | Initial Python pipeline, dashboard, docs, enhancement workflow | archived |
| 2026-01-14 | Cross-Census Dashboard | Multi-year outputs, artifact naming, edge-weighted support, figures | archived |
| 2026-01-17 | Pipeline Manager | Config system, enhancement manager, wave/API migration, pipeline orchestration | archived |
| 2026-02-06 | Wave Ledger Migration | Context wave V2/V4 migration and pulse mapping | archived |
| 2026-02-07 | Research Portfolio Genesis | First paper portfolio, panel review process, track reorganization | archived |
| 2026-04-23 | Public Dashboard Release | GitHub Pages dashboard, public repo cleanup, release docs | archived |
| 2026-04-24 | Rust Port And RPLAN Specs | Rust workspace, core crates, RPLAN/spec suite, VRA correction | archived |
| 2026-04-27 | Release Hardening And TUI | Scenario audit fixes, TUI, CI/release surfaces, international runs | archived |
| 2026-05-01 | B Foundations Expansion | B.7-B.15 algorithm papers, GeoSection/AreaSection/ApportionRegions work | archived |
| 2026-05-02 | Pure Rust METIS | `redist-metis` design, implementation, verification, shadow validation | archived |
| 2026-05-07 | Research Track Expansion | Tracks I-M and N-S, community character, board reviews, full paper program | archived |
| 2026-05-10 | RPLAN/U20 Audit Packages | RPLAN golden packages, U20 audit certificates, provenance and sidecars | archived |
| 2026-05-11 | Algorithm Atlas And T/U Frontier | Atlas pages, T/U crates, package frontier, paper alignment | archived |
| 2026-05-12 | RCOUNT Substrate And V Track | RCOUNT crate family, V.0-V.10 papers, adapters, audit fixtures | archived |
| 2026-05-13 | R Package Completion | RCOUNT audit algorithms, then RCTX/RHIST base dimensions | archived |
| 2026-05-13 | Paper Rubric Uplift | Raised below-rubric papers across K/J/U/G tracks with claim-boundary passes | archived |
| 2026-05-13 | G Ensemble Evidence Packages | Build auditable external ensemble/election/metric packages for G.1-G.3 claims | archived |
| 2026-05-13 | J Apportionment Evidence Packages | Add Census/SHA fixtures and verifier coverage for J-track apportionment claims | archived |
| 2026-05-13 | K Exact Reock Evidence Packages | Add exact polygon-MBC Reock fixture and package evidence for K-track papers | archived |
| 2026-05-13 | U Search Evidence Packages | Add package-backed U.2 sweep and U.4 parallel-tempering evidence boundaries | archived |
| 2026-05-13 | G Active Ensemble Evidence Packages | Add active synthetic external trace, election, metric, plan/context evidence for G.1-G.3 | archived |
| 2026-05-13 | M Community Source Completion | Promote M.1 and M.3 from source-only to PDF-backed paper entries | archived |
| 2026-05-13 | K.5 Length-Width Uplift | Add AABB diagnostic test and align K.5 implementation boundary | archived |
| 2026-05-13 | G.0 Methodology Package Framing | Modernize G.0 with package/diagnostic evidence boundaries | archived |
| 2026-05-13 | J Divisor Method Evidence | Add Webster/Adams/Jefferson divisor smoke package and paradox verifier coverage | archived |
| 2026-05-13 | M.3 ACS Housing Evidence | Add housing-character weight mode, ACS formula fixture, and M.3 evidence boundary | archived |
| 2026-05-13 | M.1 LODES Economic Evidence | Add economic-character formula fixture and M.1 evidence boundary | archived |
| 2026-05-13 | U.3 Simulated Annealing Evidence | Add deterministic SA seed/grid smoke fixture and U.3 evidence boundary | archived |
| 2026-05-13 | G Short-Burst Evidence | Add endpoint-retention, seed-stream, and diagnostic smoke fixture for G.6/G.12 | archived |
| 2026-05-13 | U.11 Resolution Evidence | Add GEOID mapping, population aggregation, and coarse-adjacency smoke fixture | archived |
| 2026-05-13 | U.5 Adaptive Multiscale Evidence | Add Robbins-Monro alpha-trace and MSC_STEP seed smoke fixture | archived |
| 2026-05-13 | Shared Kernel Crates | Build reusable pure-Rust graph/stat/math kernels for BISECT, ROUTE, and civic evidence projects | archived |
| 2026-05-14 | RSTAT Core Kernel | Extract reusable deterministic statistics from BISECT analysis into `rstat-core` | archived |
| 2026-05-14 | RGRAPH Core Expansion | Add reusable graph-only connected components with BISECT contiguity consumer coverage | archived |
| 2026-05-14 | RMATH Core Matrix Kernel | Extract deterministic dense matrix helpers from bloc-voting WLS/HC3 math | archived |
| 2026-05-14 | RMATH Core Vector Kernel | Extract deterministic vector helpers from spectral and Fiedler math | archived |
| 2026-05-14 | RMATH Core Eigen Kernel | Extract deterministic symmetric 2x2 eigensystem from GeoSection orientation | archived |
| 2026-05-14 | ROPT Core Pareto Kernel | Extract generic Pareto dominance and crowding utilities from NSGA-II | archived |
| 2026-05-14 | ROPT Core Seed Kernel | Extract deterministic domain-separated seed derivation from NSGA-II | archived |
| 2026-05-14 | ROPT Seed Consumer Expansion | Route SMC, multiscale, ensemble, and parallel-tempering seeds through `ropt-core` | archived |
| 2026-05-14 | RGRAPH Edge Cut Kernel | Extract reusable undirected edge-cut counting from local search, Pareto, and PT | archived |
| 2026-05-14 | RGRAPH Edge Cut Consumer Expansion | Route remaining slice-based edge-cut consumers through `rgraph-core` | archived |
| 2026-05-14 | RGRAPH Label Connectivity Kernel | Extract reusable assignment-label connectivity checks from local search, clustering, flow, and ILP | archived |
| 2026-05-14 | RGRAPH Edge Cut Adapter Kernel | Add closure-based edge-cut adapter for map/set-shaped assignments | archived |
| 2026-05-14 | RGRAPH Subset Connectivity Kernel | Extract reusable node-subset connectivity checks from pricing, SMC, Pareto, and CLI evidence helpers | archived |
| 2026-05-14 | RGRAPH Mask Edge Cut Consumer | Route bitmask edge-cut counting through `rgraph-core` adapter without adding new API | archived |
| 2026-05-14 | CLI Weighted Cut Scoring Cleanup | Consolidate repeated weighted edge-cut scoring locally without promoting it to `rgraph-core` | archived |
| 2026-06-01 | VTRACE Baseline Maintenance | First live VTRACE-governed maintenance wave after S6 internal baseline selection | active |

## Operating Model

Each active wave keeps:

- `WAVE.md`: mission, scope, pulse table, gates, and carry-forwards.
- `pulses/NN+slug.md`: one executable slice with frontmatter, Q-decisions,
  deliverables, and validation.
- `forks/pulse-NN.md`: materialized pulse context for agent execution.
- `panels/`: role reviews and consolidated findings.
- `CLOSE.md`: written when the wave is complete.

## Pulse Rules

- A pulse must be independently testable.
- Every pulse names governing roles and validation commands.
- VTRACE-governed pulses name parent `REQ-*`, `SPEC-*`, `IF-*`, `PKG-*`,
  `DES-*`, applicable `CR-*`, validation level, and affected package
  boundaries either in pulse frontmatter/body or in the materialized fork.
- Boundary claims must be explicit: replay, preservation, analytic report, or
  future work.
- Closure must distinguish pass, fail, blocked, partial, deferred, and
  not-applicable states; "implemented" alone is not sufficient.
- Public-claim or generated-artifact effects must carry claim and custody
  disposition before pulse or wave close.
- Closure notes must record risk and pitfall/invariant disposition when the
  pulse changes public evidence, pipeline behavior, package schemas, or
  workflow controls.
- Completed pulses keep their checkboxes checked; future pulses remain open.
- Wave status only advances after docs and tests agree.
