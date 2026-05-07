# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 3/4

## Summary

G.2 provides a focused analysis of partisan outcome distributions for the six study states. From a statistical methodology standpoint, the paper is better structured than G.1, but inherits G.1's data problems. The corridor framework is methodologically sound. The robustness-to-baseline analysis is a welcome addition.

## Strengths

The corridor framework is clean and replicable: the definition (Eq. 2) is unambiguous, the corridor fraction (Eq. 3) is correctly defined as a simple count, and the results table provides all necessary information to verify the claims.

The robustness analysis in Section 3.3 — that AR percentile rankings are stable within 5 percentage points across 2016, 2018, and 2020 electoral baselines — is an important methodological contribution. Electoral baseline sensitivity is a standard concern in ensemble redistricting papers and this section addresses it directly.

The p_geo parameter ($p_{\rm geo} = 0.50$ for Wisconsin vs. $0.36$ for Georgia) is useful for explaining why the same algorithm produces different partisan percentiles in different states. This is a clean, intuitive summary of the geographic sorting effect.

## Weaknesses

**The overdispersion claim in Eq. 1 is incorrect in direction.** The paper models ensemble seat distributions as Binomial($k$, $p_{\rm geo}$) with overdispersion. Overdispersion means variance > $kp(1-p)$. But for sorted states with strong contiguity constraints, the ensemble distribution often has less variance than the binomial prediction — the constraints narrow the feasible plan space. The paper should either (a) empirically verify that ensemble SDs exceed binomial SDs for each state, or (b) drop the overdispersion claim and use a more appropriate model (e.g., a beta-binomial with the actual observed overdispersion/underdispersion factor).

**The sensitivity analysis claims stability "within 5 percentage points" across baselines but provides no data.** The reader cannot verify this claim without the actual percentile values for each baseline. A table with three columns (2016, 2018, 2020 percentile) for each of the six states would make this falsifiable. As stated, it is asserted.

**The corridor fraction estimates (42%, 61%, 28%, 55%) are presented without uncertainty intervals.** Since these are proportions from a dependent MCMC sample, the effective sample size argument applies. For a chain with $\ess \approx 1{,}000$ and $\hat{\phi} = 0.42$, the 90% confidence interval is approximately $0.42 \pm 1.645\sqrt{0.42 \times 0.58 / 1{,}000} \approx 0.42 \pm 0.026$. This is a $\pm 2.6$ percentage point interval — meaningful given the interpretation. The paper should report confidence intervals for corridor fractions.

**Minnesota is introduced in Section 4.3 without a source for the ensemble data (44th percentile, $\phi = 59\%$).** If this is estimated from the AR plan alone (using the distributional model from Section 2), the methodology must be stated and the result labeled as an estimate.

## Minor Issues

- The standard deviation estimates ($\sigma_{S_D} \approx 1.0$–$1.5$ for $k = 8$–$17$) in Section 2.2 are stated without source. For a framework that underlies litigation-level claims, these need citations or derivations.
- The statement "$p_{\rm geo} \approx 0.36$ for Georgia" is not clearly derived. The statewide Democratic vote share in 2020 was approximately 46.4% — but $p_{\rm geo}$ refers to the probability that a random geographically valid district leans Democratic, which depends on how Democratic voters are spatially distributed. The paper should explain how $p_{\rm geo}$ is estimated.

## Recommendation

Accept with minor-to-moderate revisions. Fix the overdispersion direction, add data for the baseline sensitivity claim, add uncertainty intervals for corridor fractions, and source the Minnesota result.
