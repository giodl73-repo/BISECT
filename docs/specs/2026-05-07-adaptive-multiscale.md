# Spec: Adaptive Multi-scale MCMC — Self-Tuning Alpha via Robbins-Monro Approximation

**Status**: Proposed  
**Date**: 2026-05-07  
**Extends**: `redist-multiscale` crate (extends MultiScaleChain)  
**Related paper**: B.21  
**Depends on**: `redist-multiscale` (MultiScaleChain, HierarchyLevel)

---

## Overview

Adaptive Multi-scale MCMC extends MultiScaleChain by automatically tuning the coarse-move probability alpha using Robbins-Monro stochastic approximation. The user no longer needs to set alpha manually; the algorithm observes the coarse acceptance rate and adjusts alpha toward the target acceptance rate at each adaptation interval.

| Algorithm | `--search` value | Paper | Layer | Flag collision |
|-----------|-----------------|-------|-------|----------------|
| Adaptive Multi-scale MCMC | `multiscale-adaptive` | B.21 | Search | `--multiscale-steps`, `--ms-target-accept`, `--ms-adapt-interval` (distinct from all other flags) |

---

## Motivation

Multi-scale MCMC (G.11) requires the user to set alpha (P(coarse move per step)). The optimal alpha depends on the state graph, the district count k, and the county/tract population ratios — it must be tuned empirically for each state. For TX (k=38, 254 counties, 5265 tracts), alpha=0.3 works well; for IA (k=4, 99 counties, 825 tracts) alpha=0.15 is better.

Adaptive Multi-scale automatically tunes alpha using Robbins-Monro stochastic approximation (Robbins & Monro 1951): if the coarse acceptance rate exceeds the target (0.30), alpha increases; if below, alpha decreases. This converges to the optimal alpha without manual tuning.

The Robbins-Monro approach is appropriate here because: (1) the expected coarse acceptance rate is a monotone function of alpha (higher alpha → more coarse proposals → the chain spends more effort on harder proposals → acceptance rate adjusts), and (2) the step-size decay schedule gamma_t = gamma_0/sqrt(t) satisfies the classical conditions (Σγ_t = ∞, Σγ_t² < ∞) guaranteeing convergence to the root of E[accept_rate − target] = 0.

---

## Algorithm

```
AdaptiveMultiScaleChain(fine_adj, coarse_adj, fine_to_coarse, pop, k, pop_tolerance,
                        target_accept, initial_alpha, adapt_interval, total_steps, base_seed):

  alpha = initial_alpha   // default: 0.3
  gamma_0 = 0.1           // initial step size for Robbins-Monro
  coarse_accept_window = []  // sliding window of recent coarse acceptance rates

  for step in 1..=total_steps:
    // Standard multi-scale step (same as MultiScaleChain)
    u = step_seed(base_seed, step, chain=0)
    if u < alpha: coarse_move() else: fine_move()

    // Every adapt_interval steps: update alpha
    if step % adapt_interval == 0:
      t = step / adapt_interval          // adaptation round
      gamma_t = gamma_0 / sqrt(t)        // decaying step size
      recent_accept = mean(coarse_accept_window)
      alpha = clip(alpha + gamma_t * (recent_accept - target_accept), 0.05, 0.95)
      coarse_accept_window.clear()

  // alpha_trace recorded for diagnostics
```

The Robbins-Monro conditions: gamma_t = gamma_0/sqrt(t) satisfies Σγ_t = ∞ and Σγ_t² < ∞, guaranteeing convergence to the root of E[accept_rate − target] = 0 (i.e., the optimal alpha).

**Coarse balance tolerance**: coarse_tol = 3 × pop_tolerance (matches MultiScaleChain default of 2–3×; the adaptive variant uses 3× to avoid over-rejection during early adaptation when alpha may be poorly calibrated).

---

## Seeding

Same as MultiScaleChain (MSC_STEP_ prefix) — adaptation adds no new randomness:

```
step_seed(base_seed, step, chain_idx) =
  SHA-256("MSC_STEP_" || step:u64le || "_" || chain_idx:u32le || "_" || base_seed:u64le) → u64le
```

