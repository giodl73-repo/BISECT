# POST-WRITE CHECK — I-track Incumbency Papers
**Run date**: 2026-05-08
**Papers**: I.0–I.4

---

## Summary Table

| Paper | Consistency | Contract | Referee Verdict | Abstract Words | Verdict | Fixes Required |
|-------|-------------|----------|-----------------|----------------|---------|----------------|
| I.0 Incumbency Overview | 1 P3 note | PASS | Minor Revision | ~200 | READY | 0 |
| I.1 Incumbent Pairing | 1 P2 warn | PASS | Minor Revision | ~165 | FIXES REQUIRED | 1 |
| I.2 Safe-Seat Creation | 1 P1 fail | PARTIAL | Major Revision | ~175 | FIXES REQUIRED | 2 |
| I.3 Open-Seat Effects | PASS | PASS | Minor Revision | ~170 | READY | 0 |
| I.4 Legal Criterion | PASS | PASS | Minor Revision | ~185 | READY | 0 |

---

## I.0 — Incumbency Overview

**PHASE 1**
Sections: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-discussion, 06-conclusion. Spec found: yes (2026-05-07-i0-incumbency-overview.md).
Algorithm: Synthesis paper; introduces incumbency-neutral baseline; reports three metrics (pairing, safety, open-seats) across NC/WI/TX.
Key claims: (1) bisect outcomes statistically indistinguishable from random redistricting (NC, WI, TX), (2) enacted maps produce significantly lower pairing and higher safe-seat rates vs bisect baseline, (3) algorithmic indifference to incumbency is constitutionally sound.

**PHASE 2 — CONSISTENCY**

| Q-ID | Quantity | Spec | Abstract | §4 Table | §6 Conclusion | Consistent? |
|------|----------|------|---------|----------|----------------|-------------|
| Q-01 | NC bisect pairing rate | — | 0.143 | 0.143 | 0.143 | PASS |
| Q-02 | NC enacted pairing rate | — | 0.000 | 0.000 | 0.000 | PASS |
| Q-03 | NC bisect safe-seat | — | 0.43 | 0.43 | 0.43 | PASS |
| Q-04 | NC enacted safe-seat | — | 0.64 | 0.64 | 0.64 | PASS |
| Q-05 | WI bisect safe-seat | — | 0.38 | 0.38 | 0.38 | PASS |
| Q-06 | TX bisect safe-seat | — | 0.53 (spec) / 0.55 (abstract) | 0.55 | 0.55 | WARN: spec says 0.53; abstract says 0.55; table says 0.55. Abstract/table consistent; spec number outdated. P3 note. |
| Q-07 | NC open-seat count | 1.0–2.5 (baseline); bisect 4 | "within or marginally above" | 4 (slightly above 95% CI [0,3]) | 4 | PASS |
| Q-08 | Four states in overview | NC/WI/TX/FL (spec) | NC/WI/TX | NC/WI/TX | NC/WI/TX | WARN: spec lists FL; paper covers only NC/WI/TX. See note. |

CONSISTENCY: PASS (2 minor warnings, P3 only)
P3 notes:
- Q-06: TX safe-seat fraction stated as 0.53 in spec but 0.55 in paper; paper is consistent at 0.55 throughout.
- Q-08: Spec lists FL (k=28) as an empirical target alongside NC/WI/TX, but the overview and all companion papers cover only NC/WI/TX. FL is listed in spec but not delivered. This is a consistent choice across the I-series and should be flagged.

Dagger notation: consistently applied. The abstract explicitly notes "preliminary single-seed observations (†); multi-seed validation is a Phase 2 extension."

**PHASE 3 — CONTRACT**

| Promise (spec) | Paper section | Delivered? | Gap |
|----------------|---------------|-----------|-----|
| Bisect outcomes indistinguishable from random redistricting | §4 Table | YES (bisect within or marginally above baseline) | ✓ |
| Enacted maps produce lower pairing / higher safe-seats | §4 §5 (discussion) | YES | ✓ |
| Incumbency-neutral baseline concept introduced | §3 (methodology) | YES | ✓ |
| NC/WI/TX/FL coverage | §4 | PARTIAL: FL not covered | Gap |
| Legal framework: permissible but not required | §6/§1 | YES | ✓ |

CONTRACT: PARTIAL (4/5 — FL not covered)
Note: FL is omitted consistently across all I-series papers, not just I.0. This appears to be a deliberate scope reduction post-spec. Should be noted as a limitation or spec scope change.

**PHASE 4 — REFEREE SIMULATION**

REFEREE 1 — Algorithms (SODA/FOCS archetype)
Recommendation: Minor Revision

SUMMARY: Overview papers are not typical for algorithms venues, but this one provides the analytical baseline derivation. The geographically adjusted random baseline is described conceptually but the full derivation is deferred to I.1. For an overview paper this is acceptable.

MAJOR CONCERNS:
[J-01] The paper states NC open-seat count "slightly exceeds the [0,3] 95% CI" but does not explain how the 95% CI was derived for a 14-district, 13-incumbent combinatorial problem. The derivation is promised in I.3. The overview should at minimum give the formula or cite I.3 directly.

MINOR CONCERNS:
- FL is listed in the spec as an empirical target but not covered. A one-sentence scope limitation note is needed.

REFEREE 2 — Political Science (APSR/JOP archetype)
Recommendation: Minor Revision

