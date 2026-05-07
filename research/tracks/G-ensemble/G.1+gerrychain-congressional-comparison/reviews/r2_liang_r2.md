# Review: G.1 — GerryChain Congressional Comparison
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 2
**Score**: 3/4

## Summary

The NC PP score correction is the single most important change and it resolves the major inconsistency I flagged in Round 1. The corrected score of 0.337 is arithmetically consistent with the ensemble parameters and the reported empirical percentile. The attribution fixes (Autry 2021 for TX, author-estimate for CA) are appropriate. However, the uncertainty quantification — which I identified as a critical issue — has still not been corrected. The "negligible sampling error" claim remains in Section 3.4, and the formula still uses nominal $N$ rather than ESS.

## Changes Evaluated

**NC PP score (Issue 1): Resolved.** The corrected value 0.337 gives $z = 0.61$, which is consistent with 68th to 73rd percentile depending on skewness correction. This is the right fix. The internal arithmetic no longer collapses.

**Source attribution (Issues 3, 4):** The revised attribution language is adequate. "Calibrated to published figures" is honest. The GA result (38th Democratic seat percentile) is presented with the DeFord 2021 methodology notation, which is consistent with the clarified attribution.

## Critical Unresolved Issue

**Uncertainty quantification (Issue 2): Not resolved.** Section 3.4 still states "For direct comparisons (NC, WI, GA, PA), the percentile estimate has negligible sampling error given $N \geq 10{,}000$." This sentence is factually incorrect. The Herschlag NC chain has $N = 24{,}518$ but with lag-1 autocorrelation $\rho_1 \approx 0.87$, the effective sample size is approximately $\ess \approx 1{,}703$. The uncertainty in reported percentiles is $\sqrt{24{,}518/1{,}703} \approx 3.8\times$ wider than using nominal $N$. For the NC 54th Democratic seat percentile, the correct 90% CI is $54 \pm 5.5$, not $54 \pm 1.5$.

The word "negligible" is directly contradicted by the ESS analysis in G.4. Either:
(a) Replace "negligible" with the correct ESS-based uncertainty intervals, or
(b) Delete the uncertainty claim entirely and defer to G.4 for the quantification.

The paper cannot use $N = 24{,}518$ as if these were independent draws — this was my Issue 2 in Round 1 and it is unchanged.

## Additional Note

The notation inconsistency I flagged ($\bar{PP}$ vs. $\bar{pp}$) appears to be resolved in the revised version; the notation is now consistently $\bar\pp$ throughout. Good.

## Recommendation

Accept with revision. The blocking data error is fixed. Fix the uncertainty quantification by replacing "negligible sampling error" with ESS-corrected intervals or a deferral to G.4. This is a one-paragraph change but is statistically mandatory.
