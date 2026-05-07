# Spec: Short-Burst Chain Type Extension — Composable Burst Optimization

**Status**: Proposed  
**Date**: 2026-05-07  
**Extends**: `SeedCompositor::ShortBurst` (G.6, already implemented)  
**Related paper**: G.12  
**Depends on**: Forest ReCom (#117 done), Merge-Split (#122 done)

---

## Overview

Short-Burst (G.6) runs `n_bursts` short ReCom chains of `burst_length` steps and keeps the **endpoint** of each burst. The underlying chain type is currently hardcoded to standard ReCom.

This spec adds two new `SeedCompositor` variants — `ShortBurstForest` and `ShortBurstMergeSplit` — that use Forest ReCom and Merge-Split respectively as the burst chain. The existing `ShortBurst` variant is unchanged.

**Key insight**: Short-Burst is a meta-algorithm orthogonal to the underlying Markov chain. Swapping the chain provides different distributional properties:

| Chain | Acceptance rate | Targets uniform? | Per-step cost |
|-------|----------------|------------------|---------------|
| StandardRecom (current default) | ~80% | No (approx) | O(m log m) |
| ForestRecom | ~40-60% | Yes (exact) | O(m² or m³) |
| MergeSplit | ~60-70% | Approx | O(m log m) |

Using ForestRecom or MergeSplit as the burst chain provides Short-Burst's compactness optimization while improving distributional correctness.

**Flag disambiguation**: Each variant has its own `--search` flag value and is a separate `SeedCompositor` enum variant. The existing `--search short-burst` is unchanged.

---

## 1. Architecture decision

**Separate variant per chain type** (not a `chain_type` field on the existing `ShortBurst`):

```rust
// Option A — new variant per chain type (chosen):
SeedCompositor::ShortBurstForest { burst_length, n_bursts, p }
SeedCompositor::ShortBurstMergeSplit { burst_length, n_bursts, p }

// Option B — chain_type field on existing ShortBurst (rejected):
SeedCompositor::ShortBurst { burst_length, n_bursts, p, chain_type: ChainType }
```

**Rationale for Option A**: avoids a nested enum (`ChainType` inside `SeedCompositor`), keeps the compositor flat, and allows each variant to have its own CLI flag set and documentation without overloading `--search short-burst`. The existing `--search short-burst` continues to use standard ReCom unchanged.

---

## 2. ShortBurstForest

**Concept**: Run `n_bursts` Forest ReCom chains of `burst_length` steps each. Keep the **endpoint** of each burst. Sort endpoints by edge-cut ascending; return the one at percentile `p`. Identical to ShortBurst's endpoint semantics — only the underlying chain differs.

**Compositor integration**:

```rust
SeedCompositor::ShortBurstForest {
    burst_length: usize,  // Forest ReCom steps per burst (default: 20)
    n_bursts: usize,      // number of bursts (default: 50)
    p: f64,               // percentile of endpoints to return (default: 0.0 = min EC endpoint)
}
```

**CLI**: `--search short-burst-forest --burst-length 20 --n-bursts 50 --percentile 0.0`

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: short-burst-forest
  burst_length: 20
  n_bursts: 50
  percentile: 0.0
```

**Algorithm**:
```
ShortBurstForest(burst_length, n_bursts, p, base_seed):
  current_plan = initial_METIS_plan
  burst_endpoints = []
  for burst in 0..n_bursts:
    seed_i = chain_seed(base_seed, burst, "SBF_CHAIN_")
    chain = ForestRecomChain::new(initial=current_plan, base_seed=seed_i)
    for step in 0..burst_length:
      fwd_seed = forward_seed(seed_i, step)
      rev_seed = reverse_seed(seed_i, step)
      chain.step(SmallRng::seed_from_u64(fwd_seed), SmallRng::seed_from_u64(rev_seed))
    endpoint = chain.current_plan()                  // ENDPOINT, not minimum
    burst_endpoints.push((EC(endpoint), endpoint))
    current_plan = endpoint                          // chain restarts from endpoint
  sort burst_endpoints by EC ASC
  return burst_endpoints[floor(p * n_bursts)].plan
```

**Parameter scaling guidance**:

| State size | Example states | burst_length | n_bursts |
|-----------|---------------|-------------|---------|
| Small (k<=5) | VT, WY, ND, AK | 10 | 20 |
| Medium (k=8-14) | WI, NC, MN | 20 | 50 |
| Large (k=20-38) | TX, FL, CA | 30 | 100 |

Forest ReCom's lower acceptance rate means fewer plans are accepted per burst step than standard ReCom. The burst_length defaults are retained from ShortBurst; practitioners may need longer bursts for equivalent plan diversity on very sparse graphs.

---

## 3. ShortBurstMergeSplit

**Concept**: Same meta-algorithm as ShortBurstForest, using MergeSplitChain as the burst chain instead.

**Compositor integration**:

```rust
SeedCompositor::ShortBurstMergeSplit {
    burst_length: usize,  // MergeSplit steps per burst (default: 20)
    n_bursts: usize,      // number of bursts (default: 50)
    p: f64,               // percentile of endpoints to return (default: 0.0 = min EC endpoint)
}
```

**CLI**: `--search short-burst-merge-split --burst-length 20 --n-bursts 50 --percentile 0.0`

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: short-burst-merge-split
  burst_length: 20
  n_bursts: 50
  percentile: 0.0
```

**Algorithm**: Identical to ShortBurstForest with `ForestRecomChain` replaced by `MergeSplitChain`. Each burst step calls `chain.step(&mut rng_forward, &mut rng_reverse)` with forward and reverse streams derived using the `"SBMS_CHAIN_"` prefix (see seeding section).

---

## 4. Seeding specification

Both variants extend the Short-Burst seeding formula from G.6. All seeds derived from `base_seed` via SHA-256; fully reproducible given `base_seed`.

### Burst-level seeds

```
chain_seed(base_seed: u64, burst_idx: usize, prefix: &str) -> u64:
  SHA-256(prefix || burst_idx.to_le_bytes() || "_" || base_seed.to_le_bytes())
  -> least-significant 64 bits

Prefixes:
  ShortBurstForest:     "SBF_CHAIN_"   (Short-Burst Forest)
  ShortBurstMergeSplit: "SBMS_CHAIN_"  (Short-Burst MergeSplit)
```

The prefix is distinct from ShortBurst's `"SHORT_BURST_CHAIN_"`, ensuring that the same `base_seed` and `burst_idx` produce different burst seeds across variants. Silent reuse of seeds across chain types is prevented.

### Per-step seeds within a burst (Forest ReCom and MergeSplit both need two RNG streams per step)

```
forward_seed(burst_seed: u64, step: u32) -> u64:
  SHA-256("SBF_FWD_" || step.to_le_bytes() || "_" || burst_seed.to_le_bytes())
  -> least-significant 64 bits

reverse_seed(burst_seed: u64, step: u32) -> u64:
  SHA-256("SBF_REV_" || step.to_le_bytes() || "_" || burst_seed.to_le_bytes())
  -> least-significant 64 bits
```

The prefixes `"SBF_FWD_"` and `"SBF_REV_"` are shared between `ShortBurstForest` and `ShortBurstMergeSplit` — both use two-stream steps with the same internal derivation. The variant is distinguished by the burst-level prefix (`"SBF_CHAIN_"` vs `"SBMS_CHAIN_"`), which propagates into `burst_seed` and therefore into all downstream step seeds.

**Version-lock**: A test asserts that each prefix constant in source equals its expected string and that `chain_seed(0, 0, "SBF_CHAIN_")` and `chain_seed(0, 0, "SBMS_CHAIN_")` equal hard-coded expected values. This prevents silent seed-compatibility drift across algorithm versions.

---

## 5. `run_short_burst_forest()` in bisection_runner.rs

Mirrors `run_short_burst()` exactly but replaces `RecomChain` with `ForestRecomChain` and uses two RNG streams per step.

```rust
pub fn run_short_burst_forest(
    adjacency: &[Vec<usize>],
    vertex_weights: &[i64],
    edge_weights: &HashMap<(usize, usize), f64>,
    num_districts: usize,
    balance_tolerance: f64,
    niter: u32,
    base_seed: u64,
    burst_length: usize,
    n_bursts: usize,
    p: f64,
) -> Result<HashMap<usize, usize>, String>
```

Key difference from `run_short_burst()`: each burst step calls `chain.step(&mut rng_forward, &mut rng_reverse)` instead of `chain.step(&mut rng)`. The two RNG streams are derived per step from `burst_seed` using `forward_seed()` and `reverse_seed()`.

`run_short_burst_merge_split()` has the same signature; it substitutes `MergeSplitChain` for `ForestRecomChain`.

---

## 6. Audit chain

### ShortBurstForest

```json
"search": "short-burst-forest",
"burst_length": 20,
"n_bursts": 50,
"percentile": 0.0,
"base_seed": 12345678,
"burst_seeds": [derived_u64, ...],
"selected_burst_idx": 12,
"chain_type": "forest-recom",
"per_burst_acceptance_rates": [0.55, 0.61, 0.48, ...]
```

### ShortBurstMergeSplit

```json
"search": "short-burst-merge-split",
"burst_length": 20,
"n_bursts": 50,
"percentile": 0.0,
"base_seed": 12345678,
"burst_seeds": [derived_u64, ...],
"selected_burst_idx": 7,
"chain_type": "merge-split",
"per_burst_acceptance_rates": [0.63, 0.71, 0.58, ...]
```

`burst_seeds` records the actual 64-bit values derived from `chain_seed`. An auditor who knows `base_seed` can re-derive all seeds using the SHA-256 formula and verify the values match without running the code.

`selected_burst_idx` is the 0-based index into the EC-sorted endpoint list corresponding to `floor(p * n_bursts)`, allowing verification that the plan at that index was used.

`per_burst_acceptance_rates` is new relative to `ShortBurst` — it records the fraction of `burst_length` steps that were accepted within each burst (i.e., `steps_accepted / burst_length` per burst). This enables diagnosis of burst-level chain health. The field has length `n_bursts`.

**`burst_seeds` must record actual 64-bit values** (not symbolic names), so an auditor can independently verify seed derivation without running any code. The chain_type field embeds which underlying chain was used, making the audit chain self-describing.

---

## 7. Test invariants

### L0 (inline unit tests)

**Both variants**:
- `n_bursts` endpoints produced, each a valid k-district plan (contiguous, population-balanced)
- `p=0.0` returns the endpoint with minimum EC among all bursts; `p=1.0` returns the endpoint with maximum EC (ascending sort — both ends tested to catch inverted sort)
- Same `base_seed` → identical `selected_burst_idx` and plan (deterministic)
- Chain restarts from previous endpoint, not from initial plan
- `per_burst_acceptance_rates` has length `n_bursts`
- All entries of `per_burst_acceptance_rates` are in `[0.0, 1.0]`
- `n_bursts=1, burst_length=1`: degenerate case succeeds and returns the single endpoint

**ShortBurstForest-specific**:
- Burst seeds use `"SBF_CHAIN_"` prefix — hard-coded expected value for `chain_seed(0, 0, "SBF_CHAIN_")` asserted in source
- Forward and reverse step seeds are distinct: `forward_seed(s, i) != reverse_seed(s, i)` for all (s, i) — prefix `"SBF_FWD_"` vs `"SBF_REV_"` tested explicitly

**ShortBurstMergeSplit-specific**:
- Burst seeds use `"SBMS_CHAIN_"` prefix — hard-coded expected value for `chain_seed(0, 0, "SBMS_CHAIN_")` asserted in source
- `"SBMS_CHAIN_"` burst seeds are distinct from `"SBF_CHAIN_"` burst seeds for the same `(base_seed, burst_idx)` — cross-variant seed isolation tested

**Cross-variant isolation**:
- `chain_seed(base_seed, i, "SBF_CHAIN_") != chain_seed(base_seed, i, "SHORT_BURST_CHAIN_")` for all (base_seed, i) — Forest variant does not reuse standard ShortBurst seeds
- `chain_seed(base_seed, i, "SBMS_CHAIN_") != chain_seed(base_seed, i, "SHORT_BURST_CHAIN_")` for all (base_seed, i)

### L1 (integration, synthetic data)

- 4x4 grid, k=2, n_bursts=10, burst_length=5: all endpoints are valid and contiguous
- `ShortBurstForest` and `ShortBurst` (standard) produce different results on the same `base_seed` — the underlying chains differ, so trajectories differ even from the same starting plan
- `n_bursts=1, burst_length=1`: succeeds, returns a single valid endpoint
- Acceptance rates in `per_burst_acceptance_rates` are all in `[0.0, 1.0]`
- Determinism: same `base_seed` → same `burst_seeds`, same `selected_burst_idx`, same final plan

### L2 (#[ignore], real data)

- NC 2020, k=14, n_bursts=50, burst_length=20: compare final EC among `ShortBurst` (standard), `ShortBurstForest`, and `ShortBurstMergeSplit` on the same `base_seed`. Record results; do not assert strict ordering (the three chains explore different regions of plan space). Assert that all three return valid k-district plans.
- NC 2020, k=14: `per_burst_acceptance_rates` mean for `ShortBurstForest` in `[0.30, 0.75]` and for `ShortBurstMergeSplit` in `[0.40, 0.85]` (sanity bands — collapse to 0 or 1 indicates a bug).

---

## 8. Implementation priority

1. **ShortBurstForest** (1 week): most commonly requested; extends `run_short_burst()` directly by substituting `ForestRecomChain` and adding the two-stream step loop
2. **ShortBurstMergeSplit** (1 week): mirrors `ShortBurstForest` with `MergeSplitChain`; Forest ReCom must be done first to establish the two-stream seeding pattern

---

## 9. Open questions (P2, deferred)

- Should `per_burst_acceptance_rates` be optional (only recorded when `--verbose` flag is set)? Current proposal records it unconditionally since the field is small (n_bursts f64 values) and valuable for chain health diagnosis.
- Should there be a `ShortBurstMultiScale` variant? Deferred — MultiScaleChain full implementation is Phase 2.
- Should the `burst_length` default differ for Forest ReCom? Forest ReCom's lower acceptance rate means fewer plans are explored per step. If practitioners observe insufficient plan diversity at `burst_length=20`, a higher default (e.g., 30) may be warranted. Deferred to empirical evaluation after L2 results.
- Should `run_short_burst_forest()` expose a `--output-all-bursts` flag to save all endpoints for diagnostics? Deferred — consistent with the open question in the parent ShortBurst spec.
