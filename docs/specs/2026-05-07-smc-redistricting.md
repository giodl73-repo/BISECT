# Spec: Sequential Monte Carlo for Redistricting — `redist-smc` crate

**Status**: Proposed (R1 reviewed, major revisions applied)  
**Reviewed R1**: MERIDIAN 2.5/4, BENCHMARK 2/4, SURVEY 2/4, COVENANT 3/4 → avg 2.4/4  
**Date**: 2026-05-07  
**Related paper**: G.7 (SMC for calibrated redistricting ensembles)  
**Depends on**: `redist-ensemble` (RecomChain, SpanningTree), `redist-core` (adjacency, population)  
**Implements**: `redist ensemble --method smc`

---

## 1. Motivation and positioning

SMC is the only method in the G series that produces a **statistically correct** sample from the space of valid redistricting plans — i.e., a sample whose weights correctly represent the uniform distribution over all valid k-district plans of the state graph. ReCom (H.1, H.2) is a Markov chain whose stationary distribution is not the uniform distribution; its distributional properties depend on mixing time, which is hard to certify for large graphs. SMC provides calibrated posteriors without mixing assumptions.

The cost is algorithmic complexity: SMC requires a sequential construction procedure, importance weighting, and systematic resampling — more moving parts than ReCom. The output is also different: not a single plan, but a population of N weighted plans.

### What SMC is and is not

| Property | SMC | ReCom (BisectionEnsemble) | ConvergenceSweep |
|---------|-----|--------------------------|-----------------|
| Output | N weighted plans | single plan | single plan |
| Distribution | calibrated (importance-weighted) | approximate (Markov chain) | deterministic |
| Mixing assumption | none | required | none |
| Per-state runtime | minutes (N=1000) | seconds (T=100) | seconds |
| Use case | statistical inference | optimisation | certification |
| `SeedCompositor` variant | no — standalone | yes | yes |

SMC is the right tool when the question is "what fraction of valid plans have property X?" ReCom is the right tool when the question is "find a single compact valid plan."

---

## 2. Algorithm: SMC for redistricting

The algorithm is the Fifield et al. (2020) adaptation of Sequential Monte Carlo to redistricting, as implemented in the R `redist` package. We adapt it to Rust with precise seeding.

### 2.1 High-level structure

SMC constructs a distribution over k-district plans by sequentially growing one district at a time. At each stage t (t = 1 … k−1), we have N partial plans, each assigning some tracts to districts 1 … t and leaving the rest unassigned. At stage t, each particle grows one more district from the unassigned tracts using a spanning tree proposal.

```
SMC(adj, pop, k, n_particles, pop_tolerance, resample_threshold, base_seed):
  particles = [empty_plan] × n_particles   // all tracts unassigned
  log_weights = [0.0] × n_particles
  resample_count = 0

  for t in 1..k:
    // Step 1: Propose next district for each particle
    for i in 0..n_particles:
      seed_i = particle_seed(base_seed, t, i)
      rng_i = SmallRng::seed_from_u64(seed_i)
      (particles[i], log_w_increment) = propose_district(
          particles[i], adj, pop, k, t, pop_tolerance, rng_i
      )
      log_weights[i] += log_w_increment

    // Step 2: Normalize weights
    log_weights = log_weights - logsumexp(log_weights)

    // Step 3: Resample if ESS < threshold
    ess = 1.0 / sum(exp(log_weights)^2)
    if ess < resample_threshold * n_particles:
      resample_seed = resample_seed(base_seed, resample_count)
      particles = systematic_resample(particles, log_weights, resample_seed)
      log_weights = [0.0] × n_particles   // equal weights after resample
      resample_count += 1

  // Final stage: assign remaining tracts to district k
  for i in 0..n_particles:
    particles[i] = assign_remaining(particles[i], k)

  // Normalize final weights to sum to 1.0
  weights = softmax(log_weights)
  return SmcResult { particles, weights, resample_count }
```

