# Pulse 16 National Tier 2 Geometry Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, CONTOUR, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** post-execution interpretation

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

The sequential run passed all 50 States, both 435-district plan families, and
the complete 7,889,194-block universe with no failures. State-weighted and
district-weighted summaries agree in direction across all four frozen metrics.

Provisionally accept the descriptive national result pending exact
regeneration. Do not convert aligned aggregate direction into a superiority
claim: the analysis uses common block-projected geometry, excludes legal and
electoral criteria, and does not measure original comparator polygon linework.
