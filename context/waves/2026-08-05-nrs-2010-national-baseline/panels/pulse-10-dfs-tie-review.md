# Pulse 10 Initial DFS Tie Review

**Date:** 2026-08-07
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** post-census interpretation and next-gate review

## Findings

- MERIDIAN: preserve the exact selection key and return diagnostics alongside
  the existing assignment.
- DATUM: distinguish minimum-population candidates from the smaller
  minimum-population/minimum-cut set where seed-dependent moved population can
  matter.
- SCALE: census all 44 multi-district roots once; do not infer child-node or
  fallback behavior.
- BENCHMARK: assignment and all objective components must match the governed
  artifacts for every accepted State.
- BOUNDARY: a tie opportunity is not observed output sensitivity or an
  optimality statement.

## Decision

The exact census passed for all 44 roots without changing any governed
assignment or objective. Twenty-nine roots expose two
minimum-deviation/minimum-cut candidates, so the preregistered stop condition
for further seed work was not met.

Do not expand blindly across all roots. Freeze a bounded sensitivity sample
from the 29 opportunity States before execution, retain previously tested RI,
NH, and GA as negative controls, and report final assignment variation
separately from initial candidate opportunity. The current counters do not
cover child nodes or fallback paths.

**Superseded by Pulse 11:** all 29 oriented ties collapse to one physical
bipartition per root. No initial root-0 physical-cut seed batch is warranted.
