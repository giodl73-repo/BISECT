# R1 Review — Albert Liang
**Paper**: U.11 Resolution-Aware Redistricting: Geographic Granularity as a First-Class Parameter
**Score**: 2/4
**Verdict**: Major Revision

## Summary
This paper introduces a resolution system for redistricting pipelines and provides correctness proofs for GEOID prefix-based partition derivation. The technical contributions are sound, but the empirical section (Section 5) falls significantly short of the standards expected for a GIS venue. The 27% autocorrelation reduction claim is based on a single 2000-step run with a single seed, which is insufficient to estimate lag-100 autocorrelation reliably. The manifest system validation is correct but trivial. Stronger empirical methodology is required before this paper can be accepted.

## Strengths
- The `derive_partition` correctness proof is well-structured and addresses the orphan-detection edge case explicitly — this is the right level of rigor for an auditable tool.
- The `build_county_coarsening` complexity analysis is careful: it separately accounts for derive, population aggregation, and edge deduplication, and correctly distinguishes the $O(|E_T| \log |E_T|)$ sort-based and $O(|E_T|)$ expected hash-based deduplication paths.
- The manifest system (Section 4) is a practical contribution: the `fine_to_coarse_formula` field encodes the partition rule as a human-readable string, enabling independent verification without access to pipeline source code.
- The three-option taxonomy (A/B/C) is well-organized and the Phase 2 delineation is honest about implementation status.

## Concerns
- **27% autocorrelation claim from single 2000-step run**: The headline result of the paper is a 27% reduction in lag-100 Hamming autocorrelation. This figure is footnoted as "Estimated value from a single 2000-step run, seed $s=42$." Lag-100 autocorrelation requires approximately 10× the lag (i.e., ~1000 steps minimum) to produce a stable estimate from a single chain, and that estimate has high variance without multiple chains or longer runs. The paper reports 8 parallel chains, but it is unclear whether the single 2000-step run uses all 8 chains or just one. At 2000 total steps with 8 chains, each chain has only 250 steps — far too short to estimate lag-100 autocorrelation. The paper must clarify the run configuration and provide multi-run estimates with confidence intervals.
- **Manifest system reproducibility**: Section 5.2 validates the manifest system by checking SHA-256 hash, unit count, and field presence. This is a correct but trivial validation — it checks that the manifest was written correctly, not that the manifests produced by independent reruns agree. A stronger validation would run the same configuration twice from the same seed and verify that both manifests are identical. The paper should include this reproducibility check.
- **No run-to-run variability reported**: The empirical section does not report variance across chains or runs. For Table 1, only mean and max over chains are reported, with no standard deviation or confidence interval. For the 27% reduction to be meaningful, the variance of $\rho_{100}$ across chains and runs must be small relative to the claimed 27% effect. With 8 chains of 250 steps each, the variance is likely large.
- **Computational overhead claimed as "negligible"**: Section 5.1 claims that building $\mathcal{E}_C$ takes approximately 12ms and represents "< 0.1% overhead" relative to a 50,000-step run. This overhead claim is marked as estimated (`\est`) without specifying what hardware was used, what the total run time was, or whether 12ms was measured or calculated. The paper should report the actual measured overhead from the benchmark run.

## Required Changes (P1/P2)
- **P1**: Clarify the run configuration for the autocorrelation experiment: how many steps per chain, how many chains, and how was lag-100 autocorrelation estimated? A 2000-step run with 8 chains gives 250 steps per chain — report whether this is the configuration used and, if so, acknowledge that this is insufficient for stable lag-100 estimates. Either extend the run or remove the 27% figure from the abstract and headline.
- **P1**: Report variance (standard deviation or 95% confidence interval) across chains for $\rho_{100}$ in Table 1. A mean reduction without variance is not a publishable empirical result.
- **P2**: Add a reproducibility check to the manifest validation: run the same configuration twice, verify the manifests are identical, and report this as evidence that the manifest system is deterministic.
- **P2**: Report actual measured (not estimated) computational overhead for `build_county_coarsening` with hardware specification, or remove the overhead claim from the empirical section.
