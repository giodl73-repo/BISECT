# Review: G.3 — Compactness Distribution Position
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 1
**Score**: 3/4

## Summary

G.3 is the paper most directly engaging with the political geography of compact redistricting. The compactness-partisan correlation analysis is relevant to my own research on geographic sorting and its consequences. The paper's finding that $\rho(PP, S_D) \approx -0.09$ to $-0.31$ across states is plausible and the causal interpretation ("the correlation is a property of geography, not of the AR algorithm") is correct. The legal defensibility argument is the paper's strongest practical contribution.

## Strengths

The paper correctly identifies that the compactness-partisan correlation is causal at the geographic level and correlational at the plan level. The distinction ("compact plans do not cause fewer Democratic seats; rather, the same geographic property that makes compact plans produce Republican advantages also determines the correlation within the ensemble") is precisely right and anticipates the obvious challenge that compact redistricting is itself a form of gerrymandering.

The correlation values are plausible: $\rho = -0.31$ for Georgia, $\rho = -0.09$ for Wisconsin. These are consistent with the Rodden-effect literature, where the correlation between compactness and Republican over-representation is moderate in sorted states and weak in competitive states. The monotonic relationship — stronger correlation in more sorted states — is the right prediction from the geographic sorting model.

The legal defensibility argument (Section 6) is well-structured. A plan at the 68th percentile of compactness with the AR plan's specific properties is legally defensible because it is: (1) not at the extreme of any distribution; (2) consistent with a non-partisan objective function; (3) not the result of any optimization targeting partisan outcomes.

## Weaknesses

**The correlation values ($\rho \approx -0.09$ to $-0.31$) are presented without uncertainty intervals and without specifying the sample over which they are computed.** These correlations are computed over ensemble plans ($N = 10{,}000$ to $50{,}000$), but the plans are correlated MCMC draws. The effective sample size for the correlation estimate is again the ESS, not N. For $\ess \approx 700$, a correlation of $-0.18$ has a 95% confidence interval of approximately $-0.18 \pm 1.96/\sqrt{700} \approx -0.18 \pm 0.074$ — meaning the true correlation could be anywhere from $-0.25$ to $-0.11$. For Georgia's $\rho = -0.31$, the interval is $-0.31 \pm 0.074$. These intervals should be reported.

**The assertion that "any compact algorithm applied to a sorted state will produce a similar correlation" is stated as if proven but is only asserted.** This would require comparing the compactness-partisan correlation for, say, minimum-PP-maximisation vs. minimum-edge-cut objectives. The paper does not provide this comparison. It is plausible that the correlation is lower for an algorithm specifically designed to produce compact plans with fewer cross-district minority concentrations — in which case the claim would be wrong.

**The "outlier question" section (Section 4, referenced in the outline but not provided) presumably argues the AR plan is not an outlier on either compactness or partisan dimensions.** But an analysis of whether the joint distribution of (compactness, partisan outcome) has unusual properties for the AR plan is missing. Being at the 68th percentile of compactness AND near the median of partisanship is a joint outcome — what fraction of ensemble plans achieve both simultaneously? This would be more persuasive than the marginal analyses alone.

## Minor Issues

- The explanation of why the ReCom ensemble rarely achieves maximum compactness (plans require boundaries aligned with geographic features) is qualitative. A quantitative analysis of what geographic conditions produce the highest-PP plans in the ensemble would strengthen this section.
- The paper's title "Compact Without Being an Outlier" is accurate but slightly undersells the main finding. The more legally significant finding is that compactness and partisan near-median outcomes are simultaneously achievable — this is not obvious a priori.

## Recommendation

Accept with minor-to-moderate revisions. Add uncertainty intervals to correlation estimates, and add a joint distribution analysis.
