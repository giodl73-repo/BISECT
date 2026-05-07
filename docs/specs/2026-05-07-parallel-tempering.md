# Spec: Parallel Tempering — Multi-Chain Replica Exchange MCMC for Redistricting

**Status**: Proposed (R1 reviewed, P1 fixes applied)  
**Date**: 2026-05-07  
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 4/4, SURVEY 3/4, COVENANT 3/4 → avg 3.25/4  
**Extends**: `redist-ensemble` crate (new `parallel_tempering.rs`)  
**Related paper**: B.20  
**Depends on**: `redist-ensemble::forest_recom` (ForestRecomChain), `redist-ensemble::recom` (RecomChain)

---

## Overview

Single-chain MCMC (ReCom, Forest ReCom, Merge-Split) can get trapped in local optima: the chain explores a region of plan space thoroughly but cannot escape to qualitatively different regions without passing through high-EC "barriers." For large-k states (TX k=38) or states with complex geography (NC mountain/coast split), the cold chain may mix too slowly to see the full feasible plan space.

Parallel Tempering (Swendsen & Wang 1986; Geyer 1991) runs `n_replicas` chains simultaneously at different "temperatures" — here implemented as different balance tolerances `ε_0 < ε_1 < ... < ε_{n-1}`. Hot chains (high ε) accept almost any balanced plan and mix quickly; cold chains (low ε = target tolerance) mix slowly but produce distribution-correct samples. After every `swap_interval` steps, adjacent chains propose to exchange their current plans via a Metropolis criterion. This passes global diversity from hot chains down to the cold chain.

**Relationship to existing algorithms**:

| Property | ForestRecom | MergeSplit | ParallelTempering |
|----------|-------------|------------|-------------------|
| Chains | 1 | 1 | n_replicas |
| Tolerance | fixed | fixed | geometric ladder |
| Mixing speed | slow (cold) | medium | cold chain assisted by hot chains |
| Global exploration | limited | limited | high (via swaps) |
| Distributional target | uniform (exact) | uniform (approx) | uniform at cold tolerance |
| Extra cost | det ratio | cut-count ratio | n_replicas × per-step cost |

**Flag disambiguation**: `--pt-replicas`, `--pt-swap-interval`, `--pt-cold-tol`, `--pt-hot-tol` are distinct from `--merge-split-steps`, `--forest-steps`, `--burst-length`, `--flip-steps`, and `--ensemble-steps`.

---

## 1. Temperature ladder design

The geometric ladder `ε_i = ε_cold × (ε_hot/ε_cold)^(i/(N-1))` ensures equal expected swap acceptance rates between adjacent replicas (approximately 23% for typical redistricting chains, matching the empirical target: ~23% swap acceptance is optimal for Gaussian continuous systems (Kone & Kofke 2005); this may differ for discrete combinatorial chains. We use this as an empirical starting point, not a derived guarantee).

- `tolerances[0] = cold_tolerance` (cold chain — distribution-correct)
- `tolerances[n-1] = hot_tolerance` (hot chain — rapidly mixing)
- All intermediate tolerances strictly increasing

**Recommended defaults**: `n_replicas=4`, `cold_tolerance=0.005`, `hot_tolerance=0.05`, `swap_interval=10`.

**Parameter scaling table**:

| State size | n_replicas | cold_tol | hot_tol | swap_interval | Runtime (16 cores) |
|---|---|---|---|---|---|
| Small (k<=5) | 2 | 0.005 | 0.02 | 5 | <1 min |
| Medium (k=8-14) | 4 | 0.005 | 0.05 | 10 | 5-15 min |
| Large (k>=30) | 6 | 0.005 | 0.10 | 20 | 30-90 min |

**Note**: Wall-clock estimates assume serial chain execution (one chain per step). Parallelism via Rayon (one thread per replica) is a Phase 2 improvement that would reduce wall time by approximately 1/n_replicas; see Open Question 5.

