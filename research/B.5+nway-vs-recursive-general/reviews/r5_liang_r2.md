---
reviewer: Percy Liang
round: 2
score: 4
date: 2026-05-05
---

## Summary

Three of my four R1 P1 items are addressed in this revision. The missing Figure 1 is replaced by Table 2 (PP difference by k-bin), which is a reasonable substitute: the four bins provide enough resolution to establish the "broadly flat" claim without the original figure. The chamber count discrepancy is resolved (450 chamber-year combinations, correctly derived). The statistical clustering problem is addressed by restricting the primary paired t-test to the independent congressional sample. The confidence intervals issue — my R1 P1.2 — remains unaddressed.

## P1 Items: Response Assessment

**P1.1 (Missing Figure 1) — Addressed.** Table 2 provides the PP difference by k-bin with mean and range columns. This is sufficient to support the "broadly flat across k" claim. The range column shows variation within bins, which is more informative than the original figure description implied. I consider this item resolved, though a figure would still be preferable for conference presentation.

**P1.2 (Confidence intervals absent) — Not addressed.** Tables 1 and 3 still report means without confidence intervals. For the primary claim — that RB outperforms n-way by 0.003–0.004 PP — there is no 95% CI reported. For the runtime table (Table 3), means without standard deviations cannot be interpreted statistically. The methodology notes "10 independent runs" for runtime averaging, which would be sufficient to compute a confidence interval, but the interval is not given. This is a standard empirical reporting requirement and remains outstanding.

**P1.3 (Chamber count discrepancy) — Addressed.** The 450-chamber-year count is consistent across abstract, methods, and conclusion. The derivation (50 states × 3 chamber types × 3 years) is explicit.

## Reproducibility Assessment

The revision adds a specific FM post-processing description for the prime-k case (Section 5.2: "apply one additional FM inter-district sweep on boundaries adjacent to asymmetric splits"). This is progress on reproducibility. However, the lack of a replication command or script, noted in my P2, remains absent. This is a P2 gap.

The new partisan section (5.4) is reproducible in principle: it specifies 2020 presidential election data at tract level, majority vote share assignment, and the specific states (WI, NC). The |Δseats| = 0.2 aggregate is reproducible from the methodology.

## Empirical Methodology Notes

**Table 2's range column is valuable.** The range [+0.001, +0.007] for k ∈ [11, 30] is wider than I expected, suggesting that the mean ΔPP of +0.004 masks considerable state-level variation. For three of the four bins, the minimum is +0.001, implying that for some states and chamber configurations, RB provides essentially no compactness benefit. Identifying those cases would strengthen the paper's theoretical claim. This is a P2 observation for a future revision.

**The block-group PP values for case studies are still absent** (flagged by Duchin R1). This is a data-reporting gap that persists.

## Score: 4 — Accept with Minor Revisions

Three of four R1 P1 items are resolved. The confidence interval gap (P1.2) is the only remaining P1 issue, and it requires only computation from existing data (10-run runtime averages, 50-state PP differences). I recommend acceptance conditional on adding 95% CIs to Tables 1 and 3.
