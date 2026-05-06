---
reviewer: Percy Liang
round: 1
score: 2
date: 2026-05-05
---

## Summary

B.6 presents theoretical and empirical complexity results for METIS recursive bisection applied to congressional redistricting. The empirical contributions are strong: a well-specified runtime measurement protocol, a convincing OLS scaling fit, and cross-census stability. The theoretical contributions — NP-hardness and the approximation ratio — are presented as proof sketches, and from a methodology perspective, proof sketches without full proofs are unverifiable. For an empirical ML/CS venue, the empirical section would be acceptable as-is; for a theory venue, the proofs need to be complete and correct. My technical concerns about the proofs (raised in detail by other reviewers) are real, but even setting those aside, the paper has empirical methodology issues that need addressing.

## Strengths

- **Measurement protocol is fully specified.** METIS version 5.1.0, C FFI, ufactor=5, niter=100, seed=42, single-threaded, AMD Ryzen 9 5950X, 10-run average, std::time::Instant — this is the complete specification needed for replication. The paper correctly uses a fixed seed (42) to eliminate seed-to-seed runtime variability.
- **The OLS fit is methodologically correct.** Fitting log T = log a + b log n by OLS on log-transformed data is the correct approach for power-law scaling. Reporting R^2 = 0.984 and b ± one standard error (1.07 ± 0.03) is appropriate. Cross-census stability (b ≈ 1.07 for 2000, 2010, 2020) is a strong robustness check.
- **The space complexity numerical example is reproducible.** The calculation (8,057 nodes × 6 levels × 64 bytes = 3 MB) can be independently verified from the California tract count and the METIS data structure specification. This is the kind of concrete, checkable claim that strengthens a paper.

## Weaknesses / P1 Items (Required Fixes)

- **The OLS regression mixes k=1 and k>1 states without accounting for the k factor.** Table 2 includes Vermont, Delaware, and Wyoming with k=1 (trivial partition, essentially no computation). The theoretical runtime is O(n log n log k), so for k=1, log k = 0 and runtime should be near-zero regardless of n. Including these states in a power-law regression of T vs. n will bias the intercept and exponent estimates. The regression should either exclude k=1 states (with a note explaining their trivial nature) or include log(k) as an additional covariate, fitting T = a * n^b * k^c. If c ≈ 1 (linear in log k), this would confirm the theoretical prediction and strengthen the empirical result.
- **The 10-run average does not report variance.** Table 2 reports "mean of 10 runs" but gives no standard deviation or coefficient of variation for any state. For a runtime claim, the variability of the measurement matters — if runtime varies by ±20% across runs (due to thermal throttling or cache effects on a desktop workstation), the 10-run average is not reliable. Even reporting min/max across the 10 runs would suffice.
- **The empirical scaling claim does not test the log k prediction.** The theoretical bound is O(n log n log k). The empirical fit of T vs. n alone conflates n and k growth (as the paper acknowledges: "the residual super-linearity is attributable to the log k factor: larger states tend to have more districts"). But this is not tested — it is asserted. A two-variable regression of log T against log n and log(log k) would directly test whether the log k term accounts for the super-linearity. This is a straightforward analysis given the existing data.

## P2 Items (Suggestions)

- **Provide raw data as a supplementary table.** The paper reports 14 states in Table 2 but claims the full 50-state fit. Providing all 50 data points (state, n, k, mean runtime, std dev) as a supplementary table would make the OLS fit independently verifiable and allow readers to test alternative models.
- **Test the scaling at block-group resolution.** The paper claims O(n^1.07) scaling on tract data (n ≤ 8,057 per state). Block-group resolution extends this to n ≤ ~25,000. If the same exponent holds at block-group resolution, that would strongly confirm the near-linear claim. The discussion notes this as future work, but it is a straightforward extension that would significantly strengthen the empirical contribution.

## Score: 2 — Major Revision

The regression methodology issues (k=1 conflation, missing variance, untested log k prediction) are empirical problems that require reanalysis. Combined with the theoretical proof issues identified by other reviewers, the paper needs substantial revision before acceptance.