---

## 2. Algorithm

```
ParallelTempering(n_replicas, cold_tolerance, hot_tolerance, swap_interval, steps, base_seed):

  // Geometric temperature ladder
  tolerances[i] = cold_tolerance * (hot_tolerance/cold_tolerance)^(i/(n_replicas-1))
  // -> tolerances[0] = cold_tolerance, tolerances[n_replicas-1] = hot_tolerance

  // Initialise one ForestRecomChain per replica, all starting from the same METIS plan
  initial_plan = run_all_splits(adjacency, pop, k, cold_tolerance, seed=base_seed)
  chains[i] = ForestRecomChain(adj, pop, initial_plan, k, tolerance=tolerances[i])
                for i in 0..n_replicas

  accepted_cold = [(EC(initial_plan), step=0, initial_plan.clone())]

  for step in 1..=steps:

    // 1. Each chain takes one step independently
    for i in 0..n_replicas:
      r_seed = replica_seed(base_seed, i, step)
      rng_fwd = SmallRng::seed_from_u64(SHA-256("PT_FWD_" || r_seed:u64le) -> u64le)
      rng_rev = SmallRng::seed_from_u64(SHA-256("PT_REV_" || r_seed:u64le) -> u64le)
      chains[i].step(rng_fwd, rng_rev)

    // 2. After every swap_interval steps: propose replica exchange for adjacent pairs
    if step % swap_interval == 0:
      for i in 0..n_replicas-1:
        s_seed = swap_seed(base_seed, step, i)
        rng_swap = SmallRng::seed_from_u64(s_seed)
        EC_i = count_edge_cuts(chains[i].assignment)
        EC_j = count_edge_cuts(chains[i+1].assignment)
        beta_i = 1.0 / tolerances[i]
        beta_j = 1.0 / tolerances[i+1]
        // Metropolis criterion for swap
        log_ratio = (beta_i - beta_j) * (EC_i - EC_j)
        if rng_swap.gen::<f64>().ln() < log_ratio.min(0.0):
          swap(chains[i].assignment, chains[i+1].assignment)

    // 3. Record cold chain (replica 0) plan after step
    accepted_cold.push((EC(chains[0].assignment), step, chains[0].assignment.clone()))

  sort accepted_cold by (EC ASC, step ASC)
  return accepted_cold[floor(p * accepted_cold.len())]
```

**Note on cold chain recording**: Every step is recorded, not only steps where the cold chain's internal Forest ReCom proposal was accepted. The cold chain advances (or stays) each step regardless; the PT search collects all resulting plans and selects by percentile.

---

## 3. Seeding specification

All seeds derived from `base_seed` via SHA-256 with domain-separated prefixes:

```
replica_seed(base_seed, replica, step) =
  SHA-256("PT_REPLICA_" || replica:u32le || "_" || step:u64le || "_" || base_seed:u64le)
  -> least-significant 64 bits

swap_seed(base_seed, step, pair) =
  SHA-256("PT_SWAP_" || pair:u32le || "_" || step:u64le || "_" || base_seed:u64le)
  -> least-significant 64 bits

rng_fwd = SmallRng::seed_from_u64(SHA-256("PT_FWD_" || replica_seed:u64le) -> u64le)
rng_rev = SmallRng::seed_from_u64(SHA-256("PT_REV_" || replica_seed:u64le) -> u64le)
```

The four prefixes `"PT_REPLICA_"`, `"PT_SWAP_"`, `"PT_FWD_"`, `"PT_REV_"` are all distinct and embed algorithm identity. Any change to which operations use each stream must change the corresponding prefix. A test asserts that each prefix constant in source equals its expected string and that `replica_seed(0, 0, 0)` and `swap_seed(0, 0, 0)` equal hard-coded expected values, preventing silent seed-compatibility drift.