### 2.2 District proposal: `propose_district`

At stage t, particle i has districts 1 … t−1 assigned. The unassigned tracts form a connected subgraph (this is an invariant maintained throughout; see §2.4). We propose district t by:

1. Find all connected components of the unassigned subgraph. Select the **largest connected component** C (by population). Sample a **seed tract** s uniformly from all tracts in C. (Using the largest component preserves the connectivity invariant; see §2.4.)
2. Sample a **spanning tree** T of the unassigned subgraph (restricted to the connected component containing s) via Wilson's algorithm.
3. Find a **cut edge** e* in T such that removing e* yields one component containing exactly floor(remaining_pop / (k − t + 1)) ± pop_tolerance population. This is the balanced-cut criterion.
4. The component containing s becomes district t. If both components satisfy the balance criterion, choose the one containing s; the other remains unassigned.
5. The **importance weight increment** is `log_w_i += log(valid_cuts(T))`.

**Weight derivation**: We sample one valid cut edge uniformly at random from the `valid_cuts(T)` balanced cuts in T. The proposal probability for choosing this specific cut is `1 / valid_cuts(T)`. The target distribution assigns equal weight to all valid district sequences. By importance sampling, the correction factor is `target / proposal = 1 / (1 / valid_cuts(T)) = valid_cuts(T)`. Taking logs: `log_w_increment = log(valid_cuts(T))`. A tree with many balanced cuts has higher weight because we sampled it with lower-than-uniform probability. A tree with exactly one balanced cut has log_w_increment = 0 (our sample was as likely as any other; no correction needed).

```rust
fn propose_district(
    partial: &PartialPlan,
    adj: &[Vec<usize>],
    pop: &[i64],
    k: usize,
    stage: usize,       // 1-based stage index
    pop_tolerance: f64,
    rng: &mut SmallRng,
) -> (PartialPlan, f64)  // (updated partial plan, log weight increment)
```

### 2.3 Seeding specification

All seeds are derived from `base_seed` via SHA-256 with domain-separated prefixes. This ensures full reproducibility given only `base_seed`.

```
particle_seed(base_seed: u64, stage: u32, particle_idx: u32) -> u64:
  // Fixed-width encoding: 8 bytes each, no ambiguity between (stage=12, particle=3) and (stage=1, particle=23)
  SHA-256("SMC_PARTICLE_"             // 13 bytes
          || stage.to_le_bytes()      // 4 bytes (u32)
          || "_"                      // 1 byte separator
          || particle_idx.to_le_bytes() // 4 bytes (u32)
          || "_"                      // 1 byte separator
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 31 bytes

resample_seed(base_seed: u64, resample_round: u32) -> u64:
  SHA-256("SMC_RESAMPLE_"             // 13 bytes
          || resample_round.to_le_bytes() // 4 bytes (u32)
          || "_"                      // 1 byte separator
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 26 bytes

**Important**: after a systematic resample at stage t, the particle at new index j was drawn from old index `index_map[j]`. For the next stage t+1, particle j uses `particle_seed(base_seed, t+1, j)` — the seed is based on the NEW index after resampling, not the pre-resample index. This means the particle trajectory is NOT traceable through resample events from the seed alone; the `index_map` (recorded in the audit, see §6) is required to reconstruct any particle's full history.
```

The prefix strings are version-locked: any change to the proposal algorithm must change the prefix to prevent silent seed compatibility.

### 2.4 Connectivity invariant

**Invariant**: after each district is assigned, the remaining unassigned tracts form a connected subgraph.

This is guaranteed by construction if the state graph is connected and each district is grown from a connected component by removing a balanced cut. The spanning tree cut always leaves both components connected (by definition of a tree edge cut). However, at stage k−1, the last unassigned tract must be checked: if it is disconnected from the rest, it is absorbed into the most population-deficient adjacent district.

