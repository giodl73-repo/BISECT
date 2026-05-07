# Review: G.3 — Compactness Distribution Position
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 1
**Score**: 3/4

## Summary

G.3 provides a focused analysis of the AR plan's position in the ReCom compactness distribution. The distributional analysis is generally competent. The correlation analysis is valuable but contains a formula error. The paper's main empirical contribution — establishing that $\rho(PP, S_D) \approx -0.09$ to $-0.31$ — is important for understanding the interaction between compactness and partisan outcomes.

## Strengths

The Eq. 1 formulation of the skewness claim ($+0.3$ to $+0.7$) is appropriately presented as an empirical observation. The right-skew analysis of why ReCom tends to produce jagged boundaries is mechanistically sound.

The correlation table for $\rho(PP, S_D)$ is the most credible quantitative contribution in the paper — the values are plausible, the direction is theoretically predicted, and the monotonic relationship with sorting is exactly what one would expect.

The distinction between "AR does not target PP directly" and "AR's edge-cut objective correlates with PP" is methodologically important and correctly stated.

## Weaknesses

**The formula for $\Delta S_D$ in Section 5.2 has dimensional issues.** As computed:
$$\Delta S_D \approx (-0.18) \cdot \frac{0.031}{1.1} \cdot 0.18 \approx -0.09$$
The term $0.031/1.1 \approx 0.028$ has units PP-unit/seat (the ratio of PP SD to seat SD). Multiplying by the dimensionless 0.18 gives a quantity in PP-units, not seats. The result $-0.09$ happens to be numerically small but is dimensionally wrong. The correct bivariate regression formula gives:
$$E[\Delta S_D \mid \Delta \bar{PP}] = \rho \cdot \frac{\sigma_{S_D}}{\sigma_{PP}} \cdot \Delta\bar{PP}$$
where $\Delta\bar{PP} = 0.412 - 0.318 = 0.094$ for NC. This gives:
$$\Delta S_D = (-0.18) \cdot (1.1/0.031) \cdot 0.094 \approx -0.60 \text{ seats}$$

This is much larger than stated and inconsistent with the NC result showing AR near the partisan median. The error needs resolution.

**The 75th percentile inconsistency across Sections 2.3 and the G.1/G.3 stated percentiles must be resolved.** The paper states in Section 2.3 that the AR plan "falls between the 75th percentile and the ensemble maximum for all states." But the stated AR percentiles are 61st–72nd, all below the 75th. The table in Section 2 gives 75th percentile PP values of 0.341–0.355, but the AR PP values (0.371–0.412) are all above these 75th-percentile values. There is a clear inconsistency: if AR PP = 0.412 > 75th-pct PP = 0.355 for NC, then AR is above the 75th percentile — but the stated percentile is 68th.

**The skewness range (+0.3 to +0.7) lacks a source.** If this is computed from the author's ReCom chains, it should be reported with standard errors. If it is taken from the literature, the citation should be provided.

**The correlation estimates lack uncertainty intervals.** For $\ess \approx 700$ and $\hat{\rho} = -0.18$, the 95% CI is approximately $\pm 0.074$. For $\hat{\rho} = -0.31$, it is similar. These are non-negligible and should be reported.

## Minor Issues

- The paper should verify whether the Georgia $\rho = -0.31$ is significantly different from zero at the ESS-adjusted sample size. At $\ess = 700$, the test statistic for $H_0: \rho = 0$ is $t = (-0.31)\sqrt{698}/\sqrt{1-0.31^2} \approx -8.7$ — highly significant. The NC value of $-0.18$ is similarly significant ($t \approx -4.8$). Reporting significance would strengthen the correlation analysis.
- The implication drawn from the correlation analysis — that "one cannot avoid the Rodden effect by choosing a different algorithm" — is a testable claim. The paper should either reference evidence for this claim or label it as a conjecture.

## Recommendation

Accept with moderate revisions. Fix the correlation formula (this changes the numerical results), resolve the 75th/68th percentile inconsistency, and add uncertainty intervals to the correlation estimates.
