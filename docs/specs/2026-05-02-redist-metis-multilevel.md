# redist-metis Multilevel Orchestration Sub-Spec
**Date:** 2026-05-02
**Phase:** 5 of 8
**Implements:** `src/multilevel/` — CoarseningHierarchy, Pipeline<S> typestate

## CoarseningHierarchy

Arena that owns all coarsened graph levels. No pointer chains — indices are stable.

```rust
pub struct CoarseningHierarchy {
    pub levels: Vec<CsrGraph>,   // [0]=original … [n]=coarsest
    pub cmaps:  Vec<CoarseMap>,  // cmap[i] maps level[i+1] → level[i]
}
```

### build() contract
- Input: original CsrGraph (must be valid), a &dyn Coarsener
- Coarsen repeatedly until coarsener.should_stop() returns true
- Hard cap: MAX_LEVELS = 50. If cap reached without should_stop → return Err(CoarseningStalled)
- Output: hierarchy with levels[0] = original, levels[depth] = coarsest

### project_up() contract
- Takes level index `lev` and coarse partition assignment
- Returns fine partition: fine[v] = coarse[cmap[lev][v]]
- Called during uncoarsening to project partition from coarser to finer level

### depth=0 case
**depth=0 case**: if `should_stop` returns true immediately (graph is already small enough), `CoarseningHierarchy::build()` returns a hierarchy with only one level (levels[0] = original). `Pipeline::refine_and_project()` handles this correctly: the `for lev in (0..depth).rev()` loop is a no-op, and only the final refinement at level 0 runs. The full pipeline works on single-level graphs.

## Pipeline<S> Typestate

Three state markers:
- `NeedsPartition { levels_built: usize }` — coarsening complete
- `NeedsRefinement { k: u32, coarsest_n: usize }` — initial partition assigned
- `Complete` — refined and projected back to level 0

Illegal transitions are compile errors.

**`partition: Option<Partition>` invariants by state**:
- `Pipeline<NeedsPartition>`: `partition` is always `None`
- `Pipeline<NeedsRefinement>`: `partition` is always `Some(p)` where p is the initial partition at the coarsest level
- `Pipeline<Complete>`: `partition` is always `Some(p)` where p is the final refined partition at level 0
The `Option` is a Rust implementation necessity (ownership during moves); the typestate ensures it is always `Some` when semantically expected.

**Evidence fields**: The state markers carry compile-time evidence of what invariant holds, not just labels:
- `NeedsPartition { levels_built: usize }` — records how many coarsening rounds completed
- `NeedsRefinement { k: u32, coarsest_n: usize }` — locks in the partition parameters
These fields add zero runtime overhead (optimized away by the compiler) but make the state semantically meaningful for verification annotations and debug assertions.

### refine_and_project

**Uncoarsening order (critical)**: For each level `lev` from `depth-1` down to `0`:
1. **Refine FIRST** at the coarser level `lev+1` — FM improves the partition while the graph is still coarser
2. **Project DOWN** to the finer level `lev` via `project_up(lev, assignment)` — expand to the next finer level
After the loop, one final refinement runs at level 0 (the original graph).

This order — refine-then-project — is correct and matches C METIS behavior. The reverse (project-then-refine) would refine at the wrong graph level.

**Index safety**: `levels[lev + 1]` is accessed only inside the loop `for lev in (0..depth).rev()`. When `depth=0` the loop body never executes, so no out-of-bounds access is possible. When `depth >= 1`, `lev` ranges from `depth-1` to `0`, so `lev+1` ranges from `depth` to `1`, all valid indices in `levels` (which has `depth+1` entries).

## Tests

### L0:
- `hierarchy_builds_from_path`: path-100 coarsens to valid hierarchy, coarsest.n() <= 40
- `hierarchy_stalls_returns_error`: NeverStops coarsener returns CoarseningStalled error
- `project_up_correct`: all-zero coarse assignment projects to all-zero fine assignment

### Pipeline tests (L0):
- `pipeline_path10_k2_produces_valid_partition`: full NeedsPartition→NeedsRefinement→Complete chain on path-10, k=2
- `pipeline_path100_k4_produces_valid_partition`: larger graph, k=4, verifies multi-level uncoarsening
- `pipeline_no_coarsening_still_works`: graph that is already below should_stop threshold — depth=0 case, loop is no-op, final refinement runs at level 0
