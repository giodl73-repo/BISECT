# Spec: VRA-Aware MCMC — Markov Chain Sampling Over VRA-Compliant Redistricting Plans

**Status**: Proposed (R1 reviewed, P1 fixes applied)  
**Date**: 2026-05-07  
**Reviewed R1**: MERIDIAN 3/4, BENCHMARK 3/4, SURVEY 3/4, COVENANT 4/4 → avg 3.25/4  
**Extends**: `redist-ensemble` crate (new `vra_recom.rs`)  
**Related paper**: G.13  
**Depends on**: `redist-ensemble::forest_recom` (ForestRecomChain)

---

## Overview

Standard MCMC methods (ReCom, Forest ReCom, Merge-Split) sample the full space of valid k-district plans, including plans that violate the Voting Rights Act by diluting minority voting strength. VRA compliance is currently enforced at the structure layer (VRASection, B.14), but this prevents sampling the distribution over compliant plans — it only produces a single compliant plan.

VRA-Aware MCMC samples the **conditional distribution** over plans that preserve all designated majority-minority districts. It extends ForestRecomChain with a hard rejection rule: if a proposed step would reduce the minority VAP fraction of any protected district below `vap_threshold`, reject unconditionally regardless of the MH ratio. This is the correct approach for litigation: the ensemble represents valid plans under both equal population and VRA constraints.

**Relationship to related algorithms**:

| Property | ForestReCom | VraRecomChain |
|----------|-------------|---------------|
| Target distribution | Uniform over valid plans | Uniform over VRA-compliant valid plans |
| VRA enforcement | None | Hard rejection before MH step |
| Extra per-step cost | 0 | O(protected_districts) VAP checks |
| vra_rejection_rate | N/A | > 0 when majority-minority districts exist |
| Use case | General ensemble | Litigation / VRA compliance analysis |

**Flag disambiguation**: `--search vra-recom` is distinct from `--search forest-recom`, `--search merge-split`, `--search flip`, and `--search convergence`.

---

## 1. Algorithm

### Motivation

VRA compliance is a legal constraint, not a probabilistic preference. It must be enforced as a hard filter, not a soft penalty in the MH ratio. The conditional distribution over compliant plans is the correct target: every plan in the ensemble is a valid alternative that preserves majority-minority districts, giving courts and practitioners a defensible sample space.

### Protected district identification

Protected districts are identified from the initial plan at chain construction time:

- Any district whose minority VAP fraction >= `vap_threshold` is permanently protected
- Protection is not re-evaluated as the chain runs: the initial VRA status is the reference
- This matches the legal standard: the submitted plan's majority-minority districts must be preserved

If the initial plan has no majority-minority districts, `protected_districts` is empty and `vra_rejection_rate` will be 0.0 — the chain reduces to standard ForestRecomChain.

### Per-step algorithm

```
VraRecomChain(adj, pop, minority_vap, initial_plan, k, pop_tolerance, vap_threshold):
  // Identify initially-protected districts: those with minority VAP >= vap_threshold
  protected_districts = {d : minority_vap_fraction(d, initial_plan) >= vap_threshold}

  chain = ForestRecomChain(adj, pop, initial_plan, k, pop_tolerance)

  for each step (rng_fwd, rng_rev):
    // Run standard ForestRecomChain proposal
    (proposed_plan, log_w) = forest_recom_propose(chain, rng_fwd, rng_rev)

    // VRA hard check: if any protected district drops below threshold, reject
    for d in protected_districts:
      vap_fraction = minority_vap_in_district(d, proposed_plan, minority_vap)
      if vap_fraction < vap_threshold:
        return (chain.assignment, accepted=false, vra_rejection=true)

    // Proceed with standard ForestRecomChain MH acceptance
    accept_with_prob(min(1, exp(log_w)), rng_fwd)
```

