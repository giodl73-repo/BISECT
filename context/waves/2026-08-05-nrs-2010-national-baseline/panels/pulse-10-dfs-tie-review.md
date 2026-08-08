# Pulse 10 Initial DFS Tie Review

**Date:** 2026-08-07
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** instrumentation and census review before national replay

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

Proceed with the instrumented benchmark-seed replay. Stop before further
100-seed work unless at least one root exposes a minimum-cut candidate count
above one.