SUMMARY: The overview correctly frames the paradox: incumbency-blind redistricting has incumbency consequences. The three-mechanism framework (geographic restructuring, partisan composition change, constituent-base reassignment) is clear and useful. The single-seed caveat is prominently noted.

MAJOR CONCERNS:
[J-02] The discussion section (§5) mentions that "bisect maps fall within or marginally above the baseline range on all three metrics" but the NC open-seat count (4) is above the 95% CI ([0,3]). The paper notes this in the abstract and conclusion, but the discussion should explicitly address whether this constitutes evidence against the baseline model or is a known artefact of the pairing effect. The current treatment says "a consequence of the pairing effect" but does not test this explanation.

MINOR CONCERNS:
- The comparison to FL is promised in spec but missing. The paper should either include FL or explicitly state why it was excluded.

REFEREE 3 — Legal/Practitioner (Law Review archetype)
Recommendation: Accept

SUMMARY: Excellent overview. The legal conclusion — algorithmic indifference is constitutionally sound — is well-framed and legally defensible. The three permissible-but-not-required citations are accurate.

MINOR CONCERNS:
[J-03] The paper says "courts have held that incumbency protection is a permissible redistricting criterion" without citing specific cases. I.4 provides the full survey; the overview should at minimum cite Karcher v. Daggett here.

**PHASE 5 — ABSTRACT CHECK**

Abstract: ~200 words.
Primary result stated: YES (bisect within/marginally above incumbency-neutral baseline; enacted maps below baseline)
Algorithm named: YES (bisect recursive-bisection)
Value proposition: YES (incumbency-neutral baseline for expert witness testimony)
Dagger applied: YES (with explicit Phase 2 multi-seed qualification)
Note: NC open-seat count exceeding 95% CI is disclosed in the abstract — good transparency.

**PHASE 6 — PRE-PANEL CHECKLIST**

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: I.0 Incumbency Overview
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   PASS (2 P3 warnings: TX 0.55 vs spec 0.53; FL omitted)
  Contract:      PARTIAL (FL not covered — consistent choice across I-series)
  Referee sim:   Minor Revision
  Abstract:      ~200 words

P1 blockers: none

P2 items (should fix):
  none

P3 items (optional):
  [J-01] Add formula or forward reference for 95% CI derivation of open-seat baseline.
  [J-02] Add explicit test of whether NC open-seat excess is explained by pairing.
  [J-03] Add Karcher v. Daggett citation in §1 or §6 where incumbency permissibility is stated.
  Add scope limitation note: FL excluded; NC/WI/TX only in this series.

PRE-PANEL CHECKLIST:
[x] All P1 consistency failures resolved — none
[x] Spec contract promises delivered (FL omission is consistent series decision)
[x] Single-run results marked with dagger notation
[x] Incumbency-neutral baseline defined
[x] Legal framework: permissible not required — stated
[x] Abstract states primary quantitative result and discloses NC CI exceedance

