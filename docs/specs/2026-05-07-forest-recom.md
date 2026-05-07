# Spec: Forest ReCom — Reversible Recombination via Spanning Forest Sampling

**Status**: Proposed (R1 reviewed, P1 fixes applied)
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 3/4 → avg 3.0/4
**Date**: 2026-05-07
**Extends**: `redist-ensemble` crate (new `forest_recom.rs` module)
**Related paper**: G.9
**Depends on**: `redist-ensemble::spanning` (Wilson's UST already implemented)

---

## Overview

Forest ReCom is a Metropolis-Hastings chain on the space of valid k-district plans. Unlike standard ReCom (which accepts all valid splits unconditionally), Forest ReCom uses an acceptance ratio that makes the chain reversible with the **uniform distribution** over valid plans as the stationary distribution.

| Property | Forest ReCom | Standard ReCom | Flip |
|----------|-------------|----------------|------|
| Stationary distribution | Uniform over valid plans | Non-uniform (unknown) | Approx. uniform (slow mixing) |
| Acceptance | MH ratio, may reject | Always accepts | Always accepts valid flips |
| Cost per step | 2 spanning trees | 1 spanning tree | O(1) |
| Per-step speed | Slower than ReCom | Faster | Fastest |
| Mixing correctness | Theoretically guaranteed | Not guaranteed | Not guaranteed |
| Paper | G.9 | H.1, H.2 | G.8 |
| `--search` flag | `forest-recom` | `bisection-ensemble` | `flip` |
| Step flag | `--forest-steps` | `--ensemble-steps` | `--flip-steps` |

**Key insight**: in standard ReCom, we merge two districts and split via a random spanning tree cut. The proposal probability for a given new plan depends on how many balanced spanning tree cuts lead to it — plans reachable by many cuts are systematically over-proposed. Forest ReCom corrects for this via an acceptance ratio.

**Flag disambiguation** (extends the ensemble-search-algorithms table):

| Mode | Step flag | Distinct from |
|------|-----------|---------------|
| `forest-recom` | `--forest-steps` | `--ensemble-steps`, `--flip-steps`, `--burst-length` |

---

## 1. Algorithm (McCartan & Imai 2020)

### 1.1 Per-step algorithm

```
ForestRecomStep(plan, adj, pop, k, pop_tolerance, rng_forward, rng_reverse):
  // 1. Select adjacent district pair (with pair reselection, same as standard ReCom)
  pairs = adjacent_district_pairs(plan)
  shuffle(pairs, rng_forward)
  for (d_i, d_j) in pairs[:MAX_PAIR_ATTEMPTS]:
    region = tracts_in(d_i) ∪ tracts_in(d_j)

    // 2. Sample a spanning tree T of the merged region (forward tree)
    T = Wilson_UST(subgraph(region), rng_forward)

    // 3. Enumerate all balanced cut edges in T (forward direction)
    valid_cuts_forward = [e ∈ T : balanced_cut(e, region, pop, pop_tolerance)]
    if valid_cuts_forward.is_empty(): continue  // pair reselection

    // 4. Select one cut uniformly → proposed split (A, B)
    e* = uniform_sample(valid_cuts_forward, rng_forward)
    (A, B) = split_on(T, e*)

    // 5. Compute the REVERSE proposal count
    //    Independent spanning tree sample of the same merged region
    T_rev = Wilson_UST(subgraph(region), rng_reverse)
    valid_cuts_reverse = [e ∈ T_rev : balanced_cut(e, region, pop, pop_tolerance)]

    // 6. Metropolis-Hastings acceptance ratio
    //    ratio = (reverse proposal prob) / (forward proposal prob)
    //          = (1 / valid_cuts_reverse) / (1 / valid_cuts_forward)
    //          = valid_cuts_forward / valid_cuts_reverse
    ratio = |valid_cuts_forward| as f64 / |valid_cuts_reverse| as f64

    // 7. Accept with probability min(1, ratio)
    if rng_forward.gen::<f64>() < ratio.min(1.0):
      apply_split(plan, d_i, d_j, A, B)
      return StepRecord { accepted: true, ratio, n_forward: |valid_cuts_forward|, n_reverse: |valid_cuts_reverse| }
    else:
      return StepRecord { accepted: false, ratio, n_forward: |valid_cuts_forward|, n_reverse: |valid_cuts_reverse| }

  return StepRecord { accepted: false, ratio: 0.0, n_forward: 0, n_reverse: 0 }  // all pairs exhausted
```

### 1.2 Correctness argument

The proposal probability for move π → π' is:

```
P(π → π') = (1 / |pairs|) × (1 / |valid_cuts_forward(T)|)
```

The MH acceptance ratio corrects for asymmetry:

```
MH(π → π') = P(π' → π) / P(π → π')
            = |valid_cuts_forward| / |valid_cuts_reverse|
```

The resulting chain satisfies **detailed balance in expectation**: because T_rev is a single random sample estimating E[|valid_cuts|], the ratio is an unbiased estimator of the exact Metropolis-Hastings ratio. The chain targets the uniform distribution over valid plans asymptotically; individual steps satisfy expected detailed balance rather than exact per-step detailed balance. See McCartan & Imai (2020) §3.2 for the formal convergence argument.

**Two spanning tree samples per step**: the forward tree T is used to make the proposal; the reverse tree T_rev is an independent sample used only to count `valid_cuts_reverse`. This is an approximation — the exact reverse count requires the expected number of valid cuts over all spanning trees (via the matrix-tree theorem), not just one sample. The single-tree approximation is unbiased in expectation and becomes exact as the number of spanning tree samples grows. In practice, for redistricting subgraphs (typically 50–300 tracts), the variance of `|valid_cuts_reverse|` across spanning trees is low. See McCartan & Imai 2020 §3.2 for the theoretical justification and empirical validation.

### 1.3 Seeding specification

Two distinct seeds are required per step per chain to ensure reproducibility of both the forward and reverse spanning tree samples. Seeds are derived from `base_seed` via SHA-256 with domain-separated prefixes.

```
forward_seed(base_seed: u64, step: u64, chain_idx: u32) -> u64:
  SHA-256("FR_FORWARD_"           // 11 bytes
          || step.to_le_bytes()   // 8 bytes (u64)
          || "_"                  // 1 byte separator
          || chain_idx.to_le_bytes()  // 4 bytes (u32)
          || "_"                  // 1 byte separator
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 33 bytes

reverse_seed(base_seed: u64, step: u64, chain_idx: u32) -> u64:
  SHA-256("FR_REVERSE_"           // 11 bytes
          || step.to_le_bytes()   // 8 bytes (u64)
          || "_"                  // 1 byte separator
          || chain_idx.to_le_bytes()  // 4 bytes (u32)
          || "_"                  // 1 byte separator
          || base_seed.to_le_bytes()) // 8 bytes (u64)
  → first 8 bytes as little-endian u64  // total input: 33 bytes
```

The prefix strings differ (`"FR_FORWARD_"` vs `"FR_REVERSE_"`) so `forward_seed(s, t, c) ≠ reverse_seed(s, t, c)` for all (s, t, c). Fixed-width encoding ensures no length-extension collisions: `(step=12, chain=3)` and `(step=1, chain=23)` produce different byte sequences.

**Version-lock**: the prefix constants embed algorithm identity. Any change to the semantics of either spanning tree sample must change the corresponding prefix. Old and new manifests are distinguishable by prefix inspection alone; silent seed compatibility across algorithm versions is prevented.

---

## 2. `ForestRecomChain` struct

Mirrors `RecomChain` in the existing `redist-ensemble` crate.

```rust
pub struct ForestRecomChain {
    pub adj: Vec<Vec<u32>>,
    pub pop: Vec<i64>,
    pub assignment: Vec<u32>,
    pub k: u32,
    pub pop_tolerance: f64,
    pub steps_taken: u64,
    pub steps_accepted: u64,
}

pub struct StepRecord {
    pub accepted: bool,
    pub ratio: f64,           // |valid_cuts_forward| / |valid_cuts_reverse|; 0.0 if all pairs exhausted
    pub n_forward: usize,     // |valid_cuts_forward| for the selected pair
    pub n_reverse: usize,     // |valid_cuts_reverse| for the same pair
}

impl ForestRecomChain {
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        assignment: Vec<u32>,
        k: u32,
        pop_tolerance: f64,
    ) -> Self

    /// Advance the chain one step.
    /// rng_forward: seeded from forward_seed(base_seed, step, chain_idx)
    /// rng_reverse: seeded from reverse_seed(base_seed, step, chain_idx)
    pub fn step<R: Rng>(
        &mut self,
        rng_forward: &mut R,
        rng_reverse: &mut R,
    ) -> StepRecord

    /// steps_accepted / steps_taken; 0.0 if steps_taken == 0
    pub fn acceptance_rate(&self) -> f64
}
```

`ForestRecomChain` lives in `redist/crates/redist-ensemble/src/forest_recom.rs`. No new crate is required; this is an addition to the existing `redist-ensemble` crate, analogous to `flip.rs` for the Flip sampler.

---

## 3. Compositor integration (Layer 3 — Search)

```rust
SeedCompositor::ForestRecom {
    steps: usize,   // total chain steps (default: 1000)
    p: f64,         // percentile of EC distribution to return (default: 0.0)
}
```

**Selection protocol**: the compositor collects all accepted plans, sorts them by edge-cut (EC) in ascending order, and returns the plan at rank `floor(p × steps_accepted)`. Steps that are rejected leave the plan unchanged and are not added to the accepted list. The sort is ascending: `p=0.0` returns the minimum-EC accepted plan; `p=1.0` returns the maximum-EC accepted plan.

If `steps_accepted == 0` (chain never accepts — pathological case on disconnected graphs), the compositor returns the initial METIS plan as a fallback. This is recorded in the manifest via `selected_step = null`.

---

## 4. CLI flags

```bash
redist state --state NC --year 2020 \
  --search forest-recom \
  --forest-steps 1000 \
  --percentile 0.0 \
  --base-seed 12345678

redist build my_plan --year 2020 \
  --search forest-recom \
  --forest-steps 500 \
  --percentile 0.5
```

**YAML** (in `configs/{label}.yml`):

```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: forest-recom
  forest_steps: 1000
  percentile: 0.0
workers: 8
years: ["2020"]
```

**Parameter defaults**:

| Flag | Default | Notes |
|------|---------|-------|
| `--forest-steps` | 1000 | Total chain steps (accepted + rejected) |
| `--percentile` | 0.0 | Percentile of accepted plans by EC (ascending) |
| `--base-seed` | content-derived | SHA-256(census_release_id \|\| "FR_SEED_V1") per B.16 §3.1 |

**Parameter scaling guidance**:

| State size | Example states | forest_steps |
|-----------|---------------|-------------|
| Small (k≤5) | VT, WY, ND, AK | 300 |
| Medium (k=8–14) | WI, NC, MN | 1000 |
| Large (k=20–38) | TX, FL, CA | 3000 |

Expected acceptance rate on well-connected redistricting graphs: 30–70%. At 50% acceptance and `forest_steps=1000`, approximately 500 distinct plans enter the selection pool.

---

## 5. Audit chain

Forest ReCom adds the following fields to `runs/{label}/{year}/index.json`:

```json
"search": "forest-recom",
"forest_steps": 1000,
"percentile": 0.0,
"base_seed": 12345678,
"chain_idx": 0,
"steps_taken": 1000,
"steps_accepted": 612,
"acceptance_rate": 0.612,
"selected_step": 847,
"forward_seed_formula": "SHA-256('FR_FORWARD_' || step:u64le || '_' || chain:u32le || '_' || base_seed:u64le) -> first 8 bytes as u64le",
"reverse_seed_formula": "SHA-256('FR_REVERSE_' || step:u64le || '_' || chain:u32le || '_' || base_seed:u64le) -> first 8 bytes as u64le"
```

`selected_step` records the step index (0-based) whose accepted plan was returned — specifically, the step at rank `floor(p × steps_accepted)` in the EC-sorted list of accepted plans. An auditor knowing `base_seed` can:

1. Re-derive `forward_seed(base_seed, step, 0)` and `reverse_seed(base_seed, step, 0)` for any step
2. Re-run the chain from the initial plan using those seeds
3. Confirm `steps_accepted` and `acceptance_rate` match
4. Confirm the plan at `selected_step` matches the submitted plan

**`chain_idx`**: the index of this chain within a multi-chain run (0 for single-chain use). It must be recorded in the manifest so a verifier can re-derive the exact seeds. Without `chain_idx`, two chains with the same `base_seed` and `step` would produce identical forward and reverse seeds, destroying independence. Single-chain runs always record `chain_idx: 0`.

**Null case**: if `steps_accepted == 0`, `selected_step` is recorded as `null` and the returned plan is the initial METIS plan. The auditor verifies by confirming the plan matches the METIS output (which is separately recorded in the manifest under `"structure_plan"`).

**Prefix version-lock test**: a test asserts that the constant `FR_FORWARD_PREFIX` in source equals `"FR_FORWARD_"` and that `forward_seed(0, 0, 0)` equals a hard-coded expected value. An analogous test covers `FR_REVERSE_PREFIX`. These tests fail if a prefix is silently changed without bumping the version string.

---

## 6. Test invariants

### L0 (unit, always run)

- Acceptance rate in (0, 1] — chain never fully stalls on well-connected graphs (a 2-district 4-node path graph always has at least one valid cut, so ratio > 0)
- Accepted step produces valid k-district plan (contiguous, population-balanced)
- Rejected step leaves plan unchanged — assignment vector bit-for-bit identical to pre-step assignment
- Same `base_seed` → identical sequence of accepted/rejected decisions and identical final plan (deterministic)
- `p=0.0` returns the minimum-EC accepted plan; `p=1.0` returns the maximum-EC accepted plan (ascending sort — test both ends to catch inverted sort)
- `forward_seed(s, t, c) ≠ reverse_seed(s, t, c)` for all (s, t, c) — no seed collision between the two trees; test with (s=0, t=0, c=0), (s=42, t=100, c=3)
- On a 2-district path graph of 4 nodes with equal population: `ratio = |valid_cuts_forward| / |valid_cuts_reverse|` is always in `(0, ∞)` — never NaN or infinite (both counts are ≥ 1 on a connected balanced path)
- `ForestRecomChain::acceptance_rate()` returns 0.0 when `steps_taken == 0` (no division by zero)
- `steps_accepted` ≤ `steps_taken` always (acceptance is a subset of all steps)
- Prefix version-lock: `FR_FORWARD_PREFIX == "FR_FORWARD_"` and `forward_seed(0, 0, 0)` equals hard-coded expected value; same for reverse
- `steps_accepted=0` fallback: when all chain steps are rejected (possible on degenerate graphs), the compositor returns the initial METIS plan and records `selected_step=null` in the manifest. Test: run ForestRecom on a 2-node graph where `pop_tolerance=0.0` (no valid cut exists) — result must be the initial plan, not an error.
- `ratio=0.0` all-pairs-exhausted path: when all district pairs fail to yield a valid forward cut, the step returns `accepted=false` and ratio is recorded as `0.0`. Test: verify `ratio` field exists in `StepRecord` and equals `0.0` on a degenerate graph.

### L1 (integration, synthetic graphs, always run)

- 4×4 grid k=2, T=500 steps: all accepted plans are contiguous and population-balanced
- Acceptance rate > 5% on a 4×4 grid k=2 (chain productive on well-connected graphs)
- Determinism: two runs with same `base_seed` on same graph produce identical `steps_accepted`, identical `selected_step`, and identical final plan
- Plan distribution on 4-node path graph k=2 over 10K steps: approximately uniform — each of the 3 valid 2-district plans appears roughly 1/3 of the time (test: chi-squared p > 0.01 at T=10K; this tests the correctness of the MH ratio, not just acceptance)
- `forest_steps=0`: `steps_accepted=0`, `selected_step=null`, returns initial METIS plan
- `forest_steps=1`: at most 1 accepted step; chain runs once; valid result returned

### L2 (real data, `#[ignore]`)

- NC 2020 k=14, T=10K, 4 independent chains (chain_idx 0–3): R-hat < 1.05 across all chains (Gelman-Rubin convergence diagnostic on EC; confirms chains are exploring the same distribution)
- Acceptance rate on NC 2020 k=14 falls in [0.30, 0.70] — empirically typical for redistricting subgraphs; outside this range indicates a seeding or ratio computation bug
- Compare acceptance rate to standard ReCom on same graph (Forest ReCom acceptance rate is lower by construction; if Forest ReCom rate ≥ ReCom rate, the MH ratio is being incorrectly computed as ≥ 1 always)

---

## 7. Implementation plan

Forest ReCom should be implemented **after standard ReCom tests pass** (i.e., after `RecomChain` in `redist-ensemble` is stable and its L0/L1 tests are green). It reuses Wilson's UST from `redist-ensemble::spanning`.

**Week 1**: `forest_recom.rs` skeleton — `ForestRecomChain::new`, seeding functions `forward_seed` / `reverse_seed`, L0 seed-collision and version-lock tests

**Week 2**: `ForestRecomChain::step` — Wilson UST call (reuse existing), balanced-cut enumeration, MH ratio, accept/reject; L0 ratio and determinism tests

**Week 3**: `SeedCompositor::ForestRecom` variant, CLI flags (`--search forest-recom`, `--forest-steps`, `--percentile`), YAML parsing, audit manifest fields

**Week 4**: L1 integration tests (4×4 grid, 4-node path distribution test); connect to `redist state` and `redist build`

**Week 5**: L2 NC convergence test (`#[ignore]`); parameter scaling table validation

---

## 8. Open questions (deferred)

- **Exact reverse count via matrix-tree theorem**: the current spec uses a single random spanning tree sample to estimate `|valid_cuts_reverse|`. The exact expected count over all spanning trees can be computed analytically for small subgraphs via the weighted matrix-tree theorem. Deferred to Phase 2 — would eliminate the approximation and make the chain exactly correct rather than approximately correct.
- **Adjacency structure for reverse tree**: the reverse tree uses the same merged-region adjacency as the forward tree (same `subgraph(region)`). This is correct by construction. No ambiguity.
- **MergeSplit cut enumeration**: should `ForestRecomChain` be configurable to use MergeSplit's cut enumeration (which computes all balanced cuts exactly via a DFS traversal of T) instead of the two-tree approximation? MergeSplit enumeration is O(n) per tree rather than O(1) but eliminates the approximation variance. Deferred — would be a `ForestRecomChain::exact_mode` flag.
- **Parallel chains**: Forest ReCom is embarrassingly parallel across independent chains (each chain has its own `base_seed` derived from `chain_seed(global_seed, chain_idx)`). Rayon parallelism across chains is straightforward; deferred until L1 tests pass.
- **SMC integration**: can Forest ReCom steps replace ReCom steps inside the SMC proposal? In principle yes (as a proposal within importance sampling); this would produce a statistically correct chain within each SMC particle. Deferred to a follow-on spec.
