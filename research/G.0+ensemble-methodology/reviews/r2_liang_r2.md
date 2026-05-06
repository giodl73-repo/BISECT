# Review: G.0 — Ensemble Methodology
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 2
**Score**: 3/4

## Summary

The revision addresses the most important statistical issues from Round 1. The Rhat threshold inconsistency is resolved, the Vehtari 2021 citation is added, and the Hamming reference plan is now specified via cross-reference to G.4. The ESS threshold is still 400 rather than 500, which is a missed item from the REVISION-PLAN, but this is the only remaining substantive issue.

## Changes Evaluated

**Rhat threshold (1.05 vs. 1.1):** Resolved. Both occurrences now read $\hat{R} < 1.1$, and the footnote justifying 1.1 for discrete redistricting statistics (vs. Vehtari 2021's 1.01 recommendation for continuous parameters) is precisely calibrated. This is a meaningful addition that will prevent misapplication of the threshold.

**Rhat citation:** Vehtari 2021 is now the primary citation for the formula. The original Gelman 1992 is retained as the background reference. Correct.

**Hamming reference plan:** The cross-reference to G.4 Section 4 is adequate. G.4 specifies $P_{\rm ref} = P_0$ (initial plan).

**CS bridge:** The revised framing is technically correct from a statistical standpoint. ESS is a property of an estimator from a target distribution; T=600 is a termination rule for a deterministic search. They are indeed orthogonal and should not be numerically compared. The paper now says so.

**Distributional bound methodology:** Still not specified. Section 3.4 mentions using distributional summaries to bound percentiles for states without published ensembles, but the assumptions (normality? Cornish-Fisher? skewness correction?) are not stated. For right-skewed compactness distributions, a normal approximation can overstate the percentile by several points. This is a residual concern from Round 1.

## Remaining Issue

**ESS threshold is still 400 (Section 4.2).** G.4 uses 500. The REVISION-PLAN (item H3) explicitly calls for changing to 500 "consistent with G.4." The derivation from $1/(4\alpha(1-\alpha)\delta^2)$ mentioned in the REVISION-PLAN has not been added. This is a direct missed item. It is not blocking for G.0 itself since G.4 governs the calibration, but the framework paper should not contradict G.4.

The derivation is straightforward: for $\alpha = 0.95$ and $\delta = 0.01$ (1-percentile precision), ESS $\geq 1/(4 \times 0.05 \times 0.95 \times 0.0001) \approx 526$, rounded to 500 as a conservative practical threshold. Add this in one sentence.

## Recommendation

Accept with minor revisions. Change ESS threshold to 500 with the derivation. Add distributional assumption statement to Section 3.4. All blocking and most high-priority issues from Round 1 are resolved.
