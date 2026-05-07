# Panel Review — Track F: State Legislative Redistricting

**Track**: F — State Legislative Redistricting
**Review date**: 2026-05-07
**Papers reviewed**: 7 (F.0 through F.6)
**Panel composition**: Karypis (computational), Rodden (political science), Duchin (GIS/math), Stephanopoulos (election law), Liang (reproducibility/systems)

**_panel.yaml status**: Absent in all 7 paper directories — no panel metadata files exist for Track F.

---

## Track Maturity Assessment

Track F is the least mature track in the portfolio at time of review. All 7 papers entered panel review simultaneously with no prior external review history. Track F completed two full panel rounds for all 7 papers and all reached the ≥3.0/4 mean score threshold at R2. However, 3.0 at R2 is the floor, not the ceiling. Every paper carries open revision items.

Track R1 mean: 2.86/4 | Track R2 mean: 3.11/4

---

## Panel Score Summary

| Paper | R1 Mean | R2 Mean | Status |
|-------|---------|---------|--------|
| F.0 State legislative overview | 2.8/4 | 3.2/4 | Target met; 4 open items |
| F.1 50-state empirical | 2.8/4 | 3.2/4 | Target met; partisan gap open |
| F.2 Bicameral nesting | 2.8/4 | 3.2/4 | Target met; CRITICAL count error persists |
| F.3 Resolution selection | 3.0/4 | 3.0/4 | Target met; CRITICAL table missing |
| F.4 Criteria variation | 3.0/4 | 3.0/4 | Target met; legal accuracy items to verify |
| F.5 Compactness comparison | 3.0/4 | 3.0/4 | Target met; Proposition needs formalization |
| F.6 VRA state legislative | 2.6/4 | 3.0/4 | Target met; CRITICAL seed sensitivity absent |

---

## Paper-by-Paper Assessments

### F.0 — Algorithmic State Legislative Redistricting: A Research Program
**Target venue**: Internal (track overview) | **R2 mean**: 3.2/4

Good framing of the k/n > 0.05 resolution selection rule and the track's scope. Key strengths: resolution rule motivation (three independent justifications), runtime table (NH 420s, WA 180s, TX 380s), track structure clarity.

**Open items at post-R2**:
1. NH caveat in Section 5.2 for k/n_bg = 0.57 (block-group results interpreted with caution): verify added
2. Vermont row in Table 1 (k/n = 0.81, highest valid case after NH): verify added
3. VRA preview update: must change from "4 of 5" to "5 of 5" at state house scale once F.6 finalizes its seed sensitivity result
4. Compatible state count in Section 5.4: must propagate F.2's final reconciled figure (currently 40 vs. 41 ambiguous)

Reviewer-confirmed: Karcher citation corrected to Mahan v. Howell and Brown v. Thomson; California constitutional claim corrected (post-Proposition 11); compatible state count largely corrected in body text.

---

### F.1 — Algorithmic State House Redistricting: A 50-State Empirical Study
**Target venue**: State Politics and Policy Quarterly | **R2 mean**: 3.2/4

Core empirical paper. Results: mean PP 0.381 at tract resolution, 0.401 at block-group; all 50 states within ±0.5% population balance; county splits reduced in 30/38 states (mean −18%); largest reductions in PA (−41%), WI (−36%), OH (−37%).

**Significant open gap (PP1)**: No partisan outcome analysis despite gerrymandering motivation. F.2 provides partisan seat-share analysis for 30 states using Kuriwaki precinct data. F.1, the primary empirical paper, provides none. At minimum, cross-reference to F.2 for overlap states (WI, OH, PA, VA, WA).

**Other open items at post-R2**:
1. 5-seed sensitivity Table A.1 for WA, TX, NH, NE: asserted result must be shown
2. Enacted map source table: list all 38 states, sources, access dates; identify 12 missing states
3. Sub-unit splitting methodology: confirm block-level P.L. 94-171 data usage and proportional allocation rule documented