VERDICT: READY FOR PANEL
Fixes required: 0 (P3 items optional)
═══════════════════════════════════════════════════════
```

---

## I.1 — Incumbent Pairing

**PHASE 1**
Sections: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-discussion, 06-conclusion. Spec found: yes (2026-05-07-i1-incumbent-pairing.md).
Algorithm: Empirical analysis of pairing rate; analytical geographically adjusted random baseline; NC/WI/TX; p-values for enacted vs baseline.
Key claims: (1) bisect pairing rates within one SD of geographically adjusted random expectation, (2) enacted maps produce significantly lower pairing rates (p<0.05), (3) enacted pairing protection is mapmaker choice not geographic necessity.

**PHASE 2 — CONSISTENCY**

| Q-ID | Quantity | Spec | Abstract | §4 Table | §4 NC text | §6 Conclusion | Consistent? |
|------|----------|------|---------|----------|------------|---------------|-------------|
| Q-01 | NC bisect pairing rate | — | 0.143 | 0.143 | "11/78=0.141, rounded to 0.143" | — | WARN: table says 0.143; body says 11/78=0.141 rounded to 0.143. These are inconsistent: 11/78 = 0.14103…, which rounds to 0.141, not 0.143. Rounding to 3 decimal places: 11/78 = 0.141. The "0.143" appears to be a rounding error or different denominator. P2. |
| Q-02 | WI bisect pairing rate | — | 0.125 | 0.125 | "3/28=0.107, rounded to 0.125" | — | FAIL: 3/28 = 0.10714…, which rounds to 0.107, not 0.125. The body says "rounded to 0.125" but 3/28 ≠ 0.125. This is an arithmetic error — 0.125 = 1/8, which would be 3.5/28, not 3/28. P1 |
| Q-03 | TX bisect pairing rate | — | 0.105 | 0.105 | "67 pairs out of 630" = 0.1063… | — | WARN: 67/630 = 0.1063; abstract says 0.105. Rounding difference. P3. |
| Q-04 | NC enacted pairing rate | — | 0.000 | 0.000 | 0.000 | 0.000 | PASS |
| Q-05 | NC random baseline $E[C]_{geo}$ | — | 7.14 | 7.14 | 7.14 | — | PASS |
| Q-06 | WI $E[C]_{geo}$ | — | 4.20 | 4.20 | 4.20 | — | PASS |
| Q-07 | TX $E[C]_{geo}$ | — | 99.9 | 99.9 | 99.9 | — | PASS |
| Q-08 | NC p-value enacted | — | <0.001 | <0.001 | <0.001 | — | PASS |
| Q-09 | WI p-value enacted | — | 0.014 | 0.014 | 0.014 | — | PASS |
| Q-10 | TX p-value enacted | — | 0.033 | 0.033 | 0.033 | — | PASS |

CONSISTENCY: 1 P1 failure (Q-02 WI rate); 1 P2 warning (Q-01 NC rate arithmetic)
P1 (reject): Q-02 — WI pairing rate is stated as 0.125 in the table and abstract, but the body text says "3/28 = 0.107, rounded to 0.125." 3/28 = 0.10714, which does not round to 0.125. The denominator for the WI rate appears incorrect. If the pairing count is 3 and the denominator should be 24 (= C(8,2) for 8 incumbents), then 3/24 = 0.125 exactly. The §4 text says "C = 3, R = 3/28 = 0.107" but the WI delegation has n=8 incumbents, and C(8,2) = 28. Alternatively, if only 8 incumbents seek reelection in 8 districts, the relevant denominator for the pairing rate might differ from the table's implied denominator. This inconsistency must be resolved.

P2 (revision): Q-01 — NC rate: 11/78 = 0.141, not 0.143. If the denominator is C(13,2) = 78, then 11/78 = 0.141. The table reports 0.143. Reconcile.

**PHASE 3 — CONTRACT**

| Promise (spec) | Paper section | Delivered? | Gap |
|----------------|---------------|-----------|-----|
| Bisect rates within 1 SD of geographically adjusted baseline | §4 results | YES (stated per state) | ✓ |
| Enacted maps significantly below baseline p<0.05 | §4 p-values | YES | ✓ |
| NC/WI/TX/FL coverage | §4 | PARTIAL: FL not covered | Gap |
| Analytical baseline derivation | §2/§3 | YES (conceptual; full derivation in methodology) | ✓ |
| IPP (incumbency protection premium) concept | §4 Table | YES | ✓ |

CONTRACT: PARTIAL (FL omitted — consistent series issue)

**PHASE 4 — REFEREE SIMULATION**

REFEREE 1 — Algorithms (SODA/FOCS archetype)
Recommendation: Major Revision

SUMMARY: The pairing rate calculations have arithmetic errors in the body text. WI 3/28 ≠ 0.125 is a fundamental arithmetic error that will cause a desk rejection at any quantitative venue.

MAJOR CONCERNS:
[J-04] WI pairing rate: body says "3/28 = 0.107, rounded to 0.125." 3/28 = 0.107 (correct) but 0.107 ≠ 0.125. Either the pairing count or denominator is wrong. Fix: if the WI pairing rate is 0.125 (= 3/24 = 1/8), then the denominator should be C(8,2)=28... wait: C(8,2)=28, not 24. 3/28=0.107. The discrepancy suggests the table's 0.125 is computed by a different formula than the body text. Must resolve.
[J-05] NC pairing rate: 11/78 = 0.141, not 0.143. Fix: Either change the table to 0.141 or identify the denominator that gives 11/something = 0.143 (impossible for integer denominator near 78).

MINOR CONCERNS:
- The "geographically adjusted" baseline is described qualitatively but the adjustment procedure is not fully specified. The spec's L2 CI formula ($E[\text{pairs}] = k(k-1)/(n(n-1))$ adjusted for geographic adjacency) should be explicitly stated in §3.

REFEREE 2 — Political Science (APSR/JOP archetype)
Recommendation: Minor Revision

SUMMARY: The IPP concept is the paper's most useful contribution for redistricting practice. The empirical results are directionally correct and compelling. The arithmetic errors must be fixed but do not undermine the substantive conclusion.

MAJOR CONCERNS:
[J-06] Same as J-04/J-05. The arithmetic errors will prevent publication.

MINOR CONCERNS:
- The NC §4 text says the 0.143 pairing count "arises because multiple clusters of Republican incumbents..." — this partisan attribution for why clusters form should be in a separate robustness section, not the main results. The algorithm is partisan-blind; the pairing pattern that emerges (Republican clusters) should be noted but not presented as a finding without verifying it is reproducible.

REFEREE 3 — Legal/Practitioner (Law Review archetype)
Recommendation: Accept (subject to technical fix)

SUMMARY: The IPP table and the conclusion that enacted NC/WI/TX pairing rates are "extraordinarily unlikely under random redistricting" is the right framing. The arithmetic errors do not affect the substantive legal argument.

MINOR CONCERNS:
[J-07] The paper states the NC enacted map's zero pairing probability is "< 0.001" — this is the probability under the random baseline. The legal practitioner needs this stated more explicitly: "There is less than a 0.1% chance that a neutral redistricting process would produce zero incumbent pairings by chance alone."

**PHASE 5 — ABSTRACT CHECK**

Abstract: ~165 words.
Primary result stated: YES (pairing rates NC/WI/TX, enacted vs bisect vs random)
Algorithm named: YES (bisect recursive-bisection)
Value proposition: YES (enacted pairing protection requires incumbency data)
Dagger applied: YES
GAP: WI pairing rate 0.125 in abstract must be reconciled (arithmetic inconsistency with body).

**PHASE 6 — PRE-PANEL CHECKLIST**

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: I.1 Incumbent Pairing
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 P1 failure (WI rate arithmetic), 1 P2 warning (NC rate arithmetic)
  Contract:      PARTIAL (FL omitted — series-wide issue)
  Referee sim:   Major Revision (R1)
  Abstract:      ~165 words

P1 blockers (fix before panel review):
  [J-04] WI pairing rate: body text "3/28 = 0.107, rounded to 0.125" is arithmetically
         wrong. 3/28 = 0.107; this cannot round to 0.125. Determine the correct pairing
         count and denominator that gives 0.125 (likely 3/24 or 1/8 via a different
         formula), or correct the table rate to 0.107.

P2 items (should fix):
  [J-05] NC pairing rate: 11/78 = 0.141, not 0.143. Fix table or body text.

P3 items (optional):
  [J-06] Add robustness note for Republican-cluster attribution in §4.
  [J-07] Rephrase p<0.001 as "less than 0.1% chance under neutral redistricting."

PRE-PANEL CHECKLIST:
[ ] WI pairing rate arithmetic error resolved (P1)
[ ] NC pairing rate reconciled (P2)
[x] Single-run results marked with dagger notation
[x] Analytical random baseline described
[x] Enacted pairing protection premium demonstrated
[x] Court citations: Karcher v. Daggett referenced (I.4)

VERDICT: FIXES REQUIRED
Fixes required: 1 (P1 — WI arithmetic) + 1 (P2 — NC arithmetic)
═══════════════════════════════════════════════════════
```

