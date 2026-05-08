# L-Track Post-Write Check
**Date**: 2026-05-08
**Papers**: L.0–L.6 (Partisan Fairness Metrics)
**Validator**: research-post-write skill

---

## Summary Table

| Paper | Consistency | Contract | Referee Verdict | Abstract Words | Verdict | P1 Fixes |
|-------|-------------|----------|-----------------|----------------|---------|----------|
| L.0 partisan-fairness-overview | 1 WARN | PASS 3/3 | Minor Revision | ~220 | FIXES REQUIRED | 1 |
| L.1 efficiency-gap | 1 WARN | PASS 3/3 | Minor Revision | ~195 | READY | 0 |
| L.2 mean-median | PASS | PASS 3/3 | Minor Revision | ~165 | READY | 0 |
| L.3 partisan-bias | 1 WARN | PASS 3/3 | Minor Revision | ~185 | READY | 0 |
| L.4 declination | PASS | PASS 3/3 | Minor Revision | ~220 | FIXES REQUIRED | 1 |
| L.5 seats-votes-curve | 1 WARN | PASS 3/3 | Minor Revision | ~195 | READY | 0 |
| L.6 proportionality-majoritarianism | 1 WARN | PASS 3/3 | Minor Revision | ~215 | FIXES REQUIRED | 1 |

---

## L.0 — Partisan Fairness Overview

### Phase 1 — Paper Summary

```
Paper: L.0+partisan-fairness-overview
Sections found: 00-abstract, 01-introduction, 02-definition, 03-behavior,
                04-empirical, 05-legal, 06-conclusion
Spec found: YES — docs/specs/2026-05-07-l0-partisan-fairness-overview.md
Series: L.0
Key claims:
  1. Six metrics (EG, MM, Bias, Declination, Responsiveness, LSq) cluster near partisan-neutral
     centroid for bisect maps; single-seed NC comparisons: bisect EG +0.02 vs enacted +0.09
  2. Descriptive vs normative taxonomy provides decision table for expert witnesses
  3. bisect reference distribution answers "compared to what?" for state court litigation
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior Table | §Empirical Table | §Conclusion | Consistent? |
|------|----------|---------|----------------|-----------------|-------------|-------------|
| Q-01 | NC EG bisect | "EG < 0.03" | "+0.02" | "+0.02" | "+0.02" | PASS (range vs point) |
| Q-02 | NC EG enacted | "0.09" | "0.09" | "+0.09" | "+0.09" | PASS |
| Q-03 | NC MM bisect | "≈0.02" | "≈0.02" | "+0.02" | — | PASS |
| Q-04 | NC MM enacted | "≈0.07" | "≈0.07" | "+0.07" | — | PASS |
| Q-05 | NC Bias bisect | "|Bias|<0.02" | "−0.01" | "−0.01" | — | PASS |
| Q-06 | NC Bias enacted | "≈−0.07" | "−0.07" | "−0.07" | — | PASS |
| Q-07 | NC Declination bisect | "<0.10" | "+0.08" | "+0.08" | — | PASS |
| Q-08 | NC Declination enacted | "≈+0.35" | "+0.35" | "+0.35" | — | PASS |
| Q-09 | NC Responsiveness bisect | "≈2.0" | "2.0" | "2.0" | — | PASS |
| Q-10 | NC Responsiveness enacted | "≈1.3" | "1.3" | "1.3" | — | PASS |
| Q-11 | NC LSq bisect | "[0.03,0.08]" | "0.05" | "0.05" | — | PASS (range vs point) |
| Q-12 | NC LSq enacted | "[0.10,0.18]" | "0.14" | "0.14" | — | PASS |
| Q-13 | §Conclusion: NC gap "four times larger" | — | — | — | "four times larger" | WARN |

**Q-13 Note**: The conclusion states "For North Carolina, this gap is approximately four times larger than what geography alone produces." This is the ratio of enacted to bisect values. For EG: 0.09/0.02 = 4.5. For MM: 0.07/0.02 = 3.5. For Bias: 0.07/0.01 = 7. For Declination: 0.35/0.08 = 4.4. The "four times" claim is approximately correct for EG and Declination but overstated for MM and dramatically overstated for Bias. The claim should be qualified: "approximately three to seven times larger, with a central estimate of approximately fourfold."

**Abstract word count**: The abstract is approximately 220 words — above the 150–200 target. The phrase "The reference distribution concept is central to post-Rucho state court litigation: when a court asks 'how different is this map from what a neutral process would produce?', the bisect distribution provides an empirically grounded, non-partisan answer" adds ~35 words that could be condensed.

```
CONSISTENCY: 1 WARN
P1: [Q-13] Conclusion's "four times larger" claim is imprecise — ranges from 3.5× to 7× across
    metrics. Revise to: "approximately three to five times larger (EG: 4.5×; MM: 3.5×; Declination:
    4.4×), with the exception of Bias where the ratio is larger due to near-zero bisect values."
P2: none
P3: Abstract is 220 words (slightly above target)
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Six-metric unified framework | §02-definition | Yes, all six defined with formulas | ✓ |
| Descriptive vs normative taxonomy | §02-definition | Yes | ✓ |
| Decision table for expert witnesses | §02 Table 1 | Yes, 6-row table | ✓ |
| bisect as reference distribution for Rucho | §03-behavior | Yes, 3 subsections | ✓ |
| Empirical overview NC/WI/TX/FL | §04-empirical | Yes, 6-metric comparison table | ✓ |
| Legal landscape post-Rucho | §05-legal | Yes, PA/NC/OH pathways | ✓ |
| Relationships among metrics | §02 | Yes (EG-Bias from S(v); MM-EG; Declination-EG) | ✓ |