The alpha adaptation is a deterministic function of the observed acceptance rates, which are themselves determined by the step seeds. Full reproducibility is maintained: same base_seed → same alpha_trace → same final plan.

**Implementation note**: the alpha draw always consumes one RNG value before the chain step, regardless of whether alpha is 0 or 1. The alpha draw is part of the canonical step computation and must not be skipped as a shortcut. This matches the MultiScaleChain seeding contract.

---

## Data model

```rust
pub struct AdaptiveMultiScaleConfig {
    pub total_steps: usize,
    pub target_accept: f64,    // target coarse acceptance rate (default: 0.30)
    pub initial_alpha: f64,    // starting alpha (default: 0.30)
    pub adapt_interval: usize, // steps between alpha updates (default: 50)
    pub gamma_0: f64,          // initial Robbins-Monro step size (default: 0.10)
    pub pop_tolerance: f64,
    pub coarse_tol: f64,       // = 3 × pop_tolerance (same as MultiScale)
    pub p: f64,
    pub base_seed: u64,
    pub chain_idx: u32,
}

pub struct AdaptiveMultiScaleChain {
    /// Underlying multi-scale chain (fine + coarse levels, assignment, rebalance logic)
    pub inner: MultiScaleChain,
    /// Current alpha (updated every adapt_interval steps)
    pub alpha: f64,
    /// Sliding window of coarse accept/reject outcomes since last adaptation
    pub coarse_accept_window: Vec<bool>,
    /// Alpha value recorded after each adaptation round
    pub alpha_trace: Vec<f64>,
    pub config: AdaptiveMultiScaleConfig,
}
```

AdaptiveMultiScaleChain wraps MultiScaleChain rather than duplicating its fields. All projection, rebalance, and HierarchyLevel logic is inherited from MultiScaleChain unchanged.

---

## Compositor integration

```rust
SeedCompositor::MultiScaleAdaptive {
    total_steps: usize,
    p: f64,
    target_accept: f64,    // default: 0.30
    initial_alpha: f64,    // default: 0.30
    adapt_interval: usize, // default: 50
}
```

**CLI**:
```bash
redist state --state TX --year 2020 \
  --search multiscale-adaptive \
  --multiscale-steps 2000 \
  --ms-target-accept 0.30 \
  --ms-adapt-interval 50 \
  --percentile 0.0
```

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: multiscale-adaptive
  multiscale_steps: 2000
  ms_target_accept: 0.30
  ms_adapt_interval: 50
  percentile: 0.0
