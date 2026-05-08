# POST-WRITE CHECK: D.6, D.7, D.8
**Date**: 2026-05-08
**Pipeline**: research-post-write (6-phase)
**Papers**: D.6 Prison Gerrymandering · D.7 Section 203 Language Minorities · D.8 Post-Shelby Landscape

---

## PHASE 1 — PAPER INVENTORY

### D.6 — Prison Gerrymandering and Algorithmic Redistricting
**Sections found**: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-legal, 06-conclusion
**Spec found**: YES — `docs/specs/2026-05-07-d6-prison-gerrymandering.md`
**Series**: D.6 | **Target**: Yale Law Journal | **Audience**: Legal / practitioner
**Key claims**:
1. ~58% of state/federal prison population is Black or Hispanic; rural districts with prisons are systematically over-populated, giving non-incarcerated residents excess representation — confirmed in §Background and Conclusion.
2. At least 12 states have enacted prison population adjustment laws as of 2026; bisect-data implements the adjustment via `--adjust-prison-population` on `bisect fetch` — confirmed in §Background Table 1 and §Methodology.
3. Adjustment shifts 1–2 districts per state with partisan lean changes of 0.3–0.5 pp and BVAP/HVAP shifts of 0.4–0.8 pp — confirmed in §Results Table 2 and §Conclusion.

### D.7 — Section 203 Language Minority Requirements and Algorithmic Redistricting
**Sections found**: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-legal, 06-conclusion
**Spec found**: YES — `docs/specs/2026-05-07-d7-section-203-language.md`
**Series**: D.7 | **Target**: Michigan Law Review | **Audience**: Legal / practitioner
**Key claims**:
1. Section 203 covers 331 jurisdictions in 31 states (2023 determination, 88 Fed. Reg. 34,396); threshold is ≥5% or ≥10,000 voting-age citizens with below-national English literacy — confirmed in §Abstract, §Background.
2. bisect county-weight mode reduces Section 203 covered county splits by 35% in TX, 9% in CA vs. enacted maps — confirmed in §Results Table.
3. 23 counties nationally have both Section 203 coverage and Section 2 Gingles feasibility; bisect VRA mode + county-weight handles both — confirmed in §Legal.

### D.8 — The Post-Shelby VRA Landscape and Algorithmic Redistricting
**Sections found**: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-legal, 06-conclusion
**Spec found**: YES — `docs/specs/2026-05-07-d8-post-shelby-landscape.md`
**Series**: D.8 | **Target**: Columbia Law Review | **Audience**: Legal / practitioner
**Key claims**:
1. Shelby County v. Holder (570 U.S. 529, 2013) invalidated Section 4(b)'s coverage formula, making Section 5 a dead letter without formally repealing it — correctly stated.
2. Brnovich v. DNC (594 U.S. 647, 2021) weakened Section 2 vote-denial claims but expressly preserved Section 2 vote-dilution / Gingles framework for redistricting — correctly characterized.
3. Allen v. Milligan (599 U.S. 1, 2023) reaffirmed Gingles for redistricting; bisect VRA mode functions as a transparency substitute for Section 5's documentation function — confirmed.

---

## PHASE 2 — CONSISTENCY CHECK

### D.6 Consistency Registry

| Q-ID | Quantity | Abstract | §Intro/Background | §Results | §Conclusion | Consistent? |
|------|----------|---------|-------------------|----------|-------------|-------------|
| Q-01 | Prison population % Black or Hispanic | 58% | 58% (§Background) | — | 58% (§Conclusion) | PASS |
| Q-02 | Incarcerated persons (2020 census) | — | 1.4 million | — | 1.4 million | PASS |
| Q-03 | States with adjustment laws | 12 | 12 (§Background para + Table 1) | — | 12 | PASS |
| Q-04 | NC inmates reallocated | — | ~900 (§Methodology, Hoke CI) | ~1,800 (§Results, "Hoke, Scotland, Davidson counties") | — | **FAIL: 900 vs 1,800** |
| Q-05 | States in empirical study | "NC, TX, CA" | "NC, TX, CA" | NC, TX, CA | "NC, TX, CA" | PASS |
| Q-06 | NC districts affected | "1–3 districts per state" | — | 1 | "1–2 districts" | **WARN: abstract says 1–3, conclusion says 1–2, results say 1** |
| Q-07 | NC partisan shift | 0.3–0.5 pp (abstract) | — | +0.3 pp (NC) | 0.3–0.5 pp | PASS (abstract gives range, NC result is at lower end) |
| Q-08 | TX partisan shift | 0.3–0.5 pp (abstract) | — | +0.5 pp (D18), +0.4 pp (D20) | 0.3–0.5 pp | PASS |
| Q-09 | CA validation match | within 0.1% (abstract) | — | 0.1% (§Results) | — | PASS |
| Q-10 | TX prison population | — | — | ~130,000 | — | PASS (single appearance, no contradiction) |
| Q-11 | Dagger notation applied | Yes (abstract notes single-seed) | — | Consistently applied in §Results | — | PASS |

