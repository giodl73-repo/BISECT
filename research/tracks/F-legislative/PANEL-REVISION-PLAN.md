# Panel Revision Plan — Track F: State Legislative Redistricting

**Track**: F — State Legislative Redistricting
**Generated**: 2026-05-07
**Based on**: R1 and R2 panel review cycles (all 7 papers complete through R2)

---

## Overview

All 7 Track F papers met ≥3.0/4 mean at R2. No paper is currently submission-ready. This revision plan organizes remaining work into:

- **P0 (Must fix before R3)**: Errors, missing required content, internal inconsistencies
- **P1 (Should fix in R3)**: Important gaps that weaken the paper's contribution or reproducibility
- **Cross-paper (X)**: Items requiring coordinated action across multiple papers

Priority execution order: F.2 count audit → F.3 recommendation table → F.6 seed sensitivity → F.1 partisan analysis → F.4 legal updates → F.5 Proposition → F.0 propagation.

---

## F.0 — Open Items

### P0 — Must fix before R3

**P0-F0-1 — Compatible state count propagation** (dependency: F.2 count audit)
Wait for F.2 to resolve the 8-vs-9 / 40-vs-41 discrepancy. Once resolved, update F.0 Section 5.4 to carry the identical final figure. Do not submit F.0 before F.2's count is final.

**P0-F0-2 — VRA preview update (4/5 → 5/5)** (dependency: F.6 seed sensitivity)
Once F.6 confirms 5/5 seed stability for SC, update Section 5: "VRASection applied to state house chambers produces majority-minority districts in all 5 covered states (compared to 4 of 5 at congressional scale, where South Carolina fails due to geographic dispersion at the larger district size)."

**P0-F0-3 — NH caveat in Section 5.2**
Add: "Results for New Hampshire, Wyoming, Vermont, South Dakota, and North Dakota (k/n_bg > 0.20) should be interpreted with caution. Block-group resolution is itself suboptimal for these states; the 20-units-per-district heuristic is violated."

**P0-F0-4 — Vermont row in Table 1**
Add Vermont (VT | House | 150 | ~186 | 0.81).

### P1

**P1-F0-1 — O(n²·log n) degenerate-regime footnote**
Once F.3 adds the degenerate-regime derivation, update the footnote to cite F.3 Section [X].

**P1-F0-2 — TN parenthetical self-correction**
Remove the in-text parenthetical "(correction: TN 99:33 is 3:1)" — state directly.

---

## F.1 — Open Items

### P0 — Must fix before R3

**P0-F1-1 — Partisan outcome analysis**
At minimum: add cross-reference to F.2 partisan results for the gerrymandering-state overlap (WI, OH, PA, VA, WA). Preferred: add 10-state illustrative Table 5 for the documented gerrymandering states (PA, WI, OH, NC) and commission states (CA, CO, MI, AZ) using Kuriwaki precinct data already used in F.2. This does not require new computation.

**P0-F1-2 — 5-seed sensitivity Table A.1**
Add Table A.1 in appendix: seeds {42, 43, 44, 45, 46} × {WA, TX, NH, NE} showing PP score and partisan seat count. This is a required verification of an asserted result.

**P0-F1-3 — Enacted map source table**
Add Table A.2: all 38 states with enacted map data (source + access date); all 12 states without comparison data (identified by name).

**P0-F1-4 — Sub-unit splitting methodology documentation**
Add paragraph to Section 3 specifying: (a) trigger condition; (b) data source (block-level P.L. 94-171); (c) allocation rule (proportional-to-block-population).

### P1

**P1-F1-1 — METIS ufactor for high-k chambers**
Add note about NH where T* = 3,342 and block groups average ~150 persons.

**P1-F1-2 — F.1/F.5 PP baseline cross-reference**
Add note: "Mean PP of 0.381 is for 2020 census year only. Companion paper F.5 reports mean PP of 0.388 averaged across 2000/2010/2020."

---

## F.2 — Open Items

### P0 — Must fix before R3