Reviewer-confirmed: Wesberry → Reynolds citation corrected; court citations added for PA, WI, OH, NC; Wisconsin F.2 cross-reference added.

---

### F.2 — NestSection at Scale: Spine-Compatible Bicameral Redistricting
**Target venue**: State Politics and Policy Quarterly | **R2 mean**: 3.2/4

**CRITICAL UNRESOLVED ERROR (PP1)**: Table caption lists 9 gcd=1 incompatible states by name (MO, OK, TX, HI, PA, CT, RI, ME, DE). Body text (post-R2) states "41 compatible / 8 incompatible." Arithmetic: 49 − 9 = 40 ≠ 41. The three numbers (9 listed, 8 in body, 40 by arithmetic) cannot all be correct. A direct gcd audit of each listed state is required.

**Required action**: Independently compute gcd(H,S) for each of the 9 listed states. If one state actually has gcd > 1, reclassify it; final count becomes 41/8. If all 9 truly have gcd = 1, correct body text to 40/9. Propagate the final figure to F.0 Section 5.4 simultaneously.

**Other open items at post-R2**:
1. Senate PP independent baseline: add "PP (senate, independent)" column to Table A.1 to isolate nesting penalty from district-size effect
2. NH within-spine sub-unit splitting acknowledgment: the 50-district partition within each spine super-region (~87 block groups per region, k/n_bg ≈ 0.57) requires the sub-unit splitting procedure from F.1
3. Identify all 12 states excluded from partisan analysis

Reviewer-confirmed: California constitutional claim corrected (post-Proposition 11); Wisconsin and constitutional barrier footnotes added.

---

### F.3 — Resolution Selection: When Census Tracts Are Too Coarse
**Target venue**: State Politics and Policy Quarterly | **R2 mean**: 3.0/4

**CRITICAL MISSING DELIVERABLE (PP1)**: The 50-state resolution recommendation table is the paper's primary practical output and is entirely absent despite being promised in the abstract ("We supply a resolution recommendation table for all 50 states") and referenced in Sections 4 and 7. Required content: all 50 states, lower chamber, k, n_tracts, k/n, recommended resolution, notes.

**Other open items at post-R2**:
1. Configuration count formula: confirm O(n^m/k^m) formula removed and replaced with qualitative claim
2. MAUP high-k/n tier: only one data point (WA, +2 seats). Add at least one additional high-k/n state (Maine k/n ≈ 0.42 or Montana k/n ≈ 0.36) or explicitly label as conjecture
3. State law survey: identify whether any of the 12 block-group states have statutes specifying "census tract" resolution (potential legal conflict)

Reviewer-confirmed: Threshold derivation vs. calibration restructured; near-threshold handling note added; directional MAUP paragraph added (both non-zero MAUP effects favour Democrats); state law novelty acknowledgment added.

---

### F.4 — Satisfying 50 Different Rule Sets: State Criteria and Algorithmic Adaptation
**Target venue**: State Politics and Policy Quarterly | **R2 mean**: 3.0/4

**Items to verify in R3**:
1. COI weight direction: confirm δ_uv > 1 for within-COI edges (preservation) throughout Section 3.3. The original R1 error (δ_uv < 1 causing community separation) was corrected in R2 — verify the correction is complete and consistent
2. NC classification accuracy: Harper II (2023) reversed Harper I and returned legislative control. Confirm the NC entry accurately reflects the post-Harper II regime (Type I or transitional Type IV with note)
3. Ohio commission dysfunction note: confirm footnote present acknowledging Ohio functioned as Type I in practice during the 2020 redistricting cycle
4. Wisconsin classification note: confirm Type II entry includes footnote about 2022 court-ordered maps
5. YAML parameter confirmation: all keys in the worked Iowa example must correspond to documented production configuration file entries

---

### F.5 — Compactness at State Scale
**Target venue**: Political Analysis | **R2 mean**: 3.0/4