**D.6 Critical Issue — Q-04**: §Methodology (Hoke CI case study) states "approximately 900 inmates." §Results (NC section) states "approximately 1,800 incarcerated individuals from rural correctional facilities in Hoke, Scotland, and Davidson counties." These are different: §Methodology discusses only Hoke CI (900), while §Results adds Scotland and Davidson county facilities to reach 1,800. This is not labeled as such — the reader is left to infer the expansion. The jump is real (more facilities, not a contradiction) but never explained. Needs a bridging sentence.

**D.6 Q-06**: Abstract says "1–3 districts per state"; §Conclusion says "1–2 districts are affected per state." The empirical results show NC=1, TX=2, CA=validation only. Neither "3" nor "1–3" is supported by results. The abstract upper bound of 3 is undefended.

```
CONSISTENCY D.6: 2 issues
P1 (reject): [Q-04] 900 vs 1,800 prisoner count in NC — §Methodology and §Results are inconsistent without explanation
P2 (revision): [Q-06] Abstract claims "1–3 districts" but results show max 2; conclusion also says "1–2" — fix abstract
P3 (minor): none
```

### D.7 Consistency Registry

| Q-ID | Quantity | Abstract | §Background | §Results | §Conclusion | Consistent? |
|------|----------|---------|-------------|----------|-------------|-------------|
| Q-12 | Covered jurisdictions | 331 | 331 | — | 331 | PASS |
| Q-13 | States covered | 31 | — | — | — | PASS (stated once) |
| Q-14 | Federal Register cite | 88 Fed. Reg. 34,396 | 88 Fed. Reg. 34,396 | — | — | PASS |
| Q-15 | Section 203 threshold | 5% or 10,000 (abstract) | 5% or 10,000 (§Background) | — | — | PASS |
| Q-16 | TX Spanish-covered counties | — | — | 54 (§Results) | — | PASS (internal) |
| Q-17 | TX total covered jurisdictions | — | 67 (§Background Table) | 67 (§Results intro) | — | PASS |
| Q-18 | TX enacted splits (Spanish) | — | — | 17 | — | PASS |
| Q-19 | bisect geographic mode TX splits | — | — | 15 | — | PASS |
| Q-20 | bisect county-weight TX splits | — | — | 11 | — | PASS |
| Q-21 | TX reduction (county-weight) | 12% (abstract) | — | 35% (§Results) | 35% (§Conclusion) | **FAIL: abstract says 12%, results/conclusion say 35%** |
| Q-22 | CA reduction (county-weight) | 9% (abstract) | — | 9% (§Results) | 9% (§Conclusion) | PASS |
| Q-23 | Vietnamese tracts CA (county-weight) | 3 of 4 (abstract) | — | 3 of 4 (§Results) | 3 of 4 (§Conclusion) | PASS |
| Q-24 | Counties with both §203 and §2 | 23 (abstract) | — | 23 (§Legal) | 23 (§Conclusion) | PASS |
| Q-25 | Dagger notation | Applied in abstract | — | Consistently applied | — | PASS |

**D.7 Critical Issue — Q-21**: The abstract states "bisect county-weight plans reduce Section 203 county splits by 12% and 9% respectively relative to the enacted 2022 congressional maps." In the body, 12% is the reduction from the geographic mode to the enacted map for TX (geographic: 15, enacted: 17 → improvement from geographic over enacted = (17-15)/17 = 11.8% ≈ 12%). But the 35% figure is county-weight vs. enacted: (17-11)/17 = 35.3%. The abstract therefore reports the geographic-mode improvement (12%) where it should report the county-weight improvement (35%). The California figure (9%) is the county-weight improvement in both cases. This is a **P1 blocker**: the abstract headline stat for Texas uses the wrong baseline.

```
CONSISTENCY D.7: 1 critical failure
P1 (reject): [Q-21] Abstract claims 12% TX county-split reduction but paper's headline result is 35% (county-weight vs. enacted); 12% is the geographic-mode improvement, not the county-weight improvement — abstract must be corrected
P2 (revision): none
P3 (minor): none
```

### D.8 Consistency Registry

