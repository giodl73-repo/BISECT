# REVISION PLAN — F.5: Compactness at State Scale: Algorithmic State Legislative Districts Outperform Congressional
**Round**: R1 → R2 → R2 Complete
**Date**: 2026-05-05

## R1 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with revisions |
| Stephanopoulos | 3/4 | Accept with minor revisions |
| Liang | 3/4 | Accept with revisions |
| **Mean** | **3.0/4** | |

## R2 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with minor revisions |
| Stephanopoulos | 3/4 | Accept with minor revisions |
| Liang | 3/4 | Accept with minor revisions |
| **Mean** | **3.0/4** | |

## R2 Status: TARGET MET (3.0 ≥ 3.0)

## Critical Issues (Must Fix Before Resubmission)

### C1 — Abstract vs. body resolution effect inconsistency (Duchin C1, Karypis C4, Liang C2)
**Problem**: Abstract says "the resolution effect itself contributes approximately 0.020 PP units." Section 4.3 says "+0.013 PP units." This inconsistency arises because the abstract uses the F.1 baseline (state house tract PP = 0.381, BG = 0.401, difference = 0.020) while the body uses the F.5 baseline (state house tract PP = 0.388, BG = 0.401, difference = 0.013). The two baselines differ because F.1 reports 2020-only PP while F.5 averages over 2000/2010/2020.
**Action**: Reconcile the two figures:
- Option A: Use consistent baseline throughout (F.5's three-year average), giving resolution effect = 0.013 in both abstract and body.
- Option B: Explicitly distinguish 2020-only (F.1 baseline, 0.020) from three-year average (F.5 baseline, 0.013) and state both.
- Recommended: Option B with clear labelling. Add note to abstract: "For 2020 census year only, the resolution effect is approximately 0.020 PP units; averaged across 2000/2010/2020, it is 0.013."
- Also cross-reference F.1's Table 1 to explain the different means.

### C2 — Proposition regularity conditions not formally specified (Duchin C2)
**Problem**: The Proposition requires "regularity conditions" described qualitatively as "excluding states with highly fractal boundaries." These conditions must be formally stated for a mathematical Proposition.
**Action**: Add Definition 1 before the Proposition: "A state geography G is regular if: (1) its boundary is a rectifiable Jordan curve with bounded curvature κ < κ_max; (2) the maximum contiguous coastal or internal lake boundary constitutes less than [threshold]% of total perimeter; and (3) no single geographic feature (lake, military reservation) occupies more than [threshold]% of state area." Alternatively, reframe the Proposition as a Conjecture with empirical support.

### C3 — Alaska, Wyoming, Montana exceptions violate Proposition (Karypis C1, C2)
**Problem**: Table 1 shows Alaska (congressional PP 0.521 > house PP 0.489), Wyoming (0.478 > 0.461), Montana (0.438 > 0.421) — state house PP lower than congressional PP. This violates the Proposition's prediction. The explanation (single congressional district = whole state, which has high PP) implies these states don't satisfy the Proposition's regularity conditions or the comparison is degenerate (k_congressional = 1 so the O(1/√k) scaling is comparing 1/√1 to 1/√k_house which should still show house advantage). The degenerate case needs explicit handling.
**Action**: Add subsection 2.5: "Degenerate cases: states with 1 congressional seat. For Alaska (k_cong=1), Wyoming (k_cong=1), and Montana (k_cong=2), the congressional 'district' is the entire state or half the state, which has relatively high PP because its shape is determined by state geography alone. The O(1/√k) scaling law predicts a decrease from k=1 to k≥20, not an increase. For these states, Equation (3) gives Δ PP ≈ c(1/√1 - 1/√k_house) = c(1 - 1/√k_house) > 0, predicting higher house PP than congressional PP. The empirical exceptions for AK, WY, MT therefore violate the theoretical prediction. We examine why in Appendix A: [analysis of state boundary geometry]."

### C4 — Enacted map comparison data sources not specified (Liang C3, Rodden C4)
**Problem**: Section 5 compares to "enacted maps in GIS format for 35 states" with no source, access date, or state list.
**Action**: Add Table A.1 listing the 35 states with enacted map data, the source (Redistricting Data Hub URL, state GIS portal, etc.), and access date. Identify the 15 missing states.

## Important Issues (Should Fix)

### I1 — c estimated from subsample, not full dataset (Karypis C3)
**Problem**: c ≈ 0.120 fitted from "states with approximately 8 congressional and 100 state house seats" — how many is this?
**Action**: Specify the subsample size. Alternatively, fit c from the full 50-state dataset using a regression of (PP_house - PP_cong) on (1/√k_house - 1/√k_cong) and report the slope as the c estimate. This is a more principled estimation.

### I2 — Gerrymandering classification not sourced (Rodden C1, Stephanopoulos C2)
**Problem**: "States where the enacted map is identified by courts or analysts as gerrymandered" is asserted without a list or citation.
**Action**: Add Table 5: "Gerrymandering classification of 35 comparison states." Columns: State, Source (court decision, Efficiency Gap score, DRA rating), Classification (gerrymandered/competitive/commission). This makes the correlation between compactness advantage and gerrymandering status testable.

### I3 — VRA mode disabled, but states with VRA requirements are in the sample (Stephanopoulos C4)
**Problem**: VRA mode is disabled to isolate compactness, but the 13 states in F.6 would have VRA mode enabled in a legally operative map. The compactness figures in F.5 are therefore upper bounds for VRA-covered states.
**Action**: Add footnote: "VRA mode is disabled in all maps in this paper to isolate the scale and resolution effects. For states with Section 2 requirements (see companion paper F.6), the operative algorithmic map would have VRA mode enabled, which reduces compactness by approximately [X]% in majority-minority districts and [Y]% overall."

### I4 — Reock score comparison for legal robustness (Stephanopoulos C1)
**Problem**: Courts use multiple compactness measures; paper uses only PP.
**Action**: Add brief Section 3.4: "Compactness by Reock score. To verify robustness, we also compute Reock scores for all 50 states. [Report mean Reock by resolution tier.] The Reock score shows a qualitatively similar advantage for state house over congressional maps ([direction and magnitude]), confirming that the compactness advantage is not an artifact of the PP metric."

## Minor Issues (Can Fix in Proofing)

### M1 — PP∞ limiting value not defined (Duchin C3)
**Action**: Add: "As k → ∞, the scaling law PP_k → PP_∞ where PP_∞ ≈ [value from empirical fit, approximately 0.45--0.50 for typical state geographies]. For practical redistricting purposes with k ≤ 400, the O(k^{-1}) correction term is small."

### M2 — Resolution as measurement artifact (Duchin C4)
**Action**: Add note to Section 4.1: "Part of the observed PP improvement at block-group resolution may reflect more precise boundary tracing of the same geographic boundaries (a measurement effect) rather than genuinely more compact district shapes. Disentangling these effects requires holding geographic boundaries fixed while varying the base resolution — which is not possible with the current pipeline. We treat the observed PP improvement as a combination of genuine geometric improvement and measurement precision improvement."

### M3 — Cross-census k fixation (Liang C4)
**Action**: Add note: "We hold k fixed at 2020 chamber sizes for all three census years, for comparability. In reality, some states' chamber sizes changed between 2000 and 2020; the fixed-k analysis may therefore slightly overstate or understate the advantage for some states in earlier census years."

## Priorities for R2

1. Fix abstract/body resolution effect inconsistency (C1) — internal error
2. Formalise Proposition regularity conditions (C2) — mathematical rigor
3. Address AK/WY/MT Proposition violations (C3) — exceptions to core result
4. Add enacted map source table (C4)
5. Fit c from full dataset (I1)
6. Source gerrymandering classification (I2)
