# Spec: Ensemble Search Algorithms — G series extensions

**Status**: Proposed (R2 reviewed, targeted revisions applied)  
**Date**: 2026-05-06  
**Reviewed R1**: MERIDIAN 2/4, BENCHMARK 2/4, SURVEY 3/4, COVENANT 2/4 → avg 2.25/4  
**Reviewed R2**: MERIDIAN 4/4, BENCHMARK 2/4, SURVEY 3/4, COVENANT 2/4 → avg 2.75/4  
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

**Parameter scaling guidance** (for state staff and practitioners):

| State size | Example states | burst_length | n_bursts | flip_steps |
|-----------|---------------|-------------|---------|-----------|
| Small (k≤5) | VT, WY, ND, AK | 10 | 20 | 2,000 |
| Medium (k=8–14) | WI, NC, MN | 20 | 50 | 10,000 |
| Large (k=20–38) | TX, FL, CA | 30 | 100 | 50,000 |

Runtime estimate on a standard 8-core workstation: medium-state Short-Burst at defaults (~30s); large-state at scaled params (~5min). SA runtime scales automatically with `steps_per_tract × |subgraph|` and stays under 2min for any state at the default `steps_per_tract=10`.

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
"burst_seeds": [seed_0, seed_1, ...],   // all n_bursts seeds recorded
"selected_burst_idx": 12               // index of the burst whose endpoint was returned
```
The `selected_burst_idx` allows independent verification: a verifier can re-derive all burst seeds, re-run all bursts, and confirm that the plan at the recorded index matches the submitted plan.

**Chain-seed version-lock**: The prefix `"SHORT_BURST_CHAIN_"` embeds algorithm identity. Any change to burst semantics (e.g. keeping something other than the endpoint) must change this prefix. Old and new manifests are distinguishable; silent seed compatibility across algorithm versions is prevented.

**Test invariants (L0)**:
- `n_bursts` endpoints produced, each valid k-district plan
- `p=0.0` returns endpoint with the minimum EC among all bursts; `p=1.0` returns endpoint with maximum EC (sort is ascending, so rank 0 = min, rank n_bursts-1 = max — tests both ends to catch inverted sort)
- Same `base_seed` → identical `selected_burst_idx` and plan (deterministic)
- All endpoints have correct district count k
- Chain restarts from previous endpoint (not from initial plan)
- `n_bursts=1, burst_length=1`: degenerate case succeeds and returns the single endpoint
- `n_bursts=1, burst_length=0`: returns the initial plan unchanged (zero steps)

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

**Audit chain**: SMC results include `base_seed` and `n_particles` in the output JSON. `redist label-verify` does not verify SMC outputs.

**Why SMC is excluded from label-verify**: SMC produces a *weighted ensemble*, not a single plan. `label-verify` verifies a specific plan against the parameters that generated it. SMC has no "selected plan" — a user must make a separate selection decision (e.g., using `PercentileSweep` on the SMC output). The selection step would need its own provenance chain. Until a `SeedCompositor::SmcPercentile` variant is specified (deferred), the SMC output is an analysis artifact from which a user-chosen plan may be extracted. Any plan submitted from an SMC ensemble must record the selection method and index as a separate manifest entry; SMC itself is not a plan-submission workflow.

**Test invariants**: `n_particles` plans produced with non-negative weights summing to 1.0 ± 1e-9 (floating-point tolerance); same `base_seed` → identical weights; all plans valid (contiguous, population-balanced).

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

**Failure path**: If all `flip_steps` proposals fail contiguity or balance, the chain returns the initial plan (the `visited` list always contains at least the initial plan, so the return is always non-empty).

**Audit chain**: manifest records:
```json
"search": "flip",
"flip_steps": 10000,
"percentile": 0.5,
"base_seed": 12345678,
"visited_count": 3821,       // number of accepted plans in visited list
"selected_plan_rank": 1910   // floor(0.5 * 3821) — the index returned
```
The `selected_plan_rank` enables verification: re-run with same seed, confirm visited list has same length, confirm rank matches.

**Test invariants**:
- All visited plans are valid (contiguous, population-balanced)
- `p=0.0` EC ≤ `p=1.0` EC (ascending sort — tests both to catch inverted sort)
- Deterministic: same seed → same `visited_count` and same `selected_plan_rank`
- All-fail case: if every proposal fails, `visited_count=1` (initial plan only), result is the initial plan
- `flip_steps=0`: `visited_count=1`, returns initial plan
- SMC weight normalization: weights sum to 1.0 ± 1e-9 (floating-point tolerance required)

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

**Audit chain**: SA parameters recorded in `algorithm_params` manifest field with named fields:
```json
"structure": "simulated-annealing",
"sa_t0_factor": 0.01,
"sa_t_final": 1e-4,
"sa_steps_per_tract": 10,
"sa_initial_ec": 147,      // EC of the initial plan passed to SA
"sa_t0_actual": 1.47,      // max(1.0, 0.01 * 147)
"sa_n_steps": 830,         // 10 * 83 tracts in this subgraph
"sa_seed": 99123456        // seed used for acceptance-probability RNG
```
All fields named and typed so a verifier can independently reproduce the temperature schedule.

**Test invariants**:
- Deterministic: fixed seed, known synthetic subgraph, known initial EC → fixed final EC (not "on average" — one specific deterministic run with known inputs)
- Fixed-seed regression: `SA(4×4 grid, seed=42, T0=1.0, Tf=1e-4, n_steps=160)` → recorded final EC value (check this exact value in CI to catch cooling schedule bugs)
- Final plan is contiguous and population-balanced
- All acceptance decisions use SA seed, not base_seed (isolated RNG)

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
