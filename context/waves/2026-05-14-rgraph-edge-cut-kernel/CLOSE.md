# RGRAPH Edge Cut Kernel Close

## Outcome

Extracted undirected edge-cut counting into `rgraph-core` and routed the existing
local-search, Pareto, and parallel-tempering consumers through the shared graph
kernel.

## Evidence

- `rgraph-core::undirected_edge_cut` provides typed errors for assignment-length
  mismatches and out-of-bounds neighbors.
- L0 coverage exercises usize/u32 inputs, crossing counts, and error cases.
- L1 coverage preserves a redistricting-like fixture.
- Ignored L2 coverage stress-tests a larger grid split.
- Consumer coverage preserves local-search improvement, Pareto EC objectives,
  and parallel-tempering EC selection behavior.

## Boundary

`rgraph-core` owns only graph-only edge-cut counting. Compactness meaning,
optimization objectives, and sampler selection semantics remain in the consuming
BISECT crates.
