# Spec: Multi-scale MCMC — Hierarchical Mixing for Large-k Redistricting

**Status**: Proposed (R1 major revision applied — P0 and P1 fixes)  
**Reviewed R1**: MERIDIAN 2/4, BENCHMARK 2/4, SURVEY 2/4, COVENANT 2/4 → avg 2.0/4  
**Date**: 2026-05-07  
**New crate**: `redist-multiscale`  
**Related paper**: G.11  
**Depends on**: `redist-ensemble` (RecomChain), `redist-data` (multi-resolution adjacency)

---

## Overview

Multi-scale MCMC resolves the slow mixing of standard ReCom for large-k states by interleaving fine-resolution (tract-level) ReCom moves with coarse-resolution (block-group-level) ReCom moves. A coarse move restructures many tracts simultaneously, collapsing O(k) sequential fine steps into a single proposal. This is a **Search layer** variant — orthogonal to structure and weights.

| Algorithm | `--search` value | Paper | Layer | Flag collision |
|-----------|-----------------|-------|-------|----------------|
| Multi-scale MCMC | `multiscale` | G.11 | Search | `--multiscale-steps`, `--multiscale-alpha` (distinct from all ensemble flags) |

**When to use**:

| State size | Example states | Recommended `--multiscale-steps` | Recommended `--multiscale-alpha` |
|-----------|---------------|----------------------------------|----------------------------------|
| Medium (k=8–14) | WI, NC, MN | 1,000 | 0.2 |
| Large (k=20–38) | TX, FL | 2,000 | 0.3 |
| Very large (k≥40) | CA | 3,000 | 0.4 |

**Alpha heuristic caveat**: The alpha values above (0.3 for TX, 0.4 for CA) are heuristic defaults based on the scaling argument that alpha ≈ sqrt(k_fine / k_coarse); they have not been validated empirically on these specific states. Phase 2 will add an empirical sweep over alpha ∈ {0.1, 0.2, 0.3, 0.4, 0.5} to find optimal values. Practitioners should treat these as starting points. \citep{autry2021} (Autry et al. recommend alpha ∝ mixing time ratio, which is state-specific.)

**Runtime estimates**:

| State size | n_particles | steps | alpha | Sequential runtime | Rayon (16 cores) |
|-----------|-------------|-------|-------|--------------------|------------------|
| Small (k≤5) | — | 500 | 0.2 | <30 s | <5 s |
| Medium (k=8–14) | — | 2000 | 0.3 | ~5–20 min | ~1–3 min |
| Large (k≥30) | — | 5000 | 0.35 | ~1–3 hours | ~10–30 min |

Note: dual adjacency file loading (tract + block-group) adds ~2–5 s startup time.

**Flag disambiguation**:

| Mode | Step flag | Alpha flag |
|------|-----------|------------|
| `multiscale` | `--multiscale-steps` | `--multiscale-alpha` |
| `short-burst` | `--burst-length` + `--n-bursts` | — |
| `bisection-ensemble` | `--ensemble-steps` | — |
| `flip` | `--flip-steps` | — |

---

## Motivation

Standard ReCom mixes slowly for large k (TX k=38, CA k=52) because each step changes exactly two districts. To move a plan from one basin of attraction to another requires O(k) sequential steps — the effective mixing time scales linearly with k. For TX with k=38, this means >10,000 ReCom steps before the chain forgets its starting plan.

Multi-scale MCMC (Autry, Carter, Herschlag, Hunter 2021) resolves this by interleaving two types of moves:

- **Fine moves** (probability 1−α): standard ReCom on the tract adjacency graph — local refinement
- **Coarse moves** (probability α): ReCom on a coarsened adjacency graph where multiple fine tracts are grouped into single "super-nodes" — enables large-scale restructuring in one step

A coarse move that reassigns a block group (containing many tracts) is equivalent to ~dozens of sequential fine moves, reducing mixing time from O(k × n) to O(k + n/coarsening_factor).

---

## Architecture

**New crate `redist-multiscale`** (not an extension of `redist-ensemble`):

- Requires multi-resolution adjacency data loading (two pkl files per state), which depends on `redist-data` patterns not currently in `redist-ensemble`
- `redist-multiscale` wraps two `redist-ensemble::recom::RecomChain` instances (one fine, one coarse) and coordinates moves between them
- Exposes `MultiScaleChain` as the public API; the CLI creates it via `SeedCompositor::MultiScale`

