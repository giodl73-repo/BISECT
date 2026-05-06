# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Percy Liang (ML, statistical methodology)
**Round**: 2
**Score**: 3/4

## Summary

The overdispersion correction resolves the most important statistical issue from Round 1. The direction is now correct (underdispersion) and the mechanism is explained. The robustness-to-baseline claim has been given supporting data in the form of a three-column table. Several moderate-priority items remain.

## Changes Evaluated

**Overdispersion → underdispersion (B1):** Correct and mechanistically well-justified. The binomial SD vs. observed SD comparison (1.80 vs. 1.2 for Georgia) is now explicitly stated in the text. The contiguity/population-balance explanation is the right mechanistic account.

**Baseline sensitivity data:** The revised Section 3.3 now provides the state-by-state baseline comparison table I requested. This makes the "within 5 percentage points" claim verifiable. Good.

**Rodden-effect framing:** The added paragraph is appropriate and correctly framed.

## Unresolved Statistical Issues

**Corridor fraction uncertainty (H2 from REVISION-PLAN): Not added.** The corridor fractions (42%, 61%, 28%, 55%) are still point estimates without confidence intervals. For $\ess \approx 1{,}000$, the 90% CI for $\hat\phi = 0.42$ is $0.42 \pm 0.026$. These intervals are narrow but should be reported as a matter of statistical completeness.

**Minnesota (H1 from REVISION-PLAN): Partially resolved.** The \est{} marker is present. However, the corridor fraction of 59% for Minnesota is not consistent with the derivation stated in the text. The binomial model with $p_{\rm geo} \approx 0.5$ for a competitive 8-seat state gives $P(3 \leq S_D \leq 5) \approx 0.71$ under Binomial(8, 0.5), not 0.59. If the model incorporates underdispersion, the paper should state what underdispersion factor is assumed and derive the 59% from it.

**$p_{\rm geo}$ derivation (H4 from REVISION-PLAN):** Partially addressed. The text now states "$p_{\rm geo}$ is estimated as the fraction of census tracts with Democratic plurality, weighted by population and adjusted for compactness constraints." This is acceptable as a qualitative description. The actual computation for Georgia (36%) should be verified against the stated derivation: fraction of population-weighted census tracts with Democratic plurality, with compactness adjustment, should give 36%.

## Recommendation

Accept with minor revisions. The key statistical fix (overdispersion direction) is made. Add corridor fraction confidence intervals and resolve the Minnesota 59% vs. binomial 71% discrepancy.
