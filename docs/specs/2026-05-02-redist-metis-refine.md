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

## FM Pass Algorithm

1. Compute boundary set and gain table from current partition
2. Loop until budget exhausted (niter passes or no improvement):
   a. Pick highest-gain boundary vertex v
   b. Check balance: moving v from part `from` to part `to` must keep
      both parts within ε = `(total_pop * 5 + 999) / 1000` of target
      (INTEGER arithmetic only — NO floats)
   c. If balance violated: skip this vertex
   d. Apply move: update assignment, part populations, boundary set, gain table
   e. Update neighbors' gains
   f. Checkpoint if current cut < best seen cut
3. Rollback to best checkpoint

## FmState

Owns: `assignment: Vec<u32>`, `GainTable`, `BoundarySet`, `part_pop: Vec<i64>`

## Balance constraint

`epsilon = (total_pop * 5 + 999) / 1000` — ceiling of 0.5%, integer arithmetic only.
NEVER use float for balance check.

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
