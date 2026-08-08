# Pulse 13 Cross-Census DFS Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** post-census interpretation

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

The replay preserved all 86 State assignments and all 770 node
assignments/objectives. No node had multiple physical initial cuts. Seven
nodes activated v0.2, and 2000 HI/root alone activated v0.3.

Restrict deeper fallback instrumentation to those seven nodes. Preserve the
other 763 nodes as explicit nonactivated exclusions rather than expanding seed
execution nationally.