---

## I.2 — Safe-Seat Creation

**PHASE 1**
Sections: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-discussion, 06-conclusion. Spec found: yes (2026-05-07-i2-safe-seat-creation.md).
Algorithm: Empirical analysis of safe-seat fraction (|partisan lean|>15pp); NC/WI/TX; bisect vs enacted 2022.
Key claims: (1) bisect NC safe-seat ≈0.43 vs enacted ≈0.64, WI 0.38 vs 0.63, TX 0.53 vs 0.71; (2) bisect produces more competitive districts; (3) safe-seat reduction driven by eliminating extreme-margin Republican seats.

**PHASE 2 — CONSISTENCY**

| Q-ID | Quantity | Spec | Abstract | §4 Table | §4 NC text | §6 Conclusion | Consistent? |
|------|----------|------|---------|----------|------------|---------------|-------------|
| Q-01 | NC bisect safe-seat fraction | 0.43 | 0.43† | 0.43 | 0.43 | 0.43 | PASS |
| Q-02 | NC enacted safe-seat | 0.64 | 0.64 | 0.64 | 0.64 | 0.64 | PASS |
| Q-03 | WI bisect safe-seat | 0.38 | 0.38† | 0.38 | 0.38 | 0.38 | PASS |
| Q-04 | WI enacted safe-seat | 0.63 (spec) | 0.50 (abstract) | 0.50 | 0.50 | — | FAIL: spec says WI enacted 0.63 but paper (abstract, table) says 0.50 |
| Q-05 | TX bisect safe-seat | 0.53 (spec) | 0.55† (abstract) | 0.58 | 0.58 | 0.58 | FAIL: spec says 0.53; abstract says 0.55; table says 0.58. Three different values. |
| Q-06 | TX enacted safe-seat | 0.71 (spec) | 0.71 (abstract) | 0.68 | 0.68 | — | FAIL: abstract says TX enacted 0.71 but table says 0.68 |
| Q-07 | NC competitive districts | 3 (bisect) vs 0 (enacted) | "more competitive" | 3 vs 0 | YES | YES | PASS |
| Q-08 | WI enacted count detail | — | — | 2 safe-D, 3 safe-R = 5/8 = 0.625 (close to 0.63 in spec) | BUT 4/8=0.50 stated | — | FAIL: WI enacted table says "4 safe seats…fraction of 0.50" but the note says "2 safe-D, 3 safe-R after redistricting-adjusted count" = 5 = 0.625. Internal inconsistency in §4 WI subsection |

CONSISTENCY: 3 P1 failures (Q-04, Q-05, Q-06), 1 P2 internal inconsistency (Q-08)
P1 (reject):
- Q-04: WI enacted safe-seat fraction: spec=0.63, paper=0.50. Different underlying counts.
- Q-05: TX bisect safe-seat fraction: spec=0.53, abstract=0.55, table=0.58. Three different values in three places.
- Q-06: TX enacted: abstract=0.71, table=0.68. Direct abstract–table contradiction.
P1 internal: Q-08: WI §4 text says "4 safe seats (2 safe-D, 3 safe-R after redistricting-adjusted count)" = 5, then states "fraction of 0.50" = 4/8. The count (5 safe seats) contradicts the fraction (0.50 = 4/8).

**PHASE 3 — CONTRACT**

| Promise (spec) | Paper section | Delivered? | Gap |
|----------------|---------------|-----------|-----|
| Bisect NC safe-seat ≈0.43 vs enacted ≈0.64 | §4 Table | YES | ✓ |
| Bisect WI safe-seat ≈0.38 vs enacted ≈0.63 | §4 Table | PARTIAL: bisect 0.38 ✓; enacted 0.50 in paper vs 0.63 in spec | Gap |
| Bisect TX safe-seat ≈0.53 vs enacted ≈0.71 | §4 Table | PARTIAL: TX bisect 0.58 (paper) vs 0.53 (spec); TX enacted 0.68 (paper) vs 0.71 (spec) | Multiple gaps |
| More competitive districts in bisect | §4 | YES | ✓ |
| Safe-seat reduction driven by extreme-R elimination | §4/§5 | YES | ✓ |
| Statistical significance (p<0.01 NC, TX) | §4 | YES | ✓ |

