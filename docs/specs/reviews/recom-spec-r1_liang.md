---
reviewer: Percy Liang
spec: redist-ensemble ReCom crate
round: 1
score: 2
date: 2026-05-06
---

## Summary
The spec describes a reproducibility-critical piece of infrastructure — an ensemble sampler whose outputs will be cited in research papers and potentially in legal proceedings — but the JSON output format, seed specification, and chain independence guarantees are incomplete to the point where independent reproduction would not be feasible from the spec alone. The performance targets are stated without a benchmark methodology, making the 2,500x claim unverifiable.

## Strengths
- The JSON output schema includes the key summary statistics (mean, std, p5, p95, plan percentile, R-hat, ESS) that are sufficient for reproducing the top-level claims of the G-series papers. This is better than many ensemble tool outputs, which provide only histograms.
- The integration of Hamming autocorrelation alongside R-hat and ESS is the right diagnostic triple for MCMC redistricting: R-hat checks inter-chain agreement, ESS checks intra-chain efficiency, and Hamming autocorrelation checks the step-to-step mixing rate specific to the large-move ReCom proposal.
- The phased implementation plan (Phase 1: core algorithm; Phase 2: CLI + output; Phase 3: diagnostics; Phase 4: paper replication) correctly sequences the work so that the core sampler is validated before the output format is finalised.

## P1 — Required changes

- **The seed specification is insufficient for reproducibility.** The spec states that `RecomChain` contains a `SmallRng` field but does not specify: (a) how the seed is set (random from OS entropy, or user-specified?); (b) whether the seed is recorded in the JSON output; (c) how parallel chains receive different seeds (Rayon threads cannot safely share an RNG, so each chain needs its own seeded RNG, but the seeding strategy — e.g., seed + chain_index, or independent draws from a master RNG — is not specified). For a tool whose output is cited in legal proceedings, exact reproducibility requires that the seed for every chain is recorded in the output and that the seeding strategy is deterministic given the master seed. The spec must add a `seed` field to `EnsembleResult` and specify the seeding protocol for parallel chains.

- **The 2,500x speedup claim is not benchmarked.** The performance table compares GerryChain 1,000 steps against the Rust target for 10,000 steps, but the numbers are self-reported targets, not measured benchmarks. The GerryChain baseline (~47s for NC 1K steps) is plausible based on published GerryChain benchmarks, but the Rust target (0.1s for NC 10K steps) implies ~47,000 steps/second for NC — a 2,350x speedup. This claim requires: (a) a benchmark harness that measures actual Rust throughput (steps/second) for each state; (b) a comparison on identical hardware using the same graph format; and (c) a statement of the hardware configuration. Without this, the 2,500x claim is unverifiable and will be challenged in peer review. The spec must specify that Phase 2 includes a benchmark suite and that the performance table will be updated with measured values before the spec is approved.

- **The ESS of 4,821 for NC 10,000-step ensemble needs justification.** An ESS of 4,821 out of 10,000 nominal steps (ESS ratio ~0.48) implies moderate autocorrelation, which is consistent with published GerryChain results for cut-fraction in NC. However, the spec presents this as a single number without stating: (a) which statistic the ESS is computed on (cut-fraction? seat count? both?); (b) whether this is the chain-pooled ESS or the per-chain ESS; (c) whether 4,821 is sufficient for the tail quantiles (p5, p95) used in the output, which require higher ESS than the mean. For tail quantile estimation, standard practice requires ESS > 400 per quantile; with 4 chains and 10K steps each, the pooled nominal N is 40,000, so an ESS of 4,821 is marginal for p5/p95 estimation. The spec should add a power analysis or cite a published standard for minimum ESS per quantile.

## P2 — Suggested improvements

- The JSON output format should include the full `hamming_autocorr` vector (lags 1–20 are listed in the struct) rather than truncating with `"..."`. Any consumer of the output who wants to reproduce the autocorrelation plot needs the full vector, not a summary. Add a note that the JSON schema is the complete output contract.

- The spec should specify a minimum 1,000-step burn-in period that is excluded from all statistics, with `burn_in` as a CLI flag (default 1000) and a field in the JSON output. GerryChain uses a warm-up period for exactly this reason. Without burn-in, the initial assignment (the METIS-generated plan) will bias the ensemble statistics toward the plan being audited, which is precisely the opposite of what the audit is meant to achieve.

## Score: 2/4
The seed specification gap and the unverified 2,500x claim are disqualifying for a reproducibility-critical tool. The spec needs a full revision that adds a complete seed/reproducibility protocol and replaces the target performance table with a benchmark methodology before implementation begins.
