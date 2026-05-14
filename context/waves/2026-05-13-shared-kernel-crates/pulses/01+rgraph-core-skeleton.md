---
wave: shared-kernel-crates
pulse: 01
status: done
depends_on: []
governing_roles:
  - MERIDIAN
  - BENCHMARK
  - LEDGER
  - ROUTE-NUMERACY-CHECKER
---

# Pulse 01 - RGRAPH Core Skeleton

## Mission

Create `rgraph-core` as the first shared algorithm kernel crate with deterministic
shortest-path and reachability primitives.

## Scope

| Surface | Target | Non-goal |
|---|---|---|
| Crate | Workspace crate `crates/rgraph-core` with minimal dependencies. | Standalone external repository. |
| API | Directed weighted graph trait, typed errors, shortest path distance, SSSP tree, reachability. | Brandes centrality migration. |
| Tests | Tiny positive and negative synthetic graphs. | Large route or Census fixtures. |
| Docs | Wave/pulse status update. | Full `rstat-core` design. |

## Pre-implementation Scout

Run and record:

```powershell
rg -n "Dijkstra|shortest|betweenness|connectivity" crates C:\src\route\crates
Get-ChildItem crates -Directory | Select-Object Name
```

## Deliverables

- [x] Add `rgraph-core` to the workspace.
- [x] Define graph edge, graph trait, shortest-path tree, and typed error model.
- [x] Add deterministic shortest path and edge-filtered reachability helpers.
- [x] Add tests for equal shortest paths, filtered disconnection, one-node graph,
  invalid source node, and negative/non-finite weight rejection.
- [x] Update `WAVE.md` pulse status.
- [x] Run focused validation.

## Completion Notes

- `rgraph-core` exposes `DirectedWeightedGraph`, `WeightedEdge`,
  `ShortestPathTree`, typed `GraphError`, shortest-path distance, SSSP, and
  filtered reachability APIs.
- Edge weights are abstract non-negative costs. Domain crates own units.
- Invalid source/target nodes and negative/non-finite weights are typed errors.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
git diff --check
```

