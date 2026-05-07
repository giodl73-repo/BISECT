---
reviewer: Moon Duchin
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.10 introduces county-sticky edge weighting as a soft-constraint approach to subdivision preservation in redistricting. The formulation is clean, the sweep design is systematic, and the results are well-presented. From a redistricting mathematics perspective, the paper engages correctly with the bicriterion optimisation framing (compactness vs. county preservation) and correctly identifies that hard constraints are infeasible for some states. The alpha_c = 3.0 Pareto elbow claim is the paper's central empirical finding, and I have specific concerns about whether the elbow is statistically robust or an artefact of the coarse grid of alpha_c values tested.

## Strengths

- **The infeasibility analysis for hard constraints is correct.** The paper correctly notes that in Iowa, hard constraints produce infeasible instances for 6 of 12 population-boundary counties. This is a direct motivation for the soft-constraint approach. The reference to Ricca et al. (2008) who found 15-20% infeasibility rates with hard constraints on their test instances is relevant and correctly cited.
- **The three-metric outcome set is complete.** Reporting county splits, multi-county districts, and perfectly-preserved counties provides a multi-dimensional view of county preservation quality. This is better than a single metric, and the results tell a consistent story across all three.
- **The compactness cost context is correct.** The paper correctly situates the 3.0% PP cost (0.361 to 0.350) relative to the 22% improvement the B.2 baseline achieves over enacted maps. This "still substantially more compact than current maps" argument is the right way to put the compactness cost in context.

## Weaknesses / P1 Items (Required Fixes)

- **The alpha_c = 3.0 elbow may be an artifact of the coarse grid.** The paper tests only 6 values of alpha_c: {1.0, 1.5, 2.0, 3.0, 5.0, 10.0}. The jump from 2.0 to 3.0 is a factor of 1.5, from 3.0 to 5.0 is 1.67, and from 5.0 to 10.0 is 2.0. The "elbow" at alpha_c = 3.0 could be partially driven by this non-uniform grid: the large jump from 2.0 to 3.0 covers more parameter space than the 1.5-to-2.0 range. A finer grid around the elbow (e.g., {2.5, 3.0, 3.5, 4.0}) would test whether the elbow is genuinely at 3.0 or somewhere in the 2.5-4.0 range. This matters because the DIA default is specified as alpha_c = 3.0, so the precision of the elbow location has direct policy implications.
- **The Texas case study does not verify the "unavoidable splits" count.** Section 4.5 states that Texas has 89 county splits at baseline, 15 unavoidable (counties exceeding one district target) and 74 avoidable. At alpha_c = 3.0, 28 of the 74 avoidable splits are eliminated, leaving 46 avoidable. But the paper does not show that the 15 "unavoidable" splits are actually unavoidable — i.e., that no population-balanced, contiguous map can avoid them. For a large county like Harris County (pop. 4.7M, ~6 districts needed), unavoidability is clear. But some of the 15 claimed unavoidable splits may be for counties near the threshold (pop. just above one district target) where unavoidability depends on adjacent population distribution. The paper should either (a) define "unavoidable" precisely (county population > W/k, where W is state population and k is district count) and verify this criterion against TIGER data, or (b) use a more cautious phrase like "counties that necessarily require splitting due to their population."
- **The abstract and conclusion disagree on the split reduction percentage.** The abstract states "county splits fall by 34% (487 → 323)" and the conclusion states "county splits fall by 34% (487 → 323)." But Table 1 shows the split reduction as "33.7%" (the exact figure in the table). The abstract uses "34%" while Table 2 and the discussion use "34%" and "33.7%." This is a minor inconsistency, but the abstract should use the precise figure (33.7%) to be consistent with Table 1. Similarly, the multi-county district reduction: the abstract states "37%" but the table shows (312-198)/312 = 36.5%. Fix both figures.

## P2 Items (Suggestions)

- **Add a finer alpha_c grid in the elbow region.** Test {2.5, 3.0, 3.5, 4.0} to establish whether the Pareto elbow is genuinely at 3.0 or in a range around 3.0. Report the refined elbow location with an uncertainty range.
- **Provide the two-level hierarchy (county + municipal) as a first implementation.** The discussion mentions alpha_m ≈ 1.5 for municipal boundaries. Even a 3-state implementation (Iowa, Texas, California) would make the extension concrete and position the paper as the foundation for municipal preservation work.

## Score: 3 — Minor Revision

The coarse-grid alpha_c concern (P1.1) is the most important issue — the DIA default is specified as exactly 3.0, so the robustness of that choice matters. The Texas unavoidable-splits verification (P1.2) and the percentage rounding inconsistency (P1.3) are smaller fixes. The paper's core contribution is sound and the approach is correctly motivated.