```

**Data requirement**: same as MultiScaleChain — both adjacency files must exist:
- `data/{year}/{state}/{state}_adjacency_{year}.pkl` — tract-level (fine)
- `data/{year}/{state}/{state}_bg_adjacency_{year}.pkl` — block-group-level (coarse)

If the block-group adjacency file is missing, the CLI must error with the same instructional message as MultiScaleChain:
```
Error: multi-scale MCMC requires block-group adjacency for {state} {year}.
Run: redist fetch --year {year} --resolution block_group
```

---

## Audit chain

Every run with `--search multiscale-adaptive` records in `runs/{label}/{year}/index.json`:

```json
"search": "multiscale-adaptive",
"total_steps": 2000,
"target_accept": 0.30,
"initial_alpha": 0.30,
"final_alpha": 0.28,
"adapt_interval": 50,
"alpha_trace": [0.30, 0.31, 0.29, 0.28, ...],
"fine_acceptance_rate": 0.73,
"coarse_acceptance_rate": 0.29,
"selected_step": 1204,
"step_seed_formula": "SHA-256('MSC_STEP_' || step:u64le || '_' || chain_idx:u32le || '_' || base_seed:u64le)"
```

`alpha_trace` records alpha after each adaptation round (length = total_steps / adapt_interval). Enables diagnosis of convergence speed and final alpha value without re-running.

`final_alpha` is the last entry of `alpha_trace`. `fine_acceptance_rate` and `coarse_acceptance_rate` are computed over all steps (not just the last adaptation window). `selected_step` is the 0-based index into the EC-sorted list of accepted plans, matching the MultiScaleChain convention.

This ensures `redist label-verify` can confirm the search parameters and seed derivation for any submitted plan. A verifier can independently reproduce the alpha_trace from base_seed by replaying the step seeds and acceptance events.

---

## Test invariants (L0)

- `alpha_trace` length = `floor(total_steps / adapt_interval)`
- `alpha_trace` values strictly in [0.05, 0.95] (clip enforced at every adaptation step)
- If `coarse_accept == target_accept` exactly: alpha unchanged (Robbins-Monro update = 0)
- If `coarse_accept > target_accept`: alpha increases (positive update)
- If `coarse_accept < target_accept`: alpha decreases (negative update)
- Deterministic: same `base_seed` → same `alpha_trace` (acceptance events are determined by step seeds)
- `final_alpha` convergence: |final_alpha − target_accept| < |initial_alpha − target_accept| after 500+ steps (monotone approach not guaranteed but expected in expectation)
- `coarse_tol == 3.0 × pop_tolerance` when constructed with default parameters
- Step seed prefix version-lock: assert prefix constant in source equals `"MSC_STEP_"` and `step_seed(0, 0, 0)` equals the hard-coded expected value from MultiScaleChain (shared formula — must not diverge)

---

## Test invariants (L1)

- **Synthetic 2-county 4×4 grid**, adapt_interval=10, T=200: alpha converges to within 0.10 of target_accept; all accepted plans are valid (contiguous, balanced at fine level)
- **coarse_tol**: construct with `pop_tolerance=0.005`; assert `coarse_tol == 0.015` (3× rule)
- **gamma_0=0.1**: per-round alpha change ≤ 0.1 for all adaptation rounds (step sizes are bounded by gamma_0 / sqrt(1) = gamma_0 in the first round and decrease thereafter)
- **Clip enforcement**: if coarse acceptance rate is 1.0 (all accepted), alpha never exceeds 0.95; if 0.0 (none accepted), alpha never falls below 0.05
- **alpha_trace length**: for T=200, adapt_interval=10, `len(alpha_trace) == 20` exactly
- **Inherited seeding contract**: `step_seed` produces the same value as MultiScaleChain for the same inputs (shared formula, same prefix)

---

## Test invariants (L2, #[ignore])

- **NC 2020 k=14, T=2000**: final_alpha converges and |final_alpha − 0.30| < 0.10
- **TX 2020 k=38, T=5000**: final_alpha differs from NC (TX benefits from higher alpha due to more counties); specifically final_alpha(TX) > final_alpha(NC) with high probability
- **Mixing benefit**: autocorrelation of EC at lag 100 for multiscale-adaptive is not worse than for `--search multiscale --multiscale-alpha <final_alpha>` using the same final alpha (adaptive version should not be slower than its converged fixed-alpha equivalent)

---

## Relationship to MultiScaleChain

AdaptiveMultiScaleChain extends MultiScaleChain by adding:

1. An alpha state variable (updated every adapt_interval steps via Robbins-Monro)
2. A coarse acceptance window (Vec<bool> tracking per-step outcomes since last adaptation)
3. An alpha_trace (Vec<f64> for audit, length = total_steps / adapt_interval)

All other behavior is identical: same seeding formula (MSC_STEP_ prefix), same rebalance contract (§2.5 of the MultiScaleChain spec), same projection logic, same two-level HierarchyLevel structure. The adaptation loop is a thin wrapper around MultiScaleChain::step().

---

## Open questions (deferred)

1. Should adaptation stop after convergence (frozen alpha) or continue throughout? Frozen is more reproducible; continued is more responsive to non-stationarity in the chain.
2. Should target_accept differ per state based on the county/tract ratio? Current spec uses a fixed default of 0.30 for all states.
3. Can alpha_trace be compressed to save audit storage for long runs? For T=10,000 and adapt_interval=50, alpha_trace has 200 entries — modest, but compression may matter for ensemble runs.
