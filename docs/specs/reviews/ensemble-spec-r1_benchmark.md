---
reviewer: BENCHMARK
role: Test Engineer — testability
spec: Ensemble Search Algorithms (G series extensions)
round: 1
score: 2
date: 2026-05-06
---

## Summary

The spec is thin on testability. It adds four new algorithms with distinct invariants but provides no test targets, no integration point specifications precise enough to write fixture-driven tests, and no account of how the existing L0/L1/L2 pattern from PercentileSweep and BisectionEnsemble would extend to the new modes. Two structural decisions — the SMC-as-separate-crate design and the ambiguous `--search` vs `--structure` split for SA — actively complicate the test harness. A spec this test-sparse will produce an implementation that cannot be verified without running a full redistricting pipeline, which defeats the purpose of the L0 unit layer.

## Strengths

- Flip is the most testable of the four algorithms. The step invariants (boundary-tract stays adjacent to its new district, population balance maintained, contiguity preserved after every flip) are mechanically checkable with a four-tract toy graph, and the spec's description of the accept/reject rule is precise enough to write a Metropolis-Hastings acceptance test.
- The `EnsembleResult`-compatible JSON output guarantee (mentioned in the "Connection to existing work" section) provides a single integration point to test — all four modes must produce JSON that passes the same schema validator.

## P1 — Required changes

**The `--search short-burst` integration point is not specified precisely enough to write tests.** The spec lists the CLI flag and YAML key, but does not specify (a) how the burst-endpoint plans are serialised mid-run for inspection, (b) whether there is a `--dry-run` or `--output-all-bursts` flag that would allow a test to verify the burst-endpoint distribution without running a full NC ensemble, or (c) what the JSON output schema looks like for Short-Burst specifically (does it include all burst endpoints, or only the selected plan?). Without this, the only way to test Short-Burst correctness is to run 50 bursts of 20 ReCom steps on a real graph and inspect the output — a multi-minute integration test, not an L0 unit test.

**The SMC-as-separate-crate architecture makes the test harness harder.** The spec says `redist-smc` is a new crate because "the algorithm is fundamentally different." This is architecturally reasonable, but the test implications are not addressed. The existing L0/L1/L2 test pattern assumes each search mode is a variant of `SeedCompositor` and can be driven by the same test driver with different enum arms. SMC is explicitly not a `SeedCompositor` variant — it runs as a standalone `redist ensemble --method smc` call. This means the SMC integration tests cannot reuse the `SeedCompositor`-based test scaffolding and will require a separate test module, separate fixture format, and separate CI job. The spec must document this divergence and either (a) specify the SMC test harness design, or (b) justify why SMC should be a `SeedCompositor` variant after all (which would simplify the test architecture at the cost of one additional enum arm in `redist-ensemble`).

**Key invariants are absent for each mode.** The spec does not enumerate the invariants that every test should verify. Based on the algorithm descriptions, the required invariants are:

- **Short-Burst**: (1) All burst-endpoint plans are valid (population-balanced, contiguous). (2) The returned plan is at rank `floor(p * n_bursts)` of burst endpoints sorted by objective. (3) Each burst produces exactly `burst_length` ReCom steps from a fresh chain seeded by `chain_seed(base, burst)`. Invariant (3) cannot be verified without a `--output-all-bursts` flag or a test-mode hook.
- **Flip**: (1) After every accepted flip, the boundary-tract set is updated. (2) After every accepted flip, all districts remain contiguous. (3) Metropolis-Hastings acceptance uses the correct ratio (proposal probability is symmetric for boundary flips, so the ratio simplifies to min(1, exp(-β·ΔEC))). Invariant (2) is the critical one — contiguity breaks are a known failure mode for Flip chains, and the spec does not specify a contiguity check, only "check population balance + contiguity" without detail.
- **Simulated Annealing**: (1) Every accepted proposal is a valid plan. (2) The cooling schedule is monotonically decreasing. (3) At T→0, only improvements are accepted (degenerate to greedy). Invariant (3) is testable with a toy graph at T_final → 0.
- **SMC**: (1) All returned plans have positive weight. (2) Weights sum to approximately n_particles (before normalisation). (3) ESS at resample_threshold=0.5 triggers resampling at the correct step. Without a seeded test, (3) cannot be verified.

**The SA compositor placement creates a test-layer ambiguity.** SA is in the Structure layer (`--structure simulated-annealing`) while the other three modes are in the Search layer (`--search flip` etc.). This means the test for SA must invoke `redist state --structure simulated-annealing` rather than `redist state --search simulated-annealing`, which is correct but requires test fixtures that exercise the bisection runner, not just the search layer. The spec does not flag this in the implementation plan, and a developer following the spec could easily wire SA into the wrong compositor slot.

## P2 — Suggested improvements

- Add a `--max-bursts-output N` flag (or `--debug-bursts` mode) that writes all N burst-endpoint plans to a sidecar JSON file. This would enable L0 tests that inspect the burst distribution without running a full ensemble.
- Specify a minimum test graph (e.g., 9-tract, 3-district grid) that fits in L0 test fixtures. The ReCom spec uses NC (208 tracts) as a benchmark, which is too large for an L0 test. The Flip step invariants should be verified on a graph where all possible valid and invalid moves can be enumerated.
- The `FlipAccept::MetropolisHastings { beta: f64 }` parameter should be testable with a deterministic RNG fixture. The spec should note that tests must seed the RNG to avoid flaky acceptance tests.

## Score: 2/4

The absence of invariant specifications, the SMC crate split without test harness documentation, and the unverifiable Short-Burst burst-endpoint claim mean that implementation will proceed without a clear definition of correctness. The L0/L1/L2 pattern that works well for PercentileSweep and BisectionEnsemble cannot be applied to these modes without the missing specification. This requires a revision pass specifically focused on testability.