The VRA check fires BEFORE the MH acceptance decision. VRA violations are rejected unconditionally (not probabilistically). The MH ratio is only evaluated if all protected districts remain compliant.

---

## 2. Seeding specification

Identical to ForestRecomChain seeding — VRA check adds no additional randomness:

```
forward_seed(base_seed, step, chain=0) =
  SHA-256("FR_FORWARD_" || step:u64le || "_" || 0u32le || "_" || base_seed:u64le)
  -> least-significant 64 bits

reverse_seed(base_seed, step, chain=0) =
  SHA-256("FR_REVERSE_" || step:u64le || "_" || 0u32le || "_" || base_seed:u64le)
  -> least-significant 64 bits
```

The chain uses the same seed derivation as ForestRecomChain so that VRA-aware and non-VRA-aware chains produce identical proposals — the only difference is whether VRA-violating proposals are accepted. This means a verifier can replay the chain with VRA checking disabled and confirm that every accepted plan in the VRA-aware run also appears in the unconstrained run.

**Version-lock**: The prefixes `"FR_FORWARD_"` and `"FR_REVERSE_"` are inherited from ForestRecomChain and must not change. If VRA-Aware MCMC is ever given its own seed derivation (e.g., to allow per-chain VRA threshold variation), the prefixes must change and a separate version flag must be recorded in the audit chain.

**Prefix inheritance version-lock**: VraRecomChain inherits `FR_FORWARD_` and `FR_REVERSE_` seed prefixes from ForestRecomChain. An L0 test asserts that the prefix constants in `VraRecomChain` equal the constants in `ForestRecomChain` — any rename in the parent will fail this test, preventing silent divergence.

---

## 3. Rust struct and API

**`VraRecomChain` struct** (`redist-ensemble/src/vra_recom.rs`):

```rust
pub struct VraRecomChain {
    pub inner: ForestRecomChain,
    pub minority_vap: Vec<f64>,            // minority VAP fraction per tract (0.0-1.0)
    pub vap_threshold: f64,                // default: 0.50
    pub protected_districts: HashSet<u32>, // districts protected at init
    pub vra_rejections: u64,               // count of VRA-triggered rejections
    pub steps_accepted: u64,
    pub steps_taken: u64,
}

impl VraRecomChain {
    pub fn new(
        adj: Vec<Vec<u32>>,
        pop: Vec<i64>,
        minority_vap: Vec<f64>,
        assignment: Vec<u32>,
        k: u32,
        pop_tolerance: f64,
        vap_threshold: f64,
    ) -> Self

    /// Advance the chain by one step.
    /// rng_fwd: forward stream (forward_seed); rng_rev: reverse stream (reverse_seed)
    pub fn step<R: Rng>(&mut self, rng_fwd: &mut R, rng_rev: &mut R) -> VraStepRecord

    pub fn acceptance_rate(&self) -> f64  // steps_accepted / steps_taken
    pub fn vra_rejection_rate(&self) -> f64  // vra_rejections / steps_taken
    pub fn mh_rejection_rate(&self) -> f64   // (steps_taken - steps_accepted - vra_rejections) / steps_taken
}

pub struct VraStepRecord {
    pub accepted: bool,
    pub vra_rejected: bool,  // true if rejected due to VRA (before MH evaluation)
    pub log_w: f64,          // MH log weight (f64::NAN if vra_rejected)
}
```

### Minority VAP data

Minority VAP per tract is available through the existing vertex weights pipeline. The `minority_col` parameter selects which column: `HVAP` (Hispanic), `BVAP` (Black), `HVAP+BVAP` (combined), etc. The values are pre-normalized to fractions (0.0–1.0) before being passed to `VraRecomChain::new`.

---

## 4. Compositor integration

**`SeedCompositor` variant** (Search layer — orthogonal to structure and weights):

```rust
SeedCompositor::VraRecom {
    steps: usize,        // total chain steps (default: 1000)
    p: f64,              // percentile of EC distribution (default: 0.0)
    vap_threshold: f64,  // minority VAP fraction threshold (default: 0.50)
}
```