**Key property**: `replica_seed(b, r, s) != swap_seed(b, s, r)` for all (b, r, s) — the "PT_REPLICA_" prefix vs "PT_SWAP_" prefix guarantees distinctness even when `replica` and `pair` indices coincide.

---

## 4. Rust struct and API

**`ParallelTemperingChain` struct** (`redist-ensemble/src/parallel_tempering.rs`):

```rust
pub struct ParallelTemperingChain {
    pub adj: Vec<Vec<u32>>,
    pub pop: Vec<i64>,
    pub assignments: Vec<Vec<u32>>,   // one per replica; index 0 = cold chain
    pub k: u32,
    pub tolerances: Vec<f64>,         // length n_replicas, strictly increasing
    pub swap_interval: usize,
    pub steps_taken: u64,
    pub swap_attempts: u64,
    pub swap_accepted: u64,
    pub cold_chain_records: Vec<(u64, Vec<u32>)>,  // (EC, assignment) at each step
}

impl ParallelTemperingChain {
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        initial_assignment: Vec<u32>,
        k: u32,
        tolerances: Vec<f64>,
        swap_interval: usize,
    ) -> Self

    /// Advance all replicas by one step and process swap proposals if due.
    /// step: current step index (1-based, caller increments)
    /// base_seed: passed through to replica_seed and swap_seed derivation
    pub fn step(&mut self, step: u64, base_seed: u64)

    pub fn swap_acceptance_rate(&self) -> f64
    pub fn cold_chain_ec_mean(&self) -> f64
    pub fn select_plan(&self, p: f64) -> Vec<u32>  // percentile p of cold_chain_records by EC
}
```

---

## 5. Compositor integration

```rust
SeedCompositor::ParallelTempering {
    n_replicas: usize,    // number of chains (default: 4)
    swap_interval: usize, // steps between swap proposals (default: 10)
    cold_tolerance: f64,  // balance tolerance for cold chain (default: 0.005)
    hot_tolerance: f64,   // balance tolerance for hot chain (default: 0.05)
    p: f64,               // percentile of cold chain EC distribution (default: 0.0)
}
```

The compositor builds the geometric ladder from `cold_tolerance` and `hot_tolerance`, runs `steps` iterations (from `SeedCompositor` run config), collects all cold-chain plans, sorts by EC ascending, and returns the plan at index `floor(p * cold_chain_records.len())`.

---

## 6. CLI

```bash
--search parallel-tempering --pt-replicas 4 --pt-swap-interval 10 --pt-cold-tol 0.005 --pt-hot-tol 0.05 --percentile 0.0
```

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: parallel-tempering
  pt_replicas: 4
  pt_swap_interval: 10
  pt_cold_tolerance: 0.005
  pt_hot_tolerance: 0.05
  percentile: 0.0
```

---

## 7. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"search": "parallel-tempering",
"n_replicas": 4,
"swap_interval": 10,
"cold_tolerance": 0.005,
"hot_tolerance": 0.05,
"percentile": 0.0,
"base_seed": 12345678,
"steps": 1000,
"swap_acceptance_rate": 0.21,
"cold_chain_acceptance_rate": 0.44,
"selected_step": 847,
"replica_seed_formula": "SHA-256('PT_REPLICA_' || replica:u32le || '_' || step:u64le || '_' || base_seed:u64le)",
"swap_seed_formula": "SHA-256('PT_SWAP_' || pair:u32le || '_' || step:u64le || '_' || base_seed:u64le)"
```