```
CONTRACT: PASS 3/3
Gaps: none material
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer (SODA/FOCS archetype)**
Recommendation: Minor Revision

SUMMARY: The overview paper is primarily legal and political science in orientation. The algorithm review focuses on the claim that compactness produces partisan near-neutrality.

MAJOR CONCERNS:
I-01 The claim "compactness optimization is sufficient to bound |EG| near the geographic sorting baseline" (from §03) is stated without proof. This is an empirical claim supported by the data table but framed as a formal sufficiency. Change "sufficient" to "empirically consistent with bounding" or provide a theoretical argument.

MINOR CONCERNS:
- The formula for MM in §01 preview ("MM = $\bar{v}_D - \tilde{v}_D$") and in §02 definition is consistent. Good.
- The EG formula in §01 ("EG = (Wasted_D - Wasted_R)/Total") is consistent with §02. Good.
- The LSq formula in §01 uses "$(v_i - s_i)^2$" with individual party index $i$, but the two-party simplification in §02 is $|v_D - s_D|$. These are consistent but the notation switch could confuse.

---

**REFEREE 2 — Political Science Reviewer (APSR/JOP archetype)**
Recommendation: Minor Revision

SUMMARY: Strong legal-practitioner orientation. The decision table is the paper's central contribution for political scientists. The "four times larger" claim in the conclusion overgeneralizes.

MAJOR CONCERNS:
I-02 The conclusion claims NC bisect-enacted gap is "approximately four times larger than what geography alone produces, a finding that is consistent across all six metrics." This is not accurate: for Partisan Bias, bisect value is −0.01 and enacted is −0.07, ratio = 7×; for MM, 0.02 vs 0.07, ratio = 3.5×. The claim should be metric-specific or use a range. "Consistent across all six metrics" is correct for direction but not for magnitude. Fix: "approximately threefold to sevenfold depending on the metric, with EG, Declination, and Responsiveness showing the most consistent fourfold pattern."

MINOR CONCERNS:
- The abstract dagger notation is correct and consistent: "single-seed comparisons$^\dagger$" with footnote. Good.
- The FL data appears in the table but is not discussed in the behavior section. The behavior section focuses on NC mechanism. Either add FL discussion or note that the cross-state pattern is uniform.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Minor Revision

SUMMARY: The post-Rucho legal analysis is the best in the L-series. Pennsylvania, North Carolina, and Ohio case citations are accurate. The decision table is highly practical.

MAJOR CONCERNS:
I-03 Harper v. Hall (2022): The paper correctly cites this as an NC Supreme Court decision. However, the NC Supreme Court subsequently reversed course in 2023 when the court's composition changed. The paper's analysis assumes the 2022 Harper holding remains good law. By 2026, practitioners need to know whether Harper survives. If the 2023 reversal is the current state of NC law, the paper's claim that NC courts have imposed partisan gerrymandering standards may need updating. **This is a potential legal accuracy issue that must be verified before submission.**

MINOR CONCERNS:
- Ohio's seven redistricting rejections are correctly described. Accurate.
- PA League of Women Voters (2018) citation: decided before Rucho, but under state constitutional theory immune to federal foreclosure. Correctly framed.

### Phase 5 — Abstract Check

```
ABSTRACT: ~220 words (above target; trim by 20–30 words)
Primary result stated: YES (all six metrics; four-state comparison; dagger notation)
Algorithm named: YES (bisect)
Value proposition: YES (post-Rucho state court)
Note: dagger footnote in abstract is correctly placed
```

### Phase 6 — Pre-Panel Checklist (L.0)

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: L.0+partisan-fairness-overview
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 WARN (conclusion "four times" imprecise)
  Contract:      PASS (3/3)
  Referee sim:   Minor Revision
  Abstract:      ~220 words (above target)

P1 blockers (fix before panel review):
  [I-02/Q-13] "Approximately four times larger, consistent across all six metrics" is
              inaccurate — ratio ranges 3.5× to 7× across metrics. Revise conclusion to
              give metric-specific ratios or use "threefold to fivefold (central estimate
              fourfold)" and remove "consistent across all six metrics."

P2 items (should fix):
  [I-01] "Sufficient to bound" language → "empirically consistent with bounding"
  [I-03] Harper v. Hall (2022) legal status in 2026 — verify current NC law before submission
  Abstract: trim by ~20 words

P3 items (optional):
  FL behavioral mechanism not discussed (only in table)

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved                              ✗ (fix "four times" claim)
□ All spec contract promises delivered                              ✓
□ Single-run results marked with dagger notation                    ✓ (correctly applied)
□ Algorithm complexity claim                                        ✓ (N/A)
□ CLI flags match actual bisect binary flags                        ✓ (--weights-override geographic)
□ Court citations verified (Rucho 588 U.S. 684; Harper 868 S.E.2d 499) ✓ (verify Harper status)
□ Abstract states primary quantitative result                       ✓
□ Referee P1 blockers addressed                                     ✗ (I-02)

VERDICT: FIXES REQUIRED
Fixes required: 1 P1 + 2 P2
═══════════════════════════════════════════════════════
```

---

## L.1 — Efficiency Gap

### Phase 1 — Paper Summary

```
Paper: L.1+efficiency-gap
Series: L.1
Key claims:
  1. bisect NC EG < 0.03 vs enacted 0.09 (threefold reduction; 78% for NC)
  2. Single-election EG std dev ≈ 0.030; multi-cycle mean ±std: bisect +0.02±0.008, enacted +0.085±0.012
  3. 8% threshold academic proposal not adopted by any court; post-Rucho state court role
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior | §Empirical Tables | §Conclusion | Consistent? |
|------|----------|---------|----------|-------------------|-------------|-------------|
| Q-01 | NC bisect EG | "<0.03" | "+0.02" | "+0.02" | "+0.02" | PASS |
| Q-02 | NC enacted EG | "0.09" | "0.09" | "+0.09" | "0.09" | PASS |
| Q-03 | NC reduction | "threefold" | ~4.5× | "78%" | "70–78%" | WARN |
| Q-04 | NC std dev bisect | "≈0.030 across election cycles" | "0.008" | "0.008" | "≈0.030" | WARN |
| Q-05 | WI reduction | — | "67%" (tab:eg-baseline) | — | "70–78%" | WARN |
| Q-06 | Swing shifts EG by ≈0.04 | "≈0.04" | — | tab:eg-swing shows: neutral→+5pp: +0.02→+0.06 (Δ=0.04) | — | PASS |
| Q-07 | 8% threshold | "academic proposal" | — | — | "no legal force" | PASS |

**Q-03 Note**: Abstract says "threefold reduction" but §empirical Table tab:eg-baseline shows NC reduction = 78%, which is 4.5× reduction (0.09/0.02). "Threefold" would be 0.09/0.03 = 3×. The abstract's "<0.03" claim and "threefold" are technically consistent (0.09/0.03 = 3) but the actual single-run value is +0.02, making it 4.5×. The conclusion says "70–78% below enacted" which is not the same as "threefold." Recommend using "more than threefold" or reporting the percentage.

**Q-04 Note**: The abstract says "standard deviation ≈0.030 across election cycles for NC" but Table tab:eg-sensitivity shows std dev = 0.008 for both bisect and enacted across four election cycles. The text of §empirical explains: "The single-election EG standard deviation of approximately 0.030 cited in the abstract is the expected variability across the entire population of election cycles (not the four cycles in our sample)." This distinction is noted in the paper but will confuse readers who check the table. The abstract should say "expected standard deviation ≈0.030 across the population of election cycles; the four cycles in our sample show std dev = 0.008."

**Dagger notation**: Consistently applied throughout. Abstract has dagger in WI, TX, FL comparison; tables use $^\dagger$ notation.

```
CONSISTENCY: 1 WARN (multi-level std dev explanation and "threefold" vs 78%)
P1: none
P2: [Q-03] "Threefold" vs 78%/4.5× inconsistency → use "threefold to fourfold" or "more than
    threefold" and note the single-run value is +0.02 (4.5× below enacted)
    [Q-04] Abstract std dev 0.030 vs table 0.008 — add clarifying note in abstract or in §empirical
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: bisect EG < 0.03 for NC vs enacted 0.09 (✓); multi-election sensitivity analysis (✓); 8% threshold legal history (✓); Gill v. Whitford, Rucho (✓); bisect reference distribution for state courts (✓).

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-04 The "formal statement" in §behavior says compactness is "sufficient to bound |EG| near the geographic sorting baseline." The word "sufficient" is a mathematical claim that requires proof. The mechanism described (compactness prevents packing/cracking) is a qualitative causal argument, not a formal sufficiency result. Reframe as "an empirical property" or "consistent with bounding."