| Q-ID | Quantity | Abstract | §Intro | §Background/Methodology | §Results | §Legal/Conclusion | Consistent? |
|------|----------|---------|--------|------------------------|----------|-------------------|-------------|
| Q-26 | Shelby County citation | 570 U.S. 529, 2013 | 570 U.S. 529 (2013) | 570 U.S. 529 (2013) | — | 570 U.S. 529 | PASS |
| Q-27 | Brnovich citation | 594 U.S. 647, 2021 | — | 594 U.S. 647 | — | — | PASS (BIB correct) |
| Q-28 | Allen v. Milligan citation | 599 U.S. 1, 2023 | — | — | 599 U.S. 1 (2023) | 599 U.S. 1 | PASS |
| Q-29 | Abbott v. Perez citation | 585 U.S. 579, 2018 | §Intro overview mentions "Abbott v. Perez (2018)" | 585 U.S. 579 | — | — | PASS |
| Q-30 | Brnovich spec citation | spec says "141 S.Ct. 2321" | Paper uses 594 U.S. 647 | 594 U.S. 647 | — | — | **WARN: spec uses slip opinion, paper uses US Reports** |
| Q-31 | Allen spec citation | spec says "523 U.S. 1310" (wrong — that is a stay, not merits) | Paper correctly uses 599 U.S. 1 | 599 U.S. 1 | — | — | **PASS: paper fixed spec error** |
| Q-32 | 9 fully covered states | §Background lists 9 | Alabama, Alaska, Arizona, Georgia, Louisiana, Mississippi, SC, TX, VA = 9 | — | — | PASS |
| Q-33 | Congressional inaction | "as of 2026" (abstract) | §Background §3 | — | — | PASS |
| Q-34 | Covington citation | — | — | 316 F.R.D. 117 (M.D.N.C. 2016) | — | — | PASS (verified against BIB) |
| Q-35 | NY John Lewis Act | 2022 (§Results) | — | — | — | PASS (consistent with BIB) |

**D.8 Note on Q-30**: The spec states `Brnovich v. DNC (141 S.Ct. 2321, 2021)`. The paper cites 594 U.S. 647 (2021), which is the official United States Reports citation. The spec used the Westlaw slip-opinion citation. Paper is correct; spec was approximate. No action needed.

**D.8 Note on Q-31 — Critical correction**: The spec's Test Invariant states "Allen v. Milligan (523 U.S. 1310, 2023)". This is **wrong** — 523 U.S. does not exist (current volumes are ~599-600). The 523 citation would be a 2003 case. The paper correctly cites 599 U.S. 1 (2023). This is a pre-existing error in the spec, not the paper. Paper is right.

```
CONSISTENCY D.8: PASS
P1: none
P2: none
P3: [Q-30] note in paper that Brnovich has both 141 S.Ct. 2321 and 594 U.S. 647 citations (same case; practitioners use both)
```

---

## PHASE 3 — CONTRACT CHECK

### D.6 Contract

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Survey constitutional/legal framework | §Background (Evenwel + §2 Gingles mention) | YES | ✓ |
| Evenwel v. Abbott (578 U.S. 54, 2016) — correct holding stated | §Background §2 | YES — holding correctly stated: states MAY use total population; adjustment is locational not base change | ✓ |
| Survey of enacted state adjustment laws | §Background Table 1 (12 states) | YES | ✓ |
| bisect-data preprocessing handles adjusted data | §Methodology | YES — `bisect fetch --adjust-prison-population` | ✓ |
| Empirical results for TX, CA, NY (spec says NY; paper delivers NC) | §Results | PARTIAL — spec lists TX, CA, NY; paper delivers TX, CA, NC | WARN |
| Validate CA against official CRC data within 0.1% | §Results §CA | YES | ✓ |
| L0 invariants: population conservation + non-negativity | §Methodology §Data Invariants | YES | ✓ |
| Expert witness / court filing guidance | §Legal | YES — 3 subsections | ✓ |
| State-by-state scope of adjustment laws (congressional vs. state) | §Legal Table 2 | YES | ✓ |
| DHC-A file mechanics documented | §Methodology (Step 1–3) | YES | ✓ |

**NY vs. NC gap**: The spec says empirical targets include NY; the paper delivers NC. This is a minor substitution (NC is also a high-profile redistricting state with relevant prisons). The spec's target states were "TX, CA, NY, FL, PA, OH" with the note to pick the relevant ones. NC is not on that list. This needs a one-line justification in §Methodology or §Results explaining why NC was chosen over NY/FL.

```
CONTRACT D.6: PARTIAL (9/10 promises kept)
Gaps:
- Spec empirical targets include NY; paper uses NC without explanation. Minor gap but worth a footnote.
```

### D.7 Contract

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Section 203 coverage formula explained | §Background | YES | ✓ |
| 2023 coverage determination (331 jurisdictions, 31 states) | §Abstract + §Background | YES | ✓ |
| bisect county-weight reduces county splits | §Results | YES — 35% TX, 9% CA | ✓ |
| TX and CA empirical comparison (bisect vs. enacted) | §Results | YES | ✓ |
| Vietnamese community in Santa Clara County | §Results §CA | YES — 3 of 4 tracts | ✓ |
| 23 dual-coverage counties identified | §Legal | YES | ✓ |
| bisect VRA + county-weight dual optimization documented | §Legal | YES | ✓ |
| Expert witness documentation guidance | §Legal §Documentation | YES | ✓ |
| Section 203 hierarchy vs. Section 2 | §Legal §Gingles-203 Interaction | YES | ✓ |

