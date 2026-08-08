# Pulse 13 Cross-Census DFS Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** protocol review before replay

## Findings

- MERIDIAN: reuse the 2020 diagnostics unchanged; do not alter amendment
  behavior during comparison.
- DATUM: include year in every State and node key and report v0.2 and v0.3
  activation separately.
- SCALE: run 2000 and 2010 as one frozen package but preserve year-level
  summaries and failures.
- BENCHMARK: compare all 770 node paths, assignments, and primary objectives.
- BOUNDARY: documented fallback activation is a mechanism witness, not a
  robustness or superiority result.

## Decision

Proceed with the exact cross-census replay. Instrument internal fallback
candidate multiplicity only for the activated nodes found by this ledger.
