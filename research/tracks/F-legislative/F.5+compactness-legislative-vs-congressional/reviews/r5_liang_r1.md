# Review 5 — Christina Liang
**Paper**: F.5: Compactness at State Scale — Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1
**Score**: 3/4

## Summary

F.5 is a primarily theoretical and empirical paper. From a reproducibility perspective, the key questions are: (1) whether the 50-state Table 1 can be reproduced from the described pipeline; (2) whether the enacted map comparison in Section 5 is reproducible; and (3) whether the abstract's resolution effect figure (0.020) is consistent with the body (0.013).

## Strengths

The main Table 1 (50 states, three PP columns, house seats) is in principle reproducible: all algorithmic maps use the described pipeline (METIS recursive bisection, compactness=0.85, seed 42, VRA disabled), and the PP computation is standard (4πA/P^2 from TIGER shapefile geometries). The table is comprehensive and allows independent verification of any individual state's figures.

The empirical validation of the scaling law (c ≈ 0.120 fitted, 0.030 predicted vs. 0.027 observed) is a good example of how theoretical results should be validated: specific numerical prediction followed by comparison to actual data.

## Concerns

**C1 — 50-state tract PP mean inconsistency with F.1.** F.1 reports mean house PP at tract resolution as 0.381 (Table 1), while F.5 reports 0.388 (Table 1, mean row). These are the same 50-state calculation (50 states, tract resolution, 2020 census, redist pipeline) but give different means. This cannot be attributed to different census years since F.5 averages over 2000/2010/2020 while F.1 reports 2020 only — the different averaging periods could account for the discrepancy (0.381 for 2020 alone vs. 0.388 averaged over three years). But the paper should explicitly clarify this difference rather than presenting both figures without cross-reference.

**C2 — Abstract vs. body resolution effect discrepancy.** The abstract states "the resolution effect itself contributes approximately 0.020 PP units" while Section 4.3 reports +0.013 PP units as the resolution effect. The 0.020 figure appears to be 0.401 - 0.381 (F.1's BG minus F.1's tract) while 0.013 is 0.401 - 0.388 (F.5's BG minus F.5's tract). The papers use different baselines for "state house tract PP" and these baselines need to be reconciled before the resolution effect can be consistently reported.

**C3 — Enacted map comparison (Section 5) data requirements.** The comparison to "enacted maps in GIS format for 35 states" requires downloading or constructing 35 state house district shapefiles, computing PP for each district, and aggregating by state. The source of these shapefiles is not specified. The 35 states included are not listed. A replicator cannot reproduce Section 5 without knowing which states are in the sample and where the enacted shapefiles were obtained.

**C4 — Cross-census PP computation.** F.5 computes PP "for each district using the district's geographic boundary as represented in the TIGER shapefiles at the native resolution" for 2000, 2010, and 2020. But the 2000 and 2010 TIGER shapefiles have different tract definitions than 2020 — some tracts were split or merged across censuses. The algorithmic maps for 2000 and 2010 use the 2000 and 2010 tract definitions respectively. The paper notes "house seats from NCSL 2020 database" held constant across years — meaning the same k is used for all three years but the geographic units change. This is a valid design choice, but it means that the 2000 and 2010 maps use the same k as the 2020 maps, which may not reflect the actual chamber sizes at those census cycles (many states changed their chamber sizes between 2000 and 2020). The paper should specify whether k is held constant at its 2020 value for all three census years and whether this affects the cross-census stability conclusions.

## Recommendation

Accept with revisions. C1 and C2 (inconsistent PP figures across papers) must be resolved before the paper can be accepted. C3 (enacted map data sources) should be documented.
