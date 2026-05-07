# Spec: BFS Region-Growing — Greedy Geographic District Construction

**Status**: Proposed (R1 reviewed, P1 fixes applied)  
**Date**: 2026-05-07  
**Layer**: Structure (SplitStrategy) — replaces METIS  
**Related paper**: B.23  
**Implementation**: `redist-cli/src/bisection_runner.rs` (no new crate)  
**Reviewed R1**: MERIDIAN 4/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 4/4 → avg 3.5/4

---

## Overview

BFS Region-Growing is the simplest possible redistricting algorithm: seed one tract per district, then repeatedly assign each unassigned tract to the adjacent district with the greatest population deficit. This "greedy packer" fills space by proximity to seeds, producing contiguous districts by construction.

Unlike METIS (optimizes edge cut globally) or CVD (minimizes geographic distance), BFS Growth optimizes nothing — it is a fast, deterministic baseline that shows what "pure geographic proximity with population balance" looks like. Runtime: O(n log n) per bisection node. No MCMC, no spanning trees, no iterations.

BFS-based initial partitioning is used internally by METIS as the coarsening warm-start (Karypis & Kumar 1998 §4); BFS Growth elevates this to a standalone algorithm for comparison and baseline use.

BFS Growth is useful as:

1. A fast initial plan for other algorithms (better warm start than pure METIS)
2. A baseline: "how compact are plans without optimization?"
3. A legal argument: "this is the simplest geographic partition"

**Relationship to existing structure strategies**:

| Property | BFS Growth (B.23) | CVD (B.22) |
|---|---|---|
| Metric optimized | None (greedy) | Geographic distance |
| Data needed | Adjacency only | Adjacency (Phase 1), Centroids (Phase 2) |
| Iterations | 1 pass | ~20 iterations |
| Runtime | O(n log n) | O(n × k × 20) |
| Contiguity guarantee | Yes (BFS) | No (need post-processing) |
| Compactness quality | Low baseline | Medium |
| Use case | Fast baseline | Geometric packing |

---

## 1. Algorithm

```
BfsGrowth(adj, pop, k, balance_tolerance, base_seed):
  n = |adj|
  ideal_pop = sum(pop) / k

  // Step 1: Select k seed tracts (distributed across the graph)
  seeds = select_seeds(adj, pop, k, base_seed)
  // seed selection: k-farthest seeds via BFS distance (greedy, maximally spread)
  // seed[0] = population-weighted random sample using bfs_rng
  // seed[i] = tract with maximum min-BFS-distance to any of seeds[0..i]

  // Step 2: Initialise — each seed gets its district
  assignment = [None] * n
  for i in 0..k: assignment[seeds[i]] = i

  // Priority queue: (priority, tract, district)
  // priority = |pop(district) - ideal_pop|  (prefer growing most-deficit districts)
  queue = BinaryHeap::new()
  for i in 0..k:
    for &nb in &adj[seeds[i]]:
      if assignment[nb].is_none():
        queue.push((deficit(i), nb, i))

  // Step 3: Grow by BFS, always assigning to the most population-deficient district
  while !queue.is_empty():
    (_, tract, district) = queue.pop_min()
    if assignment[tract].is_some(): continue  // already assigned by another path

    assignment[tract] = district

    // Add unassigned neighbours to queue
    for &nb in &adj[tract]:
      if assignment[nb].is_none():
        queue.push((deficit(district), nb, district))

  // Step 4: Post-hoc rebalance (same 200-iter boundary-swap logic as MultiScale)
  assignment = rebalance(assignment, adj, pop, balance_tolerance, max_iters=200)

  return assignment

fn deficit(district_idx) -> i64:
  (ideal_pop - current_pop[district_idx]).abs()
```

---

## 2. Seed selection

Seeds are placed to maximally spread district starting points:

```
select_seeds(adj, pop, k, base_seed):
  bfs_rng = SmallRng::seed_from_u64(SHA-256("BFS_SEED_" || base_seed:u64le) -> u64le)

  // Seed 0: random weighted by population (denser areas first)
  seeds = [weighted_sample(range(n), pop, bfs_rng)]

  // Seeds 1..k-1: k-farthest greedy selection
  for _ in 1..k:
    // BFS distance from all current seeds
    dist = bfs_min_distance_from_seeds(adj, &seeds)
    // Add the tract with maximum min-distance
    seeds.push(argmax(dist))

  seeds
```