If connectivity is violated mid-construction (can happen in pathological graphs with population concentrations), the particle is **killed**: its weight is set to −∞ and it is excluded from the resample. The ESS drops; if ESS < threshold × N, resampling occurs. If all particles are killed (ESS = 0), the SMC run fails.

### 2.5 Systematic resampling

Given log-weights w_0 … w_{N−1} (normalised to sum to 0 in log space), systematic resampling draws N indices with replacement according to the weights using a single uniform random variable:

```
systematic_resample(particles, log_weights, resample_seed) -> (new_particles, index_map):
  weights = exp(log_weights)   // normalised, sums to 1.0
  cumsum = prefix_sum(weights) // cumsum[N-1] = 1.0
  u = uniform(0, 1/N) seeded from resample_seed  // single draw
  indices = []
  for j in 0..N:
    target = u + j as f64 / N as f64   // target ∈ [0, 1)
    // binary search in cumsum for the smallest i where cumsum[i] >= target
    i = cumsum.partition_point(|&c| c < target)
    i = i.min(N - 1)                   // clamp to [0, N-1] — never out of bounds
    indices.push(i)
  index_map = indices.clone()          // record: new particle j came from old particle index_map[j]
  return ([particles[i] for i in indices], index_map)
```

The `partition_point` (lower_bound) formulation avoids the out-of-bounds bug in the `floor(N * (u + j/N))` form. The `index_map` is recorded in the audit output (see §6).

Note: systematic resampling requires O(N log N) for the binary search variant; O(N) is achievable with a linear scan since targets are monotonically increasing, but the log-N variant is clearer to specify and verify.

Systematic resampling has lower variance than multinomial resampling and is O(N) given sorted weights.

---

## 3. New crate: `redist-smc`

SMC is complex enough to warrant its own crate, separate from `redist-ensemble`.

```
redist/crates/redist-smc/
  src/
    lib.rs              // pub use
    algorithm.rs        // run_smc() top-level
    proposal.rs         // propose_district(), spanning tree growth
    resample.rs         // systematic_resample(), ESS computation
    partial_plan.rs     // PartialPlan struct, connectivity tracking
    seeds.rs            // particle_seed(), resample_seed() SHA-256 derivation
    output.rs           // SmcResult, serialisation to NDJSON
  tests/
    L0_unit.rs          // weight normalisation, seed derivation, ESS formula
    L1_integration.rs   // small synthetic graphs (4-node, 9-node)
    L2_real.rs          // #[ignore] NC, WI real data
  Cargo.toml
```

### 3.1 Core types

```rust
pub struct PartialPlan {
    /// assignment[tract] = Some(district_id) or None (unassigned)
    pub assignment: Vec<Option<u32>>,
    /// Unassigned tract count remaining
    pub unassigned: usize,
    /// Unassigned subgraph adjacency (indices into global adj)
    pub unassigned_adj: Vec<Vec<usize>>,
}

pub struct SmcResult {
    /// N plans, each a complete assignment Vec<u32> (1-based district, length = n_tracts)
    pub plans: Vec<Vec<u32>>,
    /// Normalised importance weights, sum = 1.0 ± 1e-9
    pub weights: Vec<f64>,
    /// Number of resampling events
    pub resample_count: u32,
    /// Per-stage ESS trace (length = k−1)
    pub ess_trace: Vec<f64>,
    /// base_seed used
    pub base_seed: u64,
    /// n_particles used
    pub n_particles: usize,
}
```

### 3.2 Output format: NDJSON

Each line is one particle:
```json
{"plan": [1,1,2,2,1,3,...], "log_weight": -2.31, "particle_idx": 0}
```

Final line is a metadata record:
```json
{"type": "metadata", "base_seed": 12345678, "n_particles": 5000,
 "resample_count": 3, "ess_trace": [4823, 4201, 3892],
 "smc_version": "1.0", "ensemble_output_version": "1.0"}
```

The `log_weight` values in the output are normalised (log-sum-exp = 0). Consumers should use `exp(log_weight)` as the weight, then normalise to sum = 1.0.

