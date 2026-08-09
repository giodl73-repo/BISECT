# Pulse 15 Tier 2 Geometry Review

**Date:** 2026-08-08
**Roles:** MERIDIAN, CONTOUR, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** protocol review before implementation

## Findings

- MERIDIAN: preserve Tier 1 assignments and add geometry as a post-hoc
  evidence layer; do not modify NRS generation.
- CONTOUR: dissolve the same retained block polygons for both plan families.
  Include multipart components and holes, and prohibit smoothing or silent
  geometry repair.
- DATUM: report the projection, atomic-universe counts, per-district metrics,
  metric definitions, and all unavailable families.
- SCALE: prove Rhode Island first. National execution needs a separate frozen
  projection table, aggregation contract, failure ledger, and runtime gate.
- BENCHMARK: require known-shape identities, multipart coverage, structured
  invalid-input failures, source hashes, and byte-exact regeneration.
- BOUNDARY: describe block-projected geometry only. Do not claim the measured
  boundary is the original enacted linework or turn descriptive differences
  into superiority findings.

## Decision

Proceed with a two-district Rhode Island slice using `EPSG:32130`,
Polsby--Popper, exact GEOS minimum-bounding-circle Reock, convex-hull ratio,
and Schwartzberg. National Tier 2 remains gated on accepted and failure proof.
