# REVISION PLAN — F.6: Voting Rights Act Compliance for State Legislative Redistricting: The 42% Threshold at Chamber Scale
**Round**: R1 → R2
**Date**: 2026-05-05

## Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 2/4 | Revise and resubmit |
| Stephanopoulos | 3/4 | Accept with minor revisions |
| Liang | 2/4 | Major revision required |
| **Mean** | **2.6/4** | |

## Critical Issues (Must Fix Before Resubmission)

### C1 — 4/5 vs. 5/5 state success rate inconsistency (Duchin C4, Liang C4)
**Problem**: Abstract and introduction state threshold "holds in 4 of 5 covered states." Table 1 shows all 5 states with "Threshold met: Yes" at state house scale and majority-minority districts for all 5 (AL: 27, GA: 55, LA: 33, MS: 46, SC: 28). The Table 1 note says "South Carolina fails at congressional scale but succeeds at state house scale." So at state house scale, all 5 covered states succeed.
**Action**: Determine the correct count. If the table is right (5/5 at state house scale, 4/5 at congressional scale), the text must be updated:
- Abstract: "...produces majority-minority districts in **all 5** covered states at state house scale..." (and note that 4/5 at congressional scale for comparison)
- Introduction: Same correction.
- The "4 of 5" claim is the key finding borrowed from congressional-scale analysis and misapplied to state house scale in the abstract.

### C2 — VRASection base unit: tracts vs. block groups (Karypis C1, Liang C2)
**Problem**: VRASection algorithm description (Section 2.1) uses "tracts" throughout. But covered states require block-group resolution by F.3's rule (all 5 covered states: AL 105/1290=0.081, GA 180/1969=0.091, LA 105/1148=0.091, MS 122/664=0.184, SC 124/803=0.154 — all > 0.05). VRASection must be operating on block groups. The paper uses "tracts" when it should say "block groups."
**Action**: Update Section 2.1 throughout: replace "tracts" with "base geographic units (census block groups at the resolution appropriate for each state)." Add note: "For all five covered states, the resolution rule (F.3) requires block-group resolution; VRASection operates on block groups as the base unit."

### C3 — Block-group adjacency data for covered states not confirmed available (Liang C1)
**Problem**: F.1's preprocessing built block-group adjacency for 12 states (NH, WY, VT, SD, ND, AK, MT, NE, WA, ID, ME, WV). None of the five covered Southern states (AL, GA, LA, MS, SC) are in this list. The F.6 analysis requires block-group adjacency data for all five.
**Action**: Confirm that block-group adjacency files have been generated for AL, GA, LA, MS, SC. Add to Section 2.3: "Block-group adjacency data for the five covered states was built using build_unit_adjacency.py for each state. Storage: approximately [X] MB per state. Command: `python scripts/data/geography/build_unit_adjacency.py --resolution block_group --year 2020 --states AL GA LA MS SC`."

### C4 — β₁/β₂ ratio when β₂ is not statistically significant (Duchin C2)
**Problem**: For Georgia (β₂ = 0.006, SE = 0.017, t = 0.35), the ratio β₁/β₂ = 23 is reported as a meaningful result. But β₂ is not significantly different from zero (t < 2), so the ratio is meaningless — dividing by a statistically zero denominator.
**Action**: Change Table 3 reporting: instead of β₁/β₂ ratio for states where β₂ is not significant, report "N/A (β₂ ≈ 0)" or ">> 1 (β₂ not significant)." Add footnote: "The β₁/β₂ ratio is informative only when β₂ is statistically significant. For AL, GA, MS, and SC, β₂ is not significantly different from zero; the correct interpretation is that partisan composition explains zero of the variation in boundary decisions, which is the most favorable possible result for the Callais disentanglement requirement."

## Important Issues (Should Fix)

### I1 — Gingles precondition 3 caveat in abstract (Karypis C2)
**Problem**: The abstract claims the threshold "holds at state legislative scale" without noting that precondition 3 (majority bloc voting) must be separately established through empirical evidence.
**Action**: Add sentence to abstract: "Satisfying the 42% threshold establishes the geographic compactness precondition (Gingles precondition 1). The political cohesion (precondition 2) and majority bloc voting (precondition 3) preconditions must be separately established through electoral data analysis, which is outside the scope of this paper."

