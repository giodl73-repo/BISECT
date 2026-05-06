# REVISION PLAN — F.2: NestSection at Scale: Spine-Compatible Bicameral Redistricting for All 50 States
**Round**: R1 → R2
**Date**: 2026-05-05

## Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Revise and resubmit |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 2/4 | Revise and resubmit |
| Stephanopoulos | 3/4 | Accept with revisions |
| Liang | 3/4 | Accept with revisions |
| **Mean** | **2.8/4** | |

## Critical Issues (Must Fix Before Resubmission)

### C1 — Compatible state count inconsistency (Karypis C1, Duchin C1, Stephanopoulos C3)
**Problem**: Table 1 lists 9 states with gcd=1 (MO, OK, TX, HI, PA, CT, RI, ME, DE). 49 - 9 = 40 compatible states. But the abstract says "42," Section 3.2 says "7 incompatible," and another passage says "8 incompatible." These three counts (7, 8, 9 incompatible / 40, 41, 42 compatible) are mutually contradictory.
**Action**: Count Table 1 directly. By reviewer count: gcd=1 states are MO, OK, TX, HI, PA, CT, RI, ME, DE = 9 states. Compatible = 49 - 9 = 40. Update all references:
- Abstract: "42 states" → "40 states"
- Section 3.2: "42 compatible" and "7 incompatible" → "40 compatible" and "9 incompatible"
- Section 6.3: list of incompatible states is correct (9 names) but "7 incompatible states" introductory phrase → "9 incompatible states"
- F.0's Section 5.4 preview must also be updated to match.

### C2 — California nesting constitutional claim (Stephanopoulos C1)
**Problem**: Introduction states "California's constitution requires that 'each Senate district shall consist of two complete, contiguous Assembly districts.'" This is pre-Proposition 11 (2008) text.
**Action**: Replace with: "Under California's pre-2008 constitutional structure, senate districts were required to consist of two complete, contiguous assembly districts. The current Citizens Redistricting Commission (established by Proposition 11, 2008) draws house and senate maps independently." Retain as historical context but clarify current law.

### C3 — Non-integer H:S ratio within-spine handling not described (Duchin C4)
**Problem**: For states like Colorado (13:7, g=5, H/g=13, S/g=7), 13/7 is not an integer: senate districts cannot each contain the same number of house districts. NestSection's handling of non-integer within-spine ratios is not described.
**Action**: Add subsection 2.4: "Non-integer within-spine ratios. When H/g and S/g do not have an integer ratio (H/g)/(S/g), senate districts within each spine super-region contain either ⌊H/S⌋ or ⌈H/S⌉ house districts. For Colorado (13:7), 13/7 ≈ 1.86, so some senate districts within each spine super-region contain 1 house district and others contain 2. The assignment is determined by a secondary optimisation that minimises the edge cut of the senate-within-spine partition. All 42 compatible states succeed with this approach."

### C4 — Full 42-state verification table missing (Liang C3)
**Problem**: Table 2 shows only 7 states; the claim "all 42 compatible states succeed" is not verified by exhibition.
**Action**: Add Table A.1 (appendix) with full 42-state NestSection results: state, ratio, g, PP (house), PP (senate), max dev (H), max dev (S), house penalty, senate penalty. Mark states with small g (≤4) separately.

## Important Issues (Should Fix)

### I1 — Senate PP baseline missing (Karypis C3)
**Problem**: The paper reports senate PP < house PP by ~0.028 on average, but does not show independently drawn senate PP as a baseline. Without the independent baseline, the nesting penalty on PP cannot be separated from the district-size effect.
**Action**: Add column to Table A.1: "PP (senate, independent)" showing the PP of independently optimised senate maps for each state. The difference between this and "PP (senate, NestSection)" is the true nesting PP penalty.

### I2 — Constitutional barriers for incompatible states (Rodden C3, Stephanopoulos C4)
**Problem**: Paper suggests small adjustments (TX: S=31→30) without acknowledging constitutional barriers.
**Action**: Add footnote: "Chamber sizes are typically set by state constitution; changing them requires a constitutional amendment, which is a substantial political and procedural barrier. The suggestions here are illustrative of how compatibility could be achieved if chamber sizes were amended."

### I3 — 12 states without precinct data not identified (Liang C2, Rodden C2)
**Problem**: Partisan analysis covers 30 of 42 compatible states; the 12 excluded are not named.
**Action**: Add Table 4 footnote or appendix note identifying the 30 states with available precinct data and the 12 excluded.

### I4 — NestSection flag production status (Liang C4)
**Problem**: --chamber nest CLI flag may not be in production binary.
**Action**: Clarify: "The --chamber nest flag is implemented in [redist binary version X]. If using an earlier version, NestSection can be invoked by [alternative approach]." Or: "Results in this paper were generated using [describe method]; --chamber nest flag is planned for [future release]."

## Minor Issues (Can Fix in Proofing)

### M1 — Wisconsin House 43D→42D political context (Rodden C1)
**Action**: Add footnote: "Wisconsin's 43D/56R outcome reflects the documented urban-rural sorting of the Wisconsin electorate (Chen 2013). The 1-seat NestSection penalty (42D) does not significantly change the partisan composition of this minority-Democratic chamber."

### M2 — Senate penalty linear scaling regression (Duchin C2)
**Action**: Show the regression of senate penalty on H/S ratio explicitly: present the fitted line, R², and the estimated slope (+0.77%--0.80% per unit), instead of asserting from visual inspection.

### M3 — Iowa statutory vs. constitutional nesting (Stephanopoulos C2)
**Action**: Add parenthetical: "Iowa's nesting requirement is statutory (Iowa Code § 42.4) rather than constitutional, making amendment easier than for constitutional requirements."

## Priorities for R2

1. Fix compatible state count (C1) — fundamental numerical error, must propagate fix to F.0
2. Fix California nesting claim (C2) — legal error
3. Add non-integer ratio description (C3) — algorithm gap
4. Add full verification table (C4) — unsupported empirical claim
5. Add senate PP baseline (I1)
6. Address constitutional barrier note (I2)
