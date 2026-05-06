# Revision Plan — Ensemble Search Algorithms Spec
**Spec**: `docs/specs/2026-05-06-ensemble-search-algorithms.md`
**Round**: 1
**Date**: 2026-05-06

---

## Scores

| Reviewer | Role | Score |
|---|---|---|
| MERIDIAN | Computational Geographer — algorithm correctness | 2/4 |
| BENCHMARK | Test Engineer — testability | 2/4 |
| SURVEY | Practitioner — usability | 3/4 |
| COVENANT | Audit and Evidence — reproducibility | 2/4 |
| **Average** | | **2.25 / 4** |

**Decision**: Major revisions required. Three of four reviewers scored 2/4. The Short-Burst algorithm contains a fundamental correctness error (MERIDIAN), the seed derivation is underspecified in a way that breaks independent reproducibility (COVENANT), and the spec lacks the invariant documentation needed to write L0 tests (BENCHMARK). SURVEY's concerns are fixable in one revision pass without design changes. The spec cannot be approved at the current score.

---

## Consolidated P1 Items

Items raised by at least one reviewer as required before approval. Items raised by multiple reviewers are flagged with a count.

---

### A. Short-Burst algorithm error: min-within-burst vs endpoint-of-burst (raised by: MERIDIAN)

**Problem**: The pseudocode runs `plans.min_by(objective)` inside the burst loop, which is a greedy scan over burst steps, not the Cannon et al. 2022 algorithm. The correct algorithm keeps the plan at the *end* of each burst, not the minimum plan within the burst. The greedy-scan variant deforms the target distribution and invalidates the G.6 paper's claim that "Short-Burst outperforms long-chain ReCom."

**Required fix**: Replace `best_plans.push(plans.min_by(objective))` with `candidates.push(chain.current_plan())`. The `p` parameter then selects a rank from the burst-endpoint distribution, not a within-burst scan. Update the conceptual description accordingly: "pick the plan at the end of each burst" not "pick the best from each burst."

---

### B. `chain_seed(base, burst)` is undefined (raised by: COVENANT, MERIDIAN)

**Problem**: The Short-Burst pseudocode references `chain_seed(base, burst)` but the function is never defined. The PercentileSweep spec defines its seed derivation as `SHA-256(census_release_id || "DIA_SEED_V1" || i)`. Without an equivalent specification, two implementations of Short-Burst will produce different burst sequences given the same `base_seed`.

**Required fix**: Define `chain_seed(base, burst)` explicitly. The recommended approach, consistent with PercentileSweep's SHA-256 protocol: `chain_seed = SHA-256(base_seed || "SHORT_BURST_V1" || burst_index)` truncated to u64. Document this in the spec alongside the PercentileSweep seed formula reference.

---

### C. Audit chain integration absent for all four modes (raised by: COVENANT)

**Problem**: None of the four new modes documents what fields they contribute to the `redist label-verify` audit record. The existing audit chain records seed chain, build commit hash, and plan manifest SHA-256. The new modes add new parameters (burst_length, n_bursts, n_particles, cooling schedule, beta) and new derived values (burst-endpoint EC values, particle weights) that must be in the manifest for the audit to be verifiable.

**Required fix**: Add a subsection to each mode specification listing the fields added to the audit manifest:
- Short-Burst: `base_seed`, `burst_length`, `n_bursts`, `p`, `burst_ec_values: [f64]`, `selected_rank: usize`
- Flip: `base_seed`, `steps`, `p`, `accept` (type + beta if MH)
- SA: `n_steps`, `cooling` (type + T_0 + T_final) per bisection node
- SMC: `base_seed`, `n_particles`, `resample_threshold`, `final_particle_weights: [f64]`, `ess_per_step: [f64]`

---

### D. SMC seeding unspecified (raised by: COVENANT)

**Problem**: `base_seed: u64` is listed as a function parameter but no derivation of sub-seeds for particle generation, resampling, and mutation steps is given. SMC involves three distinct sources of randomness that must each be reproducible.

**Required fix**: Specify the sub-seed derivation: `particle_seed[i] = SHA-256(base_seed || "SMC_PARTICLE_V1" || i)`, `resample_seed[t] = SHA-256(base_seed || "SMC_RESAMPLE_V1" || t)`. Document in the spec that reproducibility requires all three seed streams to be fixed.

---

### E. SA cooling schedule underspecified for subgraph size (raised by: MERIDIAN)

**Problem**: Default T_0=1.0, T_final=0.01 with n_steps=1000 gives a decay rate that is too slow for subgraphs larger than ~100 tracts. The spec does not indicate whether these values are node-local (per bisection step) or global. For NC at the first bisection level (~208 tracts), 1000 steps from T=1.0 to T=0.01 will not converge to a good solution.

