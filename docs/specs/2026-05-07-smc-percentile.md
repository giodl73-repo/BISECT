# Spec: SmcPercentile — Single-Plan Selection from SMC Weighted Ensemble

**Status**: Proposed (R1 reviewed, P1 fixes applied)
**Date**: 2026-05-07
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 4/4 → avg 3.25/4
**Extends**: `SeedCompositor` (new `SmcPercentile` variant); depends on `redist-smc` crate
**Related**: G.7 (SMC paper), docs/specs/2026-05-07-smc-redistricting.md (SMC spec)

---

## Overview

The SMC crate (`redist ensemble --method smc`) produces a **weighted ensemble** of N plans — it cannot be used directly via the three-layer compositor which expects a single plan output. SmcPercentile bridges this gap: it runs SMC internally and selects one plan from the weighted ensemble at the p-th quantile of the weighted EC distribution.

This closes the SMC→compositor loop and gives practitioners a way to say "run SMC and return the median-weight plan" or "run SMC and return the minimum-EC plan among its weighted sample."

---

## Algorithm

```
SmcPercentile(adj, pop, k, n_particles, p, base_seed):
  // 1. Run full SMC
  result = run_smc(adj, pop, k, SmcConfig {
    n_particles,
    resample_threshold: 0.5,
    pop_tolerance: 0.005,
    base_seed,
  })

  // 2. Compute EC for each particle
  ec_plans: Vec<(f64, usize, Vec<u32>)> = result.plans.iter().enumerate()
    .map(|(i, plan)| (count_ec(plan, adj), i, plan.clone()))
    .collect()

  // 3. Sort by EC ascending; use importance weights for weighted quantile
  // Weighted p-th quantile: find the particle at cumulative weight = p
  // Sort by EC, then find smallest EC value where cumulative_weight(all plans with EC <= this) >= p
  ec_plans.sort_by (ec ASC, particle_idx ASC)   // secondary sort for determinism

  cumulative_weight = 0.0
  selected_idx = 0
  for (i, (ec, orig_idx, plan)) in ec_plans.iter().enumerate():
    cumulative_weight += result.weights[orig_idx]
    if cumulative_weight >= p:
      selected_idx = orig_idx
      break

  return ec_plans[selected_idx].plan
```

**p=0.0**: uses the **weighted 0th quantile** (not the absolute minimum EC): the algorithm finds the first particle in EC-sorted order where cumulative weight ≥ 0.0, which is always the minimum-EC particle with positive weight. If the minimum-EC particle has weight exactly zero (possible after SMC resampling where that particle was not selected), the weighted 0th quantile skips it and returns the lowest-EC particle with positive weight. This is the correct behavior for a weighted quantile — plans with zero weight are not part of the effective ensemble.
**p=0.5**: returns the median-weight plan (the plan at the 50th percentile of the weighted EC distribution).
**p=1.0**: returns the maximum-EC plan.

**Weighted vs uniform quantile**: the weighted quantile differs from the uniform quantile — plans with high weight count more toward the cumulative. A plan drawn from the uniform distribution over all plans would use a uniform quantile; the weighted quantile accounts for SMC's importance weights. SmcPercentile is the only compositor mode that selects from a **calibrated** distribution — the SMC weights ensure the selection is a valid quantile of the true uniform distribution over plans.

---

## Compositor integration

```rust
SeedCompositor::SmcPercentile {
    n_particles: usize,   // SMC particle count (default: 5000)
    p: f64,               // weighted EC quantile (default: 0.0 = minimum-EC plan)
}
```

`seed_count()` = n_particles. `is_single()` = false.

**CLI**: `--search smc-percentile --particles 5000 --percentile 0.0`

**Advanced flag**: `--smc-resample-threshold` (default: 0.5). When SMC's effective sample size drops below this fraction × n_particles, resampling occurs. For states with irregular geometry or many protected VRA districts, lower values (0.3) reduce premature collapse. The default 0.5 is appropriate for most states.

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: smc-percentile
  particles: 5000
  percentile: 0.0
