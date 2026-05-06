# Review: G.0 — Ensemble Methodology
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 3/4

## Summary

G.0 is a well-structured framework paper that introduces the conceptual machinery for the G-series. The percentile framework is correctly defined, and the convergence diagnostic overview is competent. From a statistical methodology perspective, the paper is mostly sound, with several issues that warrant attention before the framework is treated as the foundation for downstream claims.

## Strengths

The percentile definition (Eq. 1) is correctly stated as an empirical CDF evaluation. The paper's notation distinguishes the ensemble $\mathcal{E}$, the statistic $f$, and the algorithmic plan $P^*$ cleanly.

The sensitivity analysis subsection (Section 3.4) appropriately acknowledges that the percentile depends on the ensemble choice: chain type, ensemble size, and population tolerance all affect the result. This is a methodologically important caveat that many applied papers skip.

The ESS formula (Eq. 2 in Section 4.2) is correct, and the paper correctly notes that ReCom achieves higher ESS than flip chains of the same length due to faster decorrelation.

The Hamming autocorrelation (Eq. 4) is a useful additional diagnostic. The decision to measure chain mixing at the plan level (rather than just at the level of a scalar statistic) is methodologically superior for redistricting applications where the relevant space is the plan space, not R.

## Weaknesses

**The ESS threshold of 400 is presented without calibration justification.** The paper states "an ESS of at least 400 per statistic is required for reliable 95% interval estimates" but does not derive this. A standard calibration: to estimate the 95th percentile of a distribution with standard error of 1% of the range requires approximately $1/(4 \times 0.05 \times 0.95 \times 0.01^2) \approx 526$ independent samples. So the correct minimum is closer to 530, not 400. G.4 later uses 500 as the threshold, which is more defensible. The framework paper should use 500 or provide a derivation for 400.

**The sensitivity analysis paragraph (Section 3.4) mentions using "distributional summaries (mean, standard deviation, approximate percentile bounds) to bound $\pi_f(P^*)$" for states without published ensembles, but provides no description of the methodology.** Bounding a percentile from distributional summaries requires assumptions (e.g., normality or specific distributional shape). For the right-skewed compactness distribution discussed in G.3, a normal approximation would overstate the percentile. The framework paper should specify the assumptions and their validity conditions.

**The Rhat formula as presented in Section 4.1 is the Vehtari 2021 update, but cited to Gelman 1992.** More importantly, the threshold stated is $\hat{R} < 1.05$ in the body but $\hat{R} < 1.1$ in the table at the end of the section. This inconsistency will cause problems: G.4 uses 1.1 as the primary threshold. Resolve by using 1.1 consistently (the more appropriate threshold for redistricting with discrete integer-valued statistics) and citing Vehtari 2021 for the formula.

**The Hamming autocorrelation definition (Eq. 4) uses a reference plan $P_{\rm ref}$ without specifying how to choose it.** Different choices of reference plan will yield different autocorrelation estimates. The paper should specify: (a) use the initial plan $P_0$ as the reference (most natural), (b) use the AR plan $P^*$ as the reference (most relevant for the G-track purpose), or (c) use a fixed ensemble-median plan. The choice affects the numerical values reported in G.4.

## Minor Issues

- The asymptotic autocorrelation formula in Eq. 3 assumes geometric decay, which is a strong assumption. For redistricting chains, empirical autocorrelation often has a slower tail. The paper should note this assumption.
- The statement "plans at the 5th or 95th percentile are unusual but not necessarily extreme" should be quantified: how often does a non-gerrymandered map appear in the 5th or 95th percentile? This is context-dependent and should not be stated without qualification.
- The table at the end of Section 4 should include a column for "Reference" giving the source for each threshold value.

## Recommendation

Accept with revisions. Fix the Rhat threshold inconsistency, calibrate the ESS minimum (use 500 not 400), specify the Hamming reference plan, and provide assumptions for the distributional bound methodology.
