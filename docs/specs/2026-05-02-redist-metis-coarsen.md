# redist-metis Coarsening Sub-Spec
**Date:** 2026-05-02
**Phase:** 2 of 8
**Implements:** `src/coarsen/` — HEM, SHEM, MinDegreeMatch

## Purpose

Coarsening repeatedly collapses a graph by matching vertex pairs until the graph
is small enough to partition directly. Each coarsening round produces one level in
the `CoarseningHierarchy` arena.

## Stopping Criterion

`should_stop(g)` returns `true` when:
```
g.n() <= max(coarsen_to * k, 40)
```
- `coarsen_to` default = 20 (from MetisParams)
- The absolute floor of 40 prevents over-coarsening on small states (e.g. Vermont k=1 → threshold = max(20, 40) = 40)
- The multilevel orchestrator enforces MAX_LEVELS=50 as a hard cap, returning `Err(CoarseningStalled)` if should_stop never fires

## Algorithms

### HeavyEdgeMatch (HEM)
1. Visit vertices in random order (Fisher-Yates shuffle, seeded with Pcg64)
2. For each unmatched vertex v: find its heaviest-edge unmatched neighbor u
3. Match (v, u) → assign both to a new coarse vertex c
4. Unmatched vertices become their own coarse vertex
5. Build coarsened graph: merge vertex weights (i64 accumulation to prevent overflow), merge edge weights (sum parallel edges)

**Complexity**: O(n + m)

### SortedHeavyEdgeMatch (SHEM) — default
1. Compute max incident edge weight per vertex
2. **Bucket sort** vertices by this weight descending — O(n + max_weight), NOT a comparison sort
3. For each vertex in sorted order: match with heaviest unmatched neighbor
4. Build coarsened graph same as HEM

**Complexity**: O(n + m + max_weight) — effectively O(n + m) for bounded integer weights

**Key distinction from HEM**: SHEM processes high-connectivity vertices first, producing better quality coarsened graphs. The sort MUST be bucket sort, not comparison sort.

### MinDegreeMatch
1. Sort vertices by degree ascending (comparison sort, O(n log n))
2. For each vertex in order: match with heaviest unmatched neighbor
3. Build coarsened graph same as HEM

**Complexity**: O(n log n + m) — degree sort dominates

## CoarseMap Contracts

Output of every `coarsen()` call satisfies:
- `cmap.len() == fine_graph.n()` — every fine vertex has a mapping
- `cmap[v] < coarse_graph.n()` for all v
- **Surjective**: every coarse vertex is the target of at least one fine vertex
- **Not injective**: multiple fine vertices may map to the same coarse vertex

## build_coarse_graph Invariants

The shared helper that all three coarsenens use:
- **Weight accumulation**: uses `i64` intermediate arithmetic, result fits in `i32` for census-realistic inputs
- **adjwgt preservation**: if input graph has `adjwgt: None`, output MUST also have `adjwgt: None`; if `adjwgt: Some(...)`, output sums parallel edge weights
- **Parallel edge merging**: multiple edges between same coarse vertex pair are summed into one

## Test Strategy

### L0 (unit):
- `coarsened_strictly_smaller`: output n < input n
- `cmap_length_equals_fine_n`: cmap.len() == g.n()
- `cmap_targets_in_range`: all cmap[v] < coarsened.n()
- `unweighted_stays_unweighted`: adjwgt: None in → adjwgt: None out
- `weighted_stays_weighted`: adjwgt: Some in → adjwgt: Some out
- `shem_prefers_heavy_edges`: on weighted_path4, vertices connected by heavy edges are matched

### L1 (integration):
- `coarsening_terminates_path255`: path-255 graph reaches should_stop within 50 coarsening rounds

## Files
- `src/coarsen/mod.rs` — Coarsener trait (already implemented)
- `src/coarsen/hem.rs` — HeavyEdgeMatch + build_coarse_graph helper
- `src/coarsen/shem.rs` — SortedHeavyEdgeMatch (imports build_coarse_graph from hem)
- `src/coarsen/mindegree.rs` — MinDegreeMatch (imports build_coarse_graph from hem)
