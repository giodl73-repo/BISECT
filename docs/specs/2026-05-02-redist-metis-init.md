# redist-metis Initial Partition Sub-Spec
**Date:** 2026-05-02
**Phase:** 3 of 8
**Implements:** `src/init/` — GrowBisect, GrowKway, RandomBisect, MultiConstraintInit

## Purpose

Applied once at the coarsest level (after all coarsening rounds). The coarsest graph
has tens to hundreds of vertices. Initial partition seeds the refinement phase.

## Algorithm Entry Points

**Two distinct paths** — not a dispatch on k:

- **`part_recursive` path**: GrowBisect for k=2 only; k>2 delegates to GrowKway.
  For k=2: grow two BFS regions from random seeds. For k>2: **delegates to GrowKway**
  (round-robin BFS from k seeds). True recursive bisection is not implemented in v1 —
  the `part_recursive` entry point uses GrowBisect for k=2 only; k>2 calls always
  route to GrowKway internally.

- **`part_kway` path**: GrowKway directly. Seeds k regions simultaneously from k random
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
- **PP-PLAN-02 guard**: GrowBisect and GrowKway call `debug_assert!(g.is_valid())` at
  entry to prevent livelock on disconnected graphs. RandomBisect also includes this guard.
- **k > n fallback**: if k exceeds the number of vertices, seed selection falls back to
  modular assignment (`v % k`), producing fewer than k distinct parts. This is a
  degenerate case; callers should ensure k <= n. The `Partitioner::split()` API validates
  k <= n and returns `Err(TooManyParts)` before reaching the initializer.

## Test Strategy (L0)

- `grow_bisect_valid_partition`: output valid on grid 4×4
- `grow_bisect_k1_all_zero`: k=1 → all zeros, cut=0
- `grow_kway_valid_k4`: k=4 on grid 4×4
- `grow_kway_k1_all_zero`: k=1 trivial
- `random_bisect_deterministic`: same seed produces identical assignment on the same
  graph — verifies Pcg64 determinism

## Files

- `src/init/mod.rs` — InitialPartitioner trait (already implemented)
- `src/init/grow.rs` — GrowBisect + GrowKway implementations
- `src/init/random.rs` — RandomBisect (Task 9)
- `src/init/multiconstraint.rs` — **MultiConstraintInit** (v1): delegates to GrowKway.
  Multi-constraint balance is handled during FM refinement. This is a complete v1
  implementation, not a stub — the delegation to GrowKway is intentional.
