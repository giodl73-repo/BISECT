BOARD REVIEW — 2026-05-08
Portfolio: historical 12-track snapshot, A-L, 120+ papers

Reviewer: Board-level synthesis review (Claude Code)
Scope: Portfolio coherence, coverage gaps, cross-track consistency, narrative arc, implementation alignment, open P1 blockers

---

## 1. COMPLETENESS: 7.5/10

**What the portfolio covers well:**

The twelve tracks form a broadly complete research program. Core algorithmic work (B), validation (C), VRA/legal compliance (D), experimental extensions (E), state legislative redistricting (F), and ensemble methods (G-H) cover the standard redistricting-science research agenda. The newer tracks add important depth: K-compactness provides dedicated metric-by-metric coverage that was previously scattered across B and C, L-partisan-fairness concentrates the post-Rucho legal argument, J-apportionment closes the foundational logical prerequisite, and I-incumbency addresses a prominent political objection to the technology.

**Major gaps — whole missing areas (not individual papers):**

**Gap 1 — Racial bloc voting and Gingles prong 2+3.** D.5 introduces the Gingles methodology for bloc voting, but no track systematically addresses the polarized voting analysis that constitutes Gingles prongs 2 and 3 (political cohesion, white bloc defeat). D.0 acknowledges these as "additional showings" not covered by the algorithmic maps, and A.0's findings section correctly labels algorithmic districts as "opportunity districts" rather than "performing districts." However, the portfolio makes no attempt to integrate precinct-level racially polarized voting analysis into the pipeline. For a portfolio aimed at redistricting litigation support, this is a meaningful hole: expert witnesses using these papers will face cross-examination on whether the 69-district surplus translates into actual electoral opportunity. No track currently bridges that gap.

**Gap 2 — Municipal and county-level redistricting.** The portfolio treats congressional (B-D) and state legislative (F) redistricting, but local redistricting (city councils, school boards, county commissions) is entirely absent. This is a legitimate research scope decision, but it should be stated explicitly in A.0 and A.1 as a boundary condition. Currently A.0's abstract implies the technique generalizes without qualification.

**Gap 3 — Statistical significance framework.** Multiple B, C, I, and K papers use single-seed runs (marked with dagger notation) and present results as "preliminary." No track synthesizes an ensemble-level significance testing methodology. The I-track and K-track post-write checks repeatedly flag this as a P2 concern. Given that the entire portfolio rests on empirical claims about NC/WI/TX performance, a dedicated paper on ensemble-level inference (even 50-seed validation for key claims) would significantly strengthen the public-release standing of every downstream paper that cites those states.

**Gap 4 — Competitive states vs. safe states.** Almost all empirical work concentrates on NC, WI, TX, and FL. These are high-profile gerrymandering states that demonstrate large bisect-vs-enacted gaps. The portfolio would be more complete with a paper on low-partisan-salience states (e.g., VT, WY, ND single-at-large) and on states where bisect offers modest improvement.

**Covered adequately, not exhaustively:** multi-member districts (E.1), international applications (E.6 — still major revision), parallel computing performance (U.10), uncertainty quantification (C.7), adoption pathways (D.4, B.02).

---

## 2. NARRATIVE ARC: PARTIAL PASS

**Overall arc A→L assessed:**

The logical sequence is broadly coherent: A (synthesis) → B (algorithm) → C (validation) → D (VRA) → E (experimental) → F (legislative) → G (ensembles) → H (search) → I (incumbency) → J (apportionment) → K (compactness) → L (partisan fairness).

**Critical ordering issue — J before B:**

J (apportionment) documents the `bisect-apportion` crate and the mathematical prerequisite that determines *how many* seats each state gets before any district boundary is drawn. A.0's introduction (Section 1) explicitly invokes the Huntington-Hill precedent as the motivating analogy: "if mathematical objectivity resolved apportionment, why not redistricting?" This analogy is the paper's founding rhetorical move. Yet the J-track papers — which formally document that the `bisect-apportion` implementation reproduces the 2020 Census Bureau Huntington-Hill result exactly — appear *after* B.1 (the core redistricting paper) and are described in A.2 only as an afterthought.

For a reader of A.0 who wants to verify the Huntington-Hill precedent claim, J.1 and J.6 are the papers they need — but A.0 does not cite them at all. A.0's bibliography (checked: `references.bib`) references `bisect2024` generically but does not include any `deluca2026apportion`-style citation. **The J-track is an unlinked prerequisite to the main narrative.**

