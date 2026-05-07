# Review 3 — Moon Duchin
**Paper**: F.3: Resolution Selection for State Legislative Redistricting: When Census Tracts Are Too Coarse
**Round**: R1
**Score**: 3/4

## Summary

F.3 is the methods paper I most wanted to see from this research program. The resolution selection problem is real and important, and the paper makes a genuine contribution by providing a principled rule. My review focuses on whether the mathematical justification for the 0.05 threshold is rigorous, whether the MAUP analysis is correctly framed, and whether the rule has the right statistical properties.

## Strengths

The three-part motivation for the 20-units-per-district threshold (Section 3.1) is the paper's strongest contribution. The CLT argument is correctly invoked: with 20 random tract populations, the variance of the sum scales as σ^2/m (where σ is the standard deviation of tract populations), making the minimum achievable deviation proportional to 1/√m. For m=20, this is approximately 1/4.5 ≈ 22% of the per-tract standard deviation relative to the ideal district population — which is approximately 0.5% for typical tract and district populations. The argument is not fully formalised but the intuition is correct.

The empirical calibration approach (calibrate against C.1's congressional redistricting where tract = block-group results) is honest: the paper does not claim more theoretical derivation than it provides.

## Concerns

**C1 — The 0.05 threshold is not mathematically derived from the CLT argument.** Section 3.1 presents the CLT argument as motivation for the "20 units per district" heuristic, which then becomes k/n ≤ 1/20 = 0.05. But the CLT argument shows that the sum of 20 tract populations has small variance — it does not show that exactly 20 tracts is the minimum for ≤0.5% balance. The argument is: with m tracts per district, the expected boundary-swap adjustment needed for 0.5% balance scales as O(1/√m). For this to be achievable in practice (with the number of boundary tracts available), one needs m to be large enough. But the threshold m=20 is stated as empirical calibration, not derived from the CLT. The paper should more clearly separate the CLT motivation (which gives intuition) from the empirical calibration (which gives the specific threshold).

**C2 — Configuration count argument needs correction.** Section 3.1 states that "the number of distinct district configurations for a district containing m tracts grows as O(n^m/k^m) under a random graph model." This is not the correct formula. The number of valid configurations (contiguous subsets of the graph containing exactly m tracts, balanced to within 0.5% of the ideal population) is not well-approximated by n^m/k^m — this formula overcounts by ignoring contiguity and population constraints, which eliminate the vast majority of possible subsets. A more careful statement would reference the number of connected induced subgraphs of the adjacency graph with approximately m vertices, which scales differently. The conclusion (more configurations → richer optimisation) is qualitatively correct but the formula cited is wrong.

**C3 — Relationship between k/n and MAUP effect.** Section 5.4 classifies MAUP effects by k/n tier: low-k/n (k/n < 0.02): 0 seats; medium (0.02 < k/n < 0.05): ±1 seat; high (k/n > 0.05): ±2--4 seats. These are stated as results but the paper has only one high-k/n data point (WA). The low and medium tiers have two data points each (CA at 0.009, TX at 0.028, CA congressional at a lower ratio). The classification into three tiers from three data points is underspecified. A sensitivity analysis at intermediate k/n values (e.g., states with k/n = 0.03, 0.04, 0.06) would be needed to validate the tier boundaries.

**C4 — The --resolution auto flag.** Section 3.2 describes --resolution auto as automating the k/n > 0.05 rule. This is a useful feature. However, the paper should specify: what happens when k/n is very close to 0.05 (within, say, ±0.005)? The threshold is an empirical heuristic, and maps near the boundary (like WA at 0.067, only slightly above) may not benefit substantially from block-group resolution in all states. A "near-boundary" handling (e.g., run both and take the better result) would strengthen the auto-selection.

## Recommendation

Accept with minor revisions. C1 (CLT derivation clarity) and C2 (configuration count formula) are the most important mathematical concerns. The empirical contribution is solid.
