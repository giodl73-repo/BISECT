# Review: G.3 — Compactness Distribution Position
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 2/4

## Summary

G.3 addresses one of the most important methodological questions in ensemble redistricting: the relationship between compactness and partisan outcomes. The paper has a genuine empirical contribution in the $\rho(PP, S_D)$ estimates, but it has several serious technical problems — including a dimensional error in the correlation calculation, an internal inconsistency between stated percentiles and quartile positions, and an inadequate treatment of what the ReCom compactness distribution actually represents. These issues must be resolved before the paper can serve as the basis for legal claims.

## Critical Issues

**Issue 1: The stated percentiles (61st–72nd) are inconsistent with the "between 75th percentile and ensemble maximum" claim.**
Section 2.3 states: "The AR plan falls between the 75th percentile and the ensemble maximum for all states." But the AR percentiles reported in G.1 and repeated in G.3 are: NC 68th, WI 72nd, GA 65th, PA 61st. All four are BELOW the 75th percentile. The 75th percentile values in the table are: NC 0.355, WI 0.342, GA 0.348, PA 0.341. The AR values are: NC 0.412, WI 0.388, GA 0.395, PA 0.371 — all above the 75th percentile values.

This means either (a) the percentile values (61st–72nd) are wrong, or (b) the 75th percentile values in the table are wrong. Checking NC: if AR is at the 75th percentile (not 68th), the table entry "75th pct = 0.355" would mean that only 25% of plans have PP above 0.355, but AR has PP = 0.412. The z-score would be $(0.412 - 0.318)/0.031 = 3.03$ and the 75th percentile would correspond to z = 0.67, meaning PP $\approx 0.318 + 0.67 \times 0.031 = 0.339$ — not 0.355. The inconsistency is present throughout the table.

This suggests that either the AR PP values, the ensemble mean/SD, or the stated percentiles in G.1 contain a systematic data error. The paper's repeated assertion that AR is at the "61st–72nd percentile" while also being "between the 75th percentile and the ensemble maximum" cannot be reconciled without knowing which numbers are correct.

**Issue 2: The correlation calculation formula is dimensionally incorrect.**
Section 5.2 uses the formula:
$$\Delta S_D \approx \rho \cdot \frac{\sigma_{PP}}{\sigma_{S_D}} \cdot (0.68 - 0.50)$$

The term $(0.68 - 0.50) = 0.18$ is dimensionless (a percentile difference). But $\sigma_{PP}/\sigma_{S_D}$ has units seats/PP-unit. The product $\rho \cdot (\sigma_{PP}/\sigma_{S_D}) \cdot 0.18$ is then $(PP\text{-unit}/\text{seat}) \cdot 0.18$ (dimensionless) — which is not in seats. The correct formula for the regression-based expected change is:
$$\Delta S_D = \rho \cdot \frac{\sigma_{S_D}}{\sigma_{PP}} \cdot \Delta\bar{PP},$$
where $\Delta\bar{PP}$ is the absolute change in the AR plan's PP score relative to the ensemble mean: $\Delta\bar{PP} = 0.412 - 0.318 = 0.094$ for NC. Then:
$$\Delta S_D = (-0.18) \cdot (1.1/0.031) \cdot 0.094 \approx -0.60 \text{ seats}.$$

This is substantially larger than the paper's result of $-0.09$ seats — it would imply that the AR plan's compactness level predicts a 0.6-seat Democratic deficit relative to the ensemble mean. This would be inconsistent with the NC result showing AR at the 54th percentile of Democratic seats. The formula error needs to be resolved — either the formula or the numbers must change.

**Issue 3: The paper treats the ReCom compactness distribution as if it represents "all valid plans" without qualification.**
The stationary distribution of the ReCom chain is not the uniform distribution over all valid plans — it is an approximately proportional distribution that depends on the spanning-tree probabilities. Compact plans may be over- or under-represented in the ReCom distribution relative to the true uniform distribution. Without knowing the direction of this bias, we cannot interpret the 68th percentile of the ReCom distribution as "more compact than 68% of valid plans." The paper should acknowledge this limitation, which propagates from the G.0 framework issue.

## Secondary Issues

- The skewness values (+0.3 to +0.7) should be sourced — are these from the author's chains or the literature?
- The claim that the AR plan "does not target Polsby-Popper directly" is correct. The further claim that "a district with minimum edge cut will have few cross-district census-tract adjacencies, which correlates with a smooth boundary, which correlates with high Polsby-Popper" is a chain of correlations, not a derivation. A formal bounding argument would strengthen this.

## Recommendation

Major revision required. The 75th-percentile/68th-percentile inconsistency must be resolved with primary data. The correlation formula must be corrected.