**K-compactness should be cited by B-series papers:**

B.1 and B.2 both use Polsby-Popper as their primary compactness metric. Reviewer comments in B.1 (Duchin round-2 review) explicitly request multi-metric compactness comparison, pointing toward exactly what K.0–K.7 provides. B.2's results section discusses PP limitations and notes "other metrics also improve." Neither B.1 nor B.2 cites any K-track paper by code. This is an expected consequence of publication order (B papers were written before K papers), but it means the K-track's systematic multi-metric treatment exists in isolation rather than being integrated as the answer to B-track reviewers' critique.

**L-partisan-fairness should be cited by G-series papers:**

G.2 (Partisan Outcome Distributions) and G.14 (Practitioner's Comparison, score 4.0/4) both discuss efficiency gap and seats-votes curve positioning of bisect plans within the ensemble. Neither cites L.1 (dedicated efficiency gap treatment), L.5 (seats-votes curve), or L.0 (framework). L.6's proportionality analysis directly complements T.5 (ProportionalSection), but no explicit cross-reference exists in either direction.

**D-track MODULE.md is stale — missing D.6, D.7, D.8:**

The D-vra-legal MODULE.md lists only papers D.0–D.5. Papers D.6 (Prison Gerrymandering), D.7 (Section 203 Language Minorities), and D.8 (Post-Shelby Landscape) appear in the filesystem and PAPERS.md but are absent from MODULE.md. This is a tracking failure, not a narrative failure, but it means any tool that reads MODULE.md for track enumeration will miscount D-track papers as 6 instead of 9.

**Summary of cross-citation gaps:**
- A.0 does not cite J-track (apportionment prerequisite unlinked)
- B.1 and B.2 do not cite K-track (compactness multi-metric critique unaddressed)
- G.2, G.14 do not cite L-track (partisan metrics framework unlinked)
- L.6 does not cite T.5 (ProportionalSection symmetry argument unlinked)

---

## 3. A-SYNTHESIS ACCURACY: FAIL

**A.0 (synthesis metapaper) — track coverage:**

A.0's abstract mentions no track letters, no paper count, no track structure. The introduction, findings, and conclusion sections (verified: sections/01–06 reviewed) describe the research in terms of empirical findings (VRA, compactness, stability) without any explicit mention of tracks I, J, K, or L. The bibliography contains no citation for any J, K, or L-track paper. The figure caption for Figure 1 (fig:architecture) states "Ten papers form a comprehensive investigation" — this caption is stale; the portfolio now has 12 tracks and 120+ papers.

**Specific A.0 failures:**
- The abstract mentions "six findings" and references "an 85-paper portfolio" nowhere — it says nothing about tracks I/J/K/L
- Section 04-findings.tex cites `deluca2026compactness` (B.2) and `deluca2026efficiency` (C.5) but no K or L paper codes
- Section 05-implications.tex mentions "Pareto frontier analysis" with citation `deluca2026compactness` (B.2) — K-track Pareto work is not cited
- The implications section does not mention apportionment verification (J.6)

**A.2 (portfolio summary) — track coverage:**

A.2's title reads "75+ Papers Across 8 Research Tracks." The portfolio now has 12 tracks. The portfolio architecture section in A.2 lists only Tracks A–H. Tracks I, J, K, L are not listed in the portfolio architecture section. Tracks G and H receive summary sections at the end, but I, J, K, L receive no coverage whatsoever. The "Citation Guide" section refers to "75+ papers across 8 tracks" as the citation string for the full portfolio — this is materially stale.

**Specific A.2 failures:**
- "75+ Papers Across 8 Research Tracks" in the title — should be "120+ Papers Across 12 Research Tracks"
- Portfolio architecture section omits Tracks I, J, K, L entirely
- "Citation Guide" states "75+ papers across 8 tracks" — must be updated

**D-vra-legal MODULE.md track count:**
MODULE.md lists D.0–D.5 only (6 papers); D.6/D.7/D.8 are present in the filesystem and PAPERS.md but absent from MODULE.md. Track D has 9 papers, not 6. This is the only MODULE.md with this defect; all other MODULE.md files match their filesystem contents.

---

## 4. IMPLEMENTATION ALIGNMENT: PARTIAL PASS

**K-track — compactness.rs:**

K-track papers claim implementation in `crates/bisect-analysis/src/compactness.rs`. Verified: the file exports `polsby_popper`, `reock`, `convex_hull_ratio`, `schwartzberg`, `length_width_ratio`, `population_weighted_compactness`, and `all_metrics` (lines 60, 78, 96, 119, 135, 224, 252). All six K.1–K.6 metric implementations are present. CLI flag `bisect label-analyze --types compactness` is consistently referenced.

**One K-track implementation discrepancy (critical):**

K.2 (Reock) claims "Welzl's algorithm provides the exact minimum bounding circle in O(n) expected time." The actual implementation uses a centroid-plus-max-distance approximation, not Welzl. The code's own comment reads "not the true MBC via Welzl's algorithm, but matching Python's approximation exactly." This discrepancy propagates to K.0, K.1, K.6, and K.7 which all cite "K.2, Section 3" for the Reock computation method. Post-write check (K-compactness/POST-WRITE-CHECK.md) correctly identifies this as a cross-paper P1 inherited from K.2. K.2 is rated "FIXES REQUIRED" with P2 severity for the abstract-body mismatch on the gap value, and the panel review rates it Major Revision (2.2/4) with the Welzl discrepancy as the top issue.

**Additional K-track implementation issue:** K.2 contains a false Proposition ("Reock(D) ≥ PP(D) for any convex polygon D") that is falsified by a computable example in K.1 (a 1×6 rectangle gives PP ≈ 0.38 > Reock ≈ 0.21). This mathematical error is propagated into K.0 and K.7 which repeat the Proposition.

**L-track — partisan.rs:**

L-track papers claim implementation in `crates/bisect-analysis/src/partisan.rs`. Verified: `compute_efficiency_gap` (line 110), `compute_mean_median` (line 131), `compute_partisan_bias` (line 151), `compute_declination` (line 176), `compute_seats_votes_curve` (line 215), and the composite `compute_partisan_metrics` function are all present. All five L.1–L.5 core metrics are implemented.

**One L-track implementation discrepancy (critical):**

L.1 (Efficiency Gap) defines EG > 0 as Republican advantage (Wasted_D > Wasted_R). The implementation computes `(wasted_r - wasted_d)` which is positive when R wastes more votes — i.e., Democratic advantage. The sign convention is inverted between the paper and the code. The test `test_efficiency_gap_direction` uses a "packed_dem_plan" and asserts EG < 0 — consistent with the code's convention but opposite to the paper's definition. L-track post-write check rates this as a P1 blocker.

**Second L-track discrepancy:** L.5 (Seats-Votes Curve) and the partisan bias implementation have mismatched sign conventions: `SeatsVotesCurve.bias` computes `s_50 - 0.50` (positive = D-favoring, matching the paper), while `compute_partisan_bias` returns `0.5 - seats_at_50/n` (positive = R-favoring, opposite). These two bias computations exist in the same struct and will report opposite signs for the same plan.

**J-track — bisect-apportion crate:**

J-track papers claim `bisect-apportion` crate implementation. Verified: the crate exists at `crates/bisect-apportion/`, with `huntington_hill.rs`, `divisor_methods.rs`, `paradoxes.rs` (including `check_alabama_paradox`), `prime.rs`, and `lib.rs`. The `huntington_hill` function (line 19) and `apportionment_divisor` function (line 42) are present.

**J-track implementation discrepancy (severe):**

J.6's abstract describes f64 floating-point arithmetic and SHA-256 verification as "planned for Phase 2." The companion papers J.0 and J.1 describe u128 integer arithmetic and SHA-256 as implemented and verified. Post-write check identifies this as a Phase-gap problem: the J.6 abstract appears to be a stale draft from an earlier development stage. The f64 reasoning in J.6's abstract is also mathematically incorrect: the priority cross-multiplication p_i^2 × s_j(s_j+1) ≈ 3×10^20 exceeds the 53-bit mantissa limit. J.6 is rated FIXES REQUIRED with 3 P1 blockers.

**Gallagher LSq (L.6):**

L.6 describes the Gallagher LSq metric. The `gallagher_index` function exists in `crates/bisect-analysis/src/dhondt.rs` (line 45), not in `partisan.rs`. L.6 describes it as a "computed metric" without specifying implementation location; the dhondt.rs location is non-obvious for a paper in the L-partisan-fairness track. This is a P2 documentation issue, not a missing implementation.

---

## 5. OPEN P1 BLOCKERS: 22 remaining

Compiled from all POST-WRITE-CHECK and PANEL-REVIEW-BATCH files across all 12 tracks.

**I-track (4 P1s in 2 papers):**
- I.1: WI pairing rate body text "3/28 = 0.107, rounded to 0.125" is arithmetically false (3/28 = 0.107 ≠ 0.125); denominator or count must be corrected
- I.2: TX bisect safe-seat fraction: abstract=0.55, table=0.58 — must align
- I.2: TX enacted safe-seat fraction: abstract=0.71, table=0.68 — must align
- I.2: WI enacted: text says "4 safe seats (2 safe-D, 3 safe-R)" — 2+3=5 ≠ 4; count/fraction must be reconciled

**J-track (3 P1s in 2 papers):**
- J.3: Abstract claims "Adams gives Rhode Island, Montana, and Alaska each one additional seat relative to HH" but paper's own §05 shows Adams and HH agree for all 50 states in 2020 — direct internal contradiction
- J.6: Abstract claims f64 arithmetic; spec and companion papers (J.0, J.1) require/describe u128 integer arithmetic; f64 cross-multiplication reasoning is incorrect
- J.6: Abstract says SHA-256 is "planned for Phase 2"; companion papers describe it as implemented and verified

**K-track (3 P1s in 2 papers, plus 1 cross-paper propagated):**
- K.2: Abstract states Reock-PP gap "+0.12" but paper body reports "+0.15" (body's value is consistent; abstract is wrong)
- K.2: Implementation section describes Welzl's algorithm but actual code uses centroid+max-distance approximation — falsified claim propagated to K.0, K.1, K.6, K.7
- K.3: Convex hull paper has 1 P1 blocker (post-write check rates it "FIXES REQUIRED" with Major Revision verdict — abstract claims results not delivered)
- K.2 false Proposition (Reock ≥ PP for convex polygons) propagated to K.0 and K.7

**L-track (3 P1s in 3 papers):**
- L.0: Conclusion states "approximately four times larger, consistent across all six metrics" — ratio ranges 3.5× to 7× across metrics; the "consistent" claim is false
- L.1: EG sign convention inverted between paper definition (positive=R advantage) and implementation (positive=D advantage); test asserts wrong direction
- L.4 (Declination): 1 P1 per post-write summary table ("FIXES REQUIRED") — specific issue documented in full review but summary shows 1 P1 fix required
- L.6: Gallagher LSq two-party simplification derivation path is confused/misleading though final formula is correct

**D-track (4 P1s in 2 papers — D.6, D.7; D.8 is clear):**
- D.6: NC prisoner count inconsistency — §Methodology says 900, §Results uses 1,800
- D.6: Abstract "1–3 districts" should be "1–2 districts"
- D.7: Abstract claims 12% TX county-split reduction; paper's headline result is 35% (county-weight vs. enacted); 12% is the geographic-mode improvement, wrong baseline used in abstract
- D.7: Abstract Section 203 formula missing English literacy conjunct

**A-synthesis (5 P1s across A.0 and A.2):**
- A.0 figure caption: "Ten papers form a comprehensive investigation" — stale count; 12 tracks, 120+ papers
- A.0: Callais citation "608 U.S. ___ (2026)" requires verification or "slip op." treatment before submission
- A.0: Stale `redist/` workspace reference and `redist-metis` vs `bisect-metis` crate name
- A.2: Title "75+ Papers Across 8 Research Tracks" — must be updated to 12 tracks, 120+ papers
- A.2: Mixed `bisect`/`redist` commands in figure reproduction code block — `redist map` should be `bisect map`

**C-validation (1 P1 in C.3):**
- C.3: Abstract inconsistency (0.8 pp vs 14 pp advantage metric confusion); p<0.001 from n=5 paired t-test (implausible); TBD cells in primary results tables (paper is substantively incomplete)

**E-experimental (1 P1 in E.6):**
- E.6: Abstract claims results for 6 countries; paper delivers prototype data for 3 — undelivered abstract claims (Major Revision 2.1/4)

**D-vra-legal MODULE.md:**
- MODULE.md lists D.0–D.5 only; D.6, D.7, D.8 exist in filesystem and PAPERS.md but are absent from MODULE.md

---

## OVERALL VERDICT: FIXES REQUIRED

**Priority fixes (ordered by blast radius):**

**Priority 1 — A-synthesis must be updated to reflect 12-track, 120+ paper portfolio**

A.0 figure caption, A.2 title, A.2 architecture section, and A.2 citation guide all reference the 8-track, 75-paper state. A.2 omits Tracks I, J, K, L entirely. Any reader of A.2 (the document most likely to be read first by external evaluators) receives a materially incomplete picture of the portfolio. This is the single highest-priority fix: update A.2 to cover all 12 tracks and correct the paper count throughout.

**Priority 2 — L.1 EG sign convention must be corrected before any partisan analysis is published**

The efficiency gap sign inversion between paper and code (L.1, plus the secondary conflict between `SeatsVotesCurve.bias` and `compute_partisan_bias` in L.5) means that every programmatic partisan analysis output is signed opposite to the paper's description. Any expert witness filing that cites both the L.1 paper and the `bisect label-analyze --types partisan` output will contain internally inconsistent sign conventions. This must be fixed in the code or unified in the papers before Track L can be considered publication-ready.

**Priority 3 — K.2 Welzl discrepancy must be corrected and Reock ≥ PP Proposition retracted**

The false implementation claim (Welzl claimed, centroid approximation used) and the false mathematical proposition (Reock ≥ PP for convex polygons) propagate to K.0, K.1, K.6, and K.7. Any court filing that cites Reock scores and references Welzl's algorithm as the computational method will be factually wrong.

**Priority 4 — I-track arithmetic errors (I.1 and I.2) must be corrected**

WI pairing rate arithmetic (3/28 ≠ 0.125) and TX safe-seat triple inconsistency (three different values across spec/abstract/table) are desk-rejection-level errors. These are likely transcription errors not methodological failures, but they block all I-track papers from proceeding to panel review.

**Priority 5 — J.3 and J.6 abstracts must be corrected**

J.3 abstract makes a false empirical claim (Adams diverges from HH in 2020; it does not). J.6 abstract describes a stale version of the software. Both are fixable by updating the abstract text to match the paper bodies.

**Priority 6 — D.6 and D.7 data and framing errors must be corrected**

D.6 NC prisoner count inconsistency (900 vs 1,800) and D.7 Texas headline stat wrong baseline (12% geographic improvement vs 35% county-weight improvement) are factual errors with direct bearing on the legal arguments these papers make.

**Priority 7 — Cross-citations should be added to unlock the K and L tracks' value**

B.1 and B.2 should add forward references to K-track papers as the systematic multi-metric compactness companion. G.2 and G.14 should cite L.1 and L.5 for the metric definitions they use. A.0 should cite J.1/J.6 for the Huntington-Hill apportionment verification that is its founding analogy. These are not P1 blockers but significantly improve portfolio coherence and make K and L tracks discoverable from the core papers.

**Priority 8 — D-vra-legal MODULE.md must be updated to include D.6, D.7, D.8**

**Not blocking public release — P2 items for later cycles:**
- Single-seed / ensemble significance framework (systematic P2 across I, K, L tracks)
- FL coverage gap in I-track (series-wide scope decision, needs explicit disclosure)
- A.0 Callais citation verification
- C.3 substantive completion (TBD cells in results tables)
- E.6 international applications completion (3 of 6 countries delivered)
- Harper v. Hall (NC 2022) legal status verification across L.0 and L.1

---

## SCOREBOARD SUMMARY

| Dimension | Score | Status |
|-----------|-------|--------|
| Portfolio Completeness | 7.5/10 | 4 structural gaps (polarized voting, local redistricting, ensemble inference, competitive-state coverage) |
| Narrative Arc | PARTIAL PASS | J before B unlinked; K/L not cited by B/G; D MODULE.md stale |
| A-synthesis Accuracy | FAIL | A.0 misses I/J/K/L entirely; A.2 title/count stale (8 tracks → 12, 75+ → 120+) |
| Implementation Alignment | PARTIAL PASS | K.2 Welzl claim false; L.1 EG sign inverted; J.6 abstract stale; core implementations verified |
| Open P1 Blockers | 22 remaining | Spread: A(5), C(1), D(4), E(1), I(4), J(3), K(3+1), L(3+1) |

**Overall: FIXES REQUIRED before public release**

The portfolio has strong bones: the core B/C/D/G/H tracks are in good shape, K and L implementations are substantially correct (modulo sign and Welzl issues), and the J-track apportionment crate is fully implemented. The primary obstacles to release are: the A-synthesis documents are frozen at the 8-track state, two critical sign/computation discrepancies in L.1 and K.2 would cause published claims to contradict pipeline output, and 22 P1 blockers across eight tracks require resolution before any of those papers can be submitted to external venues.

Estimated work to clear all Priority 1–5 fixes: 3–5 focused sessions. A-synthesis update (Priority 1) is the most impactful single action because it is the document most likely to be read first by external reviewers and judges.
