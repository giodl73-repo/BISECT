# RGRAPH Subset Connectivity Kernel Close

## Outcome

Extracted adjacency-list node-subset connectivity into `rgraph-core`.

## Evidence

- `rgraph-core::node_subset_connected` validates subset nodes and adjacency
  neighbors before traversing the induced subgraph.
- L0 coverage includes connected, disconnected, singleton, empty, duplicate,
  out-of-bounds-node, and out-of-bounds-neighbor cases.
- L1 coverage preserves a redistricting-like path fixture.
- Ignored L2 coverage stress-tests connected and disconnected subsets on a
  larger grid.
- `bisect-column`, `bisect-smc::proposal`, `bisect-pareto::mutation`, CLI
  bisection subset checks, and SA evidence checks consume the shared helper.

## Boundary

`rgraph-core` treats node lists as sets and returns `true` for an empty subset.
Callers retain their own empty-subset policy and all pricing, proposal, mutation,
repair, and evidence semantics.