**Required fix**: State explicitly that `n_steps` and the cooling parameters are applied per-bisection-node, not globally. Provide a recommended scaling: n_steps ≈ 5–10× (tracts in the current subgraph). Add a note that the defaults are appropriate for subgraphs of ~50–200 tracts and should be increased for the first bisection level in large states.

---

### F. Invariants absent — no L0 test targets (raised by: BENCHMARK)

**Problem**: The spec does not enumerate the invariants each test should verify. Without these, the test author must reverse-engineer correctness criteria from the algorithm description. The existing L0/L1/L2 test pattern for PercentileSweep and BisectionEnsemble relies on explicit invariant lists in the spec.

**Required fix**: Add an "Invariants" section to each mode specification:
- Short-Burst: (1) All burst-endpoint plans are valid (balanced, contiguous). (2) Returned plan is at rank `floor(p * n_bursts)` of burst endpoints sorted by objective. (3) Each burst uses exactly `burst_length` ReCom steps seeded by `chain_seed(base, burst)`.
- Flip: (1) After every accepted flip, all districts remain contiguous. (2) After every accepted flip, population balance is within tolerance. (3) MH acceptance uses ratio `min(1, exp(-β·ΔEC))`.
- SA: (1) Every accepted proposal is a valid plan. (2) Temperature is monotonically non-increasing. (3) At T=0, only improving proposals are accepted.
- SMC: (1) All returned plans have positive weight. (2) Resampling triggers when ESS < `resample_threshold * n_particles`. (3) Weights normalise to 1.0.

---

### G. `--burst-length` vs `--ensemble-steps` flag collision (raised by: SURVEY)

**Problem**: PercentileSweep defines `--ensemble-steps` for local ReCom steps per bisection node in BisectionEnsemble. Short-Burst's `--burst-length` is conceptually similar but measures something different (steps per standalone burst vs steps per bisection-node ensemble). A user combining both modes in a single config will be confused, and a user of only one mode may misinterpret the other flag.

**Required fix**: Document the distinction in the CLI reference section: "`--burst-length` (Short-Burst): steps per independent burst chain. `--ensemble-steps` (BisectionEnsemble): steps per local 2-way ReCom ensemble at a single bisection node. These flags are mutually exclusive — each applies to its respective `--search` mode and is ignored when the other mode is active." Add a validation check in the CLI parser that warns if both are specified.

---

### H. Burst-endpoint plans not in output JSON (raised by: COVENANT, BENCHMARK)

**Problem**: The selected plan's rank within the burst-endpoint distribution cannot be independently verified without re-running all bursts. The audit chain requires the burst EC values to be in the output.

**Required fix**: Add `burst_ec_values: Vec<f64>` and `selected_burst_rank: usize` to the Short-Burst section of `EnsembleResult` JSON. This allows `redist label-verify` to recompute the selection without re-running the chains.

---

## P2 Items (Suggested — Not Required for Approval)

- **SA restart strategy** (MERIDIAN): Document restart-to-best-known as a tuning option for the cooling schedule.
- **Mode selection guidance** (SURVEY): Add a decision table covering optimisation target, compute budget, legal defensibility, and connection to ConvergenceSweep/PercentileSweep for each mode.
- **SMC `--steps 1` clarification** (SURVEY): The CLI example `redist ensemble --method smc --particles 5000 --state NC --steps 1` is confusing — clarify what `--steps` means in an SMC context (resampling rounds? output plans?).
- **Runtime estimates** (SURVEY): Add a runtime table analogous to the ReCom spec's performance table for each mode at VT/NC/TX tract counts.
- **`--output-all-bursts` debug flag** (BENCHMARK): Add a debug mode that writes all burst-endpoint plans to a sidecar JSON for L0 test inspection.
- **Minimum test graph specification** (BENCHMARK): Specify a 9-tract 3-district grid as the canonical L0 test fixture for Flip and SA, consistent with the level of detail in the PercentileSweep spec.
- **SMC `chain_seed` extends PercentileSweep protocol** (COVENANT): Note explicitly that the SMC seed design follows the same `SHA-256(id || tag || index)` protocol as PercentileSweep, not a distinct PRNG design, to ensure audit-chain consistency across the codebase.
- **Reproducibility-check mode** (COVENANT): Add a `--reproducibility-check` flag for each mode that re-runs a minimal fixture and compares output to a golden file.

---

## Recommended Action

Revise spec to address all 8 P1 items (A–H) before implementation begins. Items A (Short-Burst algorithm correctness) and B (chain_seed definition) are the highest priority — A is a scientific correctness error and B blocks independent reproducibility. Items C–H are specification completeness issues that can be addressed in a single revision pass.

Items A+B together affect the G.6 paper's core claim. Item D affects the G.7 paper's reproducibility. Item C affects the legal defensibility of any G-series filing that uses the new modes.

Target: revised spec ready for Round 2 review within 3–5 days.