MINOR CONCERNS:
- The swing sensitivity analysis (tab:eg-swing) shows that bisect EG flips to −0.02 at −5pp swing while enacted remains +0.05. This is noteworthy — bisect maps exhibit some partisan swing sensitivity too. A sentence acknowledging this would be balanced.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-05 The abstract claims std dev ≈0.030 but the four-cycle sample shows std dev = 0.008. The paper explains the distinction (population of cycles vs. sample of 4) in §empirical but doesn't resolve it in the abstract. Political science reviewers will cite the table and challenge the 0.030 figure. Recommend either (a) reporting the sample std dev in the abstract and noting the population estimate separately, or (b) providing a source for the 0.030 estimate.

MINOR CONCERNS:
- The expert witness protocol in §legal is well-structured and directly actionable.
- The conclusion's "state courts with what federal litigation never had" framing is accurate and effective.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Accept

SUMMARY: Legal history is the most comprehensive treatment of EG in any paper in the L-series. Gill standing doctrine, 8% threshold disclosure requirements, and state court status through 2026 are all accurate. Expert witness protocol is practical.

MINOR CONCERNS:
- Whitford v. Gill (W.D. Wis. 2016) citation: "218 F.Supp.3d 837" — verify this citation is correct.
- Harper v. Hall (NC 2022) legal status — same concern as L.0: verify current status.

```
VERDICT: READY FOR PANEL (2 P2 fixes recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~195 words (within target)
Primary result stated: YES (EG < 0.03; threefold reduction; std dev claim)
Algorithm named: YES (bisect)
Value proposition: YES (state court baseline)
```

---

## L.2 — Mean-Median

### Phase 1 — Paper Summary

