# Shared Kernel Crates - Close

## Outcome

The wave created the first reusable pure-Rust graph kernel crate,
`rgraph-core`, and proved it with both standalone synthetic tests and one BISECT
consumer.

## Landed

- `crates/rgraph-core`
  - `DirectedWeightedGraph`
  - `WeightedEdge`
  - typed `GraphError`
  - deterministic shortest-path distance and SSSP tree APIs
  - edge-filtered reachability
  - normalized weighted Brandes edge-betweenness
- `bisect-analysis`
  - `contiguity::bfs_component_count` now consumes `rgraph-core`
- Test ladder
  - L0 inline tests for shortest paths, filters, invalid nodes/weights,
    reachability, and Brandes centrality
  - L1 integration tests for weighted paths, filter behavior, centrality, and
    adapter error reporting
  - L2 ignored graph stress tests for large grid reachability and moderate-grid
    Brandes centrality
- Wave/spec docs
  - source spec updated with the first-wave decision
  - ROUTE adapter deferred until dependency location is portable

## Deferred

- ROUTE `HighwayGraph` adapter. The blocker is dependency placement: ROUTE should
  not gain a hard-coded path dependency on `C:\src\apportionment`.
- `rstat-core`. Still warranted, but it should be its own wave after the graph
  sharing pattern has a second real consumer.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis contiguity -- --test-threads=1
git diff --check
```