```
CONTRACT D.7: PASS (9/9 promises kept)
Gaps: none substantive
```

### D.8 Contract

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Shelby County holding correctly stated | §Background | YES | ✓ |
| Section 5 / Section 4(b) framework explained | §Background | YES | ✓ |
| Congressional inaction on new coverage formula | §Methodology (§Congressional Inaction) | YES | ✓ |
| Abbott v. Perez (2018) analyzed | §Methodology | YES | ✓ |
| Brnovich v. DNC (2021) — vote-denial vs. vote-dilution distinction | §Methodology | YES — expressly stated | ✓ |
| Allen v. Milligan (2023) — Gingles reaffirmed | §Results | YES | ✓ |
| State VRA analogs (CA, NY, WA) | §Results | YES | ✓ |
| bisect as transparency substitute for Section 5 | §Legal | YES — documentation + deterrence functions | ✓ |
| 3 litigation use cases | §Legal | YES — plaintiff, defendant, special master | ✓ |
| Former covered states listed | §Background | YES — 9 states listed correctly | ✓ |

```
CONTRACT D.8: PASS (10/10 promises kept)
Gaps: none
```

---

## PHASE 4 — REFEREE SIMULATION

### D.6 — Prison Gerrymandering

---
**REFEREE 1 (R1) — Algorithms / Data Pipeline**
Archetype: SODA/IEEE Data Engineering
**Recommendation**: Minor Revision

**SUMMARY**: The methodology section adequately describes the three-step adjustment algorithm using DHC-A GQ Types 101 and 201. The L0 invariants (population conservation, non-negativity) are correctly identified. The validation against California CRC data (0.1% match) is the paper's strongest empirical contribution. However, the paper leaves several pipeline implementation claims unverified.

**MAJOR CONCERNS**:
[I-01] The paper states `bisect fetch --adjust-prison-population` exists as a CLI flag, but the CLAUDE.md CLI reference does not list this flag. If the flag is not yet implemented, this is a forward-looking claim that must be labeled as such (e.g., "proposed implementation" or "forthcoming in bisect v2.x"). As written it reads as current functionality.

[I-02] The NPS county-level home distribution is used as the reallocation weight in Step 3. The paper says "for states without county-level NPS data, the adjustment uses the state-level distribution weighted by county population." This is a significant methodological assumption that is mentioned but not validated. How many states lack county-level NPS data? What is the error introduced by the state-level approximation? Without a sensitivity analysis, the 0.1% CA validation is the only quality check.

**MINOR CONCERNS**:
[I-03] GQ Types 101 and 201 are stated but not explained for a general reader. A footnote on Census GQ type taxonomy would help.
[I-04] The 900 vs. 1,800 NC inmate count (§Methodology vs. §Results) must be reconciled before submission.

---
**REFEREE 2 (R2) — Political Science**
Archetype: APSR / Journal of Politics
**Recommendation**: Minor Revision

**SUMMARY**: The partisan and VRA impact analysis is directionally sound but technically thin. The paper correctly uses single-seed dagger notation and appropriately hedges all results. The 0.3–0.5 pp partisan shift finding will be credible to political scientists working on redistricting because the magnitudes are plausible and well-grounded in the population transfer arithmetic.

**MAJOR CONCERNS**:
[I-05] The paper claims Texas has "approximately 130,000 state prisoners in 2020" and reallocates "approximately 130,000 individuals." But the §Results then says the Harris County reallocation alone adds 48,000 and Bexar adds 22,000 = 70,000. If the total reallocation is 130,000 and only 70,000 is accounted for in named districts, where do the other 60,000 go? The paper needs either a distribution breakdown or a clarification that only a portion of TX prisoners originates from Harris/Bexar.

[I-06] All empirical results are single-seed (correctly noted). A one-paragraph discussion of whether multi-seed convergence runs would materially change the boundary of the affected district would strengthen the credibility of the claims. The paper implicitly argues the adjustments are small enough not to matter at the plan level — this could be shown by noting that a ±0.3 pp partisan shift is well within single-seed variance.

**MINOR CONCERNS**:
[I-07] VEST 2020 presidential vote shares cited as the partisan metric but not introduced or justified in the methodology. A sentence in §Methodology identifying VEST as the data source would suffice.

---
**REFEREE 3 (R3) — Legal / Practitioner**
Archetype: Yale Law Journal / Public Administration
**Recommendation**: Minor Revision

**SUMMARY**: The legal analysis is the paper's strongest section. The Evenwel characterization is impeccable: the paper correctly explains that prison adjustment is a locational assignment within total population, not a change to the apportionment base, and correctly notes that Evenwel does not address prison gerrymandering. The state adjustment law survey (Table 1) is thorough and Table 2's congressional vs. state legislative scope distinction is exactly what practitioners need.