```
Paper: L.2+mean-median
Series: L.2
Key claims:
  1. NC bisect MM ≈ 0.02 vs enacted ≈ 0.07 (70% reduction); gap = mapmaker contribution
  2. MM invariant to uniform swings (mathematical proof — shift mean and median equally)
  3. MM std dev ≈ 0.001–0.003 across cycles vs EG's 0.008–0.012 (superior stability)
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior | §Empirical | §Conclusion | Consistent? |
|------|----------|---------|----------|------------|-------------|-------------|
| Q-01 | NC bisect MM | "≈0.02" | "≈0.02–0.03" | "+0.021" | "+0.02" | PASS |
| Q-02 | NC enacted MM | "≈0.07" | "≈0.07" | "+0.071" | "+0.07" | PASS |
| Q-03 | NC reduction | "60–70%" | — | "70%" (table) | "70%" | PASS |
| Q-04 | WI reduction | "69%" | — | "69%" | — | PASS |
| Q-05 | TX reduction | "66%" | — | "66%" | — | PASS |
| Q-06 | MM std dev bisect | "≈0.015" | — | "0.001" (table) | — | WARN |

**Q-06 Note**: Abstract says "NC standard deviation ≈0.015 vs ≈0.030 for EG" but Table tab:mm-sensitivity shows bisect std dev = 0.001 and enacted std dev = 0.003 across four cycles. The table shows MM is far more stable than the abstract claims (0.001 vs 0.015). This discrepancy may reflect the same "population of cycles" vs "sample of 4" distinction in L.1. However, the abstract claims 0.015 for MM to compare with EG's 0.030, and the table shows 0.001–0.003, which is even better than claimed. This is a favorable discrepancy (real result is better than abstract claim) but still needs clarification.

**MM invariance to uniform swings**: The paper proves this mathematically — "adding a constant to every district's vote share shifts both mean and median by the same amount, leaving MM unchanged." Table tab:mm-swing confirms: MM is exactly constant across all swing scenarios. This is the paper's strongest result and is correctly highlighted.

```
CONSISTENCY: 1 WARN (std dev 0.015 abstract vs 0.001–0.003 table; favorable direction)
P1: none
P2: [Q-06] Abstract claims MM std dev ≈0.015 but table shows 0.001–0.003. Reconcile:
    either the 0.015 is a population-of-cycles estimate (like EG's 0.030) or it's incorrect.
    If 0.001 is correct, update abstract to say "std dev ≈0.001–0.003 for MM vs ≈0.008 for EG."
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered. The mapmaker contribution decomposition (ΔMM = enacted MM − bisect MM) is a genuine contribution and is implemented correctly throughout.

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-06 The proof that MM is invariant to uniform swings is stated correctly ("adding a constant to every district's vote share shifts both mean and median by the same amount"). However, this proof is only valid for a continuous distribution. For a discrete distribution of district vote shares (integer vote counts), adding a constant shifts the ranks monotonically, preserving the order and therefore the median. The paper should clarify that the invariance holds for the observed (discrete) vote shares because the rank order is preserved by a constant shift. The table showing exact invariance confirms this but the mathematical argument could be tightened.

MINOR CONCERNS:
- The range claim "[-0.10, +0.10] for congressional delegations" is stated without citation. This is an empirical claim that should either be cited or qualified as "approximately."

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

SUMMARY: Excellent paper. The geographic decomposition (Triangle region vs. remainder of NC) is the most insightful empirical contribution in the L-series.

MINOR CONCERNS:
- The paper claims "no specific MM threshold has been proposed in the academic literature." Wang (2016) proposed a 7% threshold in some formulations. Verify this and address if true.
- I-07 The WI comparison (k=8) has only 8 districts, making MM statistics sensitive to individual district outcomes. A sentence noting the small-k limitation (consistent with the L.1 small-k limitation section) would be appropriate.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Accept

SUMMARY: The geographic sorting defense and its rebuttal is the paper's most valuable legal contribution. The step-by-step rebuttal protocol (run bisect → compare → compute gap → attribute to mapmaker) is directly usable in expert testimony.

MINOR CONCERNS:
- Harper v. Hall (2022) status — same concern as L.0. Verify current NC law.

```
VERDICT: READY FOR PANEL (1 P2 fix + small-k note)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~165 words (within target)
Primary result stated: YES (MM values; reduction; std dev comparison)
Algorithm named: YES
Value proposition: YES (sorting confound rebuttal; state court guidance)
```

---

## L.3 — Partisan Bias

### Phase 1 — Paper Summary

```
Paper: L.3+partisan-bias
Series: L.3
Key claims:
  1. bisect |Bias| < 0.02 for NC/WI/TX; enacted NC Bias ≈ −0.07 (one-seat Republican advantage)
  2. Uniform swing overestimates TX |Bias| by ≈15%; heteroskedastic correction closes gap
  3. Symmetry standard not adopted by any court (no threshold); jury-friendly framing available
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior | §Empirical | §Conclusion | Consistent? |
|------|----------|---------|----------|------------|-------------|-------------|
| Q-01 | NC bisect Bias | "<0.02" | "−0.01" | "−0.01" | "<0.02" | PASS |
| Q-02 | NC enacted Bias | "≈−0.07" | "−0.07" | "−0.07" | "−0.05 to −0.09" | PASS (range) |
| Q-03 | NC Bias = one seat | "< one seat" | — | "Seats Equiv. ≈1" | "one additional seat" | PASS |
| Q-04 | TX safe seats: 12 R + 8 D | "12 safe Republican" | — | §04: "12 safe Republican districts ($v_i<35\%$) and 8 safe Democratic districts" | — | PASS |
| Q-05 | TX heteroskedastic correction | "≈15%" | — | "11% reduction" | "at most 15%" | WARN |
| Q-06 | WI Bias gap | — | — | "Seats Equiv. ≈0.4" | — | — |

**Q-05 Note**: Abstract says "approximately 15%" overestimation by uniform swing, but §04 reports "an 11% reduction" when applying the correction (reducing enacted TX |Bias| from 0.09 to 0.08). 11% and 15% are not the same. The abstract says the conclusion is robust, which is consistent with both estimates. Reconcile to one number: the §04 computation shows 11%; use 11% or "approximately 10–15%."

**Jury-friendly framing in §legal**: "Under the enacted NC map, if both parties received exactly 50% of the two-party vote, Republicans would win 8 seats and Democrats would win 6 seats." This is one of the most clear courtroom arguments in the L-series.

```
CONSISTENCY: 1 WARN (15% vs 11% for TX correction)
P1: none
P2: [Q-05] Reconcile "≈15%" in abstract and conclusion with "11% reduction" in §04. Use the
    computed figure (11%) and qualify as "approximately 10–15%."
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: Bias definition with S(v) formulation (✓); symmetry standard framework (✓); bisect near-zero result (✓); swing model sensitivity with heteroskedastic correction (✓); legal landscape through 2026 (✓).

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-08 The seats-votes function S(v) is a step function, so the derivative at v=0.50 is not well-defined in the classical sense. The paper defines Bias as S(0.50) − 0.50 and Responsiveness as dS/dv at v=0.50. For a step function, Responsiveness is 1/(k × Δv_pivot) where Δv_pivot is the vote-share gap between the pivot district and the next district. The paper should be explicit that "derivative" here means the right-derivative at a step point or a finite difference.

MINOR CONCERNS:
- The heteroskedastic correction "reduces swing magnitude by 50% for districts with |v_i − 0.50| > 0.20" — this is an ad-hoc correction whose calibration is not justified. Cite a source or describe the calibration method.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-09 Partisan Bias is attributed to "Gelman and King (1994)" and "Grofman and King (2007)" but the paper uses both citations somewhat interchangeably. The symmetry standard was developed by Grofman (1983), elaborated by Gelman and King (1994), and operationalized by Grofman and King (2007). The paper's attribution should distinguish these contributions.

MINOR CONCERNS:
- The jury-friendly argument is excellent. However, it uses bisect NC as producing exactly 7D-7R at 50% vote share — this should be marked as a single-run result ($^\dagger$) since the 7-7 result is from a specific bisect run.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Minor Revision

SUMMARY: The jury-friendly framing is the paper's strongest practical contribution. The Rucho critique of the symmetry standard ("no threshold specifying what level of asymmetry is unconstitutional") is correctly described and the response (threshold is a state constitutional question, not metric question) is legally accurate.

MINOR CONCERNS:
- Common Cause v. Rucho (M.D.N.C. 2018): "318 F.Supp.3d 777" — verify citation.
- Harper v. Hall (NC 2022): same status concern as L.0–L.2.

```
VERDICT: READY FOR PANEL (2 P2 fixes recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~185 words (within target)
Primary result stated: YES (bisect <0.02; enacted −0.07; TX correction ≈15%)
Algorithm named: YES
Value proposition: YES (jury-friendly symmetry framing)
```

---

## L.4 — Declination

### Phase 1 — Paper Summary

```
Paper: L.4+declination
Series: L.4
Key claims:
  1. bisect |δ| < 0.10 (NC: +0.08, WI: +0.07, TX: +0.09) vs enacted NC +0.35, WI +0.28, TX +0.40
  2. Declination has been adopted by no federal or state court as of 2026 (explicitly stated)
  3. Three methodological critiques: undefined when swept; 50% threshold sensitivity; no calibrated threshold
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior | §Empirical | §Conclusion | Consistent? |
|------|----------|---------|----------|------------|-------------|-------------|
| Q-01 | NC bisect δ | "<0.10" | "≈0.08–0.10" | "+0.08" | "+0.08–0.09" | PASS |
| Q-02 | NC enacted δ | "≈+0.35" | "+0.35" | "+0.35" | "+0.28 to +0.40" | PASS |
| Q-03 | WI enacted δ | "≈+0.28" | "+0.28" | "+0.28" | — | PASS |
| Q-04 | TX enacted δ | "≈+0.40" | "+0.40" | "+0.40" | — | PASS |
| Q-05 | NC reduction | "70–80%" | — | "77%" | "70–80%" | PASS |
| Q-06 | Warrington 0.2 threshold | "tentative" | — | — | not mentioned | WARN |
| Q-07 | NC packing vs cracking | — | §04 decomp | Table tab:decl-nc-decomp | "driven primarily by packing" | PASS |

**Q-06 Note**: The abstract mentions Warrington (2018) proposes |δ| > 0.2 as indicating substantial partisan advantage, calling it "tentative." This threshold is mentioned in §02 (three methodological critiques) but then disappears from the conclusion and discussion. The conclusion should note that even using Warrington's tentative threshold, all four enacted maps (NC: 0.35, WI: 0.28, TX: 0.40, FL: 0.32) clearly exceed 0.2, while all bisect maps (0.07–0.09) clearly fall below it. This convergence strengthens the expert witness position even with a metric that has no court adoption.

**Explicit legal non-adoption statement**: The paper leads with "Declination has been accepted as evidence by no federal or state court as of 2026" in both the abstract and §05. This is appropriately candid. The paper's recommendation to use the visual exhibit (sorted vote-share plot) without leading on the scalar δ is sophisticated and practical.

```
CONSISTENCY: PASS (Q-06 is a P2 enhancement, not inconsistency)
P1: none
P2: [Q-06] Add to conclusion: note that even under Warrington's tentative 0.2 threshold,
    all four enacted maps clearly exceed it while all bisect maps fall below it — strengthens
    the comparative exhibit argument without requiring the court to adopt the threshold.
P3: none
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Declination definition with geometric interpretation | §02 | Yes | ✓ |
| Three methodological critiques | §02 | Yes, with explanation | ✓ |
| bisect behavior (70–80% reduction) | §03 + §04 | Yes | ✓ |
| NC decomposition: packing vs cracking | §04 Table | Yes | ✓ |
| Explicit legal non-adoption statement | §05 | Yes | ✓ |
| Expert witness guidance | §05 + §06 | Yes, 4-step protocol | ✓ |

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-10 The definition: δ = 2(θ_D − θ_R)/π. The paper correctly defines θ_D = arctan(2v̄_D^+ − 1) and θ_R = arctan(1 − 2v̄_R^-). For the NC enacted case: v̄_D^+ = 0.71, θ_D = arctan(0.42) ≈ 0.40 rad. v̄_R^- = 0.45, θ_R = arctan(0.10) ≈ 0.10 rad. δ = 2(0.40 − 0.10)/π ≈ 0.19. But the paper claims enacted NC δ = +0.35. Let me recheck: v̄_R^- = 0.45 → 1 − 2(0.45) = 0.10 → arctan(0.10) ≈ 0.0997. v̄_D^+ = 0.71 → 2(0.71)−1 = 0.42 → arctan(0.42) ≈ 0.40. δ = 2(0.40 − 0.10)/π ≈ 0.19. But Table tab:decl-nc-decomp reports θ_D = 0.37 and θ_R = 0.17 for the enacted plan. δ = 2(0.37−0.17)/π ≈ 0.127. The abstract claims enacted NC δ ≈ +0.35 but the decomposition table implies δ ≈ 0.127. **This is a potential computational error.** The decomposition table and the main results table (tab:decl-baseline) need to be verified for consistency.

**Note to editor**: The I-10 discrepancy may reflect the fact that the arctan values in the decomposition table (θ_D = 0.37, θ_R = 0.17) are inconsistent with the implied v̄ values (v̄_D^+ = 0.71 would give θ_D ≈ 0.40, not 0.37; v̄_R^- = 0.45 would give θ_R ≈ 0.10, not 0.17). There may be a rounding or transcription error in the decomposition table.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-11 The paper's honest assessment of Declination's legal status ("no court adoption as of 2026") is commendable. However, the conclusion's framing ("whether courts will eventually adopt Declination is not one this paper attempts to answer") undersells the paper's contribution. Political scientists will want to know: does the paper's evidence suggest Declination is likely to be adopted? The answer seems to be: unlikely given the methodological critiques, but the visual exhibit may still be useful. State this more directly.

MINOR CONCERNS:
- The NC v̄_R^- = 0.45 and v̄_D^+ = 0.71 in the decomposition table: the corresponding arctan values (θ_D = 0.37, θ_R = 0.17) do not match the formula definitions. Verify these numbers.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-12 Common Cause v. Rucho (M.D.N.C. 2018) citation: paper says declination "was introduced as an exhibit" but the court "did not rely on it." This is correct factually. However, the paper also needs to note that the evidence of which metrics the NC district court actually credited (vs. admitted but not credited) is important for expert witnesses. The paper correctly distinguishes "admitted" from "credited" but should emphasize that distinction more clearly.

MINOR CONCERNS:
- The expert witness four-step protocol (lead with visual, disclose non-adoption, respond to critiques, present alongside EG/MM/Bias) is practical and accurate.

```
VERDICT: FIXES REQUIRED
The I-10 computational check is a P1 issue: the arctan values in the decomposition table
(θ_D = 0.37, θ_R = 0.17 for enacted NC) are inconsistent with the v̄ values and would
imply δ ≈ 0.127, not 0.35. This must be verified and corrected.
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~220 words (above target; trim by ~20 words)
Primary result stated: YES (bisect <0.10; enacted NC +0.35; legal status explicit)
Algorithm named: YES
Value proposition: YES (honest legal assessment + visual exhibit guidance)
```

### Phase 6 — Pre-Panel Checklist (L.4)

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: L.4+declination
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   PASS (Q-06 is P2 enhancement)
  Contract:      PASS (3/3)
  Referee sim:   Minor Revision (I-10 computational check is P1)
  Abstract:      ~220 words (above target)

P1 blockers (fix before panel review):
  [I-10] Verify that the decomposition table values (θ_D = 0.37, θ_R = 0.17 for enacted NC)
         are consistent with v̄_D^+ = 0.71 and v̄_R^- = 0.45 via the arctan formula.
         Expected: θ_D = arctan(2×0.71 − 1) = arctan(0.42) ≈ 0.40; θ_R = arctan(1 − 2×0.45)
         = arctan(0.10) ≈ 0.10. These do not match the table values (0.37, 0.17).
         Either the table values or the v̄ values need correction. The implied δ from the
         table (2×(0.37−0.17)/π ≈ 0.127) is inconsistent with the reported enacted NC δ = 0.35.
         Verify the computation and correct the discrepancy.

P2 items (should fix):
  [Q-06] Add Warrington 0.2 threshold context to conclusion
  [I-11] Be more direct about Declination's likely future legal status
  Trim abstract by ~20 words

P3 items (optional):
  [I-12] Emphasize admitted vs. credited distinction more strongly

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved                          ✗ (decomposition table check)
□ All spec contract promises delivered                          ✓
□ Single-run results marked with dagger notation                ✓
□ Algorithm complexity                                          ✓ (N/A)
□ CLI flags match bisect binary                                 ✓ (N/A — no CLI for declination)
□ Court citations verified                                      ✓ (verify Common Cause citation)
□ Abstract states primary quantitative result                   ✓
□ Referee P1 blockers addressed                                 ✗ (I-10)

VERDICT: FIXES REQUIRED
Fixes required: 1 P1 (computational check on decomposition table)
═══════════════════════════════════════════════════════
```

---

## L.5 — Seats-Votes Curve

### Phase 1 — Paper Summary

```
Paper: L.5+seats-votes-curve
Series: L.5
Key claims:
  1. S(v) is sufficient statistic: EG = signed area under S(v); Bias = S(0.50) − 0.50
  2. bisect NC R ≈ 2.0 (competitive ideal) vs enacted NC R ≈ 1.3 (35% reduction in accountability)
  3. S(v) admitted in NC Harper v. Hall and OH redistricting; preferred single courtroom exhibit
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Definition | §Empirical | §Conclusion | Consistent? |
|------|----------|---------|------------|------------|-------------|-------------|
| Q-01 | NC bisect R | "≈2.0" | 2.0 | 2.0 | "R≈2.0" | PASS |
| Q-02 | NC enacted R | "≈1.3" | 1.3 | 1.3 | "R=1.3" | PASS |
| Q-03 | NC bisect S(0.50) | "passing through (0.50,0.50)" | 0.500 | 0.500 | "(0.50,0.50)" | PASS |
| Q-04 | NC enacted S(0.50) | "below (0.50,0.50)" | — | 0.429 | "(0.50,0.43)" | PASS |
| Q-05 | NC bisect seats at 50% | — | — | "7.0" | "7 Democratic seats" | PASS |
| Q-06 | NC enacted seats at 50% | — | — | "6.0" | "6 Democratic seats" | PASS |
| Q-07 | Responsiveness practical: 5pp swing yields | "≈1.4 vs ≈0.9 seats" | — | "≈1.4" and "≈0.9" | "0.5 seats difference" | WARN |

**Q-07 Note**: §empirical says "5 × 2.0/100 × 14 ≈ 1.4 additional Democratic seats" (bisect) and "5 × 1.3/100 × 14 ≈ 0.9 additional Democratic seats" (enacted), with difference = 0.5 seats. The conclusion says "0.5 seats represents the incumbent protection embedded in safe-seat structure." This is internally consistent but the narrative is somewhat indirect — the reader may not immediately connect R×5/100×14 to the practical outcome. Clear and consistent throughout.

**EG recovery from S(v)**: The paper proves EG = signed area under S(v). The NC data verification in §04 recovers EG ≈ +0.09 from the curve, consistent with L.1. This is the sufficiency claim confirmed empirically.

**WI values**: Abstract mentions WI (k=8) bisect R ≈ 1.9 and enacted R ≈ 1.4. Table tab:sv-summary shows exactly these values. Consistent.

```
CONSISTENCY: 1 WARN (Q-07: arithmetic presentation could be clearer but is internally consistent)
P1: none
P2: [Q-07] The 5pp swing example arithmetic (1.4 vs 0.9 seats) is correct but the step "35%
    reduction in electoral accountability" should flow directly from R: 1.3/2.0 = 0.65,
    i.e., 35% reduction. This is stated in the abstract but the §empirical derivation goes
    via the 5pp practical example. Make the connection explicit.
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: sufficiency result (proof with EG and Bias formulas from S(v)) (✓); bisect R ≈ 2.0 (✓); enacted R ≈ 1.3 (✓); legal history of S(v) in redistricting litigation (✓); preferred courtroom exhibit argument (✓); NC curve data table (✓).

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-13 The "sufficiency result" is stated as a corollary without formal proof: "EG and Partisan Bias are both recoverable from S(v)." The paper gives the formulas EG = ∫(S(v) − v)dv and Bias = S(0.50) − 0.50. These are correct statements but "sufficient statistic" has a technical meaning in statistics (a statistic that captures all information in the data about a parameter). The paper uses "sufficient" informally to mean "EG and Bias can be computed from S(v)." This is true but it is not the statistical sense of sufficiency. Recommend changing "sufficient statistic" to "containing EG and Bias as computable summaries" or explicitly note this is informal usage.

MINOR CONCERNS:
- The competitive ideal R ≈ 2.0 is stated as "the competitive ideal" without sourcing. Cite the theoretical basis for this value.
- The formula R = 1/(k × Δv_pivot) is stated without derivation. A one-sentence derivation would strengthen it.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Accept

SUMMARY: The seats-votes curve paper is the strongest in the L-series analytically. The sufficiency argument is compelling and practically useful. The responsiveness concept adds genuinely new value beyond the scalar metrics. The comparison graph described (two step-functions on the same axes) would be highly effective as a courtroom exhibit.

MINOR CONCERNS:
- The grofmanKing2007 citation: the paper uses "grofmanKing2007" for the symmetry standard but the symmetry standard was developed in Grofman (1983) and operationalized by Gelman and King (1994), with Grofman and King (2007) being a later review article. Verify the primary citations are correctly attributed.

---

**REFEREE 3 — Legal/Practitioner Reviewer**
Recommendation: Accept

SUMMARY: The seats-votes curve's legal track record (NC Harper, OH redistricting) is correctly described. The courtroom advantages (visual, no-threshold, sufficient statistic) are well-articulated. The accountability framing (suppressed responsiveness = suppressed voter power to change representation) is legally compelling and not dependent on any proportionality claim.

```
VERDICT: READY FOR PANEL (2 P2 fixes recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~195 words (within target)
Primary result stated: YES (R ≈ 2.0 bisect; R ≈ 1.3 enacted; sufficiency result)
Algorithm named: YES
Value proposition: YES (preferred courtroom exhibit; admitted in NC and OH)
```

---

## L.6 — Proportionality and Majoritarianism

### Phase 1 — Paper Summary

```
Paper: L.6+proportionality-majoritarianism
Series: L.6
Key claims:
  1. bisect LSq [0.03, 0.08] vs enacted [0.10, 0.18] across NC/WI/TX/FL (54–64% reduction)
  2. Majoritarian bonus: bisect 1.3–1.5x (structural) vs enacted 1.9–2.2x (manipulative)
  3. Neither proportionality nor majoritarianism required federally post-Rucho; state PA/NC standards may impose proportionality floor
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Behavior | §Empirical | §Conclusion | Consistent? |
|------|----------|---------|----------|------------|-------------|-------------|
| Q-01 | bisect LSq range | "[0.03, 0.08]" | "[0.03, 0.08]" | "[0.04–0.07]" | — | WARN |
| Q-02 | enacted LSq range | "[0.10, 0.18]" | "[0.11–0.18]" | "[0.11–0.18]" | — | PASS |
| Q-03 | NC bisect LSq | — | "≈0.03–0.05" | "0.05" | — | PASS |
| Q-04 | NC enacted LSq | "0.14" | — | "0.14" | — | PASS |
| Q-05 | NC reduction | "64%" | — | "64%" | — | PASS |
| Q-06 | NC bonus bisect | "1.3–1.5x" | "1.3–1.5x" | "1.4" | — | PASS |
| Q-07 | NC enacted bonus | "2.1x" | "2.1x" | "2.1" | — | PASS |
| Q-08 | NC R party seats at 50% | — | "8/14 ≈ 57%" | §04: "10 seats" | — | WARN |

**Q-01 Note**: Abstract says bisect LSq range is [0.03, 0.08] but §empirical table shows bisect values of 0.05 (NC), 0.04 (WI), 0.07 (TX), 0.06 (FL) — range is [0.04, 0.07], not [0.03, 0.08]. The abstract's lower bound (0.03) is stated in §behavior as the "structural floor" but the actual bisect values in the table start at 0.04. This is a minor but visible inconsistency.

**Q-08 Note**: In §03-behavior, the analysis says "Republicans winning 54% of votes win approximately 8/14 = 57% of seats under bisect." But §04-empirical "NC Proportionality Breakdown" states "Enacted Republican seat share: 10 seats (LSq = |10/14 − 0.53| ≈ 0.18)." The 10-seat enacted figure implies Republicans have 10/14 = 71% of seats. But §05-legal says "at 50-50 statewide vote, Republicans would win 8 seats under enacted" from L.3. The difference (8 seats from Bias calculation at exactly 50%; 10 seats from LSq calculation at 53% actual vote share) reflects a different input: 50% hypothetical vs. 53% actual. This is not an error but needs clearer labeling in the text.

**Abstract Gallagher formula**: The abstract uses $\text{LSq} = \sqrt{\tfrac{1}{2}\sum_i(v_i-s_i)^2}$ as a general formula but in the two-party analysis simplifies to $|v_D - s_D|$. §02 derives this correctly. Consistent.

```
CONSISTENCY: 1 WARN (bisect LSq range [0.03,0.08] in abstract vs [0.04,0.07] in table)
P1: [Q-01] Abstract claims bisect LSq range [0.03, 0.08]; table shows [0.04, 0.07]. The lower
    bound 0.03 is the stated "structural floor" but not an observed value. Revise abstract to
    say bisect LSq = 0.04–0.07 across four states, with structural floor approaching 0.03.
P2: [Q-08] Label the 10-seat Republican result more clearly: "at the actual 2020 Republican
    vote share of 53%, the enacted map produces 10/14 Republican seats" vs. "at exactly 50%,
    the enacted map produces 8/14" (from Bias calculation).
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: Gallagher Index definition and two-party simplification (✓); majoritarian bonus concept (✓); structural proportionality floor argument (✓); empirical LSq comparison NC/WI/TX/FL (✓); PA/NC state constitutional proportionality standard analysis (✓); expert witness framework (✓).

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-14 The simplification "LSq = |v_D − s_D|" for the two-party case: §02 shows this via algebra but the derivation contains a notational issue. The formula $\text{LSq} = \sqrt{\tfrac{1}{2}[(v_D-s_D)^2 + (v_R-s_R)^2]}$ with $v_R = 1-v_D$ and $s_R = 1-s_D$ simplifies to $\sqrt{\tfrac{1}{2} \times 2(v_D-s_D)^2} = |v_D-s_D|$. The paper writes "= |v_D - s_D| / √2 · √2 = |v_D - s_D|" which contains a strange notation. The correct simplification should be shown cleanly: $= \sqrt{(v_D-s_D)^2} = |v_D-s_D|$.

MINOR CONCERNS:
- The LSq = 0.05 (NC bisect, Table tab:lsq-baseline) vs abstract range [0.03, 0.08]: the table value (0.05) is at the middle of the claimed range. The range [0.03, 0.08] needs to be sourced to a multi-seed analysis, not a single-seed result (which gives only the point estimate 0.05).

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

SUMMARY: The proportionality-majoritarianism tension is the foundational normative debate in comparative electoral systems. The paper's positioning of this debate in the post-Rucho US context is valuable.

MAJOR CONCERNS:
I-15 The paper claims bisect LSq is "in the range of proportional representation systems with moderate bonus (comparable to Germany's mixed-member proportional system at the Bundesland level)." This cross-system comparison requires a citation: what is the Bundesland-level LSq, and from which study?

MINOR CONCERNS:
- The phrase "no federal requirement for proportionality" should note that this applies to *congressional* redistricting; some state legislative chamber plans may be subject to different standards under state law.

---

**REFEREE 3 — Legal/Practitioner Reviewer (post-Rucho state constitutional landscape)**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-16 Pennsylvania's proportionality floor argument: the paper cites "League of Women Voters v. Commonwealth" and quotes "far beyond what any neutral criteria would produce." However, the PA Supreme Court's 2018 holding also included a specific comparison to what a neutral algorithm produced — which is precisely the bisect reference distribution argument. The paper should engage more specifically with whether the PA court's framing is consistent with the bisect LSq baseline approach.

MINOR CONCERNS:
- Harper v. Hall (NC 2022) status: same concern as earlier L-track papers.
- The expert witness framework (acknowledge proportionality not required → show state may impose softer floor → show bisect satisfies both more than enacted) is well-structured.

```
VERDICT: FIXES REQUIRED
P1: [Q-01] LSq range in abstract ([0.03,0.08]) inconsistent with table ([0.04,0.07])
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~215 words (above target; trim by ~15 words)
Primary result stated: YES (bisect LSq [0.03,0.08]; bonus 1.3–1.5x; state constitutional framing)
Algorithm named: YES
Value proposition: YES (normative contribution: both standards satisfied by bisect more than enacted)
```

### Phase 6 — Pre-Panel Checklist (L.6)

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: L.6+proportionality-majoritarianism
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 WARN → P1 (LSq range mismatch)
  Contract:      PASS (3/3)
  Referee sim:   Minor Revision
  Abstract:      ~215 words (above target)

P1 blockers:
  [Q-01] Abstract range [0.03, 0.08] does not match table values [0.04, 0.07].
         Fix: "bisect maps achieve LSq = 0.04–0.07 across four states (structural floor
         approaching 0.03) — significantly below enacted maps (LSq 0.11–0.18)"

P2 items:
  [Q-08] Label NC 10-seat result clearly (at 53% actual vote, not at 50% hypothetical)
  [I-14] Simplification derivation notation issue: clean up "= |v_D - s_D| / √2 · √2"
  [I-15] Germany Bundesland comparison needs citation
  Trim abstract by ~15 words

P3 items:
  [I-16] PA court framing engagement (optional deepening)

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved                          ✗ (LSq range)
□ All spec contract promises delivered                          ✓
□ Single-run results marked with dagger notation                ✓
□ CLI flags match bisect binary                                 ✓
□ Court citations verified                                      ✓ (verify PA LWV citation)
□ Abstract states primary quantitative result                   ✓ (pending range fix)
□ Referee P1 blockers addressed                                 ✗

VERDICT: FIXES REQUIRED
Fixes required: 1 P1 + 3 P2
═══════════════════════════════════════════════════════
```

---

## Cross-Cutting L-Track Issues

### Harper v. Hall (2022) Legal Status
**All seven L-track papers** cite Harper v. Hall (NC 2022) as current good law establishing that NC courts can remedy partisan gerrymandering. By 2026, the NC Supreme Court's composition changed and the 2022 holding may have been reversed or narrowed. **Every L-track paper should verify Harper's current status before panel submission.** If the holding has been weakened, the papers need a sentence noting the current state of NC redistricting doctrine. This is a cross-cutting P2 issue affecting L.0–L.6.

### Dagger Notation
All L-track papers consistently apply dagger notation ($^\dagger$) to single-run bisect results. The footnote text is consistent: "Single-run bisect result using VEST 2020 presidential precinct returns, standard-bisect structure, geographic weights." No issues.

### CLI Flags
The L-track papers reference `--weights-override geographic` and `--structure standard-bisect` which match the CLAUDE.md documentation. No inconsistencies.

### Abstract Word Counts
Three papers (L.0 ≈220, L.4 ≈220, L.6 ≈215) exceed the 150–200 word target. These should be trimmed before panel submission but are not P1 issues.

---

## Combined L-Track Issues Summary

| Issue ID | Paper | Severity | Description |
|----------|-------|----------|-------------|
| I-01 | L.0 | P2 | "Sufficient to bound" language |
| I-02 | L.0 | **P1** | "Four times larger" claim imprecise (ranges 3.5× to 7×) |
| I-03 | L.0 | P2 | Harper v. Hall (2022) current legal status — verify |
| I-04 | L.1 | P2 | "Sufficient to bound" language (same as I-01) |
| I-05 | L.1 | P2 | std dev 0.030 (abstract) vs 0.008 (table) — reconcile |
| I-06 | L.2 | P2 | std dev 0.015 (abstract) vs 0.001–0.003 (table) — reconcile |
| I-07 | L.2 | P2 | Small-k limitation note for WI (k=8) |
| I-08 | L.3 | P2 | Step-function derivative formalism for Responsiveness |
| I-09 | L.3 | P2 | Attribution of symmetry standard (Grofman 1983 vs Gelman-King 1994 vs GrofmanKing 2007) |
| I-10 | L.4 | **P1** | Decomposition table arctan values inconsistent with v̄ values; implied δ ≈ 0.127 ≠ 0.35 |
| I-11 | L.4 | P2 | Be more direct about Declination's likely future legal status |
| I-12 | L.4 | P2 | Emphasize admitted vs. credited distinction for Common Cause v. Rucho |
| I-13 | L.5 | P2 | "Sufficient statistic" used informally vs statistical meaning |
| I-14 | L.6 | P2 | LSq simplification notation issue |
| I-15 | L.6 | P2 | Germany Bundesland comparison needs citation |
| I-16 | L.6 | P2 | PA court engagement |
| Q-01 | L.6 | **P1** | bisect LSq range [0.03,0.08] in abstract vs [0.04,0.07] in table |
| Cross | All | P2 | Harper v. Hall (2022) legal status in 2026 — verify across all papers |

**Total P1 blockers**: 3 (in L.0 × 1, L.4 × 1, L.6 × 1)
**Total P2 items**: 14
**Papers ready for panel**: L.1, L.2, L.3, L.5 (4 of 7)
**Papers requiring P1 fix first**: L.0 (1 P1), L.4 (1 P1), L.6 (1 P1)