**Dependency graph**:
```
redist-multiscale
├── redist-ensemble (RecomChain, recom::step)
└── redist-data     (adjacency loading, pkl deserialization)
```

---

## Algorithm

```
MultiScaleChain(fine_adj, coarse_adj, fine_to_coarse, pop, k, pop_tolerance, alpha, base_seed):
  // fine_adj:        tract-level adjacency (n_tracts nodes)
  // coarse_adj:      block-group adjacency (n_bg nodes, each is a set of tracts)
  // fine_to_coarse:  mapping from tract index → block-group index
  // alpha:           probability of coarse move per step (default: 0.3)

  assignment_fine   = initial_METIS_plan(fine_adj, pop, k)         // tract-level
  assignment_coarse = aggregate(assignment_fine, fine_to_coarse)    // block-group level

  fine_chain   = RecomChain(fine_adj, pop, assignment_fine, k, pop_tolerance)
  coarse_chain = RecomChain(coarse_adj, coarse_pop, assignment_coarse, k, coarse_tol)

  for each step in 0..total_steps:
    seed = step_seed(base_seed, step, chain_idx)
    rng  = SmallRng::seed_from_u64(seed)

    if rng.gen::<f64>() < alpha:
      // Coarse move: run one ReCom step at the block-group level
      coarse_chain.step(&mut rng)
      // Project coarse assignment back to fine level:
      // all tracts in moved block-groups adopt new district assignment
      for bg in affected_block_groups:
        for tract in fine_to_coarse_inverse[bg]:
          assignment_fine[tract] = assignment_coarse[bg]
      // Recheck fine-level contiguity and rebalance if needed
      // (coarse move may violate fine-level balance — apply boundary swaps)
      assignment_fine = rebalance(assignment_fine, fine_adj, pop, pop_tolerance)
    else:
      // Fine move: run one standard ReCom step at tract level
      fine_chain.step(&mut rng)
      // Update coarse assignment to match
      assignment_coarse           = aggregate(assignment_fine, fine_to_coarse)
      coarse_chain.assignment     = assignment_coarse
```

**Coarse balance tolerance**: the coarse chain uses `coarse_tol = 2 × pop_tolerance` (recommended). Coarse moves are corrected by the projection + rebalance step at the fine level, so a looser coarse tolerance avoids blocking useful coarse proposals while the fine-level guarantee is maintained by `rebalance()`.

**Projection rebalancing**: when a coarse move violates fine-level population balance, `rebalance()` applies the same boundary-swap logic used in `split_subgraph`. See §2.5 for the full contract.

### 2.5 Rebalance Contract

After a coarse move, fine-level population balance may be violated because block-group boundaries do not align with the balanced-split criterion at the tract level. The `rebalance()` function must satisfy:

- **Input**: a fine-level plan that may violate `pop_tolerance` for some districts
- **Output**: a fine-level plan that satisfies `pop_tolerance` for all districts (or the original plan if rebalancing is impossible without violating contiguity)
- **Method**: boundary-swap loop — move boundary tracts from overweight districts to adjacent underweight districts, checking contiguity after each move (same logic as `split_subgraph`'s post-hoc boundary-swap in `bisection_runner.rs`)
- **Termination**: at most 200 boundary-swap iterations (matching the existing `split_subgraph` limit)
- **Failure**: if rebalancing fails (boundary swaps exhausted), reject the coarse move (treat as if `valid_cuts == 0`)

---

## Data model

```rust
pub struct HierarchyLevel {
    /// Adjacency graph at this resolution (indices are coarse-level nodes)
    pub adj: Vec<Vec<u32>>,
    /// Population of each coarse node (sum of fine tracts)
    pub pop: Vec<i64>,
    /// For each coarse node: list of fine tract indices it contains
    pub coarse_to_fine: Vec<Vec<usize>>,
    /// For each fine tract: which coarse node it belongs to
    pub fine_to_coarse: Vec<usize>,
}

pub struct MultiScaleChain {
    /// Finest resolution (tracts)
    pub fine: HierarchyLevel,
    /// Coarser resolution (block groups or counties)
    pub coarse: HierarchyLevel,
    /// Current tract-level assignment (length = n_tracts)
    pub assignment: Vec<u32>,
    pub k: u32,
    pub pop_tolerance: f64,
    /// Coarse balance tolerance (recommended: 2 × pop_tolerance)
    pub coarse_tol: f64,
    /// Probability of coarse move per step (default: 0.3)
    pub alpha: f64,
    pub total_steps: usize,
    pub steps_taken: u64,
    pub fine_steps: u64,
    pub coarse_steps: u64,
}
```

---

## Seeding

```
step_seed(base_seed, step, chain_idx) =
  SHA-256("MSC_STEP_" || step:u64le || "_" || chain_idx:u32le || "_" || base_seed:u64le) → u64le
```

A single seed per step determines both whether the step is fine or coarse (via the alpha comparison) and seeds the ReCom step at that level. This guarantees:

- The fine/coarse selection sequence is fully determined by `base_seed`
- The same `base_seed` produces the identical trajectory (same move type sequence, same plan outcome)
- No secondary seed is needed: the step seed is derived fresh from the global base seed for every step

`chain_idx` is 0 for single-chain use and must be recorded in the audit manifest.

**Implementation note**: the alpha draw always consumes one RNG value before the chain step, even when alpha=0.0. Implementations must NOT skip the alpha draw as a shortcut, even when alpha is 0 or 1 — doing so would break determinism guarantees for seeds that have been produced and recorded. The alpha draw is part of the canonical step computation.

**Chain-seed version-lock**: the prefix `"MSC_STEP_"` embeds algorithm identity. Any change to the step semantics must change this prefix. Old and new manifests are distinguishable; silent seed compatibility across algorithm versions is prevented. **Enforcement**: a test asserts that the prefix constant in source equals `"MSC_STEP_"` and that `step_seed(0, 0, 0)` equals a hard-coded expected value — this test will fail if the prefix or formula is silently changed.

---

## Compositor integration

This is a **Search layer** variant (alongside `short-burst`, `flip`, `bisection-ensemble`):

```rust
SeedCompositor::MultiScale {
    total_steps: usize,    // total fine + coarse steps combined (default: 2000)
    p: f64,                // percentile of visited plans to return (default: 0.0 = min EC)
    alpha: f64,            // probability of coarse move per step (default: 0.3)
}
```

**CLI**:
```bash
redist state --state TX --year 2020 \
  --search multiscale \
  --multiscale-steps 2000 \
  --multiscale-alpha 0.3 \
  --percentile 0.0
```

**YAML**:
```yaml
algorithm:
  structure: prime-factor
  weights: county
  search: multiscale
  multiscale_steps: 2000
  multiscale_alpha: 0.3
  percentile: 0.0
```

**Data requirement**: both adjacency files must exist for the state and year:
- `data/{year}/{state}/{state}_adjacency_{year}.pkl` — tract-level (fine)
- `data/{year}/{state}/{state}_bg_adjacency_{year}.pkl` — block-group-level (coarse)

The CLI must detect `--search multiscale` and load both files before constructing the chain. If the block-group adjacency file is missing, the CLI must error with a clear message:
```
Error: multi-scale MCMC requires block-group adjacency for {state} {year}.
Run: redist fetch --year {year} --resolution block_group
```

---

## Audit chain

Every run with `--search multiscale` records in `runs/{label}/{year}/index.json`:

```json
"search": "multiscale",
"multiscale_steps": 2000,
"alpha": 0.3,
"percentile": 0.0,
"base_seed": 12345678,
"chain_idx": 0,
"fine_steps_taken": 1411,
"coarse_steps_taken": 589,
"fine_acceptance_rate": 0.73,
"coarse_acceptance_rate": 0.41,
"selected_step": 1204,
"step_seed_formula": "SHA-256('MSC_STEP_' || step:u64le || '_' || chain_idx:u32le || '_' || base_seed:u64le)"
```

All fields are named and typed so a verifier can independently reproduce the fine/coarse selection sequence from `base_seed` using the SHA-256 formula. `chain_idx` is 0 for single-chain use and must be recorded. The `selected_step` records which step's plan was returned — it is the 0-based index into the EC-sorted list of ACCEPTED plans (plans where at least one district changed from the previous step). Rejected steps (where all districts are unchanged) do not enter the selection pool. This matches the Forest ReCom and Merge-Split convention. `fine_steps_taken + coarse_steps_taken` must equal `multiscale_steps` exactly.

This ensures `redist label-verify` can confirm the search parameters and seed derivation for any submitted plan.

---

## Test invariants (L0)

- Fine/coarse step selection frequency matches alpha: over 10,000 steps, `coarse_steps_taken / total_steps ≈ alpha ± 0.05` (5% tolerance)
- Fine step produces a valid fine-level plan: contiguous districts, population within `pop_tolerance`
- Coarse step followed by projection and rebalance produces a valid fine-level plan: contiguous districts, population within `pop_tolerance`
- Deterministic: same `base_seed` → identical `fine_steps_taken`, `coarse_steps_taken`, and final plan
- Step count invariant: `fine_steps_taken + coarse_steps_taken == total_steps` after every run
- `p=0.0` returns the plan with minimum EC among all visited plans; `p=1.0` returns the plan with maximum EC (ascending sort — tests both ends to catch inverted sort)
- Prefix version-lock: assert that the step seed prefix constant in source equals `"MSC_STEP_"` and that `step_seed(0, 0, 0)` equals a hard-coded expected value
- Missing block-group adjacency file: CLI errors with the instructional message (not a panic)

---

## Test invariants (L1)

- **Synthetic 2-level hierarchy**: 16-tract grid with 4 block-groups of 4 tracts each, k=2, T=500 steps: all visited plans are valid (contiguous, population-balanced at both fine and coarse levels)
- **alpha=0.0**: alpha=0.0 DOES NOT produce identical output to standard ReCom (the alpha comparison consumes one RNG draw not present in standard ReCom). The correct test: with alpha=0.0, zero coarse steps are taken (`coarse_steps_taken == 0`), and all accepted plans satisfy the fine-level validity constraints. This is verifiable without reference to standard ReCom output.
- **alpha=1.0**: zero fine moves (`fine_steps_taken == 0`), coarse-only chain still produces valid fine-level plans after projection and rebalance for every step
- **Coarse-to-fine projection**: after every coarse move, every fine tract's assignment matches its block-group's coarse assignment (checked exhaustively for each step)
- **Step count**: for any total_steps T, `fine_steps_taken + coarse_steps_taken == T` after the chain completes (invariant checked post-run, not just at the end)

---

## Test invariants (L2, #[ignore])

- **TX 2020 k=38 block-group hierarchy, T=2000 steps**: autocorrelation of EC at lag 100 is lower for `--search multiscale --multiscale-alpha 0.3` than for `--search convergence` (standard ReCom). This verifies the mixing-time benefit claimed in the motivation. Acceptance threshold: autocorrelation at lag 100 ≤ 0.5 for multiscale vs ≥ 0.7 for standard (typical values from Autry et al. 2021).
- **Acceptance rates**: fine acceptance rate ~70% (typical ReCom), coarse acceptance rate 30–50% (coarse moves are harder to satisfy). Values outside these ranges should produce a test warning (not failure) for investigation.

---

## Open questions (deferred)

1. **Three levels**: should the spec support three levels (tract + block-group + county) or fix at two? Defer to Phase 2; this spec covers two-level only.
2. **Coarse balance tolerance**: should coarse balance tolerance be the same as fine or looser? Looser (2×) is recommended since coarse moves are corrected by the projection step; this is the default but not enforced.
3. **Coarse adjacency data availability**: block-group adjacency files must exist (`redist fetch --resolution block_group`). The CLI must check and error with an actionable message if missing (see §Compositor integration).
4. **Projection rebalancing**: when a coarse move violates fine-level balance, boundary swaps are needed. This spec defers to the same boundary-swap logic in `split_subgraph`. The exact implementation is left to the `redist-multiscale` crate author; the invariant (fine-level balance after every coarse step) is testable regardless of implementation.
5. **`--output-all-steps`**: should the CLI support writing all visited plans to disk for diagnostics? Deferred — large output for T=2000 steps.
6. **SMC integration**: could the multi-scale chain serve as the proposal kernel for SMC? Deferred to G.12 or later.