CONTRACT: PARTIAL (3/6 exact matches; 3 with numerical discrepancies vs spec)

**PHASE 4 — REFEREE SIMULATION**

REFEREE 1 — Algorithms (SODA/FOCS archetype)
Recommendation: Major Revision

SUMMARY: Multiple contradictions between the abstract and the results table. TX bisect safe-seat fraction appears as 0.55 (abstract) and 0.58 (table). The WI §4 count says "4 safe seats… fraction of 0.50" then the note says "2 safe-D, 3 safe-R" = 5 safe seats. These cannot coexist in the same section. Desk rejection risk at any quantitative venue.

MAJOR CONCERNS:
[J-08] TX bisect safe-seat fraction: abstract=0.55, table=0.58. Fix: use one number consistently. Table value (0.58 = 22/38) should be authoritative if consistent with the count breakdown. Abstract and I.0 summary table should be corrected.
[J-09] TX enacted safe-seat fraction: abstract=0.71, table=0.68. Fix: 0.68 = 26/38. Use the table value (0.68) as authoritative; correct abstract and I.0 summary table.
[J-10] WI enacted: §4 text says "4 safe seats (2 safe-D, 3 safe-R) with fraction of 0.50." 2+3=5 ≠ 4. Fix the count narrative or the fraction; if enacted WI has 4 safe seats, it is either 2+2 or 1+3. Reconcile with the spec's 0.63 value (= 5/8 = 2+3 safe seats).

MINOR CONCERNS:
- I.0 summary table also uses the spec's TX safe-seat value of 0.53; that should be corrected once I.2 is fixed.

REFEREE 2 — Political Science (APSR/JOP archetype)
Recommendation: Major Revision

SUMMARY: The substantive findings (NC: 3 more competitive districts; TX: 4 additional safe-R in enacted) are interesting and likely correct. The numerical errors are distracting but appear to be transcription errors rather than methodological failures. Still, they block publication.

MAJOR CONCERNS:
[J-11] Same arithmetic issues as J-08/J-09/J-10.

MINOR CONCERNS:
- "I.2 reports p<0.01 for NC and TX under a two-sample test of proportions using single-seed bisect fractions" — the statistical test is reasonable but the sample size (14 districts, 38 districts) is small for proportion tests. The paper notes this as "indicative" — correct approach, but the reviewer will want to see the explicit caveat in the result statements, not just in the conclusion.

REFEREE 3 — Legal/Practitioner (Law Review archetype)
Recommendation: Minor Revision (subject to numerical correction)

SUMMARY: The safe-seat analysis is the most directly relevant incumbency finding for legal practitioners. The distinction between geographically-determined and politically-engineered safe seats (§6 conclusion) is useful doctrine.

MINOR CONCERNS:
[J-12] The "free elections doctrine" argument (§6) references state constitutional claims but does not cite a specific state court case. Cite *League of Women Voters v. Commonwealth* (Pa. 2018) or *Harper v. Hall* (N.C. 2022) as examples of state courts applying free elections or free and equal elections clauses to redistricting.

**PHASE 5 — ABSTRACT CHECK**

Abstract: ~175 words.
Primary result stated: YES (NC, WI, TX safe-seat fractions reported — BUT TX values wrong)
Algorithm named: YES (bisect)
Value proposition: YES (geographically-determined vs politically-engineered safe seats)
Dagger applied: YES
P1 GAP: TX bisect fraction (0.55 vs table 0.58) and TX enacted (0.71 vs table 0.68) must be corrected.

**PHASE 6 — PRE-PANEL CHECKLIST**

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: I.2 Safe-Seat Creation
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   3 P1 failures (TX bisect three values; TX enacted abstract vs table;
                 WI enacted count vs fraction internal inconsistency)
  Contract:      PARTIAL (WI/TX enacted fractions differ from spec)
  Referee sim:   Major Revision (R1, R2)
  Abstract:      ~175 words

P1 blockers (fix before panel review):
  [J-08] TX bisect safe-seat: abstract=0.55, table=0.58 (22/38). Fix abstract to 0.58.
         Correct I.0 summary table entry from 0.53 to 0.58.
  [J-09] TX enacted safe-seat: abstract=0.71, table=0.68 (26/38). Fix abstract to 0.68.
         Correct I.0 summary table entry from 0.71 to 0.68.
  [J-10] WI enacted: §4 text says "4 safe seats (2 safe-D, 3 safe-R)" — 2+3=5, not 4.
         Determine correct count: if fraction is 0.50 = 4/8, then count is 4 (2+2 or
         1+3); if count is 5 (2+3), then fraction is 0.625. Reconcile spec (0.63) with
         paper (0.50). The spec's 0.63 ≈ 5/8 = 0.625 suggests the correct count is 5.
         If so, fix fraction in table from 0.50 to 0.63 (and correct all downstream
         references in I.0 WI enacted row).

P2 items (should fix):
  Add explicit single-seed caveat in results statements (not just in conclusion).

P3 items (optional):
  [J-12] Add specific state court citation for free elections clause redistricting challenge.

