# Review 5 — Christina Liang
**Paper**: F.6: Voting Rights Act Compliance for State Legislative Redistricting — The 42% Threshold at Chamber Scale
**Round**: R1
**Score**: 2/4

## Summary

F.6 is the paper with the most demanding data requirements in the F track. The VRASection analysis requires: block-group adjacency data for the 13 analysis states, precinct-level election returns interpolated to block groups (Kuriwaki 2023), and minority population data at block-group resolution. My review focuses on whether these data requirements are met and whether the results can be reproduced.

## Strengths

The Callais regression (Eq. 1) is specified completely enough to be reproduced: the unit of observation is an adjacent tract pair (i,j), the outcome is an indicator for whether the pair spans a district boundary, and the predictors are differences in minority population share and Democratic vote share plus geographic controls. The standard errors (HC3 robust) and significance thresholds are specified. A replicator who has the district assignment outputs and the precinct data can run this regression.

The block-group minority population data source (2020 P.L. 94-171 redistricting data file) is the correct source for racial population data at block-group resolution. This file is publicly available from the Census Bureau.

## Concerns

**C1 — Covered states require block-group resolution but adjacency data availability not confirmed.** By F.3's k/n > 0.05 rule: Alabama (105/1290 = 0.081 > 0.05), Louisiana (105/1148 = 0.091 > 0.05), Mississippi (122/664 = 0.184 > 0.05), South Carolina (124/803 = 0.154 > 0.05), Georgia (180/1969 = 0.091 > 0.05). All five covered states require block-group resolution. For VRASection to operate at block-group resolution, block-group adjacency data must exist for all five states. The paper does not confirm that this data has been generated or is available. The 12-state block-group requirement from F.1 (NH, WY, VT, SD, ND, AK, MT, NE, WA, ID, ME, WV) does not include any of the five covered states. Block-group adjacency data for AL, GA, LA, MS, SC must be generated separately from F.1's preprocessing.

**C2 — VRASection resolution specification.** The algorithm description (Section 2.1) consistently refers to "tracts" — "Phase 1: identifies the set of candidate tracts whose minority population share exceeds the configuration threshold" — but for block-group-resolution runs, the base units are block groups, not tracts. If VRASection is actually operating on block groups (as the k/n ratios require), the paper's algorithm description uses incorrect terminology throughout. Either the algorithm uses tracts (inconsistent with F.3's resolution rule) or block groups (inconsistent with the paper's terminology). This ambiguity makes the algorithm description unverifiable.

**C3 — Kuriwaki precinct data interpolation to block groups.** The partisan analysis uses "2020 presidential precinct returns interpolated to the relevant resolution level (Kuriwaki 2023)." For the Callais regression, precinct returns must be interpolated to block groups (not tracts). This requires either: (a) Kuriwaki's data natively provides block-group level allocation (unlikely — his main dataset is at census block level), or (b) the researchers performed a further aggregation from block level to block group. The interpolation methodology is not described. For states like Mississippi (664 tracts, ~1,800 block groups) where precinct boundaries may not align well with block-group boundaries, the interpolation assumptions could affect the Callais regression results.

**C4 — 4/5 vs. 5/5 state success rate.** The abstract and introduction state the threshold succeeds "in 4 of 5 covered states." Table 1 shows all five covered states with "Threshold met: Yes" at state house scale. This is an internal inconsistency that must be resolved. By my reading, all five covered states succeed at state house scale (South Carolina: 28 majority-minority districts, threshold met). If this is correct, the abstract claim of "4 of 5" is wrong and should be "5 of 5."

**C5 — 13-state analysis set: 8 additional states not confirmed analysed.** The methodology (Section 2.3) adds 8 additional states to the 5 covered states. Table 2 (mm_comparison) shows all 13 states with results. But the data requirements for all 13 states include: block-group adjacency data, minority population data, and precinct data. For large states (CA: 8,997 tracts × ~3 block groups/tract = ~27,000 block groups; TX: ~15,000 block groups), building adjacency files is computationally intensive. The paper does not confirm that all 13 states' block-group adjacency data has been generated.

## Recommendation

Major revision required. C1 (block-group adjacency data for covered states not confirmed available), C2 (resolution terminology inconsistency), and C4 (4/5 vs. 5/5 success rate inconsistency) are substantive errors. The reproducibility infrastructure for this paper is the least clearly documented in the F track.