The BFS-farthest strategy ensures seeds are geographically spread across the state, so each district starts growing from a distinct region rather than clustering at a dense population centre.

---

## 3. Seeding specification

All seeds derived from `base_seed` via SHA-256 with domain-separated prefix:

```
bfs_seed(base_seed) = SHA-256("BFS_SEED_" || base_seed:u64le) -> u64le (least-significant 64 bits)
bfs_rng = SmallRng::seed_from_u64(bfs_seed(base_seed))
```

The prefix `"BFS_SEED_"` embeds algorithm identity. Any change to how the seed is used must change the prefix. A test asserts that the prefix constant in source equals `"BFS_SEED_"` exactly and that `bfs_seed(0)` equals a hard-coded expected value, preventing silent seed-compatibility drift.

Same `base_seed` → identical seed placement → identical BFS assignment → identical final plan.

---

## 4. Compositor integration

```rust
SplitStrategy::BfsGrowth {
    balance_post_iters: usize,  // max rebalance iterations (default: 200)
}
```

No additional hyperparameters — BFS Growth has no tunable knobs beyond the standard balance tolerance inherited from the compositor's `--balance-tolerance` flag.

---

## 5. CLI

```bash
--structure bfs-growth
```

**YAML**:
```yaml
algorithm:
  structure: bfs-growth
  weights: geographic
  search: single
  balance_tolerance: 0.5
workers: 8
years: ["2020"]
```

---

## 6. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"structure": "bfs-growth",
"balance_post_iters": 200,
"base_seed": 12345678,
"bfs_seed": 876543210,
"rebalance_succeeded": true,
"rebalance_iterations": 47,
"final_ec": 3124,
"final_pp_mean": 0.198
```

`rebalance_succeeded` records whether the post-hoc rebalance converged within `balance_post_iters`. If false, the plan may slightly violate `balance_tolerance`, but all districts still contain at least 1 tract and no district population exceeds `2 × ideal_pop` (the worst-case single coarse-step imbalance). An auditor who knows `base_seed` can re-derive `bfs_seed`, replay seed selection and BFS assignment, and verify the submitted plan.

---

## 7. Test invariants

### L0 (inline unit tests)

- All n tracts assigned — no unassigned tracts after BFS + rebalance
- All k districts non-empty
- All districts contiguous — BFS growth guarantees this by construction (tracts are added to connected frontier)
- Same `base_seed` → identical assignment (deterministic)
- `rebalance_iterations <= balance_post_iters`
- Seeds are distinct — no duplicate seed tracts
- Seeds are maximally spread: min pairwise BFS distance among seeds >= n/k (approximately)
- `bfs_seed` prefix constant in source equals `"BFS_SEED_"` exactly; hard-coded expected value for `bfs_seed(0)` asserted
- `rebalance_failed_plan_still_bounded`: construct a graph where rebalance fails (e.g., highly constrained topology with `balance_post_iters=0`); assert all districts non-empty and no district exceeds `2 × ideal_pop`

### L1 (integration, synthetic data)

- 4x4 grid k=4: produces a valid 4-partition in O(n log n) time
- Population balance: all districts within `balance_tolerance` of ideal after rebalance
- Determinism: two runs with same `base_seed` give identical assignment
- Contiguity: BFS from any tract in each district reaches all tracts in that district

### L2 (`#[ignore]`, real data)

- NC 2020 k=14: BFS Growth EC vs single METIS call. Expected: BFS Growth has higher EC than METIS (METIS explicitly minimizes EC; BFS Growth does not), but BFS Growth is faster.
- VT 2020 k=1: trivial single-district plan, rebalance is a no-op.

---

## 8. Open questions (deferred)

1. Should seed selection use population-weighted sampling instead of geographic spread for high-density states (NYC, LA)?
2. Can BFS Growth be used as warm-start for MCMC chains? (Yes — better than pure METIS for Forest ReCom.)

## 9. Design decisions

**Priority function** (formerly Q3): The deficit-only priority function `|ideal_pop - current_pop[district]|` is intentionally simple. In high-density states (NY, CA, IL), seed quality matters more than the growth priority function; Phase 2 will evaluate a composite `w_deficit * deficit + w_dist * distance_to_seed` priority. For now, deficit-only is the stable default.

---

## References

- `karypis1998`: Karypis, G., & Kumar, V. (1998). "A fast and high quality multilevel scheme for partitioning irregular graphs." SIAM Journal on Scientific Computing 20(1), 359–392.
