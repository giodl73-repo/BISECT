# Algorithm Atlas Scorecard

**Date:** 2026-05-11
**Rubric:** `docs/algorithm-atlas/RUBRIC.md`
**Reviewer:** Codex using the BISECT rubric and `.roles` lenses
**Latest pass:** upgraded mechanics visuals, examples, failure modes, evidence
references, worked ledgers, output snippets, and reading checklists across the
atlas.

This scorecard grades the current atlas pages against the six BISECT dimensions:

- **B** Behavioral Mechanics
- **I** Integration With BISECT
- **S** Spatial And Visual Specificity
- **E** Explanatory Story
- **C** Claim Boundary And Correctness
- **T** Traceability And Evidence

Scores are out of 60. The main scoring pressure is visual and explanatory
depth: a page that has correct prose but does not show the actual algorithmic
object loses points under **S** and usually **E**.

## Summary

| Band | Pages |
|---|---|
| Excellent, 54-60 | U.16, U.18, U.19, U.20 |
| Good, 45-53 | T.14, T.15, T.16, T.17, U.17, ReCom, SMC, Multiscale, Three-Layer Compositor, GeoSection, AreaSection, ApportionRegions, County-Sticky, Seed Search |
| Serviceable, 36-44 | None |
| Thin, 24-35 | None |
| Failing, 0-23 | None |

The atlas is now broadly good by the BISECT rubric: every page has at least one
mechanics visual, an integration story, a claim boundary, evidence anchors, and
a worked ledger or output-reading guide. The remaining path to excellence is
visual polish rather than rescue: improve SVG artistry, add real rendered
package snippets where available, and move the strongest examples into
publication-grade figures.

## Score Matrix

| Page | B | I | S | E | C | T | Total | Band |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| T.14 Spectral Partitioning | 9 | 8 | 9 | 9 | 8 | 8 | 51 | Good |
| T.15 Capacity Clustering | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| T.16 Hierarchical Regionalization | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| T.17 Flow Construction | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| U.16 Branch-And-Cut | 9 | 9 | 9 | 9 | 9 | 9 | 54 | Excellent |
| U.17 Branch-And-Price | 9 | 9 | 9 | 9 | 9 | 8 | 53 | Good |
| U.18 Local Search | 9 | 9 | 9 | 9 | 9 | 9 | 54 | Excellent |
| U.19 Evolutionary Comparison | 9 | 9 | 9 | 9 | 9 | 9 | 54 | Excellent |
| U.20 RPLAN Audit Certificates | 10 | 10 | 9 | 9 | 10 | 10 | 58 | Excellent |
| ReCom Ensemble | 9 | 8 | 9 | 9 | 9 | 9 | 53 | Good |
| Sequential Monte Carlo | 9 | 8 | 9 | 9 | 9 | 9 | 53 | Good |
| Multiscale MCMC | 8 | 8 | 9 | 9 | 10 | 8 | 52 | Good |
| Three-Layer Compositor | 8 | 9 | 8 | 9 | 8 | 8 | 50 | Good |
| GeoSection | 9 | 8 | 9 | 9 | 8 | 8 | 51 | Good |
| AreaSection | 9 | 8 | 10 | 9 | 9 | 8 | 53 | Good |
| ApportionRegions | 9 | 8 | 9 | 9 | 9 | 8 | 52 | Good |
| County-Sticky Weights | 8 | 8 | 9 | 9 | 9 | 8 | 51 | Good |
| Seed Search Modes | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |

## Page Notes

### T.14 Spectral Partitioning - 51/60

**Roles applied:** MERIDIAN, DATUM, COVENANT, BENCHMARK

Strongest element: clear mechanics and visual sequence from Laplacian to
Fiedler ordering to sweep recursion.

Weakest element: traceability does not yet point to focused tests or exact
summary fields.

Next upgrades:

- Add test/fixture references for path, two-clique, determinism, and recursive
  odd/non-power-of-two cases.
- Add one small visual of an actual split package sidecar or spectral summary.

### T.15 Capacity Clustering - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: status taxonomy and capacity-pressure diagrams.

Weakest element: mechanics could show the assignment order and bounded repair
loop more concretely.

Next upgrades:

- Add an assignment-order diagram showing a unit rejected by one near-full
  cluster and accepted by another.
- Reference capacity/infeasibility/determinism tests.

### T.16 Hierarchical Regionalization - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: merge log as an explanation artifact.

Weakest element: scoring rule is named but not visually unpacked.

Next upgrades:

- Add candidate-pair scoring diagram with eligible, rejected, and selected
  adjacent pairs.
- Reference hierarchy/capacity/determinism fixtures.

### T.17 Flow Construction - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, LEDGER

Strongest element: constrained-assignment framing and infeasibility witness.

Weakest element: visual network is still high-level; it should show one
capacity-bound failure with numbers.

Next upgrades:

- Add a tiny numeric flow example with supply totals and capacity bounds.
- Reference flow capacity/infeasibility/determinism tests.

### U.16 Branch-And-Cut - 54/60

**Roles applied:** MERIDIAN, SCALE, DATUM, COVENANT, BENCHMARK

Strongest element: path8 now makes the abstract connectivity cut concrete, and
the page ties model evidence to `node_root.lp` and `ilp-solve-report.json`.

Next upgrades:

