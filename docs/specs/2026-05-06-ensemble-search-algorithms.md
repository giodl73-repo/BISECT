# Spec: Ensemble Search Algorithms — G series extensions

**Status**: Proposed (R1 reviewed, major revisions applied)  
**Date**: 2026-05-06  
**Reviewed**: MERIDIAN 2/4, BENCHMARK 2/4, SURVEY 3/4, COVENANT 2/4 → avg 2.25/4  
**Extends**: `redist-ensemble` crate, `SeedCompositor` three-layer compositor  
**Related papers**: G.6 (Short-Burst), G.7 (SMC), G.8 (Flip), B.19 (Simulated Annealing)

---

## Overview

Four new ensemble/search algorithms, each a `SeedCompositor` variant and `--search` CLI flag.  
All are **Layer 3 (Search)** — orthogonal to structure and weights.

| Algorithm | `--search` value | Paper | Layer | `--search` flag collision |
|-----------|-----------------|-------|-------|--------------------------|
| Short-Burst | `short-burst` | G.6 | Search | `--burst-length` (distinct from `--ensemble-steps`) |
| SMC | standalone only | G.7 | Standalone | not a `--search` flag — see §2 |
| Flip | `flip` | G.8 | Search | `--flip-steps` (distinct from both above) |
| Simulated Annealing | `simulated-annealing` | B.19 | **Structure** | `--sa-steps`, `--sa-t0`, `--sa-tf` |

**Flag disambiguation**: Each mode has its own step-count flag to avoid collision:

| Mode | Step flag | Distinct from |
|------|-----------|---------------|
| `short-burst` | `--burst-length` (per burst) + `--n-bursts` | `--ensemble-steps` |
| `bisection-ensemble` | `--ensemble-steps` (per node) | `--burst-length` |
| `flip` | `--flip-steps` (total flip proposals) | both above |
| `convergence` | `--seeds` (=threshold) | all above |

---

## 1. Short-Burst (G.6)

**Concept (Cannon, Duchin et al. 2022)**: Run `n_bursts` short ReCom chains of length `burst_length`. Keep the **chain endpoint** from each burst (not the step-minimum — that would be greedy scan, not Short-Burst). Sort endpoints by objective; return the one at percentile `p`.

The key insight: short bursts starting from a good initial plan tend to stay near good plans. The **endpoint** is kept because it represents a valid Markov chain state from which the next burst can restart; selecting the minimum within a burst would bias the sample and break the Markov property.

**Corrected algorithm**:
```
ShortBurst(burst_length, n_bursts, objective_fn, p, base_seed):
  current_plan = initial_METIS_plan
  burst_endpoints = []
  for burst in 0..n_bursts:
    seed_i = chain_seed(base_seed, burst)           // defined below
    chain = RecomChain::new(initial=current_plan, seed=seed_i)
    for _ in 0..burst_length:
      chain.step(rng)
    endpoint = chain.current_plan()                 // ENDPOINT, not minimum
    burst_endpoints.push((objective_fn(endpoint), endpoint))
    current_plan = endpoint                         // chain restarts from endpoint
  sort burst_endpoints by objective ASC
  return burst_endpoints[floor(p * n_bursts)].plan
```

**`chain_seed` definition**:
```
chain_seed(base_seed: u64, burst_idx: usize) -> u64:
  SHA-256("SHORT_BURST_CHAIN_" || burst_idx.to_le_bytes() || "_" || base_seed.to_le_bytes())
  → least-significant 64 bits
```

**Compositor integration**:
```rust
SeedCompositor::ShortBurst {
    burst_length: usize,  // ReCom steps per burst (default: 20)
    n_bursts: usize,      // number of bursts (default: 50)
    p: f64,               // percentile of endpoints to return (default: 0.0 = min EC endpoint)
}
```

**CLI**: `--search short-burst --burst-length 20 --n-bursts 50 --percentile 0.0`

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: short-burst
  burst_length: 20
  n_bursts: 50
  percentile: 0.0
```

**Audit chain**: Short-Burst adds to the build manifest:
```json
"search": "short-burst",
"burst_length": 20,
"n_bursts": 50,
"percentile": 0.0,
"base_seed": 12345678,
"burst_seeds": [seed_0, seed_1, ...]   // all n_bursts seeds recorded
```

**Test invariants (L0)**:
- `n_bursts` endpoints produced, each valid k-district plan
- `p=0.0` returns minimum-EC endpoint; `p=1.0` returns maximum-EC endpoint
- Same `base_seed` → identical result (deterministic)
- All endpoints have correct district count k
- Chain restarts from previous endpoint (not from initial plan)

---

## 2. Sequential Monte Carlo / SMC (G.7)

**Concept** (Imai, Kane, Fifield 2020 — the R `redist` package): generates a *weighted* sample of valid plans via sequential importance resampling. More statistically correct than MCMC for inference; produces calibrated posteriors.

**Architecture**: SMC is **not** a `SeedCompositor` variant. It is a standalone ensemble method accessed via `redist ensemble --method smc`, not via `redist state --search smc`. The reason: SMC doesn't produce a single plan — it produces a population of weighted plans. Integration into the single-plan compositor would require choosing one plan from the weighted population, which is semantically equivalent to a PercentileSweep on a weighted ensemble (deferred to a follow-on spec).

**Seeding specification**:
```
SmcResult = run_smc(adj, pop, k, pop_tolerance, n_particles=5000,
                    resample_threshold=0.5, base_seed: u64)

