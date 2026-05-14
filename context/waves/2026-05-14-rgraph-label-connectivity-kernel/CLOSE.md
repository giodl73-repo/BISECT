# RGRAPH Label Connectivity Kernel Close

## Outcome

Extracted assignment-label connectivity checks into `rgraph-core` and routed the
duplicated BISECT wrappers through the shared graph kernel.

## Evidence

- `rgraph-core::assignment_label_connected` checks one label-induced subgraph.
- `rgraph-core::assignment_labels_connected` checks a caller-provided label set.
- L0 coverage includes connected labels, disconnected labels, missing labels,
  length mismatches, and out-of-bounds neighbors.
- L1 coverage preserves a redistricting-like assignment fixture.
- Ignored L2 coverage stress-tests connected and disconnected labels on a larger
  grid.
- `bisect-local-search`, `bisect-clustering`, `bisect-flow`, and `bisect-ilp`
  consume the shared helper while keeping their domain wrappers.

## Boundary

`rgraph-core` owns only label-induced graph connectivity over adjacency lists.
Population balance, feasibility, repair, search, and branch-and-cut semantics
remain in the consuming crates.
