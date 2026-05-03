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

## Pipeline<S> Typestate

Three state markers:
- `NeedsPartition { levels_built: usize }` — coarsening complete
- `NeedsRefinement { k: u32, coarsest_n: usize }` — initial partition assigned
- `Complete` — refined and projected back to level 0

Illegal transitions are compile errors.

## Tests

### L0:
- `hierarchy_builds_from_path`: path-100 coarsens to valid hierarchy, coarsest.n() <= 40
- `hierarchy_stalls_returns_error`: NeverStops coarsener returns CoarseningStalled error
- `project_up_correct`: all-zero coarse assignment projects to all-zero fine assignment
