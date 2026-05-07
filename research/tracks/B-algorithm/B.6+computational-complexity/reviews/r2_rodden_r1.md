---
reviewer: Jonathan Rodden
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.6 establishes the computational complexity of METIS recursive bisection for congressional redistricting. The paper makes two types of claims: theoretical (NP-hardness, approximation ratio, runtime bound) and empirical (O(n^1.07) scaling on 50-state data). From a political science perspective, the theoretical contributions are beyond my technical expertise to evaluate in detail, but the motivation, empirical analysis, and policy implications are within scope. The paper is well-framed for a legal and policy audience. My main concerns are about the empirical section's omissions and the policy implications of the approximation ratio claim.

## Strengths

- **The policy motivation is compelling and correct.** The paper correctly frames NP-hardness as a justification for heuristic redistricting — not a limitation but a theoretical necessity. This framing is directly useful for legislative and court contexts where the choice of a heuristic algorithm might otherwise appear arbitrary.
- **The cross-census scaling stability.** Table 3 shows that the scaling exponent b = 1.07 is consistent across 2000, 2010, and 2020 census data. This is the most reassuring empirical result in the paper: it suggests that the near-linear scaling will continue to hold for future census cycles, which is important for long-term DIA planning.
- **The space complexity result.** The demonstration that California (the largest state) requires under 3 MB at tract resolution is a concrete, memorable result that practitioners can use to argue that the algorithm is implementable on standard government hardware.

## Weaknesses / P1 Items (Required Fixes)

- **The approximation ratio is not calibrated against actual results.** Proposition 1 claims an O(sqrt(log n)) approximation ratio for METIS on planar graphs. For n = 74,000 (all U.S. tracts), the paper computes this as approximately 4.1. But the paper does not demonstrate that METIS actually achieves an approximation ratio of 4.1 or better on any specific instance. Without an "oracle" solution or bound on the true optimum for even one state, the approximation ratio claim is unverifiable. B.7 notes the DIA seed is within 2.9% of the best seed found among 10,000 — but 10,000 seeds is not a lower bound on the true optimum. The paper should either (a) frame the 2.9% gap as the empirical approximation ratio (with the caveat that the reference is the best of 10,000 seeds, not the true optimum), or (b) remove the numerical evaluation of the theoretical bound (4.1) as it overstates the precision of the guarantee.
- **The runtime table conflates states with different k.** Table 2 lists states from Vermont (k=1) to California (k=52). But the runtime for k=1 (trivial partition: no bisection needed) is not comparable to runtime for k=52 (6 bisection levels). Including Vermont and Wyoming with k=1 in the same OLS regression as California with k=52 will pull the exponent toward a larger value because runtime for k=1 is near zero regardless of n. The OLS fit should either (a) exclude k=1 states, or (b) include log(k) as a second predictor.
- **The paper does not discuss the role of population heterogeneity.** Census tracts vary substantially in population (from ~1,000 to ~100,000 in some states). Highly heterogeneous tract populations require more FM refinement to achieve population balance, which increases runtime. The paper does not test whether population heterogeneity (e.g., variance of tract populations within a state) is a predictor of runtime residuals in the OLS fit. This would be a straightforward robustness check.

## P2 Items (Suggestions)

- **Add a projection for 2030 Census.** The paper mentions that 2030 Census is expected to enumerate ~78,000–82,000 tracts. A runtime projection using the fitted O(n^1.07) model would be directly useful for DIA planning — e.g., "expected runtime for the 2030 Census is X ms for California."
- **Discuss the implications for block-level redistricting.** The paper notes that the O(n^1.07) fit may not generalise to block-level (n ≈ 8.1M nationally). A brief discussion of what would change at block level (memory requirements, whether near-linear scaling still holds) would help practitioners who might consider block-level redistricting in the future.

## Score: 3 — Minor Revision

The approximation ratio calibration (P1.1) and the OLS regression treatment of k=1 states (P1.2) are the main issues. These are fixable with minor reanalysis. The population heterogeneity robustness check (P1.3) is important but could be deferred to a revision note if the OLS fit residuals don't show a pattern.
