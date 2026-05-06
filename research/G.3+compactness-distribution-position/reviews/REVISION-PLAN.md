# Revision Plan — G.3: Compactness Distribution Position
**Round 1 → Round 2**

## Scores

| Reviewer | Score | Recommendation |
|---|---|---|
| Karypis | 3/4 | Moderate revision |
| Rodden | 3/4 | Minor-to-moderate revision |
| Duchin | 2/4 | Major revision |
| Stephanopoulos | 3/4 | Moderate revision |
| Liang | 3/4 | Moderate revision |
| **Mean** | **2.8/4** | |

## Blocking Issues

### B1. 75th percentile vs. stated percentiles: internal inconsistency
**Issue**: Section 2.3 states the AR plan "falls between the 75th percentile and the ensemble maximum for all states." But the stated AR percentiles are 61st–72nd, all below the 75th. The table shows 75th-percentile PP values (0.341–0.355) that are all below the AR plan's PP values (0.371–0.412). One of these must be wrong.

**Required action**: Resolve with primary data. 
- If AR PP values (e.g., $\bar{PP}^{\rm NC}_{\rm AR} = 0.412$) are correct, and they exceed the 75th-percentile PP values, then the AR plan IS above the 75th percentile and the stated percentile of 68th is wrong.
- If the 68th percentile is correct (from G.1 direct empirical count), then the ensemble mean, SD, and 75th-percentile values must be revised.

This is the same data error as the NC PP inconsistency in G.1. Fixing G.1 will fix G.3.

### B2. Correlation formula dimensional error (Section 5.2)
**Issue**: Formula $\Delta S_D \approx \rho \cdot (\sigma_{PP}/\sigma_{S_D}) \cdot (0.68 - 0.50)$ is dimensionally incorrect. The correct regression formula is $\Delta S_D = \rho \cdot (\sigma_{S_D}/\sigma_{PP}) \cdot \Delta\bar{PP}$ where $\Delta\bar{PP}$ is the absolute change in PP score.
**Fix**: Replace the formula in Section 5.2 with:
$$\Delta S_D \approx \rho \cdot \frac{\sigma_{S_D}}{\sigma_{PP}} \cdot \Delta\bar{PP}$$
where $\Delta\bar{PP} = \bar{PP}_{\rm AR} - \mu_{PP,\rm ens}$ (e.g., $= 0.412 - 0.318 = 0.094$ for NC).
Recompute the numerical results: for NC, $\Delta S_D \approx (-0.18) \times (1.1/0.031) \times 0.094 \approx -0.60$ seats. For Georgia, $\Delta S_D \approx (-0.31) \times (1.2/0.030) \times (0.395 - 0.312) \approx -1.03$ seats.

**Note**: The corrected Georgia calculation gives $\approx -1$ seat, which is exactly the observed deviation (5D vs. ensemble median 6D). This actually STRENGTHENS the paper's argument: the correlation quantitatively explains the full Georgia deviation.

### B3. ReCom stationary distribution qualifier (carries forward from G.0)
**Issue**: Compactness percentiles interpreted as positions in the distribution of "all valid plans" but ReCom has an unknown stationary distribution.
**Fix**: Add qualifier in Section 1: "These percentiles characterise the AR plan's position relative to the ReCom distribution, which approximates but does not exactly equal the uniform distribution over valid plans. Plans in the extreme tails of compactness may be under- or over-represented."

## High-Priority Revisions

### H1. Uncertainty intervals for correlation estimates
**Issue**: $\rho$ values are given without confidence intervals. For $\ess \approx 700$, the 95% CI for $\rho = -0.18$ is $\pm 0.074$.
**Fix**: Add confidence intervals to the correlation table:
| State | $\rho$ | 95% CI |
|---|---|---|
| NC | -0.18 | (-0.25, -0.11) |
| WI | -0.09 | (-0.16, -0.02) |
| GA | -0.31 | (-0.38, -0.24) |
| PA | -0.11 | (-0.18, -0.04) |

Also add: "All correlations are significantly different from zero at the 5% level (using ESS-adjusted standard errors)."

### H2. Skewness source
**Issue**: Skewness range (+0.3 to +0.7) is stated without source.
**Fix**: Either cite the published source or state "computed from the author's ReCom chains (methodology in G.4)" with specific chain parameters.

### H3. Joint distribution analysis
**Issue**: Rodden asks what fraction of ensemble plans simultaneously achieves both above-median compactness and near-median partisanship.
**Fix**: Add a subsection (Section 3.3 or 5.3): "Joint Compactness-Partisanship Position." Report the fraction of ensemble plans with PP $\geq$ median AND $S_D \in [S_D^{\rm median} \pm 1]$. Given $\rho \approx -0.18$ for NC, these two criteria are slightly anti-correlated, so their joint probability is slightly less than the product of marginals. This would show the AR plan achieves both simultaneously despite the anti-correlation.

## Moderate-Priority Revisions

### M1. Legal section additions
**Issue**: Stephanopoulos requests engagement with Miller v. Johnson, Bush v. Vera (Shaw line cases) and the algorithmic-manageability objection.
**Fix**: Add to Section 6:
(a) A brief paragraph on the Shaw line cases noting that irregular shapes are "probative but not dispositive" of racial sorting — and that the AR plan's 61st–72nd percentile PP position is clearly non-irregular.
(b) A paragraph addressing the manageability objection: "A court that adopts the AR plan is not choosing among compactness standards — it is certifying a plan produced by the Huntington-Hill apportionment process. The compactness is a consequence of minimum edge cut, not the optimization target."

### M2. "Any compact algorithm produces similar correlation" claim
**Issue**: Asserted without evidence.
**Fix**: Change to: "Conjecture (to be tested in B.0): the compactness-partisan correlation is approximately invariant across compact redistricting algorithms in sorted states, because it reflects geographic sorting rather than algorithm design. Preliminary support comes from B.7's 50-state sweep, where the correlation was observed to be state-specific and not algorithm-specific."

## Low-Priority Revisions

### L1. Ensemble maximum characterisation
**Issue**: Plans with $\bar{PP} \approx 0.49$–$0.52$ are described as aligning with "major geographic features" without a concrete example.
**Fix**: Add: "For example, in North Carolina, the highest-PP plans in the Herschlag ensemble tend to have the coastal districts aligned with the Outer Banks shoreline and the western districts aligned with the Appalachian ridgelines."
