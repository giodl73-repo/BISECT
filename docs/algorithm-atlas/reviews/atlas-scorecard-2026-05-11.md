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
| Excellent, 54-60 | GeoSection, Seed Search, U.20 |
| Good, 45-53 | T.14, T.15, T.16, T.17, U.16, U.17, U.18, U.19, ReCom, SMC, Multiscale, Three-Layer Compositor, AreaSection, ApportionRegions, County-Sticky |
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
| T.14 Spectral Partitioning | 8 | 8 | 7 | 8 | 8 | 8 | 47 | Good |
| T.15 Capacity Clustering | 8 | 8 | 6 | 8 | 8 | 8 | 46 | Good |
| T.16 Hierarchical Regionalization | 8 | 8 | 6 | 8 | 8 | 8 | 46 | Good |
| T.17 Flow Construction | 8 | 8 | 6 | 8 | 8 | 8 | 46 | Good |
| U.16 Branch-And-Cut | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.17 Branch-And-Price | 9 | 9 | 7 | 8 | 9 | 8 | 50 | Good |
| U.18 Local Search | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.19 Evolutionary Comparison | 9 | 9 | 7 | 8 | 9 | 9 | 51 | Good |
| U.20 RPLAN Audit Certificates | 10 | 10 | 7 | 9 | 10 | 10 | 56 | Excellent |
| ReCom Ensemble | 8 | 8 | 7 | 8 | 9 | 9 | 49 | Good |
| Sequential Monte Carlo | 8 | 8 | 7 | 8 | 9 | 9 | 49 | Good |
| Multiscale MCMC | 8 | 8 | 6 | 8 | 10 | 8 | 48 | Good |
| Three-Layer Compositor | 8 | 9 | 6 | 8 | 8 | 8 | 47 | Good |
| GeoSection | 10 | 9 | 10 | 10 | 9 | 9 | 57 | Excellent |
| AreaSection | 9 | 8 | 8 | 9 | 9 | 8 | 51 | Good |
| ApportionRegions | 8 | 8 | 7 | 8 | 9 | 8 | 48 | Good |
| County-Sticky Weights | 8 | 8 | 6 | 8 | 9 | 8 | 47 | Good |
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

**AreaSection - 51/60.** Strong population/area feasibility story, but the main
figure should adopt countable 2D block arrays and explicitly show feasible root
workloads.

**U.16, U.17, U.18, U.19 - 50-51/60.** These pages are accurate and
well-evidenced, but their primary visuals still need more complete
candidate-set -> decision-rule -> artifact-consequence paths.

**ReCom and SMC - 49/60.** Sampling pages explain diagnostics and genealogy,
but the visuals should better show candidate transitions and why a sampled step
or particle is accepted, rejected, copied, or diagnosed.

**T.14-T.17 - 46-47/60.** The T pages have good prose and ledgers, but their
main figures predate the stricter rubric. Each needs a gold-standard teaching
figure: candidates, decision rule, and final artifact/status consequence.

**Multiscale, Three-Layer Compositor, ApportionRegions, County-Sticky -
47-48/60.** These pages are conceptually useful but not yet visually
self-explanatory. They need countable candidate objects and clearer downstream
consequences.

## Upgrade Priority

1. **AreaSection**: convert the strong but still abstract area figure into
   countable 2D block arrays with explicit feasible root workloads.
2. **County-Sticky Weights**: show a county block map where candidate cuts cross
   or respect county boundaries, then show the weighted decision consequence.
3. **ApportionRegions**: replace factor-tree prose visuals with countable seat
   workloads, prime fallback, and reused spine consequences.
4. **T.14-T.17**: upgrade the construction-family visuals to show candidates,
   decision rules, and artifact/status consequences.
5. **U-series and sampling pages**: add richer primary teaching figures only
   after the spatial/compositor family reaches the GeoSection standard.

## Acceptance Target

Under the stricter rubric, every page remains at least good, but only
GeoSection, Seed Search, and U.20 currently score excellent. The next target is
to raise the spatial/compositor family to **54/60 or higher** with
gold-standard teaching figures:

- T.14 Spectral Partitioning
- T.15 Capacity Clustering
- T.16 Hierarchical Regionalization
- T.17 Flow Construction
- Three-Layer Compositor
- AreaSection
- ApportionRegions
- County-Sticky Weights
