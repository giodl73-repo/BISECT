# Pulse 08 Rhode Island Sensitivity Review

**Date:** 2026-08-07
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** protocol review before diagnostic-seed generation

## Findings

- MERIDIAN: keep the governed benchmark authoritative and vary only the
  discovery seed on the same block context, district count, and NRS v0.3
  refinement path.
- DATUM: make the schedule's seed formula byte-exact, retain all 100 indices,
  and preserve duplicate seeds or command failures rather than replacing them.
- SCALE: Rhode Island is a representative complete two-district slice, not a
  national robustness estimate. Report distributions and benchmark placement,
  not independent-observation p-values.
- BENCHMARK: bind the executable and inputs, pack every accepted assignment,
  and require exact full-package regeneration.
- BOUNDARY: omit partisan, VRA, compactness, legal, ensemble, and optimality
  claims. A lower weighted cut does not override the lexicographic population
  objective or authorize benchmark replacement.

## Complexity Removed

- No new generator or generic experiment framework.
- No post-outcome seed filtering or selected-map publication.
- No national 5,000-run expansion before the Rhode Island package proves the
  derivation, storage, and verifier contracts.
- No election or demographic overlay.

## Decision

Proceed with the 100 frozen diagnostic indices. Stop with a structured package
even if every seed fails or the benchmark is atypical.
