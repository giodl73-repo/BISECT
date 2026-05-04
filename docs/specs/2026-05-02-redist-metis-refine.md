# redist-metis FM Refinement Sub-Spec
**Date:** 2026-05-02
**Phase:** 4 of 8
**Implements:** `src/refine/` — GainTable, BoundarySet, FmState, FiducciaMattheyses, GreedyKWay

## FM Refinement Overview

Fiduccia-Mattheyses (FM) improves a partition by moving boundary vertices to
reduce edge cut while maintaining population balance. Applied at every level
during uncoarsening.

## GainTable

Bucket sort structure for O(1) max-gain lookup.

- Gains ∈ `[-max_gain, +max_gain]` where `max_gain = max edge weight × max degree`
- Storage: `buckets[gain + max_gain]` = list of vertex IDs at this gain
- Offset indexing ensures array indices are always non-negative
- `insert(v, gain)`: add vertex v with this gain
- `remove(v)`: remove vertex v (swap-with-last for O(1))
- `update(v, new_gain)`: remove then insert
- `pop_max()`: remove and return vertex with highest gain
- `peek_max()`: return (vertex, gain) without removing

**top_bucket tracking**: `top_bucket` is a cached hint for the highest non-empty bucket. It is updated on `insert` (if new gain > current top_bucket) and after `pop_max` (set to the just-popped gain). It is NOT decremented on `remove`. Therefore `peek_max` may need to scan downward from `top_bucket` through empty buckets to find the true maximum — O(max_gain) scan in the worst case, but O(1) amortized in practice.

## FM Pass Algorithm

1. Compute boundary set and gain table from current partition
2. Loop until budget exhausted (niter passes or no improvement):
   a. Pick highest-gain boundary vertex v
   b. **Locked-vertex invariant (FM defining property)**: After vertex v is popped from the gain table and moved, it is *locked* for the remainder of the pass — it cannot be moved again. Locked neighbors are skipped during gain updates. This is what distinguishes FM from plain greedy search. The implementation tracks this via a `locked: Vec<bool>` vector, set to `true` when a vertex is popped.
   c. Check balance: moving v from part `from` to part `to` must keep
      both parts within ε = `(total_pop * 5 + 999) / 1000` of target
      (INTEGER arithmetic only — NO floats)
   d. If balance violated: skip this vertex
   e. Apply move: update assignment, part populations, boundary set, gain table
   f. Update neighbors' gains (skipping locked vertices)
   g. Checkpoint if current cut < best seen cut
3. Rollback to best checkpoint

## FmState

Owns: `assignment: Vec<u32>`, `GainTable`, `BoundarySet`, `part_pop: Vec<i64>`

## Balance constraint

`epsilon = (total_pop * 5 + 999) / 1000` — ceiling of 0.5%, integer arithmetic only.
NEVER use float for balance check.

This epsilon formula is the **legally-cited balance property**. It is enforced at move time (input gate) in FM, and verified at output time by the Prusti postcondition:
```
#[ensures(population_balance(&result, graph) <= params.epsilon)]
```
The output-only Prusti verification is sufficient: any intermediate imbalance that FM fails to correct surfaces as a Prusti verification failure on the returned Partition.

## Contracts / Invariants

- Each vertex is moved at most once per FM pass. After being popped and moved, a vertex is locked and excluded from further consideration in that pass.
- Population balance is maintained within ε at every accepted move (enforced at input gate).
- The partition returned by FM satisfies `population_balance(&result, graph) <= params.epsilon` (Prusti postcondition).
- Edge cut is non-increasing from the best checkpoint seen during the pass (rollback guarantee).

## Tests

### L0 (GainTable):
- `gain_table_max_is_correct`
- `gain_table_remove_max`
- `gain_table_update_gain`
- `gain_table_negative_gain`

### L0 (FM):
- `fm_does_not_increase_cut`
- `fm_oracle_dumbbell_bisect` (cut = 1)
- `fm_preserves_population_balance`
- `fm_locked_vertex_invariant`: (future) verify no vertex appears in more than one move per pass. Currently verified by code inspection and the dumbbell oracle; a dedicated test is deferred to Phase 6 verification.