**P0-F2-1 — Compatible state count audit (CRITICAL)**
Independently compute gcd(H,S) for all 9 listed incompatible states (MO, OK, TX, HI, PA, CT, RI, ME, DE) using 2020 NCSL chamber sizes. Verify whether one of the 9 actually has gcd > 1. Choose one of:
- If one should be reclassified as compatible: update list to 8 states; body text's "41 compatible" is correct
- If all 9 truly have gcd = 1: correct body text to "40 compatible / 9 incompatible"

Update F.0 Section 5.4 to propagate the final figure simultaneously.

**P0-F2-2 — Table caption vs. body agreement**
After P0-F2-1: ensure table caption, abstract, body, results section, and conclusion all state the identical incompatible state count.

**P0-F2-3 — NH within-spine sub-unit splitting**
Add to NH case study: "NestSection applies the sub-unit splitting procedure from F.1 recursively within each spine super-region. The 50-district partition within each super-region faces k/n_bg ≈ 0.57 — identical to the full-state NH problem."

### P1

**P1-F2-1 — Senate PP independent baseline**
Add "PP (senate, independent)" column to Table A.1. Run senate redistricting with and without the nesting constraint. The difference isolates the true nesting penalty from the district-size effect.

**P1-F2-2 — Identify 12 excluded partisan states**
List the 30 states included in partisan analysis and the 12 excluded (Kuriwaki precinct data not available).

---

## F.3 — Open Items

### P0 — Must fix before R3

**P0-F3-1 — Add 50-state recommendation table (CRITICAL)**
This is the paper's primary practical output and is entirely absent. Required table content:

| State | Chamber | k | n_tracts | k/n | Recommended resolution | Notes |
|-------|---------|---|----------|-----|----------------------|-------|
| Alabama | House | 105 | 1,290 | 0.081 | block_group | |
| ... | | | | | | |
| New Hampshire | House | 400 | ~320 | 1.250 | block (or BG with caveat) | k/n_bg = 0.57 |
| ... | | | | | | |

For states where k/n_bg > 0.20 (NH, WY, VT, SD, ND): note that block-group resolution is itself suboptimal.

**P0-F3-2 — Confirm configuration count formula removal**
Confirm O(n^m/k^m) formula has been removed and replaced with qualitative claim.

### P1

**P1-F3-1 — Add additional high-k/n MAUP data points**
The ±2–4 seat range for high-k/n states rests on one data point (WA). Add Maine (k/n ≈ 0.42) or Montana (k/n ≈ 0.36). If not added, explicitly label as "conjecture from one observation."

**P1-F3-2 — State law survey**
Identify whether any of the 12 block-group states have statutes specifying "census tract" resolution. Report as positive finding or null result.

---

## F.4 — Open Items

### P0 — Must fix before R3

**P0-F4-1 — COI weight direction verification**
Verify production algorithm's COI edge weight direction. Confirm δ_uv > 1 for within-COI edges appears consistently throughout Section 3.3. If original implementation used δ_uv < 1, the empirical results may need recomputation.

**P0-F4-2 — NC classification update**
Confirm NC entry accurately reflects post-Harper II (2023) situation: Type I (legislative control) as of Harper II. Recommended: "North Carolina | IV→I | ... | Harper I reversed March 2023; current regime is legislative control."

**P0-F4-3 — Ohio commission dysfunction note**
Confirm footnote present: "Ohio's Redistricting Commission was found by the Ohio Supreme Court to have violated Amendment 1's partisan fairness requirements multiple times during the 2020 cycle. Commission form does not guarantee substantive compliance."

### P1

**P1-F4-1 — Results-based partisan neutrality paragraph**
Florida's Amendment 6 prohibits maps that "result in" partisan advantage, not merely those drawn "with intent." The structural neutrality argument addresses only the intent prong. Add paragraph acknowledging this distinction.

---

## F.5 — Open Items

### P0 — Must fix before R3

**P0-F5-1 — Abstract vs. body resolution effect reconciliation**
Add temporal scope labels to both figures:
- Abstract: "0.020 PP units (2020 census year only)"
- Section 4.3: "0.013 PP units (three-year average across 2000/2010/2020)"

