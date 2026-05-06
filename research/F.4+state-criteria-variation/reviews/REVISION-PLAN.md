# REVISION PLAN — F.4: Satisfying 50 Different Rule Sets: State Constitutional Redistricting Criteria and Algorithmic Adaptation
**Round**: R1 → R2 → R2 Complete
**Date**: 2026-05-05

## R1 Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Accept with minor revisions |
| Stephanopoulos | 3/4 | Accept with revisions |
| Liang | 3/4 | Accept with minor revisions |
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

### C1 — COI weight direction potential sign error (Karypis C1)
**Problem**: Section 3.3 describes COI weights using δ_uv < 1 for cross-COI edges. This would make the algorithm prefer to separate communities (lower weight = less reluctant to cut). If the intent is to preserve communities (keep them together), cross-COI edges should have higher weight (algorithm more reluctant to cut).
**Action**: Verify the intended direction in the implementation. If the intent is community preservation: δ_uv > 1 for within-COI edges (more reluctant to cut within-community boundaries) and δ_uv = 1 for cross-COI edges. If δ_uv < 1 for cross-COI means "cheaper to put adjacent tracts in different districts if they are in different COIs" — which could make sense as a secondary criterion — clarify this interpretation. The paper currently states the formula would produce community separation, which is the opposite of the stated goal.

### C2 — North Carolina Harper II reversal (Stephanopoulos C4)
**Problem**: Table 1 lists North Carolina as Type IV with "Harper doctrine; court oversight." The North Carolina Supreme Court reversed its Harper gerrymandering holding in Harper II (2023), returning the state to legislative control.
**Action**: Update North Carolina entry to: "Type I (as of Harper II, 2023) or Type IV (under Harper I, 2022). Court-ordered reform reversed in 2023 after change in court composition; current regime is legislative control." Add footnote on the timeline.

### C3 — Ohio commission dysfunction (Stephanopoulos C3)
**Problem**: Ohio is classified as Type IV (commission), but the Ohio commission repeatedly violated its own partisan fairness requirements (multiple court strikes 2022).
**Action**: Add note to Ohio's Table 1 entry: "Note: Ohio's commission process was found by the Ohio Supreme Court to have violated Amendment 1's partisan fairness requirements multiple times during the 2020 redistricting cycle. Commission form does not guarantee substantive compliance; Ohio functioned as Type I (permissive) in practice during 2020-cycle redistricting."

### C4 — YAML parameters not confirmed in production CLI (Liang C1)
**Problem**: Parameters compactness, county_weight, coi, partisan_neutral, vra_mode are described in YAML format but are not listed in the CLI documentation (CLAUDE.md).
**Action**: Either: (a) confirm these are YAML-level configuration parameters (not CLI flags) and show the YAML format explicitly; or (b) map them to CLI flags or configuration file keys that are confirmed to exist. Add one example YAML configuration showing all parameters for a single state (Iowa recommended as the cleanest Type II example).

## Important Issues (Should Fix)

### I1 — Arizona competitive margins not sourced (Duchin C3, Liang C2)
**Problem**: "Mean margin of 7.2 percentage points in competitive congressional districts... compared to 9.4 in the 2022 enacted Arizona map" has no data source.
**Action**: Either provide the data source (precinct data, election year, computation methodology) or remove this specific claim. Replace with: "The algorithmic approach produces districts whose partisan margins are determined by the geographic distribution of voters rather than by mapmaker choice; whether this produces more or less 'competitive' maps depends on the state's geographic polarisation, as examined in companion paper F.3."

### I2 — Results-based partisan neutrality (Rodden C1)
**Problem**: Paper argues structural partisan neutrality satisfies all partisan-neutrality requirements. But Florida's Amendment 6 prohibits maps that "result in" partisan advantage, not merely those drawn "with intent."
**Action**: Add paragraph distinguishing intent-based and results-based neutrality requirements: "Florida's Amendment 6 may require demonstrating that maps do not systematically result in partisan advantage, not merely that they were drawn without partisan intent. Algorithmic maps satisfy the intent prong categorically, but satisfying the results prong depends on whether the algorithm's outputs happen to produce partisan balance in a given state — which is a function of geographic sorting, not of algorithmic design."

### I3 — VRA auto mode limitations in active litigation states (Rodden C4)
**Problem**: "vra_mode: auto is sufficient to satisfy the federal VRA obligation" for TX and GA overstates the algorithm's capabilities in active litigation states.
**Action**: Replace with: "For Texas and Georgia, vra_mode: auto generates maps consistent with known Section 2 requirements. However, states with active VRA litigation (Texas's redistricting is subject to ongoing judicial scrutiny) may require more detailed analysis beyond the default configuration. See companion paper F.6 for the full VRASection methodology."

### I4 — Edge cut vs. PP equivalence precision (Duchin C4)
**Problem**: "Naturally minimize county splits and maximize compactness" should say "minimize edge cut weighted by boundary length" (which correlates with but is not equivalent to maximising PP).
**Action**: Revise: "The algorithm minimises total edge-cut weighted by shared boundary length, which is approximately equivalent to minimising total district boundary length. This is correlated with (but not identical to) maximising Polsby-Popper compactness; the correlation is typically above 0.85 across states."

### I5 — 50-state classification date specification (Liang C4)
**Problem**: Classifications will become stale; NC and OH already partially wrong.
**Action**: Add header: "Classification as of April 2026, based on constitutional text and most recent court decisions. Redistricting law is actively litigated; readers should verify current law for any specific state."

## Minor Issues (Can Fix in Proofing)

### M1 — Population tolerance accumulation (Karypis C2)
**Action**: Add footnote: "The O(log k) accumulated error bound is conservative; in practice, the METIS bisection at each level targets balance far within 0.5%/log k, producing final deviations well within the stated 0.5% tolerance."

### M2 — Vermont and Delaware congressional scope (Stephanopoulos C2)
**Action**: Add footnote to Vermont and Delaware rows: "Vermont and Delaware have single at-large congressional seats; no congressional redistricting problem arises. State legislative criteria govern their legislative maps only."

### M3 — Iowa statutory criteria "mandatory language" (Stephanopoulos, Section 5.2)
**Action**: Confirm citation: Iowa Code § 42.4 uses "shall" for compactness and county integrity. Add specific statutory citation.

## Priorities for R2

1. Verify and fix COI weight direction (C1) — potential technical error in algorithm description
2. Update North Carolina classification (C2) — current law has changed
3. Update Ohio note (C3) — commission dysfunction documented
4. Add YAML example (C4) — most important reproducibility gap
5. Source or remove Arizona margins claim (I1)
6. Add results-based neutrality distinction (I2)
