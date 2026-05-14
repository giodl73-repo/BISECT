---
wave: shared-kernel-crates
pulse: 02
status: done
depends_on: [01]
governing_roles:
  - MERIDIAN
  - BENCHMARK
  - ROUTE-OPTIMIZATION-METHODOLOGIST
---

# Pulse 02 - Brandes Centrality Compatibility

## Mission

Add weighted Brandes edge-betweenness to `rgraph-core` and prove it matches the
ROUTE synthetic expectations before any route migration.

## Deliverables

- [x] Add normalized edge-betweenness API with documented tie handling.
- [x] Port ROUTE's equal-shortest-path and non-shortest-direct-edge synthetic
  cases into `rgraph-core` tests.
- [x] Keep route scoring and corridor semantics outside `rgraph-core`.
- [x] Run focused validation.

## Completion Notes

- `edge_betweenness` and `edge_betweenness_with_filter` return normalized
  edge-centrality values.
- ROUTE's synthetic equal-shortest-path and non-shortest-direct-edge cases are
  represented as `rgraph-core` unit tests without importing ROUTE domain types.