- Add a rendered snippet from the ILP report showing status, gap, and cut count.
- Link any future exact benchmark tests once they are split from the package
  fixture itself.

### U.17 Branch-And-Price - 53/60

**Roles applied:** MERIDIAN, DATUM, COVENANT, BENCHMARK

Strongest element: pricing and master-problem roles are now separated visually,
with tests and crate modules named.

Next upgrades:

- Add a tiny table of column coverage and cost.
- Show formulation-only status beside solved tiny enumeration in one output
  snippet.

### U.18 Local Search - 54/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: the before/after move now shows exactly what changes and
what validity gates must remain true.

Next upgrades:

- Add a short output excerpt from `local-search-summary.json`.
- Add an explicit no-op fixture note when no improving valid move exists.

### U.19 Evolutionary Comparison - 54/60

**Roles applied:** SCALE, DATUM, COVENANT, BENCHMARK

Strongest element: the selected-frontier story now includes repair-aware
crossover/fallback, so validity is not treated as hidden cleanup.

Next upgrades:

- Add a small frontier table showing rank, objectives, and selected index.
- Add a longer uncertainty note if future evolutionary runs add stochastic
  convergence diagnostics.

### U.20 RPLAN Audit Certificates - 58/60

**Roles applied:** COVENANT, LEDGER, BOUNDARY, BENCHMARK

Strongest element: this is now the clearest fixed-point page. It explains both
acceptance and rejection, and it points to positive and negative fixtures.

Next upgrades:

- Add direct schema/version references if the RPLAN schema docs move into a
  stable user-facing location.

### ReCom Ensemble - 53/60

**Roles applied:** SCALE, MERIDIAN, DATUM

Strongest element: one-step mechanics and multi-chain diagnostics now both
appear, which separates sampling behavior from final-plan audit.

Next upgrades:

- Add a real diagnostics output excerpt once the atlas chooses a canonical
  ensemble run.

### Sequential Monte Carlo - 53/60

**Roles applied:** SCALE, DATUM, BENCHMARK, LEDGER

Strongest element: proposal, weighting, ESS, and resampling genealogy are now
visible as a single story.

Next upgrades:

- Add a compact NDJSON example line for metadata and one particle event.

### Multiscale MCMC - 52/60

**Roles applied:** SCALE, MERIDIAN, DATUM, TRENCH

Strongest element: the page is appropriately cautious and now shows the
coarse-to-fine projection/rebalance handoff.

Next upgrades:

- Add exact config field names once the public multiscale config surface
  stabilizes.

### Three-Layer Compositor - 50/60

**Roles applied:** SURVEY, MERIDIAN, DATUM

Strongest element: the page now has a complete example run that keeps
structure, weights, and search independently visible.

Next upgrades:

- Add an actual CLI/config excerpt once the compositor command surface settles.
- Add a manifest snippet showing config hash and sidecars.

### GeoSection - 51/60

**Roles applied:** MERIDIAN, DATUM, SCALE

Strongest element: the caterpillar failure and normalized top-level split are
now visually legible.

Next upgrades:

- Add a small ratio-scan output table with seed budget and selected split.

### AreaSection - 53/60

**Roles applied:** MERIDIAN, CONTOUR, DATUM, SCALE

Strongest element: the page now shows why equal population and land-area balance
are distinct constraints, with Lorenz feasibility called out.

Next upgrades:

- Add an example ALAND/population row and the computed area share.

### ApportionRegions - 52/60

**Roles applied:** MERIDIAN, DATUM, SCALE

Strongest element: prime fallback and geographic reuse are now explicit rather
than implied by the factor tree.

Next upgrades:

- Add a tiny region-spine fixture table if the compositor gains one.

### County-Sticky Weights - 51/60

**Roles applied:** MERIDIAN, WARD, BOUNDARY, DATUM

Strongest element: the numeric edge-weight example and state-law caveat make the
weights-layer boundary much clearer.

Next upgrades:

- Add a real county-split summary example when package sidecars expose one.

### Seed Search Modes - 50/60

**Roles applied:** MERIDIAN, SCALE, DATUM, SURVEY

Strongest element: the seed stream and percentile diagrams now show why search
mode changes the question being answered.

Next upgrades:

- Add a tiny candidate list with seed, score, and percentile selection.

## Upgrade Priority

1. **Visual polish**: replace the simplest schematic SVGs with richer
   publication-grade figures where the algorithm has spatial structure.
2. **Rendered package snippets**: use real sidecar excerpts for U.16, U.17,
   U.18, ReCom, SMC, and the compositor when canonical package outputs settle.
3. **Config stabilization**: add exact public config fields for the compositor
   and multiscale pages once those surfaces settle.
4. **Canonical fixture tables**: add tiny data rows for AreaSection,
   ApportionRegions, County-Sticky, and Seed Search.

## Acceptance Target

The atlas now meets the minimum maturity target: every page scores at least
**50/60** after the latest pass. The next target is to raise the older T pages
and compositor family toward **54/60 or higher** with output snippets and
canonical fixture tables:

- T.14 Spectral Partitioning
- T.15 Capacity Clustering
- T.16 Hierarchical Regionalization
- T.17 Flow Construction
- U.20 RPLAN Audit Certificates
- Three-Layer Compositor
- GeoSection
- AreaSection
- ApportionRegions
