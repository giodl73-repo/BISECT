---
reviewer: Percy Liang
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.5 presents a systematic empirical comparison of recursive bisection and direct k-way partitioning across 727 chamber configurations. The experimental design is transparent and the scope is appropriate for the claim. The paper is reproducible in principle — the METIS version, hyperparameters, hardware, and data sources are specified. My concerns are about the depth of the empirical analysis: the paper reports aggregate means but does not adequately characterise uncertainty, the figure referenced in Section 4.2 is absent ("not shown; see supplementary data"), and the 727-chamber count is not verified against the described methodology. These are fixable issues that reduce confidence in the results without invalidating them.

## Strengths

- **Reproducibility specification.** The paper specifies METIS version (5.1.0), C FFI interface, hyperparameters (ufactor=5, niter=100, ncuts=5, seed=42), hardware (AMD Ryzen 9 5950X), and data source (2020 Census TIGER/Line). This is the minimum required for independent replication and the paper meets it. The reference to the DIA seed formula for reproducibility is appropriate.
- **Cross-resolution confirmation.** Testing at both tract and block-group resolution and finding consistent results is good empirical practice. The ~3x runtime increase at block-group resolution is consistent with the ~3x increase in n, providing a sanity check on the scaling.
- **10-seed averaging for runtime.** Averaging runtime over 10 independent seeds before reporting is correct practice for noisy measurements. The paper correctly notes that timing is measured as wall-clock time on single-threaded METIS.

## Weaknesses / P1 Items (Required Fixes)

- **Figure 1 is absent.** Section 4.2 states "Figure 1 (not shown; see supplementary data) plots PP_RB - PP_NW against k for all chambers." A paper claiming that "the advantage is broadly flat across k in [1, 100] with slight narrowing at k > 80" cannot support this claim with a missing figure. This is the central empirical result for the prime-k and large-k analyses. Either include the figure or replace the claim with the tabular data that supports it. "See supplementary data" is not acceptable for a peer-reviewed submission.
- **Confidence intervals are not reported.** Table 1 reports mean PP and Cohen's d but no confidence intervals. Table 3 reports runtime means but no standard deviations or confidence intervals. For paired t-test results, reporting the 95% CI on the mean difference is standard. The current presentation makes it impossible to assess whether the 0.003–0.004 PP difference would replicate in a different census year or with a different seed. Cross-census replication (2000, 2010, 2020) would be straightforward given that the data exist.
- **The 727-chamber count cannot be verified.** Section 3.1 describes congressional (435 districts, 50 states), state senate (49 chambers, Nebraska excluded), and state house (50 chambers) configurations. The math gives at most 50 + 49 + 50 = 149 states-by-chamber-type combinations, not 727. The abstract says "727 chamber configurations" but the methodology does not account for this number. If 727 includes both census resolutions × both census years × chamber types, this must be stated explicitly. If it refers to individual district-level runs, that too must be clarified.

## P2 Items (Suggestions)

- **Provide a replication archive or script.** The paper specifies all parameters needed for replication, but providing a shell script or Makefile target (e.g., `redist state --chamber senate --year 2020 --alpha-c 1.0`) would make independent replication tractable. Even a README describing the command sequence would improve the paper's reproducibility posture.
- **Report effect size separately for 2000, 2010, 2020.** The paper reports results for 2020 Census data. If the 0.003–0.004 PP advantage is consistent across all three census years, reporting this would substantially strengthen the conclusion that the effect is a structural property of RB on planar graphs rather than an artifact of the 2020 configuration.

## Score: 3 — Minor Revision

The missing figure (P1.1) and absent confidence intervals (P1.2) are the primary issues. The chamber-count discrepancy (P1.3) needs resolution. These are fixes that do not require new experiments — the data appear to exist, the presentation is incomplete.