---

## 4. CLI interface

SMC is a subcommand of `redist ensemble`, not of `redist state` or `redist build`:

```bash
# Generate 5000-particle SMC ensemble for NC congressional (k=14)
redist ensemble --method smc \
  --state NC --year 2020 \
  --particles 5000 \
  --resample-threshold 0.5 \
  --pop-tolerance 0.005 \
  --base-seed 12345678 \
  --output nc_smc_2020.ndjson

# With more particles for publication-quality inference
redist ensemble --method smc \
  --state NC --year 2020 \
  --particles 50000 \
  --base-seed 42 \
  --output nc_smc_50k.ndjson
```

### 4.1 Parameter defaults

| Flag | Default | Notes |
|------|---------|-------|
| `--particles` | 5000 | Practical for 50-state sweep; 50K+ for publication |
| `--resample-threshold` | 0.5 | ESS < 0.5 × N triggers resample |
| `--pop-tolerance` | 0.005 | ±0.5% population balance (strict for congressional) |
| `--base-seed` | content-derived | SHA-256(census\_release\_id \|\| "SMC\_SEED\_V1") where census\_release\_id = `{state}_{year}_{redistricting_data_release_date}` per B.16 §3.1 |

### 4.2 Parameter scaling

| State size | n_particles | Expected runtime (sequential) | Expected runtime (Rayon, 16 cores) | Peak memory | Resample events |
|-----------|------------|-------------------------------|--------------------------------------|-------------|----------------|
| Small (k≤5, e.g. VT) | 1000 | <1 min | <10 s | <50 MB | 1–2 |
| Medium (k=8–14, e.g. WI, NC) | 5000 | 30–90 min | 5–15 min | 200–500 MB | 2–4 |
| Large (k=30+, e.g. TX, CA) | 20000 | 8–24 hours | 1–3 hours | ~1 GB | 5–10 |

Runtime estimates are approximate and depend on tract count and spanning tree cost per proposal. The sequential estimates apply during Phase 1 (before Rayon parallelism is implemented in Week 5). All timing measurements should be reported with hardware spec (CPU model, core count, RAM).

Large states require more particles because ESS degrades faster at each stage. For TX (k=38), fewer than 10,000 particles typically produces ESS→0 before the final stage. The 1 GB memory figure for TX (k=38, N=20K, 5200 tracts) can be reduced by storing `Vec<u8>` instead of `Vec<u32>` for plans with k ≤ 255.

---

## 5. Integration with existing tools

### 5.1 Not a `SeedCompositor` variant

SMC produces N weighted plans, not one plan. It cannot be integrated into the single-plan compositor without a selection step. Two deferred options:

1. `SeedCompositor::SmcPercentile { n_particles, p }` — run SMC, then return the plan at the p-th percentile of the weighted EC distribution. This treats SMC as a search method.
2. `redist ensemble analyze` — compute distributional statistics over the weighted SMC ensemble (fraction of plans satisfying property X, weighted mean EC, etc.).

Both are Phase 2. For now, SMC is analysis-only.

### 5.2 `redist label-verify` exclusion and file-level integrity

`redist label-verify` does not verify SMC outputs because there is no single "submitted plan" — SMC produces a weighted ensemble. However, the NDJSON output file includes a `file_sha256` field (see §6) that allows any consumer to verify the file has not been modified after generation. Any plan extracted from an SMC ensemble and submitted to a redistricting body must:
1. Record the extraction method (percentile rank, maximum-weight, etc.) and particle index
2. Record the `file_sha256` of the source NDJSON file
3. Record the `base_seed` and `n_particles` from the metadata line

This provides a three-step audit chain: file integrity → algorithm parameters → extracted plan.

### 5.3 Comparison with GerryChain

The R `redist` package (McCartan et al. 2023) implements SMC via `redist_smc()`. Our implementation should be validated against the R package output on a small test case (same seed, same graph, same k) in the L2 test suite. This validates correctness of the algorithm, not just the implementation.

