# Review 5 — Reviewer: Christina Liang (Computational Social Science / Causal Inference)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 1
**Score:** 3/4

## Summary

A well-executed empirical study on parameter sensitivity in algorithmic redistricting. The main finding is believable and the methodology is transparent. The paper would benefit from stronger statistical inference — the current analysis is largely descriptive (reporting ranges and differences from baseline) rather than inferential. That said, for the policy audience this paper targets, the descriptive approach is defensible.

## Strengths

The study design is appropriate for the research question. OAT sensitivity analysis is the standard approach for parameter screening in computational models, and the five-level grid for each parameter is adequate for detecting any monotone or near-monotone effects. The connection to Morris (1991) elementary effects method is correctly cited.

The runtime analysis for the ConvergenceSweep threshold (Table 3) is a practical contribution that is often missing from algorithm papers: showing that T=600 provides an adequate safety margin at a 54-minute wall time for 50 states gives implementers the information they need to make deployment decisions.

The statistical noise context — sigma ≈ 0.8 seats from B.7 seed variance — is the right denominator. Placing the 0.3-seat maximum partisan effect against a 0.8-seat noise floor shows that parameter effects are undetectable against algorithmic noise. This is the clearest presentation of the main finding.

## Weaknesses and Concerns

The analysis lacks formal statistical inference. For each parameter, the paper reports the range of D_nat and computes the standardized sensitivity index, but does not test whether any of the observed effects are statistically different from zero. Given that the baseline seed variance is 0.8 seats (from B.7), a 0.2-seat shift in D_nat is well within the noise — but this should be established with a formal test, not just by comparison.

Specifically: for each parameter, fit a simple regression D_nat = a + b * parameter_value, test the null H0: b = 0, and report the p-value. Given sigma ≈ 0.8 and 5 data points per parameter, the power to detect a 0.2-seat effect will be low — but that is precisely the point. The paper should show that the effects are statistically indistinguishable from zero given the available data, not just that they are small in an absolute sense.

The description of the sweep design is slightly ambiguous about the number of random seeds per run. Each "run" is described as "a full 50-state sweep using the redist states command" — does this use a single seed or the best seed from a ConvergenceSweep? If it uses a single seed, the D_nat variance across runs at the same parameter setting is not zero (it is approximately sigma from B.7). If it uses the ConvergenceSweep best-of-T seeds, the variance is much smaller. This matters for interpreting the 0.1-seat differences in Tables 1-5: are these real parameter effects or just seed noise?

The paper should clarify: at each parameter setting, how many independent runs were conducted? If only one run per setting, then the 0.1-seat differences in the tables have no standard error and cannot be distinguished from noise.

## Minor Issues

- The "population neutrality" argument (Section 5.3) would be strengthened by a correlation analysis: what is the actual correlation between tract-level partisan lean and tract population across all 50 states? If this correlation is near zero, the neutrality argument is empirically grounded.
- The paper does not report the actual computational cost of the sweep (26 runs × ~54 minutes per run at T=600 ≈ 23 hours total). Reporting this confirms the feasibility of the analysis and helps readers assess whether a full factorial design (3,125 runs) is indeed computationally expensive or merely expensive by the standards of an academic exercise.
- The recommendation to use `ufactor = 1%` as "the smallest value that consistently succeeds across all 50 states" (Section 7.2) implies that ufactor = 0.1% caused METIS failures in some states. Which states? This is important practical information that should be reported.

## Recommendation

Accept with minor revisions. Clarify the seed handling per parameter setting (single run vs. ConvergenceSweep), add a formal regression test for the null of zero parameter effect, and document the ufactor=0.1% METIS failure states.
