# Review: G.3 — Compactness Distribution Position
**Reviewer**: Jonathan Rodden (Political geography, partisan sorting)
**Round**: 2
**Score**: 3/4

## Summary

The correlation formula correction resolves the main technical problem and, importantly, produces a result that is stronger than the original: the corrected calculation for Georgia gives −1.03 seats, which is exactly the observed deviation. This is now a quantitative, not merely qualitative, explanation of the Georgia result. From a political geography standpoint, this is a meaningful finding: the compactness-partisan correlation in the Rodden-sorted Georgia ensemble fully accounts for the AR plan's deviation from the seat-share median.

## Changes Evaluated

**Correlation formula (B2):** The fix is correct and the result is actually more compelling post-correction. The original wrong formula gave −0.12 seats for Georgia (explaining only part of the deviation), while the correct formula gives −1.03 seats (explaining all of it). This changes the paper's conclusion from "the correlation accounts for part of the Georgia deviation" to "the correlation quantitatively explains the full Georgia deviation." This is a significant strengthening.

The NC calculation now gives −0.12 seats (rounds to zero), which is consistent with NC's observed result of being near the partisan median. The corrected formula is internally consistent across both states.

**NC table update:** Consistent with G.1. Good.

## Outstanding Issues

**H1 (Correlation uncertainty): Not added.** The correlation table still has no confidence intervals. For $\ess \approx 700$, all four correlations are significantly different from zero at the 5% level. Adding the CIs would actually strengthen the paper's argument by demonstrating statistical significance — the failure to add them is puzzling.

The REVISION-PLAN provided exact values:
- NC: (-0.25, -0.11)
- WI: (-0.16, -0.02)
- GA: (-0.38, -0.24)
- PA: (-0.18, -0.04)

These are a one-table addition that should not require any new analysis.

**H3 (Joint distribution analysis): Not added.** The fraction of ensemble plans simultaneously achieving above-median compactness and near-median partisanship is not reported. Given the corrected $\rho = -0.18$ for NC and that the two criteria are anti-correlated, the joint probability is slightly below the product of marginals. This is a clean empirical question that would demonstrate the AR plan achieves a jointly favorable position despite the anti-correlation.

**"Any compact algorithm produces similar correlation" (M2): Still asserted.** The revised text adds "Conjecture (to be tested in B.0)" which is better than the original assertion. This is adequate for Round 2.

## Recommendation

Accept with minor revisions. Add the correlation confidence intervals (a small table already specified in the REVISION-PLAN), and ideally the joint distribution analysis. The paper's core contribution — quantitative explanation of the Georgia deviation via the corrected formula — is now a genuine empirical result.
