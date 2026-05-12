# Atlas-To-Paper Alignment Tracker

**Date opened:** 2026-05-12
**Purpose:** keep the golden Algorithm Atlas, paper PDFs, LaTeX sources, and
RPLAN evidence packages telling the same story.

The atlas now uses a strict teaching contract: show candidate objects, why they
exist, the decision rule, the downstream consequence, the claim boundary, and
the evidence fields. Paper revisions should inherit that contract instead of
drifting back into prose-only algorithm descriptions.

This is a quality-improvement tracker, not peer review. When using the local
`panel` publication process, follow its disclosure rule: simulated reviewer
perspectives are for pre-submission strengthening and are not real peer review
or endorsement.

## Alignment Rubric

Each paper should be checked against six atlas-derived requirements:

| Code | Requirement | Paper evidence |
|---|---|---|
| C1 | Candidate objects are explicit | Candidate cuts, seeds, merges, arcs, columns, moves, offspring, particles, or verifier paths are named before the result |
| C2 | Candidate purpose is explained | The paper says why the candidate set exists: adjacency, ratio scan, Fiedler ordering, capacity, pricing, frontier, proposal seed, etc. |
| C3 | Decision rule is visible | The text or figure shows accepted/rejected/skipped/repaired/resampled/selected/proven behavior |
| C4 | Consequence is carried forward | Recursive workload, sidecar, solve report, merge log, frontier package, NDJSON event, or certificate consequence is named |
| C5 | Claim boundary is explicit | The paper states what the method does not prove |
| C6 | Evidence path is inspectable | RPLAN/RCTX, transcript, report, package, benchmark, or source path is cited |

## Paper Mapping

| Atlas page | Paper/PDF | Source | Evidence package status | Alignment status |
|---|---|---|---|---|
| [GeoSection](../algorithm-atlas/geosection.md) | [T.1](T.1+geosection-ratio-optimal-bisection.pdf) | `research/tracks/T-plan-construction/T.1+geosection-ratio-optimal-bisection/` | Legacy/compositor evidence; needs package-path audit in paper | Aligned 2026-05-12; rebuilt PDF |
| [AreaSection](../algorithm-atlas/areasection.md) | [T.2](T.2+areasection-dual-population-area-constraint.pdf) | `research/tracks/T-plan-construction/T.2+areasection-dual-population-area-constraint/` | Legacy/compositor evidence; needs package-path audit in paper | Aligned 2026-05-12; rebuilt PDF |
| [County-Sticky Weights](../algorithm-atlas/county-sticky-weights.md) | [T.3](T.3+subdivision-respecting-redistricting.pdf) | `research/tracks/T-plan-construction/T.3+subdivision-respecting-redistricting/` | Legacy/compositor evidence; needs package-path audit in paper | Aligned 2026-05-12; rebuilt PDF |
| [ApportionRegions](../algorithm-atlas/apportionregions.md) | [T.4](T.4+apportion-regions.pdf) | `research/tracks/T-plan-construction/T.4+apportion-regions/` | Legacy/compositor evidence; needs package-path audit in paper | Aligned 2026-05-12; rebuilt PDF |
| [Seed Search Modes](../algorithm-atlas/seed-search-modes.md) | [U.1](U.1+convergence-sweep.pdf), [U.8](U.8+percentile-sweep.pdf), [U.9](U.9+bisection-ensemble.pdf) | `research/tracks/U-search-optimization/` | Search-mode evidence split across papers | Queued |
| [Three-Layer Compositor](../algorithm-atlas/three-layer-compositor.md) | [U.12](U.12+algorithm-selection-matrix.pdf) and portfolio docs | `research/tracks/U-search-optimization/U.12+algorithm-selection-matrix/` | Cross-family evidence | Queued |
| [T.14 Spectral Partitioning](../algorithm-atlas/t14-spectral-partitioning.md) | [T.14](T.14+spectral-partitioning.pdf) | `research/tracks/T-plan-construction/T.14+spectral-partitioning/` | Golden, method, and benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [T.15 Capacity Clustering](../algorithm-atlas/t15-capacity-clustering.md) | [T.15](T.15+capacity-constrained-clustering.pdf) | `research/tracks/T-plan-construction/T.15+capacity-constrained-clustering/` | Golden and benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [T.16 Hierarchical Regionalization](../algorithm-atlas/t16-hierarchical-regionalization.md) | [T.16](T.16+hierarchical-regionalization.pdf) | `research/tracks/T-plan-construction/T.16+hierarchical-regionalization/` | Golden and benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [T.17 Flow Construction](../algorithm-atlas/t17-flow-construction.md) | [T.17](T.17+flow-based-construction.pdf) | `research/tracks/T-plan-construction/T.17+flow-based-construction/` | Golden and benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [U.16 Branch-And-Cut](../algorithm-atlas/u16-branch-and-cut.md) | [U.16](U.16+branch-and-cut-redistricting.pdf) | `research/tracks/U-search-optimization/U.16+branch-and-cut-redistricting/` | Golden and path8 benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [U.17 Branch-And-Price](../algorithm-atlas/u17-branch-and-price.md) | [U.17](U.17+branch-and-price-redistricting.pdf) | `research/tracks/U-search-optimization/U.17+branch-and-price-redistricting/` | Golden package exists | Aligned 2026-05-12; rebuilt PDF |
| [U.18 Local Search](../algorithm-atlas/u18-local-search.md) | [U.18](U.18+large-neighborhood-search.pdf) | `research/tracks/U-search-optimization/U.18+large-neighborhood-search/` | Golden, method, and benchmark packages exist | Aligned 2026-05-12; rebuilt PDF |
| [U.19 Evolutionary Comparison](../algorithm-atlas/u19-evolutionary-comparison.md) | [U.19](U.19+evolutionary-search-comparison.pdf) | `research/tracks/U-search-optimization/U.19+evolutionary-search-comparison/` | Selected-frontier golden package exists | Aligned 2026-05-12; rebuilt PDF |
| [U.20 RPLAN Audit Certificates](../algorithm-atlas/u20-rplan-audit-certificates.md) | [U.20](U.20+plan-audit-certificates.pdf) | `research/tracks/U-search-optimization/U.20+plan-audit-certificates/` | Golden corpus and grid10 audit benchmark exist | Aligned 2026-05-12; rebuilt PDF |
| [ReCom Ensemble](../algorithm-atlas/recom-ensemble.md) | [U.10](U.10+bisect-ensemble.pdf), [G.4](G.4+ensemble-diagnostics-paper.pdf), [G.10](G.10+merge-split.pdf) | `research/tracks/U-search-optimization/U.10+bisect-ensemble/`, `research/tracks/G-ensemble/` | Ensemble outputs; package alignment unclear | Aligned 2026-05-12; rebuilt PDF |
| [Sequential Monte Carlo](../algorithm-atlas/sequential-monte-carlo.md) | [G.7](G.7+smc-redistricting.pdf) | `research/tracks/G-ensemble/G.7+smc-redistricting/` | SMC NDJSON/test evidence; package alignment unclear | Aligned 2026-05-12; rebuilt PDF |
| [Multiscale MCMC](../algorithm-atlas/multiscale-mcmc.md) | [G.11](G.11+multiscale-mcmc.pdf) | `research/tracks/G-ensemble/G.11+multiscale-mcmc/` | Multiscale sampler evidence; package alignment unclear | Aligned 2026-05-12; rebuilt PDF |

