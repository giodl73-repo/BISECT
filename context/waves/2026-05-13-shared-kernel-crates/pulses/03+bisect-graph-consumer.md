---
wave: shared-kernel-crates
pulse: 03
status: done
depends_on: [02]
governing_roles:
  - MERIDIAN
  - BENCHMARK
  - TRENCH
---

# Pulse 03 - BISECT Graph Consumer

## Mission

Wire one BISECT graph-only diagnostic or helper to `rgraph-core` so the crate has
a real in-repo consumer.

## Deliverables

- [x] Select a graph-only helper that does not import route or civic package
  semantics.
- [x] Replace or wrap duplicated traversal logic.
- [x] Add compatibility tests that would catch drift.
- [x] Run `rgraph-core` and touched package tests.

## Completion Notes

- `bisect-analysis::contiguity::bfs_component_count` now adapts adjacency lists
  to `rgraph-core` and calls `reachable_nodes_with_filter`.
- Existing contiguity tests cover the migration and preserve public behavior.

