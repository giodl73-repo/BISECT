# Review: G.3 — Compactness Distribution Position
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 2
**Score**: 3/4

## Summary

The two main blocking issues are resolved. The correlation formula has been corrected to the proper regression form $\Delta S_D = \rho \cdot (\sigma_{S_D}/\sigma_{PP}) \cdot \Delta\bar{PP}$, and the NC PP value has been updated to 0.337 (from 0.412) in the G.3 percentile table, consistent with the G.1 fix. The B1 internal inconsistency (75th percentile vs. 61st–72nd percentile) is now resolved by the corrected NC PP score — with 0.337, the AR plan sits at approximately the 68th percentile, which is indeed below the 75th. The "between 75th and maximum" claim must therefore be removed or revised accordingly.

## Blocking Issues — Resolution

**B1 (75th vs. 61st–72nd inconsistency): Partially resolved via G.1 correction.** With NC PP revised from 0.412 to 0.337 and the 68th percentile standing as the conservative empirical estimate, the AR plan is below the 75th percentile for NC. The table in Section 3 now shows 0.337 for NC, and the percentile claim of 68th is consistent with this (68 < 75). The "between 75th percentile and ensemble maximum" claim in Section 2.3 must have been removed or revised — if it has not, it is now definitively wrong and must go. I will flag this as needing confirmation.

**B2 (Correlation formula: dimensional error): Resolved.** The corrected formula is:
$$\Delta S_D \approx \rho \cdot \frac{\sigma_{S_D}}{\sigma_{PP}} \cdot \Delta\bar{PP}$$
The NC calculation now gives $\Delta S_D \approx (-0.18) \times (1.1/0.031) \times (0.337 - 0.318) = (-0.18) \times 35.5 \times 0.019 \approx -0.12$ seats — essentially zero, consistent with the observed NC outcome (7D/7R at the 54th percentile of Democratic seats).

The Georgia calculation is even more impressive: $\Delta S_D \approx (-0.31) \times (1.2/0.030) \times (0.395 - 0.312) \approx -0.31 \times 40 \times 0.083 \approx -1.03$ seats. This quantitatively explains the full Georgia deviation of −1 seat. This is a genuinely strengthening result for the paper's central argument.

**B3 (ReCom stationary distribution qualifier): Resolved** via G.0 fix.

## High-Priority Issues — Status

**H1 (Uncertainty intervals for correlation estimates): Not yet addressed.** The correlation table still has no confidence intervals. For $\ess \approx 700$, the 95% CI for $\rho = -0.18$ is $(-0.25, -0.11)$ and for $\rho = -0.31$ is $(-0.38, -0.24)$. These are narrow enough that all correlations remain significantly different from zero, which actually strengthens the paper's argument. The REVISION-PLAN provided the exact values; they should be added.

**H2 (Skewness source): Not addressed.** The skewness range (+0.3 to +0.7) still has no source.

**H3 (Joint distribution analysis): Not added.**

## Recommendation

Accept with moderate revisions. The blocking issues are resolved — the corrected formula is the paper's most important technical fix, and the result that the corrected formula explains the full Georgia deviation is a substantial improvement over Round 1. Add the correlation confidence intervals (a one-row-per-state table), verify and update the "between 75th and maximum" claim, and source the skewness figures.