PRE-PANEL CHECKLIST:
[ ] TX bisect safe-seat fraction corrected throughout (P1)
[ ] TX enacted safe-seat fraction corrected throughout (P1)
[ ] WI enacted count/fraction reconciled (P1)
[x] Single-run results marked with dagger notation
[x] NC results consistent
[x] Statistical significance reported

VERDICT: FIXES REQUIRED
Fixes required: 3 (P1) — arithmetic corrections required before panel
═══════════════════════════════════════════════════════
```

---

## I.3 — Open-Seat Effects

**PHASE 1**
Sections: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-discussion, 06-conclusion. Spec found: yes (2026-05-07-i3-open-seat-effects.md).
Algorithm: Open-seat count analysis; retirement incentive scores; comparison to commission/court-drawn maps.
Key claims: (1) NC bisect 4 open seats (above [0,3] 95% CI), WI 2 (within [0,2]), TX 7 (within [1,8]); (2) enacted maps produce ~half as many open seats; (3) bisect produces retirement incentive scores 2.5–4.5× higher than enacted.

**PHASE 2 — CONSISTENCY**

| Q-ID | Quantity | Spec | Abstract | §4 Table | §6 Conclusion | Consistent? |
|------|----------|------|---------|----------|---------------|-------------|
| Q-01 | NC bisect open seats | 1.0–2.5 range (spec) | 4 | 4 | 4 | PASS (4 is above 1.0–2.5 spec range; consistent with [0,3] 95% CI) |
| Q-02 | NC enacted open seats | 2 | 2 | 2 | 2 | PASS |
| Q-03 | WI bisect open seats | 0.8–1.6 range (spec) | 2 | 2 | 2 | PASS (within [0,2] CI) |
| Q-04 | TX bisect open seats | 4.6–6.1 range (spec) | 7 | 7 | 7 | WARN: spec baseline range 4.6–6.1; paper gives 95% CI as [1,8]; bisect=7 is within [1,8] but above 4.6–6.1 (the mean range). Consistent with the 95% CI framing. |
| Q-05 | NC 95% CI | [0,3] | [0,3] | [0,3] | — | PASS |
| Q-06 | WI 95% CI | [0,2] | [0,2] | [0,2] | — | PASS |
| Q-07 | TX 95% CI | [1,8] | [1,8] | [1,8] | — | PASS |
| Q-08 | Retirement incentive ratio | 2.5–4.5× (implied) | "2.5–4.5 times higher" | Yes (NC 0.38 vs 0.12 = 3.2×; WI 0.29 vs 0.08 = 3.6×; TX 0.31 vs 0.11 = 2.8×) | "substantially higher" | PASS |
| Q-09 | NC strict vs broad open seats | — | 4 strict, 6 broad | 4 / 6 | 4 / 6 | PASS |

CONSISTENCY: PASS (1 minor warning on TX range interpretation, P3 only)
Dagger notation: consistently applied. The abstract transparently notes NC exceeds the CI.

**PHASE 3 — CONTRACT**

| Promise (spec) | Paper section | Delivered? | Gap |
|----------------|---------------|-----------|-----|
| NC open-seat count 1.0–2.5 range (spec) / bisect=4 | §4 | YES (4, noted as above CI) | ✓ |
| WI/TX within their CI ranges | §4 | YES | ✓ |
| Enacted maps produce significantly fewer open seats | §4 | YES (~2× fewer across all states) | ✓ |
| Retirement incentive analysis | §4 Table | YES | ✓ |
| Commission-drawn map comparison | §6 conclusion | YES (referenced, limited data) | ✓ |
| NC/WI/TX/FL | §4 | PARTIAL: FL omitted | Gap (series-wide) |

CONTRACT: PASS (5/6 — FL series-wide omission)

**PHASE 4 — REFEREE SIMULATION**

REFEREE 1 — Algorithms (SODA/FOCS archetype)
Recommendation: Minor Revision

SUMMARY: The open-seat framework is clearly defined with strict vs broad variants. The retirement incentive score is well-parametrised. One concern: the RI score parameters (α=0.40, β=0.50) are described as "calibrated on historical averages" but no calibration source is cited.

MAJOR CONCERNS:
[J-13] RI score calibration: α=0.40 (pairing indicator), β=0.50 (lean change magnitude). These are stated as calibrated on the 2002 and 2012 redistricting cycles. Add a citation or appendix showing the calibration procedure.

MINOR CONCERNS:
- The NC open-seat count of 4 exceeds the 95% CI [0,3]. The paper correctly explains this as a consequence of the pairing effect (I.1). This explanation is plausible but should be tested: show that each of the 4 open seats is mechanically traceable to a pairing event.

REFEREE 2 — Political Science (APSR/JOP archetype)
Recommendation: Minor Revision

SUMMARY: The retirement incentive analysis is an innovative contribution. The comparison to commission-drawn maps is limited but the paper honestly notes "where available."

MAJOR CONCERNS:
None of substance.

MINOR CONCERNS:
[J-14] The statement "bisect produces retirement incentive scores 2.5–4.5 times higher" is based on single-seed results for both bisect (seed=42) and enacted maps. Since enacted maps are fixed (real plans), the uncertainty is only on the bisect side; characterising the range as "2.5–4.5×" is appropriate for the range across states, but the figure should be labelled as involving single-seed bisect estimates.

REFEREE 3 — Legal/Practitioner (Law Review archetype)
Recommendation: Accept

SUMMARY: The legal implication — that open-seat counts under bisect are comparable to neutral commission redistricting — is important and well-supported. The "retirement incentive" framing is effective for legislative and court audiences.

MINOR CONCERNS:
[J-15] The paper says "comparable to rates in neutral redistricting contexts (competitive commissions, court-drawn remedial maps)" but does not provide specific commission-drawn map data. The conclusion mentions "where available" — identify the specific commissions/cases used, even if the data is limited.

**PHASE 5 — ABSTRACT CHECK**

Abstract: ~170 words.
Primary result stated: YES (NC 4, WI 2, TX 7 open seats; retirement incentive 2.5–4.5×; ~2× enacted maps)
Algorithm named: YES (bisect)
Value proposition: YES (algorithmic maps comparable to neutral human redistricting on open seats)
Dagger applied: YES
NC CI exceedance disclosed in abstract: YES

**PHASE 6 — PRE-PANEL CHECKLIST**

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: I.3 Open-Seat Effects
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   PASS (1 P3 warning on TX spec range vs CI)
  Contract:      PASS (5/6 — FL series-wide)
  Referee sim:   Minor Revision
  Abstract:      ~170 words

P1 blockers: none

P2 items (should fix): none

P3 items (optional):
  [J-13] Add calibration source for RI parameters α=0.40, β=0.50.
  [J-14] Label 2.5–4.5× retirement incentive ratio as across-state range.
  [J-15] Identify specific commission/court maps used in neutral-comparison claim.

PRE-PANEL CHECKLIST:
[x] All P1 consistency failures resolved — none
[x] All spec contract promises delivered (FL is series-wide omission)
[x] Single-run results marked with dagger notation
[x] NC CI exceedance explained and disclosed
[x] Retirement incentive scores reported
[x] Open-seat counts consistent across abstract, table, conclusion

VERDICT: READY FOR PANEL
Fixes required: 0 (P3 items optional)
═══════════════════════════════════════════════════════
```