---

## 6. Audit chain

The SMC output file records full reproducibility. Two types of records:

**Per-resample records** (one per resampling event, embedded in the NDJSON stream between stages):
```json
{"type": "resample", "stage": 5, "resample_round": 2,
 "ess_before": 1203.4, "resample_seed": 9876543210,
 "index_map": [2, 2, 7, 14, 0, ...]}   // new_particle[j] came from old_particle[index_map[j]]
```

The `index_map` field is critical: it records exactly which pre-resample particle each post-resample slot was copied from. Without it, a verifier cannot trace any particle's trajectory across resampling events. The array length equals `n_particles`.

**Final metadata record** (last line of the NDJSON file):
```json
{
  "type": "metadata",
  "base_seed": 12345678,
  "n_particles": 5000,
  "resample_threshold": 0.5,
  "pop_tolerance": 0.005,
  "k": 14,
  "state": "NC",
  "year": "2020",
  "resample_count": 3,
  "resample_rounds": [2, 5, 10],
  "ess_trace": [4823.1, 4201.4, 3892.7, ...],
  "particle_seed_formula": "SHA-256('SMC_PARTICLE_' || stage:u32le || '_' || particle:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le",
  "resample_seed_formula": "SHA-256('SMC_RESAMPLE_' || round:u32le || '_' || base_seed:u64le) → first 8 bytes as u64le",
  "smc_version": "1.0",
  "ensemble_output_version": "1.0",
  "file_sha256": "a3f4..."   // SHA-256 of the entire NDJSON file except this last line
}
```

The `file_sha256` field is the SHA-256 of all bytes in the file preceding the final metadata line (i.e., all particle records and resample records). This allows a verifier to confirm the file was not modified after generation without re-running the algorithm.

**Verification protocol** (5 steps):
1. Verify `file_sha256` matches a fresh SHA-256 of the file body
2. Re-derive all `particle_seed(base_seed, stage, particle_idx)` values and confirm they match (optional spot check)
3. For each resample event, re-run `systematic_resample` with the recorded `resample_seed` and confirm `index_map` matches
4. Re-run `propose_district` for any particle at any stage using the recorded seeds and `index_map` to trace its pre-resample ancestors
5. Confirm final `weights` sum to 1.0 ± 1e-6 (using Kahan summation)

---

## 7. Test invariants

### L0 (unit, always run)

- `particle_seed(s, t, i) ≠ particle_seed(s, t, j)` for i ≠ j — seeds are distinct across particles
- `particle_seed(s, t, i) ≠ resample_seed(s, r)` for all t, i, r — particle and resample seeds never collide
- `systematic_resample` with equal weights: each of the N indices should appear either floor(N/N)=1 or ceil(N/N)=1 times; for non-power-of-two N with a uniform draw u=0, verify indices are `[0, 1, 2, ..., N-1]`
- `ESS(uniform weights w_i = 1/N)`: `1.0 / sum(w_i^2) = 1.0 / (N * (1/N)^2) = N`. In f64 arithmetic, this is subject to rounding; test as `(ess - N as f64).abs() < 1e-9`
- `ESS(one weight = 1.0, rest = 0.0) = 1.0 ± 1e-12` — degenerate case
- Weights after resampling: post-resample log_weights set to 0.0 (equal weights); `exp(0.0) = 1.0` per particle
- Same `base_seed` → identical `particle_seed` and `resample_seed` values (exact equality)
- `softmax(log_weights).sum()`: must equal `1.0 ± 1e-6` for N ≤ 50,000; the spec mandates Kahan compensated summation for the final weight normalisation step. The 1e-9 tolerance is too tight at N=50K (accumulated f64 error ≈ N × ε ≈ 5.5e-12 per operation, ~5e-12 total for small N but grows with N). Use 1e-6.
- `index_map` from `systematic_resample` has length exactly N
- All `index_map` values are in `[0, N−1]` (no out-of-bounds)