```

---

## Relationship to other search modes

| Mode | Distribution | Selection | Particles | Cost |
|------|-------------|-----------|-----------|------|
| `convergence` | Unknown (METIS seeds) | Min EC | N seeds | O(N × METIS) |
| `percentile` | Unknown (METIS seeds) | p-th EC | N seeds | O(N × METIS) |
| `smc-percentile` | **Calibrated (uniform)** | p-th weighted EC | N particles | O(N × SMC) |
| `forest-recom` | ≈Uniform (MH) | p-th EC | T steps | O(T × FR) |

SmcPercentile is the only mode that selects from a calibrated distribution. The SMC weights ensure the selection is a valid quantile of the true uniform distribution over plans, unlike METIS-seeded modes whose sample distribution is unknown.

---

## Seeding

SmcPercentile uses the SMC seeding formula directly (no additional seed derivation needed — SMC's base_seed fully determines the run):
```
smc_base_seed = SHA-256("SMCP_RUN_" || base_seed:u64le) -> u64le
```

The `"SMCP_RUN_"` prefix distinguishes SmcPercentile runs from standalone SMC runs with the same base_seed. Any change to the selection semantics (e.g., switching from weighted to uniform quantile) must change this prefix. Old and new manifests are distinguishable; silent seed compatibility across algorithm versions is prevented.

**Enforcement**: a test asserts that the prefix constant in source equals `"SMCP_RUN_"` and that the derived seed for a known `base_seed` equals a hard-coded expected value — this test will fail if the prefix or formula is silently changed without updating the version string.

---

## Audit chain

```json
"search": "smc-percentile",
"n_particles": 5000,
"percentile": 0.0,
"base_seed": 12345678,
"smc_base_seed": 987654321,
"smc_resample_count": 3,
"smc_ess_min": 1203.4,
"selected_particle_idx": 2847,
"selected_particle_ec": 2193,
"selected_particle_weight": 0.000847,
"smc_seed_formula": "SHA-256('SMCP_RUN_' || base_seed:u64le) -> u64le"
```

`selected_particle_idx` is the index in the SMC result `plans` array (0-based). An auditor can re-run SMC with `smc_base_seed`, sort by (EC ASC, idx ASC), and confirm the plan at the cumulative-weight p-th quantile matches the submitted plan without needing to reproduce the full selection logic independently.

---

## Test invariants (L0)

- `p=0.0` returns the minimum-EC plan from the weighted sample (lowest EC among all particles with positive weight); L0 test: when one particle has EC=1 (minimum) and weight=0.0, and all others have EC=2 and weight=1/N, `p=0.0` returns an EC=2 plan (not EC=1)
- `p=1.0` returns the maximum-EC plan (ascending sort — tests both ends to catch inverted sort)
- Same `base_seed` → identical `selected_particle_idx` (deterministic)
- `selected_particle_weight > 0` (selected particle has positive weight)
- Weighted quantile: for uniform weights, `p=0.5` returns plan at rank `floor(0.5 × n_particles)` in EC order (matches simple median)
- `n_particles=1`: single plan returned regardless of p value
- `smc_base_seed` derived from `base_seed` via `"SMCP_RUN_"` prefix (hard-coded test vector)

---

## Test invariants (L1)

- 4-node path k=2, n_particles=100: returns valid 2-district plan
- All returned plans are contiguous and population-balanced (inherited from SMC invariants)
- Determinism: same base_seed → same `selected_particle_idx` across two independent runs
- `p=0.0` EC ≤ `p=1.0` EC (ascending weighted quantile)

---

## Test invariants (L2, #[ignore])

- NC 2020 k=14, n_particles=1000: the **median** EC (p=0.5) from `smc-percentile` is within ±5% of the median EC from a `forest-recom` p=0.5 run for 1000 steps. This distributional comparison (medians) is more stable than a point-estimate minimum comparison, since both methods sample from distributions over plans and minimum values depend heavily on sample size.
- Weighted vs uniform quantile comparison: for n_particles=1000 on NC, the `p=0.5` plan from weighted quantile differs from the uniform median in at most 3% of cases (weights are approximately uniform after many resamples)

---

## Open questions (deferred)

1. Should SmcPercentile expose `--smc-resample-threshold` and `--smc-pop-tolerance` to override SMC defaults?
2. Should the full SMC NDJSON output be written to an intermediate file for diagnostics alongside the selected plan?
3. Should `p=0.0` use exact minimum (find the single minimum-EC particle) or weighted minimum (find the particle minimising expected EC under the weight distribution)? Currently uses exact minimum.
