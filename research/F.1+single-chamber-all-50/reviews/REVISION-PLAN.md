# REVISION PLAN — F.1: Algorithmic State House Redistricting: A 50-State Empirical Study
**Round**: R1 → R2 → R2 Complete
**Date**: 2026-05-05

## R1 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with revisions |
| Stephanopoulos | 3/4 | Accept with minor revisions |
| Liang | 2/4 | Major revision required |
| **Mean** | **2.8/4** | |

## R2 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with minor revisions |
| Stephanopoulos | 4/4 | Accept |
| Liang | 3/4 | Accept with minor revisions |
| **Mean** | **3.2/4** | |

## R2 Status: TARGET MET (3.2 ≥ 3.0)

## Critical Issues (Must Fix Before Resubmission)

### C1 — Wesberry standard misapplied to state legislative results (Stephanopoulos C1)
**Problem**: Paper characterises ±0.5% as satisfying "the Wesberry standard." Wesberry governs congressional districts; state legislative districts are governed by Reynolds v. Sims.
**Action**: Revise Section 5.2 to: "All 50 algorithmic maps achieve ±0.5%, well within the federal Wesberry standard for congressional districts and far within the Reynolds v. Sims standard (±5%) for state legislative plans." Emphasise the right standard applies.

### C2 — Sub-unit splitting procedure for NH undocumented (Liang C3, Karypis C1)
**Problem**: Section 5.3 mentions "population-weighted sub-unit assignment when a block group is the only unit available" but the mechanism is not documented. For k/n_bg = 0.57 (NH), this procedure is critical to the claimed ±0.48% balance result.
**Action**: Add methodology paragraph describing: (a) when sub-unit splitting is triggered (block group population > target remaining), (b) what data is used (block-level P.L. 94-171 data), (c) the specific allocation rule (proportional-to-block-population assignment of residual population). Confirm this is implemented in the Rust binary.

### C3 — 5-seed sensitivity analysis asserted but not shown (Liang C4)
**Problem**: "All [5-seed sensitivity checks] produce identical partisan outcomes and PP scores within ±0.005" but no table is provided.
**Action**: Add Table A.1 in appendix showing PP and partisan seat counts for seeds {42, 43, 44, 45, 46} for WA, TX, NH, and NE. If partisan data is not available for all four states, show only PP.

### C4 — Nebraska at k/n_bg = 0.040 classified as requiring block-group resolution (Duchin C3)
**Problem**: Nebraska (49 districts, 570 tracts, k/n_tract = 0.086) is classified as requiring block-group resolution. At block-group resolution (1,231 block groups), k/n_bg = 0.040 < 0.05 — within the adequate range. This creates a circularity: the rule classifies by k/n_tract but at block-group resolution the ratio is adequate.
**Action**: Add clarification: the rule is applied at tract resolution to determine whether to upgrade. Nebraska's k/n_tract = 0.086 > 0.05 triggers the upgrade. Once at block-group resolution, k/n_bg = 0.040 is adequate, confirming the upgrade was appropriate (and not over-upgraded to block level). This is correct behavior, not a circularity.

## Important Issues (Should Fix)

### I1 — Enacted map comparison lacks systematic description (Liang C5, Rodden C2)
**Problem**: 38 states have comparison data but the 12 missing are not identified and the source of enacted shapefiles is not specified.
**Action**: Add Table A.2 listing all 38 states with enacted map data and the source (Redistricting Data Hub, state GIS portal, etc.) with access date. Identify the 12 missing states and note whether they are known gerrymandering states.

### I2 — No partisan outcome analysis (Rodden C1)
**Problem**: The paper's political motivation centres on gerrymandering, but partisan outcomes are entirely excluded.
**Action**: Add a brief partisan analysis note: for the states where Kuriwaki precinct data is available (used in F.2), report the algorithmic map's estimated partisan seat share. Even a 5-state illustrative table would address this gap.

### I3 — METIS ufactor applicability at high k (Karypis C3)
**Problem**: ufactor=5 (0.5% per-part imbalance) may be infeasible at the METIS level for NH (T* = 3,342, block groups ~150 persons each). Balance is achieved via post-processing, but the relationship between ufactor and post-processing is not described.
**Action**: Add methodology note: "For high-k chambers where the METIS ufactor target is infeasible at the unit level (e.g., NH where single block groups may exceed 0.5% of T*), METIS is run with a relaxed ufactor and the ±0.5% constraint is achieved by the boundary-swap post-processing algorithm. METIS ufactor is set to [X] for these states."

### I4 — Compactness comparison confound (Duchin C4)
**Problem**: F.1 presents the PP advantage over congressional maps without noting that smaller districts mathematically produce higher PP (F.5 derivation).
**Action**: Add sentence in Section 4.1: "Note that the higher PP at state house scale relative to congressional scale is primarily a scale effect, not evidence of superior algorithmic performance at state house scale — see companion paper F.5 for a decomposition."

## Minor Issues (Can Fix in Proofing)

### M1 — t-test comparison of balance overstates legal significance (Stephanopoulos C2)
**Action**: Reframe: "The algorithmic maps achieve statistically significantly tighter population balance (t=3.8, p<0.001), though all 38 enacted maps fall within the constitutionally permissible ±5% Reynolds range."

### M2 — "Historically aggressive gerrymanders" needs court citations (Stephanopoulos C4)
**Action**: Add court citations for PA (LWV v. Commonwealth, 2018), WI (Gill v. Whitford, 2018), OH (Ohio Supreme Court decisions 2022), NC (Harper v. Hall, 2022).

### M3 — Vermont case study absent despite being hardest block-group case (Karypis implicit)
**Action**: Consider adding Vermont to case studies or at least noting: "Vermont (k=150, n_tracts=186, k/n=0.81) presents similar challenges to NH but at smaller scale; results are consistent with the NH pattern."

## Priorities for R2

1. Fix Wesberry misapplication (C1) — legal error
2. Document sub-unit splitting for NH (C2) — undocumented algorithm step
3. Add 5-seed sensitivity table (C3) — asserted result needs evidence
4. Clarify Nebraska threshold circularity (C4) — respond to Duchin concern
5. Add enacted map source table (I1)
6. METIS ufactor note (I3)
