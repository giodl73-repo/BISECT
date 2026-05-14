# RGRAPH Mask Edge Cut Consumer Close

## Outcome

Routed column-pricing bitmask edge-cut counting through the existing
`rgraph-core::undirected_edge_cut_by` adapter.

## Evidence

- `bisect-column::pricing::subset_edge_cut` now delegates graph traversal and
  one-count-per-undirected-edge behavior to `rgraph-core`.
- Bitmask membership remains local to `bisect-column`.
- No new shared API was added.

## Boundary

Weighted cut scoring loops remain local until a dedicated weighted-edge-cut
boundary is justified by reuse beyond repeated inline scoring in one module.
