# Review 1 — George Karypis
**Paper**: F.1: Algorithmic State House Redistricting — A 50-State Empirical Study
**Round**: R1
**Score**: 3/4

## Summary

F.1 presents the first systematic 50-state application of minimum-edge-cut recursive bisection to state house chambers. The paper covers all 50 lower chambers at the 2020 census, using adaptive resolution selection (block-group when k/n > 0.05, tract otherwise). Results include compactness, population balance, county splits, and runtime across all 50 states, with four case studies. This is a substantial empirical contribution with generally sound methodology.

## Strengths

The resolution assignment table (Table 2) is the most important technical element of the paper and it is well-executed. The 12 states requiring block-group resolution are correctly identified, and the k/n ratios are specific enough to be independently verifiable from 2020 TIGER/Line data. The distinction between the five states where block-group resolution is itself suboptimal (NH, WY, VT, SD, ND) and the remaining seven is appropriately noted in the limitations.

The case study of New Hampshire (Section 5.3) is algorithmically the most interesting: k/n_bg = 0.57 means that average districts contain 1.75 block groups — below the 20-unit threshold — yet the pipeline achieves ±0.48% balance via "population-weighted sub-unit assignment." The mechanism for this sub-unit assignment deserves more explanation. How does the pipeline handle the case where a block group is assigned to a district that already meets population balance but an adjacent block group must be split? Is this a lookup into block-level data, or an algebraic interpolation?

## Concerns

**C1 — Nebraska tract count discrepancy.** F.1's Table 2 lists Nebraska as having 570 census tracts with k/n = 0.086. F.0's Table 1 lists Nebraska as having "~570" tracts. These are consistent. However, the Nebraska case study (Section 5.4) states the block-group count is 1,231 with k/n_bg = 0.040. Independently: 49 districts / 1,231 block groups = 0.0398, which rounds to 0.040 — correct. However, for NH: 400 / 698 = 0.573, but F.1 Section 5.3 states k/n_bg = 0.57 and block groups = 698. The 2020 Census counts 698 block groups in NH — this is verifiable and appears correct.

**C2 — Texas tract count mismatch.** F.1's Table 2 lists Texas as having 5,265 census tracts. The F.1 abstract says 5,265 tracts for Texas. The case study (Section 5.2) says T* = 195,448 for 150 districts from a 2020 population of 29,317,186. The arithmetic gives 29,317,186 / 150 = 195,448 — correct. However, the Texas tract count from 2020 TIGER data is 5,265, consistent with the table. These figures appear internally consistent.

**C3 — METIS ufactor applicability at high k.** The paper states that METIS 5.x is used with ufactor=5 (0.5% per-part imbalance target) for all 50 states. For NH with k=400 districts and T* = 3,342 persons, a 0.5% imbalance allows ±17 persons per district. With block groups averaging 150--200 persons in NH, the ufactor=5 target is effectively impossible at the METIS level — the algorithm cannot achieve 0.5% balance using whole block groups. The boundary-swap post-processing presumably achieves the ±0.48% reported figure, but the relationship between the METIS ufactor and the post-processing balance result is not described. This is a methodological gap that could lead readers to misunderstand what METIS is actually solving for NH.

**C4 — Runtime claim needs clarification.** Table 4 (implicit, in Section 4.4) reports NH runtime as 420 seconds. F.0's Table 2 also reports 420 seconds for NH. These are consistent. However, both papers report this as "single seed" runtime. The 5-seed sensitivity analysis mentioned in F.1 Section 7 ("all produce identical partisan outcomes and PP scores within ±0.005") would therefore require approximately 5 × 420 = 35 minutes for NH alone. This should be acknowledged.

## Recommendation

Accept with minor revisions. The empirical contribution is substantial and the methodology is sound. C3 (ufactor applicability at high k) is the most important concern and should be addressed in the methodology section.