## Stage Plan

1. **T.14-T.17 construction pass** - completed 2026-05-12
   - Compare each paper's method, implementation, evaluation, audit, and
     limitations sections against the atlas page.
   - Add small paper-language patches only where the atlas exposes a missing
     candidate, decision, consequence, or evidence phrase.
   - Rebuild PDFs if LaTeX sources change.

2. **U.16-U.20 exact/search/audit pass** - completed 2026-05-12
   - Preserve category boundaries: solver proof claims stay in solver reports;
     plan validity claims stay in RPLAN/RCTX/certificates.
   - Make U.20 fixed-point language consistent across all U-series papers.
   - Rebuild PDFs if LaTeX sources change.

3. **Sampling pass** - completed 2026-05-12
   - Align ReCom, SMC, and Multiscale papers with the atlas emphasis on
     proposed transitions, accepted/rejected/copied steps, diagnostics, and
     finite-sample claim boundaries.

4. **Compositor and legacy T-family pass**
   - Bring T.1-T.4, seed search, and the compositor guide up to the same
     candidate-decision-consequence language.
   - T.1-T.4 completed 2026-05-12; seed search and compositor remain queued.

5. **Panel quality-improvement pass**
   - Run or simulate the local paper-level review process only after source
     alignment is complete.
   - Track P1/P2/P3 suggestions as improvement items, not reviewer mandates.

## Acceptance Checklist

- [ ] Every atlas-covered paper has an explicit C1-C6 alignment row.
- [x] Every edited source paper rebuilds cleanly for the completed T.14-T.17 pass.
- [x] Every edited source paper rebuilds cleanly for the completed U.16-U.20 pass.
- [x] Every edited source paper rebuilds cleanly for the completed sampling pass.
- [x] `docs/PAPERS.md` links to this tracker.
- [x] Changed T.14-T.17 PDFs are copied to `docs/papers/`.
- [x] Changed U.16-U.20 PDFs are copied to `docs/papers/`.
- [x] Changed sampling PDFs are copied to `docs/papers/`.
- [x] Every edited source paper rebuilds cleanly for the completed T.1-T.4 legacy pass.
- [x] Changed T.1-T.4 PDFs are copied to `docs/papers/`.
- [ ] Review language follows the panel disclosure boundary.
