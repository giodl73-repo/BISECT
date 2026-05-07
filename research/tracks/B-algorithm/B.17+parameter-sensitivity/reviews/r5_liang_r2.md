# Review 5 — Reviewer: Christina Liang (Computational Social Science / Causal Inference)
**Paper:** B.17 — How Sensitive Are Redistricting Outcomes to Algorithm Parameters? A 50-State Sweep
**Round:** 2
**Score:** 3/4

## Summary

My Round 1 concerns centered on statistical inference (the absence of formal tests) and the seed handling ambiguity. The revision addresses both: the methodology section now clarifies that each run uses the ConvergenceSweep best-of-T result, which substantially reduces seed variance; and the binomial test results are now explicitly reported. The paper is ready for acceptance. My score remains at 3/4 — the statistical inference is improved but not complete, and the minor issues I raised in Round 1 are only partially resolved.

## Addressed Issues

The seed handling clarification is now in Section 3.1: each pipeline run uses the ConvergenceSweep best-of-T seeds, so D_nat values are optimized outcomes rather than single-seed artifacts. This is the clarification I needed. The implication — that the 0.1-seat differences in Tables 1-5 are the differences between optimized outcomes at different parameter settings, not between single-seed outcomes — is now clearly stated.

The binomial test results are reported: all five parameters, all non-baseline comparisons, produce non-significant Holm-corrected p-values. The paper correctly interprets this as showing that the effects are statistically indistinguishable from zero at the available sample size. This is exactly the inferential conclusion I requested.

The METIS failure states at ufactor=0.1% are now documented in a footnote: Alaska, Wyoming, and Montana fail to converge at ufactor=0.1%, which is the reason the operational floor is 0.5%. This is important practical information.

## Remaining Concerns

The formal regression test for the null of zero parameter effect (fitting D_nat = a + b × parameter_value and testing H0: b = 0) is still not present. The paper reports the binomial test, which tests whether more-vs-less favorable settings produce more-vs-less Democratic outcomes, but does not test whether the slope of the D_nat-parameter relationship is distinguishable from zero. Given σ ≈ 0.8 and 5 data points, the power would be low — but that is precisely the point, as I noted in Round 1. The paper's reliance on the binomial test is defensible but not equivalent to the regression null test I requested.

The total computational cost of the sweep is now partially reported (26 runs × ~54 minutes = ~23 hours), which addresses my minor issue. However, the population-partisan correlation analysis that would empirically ground the neutrality claim is still absent. This is noted also by Rodden and I maintain it as a desirable addition.

The ConvergenceSweep seed variance at different T levels (specifically, whether T=200 has higher seed variance than T=600) is noted in the revision but not quantified. The paper says the outcomes are "identical" for T≥200, but identical means and different variances are consistent — and I was specifically asking about variance, not mean.

## Minor Issues

- The computational cost table (26 runs × 54 min) implies the analysis takes approximately one day of wall time. The paper should confirm whether this is parallelizable (each run is independent and can be distributed across machines).
- The regression null tests remain the cleaner statistical approach to the zero-effect claim, and I would recommend adding them even if only as a footnote.

## Recommendation

Accept. The seed handling clarification and binomial test reporting address the key issues from Round 1. The formal regression null test would strengthen the inferential foundation, but the binomial test approach is defensible for the policy audience this paper targets.