`selected_step` is the step index (0-based, from the cold chain's record list sorted by EC ascending) at rank `floor(p * (steps+1))`. An auditor who knows `base_seed` can re-derive all seeds, replay the full chain, and verify that the plan at `selected_step` matches the submitted plan.

**Note**: `selected_step` is a **rank** in the EC-sorted cold chain record list (0 = minimum EC), not a temporal step index. An auditor who re-runs the chain collects all cold-chain records, sorts by (EC ASC, step ASC), and confirms the plan at position `selected_step` matches the submitted plan.

`cold_chain_acceptance_rate` is the fraction of steps where the cold chain's internal Forest ReCom proposal was accepted (distinct from `swap_acceptance_rate`). Recording both rates allows detection of pathological conditions: a very low `cold_chain_acceptance_rate` with a normal `swap_acceptance_rate` suggests the cold tolerance is too tight for this state.

---

## 8. Test invariants

### L0 (inline unit tests)

- Swap acceptance rate in (0, 1): geometric ladder produces non-trivial swaps (not always-accept, not always-reject)
- Cold chain EC <= hot chain EC on average (cold chain has tighter tolerance = lower expected EC)
- Deterministic: same `base_seed` -> identical chain trajectories and swap decisions
- All `n_replicas` chains produce valid plans (contiguous, population-balanced) at every step
- `replica_seed` distinct from `swap_seed` for same (step, pair): prefix isolation guarantees no collision
- Temperature ladder: `tolerances[0] == cold_tolerance`, `tolerances[n-1] == hot_tolerance`, all strictly increasing
- `selected_step` is in [0, steps] (inclusive — step 0 is the initial plan)
- `p=0.0` returns minimum-EC cold chain plan; `p=1.0` returns maximum-EC cold chain plan (ascending sort, both ends tested to catch inverted sort)
- `n_replicas=1`: degenerate case (no swaps possible) — chain runs as single ForestRecomChain, no panics
- `replica_seed` prefix constant in source equals `"PT_REPLICA_"` exactly; hard-coded expected value for `replica_seed(0, 0, 0)` asserted
- `swap_seed` prefix constant in source equals `"PT_SWAP_"` exactly; hard-coded expected value for `swap_seed(0, 0, 0)` asserted

### L1 (integration, synthetic data)

- 4x4 grid, k=2, `n_replicas=2`, T=100 steps: both chains produce valid plans throughout
- Swap events occur: `swap_accepted > 0` after T=100 steps with geometric ladder on a well-connected grid
- Swap events bounded: `swap_attempts == floor(T / swap_interval)` exactly
- Cold chain acceptance rate > 0 (chain is not stalled)
- Determinism: two runs with same seed produce identical trajectories, swap decisions, and `selected_step`
- `cold_chain_records.len() == steps + 1` (initial plan + one record per step)

### L2 (#[ignore], real data)

- NC 2020, k=14, `n_replicas=4`, T=1000: cold chain minimum EC <= single-chain Forest ReCom minimum EC over the same step count (better global exploration due to swaps)
- Swap acceptance rate in [0.15, 0.35] (optimal range for geometric ladder — outside this range indicates misconfigured temperature spacing)
- NC 2020, k=14, `n_replicas=4`, T=1000: `cold_chain_acceptance_rate` in [0.10, 0.80] (sanity band — collapse to 0 indicates cold tolerance too tight; collapse to 1 indicates cold tolerance too loose)

---

## 9. Open questions (P2, deferred)

1. Should hot chains use standard ReCom (faster, higher acceptance per step) while only the cold chain uses Forest ReCom (distributional correctness)? Mixed-chain design complicates the swap MH criterion but may improve wall-clock performance significantly.
2. Should the swap criterion use EC directly or a plan-space distance metric (e.g., symmetric difference of district assignments)? EC is fast to compute; plan distance may better reflect thermodynamic "energy."
3. Adaptive replica count: add or remove replicas based on observed swap acceptance rates during the run? Would require dynamic ladder adjustment and complicates the audit chain.
4. Non-adjacent swaps: current spec only swaps adjacent pairs (i, i+1). Allowing non-adjacent swaps (e.g., randomly chosen pairs) may improve mixing but requires adjusting the MH criterion.
5. Parallel execution: chains at different temperatures are independent between swap steps — embarrassingly parallel. Should the spec mandate parallel execution (e.g., via `rayon`) or leave it to the caller?
