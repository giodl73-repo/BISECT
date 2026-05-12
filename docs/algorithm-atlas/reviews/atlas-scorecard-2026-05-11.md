# Algorithm Atlas Scorecard

**Date:** 2026-05-11
**Rubric:** `docs/algorithm-atlas/RUBRIC.md`
**Reviewer:** Codex using the BISECT rubric and `.roles` lenses
**Latest pass:** rescored against the upgraded gold-standard visual rubric.
GeoSection is the reference standard; Seed Search has started the next upgrade
wave.

This scorecard grades the current atlas pages against the six BISECT dimensions:

- **B** Behavioral Mechanics
- **I** Integration With BISECT
- **S** Spatial And Visual Specificity
- **E** Explanatory Story
- **C** Claim Boundary And Correctness
- **T** Traceability And Evidence

Scores are out of 60. The main scoring pressure is now stricter than the first
atlas pass: a page must show candidate objects, why those candidates exist, the
decision rule, and the downstream workload or artifact consequence. Correct
prose plus generic diagrams no longer earns an excellent score.

## Summary

| Band | Pages |
|---|---|
| Excellent, 54-60 | T.14, T.15, T.16, T.17, GeoSection, AreaSection, Seed Search, County-Sticky, ApportionRegions, Three-Layer Compositor, Multiscale, U.20 |
| Good, 45-53 | U.16, U.17, U.18, U.19, ReCom, SMC |
| Serviceable, 36-44 | None |
| Thin, 24-35 | None |
| Failing, 0-23 | None |

The atlas remains useful, but the stricter rubric makes the next work clearer.
Many pages have accurate prose and supporting diagrams, yet their primary
visuals do not fully explain candidate purpose, decision rule, and consequence.
Those pages should be treated as good teaching drafts until their primary
figures are upgraded.

## Score Matrix

| Page | B | I | S | E | C | T | Total | Band |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| T.14 Spectral Partitioning | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| T.15 Capacity Clustering | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| T.16 Hierarchical Regionalization | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| T.17 Flow Construction | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| U.16 Branch-And-Cut | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.17 Branch-And-Price | 9 | 9 | 7 | 8 | 9 | 8 | 50 | Good |
| U.18 Local Search | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.19 Evolutionary Comparison | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.20 RPLAN Audit Certificates | 10 | 10 | 7 | 9 | 10 | 10 | 56 | Excellent |
| ReCom Ensemble | 8 | 8 | 7 | 8 | 9 | 9 | 49 | Good |
| Sequential Monte Carlo | 8 | 8 | 7 | 8 | 9 | 9 | 49 | Good |
| Multiscale MCMC | 9 | 9 | 9 | 10 | 10 | 8 | 55 | Excellent |
| Three-Layer Compositor | 9 | 10 | 9 | 10 | 9 | 9 | 56 | Excellent |
| GeoSection | 10 | 9 | 10 | 10 | 9 | 9 | 57 | Excellent |
| AreaSection | 10 | 9 | 10 | 10 | 9 | 8 | 56 | Excellent |
| ApportionRegions | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| County-Sticky Weights | 9 | 9 | 10 | 10 | 9 | 8 | 55 | Excellent |
| Seed Search Modes | 9 | 9 | 9 | 10 | 9 | 8 | 54 | Excellent |

## Page Notes

### Excellent Under The New Rubric

**GeoSection - 57/60.** The page now shows countable 2D root-allocation
thumbnails, recursive workload chips, the normalized decision rule, and the
effect of the chosen first split on later recursion. It is the current visual
reference.

**Seed Search Modes - 54/60.** The page now shows why candidate plans exist,
how seed-derived candidates differ, how mode changes the selection rule, and
what each selected result may claim.

**U.20 RPLAN Audit Certificates - 56/60.** The fixed-point story is strong and
claim-safe. Its next visual upgrade should make positive and negative verifier
paths as concrete as GeoSection's root-allocation workload.

### Good But Still Below The Gold Standard

**AreaSection - 56/60.** The page now uses countable population/area block
arrays, shows a skipped dense-core candidate, shows the feasibility decision
rule, and names the recursive workload created by the feasible root.

**County-Sticky Weights - 55/60.** The page now shows county-labeled block
units, candidate cuts that cross or respect county boundaries, weighted
decision scores, and the correct consequence: county splits are discouraged,
not banned.

**ApportionRegions - 55/60.** The page now shows the countable seat-workload
tree before geography enters: prime fallback, child factor logic, and the final
regional workload consequence.

**Three-Layer Compositor - 56/60.** The page now shows candidate choices in
structure, weights, and search, the resolved recipe, the config-hash
consequence, and sidecars that identify which layer produced each piece of
evidence.

**Multiscale MCMC - 55/60.** The page now shows fine units, coarse candidate
moves, projection, rebalance gates, output status fields, and the sampler
correctness claim boundary.

**T.14 Spectral Partitioning - 55/60.** The page now shows the graph-to-Fiedler
ordering candidate path, sweep-cut decision table, recursive workload
consequence, and package-summary evidence boundary.

**T.15 Capacity Clustering - 55/60.** The page now shows why deterministic
farthest-point seeds are tried, how assignment candidates are tested against
capacity, why a nearest cluster can be rejected, and which status evidence is
carried into RPLAN/RCTX/certificate export.

**T.16 Hierarchical Regionalization - 55/60.** The page now shows singleton
regions, adjacent-pair candidate enumeration, eligibility gating, a selected
merge witness, and the merge-tree/package consequences BISECT must preserve.

**T.17 Flow Construction - 55/60.** The page now shows unit supply, eligible
arcs, selected assignment flow, explicit capacity bounds, a concrete overfill
failure, and the valid/infeasible/invalid status fork that controls export.

**U.16, U.17, U.18, U.19 - 50-51/60.** These pages are accurate and
well-evidenced, but their primary visuals still need more complete
candidate-set -> decision-rule -> artifact-consequence paths.

**ReCom and SMC - 49/60.** Sampling pages explain diagnostics and genealogy,
but the visuals should better show candidate transitions and why a sampled step
or particle is accepted, rejected, copied, or diagnosed.

## Upgrade Priority

1. **U-series and sampling pages**: add richer primary teaching figures only
   after the spatial/compositor family reaches the GeoSection standard.

## Acceptance Target

Under the stricter rubric, every page remains at least good, but only
T.14, T.15, T.16, T.17, GeoSection, AreaSection, County-Sticky, ApportionRegions, Three-Layer
Compositor, Multiscale, Seed Search, and U.20 currently score excellent. The
next target is to raise the remaining U-series and sampling pages to **54/60 or
higher** with gold-standard teaching figures.