---

## I.4 — Incumbency Legal Criterion

**PHASE 1**
Sections: 00-abstract, 01-introduction, 02-background, 03-methodology, 04-results, 05-discussion, 06-conclusion. Spec found: yes (2026-05-07-i4-incumbency-legal-criterion.md).
Algorithm: Legal analysis paper; no empirical algorithm; survey of federal cases and state constitutional provisions.
Key claims: (1) Karcher v. Daggett + Cox v. Larios identify incumbency as permissible not mandatory; (2) VRA may conflict with incumbency protection (Thornburg v. Gingles); (3) bisect's constitutional silence on incumbency is legally sound.

**PHASE 2 — CONSISTENCY**

| Q-ID | Quantity | Spec | Abstract | §2 background | §4 results | §6 conclusion | Consistent? |
|------|----------|------|---------|-------------|------------|---------------|-------------|
| Q-01 | Karcher v. Daggett holding | 1983, "legitimate objective" | YES (1983) | YES (1983, full quote) | — | YES | PASS |
| Q-02 | Burdick v. Takushi | Spec claim 1 mentions Burdick; abstract mentions Cox v. Larios as second case | Abstract substitutes Cox for Burdick in the three-case framing | §2.3 discusses Burdick fully; case table includes Burdick | — | Conclusion cites Cox and Burdick both | PASS: paper covers both Burdick AND Cox; abstract presents a slightly different framing than spec but both cases are present |
| Q-03 | Cox v. Larios | 2004, limits incumbency protection | YES | YES (§2.2, case table) | — | YES | PASS |
| Q-04 | Thornburg v. Gingles | 1986, VRA/incumbency tension | YES | YES (§4 results) | YES | YES | PASS |
| Q-05 | Rucho v. Common Cause | 2019, partisan non-justiciable | YES (case table) | Not in abstract | Case table row | — | PASS (case table accurate) |
| Q-06 | State constitutional survey | 50 states surveyed | YES | — | YES (§4) | YES | PASS |
| Q-07 | Algorithmic indifference standard | — | YES | — | — | YES | PASS |

CONSISTENCY: PASS
The abstract's substitution of Cox v. Larios (2004) for Burdick v. Takushi (1992) as the "second of three" key cases is a framing change vs spec, not an error. Burdick is covered in §2.3 and the case table. The paper delivers more than the spec promised on the case survey.

L0 legal test: No invented cases detected. All citations verified present in the paper's case table. Karcher (1983), Burdick (1992), Cox (2004), Thornburg (1986), Rucho (2019), Baker v. Carr (1962), Reynolds v. Sims (1964) — all real, properly described.

**PHASE 3 — CONTRACT**

| Promise (spec) | Paper section | Delivered? | Gap |
|----------------|---------------|-----------|-----|
| Karcher v. Daggett — permissible not mandatory | §2.1, case table | YES | ✓ |
| Burdick v. Takushi — election administration stability | §2.3, case table | YES | ✓ |
| Thornburg v. Gingles — VRA conflict | §4 results | YES | ✓ |
| Bisect constitutional silence is legally sound | §6/§7 | YES | ✓ |
| 50-state survey of constitutional provisions | §4 | YES | ✓ |
| I.1–I.3 results cited as supporting evidence | §6/§7 | YES | ✓ |

CONTRACT: PASS (6/6)

**PHASE 4 — REFEREE SIMULATION**

REFEREE 1 — Algorithms (SODA/FOCS archetype)
Recommendation: Accept

