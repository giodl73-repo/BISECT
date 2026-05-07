# Review: G.2 — Partisan Outcome Distributions
**Reviewer**: Moon Duchin (Ensemble methods, metric geometry of redistricting)
**Round**: 1
**Score**: 2/4

## Summary

G.2 is a companion to G.1 and shares several of G.1's problems while introducing new ones of its own. The proportionality corridor framework is the paper's genuine contribution, but the paper's central empirical claim — that AR is "within one seat of the ensemble median in five of six states" — depends entirely on the G.1 ensemble data, which has unresolved provenance and internal consistency problems. Until G.1 is corrected, the G.2 empirical claims cannot be evaluated.

## Critical Issues

**Issue 1: G.2's empirical results are downstream of G.1's unresolved data problems.**
The results table in Section 3 of G.2 is copied from G.1, including the NC 54th percentile and GA 38th percentile. If those numbers are wrong (as Duchin's G.1 review argues), G.2's conclusions are affected. G.2 should be revised only after G.1's data issues are resolved.

**Issue 2: The binomial model with overdispersion is stated backwards.**
The paper states the ensemble distribution is "approximately Binomial($k$, $p_{\rm geo}$) with overdispersion." Overdispersion means the variance is greater than the binomial variance $k p_{\rm geo}(1-p_{\rm geo})$. But the paper elsewhere uses $\sigma_{S_D} \approx 1.2$ for Georgia (G.1), while the binomial SD at $p_{\rm geo} \approx 0.36$, $k = 14$ gives $\sigma = 1.80$. The stated overdispersion is contradicted by the numbers: the actual ensemble variance appears to be LESS than the binomial prediction, not more. Ensemble redistricting distributions are often underdispersed relative to the binomial because the contiguity and population balance constraints narrow the feasible plan space. The paper has the direction of the overdispersion wrong.

**Issue 3: The "five of six states" claim is based on comparing AR to the ensemble median, but the correct comparison for evaluating partisan neutrality is to the proportional seat count, not the median.**
The paper distinguishes between these in Section 4 (the corridor analysis), but then the abstract, introduction, and conclusion all lead with the "near the 50th percentile" claim as the primary evidence of non-bias. This framing is misleading in sorted states, where the ensemble median is itself skewed relative to proportionality. In Georgia, the ensemble median of 6D represents geographic Republican advantage — matching the median does not demonstrate non-bias, it demonstrates that the algorithm replicates the geographic bias that the ensemble methodology is designed to measure.

A more honest framing: AR is not an algorithmic outlier (it falls near the geographic median), but this does not establish that the outcome is partisan-neutral by any fairness standard.

## Secondary Issues

**The corridor fractions (42% for NC, 61% for WI, 28% for GA, 55% for PA) are presented without sources.**
These are specific numerical claims about the fraction of ensemble plans falling within specified seat ranges. The source for each should be cited.

**Minnesota (Section 4.3) is presented without ensemble data.**
A 44th Democratic seat percentile and 59% corridor fraction are quoted for Minnesota with no ensemble source. If this is a new result, the methodology must be stated. If it is estimated, it must be marked \est.

**The delta_S_D calculation in G.3 (cross-referenced from G.2) contains an error.**
The correlation calculation $\Delta S_D \approx \rho \cdot (\sigma_{PP}/\sigma_{S_D}) \cdot (\text{percentile} - 0.50)$ is presented as if the last term is a Z-score, but $(0.68 - 0.50) = 0.18$ is a percentile difference, not a standard deviation difference. The correct formula would use $\Phi^{-1}(0.68) \approx 0.47$ as the standardized value. The calculation is dimensionally wrong as written.

**The "algorithm choice determines which tail you land in" section (Section 5) cites B.0 for the bakeoff but B.0 is not yet established in the G-series.** If B.0 is an unpublished companion paper, it should be clearly labeled as such.

## Recommendation

Major revision required pending G.1 corrections. Fix the overdispersion direction, provide sources for corridor fractions, and correct the framing to distinguish "near geographic median" from "partisan-neutral."
