# Spec: Ensemble Search Algorithms — G series extensions

**Status**: Proposed  
**Date**: 2026-05-06  
**Extends**: `redist-ensemble` crate, `SeedCompositor` three-layer compositor  
**Related papers**: G.6 (Short-Burst), G.7 (SMC), G.8 (Flip), B.19 (Simulated Annealing)

---

## Overview

The `redist-ensemble` crate currently implements ReCom (Recombination). This spec adds four new ensemble/search algorithms, each accessible as a `SeedCompositor` variant and a `--search` CLI flag:

| Algorithm | `--search` value | Paper | Crate | Status |
|-----------|-----------------|-------|-------|--------|
| Short-Burst | `short-burst` | G.6 | `redist-ensemble` | Proposed |
| SMC | `smc` | G.7 | `redist-smc` (new) | Proposed |
| Flip proposals | `flip` | G.8 | `redist-ensemble` | Proposed |
| Simulated Annealing | `simulated-annealing` | B.19 | `redist-apportion` | Proposed |

All four are **compositor Layer 3 (Search)** additions — orthogonal to structure and weights.

---

## 1. Short-Burst (G.6)

**Concept**: Run many short ReCom chains of length ℓ, pick the best plan from each burst by some objective. Proven to outperform long-chain ReCom for optimization targets (Cannon, Duchin et al. 2022).

**Algorithm**:
```
ShortBurst(burst_length, n_bursts, objective, p):
  best_plans = []
  for burst in 0..n_bursts:
    chain = RecomChain::new(initial=current_plan, seed=chain_seed(base, burst))
    plans = run for burst_length steps
    best_plans.push(plans.min_by(objective))
  sort best_plans by objective
  return best_plans[floor(p * n_bursts)]
```

**Compositor integration**:
```rust
SeedCompositor::ShortBurst {
    burst_length: usize,   // steps per burst (default: 20)
    n_bursts: usize,       // number of bursts (default: 50)
    p: f64,                // percentile of burst winners to return (default: 0.0 = minimum)
}
```

**CLI**: `--search short-burst --burst-length 20 --n-bursts 50 --percentile 0.0`

**YAML**: `search: short-burst, burst_length: 20, n_bursts: 50, percentile: 0.0`

**Objective function**: edge cut fraction (default), or PP score if `--objective polsby-popper`

**Research question for G.6**: Does Short-Burst find lower-EC plans than ConvergenceSweep for the same total step count? Expected: yes for small maps; similar for large maps where METIS already finds the global minimum.

---

## 2. Sequential Monte Carlo / SMC (G.7)

**Concept**: The R `redist` package's main algorithm (Imai, Kane, Fifield 2020). Generates a *weighted* sample of valid plans via sequential importance resampling. Produces properly calibrated posterior distributions — more statistically correct than MCMC for inference.

**New crate**: `redist-smc` (separate from `redist-ensemble` because the algorithm is fundamentally different)

**High-level API**:
```rust
pub fn run_smc(
    adj: Vec<Vec<u32>>,
    pop: Vec<i64>,
    k: u32,
    pop_tolerance: f64,
    n_particles: usize,   // number of plans in the population (default: 5000)
    resample_threshold: f64, // ESS threshold for resampling (default: 0.5)
    base_seed: u64,
) -> SmcResult {
    // Returns weighted sample of k-district plans
    // Each plan has an associated importance weight
}
```

**CLI**: `redist ensemble --method smc --particles 5000 --state NC`

**Compositor integration**: SMC is not a `SeedCompositor` variant — it's a separate ensemble method that runs as a standalone `redist ensemble --method smc` call, not as part of `redist state --search smc`. The output JSON is compatible with `EnsembleResult`.

**Research question for G.7**: Do SMC and ReCom give different percentile estimates for the bisection plan? Expected: SMC may place the plan at a higher percentile (more representative) because SMC covers the full feasible space better than ReCom chains.

---

## 3. Flip Proposals (G.8)

**Concept**: Instead of merging two districts and resplitting (ReCom), flip individual tracts from one district to an adjacent district. Simpler, faster per step, but much slower to mix across the plan space. Good for local sensitivity analysis.

**Algorithm**:
```
FlipChain step:
  1. Pick random boundary tract (tract adjacent to a different district)
  2. Pick which neighbor district to flip it to
  3. Check population balance + contiguity
  4. Accept/reject (Metropolis-Hastings or always-accept)
```

