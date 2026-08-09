# Pulse 16 National Tier 2 Geometry Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, CONTOUR, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** protocol review before national execution

## Findings

- MERIDIAN: generalize only the accepted post-hoc analyzer; preserve NRS and
  Tier 1 comparator assignments byte-for-byte.
- CONTOUR: use the frozen CONUS, Alaska, and Hawaii equal-area projections and
  the accepted complete-boundary rule.
- DATUM: publish district and State ledgers, both national estimands, source
  counts, and all failures.
- SCALE: process exactly one State at a time and record elapsed time in the
  execution transcript, not deterministic package content. Do not parallelize
  regeneration.
- BENCHMARK: require 50 States, 435 districts per plan family, exact ledger
  recomputation, and byte-identical clean-tree regeneration.
- BOUNDARY: heterogeneous State geometry makes a national mean descriptive
  only. Prohibit ranks, significance tests, winners, and composite scores.

## Decision

Proceed sequentially. Stop and publish a partial package if any State fails;
do not alter geometry semantics or choose replacement inputs after outcomes.
