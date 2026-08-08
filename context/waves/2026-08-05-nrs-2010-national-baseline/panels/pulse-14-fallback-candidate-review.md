# Pulse 14 Fallback Candidate Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** post-census interpretation

## Findings

- MERIDIAN: preserve both fallback algorithms and append diagnostics to their
  existing return values.
- DATUM: distinguish evaluated candidates, oriented ties, and distinct
  unlabeled partitions after deviation and cut.
- SCALE: replay six State packages sequentially; do not rerun 1,155 nodes.
- BENCHMARK: require complete containing-State assignment preservation, not
  only local node equality.
- BOUNDARY: v0.3 is a first-feasible algorithm, so its diagnostics must not be
  described as exhaustive bridge-candidate enumeration.

## Decision

The replay preserved all six State assignments and all eight activated
stage/node assignments and objectives. Candidate-level ties occurred only at
2000 HI/root: v0.2 had five minimum-deviation/minimum-cut finalists and v0.3
had two. Both finalist sets represented one unlabeled physical partition.

Stop governed fallback seed expansion because no stage met the precommitted
multiple-physical-partition gate. Retain the diagnostics as evidence and do
not reinterpret this bounded mechanism result as plan robustness or
optimality.