**Compositor integration**:
```rust
SeedCompositor::Flip {
    steps: usize,       // number of flip steps (default: 10000)
    p: f64,             // percentile of visited plans to return (default: 0.0)
    accept: FlipAccept, // AlwaysAccept | MetropolisHastings { beta: f64 }
}
```

**CLI**: `--search flip --steps 10000 --percentile 0.5`

**Implementation**: Add `FlipChain` struct to `redist-ensemble/src/flip.rs`. Much simpler than Wilson UST — no spanning tree required, just adjacency check.

**Research question for G.8**: For small local adjustments, is Flip faster than ReCom? Expected: yes per step (O(1) vs O(n/k)), but many more steps needed for equivalent mixing.

---

## 4. Simulated Annealing (B.19)

**Concept**: Start from any valid plan (e.g., from METIS), propose random moves (flip or merge-resplit), accept worse plans with decreasing probability `exp(-ΔEC / T)` where T is temperature. Unlike METIS which greedily minimises, SA can escape local optima.

**Algorithm**:
```
SimulatedAnnealing(initial_plan, cooling_schedule, n_steps):
  plan = initial_plan
  for step in 0..n_steps:
    T = cooling_schedule(step)
    proposed = random_flip_or_merge_resplit(plan)
    if valid(proposed):
      delta_EC = EC(proposed) - EC(plan)
      if delta_EC < 0 or random() < exp(-delta_EC / T):
        plan = proposed
  return plan
```

**Cooling schedules**: Linear, exponential, step. Default: exponential with T_0=1.0, T_final=0.01.

**Compositor integration** (Structure layer, not Search — SA replaces METIS at each bisection node):
```rust
SplitStrategy::SimulatedAnnealing {
    n_steps: usize,           // annealing steps per split (default: 1000)
    cooling: CoolingSchedule, // Exponential { t0, t_final } | Linear | Step
}
```

**CLI**: `--structure simulated-annealing --sa-steps 1000`

**Note**: SA is a structure-layer variant (replaces METIS), while the others above are search-layer variants (modify how plans are selected). This is the correct compositor separation.

**Research question for B.19**: Does SA find more compact plans than METIS for the same compute budget? Expected: yes for small maps; competitive for medium; METIS wins for large maps due to the multilevel coarsening advantage.

---

## Implementation priority

1. **Flip** (1 week) — simplest, builds on existing `redist-ensemble` infrastructure
2. **Short-Burst** (1 week) — builds directly on `RecomChain`
3. **Simulated Annealing** (2 weeks) — new structure mode, needs `split_subgraph_sa()` in `bisection_runner.rs`
4. **SMC** (1 month) — new crate, complex algorithm, high statistical value

---

## CLI / YAML surface (complete picture after all four)

```bash
# Short-Burst: 50 bursts of 20 ReCom steps, return minimum EC plan
redist state --state NC --search short-burst --burst-length 20 --n-bursts 50

# Flip: 10K flip steps, median plan
redist state --state NC --search flip --steps 10000 --percentile 0.5

# Simulated Annealing bisection
redist state --state NC --structure simulated-annealing --sa-steps 1000

# SMC ensemble (standalone, not via redist state)
redist ensemble --method smc --particles 5000 --state NC --steps 1

# Full compositor with Short-Burst search
redist build official_2020 --year 2020 --search short-burst --burst-length 20 --n-bursts 100
```

YAML:
```yaml
algorithm:
  structure: prime-factor       # bisection tree
  weights: county               # edge weights
  search: short-burst           # search strategy
  burst_length: 20
  n_bursts: 100
  percentile: 0.0               # return minimum from bursts
```

---

## Connection to existing work

- All new search modes inherit the `--percentile` / `--ensemble-steps` infrastructure from H.0/H.1
- All produce `EnsembleResult`-compatible JSON for diagnostics (G.4)
- Short-Burst answers the "which chain length is optimal?" question that B.7 raised
- SMC provides the gold standard for ensemble comparison that G.1's GerryChain estimates approximated
- The G series becomes: G.0 (methodology) + G.1 (ReCom) + G.2/3 (positions) + G.4 (diagnostics) + G.5 (mixing) + G.6 (Short-Burst) + G.7 (SMC) + G.8 (Flip) — a complete ensemble comparison portfolio
