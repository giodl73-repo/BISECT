# Review: G.3 — Compactness Distribution Position
**Reviewer**: George Karypis (Graph partitioning, METIS)
**Round**: 1
**Score**: 3/4

## Summary

G.3 situates the AR plan within the compactness distribution of ReCom ensembles. The right-skewed distribution shape, the explanation of why METIS edge cut does not directly maximize Polsby-Popper, and the compactness-partisan correlation analysis are all technically correct. This is the paper where the AR algorithm's properties are most naturally discussed from a graph-partitioning perspective, and the analysis reflects a sound understanding of what METIS optimizes.

## Strengths

The explanation of why AR falls at the 61st–72nd PP percentile rather than near the maximum is technically precise and correct: METIS minimizes edge cut (number of cross-district adjacencies), which correlates with smooth boundaries, which correlates with high Polsby-Popper, but the correlation is not perfect. Plans with minimum edge cut can have irregular boundaries if the tract adjacency graph happens to route around geographic features. This is exactly right.

The right-skew analysis (Section 2.1) correctly attributes the distributional shape to the ReCom proposal's tendency to produce jagged boundaries via random spanning-tree cuts. The skewness range (+0.3 to +0.7) is cited as empirical and is plausible given the literature.

The compactness-partisan correlation table ($\rho \approx -0.09$ to $-0.31$) is the paper's most empirically interesting contribution. The direction is correct: in geographically sorted states, compact districts tend to produce Republican outcomes. The correlation is small but non-zero.

## Weaknesses

**The correlation calculation in Section 5.2 contains a dimensional error.**
The formula for $\Delta S_D$ due to compactness positioning uses:
$$\Delta S_D \approx \rho \cdot \frac{\sigma_{PP}}{\sigma_{S_D}} \cdot (0.68 - 0.50)$$
The last term $(0.68 - 0.50) = 0.18$ is a percentile difference. But in the regression framework, the correct formula for the expected change in $S_D$ given a one-unit change in the Z-score of PP is:
$$\Delta S_D = \rho \cdot \frac{\sigma_{S_D}}{\sigma_{PP}} \cdot \Delta \bar{PP}$$
where $\Delta \bar{PP}$ is the absolute change in mean PP, not the percentile difference. The paper conflates these. Using $\Phi^{-1}(0.68) \approx 0.47$ as the Z-score and then converting to actual PP units gives $\Delta \bar{PP} = 0.47 \times \sigma_{PP} = 0.47 \times 0.031 \approx 0.015$. Then $\Delta S_D = (-0.18) \times (1.1/0.031) \times 0.015 \approx -0.10$ seats. The direction is the same but the formula derivation in the paper is wrong.

**The "between 75th percentile and ensemble maximum" claim requires more careful support.** The paper states the AR plan "falls between the 75th percentile and the ensemble maximum for all states" (Section 2.3). But the stated AR percentiles are 61st–72nd, and the 75th percentile values in the table are (for NC) 0.355 vs. AR value 0.412. This is consistent with AR being above the 75th percentile in NC but the paper states the percentile is 68th (below 75th). This appears internally inconsistent: either the AR value exceeds the 75th percentile (implying AR is above the 75th percentile in compactness) or the AR percentile is 68th (implying AR is below the 75th percentile in rank). The paper cannot claim both.

**The Autry 2021 Metropolized Forest ReCom is mentioned in G.0 as addressing the non-reversibility issue.** G.3 should discuss whether the Metropolized version changes the compactness distribution shape — since the Metropolis correction could shift the stationary distribution toward higher-compactness plans. If Autry's ensemble has a different compactness distribution than standard ReCom, then using standard ReCom ensemble results to assess the AR plan's position requires justification.

## Minor Issues

- The skewness values (+0.3 to +0.7) are described as "empirically observed" but no source is given. Are these from the author's own chain runs or from the literature?
- The ensemble maximum values ($\approx 0.49$–$0.52$) are described as representing plans that "align with major geographic features (rivers, mountain ranges)." This is an interesting claim that deserves a citation or a concrete example — which specific NC plan achieves PP ≈ 0.52?

## Recommendation

Accept with moderate revisions. Fix the dimensional error in the correlation calculation and resolve the 75th percentile/68th percentile inconsistency.