Internal seeds:
  particle_i_seed = chain_seed(base_seed, i)  // same formula as Short-Burst
  resample_seed   = chain_seed(base_seed, n_particles + resample_round)
```

All internal seeds derived from `base_seed` via SHA-256. Fully reproducible given `base_seed`.

**CLI (standalone only)**:
```bash
redist ensemble --method smc --particles 5000 --state NC --year 2020
# Produces: nc_smc_ensemble.json with weighted plan sample
```

**Audit chain**: SMC results include `base_seed` and `n_particles` in the output JSON. `redist label-verify` does not verify SMC outputs (they are analysis artifacts, not plan-generation artifacts).

**Test invariants**: `n_particles` plans produced with non-negative weights summing to 1; same `base_seed` → identical weights; all plans valid.

---

## 3. Flip Proposals (G.8)

**Concept**: Flip individual boundary tracts to adjacent districts. O(1) per step (vs O(n/k log n/k) for ReCom). Faster per step, much slower to mix, good for local sensitivity analysis around an existing plan.

**Algorithm**:
```
FlipChain(initial_plan, flip_steps, p, base_seed):
  plan = initial_plan
  visited = [(EC(plan), plan)]
  rng = SmallRng::seed_from_u64(chain_seed(base_seed, 0))
  for _ in 0..flip_steps:
    tract = random boundary tract (adjacent to different district)
    target_district = random adjacent district
    proposed = flip(plan, tract, target_district)
    if valid(proposed):  // population balance + contiguity
      plan = proposed
      visited.push((EC(plan), plan))
  sort visited by EC ASC
  return visited[floor(p * visited.len())]
```

**Compositor integration**:
```rust
SeedCompositor::Flip {
    flip_steps: usize,  // total flip proposals (default: 10000)
    p: f64,             // percentile of visited plans (default: 0.0)
}
```

**CLI**: `--search flip --flip-steps 10000 --percentile 0.5`

**Audit chain**: `flip_steps`, `percentile`, `base_seed` recorded in manifest.

**Test invariants**: All visited plans are valid; `p=0.0` ≤ EC of `p=1.0`; deterministic with same seed.

---

## 4. Simulated Annealing (B.19)

**Note**: SA is a **Structure layer** variant (replaces METIS at each bisection node), not Search. `SplitStrategy::SimulatedAnnealing`, not `SeedCompositor`.

**Corrected cooling schedule** — scaled to subgraph size `m` (tracts in the merged region):

```
T_0 = max(1.0, 0.01 * EC(initial_plan))    // proportional to initial EC
T_final = 1e-4                              // near-zero, allows greedy at end
n_steps = 10 * m                            // 10 steps per tract — scales with size
```

The initial temperature `T_0` is scaled to the initial edge cut so the acceptance probability for a typical move is ~37% at start and ~0% at end. Fixed T_0=1.0 does not scale across subgraphs with EC ranging from 5 to 5000.

**Compositor integration** (Structure layer):
```rust
SplitStrategy::SimulatedAnnealing {
    steps_per_tract: usize,   // n_steps = steps_per_tract * |region| (default: 10)
    t0_factor: f64,           // T_0 = t0_factor * EC(initial) (default: 0.01)
    t_final: f64,             // (default: 1e-4)
}
```

**CLI**: `--structure simulated-annealing --sa-steps-per-tract 10 --sa-t0-factor 0.01`

**Audit chain**: SA parameters recorded in `algorithm_params` manifest field.

**Test invariants**: Final plan EC ≤ initial plan EC (SA should not increase EC on average); deterministic with same seed.

---

## Implementation priority

1. **Flip** (1 week) — simplest, pure addition to `redist-ensemble`
2. **Short-Burst** (1 week) — builds on `RecomChain`; corrected endpoint semantics
3. **Simulated Annealing** (2 weeks) — new structure mode, `split_subgraph_sa()` in `bisection_runner.rs`
4. **SMC** (1 month) — new `redist-smc` crate; high value, complex

---

## Audit chain summary — all four modes

Every mode that produces a plan must record in `runs/{label}/{year}/index.json`:
```json
"search_mode": "short-burst",
"search_params": {
  "burst_length": 20,
  "n_bursts": 50,
  "percentile": 0.0,
  "base_seed": 12345678,
  "total_steps": 1000   // burst_length * n_bursts
},
"search_seed_formula": "SHA-256('SHORT_BURST_CHAIN_' || i || '_' || base_seed)"
```

This ensures `redist label-verify` can confirm the search parameters and seed derivation for any submitted plan.

---

## Open questions (P2, deferred)

- Should Short-Burst expose `--output-all-bursts` to save all endpoints for diagnostics?
- Can SMC be integrated into the compositor as `SeedCompositor::Smc { n_particles }` by sampling one plan from the weighted population at a given percentile?
- Should SA use parallel tempering (multiple temperature chains) for large subgraphs?
