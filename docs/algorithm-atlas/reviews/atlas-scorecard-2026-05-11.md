# Algorithm Atlas Scorecard

**Date:** 2026-05-11
**Rubric:** `docs/algorithm-atlas/RUBRIC.md`
**Reviewer:** Codex using the BISECT rubric and `.roles` lenses

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
| Excellent, 54-60 | None yet |
| Good, 45-53 | T.14, T.15, T.16, T.17, U.18, U.20, ReCom |
| Serviceable, 36-44 | U.16, U.17, U.19, SMC, Multiscale, Three-Layer Compositor, GeoSection, AreaSection, ApportionRegions, County-Sticky, Seed Search |
| Thin, 24-35 | None, but County-Sticky and Seed Search are close |
| Failing, 0-23 | None |

The atlas is no longer a stub. The next improvement pass should aim to move
the newest Bisection Compositor pages from serviceable to good by adding second
diagrams, examples, output evidence, tests/fixtures, and stronger BISECT
operational details.

## Score Matrix

| Page | B | I | S | E | C | T | Total | Band |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| T.14 Spectral Partitioning | 9 | 8 | 9 | 9 | 8 | 8 | 51 | Good |
| T.15 Capacity Clustering | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| T.16 Hierarchical Regionalization | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| T.17 Flow Construction | 8 | 8 | 9 | 9 | 8 | 8 | 50 | Good |
| U.16 Branch-And-Cut | 8 | 8 | 7 | 7 | 8 | 8 | 46 | Good |
| U.17 Branch-And-Price | 8 | 8 | 7 | 7 | 8 | 7 | 45 | Good |
| U.18 Local Search | 8 | 9 | 8 | 8 | 8 | 8 | 49 | Good |
| U.19 Evolutionary Comparison | 8 | 8 | 7 | 7 | 8 | 7 | 45 | Good |
| U.20 RPLAN Audit Certificates | 9 | 9 | 8 | 8 | 9 | 8 | 51 | Good |
| ReCom Ensemble | 8 | 7 | 8 | 8 | 8 | 7 | 46 | Good |
| Sequential Monte Carlo | 8 | 7 | 8 | 8 | 8 | 8 | 47 | Good |
| Multiscale MCMC | 7 | 7 | 7 | 7 | 9 | 7 | 44 | Serviceable |
| Three-Layer Compositor | 6 | 8 | 5 | 6 | 7 | 6 | 38 | Serviceable |
| GeoSection | 7 | 7 | 5 | 6 | 7 | 5 | 37 | Serviceable |
| AreaSection | 7 | 7 | 5 | 6 | 8 | 5 | 38 | Serviceable |
| ApportionRegions | 7 | 7 | 6 | 6 | 8 | 6 | 40 | Serviceable |
| County-Sticky Weights | 6 | 7 | 5 | 6 | 7 | 5 | 36 | Serviceable |
| Seed Search Modes | 6 | 7 | 5 | 6 | 7 | 5 | 36 | Serviceable |

## Page Notes

### T.14 Spectral Partitioning - 51/60

**Roles applied:** MERIDIAN, DATUM, COVENANT, BENCHMARK

Strongest element: clear mechanics and visual sequence from Laplacian to
Fiedler ordering to sweep recursion.

Weakest element: traceability does not yet point to focused tests or exact
summary fields.

Required upgrades:

- Add test/fixture references for path, two-clique, determinism, and recursive
  odd/non-power-of-two cases.
- Add one small visual of an actual split package sidecar or spectral summary.

### T.15 Capacity Clustering - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: status taxonomy and capacity-pressure diagrams.

Weakest element: mechanics could show the assignment order and bounded repair
loop more concretely.

Required upgrades:

- Add an assignment-order diagram showing a unit rejected by one near-full
  cluster and accepted by another.
- Reference capacity/infeasibility/determinism tests.

### T.16 Hierarchical Regionalization - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: merge log as an explanation artifact.

Weakest element: scoring rule is named but not visually unpacked.

Required upgrades:

- Add candidate-pair scoring diagram with eligible, rejected, and selected
  adjacent pairs.
- Reference hierarchy/capacity/determinism fixtures.

### T.17 Flow Construction - 50/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, LEDGER

Strongest element: constrained-assignment framing and infeasibility witness.

Weakest element: visual network is still high-level; it should show one
capacity-bound failure with numbers.

Required upgrades:

- Add a tiny numeric flow example with supply totals and capacity bounds.
- Reference flow capacity/infeasibility/determinism tests.

### U.16 Branch-And-Cut - 46/60

**Roles applied:** MERIDIAN, SCALE, DATUM, COVENANT, BENCHMARK

Strongest element: status and bound/gap language is appropriately careful.

Weakest element: only one mechanics picture after the hero; no tiny ILP fixture
walkthrough.

Required upgrades:

- Add a path8 example showing candidate disconnected solution, added cut, and
  solved status.
- Reference `node_root.lp`, `ilp-solve-report.json`, and exact benchmark tests.

### U.17 Branch-And-Price - 45/60

**Roles applied:** MERIDIAN, DATUM, COVENANT, BENCHMARK

Strongest element: column/master distinction is clear.

Weakest element: pricing is underspecified visually; the page shows columns but
not how pricing creates or rejects them.

Required upgrades:

- Add a pricing-round diagram with generated, duplicate, infeasible, and chosen
  columns.
- Add test/fixture references for pricing/master/formulation-only behavior.

### U.18 Local Search - 49/60

**Roles applied:** MERIDIAN, TRENCH, COVENANT, BENCHMARK

Strongest element: starting-plan to descendant-plan story is clear.

Weakest element: no concrete before/after edge-cut example.

Required upgrades:

