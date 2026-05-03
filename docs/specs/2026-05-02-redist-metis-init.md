# redist-metis Initial Partition Sub-Spec
**Date:** 2026-05-02
**Phase:** 3 of 8
**Implements:** `src/init/` — GrowBisect, GrowKway, RandomBisect, MultiConstraintInit

## Purpose

Applied once at the coarsest level (after all coarsening rounds). The coarsest graph
has tens to hundreds of vertices. Initial partition seeds the refinement phase.

## Algorithm Entry Points

**Two distinct paths** — not a dispatch on k:

- **`part_recursive` path**: Uses GrowBisect recursively. For k=2: grow two BFS
  regions from random seeds. For k>2: bisect into ceil(k/2) and floor(k/2) halves,
  recurse on each half.

- **`part_kway` path**: Uses GrowKway. Seeds k regions simultaneously from k random
  distinct vertices, then expands in round-robin BFS order.

## GrowBisect Algorithm

1. `debug_assert!(g.is_valid())` — PP-PLAN-02: livelock prevention
2. If k=1: return all-zeros partition immediately
3. Pick two random distinct seed vertices (Pcg64 seeded)
4. BFS-expand two regions alternately, assigning each unvisited vertex to the
   expanding region
5. Assign any remaining unvisited vertices to part 0 (shouldn't happen on valid
   connected graph, but safe fallback)

## GrowKway Algorithm

1. `debug_assert!(g.is_valid())` — PP-PLAN-02
2. If k=1: return all-zeros immediately
3. Pick k random distinct seed vertices
4. BFS-expand all k regions in round-robin order (one vertex dequeued per region
   per round)
5. Assign any remaining unvisited to part 0

## Contracts

- Output `assignment.len() == g.n()`
- Output `assignment[i] < k` for all i
- Output contains every part 0..k at least once (for connected graphs)
- All randomness via Pcg64 (seed parameter)
- `debug_assert!(g.is_valid())` at entry of all partition functions (PP-PLAN-02)

## Test Strategy (L0)

- `grow_bisect_valid_partition`: output valid on grid 4×4
- `grow_bisect_k1_all_zero`: k=1 → all zeros, cut=0
- `grow_kway_valid_k4`: k=4 on grid 4×4
- `grow_kway_k1_all_zero`: k=1 trivial

## Files

- `src/init/mod.rs` — InitialPartitioner trait (already implemented)
- `src/init/grow.rs` — GrowBisect + GrowKway implementations
- `src/init/random.rs` — RandomBisect stub (Task 9)
- `src/init/multiconstraint.rs` — MultiConstraintInit stub (Task 9)