**P0-F5-2 — Proposition regularity conditions or Conjecture conversion**
Choose one:
- Option A: Add formal Definition 1 with bounded curvature condition, max coastal boundary fraction, max single-feature area
- Option B: Replace "Proposition" with "Conjecture" and add empirical support note

**P0-F5-3 — AK/WY/MT Appendix A verification**
Confirm degenerate-case analysis is present with explanation of why empirical exceptions occur despite the theoretical prediction (state house PP < congressional PP for k_cong = 1 or 2).

### P1

**P1-F5-1 — c parameter estimation from full dataset**
Confirm c ≈ 0.120 estimated from full 50-state regression, not a subsample. Report coefficient, SE, R².

**P1-F5-2 — Enacted map source table**
Table A.1: all 35 states, sources, access dates.

---

## F.6 — Open Items

### P0 — Must fix before R3

**P0-F6-1 — Seed sensitivity analysis for VRASection (CRITICAL)**
Run VRASection for seeds 42, 43, 44, 45, 46 for South Carolina (5/5 at state house scale — the paper's headline new finding) and Alabama (27 MM districts — largest count). Report for each seed: number of MM districts, mean PP of MM districts, population balance status.

If SC is seed-stable: "5 of 5 covered states confirmed stable across 5 seeds."
If SC is seed-unstable in some seeds: qualify the 5/5 claim accordingly and update F.0.

**P0-F6-2 — Block-group adjacency files confirmation**
Add to Section 2.3: document that block-group adjacency files for AL, GA, LA, MS, SC exist and are accessible, with file sizes and the `redist fetch` command used.

**P0-F6-3 — 5/5 count propagation to F.0**
After P0-F6-1 confirms result, update F.0 Section 5 VRA preview to match.

### P1

**P1-F6-1 — Scale-invariance empirical check**
Brief comparison of variance of Black population share across block groups vs. tracts for one covered state (Georgia recommended).

**P1-F6-2 — Callais citation finalization**
Determine whether Louisiana v. Callais has been decided as of submission date; insert full citation if available.

---

## Cross-Paper Coordination

### X1 — Compatible state count (F.0 and F.2) [P0]
F.2 must complete P0-F2-1 (gcd audit) first. After F.2's final figure is settled, run a text search across both papers for all count references and update simultaneously. The two papers cannot be submitted with different counts.

### X2 — VRA state-scale success rate (F.0 and F.6) [P0]
Sequencing: F.6 must complete P0-F6-1 (seed sensitivity for SC) before F.0 can finalize its VRA preview.

### X3 — PP baseline labelling (F.1 and F.5) [P1]
Both papers must be updated simultaneously:
- F.1: "Mean PP 0.381 (2020 census year only)"
- F.5: "Mean PP 0.388 (three-year average across 2000/2010/2020)"
Add explicit cross-reference in F.5.

---

## Revision Schedule

| Week | Action |
|------|--------|
| 1 | F.2: P0-F2-1 (gcd audit) → determine final compatible state count |
| 1 | F.6: P0-F6-1 (seed sensitivity for SC, AL) → determine 5/5 stability |
| 2 | F.3: P0-F3-1 (add recommendation table) + P0-F3-2 (verify formula removed) |
| 2 | F.4: P0-F4-1, P0-F4-2, P0-F4-3 (verify COI direction, NC, OH corrections) |
| 2 | F.0: P0-F0-1 + P0-F0-2 (propagate F.2 count and F.6 VRA rate) |
| 3 | F.1: P0-F1-1 (partisan analysis), P0-F1-2 (seed sensitivity table), P0-F1-3 (enacted map source) |
| 3 | F.5: P0-F5-1 (PP baseline labels), P0-F5-2 (Proposition formalization) |
| 4 | All papers: P1 items, cross-paper X3 coordination |
| 5 | R3 review cycle initiated (F.3 and F.4 first, as nearest to ready) |
| 6–7 | R3 reviews returned; revise and target submissions for F.3, F.4, F.0 |

## Submission Priority

Based on revision burden: F.3 and F.4 first (nearest to ready), then F.0 (after cascades from F.2 and F.6), then F.1 and F.5, then F.6, and F.2 last (after count audit resolves the fundamental count discrepancy).