**MAJOR CONCERNS**:
[I-08] The paper relies on Table 1 for statutory citations without primary-source verification in every instance. Specifically, Michigan's citation to Mich. Comp. Laws § 168.931 (enacted 2022) is in the table but the 2022 enactment is recent and worth flagging with a practitioner caution: "Verify current statute; this is the most recent enactment and may have been amended." Without this caveat, a practitioner following the table without independent verification could encounter a different statutory framework.

[I-09] The paper states Colorado is "the only state whose adjustment law applies explicitly to congressional redistricting." This is a strong and consequential claim for practitioners. If it is wrong (e.g., Washington's law also covers congressional redistricting through a broader scope provision), practitioners will be misled. The paper should add a footnote with a verification note or cite a primary authority that explicitly excludes the other eleven states from congressional applicability.

**MINOR CONCERNS**:
[I-10] The §Legal section advises expert witnesses to use adjusted MVAP for Gingles analysis in adjustment states. This is sound but the paper should note that courts have not uniformly accepted adjusted data for VRA analysis — practitioners should verify whether the applicable federal district court has addressed the point.

---

### D.7 — Section 203 Language Minorities

---
**REFEREE 1 (R1) — Algorithms / Data Pipeline**
Archetype: Data science / computational redistricting
**Recommendation**: Accept

**SUMMARY**: The county-split metric and community integrity metric are clearly defined with formal notation. The bisect mode comparison (geographic vs. county-weight) is well-framed. The paper appropriately restricts results to single-seed runs with dagger notation. The methodology is stronger than D.6 because there is no unresolved count discrepancy.

**MAJOR CONCERNS**:
[I-11] The abstract states 12% TX county-split reduction but the body delivers 35%. This is the most important issue in the paper and must be fixed (see Consistency check Q-21). Until corrected, the abstract is literally wrong about the headline empirical result.

**MINOR CONCERNS**:
[I-12] The community integrity metric is defined for the plurality district, not for a specific threshold. A score of 0.73 (Harris County Vietnamese) sounds strong, but the metric depends on the number of identified tracts and the district count. A brief note on whether 0.73 is "good" in the Texas 38-district context would help readers calibrate.
[I-13] The paper does not state how Vietnamese-majority tracts are defined (≥25% threshold noted in CA section, but no parallel definition given for TX). Methodological consistency requires the same definition across both state analyses.

---
**REFEREE 2 (R2) — Political Science**
Archetype: APSR / State Politics & Policy
**Recommendation**: Minor Revision

**SUMMARY**: The paper makes a novel and useful contribution by connecting Section 203 administration to redistricting metrics. The Section 203 / Section 2 interaction is correctly characterized: Section 2 is legally paramount, Section 203 is a secondary optimization. The 23-county dual-coverage finding is the paper's strongest empirical claim and is appropriately grounded.

**MAJOR CONCERNS**:
[I-14] The paper analyzes Texas (38 districts) and California (52 districts) but the number of districts per state is never stated. A reader unfamiliar with congressional apportionment cannot evaluate whether 17 county splits in a 38-district state is more or less than expected. A brief statement of total county count and district count would contextualize the split counts.

[I-15] The 12% abstract error (Q-21 in consistency check) affects the reader's ability to evaluate the paper's core claim. This is a political science concern as well as an algorithms concern.

**MINOR CONCERNS**:
[I-16] The dispersed community analysis in §Methodology is conceptually valuable but not empirically supported. The paper identifies the problem (dispersed communities may not benefit from compactness optimization) without showing any data on dispersed communities. Either add data or reframe this as a theoretical limitation requiring future work.

---
**REFEREE 3 (R3) — Legal / Practitioner**
Archetype: Michigan Law Review / election administration
**Recommendation**: Accept with minor revision

**SUMMARY**: This paper fills a genuine gap. Section 203 is systematically overlooked in redistricting analysis. The legal framework (coverage formula, enforcement, Federal Register determination) is accurately stated. The hierarchy (Section 2 > Section 203) is correctly established. The Chinatown Voter Education Alliance citation is a nice touch for legal readers.

**MAJOR CONCERNS**:
[I-17] The 2023 coverage determination (88 Fed. Reg. 34,396) is stated as covering 331 jurisdictions consistently. However, the abstract also says "5%/10,000 threshold" which is the correct shorthand for the two alternative prongs. One minor risk: the paper never explicitly states that BOTH prongs (population AND literacy) must be satisfied simultaneously, as opposed to either population criterion independently triggering coverage. The background correctly states this (§Background §Coverage Formula) but the abstract's shorthand "exceeds 5% of voting-age citizens or 10,000 voting-age citizens" omits the English literacy conjunct — a reader who reads only the abstract will misunderstand the coverage formula as requiring population alone.

**MINOR CONCERNS**:
[I-18] The paper cites Chinatown Voter Education Alliance v. Ravitz (2009) for private plaintiff standing under Section 203. This citation is correct but the case is obscure. A parenthetical explaining that DOJ has primary enforcement authority but private plaintiffs may also sue would help practitioners.

---

### D.8 — Post-Shelby Landscape

---
**REFEREE 1 (R1) — Algorithms / Data Pipeline**
Archetype: Computational social science
**Recommendation**: Accept

**SUMMARY**: D.8 is primarily a legal synthesis paper with algorithmic framing. The bisect pipeline's role as transparency substitute for Section 5 is argued coherently. The SHA audit chain claim is consistent with bisect's documented functionality. No methodology issues. This paper functions well as legal synthesis.

**MAJOR CONCERNS**: None.

**MINOR CONCERNS**:
[I-19] The paper claims bisect "operates without partisan data" (§Introduction). This is true for the base algorithm but the paper elsewhere cites VEST presidential vote shares for comparison. A reader unfamiliar with bisect may wonder whether VEST data enters the pipeline at any point. A clarification that VEST is used for post-hoc analysis only, not as a bisect input, would be precise.
[I-20] Use Case 1 (plaintiff reference plan) says bisect with VRA mode provides a "reasonably configured" district under Allen v. Milligan. The connection to the Allen "reasonably configured" language is correct but should cite the specific passage from Allen (599 U.S. at 18-20, where the Court discusses the compactness of the illustrative districts).

---
**REFEREE 2 (R2) — Political Science**
Archetype: APSR / comparative voting rights
**Recommendation**: Accept

**SUMMARY**: The enforcement asymmetry framing (ex ante Section 5 vs. ex post Section 2) is correct, well-argued, and empirically grounded by the Table comparing pre- and post-Shelby enforcement dimensions. The state VRA analog section (CA, NY, WA) is accurate and useful. The claim that NY's John R. Lewis VRA is "the most comprehensive restoration of preclearance-like review" is defensible and appropriately hedged.

**MAJOR CONCERNS**: None.

**MINOR CONCERNS**:
[I-21] The paper mentions "Covington v. North Carolina" (316 F.R.D. 117) finding 28 of 170 state legislative districts were racially gerrymandered. This is cited in passing as an example of post-Shelby redistricting consequences. The citation is correct, but the paper does not note that this was a state legislative case, not a congressional case — which is the focus of the bisect pipeline. A clarifying parenthetical would be appropriate.
[I-22] Alabama's 2021 plan timeline is used as evidence of the enforcement asymmetry: "not remedied until after the 2022 election." This is accurate (Allen v. Milligan was decided June 8, 2023). The paper is correct.

---
**REFEREE 3 (R3) — Legal / Practitioner**
Archetype: Columbia Law Review / constitutional law
**Recommendation**: Accept

**SUMMARY**: D.8 is the strongest of the three papers on legal accuracy. Every case is correctly cited with accurate holdings. The characterization of Brnovich as limited to vote-denial claims (not redistricting/vote-dilution) is exactly right and important for practitioners to understand. The Footnote 19 of Brnovich reference (preserving vote-dilution analysis) is cited correctly (594 U.S. at 680 n.19) — this precision will be noticed and appreciated by law review editors.

**MAJOR CONCERNS**: None.

**MINOR CONCERNS**:
[I-23] The paper characterizes Abbott v. Perez as raising "the burden on plaintiffs seeking to prove discriminatory intent." This is correct for Equal Protection Clause claims (racial gerrymandering under Shaw v. Reno) but potentially ambiguous for practitioners who may conflate this with the Gingles results test. A clarifying sentence — "Abbott addressed intent-based Equal Protection claims, not the results-based Gingles framework" — is already implied in the text but could be made explicit.
[I-24] The spec's test invariant for Allen v. Milligan listed the citation as "523 U.S. 1310" which is wrong (no such volume). The paper correctly uses 599 U.S. 1. No action needed in the paper, but the spec should be flagged if it is used for future reference.

---

## PHASE 5 — ABSTRACT CHECK

### D.6 Abstract
**Word count**: 170 words
**Primary result stated**: YES — "1–3 districts per state, with partisan lean changes of 0.3–0.5 percentage points"
**Algorithm named**: YES — bisect-data preprocessing module, `bisect fetch`
**Value proposition**: YES — "provides expert witnesses and courts with a verifiable, neutral implementation"
**Issue**: "1–3 districts per state" is not supported by results (max is 2). Must be corrected to "1–2 districts."

### D.7 Abstract
**Word count**: 188 words
**Primary result stated**: YES — "12% and 9% respectively" — BUT 12% is the wrong figure for Texas (should be 35%)
**Algorithm named**: YES — bisect county-weight mode (`--weights-override county`)
**Value proposition**: YES — administrative burden reduction, expert witness guidance
**Issue**: The headline figure for Texas (12%) is wrong. Must be corrected to 35% before submission.

### D.8 Abstract
**Word count**: 204 words (slightly over 200-word target)
**Primary result stated**: PARTIAL — paper's primary contribution is the "transparency substitute" argument, stated but without quantification (appropriate for legal synthesis)
**Algorithm named**: YES — bisect VRA mode
**Value proposition**: YES — post-Shelby navigation for redistricting practitioners
**Issue**: 204 words is 4 over the 200-word target. Minor trim possible.

---

## PHASE 6 — PRE-PANEL CHECKLISTS

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: D.6 — Prison Gerrymandering
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   2 issues (1 P1, 1 P2)
  Contract:      PARTIAL (9/10 — NY vs. NC substitution unexplained)
  Referee sim:   Minor Revision (R1, R2, R3 all Minor Revision)
  Abstract:      170 words

P1 blockers (fix before panel review):
[I-04/Q-04] NC inmate count: §Methodology says "~900" (Hoke CI only);
            §Results says "~1,800" (Hoke + Scotland + Davidson).
            → Add a sentence in §Results: "The 900 Hoke CI inmates plus
              approximately 900 from Scotland and Davidson county facilities
              total approximately 1,800 statewide prison adjustments" —
              or restructure §Methodology to introduce all three facilities.

[I-11/Q-21] *** D.6 has no Q-21; see D.7 ***

[Q-06] Abstract says "1–3 districts per state"; results show max 2;
        conclusion says "1–2".
        → Fix abstract to "1–2 districts per state" to match results.

P2 items (should fix):
[I-01]  `--adjust-prison-population` flag — verify it is implemented in
        the bisect binary. If not implemented, label as "proposed" or
        add to REDIST_CLI.md as upcoming.
[I-05]  TX 130,000 reallocation: 48k (Harris) + 22k (Bexar) = 70k accounted
        for. Explain where remaining 60k originates or clarify scope of the
        Harris/Bexar discussion.
[I-09]  Colorado "only state" claim for congressional scope — add a footnote
        with primary statutory verification.
[Spec]  Add one-sentence justification for using NC instead of NY as the
        third case study state.

P3 items (optional):
[I-03]  Footnote on GQ Type taxonomy (101 = Federal, 201 = State).
[I-07]  Introduce VEST 2020 in §Methodology as partisan data source.
[I-10]  Note that courts have not uniformly accepted adjusted MVAP for VRA.

PRE-PANEL CHECKLIST D.6:
□ P1: NC count discrepancy (900 vs. 1,800) resolved in §Results
□ P1: Abstract "1–3 districts" corrected to "1–2 districts"
□ P2: --adjust-prison-population flag verified as implemented or labeled prospective
□ P2: TX 130,000 distribution accounted for
□ P2: Colorado "only state" claim verified or hedged
□ P2: NC substitution for NY justified in paper
□ Single-run results marked with dagger notation: YES (verified)
□ CLI flags match actual bisect binary flags: NEEDS VERIFICATION
□ Court citations verified — Evenwel 578 U.S. 54 (2016): PASS
□ Abstract states primary quantitative result: YES (with fix)
□ Referee P1 blockers addressed: 4 items (I-01, I-04/Q-04, Q-06, I-05)

VERDICT: FIXES REQUIRED
Fixes required: 6 (2 P1, 4 P2)
═══════════════════════════════════════════════════════
```

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: D.7 — Section 203 Language Minorities
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 critical failure (P1)
  Contract:      PASS (9/9 promises)
  Referee sim:   Accept with minor revision (R3 Accept; R1/R2 minor revision)
  Abstract:      188 words

P1 blockers (fix before panel review):
[I-11/Q-21] Abstract Texas figure: "12%" must become "35%".
            The 12% figure is the geographic-mode improvement over enacted
            (not the county-weight improvement). The paper's headline
            result is county-weight vs. enacted = 35%.
            → Abstract sentence currently reads:
              "bisect county-weight plans reduce Section 203 county
               splits by 12% and 9% respectively"
            → Must read:
              "bisect county-weight plans reduce Section 203 covered
               county splits by 35% in Texas and 9% in California"
            Also add: "The bisect standard geographic mode achieves a
               12% reduction in Texas, confirming that county-weight
               optimization provides additional benefit beyond geographic
               compactness alone."

[I-17]     Abstract Section 203 formula: "exceeds 5% of voting-age
            citizens or 10,000 voting-age citizens" omits the English
            literacy conjunct. 
            → Fix to: "exceeds 5% of voting-age citizens or 10,000
               voting-age citizens, AND the group's English literacy rate
               falls below the national average"

P2 items (should fix):
[I-13]  Vietnamese-majority tract definition in TX (threshold undefined
        vs. "≥25%" used in CA). Use consistent definition.
[I-14]  Add district count and county count for TX (38 districts, 254
        counties) and CA (52 districts, 58 counties) to contextualize
        split numbers.
[I-16]  Dispersed community analysis in §Methodology: add data or
        reframe as "future work" limitation.

P3 items (optional):
[I-12]  Calibration note for community integrity scores (0.73 = strong
        in a 38-district state).
[I-18]  Expand Chinatown Voter Education Alliance parenthetical for
        practitioner readers.

PRE-PANEL CHECKLIST D.7:
□ P1: Abstract 12% → 35% corrected for Texas county-weight figure
□ P1: Abstract Section 203 formula — English literacy conjunct added
□ P2: Vietnamese tract definition consistent across TX and CA
□ P2: District/county counts stated for TX and CA
□ Single-run results marked with dagger notation: YES (verified)
□ CLI flags match actual bisect binary flags: --weights-override county, vra-aligned — verified
□ Court citations verified — Thornburg v. Gingles 478 U.S. 30 (1986): PASS
□ Shelby County 570 U.S. 529 (2013): PASS
□ Abstract states primary quantitative result: YES (after fix)
□ Referee P1 blockers addressed: 2 items (I-11, I-17)

VERDICT: FIXES REQUIRED
Fixes required: 5 (2 P1, 3 P2)
═══════════════════════════════════════════════════════
```

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: D.8 — Post-Shelby VRA Landscape
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   PASS (no failures)
  Contract:      PASS (10/10 promises)
  Referee sim:   Accept (R1, R2, R3 all Accept)
  Abstract:      204 words (4 over target)

P1 blockers (fix before panel review):
NONE

P2 items (should fix):
[I-19]  Clarify that VEST presidential vote shares are post-hoc analysis
        only, not a bisect pipeline input, to prevent reader confusion.
[I-20]  Add pinpoint cite to Allen v. Milligan (599 U.S. at 18-20)
        for the "reasonably configured" language.
[I-23]  Add explicit clarification that Abbott v. Perez addresses
        Equal Protection intent claims, not Gingles results test.

P3 items (optional):
[I-21]  Note that Covington v. NC involved state legislative districts,
        not congressional districts.
Abstract: trim 4 words to meet 200-word target.

PRE-PANEL CHECKLIST D.8:
□ P1 blockers: NONE
□ P2: VEST clarification (post-hoc analysis, not bisect input)
□ P2: Allen v. Milligan pinpoint cite (at 18-20)
□ P2: Abbott v. Perez scope clarification
□ Single-run results: D.8 is legal synthesis — no empirical results, no dagger needed
□ CLI flags: --weights-override vra-aligned verified
□ Court citations verified:
  □ Shelby County 570 U.S. 529 (2013): PASS
  □ Brnovich v. DNC 594 U.S. 647 (2021): PASS (paper corrects spec's 141 S.Ct. citation)
  □ Allen v. Milligan 599 U.S. 1 (2023): PASS (paper corrects spec's wrong "523 U.S. 1310")
  □ Abbott v. Perez 585 U.S. 579 (2018): PASS
  □ Beer v. United States 425 U.S. 130 (1976): PASS
  □ Covington v. NC 316 F.R.D. 117 (M.D.N.C. 2016): PASS
□ Abstract states primary contribution: YES (transparency substitute argument)
□ Referee P1 blockers addressed: N/A

VERDICT: READY FOR PANEL (with P2 cleanup recommended)
Fixes required: 3 P2 (no P1)
═══════════════════════════════════════════════════════
```

---

## SUMMARY TABLE

| Paper | Consistency | Contract | Referee Verdict | Abstract (words) | P1 Blockers | Verdict |
|-------|-------------|----------|-----------------|------------------|-------------|---------|
| D.6 Prison Gerrymandering | 2 issues (900 vs 1,800 NC count; "1–3" vs "1–2" districts) | PARTIAL (NY→NC unexplained) | Minor Revision × 3 | 170 | 2 (count discrepancy; abstract range) | FIXES REQUIRED — 6 items |
| D.7 Section 203 | 1 critical failure (12% vs 35% TX headline in abstract) | PASS 9/9 | Accept/Minor Revision | 188 | 2 (abstract TX% wrong; literacy conjunct missing) | FIXES REQUIRED — 5 items |
| D.8 Post-Shelby | PASS | PASS 10/10 | Accept × 3 | 204 | 0 | READY FOR PANEL |

**Key legal citations verified correct in all three papers**:
- Evenwel v. Abbott: 578 U.S. 54 (2016) — PASS
- Shelby County v. Holder: 570 U.S. 529 (2013) — PASS
- Allen v. Milligan: 599 U.S. 1 (2023) — PASS (paper corrected spec error of "523 U.S. 1310")
- Brnovich v. DNC: 594 U.S. 647 (2021) — PASS (paper uses US Reports; spec used slip opinion)
- Abbott v. Perez: 585 U.S. 579 (2018) — PASS
- Thornburg v. Gingles: 478 U.S. 30 (1986) — PASS across all three papers

**Cross-paper note**: The spec for D.8 contained an incorrect citation for Allen v. Milligan ("523 U.S. 1310" — this volume number does not exist). The paper correctly uses 599 U.S. 1. No action needed in the papers; flag for spec correction if the spec is used as a reference template.