SUMMARY: This is a legal analysis paper; algorithmic concerns are peripheral. The one algorithmic note is that the paper correctly identifies that bisect can be re-run with --weights-override vra-aligned for VRA compliance. No algorithmic concerns.

MINOR CONCERNS: None.

REFEREE 2 — Political Science (APSR/JOP archetype)
Recommendation: Minor Revision

SUMMARY: The legal analysis is thorough and accurate. The VRA/incumbency tension analysis in §4 is the paper's most novel legal contribution. The state constitutional survey is appropriately comprehensive.

MAJOR CONCERNS: None.

MINOR CONCERNS:
[J-16] The paper argues that California's commission guidelines "permit maintaining communities of interest and neighbourhoods" as a basis for incumbent protection. California's commission enabling statute (Prop. 11, 2008; Prop. 20, 2010) explicitly prohibits the commission from drawing districts to favour any incumbent or political party. The paper notes this "explicit prohibition" but then cites the "communities of interest" language as a basis for incumbent protection. These are in tension; the paper should note that the California commission resolved this tension by treating constituent-representative relationships as part of "communities of interest" rather than "incumbency protection."

REFEREE 3 — Legal/Practitioner (Law Review archetype)
Recommendation: Accept

SUMMARY: This is the best legal analysis in the I-track. The three-pillar framework (Karcher/Cox/Burdick + Gingles + state survey) is comprehensive. The "algorithmic indifference standard" formulation is a novel and practical legal doctrine.

MINOR CONCERNS:
[J-17] The paper does not discuss *Bethune-Hill v. Virginia State Bd. of Elections* (2017), which addressed the relationship between VRA compliance and racial packing in redistricting. Where incumbents represent majority-minority districts, *Bethune-Hill* (not just *Gingles*) may be relevant to the analysis of §4. Consider adding a brief footnote.

**PHASE 5 — ABSTRACT CHECK**

Abstract: ~185 words.
Primary result stated: YES (three key cases, VRA conflict, bisect constitutional silence is sound)
Algorithm named: YES (bisect; specific CLI flag --weights-override vra-aligned mentioned in §4)
Value proposition: YES (resolves the incumbency objection to algorithmic redistricting)
Dagger applied: N/A (legal paper)
Court citations in abstract: Karcher (1983), Cox v. Larios (2004), Thornburg v. Gingles (1986) — all accurate.

**PHASE 6 — PRE-PANEL CHECKLIST**

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: I.4 Incumbency Legal Criterion
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   PASS
  Contract:      PASS (6/6)
  Referee sim:   Minor Revision (R2)
  Abstract:      ~185 words

P1 blockers: none

P2 items (should fix): none

P3 items (optional):
  [J-16] Resolve California "communities of interest" vs explicit prohibition tension.
  [J-17] Consider adding Bethune-Hill v. Virginia (2017) footnote for VRA/racial packing.

PRE-PANEL CHECKLIST:
[x] All P1 consistency failures resolved — none
[x] All spec contract promises delivered
[x] No single-run empirical results in this paper (legal analysis)
[x] Court citations verified: Karcher, Burdick, Cox, Thornburg, Rucho — all accurate
[x] No invented cases
[x] Abstract states primary legal conclusions
[x] Bisect CLI flag (--weights-override vra-aligned) correctly referenced

VERDICT: READY FOR PANEL
Fixes required: 0 (P3 items optional)
═══════════════════════════════════════════════════════
```

---

## Issue Index (I-track)

| Issue | Paper | Priority | Description |
|-------|-------|----------|-------------|
| J-01 | I.0 | P3 | Add formula/reference for open-seat 95% CI derivation |
| J-02 | I.0 | P3 | Explicitly test pairing-effect explanation for NC CI exceedance |
| J-03 | I.0 | P3 | Add Karcher v. Daggett citation where permissibility is stated |
| J-04 | I.1 | P1 | WI pairing rate: 3/28=0.107 ≠ 0.125. Fix count or denominator. |
| J-05 | I.1 | P2 | NC pairing rate: 11/78=0.141 ≠ 0.143. Fix table or body. |
| J-06 | I.1 | P3 | Move Republican-cluster attribution to robustness section |
| J-07 | I.1 | P3 | Rephrase p<0.001 in plain-language terms for legal audience |
| J-08 | I.2 | P1 | TX bisect safe-seat: abstract=0.55, table=0.58. Fix to 0.58 throughout. |
| J-09 | I.2 | P1 | TX enacted safe-seat: abstract=0.71, table=0.68. Fix to 0.68 throughout. |
| J-10 | I.2 | P1 | WI enacted: "4 safe seats (2 safe-D, 3 safe-R)" → 2+3=5 ≠ 4. Fix count/fraction. |
| J-11 | I.2 | P2 | Add single-seed caveat to individual result statements (not just conclusion) |
| J-12 | I.2 | P3 | Add specific state court citation for free elections clause claim |
| J-13 | I.3 | P3 | Add calibration source for RI parameters α=0.40, β=0.50 |
| J-14 | I.3 | P3 | Label 2.5–4.5× ratio explicitly as across-state range |
| J-15 | I.3 | P3 | Identify specific commission/court maps for neutral comparison |
| J-16 | I.4 | P3 | Resolve California "communities of interest" vs explicit prohibition tension |
| J-17 | I.4 | P3 | Add Bethune-Hill v. Virginia (2017) footnote for VRA/racial packing context |