The compositor collects all accepted plans during the run, sorts them by edge-cut ascending, and returns the plan at index `floor(p * steps_accepted)`. If `steps_accepted == 0`, the initial plan is returned (same failure path as ForestRecomChain).

Data requirement: minority VAP column must be loaded (same pipeline as `--weights-override vra-aligned`). If no minority VAP data is present, the compositor returns an error before running the chain.

---

## 5. CLI

```bash
--search vra-recom --forest-steps 1000 --percentile 0.0 --vra-threshold 0.50
```

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: vra-aligned
  search: vra-recom
  forest_steps: 1000
  percentile: 0.0
  vra_threshold: 0.50
```

**Parameter guidance**:

| State | Protected districts | Recommended steps |
|-------|---------------------|-------------------|
| NC 2020 | 2 (NC-1, NC-12) | 2000 |
| TX 2020 | 7-9 (Hispanic + Black) | 3000 |
| GA 2020 | 2-3 (Black-majority) | 2000 |
| VT 2020 | 0 | 500 (no VRA cost) |

When `protected_districts` is large (>= 5), `vra_rejection_rate` is typically high (>0.40). Increase `forest_steps` proportionally to maintain effective sample size (`steps_accepted` target: >= 200).

If `protected_districts` is empty after chain construction (no district in the initial plan has minority VAP ≥ vap_threshold), the CLI emits a warning: `WARNING: VRA-aware chain constructed with 0 protected districts — VRA enforcement is a no-op. Check --vra-threshold or use --weights-override vra-aligned to produce an initial VRA-compliant plan.`

---

## 6. Audit chain

Every run appends to `runs/{label}/{year}/index.json`:

```json
"search": "vra-recom",
"forest_steps": 1000,
"percentile": 0.0,
"base_seed": 12345678,
"vap_threshold": 0.50,
"protected_districts": [3, 7, 12],
"steps_taken": 1000,
"steps_accepted": 412,
"vra_rejections": 231,
"vra_rejection_rate": 0.23,
"mh_rejections": 357,
"selected_step": 291,
"forward_seed_formula": "SHA-256('FR_FORWARD_' || step:u64le || '_' || 0u32le || '_' || base_seed:u64le)",
"reverse_seed_formula": "SHA-256('FR_REVERSE_' || step:u64le || '_' || 0u32le || '_' || base_seed:u64le)"
```

`protected_districts` lists which district IDs were protected at chain construction.  
`vra_rejection_rate` is the fraction of proposals rejected due to VRA (not MH).  
`steps_accepted + vra_rejections + mh_rejections == steps_taken` must hold exactly.

An auditor who knows `base_seed` and `protected_districts` can re-derive all seeds and re-run the chain to verify `steps_accepted` and the plan at `selected_step` match the submitted plan.

---

## 7. Test invariants

### L0 (inline unit tests)

- All accepted plans preserve all protected districts at >= `vap_threshold` minority VAP
- VRA rejection fires when a constructed violation is proposed (test with synthetic `minority_vap`)
- `steps_accepted + vra_rejections + mh_rejections == steps_taken` for every step count
- Same `base_seed` -> identical trajectory (VRA check is deterministic given same `minority_vap`)
- `p=0.0` EC <= `p=1.0` EC (ascending sort — both ends tested to catch inverted sort)
- `protected_districts` computed correctly from `initial_plan` and `vap_threshold`
- `VraStepRecord.log_w` is `f64::NAN` when `vra_rejected == true`
- `vra_rejection_rate()` == 0.0 when `protected_districts` is empty
- `forward_seed(b, s, 0) != reverse_seed(b, s, 0)` for all (b, s) — inherited prefix distinctness

### L1 (integration, synthetic data)

- Synthetic 4x4 grid with `minority_vap` set so district 1 is protected: all accepted plans keep district 1 with minority VAP >= threshold
- `vra_rejection_rate > 0` when protected districts exist (chain actively enforces VRA)
- `vra_rejection_rate == 0.0` when no districts are protected (chain reduces to ForestRecomChain)
- Acceptance rate > 0 even with VRA constraint (chain not stalled): assert `steps_accepted >= 1` over 500 steps on a well-connected graph with one protected district
- `steps_accepted <= steps_taken` always holds
- `selected_step` is in `[0, steps_accepted)` when `steps_accepted > 0`
- Determinism: same seed + same `minority_vap` -> same `steps_accepted`, same `selected_step`, same final plan
- **Cross-chain replay invariant**: construct an unconstrained `ForestRecomChain` with the same base_seed. For every step, the unconstrained chain makes the same proposal as the VRA-aware chain (identical seeds → identical proposals). Confirm that every plan accepted by the VRA-aware chain also has minority_vap ≥ vap_threshold for all protected districts when the unconstrained plan is checked.

### L2 (#[ignore], real data)

- NC 2020, k=14, BVAP data, T=5000: all accepted plans preserve NC-1 and NC-12 (or whichever districts have BVAP >= 0.50 in the initial plan) at >= 0.50 BVAP
- Compare EC distribution to non-VRA Forest ReCom over NC 2020: VRA-constrained chain explores a subset of the plan space; assert EC mean is not more than 5% higher than unconstrained (VRA constraint should not catastrophically inflate EC)
- `vra_rejection_rate` in [0.05, 0.70] for NC 2020 k=14 (sanity band — 0.0 means VRA is not enforcing, > 0.70 means the chain is nearly stalled)

---

## 8. Relationship to VRASection (B.14)

VRASection is a **structure layer** algorithm: it modifies the METIS edge weights to bias toward VRA-compliant splits during the initial bisection. VRA-Aware MCMC is a **search layer** algorithm: it samples the conditional distribution over already-VRA-compliant plans.

They are complementary:
- Use VRASection to generate an initial VRA-compliant plan (guaranteed starting point)
- Use VRA-Aware MCMC to sample around that plan, exploring the space of compliant alternatives

The recommended composite workflow:
```yaml
structure: ratio-optimal-vra   # VRASection: ensures compliant initial plan
weights: vra-aligned           # loads minority VAP weights
search: vra-recom              # VraRecomChain: samples compliant neighbourhood
```

This ensures both the starting plan and the entire ensemble are VRA-compliant, satisfying the legal requirement that the submitted plan and all proposed alternatives respect Section 2.

---

## 9. Open questions (deferred)

1. **No initial majority-minority districts**: If the initial plan has no majority-minority districts, `protected_districts` is empty and `vra_rejection_rate` will be 0.0 — the chain reduces to standard Forest ReCom. No error is raised. Callers who expect VRA enforcement should validate that `protected_districts` is nonempty before trusting the ensemble.

2. **Per-district thresholds**: Should `vap_threshold` be per-district, allowing different thresholds for different communities (e.g., 0.50 for NC-1, 0.45 for NC-12)? The current spec uses a single global threshold for simplicity. Deferred — would require a `HashMap<u32, f64>` threshold map and a more complex CLI.

3. **Incremental VAP tracking**: Can the VRA check be made O(1) per step instead of O(district_size) via incremental VAP tracking? When a ReCom step swaps tracts between two districts, only the two affected districts need VAP recomputation. If only one of those districts is protected, cost is O(swapped_tracts) not O(district_size). Deferred — premature optimization until profiling shows VAP check is a bottleneck.

4. **Combined minority VAP columns**: Should `HVAP+BVAP` be a first-class column or computed from `HVAP` and `BVAP` at load time? Currently deferred to the weights pipeline; the `minority_vap` vector passed to `VraRecomChain` is assumed to be precomputed.
