# RGRAPH Edge Cut Adapter Kernel Close

## Outcome

Added a closure-based edge-cut adapter for callers whose assignment labels are
not stored as a simple slice.

## Evidence

- `rgraph-core::undirected_edge_cut_by` owns only adjacency traversal,
  out-of-bounds validation, and one-count-per-undirected-edge semantics.
- `undirected_edge_cut` remains the assignment-slice helper.
- CLI bisection-runner HashMap defaults remain local to the caller.
- SA evidence side-membership semantics remain local to the caller.

## Boundary

This does not move assignment defaults, set membership, evidence semantics, or
recursive bisection selection policy into `rgraph-core`.