- Add before/after boundary-unit move with initial/final edge cut and population
  deviation.
- Reference no-op and validity-preservation fixtures.

### U.19 Evolutionary Comparison - 45/60

**Roles applied:** SCALE, DATUM, COVENANT, BENCHMARK

Strongest element: selected-frontier packaging is well scoped.

Weakest element: evolutionary mechanics are compressed; crossover/mutation and
validity fallback need their own visual.

Required upgrades:

- Add crossover/mutation diagram showing valid child vs fallback parent.
- Add uncertainty/claim-boundary note that frontier samples do not prove the
  complete Pareto set.

### U.20 RPLAN Audit Certificates - 51/60

**Roles applied:** COVENANT, LEDGER, BOUNDARY, BENCHMARK

Strongest element: fixed-point story and separation between certificate and
algorithm lineage.

Weakest element: traceability should explicitly reference schema/version and
negative fixtures.

Required upgrades:

- Add a failure-path diagram for mismatched context hash, profile mismatch, and
  malformed lineage.
- Reference RPLAN schema docs and negative verifier fixtures.

### ReCom Ensemble - 46/60

**Roles applied:** SCALE, MERIDIAN, DATUM

Strongest element: one ReCom step is visible and claim boundary is appropriately
qualified.

Weakest element: output evidence and diagnostics are mostly prose.

Required upgrades:

- Add R-hat/ESS diagnostic visual or output snippet.
- Reference L1/L2 ensemble tests and seed derivation.

### Sequential Monte Carlo - 47/60

**Roles applied:** SCALE, DATUM, BENCHMARK, LEDGER

Strongest element: particle genealogy and resampling are visible.

Weakest element: proposal step is not visualized; NDJSON/schema details are not
shown.

Required upgrades:

- Add staged district proposal visual before the resampling diagram.
- Add NDJSON metadata/output format references.

### Multiscale MCMC - 44/60

**Roles applied:** SCALE, MERIDIAN, DATUM, TRENCH

Strongest element: claim boundary is unusually strong and correctly cautious.

Weakest element: mechanics and visuals do not show a specific coarse proposal
becoming a fine rebalance.

Required upgrades:

- Add a concrete coarse-to-fine projection and rebalance failure/success path.
- Reference exact config fields and L0 tests.

### Three-Layer Compositor - 38/60

**Roles applied:** SURVEY, MERIDIAN, DATUM

Strongest element: the three orthogonal choices are clear.

Weakest element: page is conceptual; it does not show a complete example run.

Required upgrades:

- Add a worked YAML/CLI example transformed into structure/weights/search.
- Add a diagram showing how changing one layer preserves the others.
- Add output evidence: config hash, run manifest, sidecars.

### GeoSection - 37/60

**Roles applied:** MERIDIAN, DATUM, SCALE

Strongest element: ratio-normalization concept is present.

Weakest element: visual does not show the caterpillar failure as area division.

Required upgrades:

- Add a before/after map-like diagram: 1:(k-1) caterpillar vs normalized
  balanced top-level split.
- Add seed budget, ratio scan output, and tests/implementation references.

### AreaSection - 38/60

**Roles applied:** MERIDIAN, CONTOUR, DATUM, SCALE

Strongest element: population vs area constraint distinction is accurate.

Weakest element: visual does not actually show land-area division or Lorenz
feasibility.

Required upgrades:

- Add an area-division diagram showing dense urban population vs sparse land
  area.
- Add Lorenz feasibility mini-chart.
- Add data provenance requirements for ALAND/population.

### ApportionRegions - 40/60

**Roles applied:** MERIDIAN, DATUM, SCALE

Strongest element: factor-tree mechanics are understandable.

Weakest element: diagram shows k=14 but not a real region split or prime fallback
case.

Required upgrades:

- Add k=17 prime fallback example through two levels.
- Add region-spine visual showing geographic reuse property.
- Add implementation and fixture references.

### County-Sticky Weights - 36/60

**Roles applied:** MERIDIAN, WARD, BOUNDARY, DATUM

Strongest element: correctly classified as weights layer, not structure.

Weakest element: too thin; lacks numeric weight example and county-split
evidence.

Required upgrades:

- Add numeric edge-weight diagram: boundary length times county multiplier.
- Add compactness/county-split trade-off visual.
- Add state-law caveat: discouraging county splits is not the same as satisfying
  state subdivision preservation law.

### Seed Search Modes - 36/60

**Roles applied:** MERIDIAN, SCALE, DATUM, SURVEY

Strongest element: distinguishes single, multi, convergence, percentile, and
bisection-ensemble.

Weakest element: lacks concrete seed stream, stopping, and percentile examples.

Required upgrades:

- Add seed-stream timeline showing convergence threshold.
- Add sorted-candidate percentile visual.
- Add claim-boundary table: each mode answers a different question.

## Upgrade Priority

1. **GeoSection and AreaSection**: these most directly triggered the user's
   concern. They discuss area/ratio division but do not show enough area
   division.
2. **Seed Search Modes and County-Sticky**: both are close to thin and need
   concrete examples.
3. **U.16-U.19**: add second mechanics diagrams and test/report references.
4. **Sampling pages**: add diagnostics/output format visuals.
5. **T.14-T.17**: polish with test references and one more concrete numeric
   example each.

## Acceptance Target

Before calling the atlas mature, every page should score at least **45/60**, and
the key teaching pages should score **54/60 or higher**:

- T.14 Spectral Partitioning
- T.15 Capacity Clustering
- T.16 Hierarchical Regionalization
- T.17 Flow Construction
- U.20 RPLAN Audit Certificates
- Three-Layer Compositor
- GeoSection
- AreaSection
- ApportionRegions
