---
reviewer: COVENANT
role: Audit and Evidence — reproducibility
spec: Ensemble Search Algorithms (G series extensions)
round: 1
score: 2
date: 2026-05-06
---

## Summary

Reproducibility is the weakest part of this spec. Short-Burst's seed specification has a gap (`chain_seed(base, burst)` is cited but never defined), SMC seeding is described as `base_seed: u64` with no specification of how internal resampling steps derive seeds, and none of the four modes documents how their run parameters integrate with the existing SHA-256 audit chain from `redist label-verify`. For a project whose legal defensibility rests on the determinism guarantee, launching four new search modes without specifying their seed protocols is a significant gap — particularly because two of the modes (Short-Burst and SMC) involve multiple layers of derived randomness that must all be reproducible independently.

## Strengths

- The base structure of Short-Burst (`for burst in 0..n_bursts: seed=chain_seed(base, burst)`) shows awareness that burst-level seed derivation must be deterministic and enumerable. This is the right design — it means any individual burst can be re-run independently by computing `chain_seed(base, burst_index)`. The approach parallels the PercentileSweep SHA-256 seed chain design correctly at the conceptual level.
- The Simulated Annealing spec is the most reproducible of the four. The cooling schedule parameters (T_0, T_final, n_steps) are deterministic given fixed inputs, and the proposal sequence is seeded through the same mechanism as the bisection runner. SA does not introduce new sources of non-determinism beyond what the existing bisection infrastructure already manages.

## P1 — Required changes

**`chain_seed(base, burst)` is undefined.** The pseudocode says `seed=chain_seed(base, burst)` but never specifies the derivation function. This is the most critical gap. The PercentileSweep spec defines its seed derivation as `SHA-256(census_release_id || "DIA_SEED_V1" || i)`, which is fully specified and auditable. Short-Burst must have an equivalent: either the same SHA-256 chain extended with a burst index, or a multiplicative spread (e.g., `base.wrapping_add(burst * MAGIC_CONSTANT)`), or some other function. Without this definition, two implementations of Short-Burst will produce different burst sequences even given the same `base_seed`, making the G.6 paper results non-reproducible by third parties.

**The `p`-percentile selection is not independently verifiable as specified.** The pseudocode ends with `return best_plans[floor(p * n_bursts)]`, which appears straightforward. However, the intermediate state — the full list of burst-endpoint plans sorted by objective — is not written to the output JSON. An auditor verifying that the returned plan is actually at rank `floor(p * n_bursts)` of the burst-endpoint distribution cannot do so without re-running all 50 bursts. The spec must require that the `EnsembleResult` JSON include (a) the burst-endpoint objective values (e.g., `burst_ec_values: [f64; n_bursts]`), and (b) the selected rank. This allows `redist label-verify` to recompute the selection without re-running the chains.

**SMC seeding is not specified.** The spec gives `base_seed: u64` as a function parameter but does not describe how this seed propagates through the SMC algorithm's internal operations: (1) the initial particle generation step (which draws n_particles independent plans from the prior), (2) the sequential resampling steps (which resample the particle population), and (3) the mutation steps applied after resampling. Each of these steps requires independent pseudo-random draws, and the derivation of the sub-seeds from `base_seed` must be fully specified for reproducibility. The existing SHA-256 chain protocol (as used in PercentileSweep) should be extended to SMC with explicit documentation of the mapping `base_seed → particle_seed[i] → resample_seed[t]`. Without this, SMC results will vary across platforms with different PRNG implementations, and the G.7 paper results will not be independently reproducible.

**Audit chain compatibility is not documented for any of the four modes.** The existing `redist label-verify` audit chain records the SHA-256 of the plan manifest, the seed chain, and the build commit hash. None of the four new modes specifies what additional fields they contribute to the audit record. At minimum, the spec must document:
- Short-Burst: `base_seed`, `burst_length`, `n_bursts`, `p`, `burst_ec_values` added to manifest.
- Flip: `base_seed`, `steps`, `p`, `accept` (including `beta` if MH) added to manifest.
- SA: `n_steps`, `cooling` (type + parameters) added to the bisection-node manifest entry.
- SMC: `base_seed`, `n_particles`, `resample_threshold`, `particle_weights` (final) added to a new `SmcManifest` section.

Without these additions, `redist label-verify` cannot verify any run that uses the new modes, breaking the audit chain for any G-series paper that uses Short-Burst, Flip, or SMC.

## P2 — Suggested improvements

- Add a `--reproducibility-check` mode for each new search algorithm (analogous to how `--print-only` provides a dry-run sanity check for the pipeline). This would re-run a single burst/step/particle from a known-good fixture and compare the output, allowing CI to verify that the PRNG implementation has not changed across Rust compiler versions or target platforms.
- For SMC specifically, the particle weight output in `SmcResult` should include both the unnormalised and normalised weights, plus the ESS at each resampling step. This allows an auditor to verify that the ESS threshold triggered resampling at the expected steps without re-running the full algorithm.
- The Flip chain's `p`-percentile selection (return plan at rank `floor(p * steps)` from visited plans) implies that all visited plans are stored in memory for the duration of the run. For 10,000 steps on NC (208 tracts), this is ~2,000 plans × ~200 tracts × 4 bytes = ~1.6 MB — acceptable. But the spec should confirm that all visited plans are retained, not just a reservoir sample, or specify the reservoir sampling strategy if memory is a concern.

## Score: 2/4

The `chain_seed` underspecification and missing audit chain integration are correctness and legal defensibility issues, not style preferences. A G.6 paper based on a Short-Burst implementation with an unspecified `chain_seed` function cannot be independently reproduced. The SMC seeding gap is equally serious. These must be resolved before implementation begins.
