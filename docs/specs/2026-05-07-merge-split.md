# Spec: Merge-Split MCMC — Reversible Recombination with Explicit Acceptance Ratio

**Status**: Proposed (R1 reviewed, P1 fixes applied)  
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 3/4 → avg 3.0/4  
**Date**: 2026-05-07  
**Extends**: `redist-ensemble` crate (new `merge_split.rs` module)  
**Related paper**: G.10  
**Depends on**: `redist-ensemble::spanning` (Wilson's UST), `redist-ensemble::recom` (pair selection)

---

## Overview

Merge-Split is a Metropolis-Hastings MCMC chain for redistricting that is explicitly reversible. It differs from standard ReCom in one key way: instead of always accepting any valid split, it uses a **cut-count ratio** as the acceptance criterion. This makes the chain reversible without requiring a matrix-tree determinant (unlike Forest ReCom).

**Relationship to existing algorithms**:

| Property | StandardReCom | MergeSplit | ForestRecom |
|----------|--------------|------------|-------------|
| Acceptance ratio | Always 1 | cut_new / cut_fwd | det_new / det_fwd |
| Targets uniform? | No (approx) | Approx | Yes (exact) |
| Per-step cost | O(m log m) | O(m log m) | O(m² or m³) |
| Extra RNG draws | 0 | 1 reverse tree | 1 reverse tree |
| Acceptance rate | High (~80%) | Medium (~60%) | Lower (~40%) |

Merge-Split occupies the middle ground: faster than Forest ReCom (no determinant), more theoretically grounded than standard ReCom (non-trivial MH ratio). The per-step cost is O(m log m) — two Wilson UST samples — matching standard ReCom. The stationary distribution guarantee is approximate (like standard ReCom) rather than exact (Forest ReCom), because the ratio uses cut counts rather than spanning-tree counts.

**Flag disambiguation**: `--merge-split-steps` is distinct from `--forest-steps`, `--flip-steps`, `--burst-length`, and `--ensemble-steps`.

---

## 1. Algorithm (Janson 2022)

### Conceptual derivation

The naive approach — use the same spanning tree T for both forward and reverse counting — collapses to standard ReCom. Since both directions look at the same tree T with the same balance constraint, the cut counts are symmetric and the acceptance ratio is always 1. The non-trivial formulation breaks this symmetry by sampling a **second independent spanning tree** from the proposed state, then counting how many cuts of *that* tree lead back to the current state.

This two-tree construction makes forward and reverse counts independent draws, producing a non-trivial ratio that corrects for the varying density of balanced cuts across the plan space.

### Per-step algorithm

```
MergeSplitStep(plan, adj, pop, k, pop_tolerance, rng, rng_reverse):
  // 1. Select adjacent district pair (with pair reselection on failure)
  pairs = adjacent_district_pairs(plan)
  shuffle(pairs, rng)
  for (d_i, d_j) in pairs[:MAX_PAIR_ATTEMPTS]:
    region = tracts_in(d_i) union tracts_in(d_j)

    // 2. Sample a spanning tree T of the merged region (forward tree)
    T = Wilson_UST(subgraph(region), rng)

    // 3. Count ALL valid balanced cut edges in T (forward count)
    valid_cuts_forward = count_balanced_cuts(T, region, pop, pop_tolerance)
    if valid_cuts_forward == 0: continue  // pair reselection

    // 4. Select one cut uniformly at random -> proposed split (A, B)
    e_star = uniform_sample_cut(T, valid_cuts_forward, rng)
    (A, B) = split_on(T, e_star)

    // 5. Sample a NEW spanning tree T_new from the proposed state (reverse tree)
    //    rng_reverse is a separate RNG seeded from reverse_seed (see seeding section)
    T_new = Wilson_UST(subgraph(region), rng_reverse)

    // 6. Count ALL balanced cut edges in T_new — this is the reverse proposal denominator.
    //    Any balanced cut of T_new is a valid reverse proposal from the proposed plan (A, B);
    //    the probability of landing on ANY specific reverse outcome is 1/valid_cuts_reverse.
    //    This is why the ratio is valid_cuts_forward / valid_cuts_reverse.
    valid_cuts_reverse = count_balanced_cuts(T_new, region, pop, pop_tolerance)

    // 7. Metropolis-Hastings acceptance
    //    ratio = P(reverse path) / P(forward path)
    //          = (1/valid_cuts_reverse) / (1/valid_cuts_forward)
    //          = valid_cuts_forward / valid_cuts_reverse
    //    Note: ratio > 1 is valid MH; min(ratio, 1.0) gives the acceptance probability
    //    Guard: if valid_cuts_reverse == 0, the proposed plan (A, B) has no valid reverse split,
    //    making the reverse proposal probability 0, which means the acceptance ratio is 0
    //    and the move is rejected.
    if valid_cuts_reverse == 0:
      return StepRecord { accepted: false, valid_cuts_forward, valid_cuts_reverse: 0, ratio: 0.0 }
    ratio = valid_cuts_forward as f64 / valid_cuts_reverse as f64

    if rng.gen::<f64>() < ratio.min(1.0):
      apply_split(plan, d_i, d_j, A, B)
      return StepRecord { accepted: true, valid_cuts_forward, valid_cuts_reverse, ratio }
    else:
      return StepRecord { accepted: false, valid_cuts_forward, valid_cuts_reverse, ratio }

  return StepRecord { accepted: false, valid_cuts_forward: 0, valid_cuts_reverse: 0, ratio: 0.0 }
```

`MAX_PAIR_ATTEMPTS` defaults to `k` (number of districts). If all pairs are exhausted, the step returns `accepted: false` without modifying the plan.

### Acceptance ratio derivation

For a Metropolis-Hastings chain to be reversible with respect to a target distribution pi, it suffices that the acceptance probability satisfies detailed balance:

```
pi(x) * Q(x -> y) * alpha(x -> y) = pi(y) * Q(y -> x) * alpha(y -> x)
```

where Q(x -> y) is the proposal probability. In Merge-Split:

```
Q(x -> y) = (1 / n_pairs) * (1 / valid_cuts_forward(T))
Q(y -> x) = (1 / n_pairs) * (1 / valid_cuts_reverse(T_new))
```

The pair selection is symmetric (same pairs in both directions), so it cancels. The MH ratio for a uniform target (pi constant) is:

```
alpha(x -> y) = min(1, Q(y -> x) / Q(x -> y))
              = min(1, valid_cuts_forward / valid_cuts_reverse)
```

This is what the algorithm implements. For a non-uniform target, multiply by pi(y)/pi(x).

---

## 2. Seeding specification

Two independent RNG streams per step — forward and reverse — each deterministically derived from a shared base seed.

```
step_seed(base_seed, step, chain_idx) =
  SHA-256("MS_STEP_" || step:u64le || "_" || chain_idx:u32le || "_" || base_seed:u64le)
  -> least-significant 64 bits

reverse_seed(base_seed, step, chain_idx) =
  SHA-256("MS_REVERSE_" || step:u64le || "_" || chain_idx:u32le || "_" || base_seed:u64le)
  -> least-significant 64 bits
```

`step_seed` drives the forward RNG (pair shuffle, forward UST, cut selection). `reverse_seed` drives the reverse RNG (reverse UST only). The two prefixes guarantee `step_seed != reverse_seed` for all (step, chain_idx, base_seed) — verified by L0 test.

**Version-lock**: The prefixes `"MS_STEP_"` and `"MS_REVERSE_"` embed algorithm identity. Any change to which operations use each stream must change these prefixes. A test asserts that each prefix constant in source equals its expected string and that `step_seed(0, 0, 0)` and `reverse_seed(0, 0, 0)` equal hard-coded expected values, preventing silent seed-compatibility drift.

---

## 3. Rust struct and API

**`MergeSplitChain` struct** (`redist-ensemble/src/merge_split.rs`):

```rust
pub struct MergeSplitChain {
    pub adj: Vec<Vec<u32>>,
    pub pop: Vec<i64>,
    pub assignment: Vec<u32>,
    pub k: u32,
    pub pop_tolerance: f64,
    pub steps_taken: u64,
    pub steps_accepted: u64,
}

impl MergeSplitChain {
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        assignment: Vec<u32>,
        k: u32,
        pop_tolerance: f64,
    ) -> Self

    /// Advance the chain by one step.
    /// rng: forward stream (step_seed); rng_reverse: reverse stream (reverse_seed)
    pub fn step<R: Rng>(&mut self, rng: &mut R, rng_reverse: &mut R) -> StepRecord

    pub fn acceptance_rate(&self) -> f64
}

pub struct StepRecord {
    pub accepted: bool,
    pub valid_cuts_forward: usize,
    pub valid_cuts_reverse: usize,
    pub ratio: f64,
}
```

---

## 4. Compositor integration

**`SeedCompositor` variant** (Search layer — orthogonal to structure and weights):

```rust
SeedCompositor::MergeSplit {
    steps: usize,   // total chain steps (default: 1000)
    p: f64,         // percentile of visited plans by EC (default: 0.0 = minimum EC)
}
```

The compositor collects all **accepted** plans during the run, sorts them by edge-cut ascending, and returns the plan at index `floor(p * steps_accepted)`. If `steps_accepted == 0`, the initial plan is returned (failure path identical to Flip).

---

## 5. CLI

```bash
--search merge-split --merge-split-steps 1000 --percentile 0.0
```

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: merge-split
  merge_split_steps: 1000
  percentile: 0.0
```

**Parameter scaling guidance**:

| State size | Example states | merge_split_steps |
|-----------|---------------|------------------|
| Small (k<=5) | VT, WY, ND, AK | 500 |
| Medium (k=8-14) | WI, NC, MN | 1000 |
| Large (k=20-38) | TX, FL, CA | 2000 |

---

## 6. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"search": "merge-split",
"merge_split_steps": 1000,
"percentile": 0.0,
"base_seed": 12345678,
"steps_taken": 1000,
"steps_accepted": 743,
"acceptance_rate": 0.743,
"selected_step": 612,
"step_seed_formula": "SHA-256('MS_STEP_' || step:u64le || '_' || chain:u32le || '_' || base_seed:u64le)",
"reverse_seed_formula": "SHA-256('MS_REVERSE_' || step:u64le || '_' || chain:u32le || '_' || base_seed:u64le)"
```

`selected_step` is the index within the accepted-plans list (0-indexed, sorted by EC ascending) that corresponds to `floor(percentile * steps_accepted)`. An auditor who knows `base_seed` can re-derive all seeds and re-run the chain to verify `steps_accepted` and the plan at `selected_step` match the submitted plan.

**Note on `selected_step` semantics**: in this spec, `selected_step` is the 0-based index within the EC-sorted list of ACCEPTED plans (not the chain step index). This matches Forest ReCom's definition. A verifier replays all accepted plans in order, sorts by EC, and confirms the plan at position `selected_step` matches the submitted plan.

---

## 7. Test invariants

### L0 (inline unit tests)

- Acceptance ratio `valid_cuts_forward / valid_cuts_reverse` is always a positive finite f64 when both counts are nonzero
- `ratio.min(1.0)` is in [0.0, 1.0] — valid MH acceptance probability
- `valid_cuts_reverse=0` guard: on a graph where the proposed split has no valid reverse cuts (can be constructed synthetically), the step must be rejected (not panic with division by zero). `StepRecord.ratio` must equal `0.0`.
- Accepted step produces a valid k-district plan (contiguous, population-balanced)
- Rejected step leaves `plan.assignment` byte-for-byte identical to pre-step state
- Same `base_seed` -> identical step sequence (determinism)
- `p=0.0` EC <= `p=1.0` EC on any non-degenerate run (ascending sort, both ends tested to catch inverted sort)
- `step_seed(s, i, b) != reverse_seed(s, i, b)` for all (s, i, b) — prefix distinctness
- `step_seed` prefix constant in source equals `"MS_STEP_"` exactly; hard-coded expected value for `step_seed(0, 0, 0)` asserted
- `reverse_seed` prefix constant in source equals `"MS_REVERSE_"` exactly; hard-coded expected value for `reverse_seed(0, 0, 0)` asserted
- Acceptance rate > 0 on a well-connected graph run for 100 steps (chain never fully stalls)

### L1 (integration, synthetic data)

- 4x4 grid, k=2, T=500 steps: all accepted plans contiguous and population-balanced
- Acceptance rate > 10% (Merge-Split accepts more liberally than Forest ReCom)
- Determinism: same seed -> same `steps_accepted`, same `selected_step`, same final plan
- `selected_step` is in [0, steps_accepted) — index within bounds
- `steps_accepted <= steps_taken` always holds
- Failure path: if no pair has valid cuts for 100 consecutive steps on a degenerate graph, `steps_accepted=0` and the initial plan is returned unchanged

### L2 (#[ignore], real data)

- NC 2020, k=14, T=10,000 steps, 4 independent chains: R-hat < 1.05 across all 4 chains (convergence diagnostic)
- MergeSplit acceptance rate vs ForestRecom: MergeSplit's single-forward-tree proposal does not guarantee strictly higher acceptance than ForestRecom's two-tree approximation on all graphs. Record both rates; assert that MergeSplit rate is not more than 20 percentage points LOWER than ForestRecom (i.e., MergeSplit should be competitive). Do not assert strict ordering.
- NC 2020, k=14, T=10,000: `steps_accepted / steps_taken` in [0.30, 0.85] (sanity band — collapse to 0 or 1 indicates a bug)

---

## 8. Open questions (P2, deferred)

- Is the two-tree Janson formulation truly Markovian? Yes — the reverse tree is sampled fresh at each step, not stored from a previous step. The chain state is the plan assignment only; T and T_new are local to each step.
- Should `valid_cuts_reverse` use the same population-balance constraint as forward? Yes — a symmetric constraint is required for detailed balance. Asymmetric constraints would break the MH derivation.
- Can the reverse tree be avoided for specific graph structures (paths, trees)? Yes — on path graphs the number of balanced cuts is determined by the population prefix sum, so a closed-form reverse count exists. Deferred.
- Should the implementation expose `--output-all-accepted` to save all accepted plans for diagnostics? Deferred to follow-on spec.
- Non-uniform target: the current spec targets the approximately-uniform distribution over valid plans. Extending to a non-uniform target (e.g., weighted by compactness) requires multiplying the MH ratio by `pi(proposed) / pi(current)`. Deferred to G.10 paper.