**PP1 — Abstract/body resolution effect reconciliation**:
Abstract states "approximately 0.020 PP units" for resolution effect; Section 4.3 states "+0.013 PP units." Both are correct (different temporal baselines) but must be clearly labeled. Option B (both figures labeled with temporal scope) must be confirmed present: "0.020 (2020 census year only)" and "0.013 (three-year average across 2000/2010/2020)."

**PP1 — Proposition regularity conditions**:
Proposition 1 (PP scales as O(1/√k)) states regularity conditions qualitatively. Must either: (a) add formal Definition 1 with bounded curvature condition, maximum coastal boundary fraction, and maximum single-feature area; or (b) relabel as "Conjecture 1" with empirical support note. Option B is adequate for Political Analysis.

**Other open items at post-R2**:
1. AK/WY/MT Appendix A: verify degenerate-case analysis present and complete (why state house PP < congressional PP for k_cong = 1 or 2)
2. c parameter estimation: confirm fit from full 50-state dataset (not just subsample of "states with ~8 congressional and ~100 state house seats")
3. Enacted map source table: confirm Table A.1 lists all 35 states with sources and access dates
4. F.1/F.5 PP cross-reference: add explicit note that F.1's 0.381 (2020 only) and F.5's 0.388 (3-year average) represent the same methodology applied to different temporal scopes

---

### F.6 — VRA Compliance for State Legislative Redistricting
**Target venue**: Yale Law Journal | **R2 mean**: 3.0/4

**CRITICAL MISSING ANALYSIS (PP1)**: Seed sensitivity for VRASection still absent despite being I4 in the revision plan. South Carolina's 5/5 state house scale result (the paper's headline finding vs. 4/5 at congressional scale) rests on seed 42 alone. Before any submission, a 5-seed test (seeds 42–46) for South Carolina and Alabama is required. If SC is seed-stable across all 5 seeds, the 5/5 claim is supported. If any seed fails, the claim must be qualified.

**Other open items at post-R2**:
1. Scale-invariance empirical check: a brief comparison of minority population variance at block-group vs. tract level for one covered state (Georgia recommended) would strengthen the scale-invariance claim
2. Callais citation: determine current status of Louisiana v. Callais as of submission date; insert full citation or "decision pending" note
3. β₁/β₂ ratio correction: confirm Table 3 reports "N/A (β₂ ≈ 0)" for states where β₂ is not statistically significant (Georgia: t = 0.35), rather than a finite ratio
4. Block-group adjacency files for AL, GA, LA, MS, SC: confirm files exist and document file sizes

---

## Cross-Paper Issues

### Issue X1 — Compatible state count (F.0 and F.2)
F.2's Table 1 lists 9 gcd=1 states but the body says 8/41. F.0's Section 5.4 must propagate whatever final figure F.2 settles on after the gcd audit. Both papers must carry the identical count before either is submitted.

### Issue X2 — VRA state-scale success rate (F.0 and F.6)
F.0's Section 5 preview must match F.6's final seed-sensitivity result. F.6 must finalize seed sensitivity for South Carolina (P0-F6-1) before F.0 can safely update its VRA preview.

**Dependency**: F.6 seed sensitivity → F.0 VRA count update.

### Issue X3 — PP baseline inconsistency (F.1 and F.5)
F.1 reports mean house PP = 0.381 (2020 only); F.5 reports 0.388 (3-year average). Both papers must explicitly label their temporal scope. Add cross-reference in F.5 confirming consistency with F.1's 2020-only figure.

---

## Track F vs. Other Tracks — Maturity Comparison

Track F is the only track with no papers at 3.5+ mean and no papers currently submission-ready. The primary barriers are: political outcome analysis (F.1), algorithmic description accuracy (F.2 count, F.4 COI direction), missing deliverables (F.3 table), statistical methodology (F.6 seed sensitivity), and mathematical rigor (F.5 Proposition).

---

*Panel convened 2026-05-07. Track F — seven papers, single sub-track: state-legislative.*