### L1 (integration, synthetic graphs, always run)

- `run_smc(4-node path, pop=[100,100,100,100], k=2, N=100, seed=42)` → 100 valid 2-district plans; weights sum to 1.0 ± 1e-6 (Kahan); no plan has 0 tracts
- `run_smc(..., k=2, N=100, seed=42)` called twice → identical `plans`, `weights`, `resample_count`, and `index_map` values (determinism)
- All plans are contiguous: for each plan, for each district d, `bfs(adj, tracts_in_district_d)` visits all tracts in d (contiguity check is via BFS from any tract in d; every other tract must be reachable)
- All plans satisfy population balance: for each plan, `|pop(district_d) − total_pop / k| ≤ pop_tolerance × total_pop` for all d
- No plan has a district with 0 tracts; no tract is assigned to district 0 or > k
- ESS trace has length exactly k−1
- `resample_count` ≤ k−1 (cannot resample more than once per stage)
- After each resample, `index_map.len() == n_particles` and all values are in `[0, n_particles−1]`
- `run_smc(single-node graph, pop=[1000], k=1, N=10)` → all 10 plans are the trivial single-district plan; all weights equal 0.1 ± 1e-6; no resampling (ESS = N throughout)

### L2 (real data, `#[ignore]`)

- `run_smc(NC, 2020, k=14, N=1000, seed=42)` → 1000 plans; mean weighted EC within 10% of BisectionEnsemble(p=0.5) result
- `run_smc(VT, 2020, k=1, N=100)` → all plans are the trivial single-district plan
- Cross-validation: compare weighted EC distribution to GerryChain/R redist SMC on the same small graph

---

## 8. Implementation plan

### Phase 1 (this spec): `redist-smc` crate skeleton

1. **Week 1**: `PartialPlan`, `seeds.rs`, L0 tests for seeding and ESS
2. **Week 2**: `proposal.rs` — spanning tree growth + balanced cut + weight increment
3. **Week 3**: `algorithm.rs` — full SMC loop, resampling, output NDJSON
4. **Week 4**: L1 integration tests; CLI wiring in `redist ensemble`
5. **Week 5–6**: Performance optimisation (Rayon per-particle parallelism within a stage); L2 NC test

### Phase 2 (follow-on spec): `SeedCompositor::SmcPercentile`

Once `redist-smc` is stable, a follow-on spec will define:
- `SeedCompositor::SmcPercentile { n_particles, p }` — run SMC, select plan at percentile p of weighted EC distribution
- `redist ensemble analyze` — weighted statistics over SMC ensemble
- Validation against R `redist::redist_smc()` output for NC, WI

### Known risks

1. **ESS collapse on large states**: TX (k=38) with N=5000 may produce ESS→0 before completing all stages. Mitigation: adaptive particle count (double N if ESS < 0.1 × N).
2. **Connectivity violation**: pathological population distributions can strand isolated tracts. Mitigation: kill the particle (weight → −∞); resample immediately if > 50% of particles killed.
3. **Memory**: 50K particles × 5200 TX tracts × 4 bytes = 1GB. Mitigation: store plan as `Vec<u8>` (k ≤ 255 districts) or stream to disk.

---

## 9. Open questions (deferred)

- **Proposal distribution**: the Fifield et al. proposal grows one district per stage. An alternative (McCartan et al. 2023) proposes a merge-split step within SMC. Which is better for large k?
- **Parallel resampling**: systematic resampling is sequential by design. Can it be parallelised without losing the single-RNG guarantee?
- **Incremental re-run**: if `base_seed` changes, the entire SMC run must restart. Can we checkpoint at resample events?
- **SMC for legislative chambers**: k can be 100+ for state house districts. The k−1 stages imply O(k × N × n) total work — potentially prohibitive for large k. Subplan SMC (group districts into blocks) may be necessary.