### I2 — Mississippi resolution: k/n_tract = 0.184 > 0.05 (Karypis C3)
**Problem**: Mississippi (k=122, n_tracts=664, k/n=0.184) requires block-group resolution by F.3's rule. This is addressed in C2 above, but the specific Mississippi case should be highlighted because its high k/n ratio (0.184) makes it more sensitive to resolution choice than the other covered states.
**Action**: After C2 fix, add note in Section 2.2: "Mississippi has the highest k/n ratio among the covered states (0.184), making it the most sensitive to resolution choice. At block-group resolution, Mississippi has approximately 1,800 block groups, giving k/n_bg = 0.068 — just above the adequate-range threshold. Results for Mississippi should be interpreted with this in mind."

### I3 — Louisiana v. Callais connection (Karypis C4)
**Problem**: Louisiana is subject to the Callais litigation and the paper's Callais regression directly validates the methodology for that case.
**Action**: Add note in Section 4: "Louisiana is currently subject to the Section 2 litigation that developed the Callais disentanglement framework (Louisiana v. Callais, 2025). The Callais regression result for Louisiana (β₂ = -0.002, not significant) directly demonstrates that an algorithmic state house map for Louisiana would satisfy the Callais disentanglement requirement: boundary decisions in VRA-required majority-minority districts are explained entirely by minority population concentration (β₁ = 0.151), not by partisan composition."

### I4 — Seed sensitivity for VRASection (Duchin C3)
**Problem**: VRASection results may depend on the initial METIS partition, which is seed-dependent. Borderline cases (South Carolina) may not succeed in all seeds.
**Action**: Add Section 6.1: "Seed sensitivity of VRASection. We test 5 seeds (42--46) for the two most constrained states in the covered-state analysis (South Carolina and Alabama). [Report number of MM districts achieved at each seed.] Results are consistent across seeds for Alabama but [show result] for South Carolina, confirming [stability / identifying variability]."

### I5 — Coalition districts and crossover districts not addressed (Stephanopoulos C1)
**Problem**: Paper focuses on single-minority MM districts; coalition districts (multiple minority groups) and crossover districts (minority + some majority voters) are not addressed.
**Action**: Add subsection 3.3: "Coalition and crossover districts. In states with multiple minority groups (TX: Hispanic 39.3%, Black 11.8%; CA, NY, NM), VRASection's single-minority analysis may miss coalition district opportunities. The 42% threshold analysis in this paper applies only to single-minority majority-minority districts. Coalition district analysis requires additional methodology (combining minority populations) and is deferred to future work."

## Minor Issues (Can Fix in Proofing)

### M1 — Callais citation needs completion (Stephanopoulos C3)
**Problem**: "Louisiana v. Callais, --- U.S. --- (2025)" has no full citation.
**Action**: If Callais has been decided, insert full citation. If still pending at draft time, note: "Louisiana v. Callais, cert. granted [date]; decision pending."

### M2 — Wisconsin not in analysis (Rodden C2)
**Action**: Add footnote to Section 2.3: "Wisconsin is not included in the additional-state analysis because its precinct-level data from the 2020 election is not available in the Kuriwaki (2023) interpolated dataset at block-group resolution. Wisconsin has a substantial Black population in Milwaukee (26% Black in Milwaukee County); the state may face Section 2 obligations at state house scale that are not addressed here."

### M3 — Georgia minority group breakdown (Rodden C3)
**Action**: Add footnote to Georgia row: "The 55 majority-minority districts in Georgia include both majority-Black and majority-Hispanic districts; the majority-Black districts are concentrated in metro Atlanta and south Georgia, while majority-Hispanic districts are concentrated in the Gainesville and Dalton corridors. The analysis treats these as separate minority groups, each subject to independent Gingles analysis."

## Priorities for R2

1. Fix 4/5 vs. 5/5 success rate (C1) — core claim of the paper is stated incorrectly
2. Fix VRASection base unit terminology (C2) — algorithmic description error
3. Confirm block-group adjacency data for covered states (C3) — reproducibility requirement
4. Fix β₁/β₂ ratio when β₂ is not significant (C4) — statistical methodology error
5. Add Gingles precondition 3 caveat to abstract (I1)
6. Add seed sensitivity test (I4)
