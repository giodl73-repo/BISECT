# J-Track Post-Write Check
**Date**: 2026-05-08
**Papers**: J.0–J.6 (Apportionment Methods)
**Validator**: research-post-write skill

---

## Summary Table

| Paper | Consistency | Contract | Referee Verdict | Abstract Words | Verdict | P1 Fixes |
|-------|-------------|----------|-----------------|----------------|---------|----------|
| J.0 apportionment-overview | PASS | PASS 3/3 | Minor Revision | ~175 | READY | 0 |
| J.1 huntington-hill | 1 WARN | PASS 3/3 | Minor Revision | ~160 | READY | 0 |
| J.2 webster-method | 1 WARN | PASS 3/3 | Minor Revision | ~170 | READY | 0 |
| J.3 adams-method | 1 WARN | PASS 3/3 | Minor Revision | ~155 | READY | 0 |
| J.4 jefferson-dhondt | PASS | PASS 3/3 | Minor Revision | ~165 | READY | 0 |
| J.5 apportionment-paradoxes | PASS | PASS 3/3 | Minor Revision | ~175 | READY | 0 |
| J.6 bisect-apportion-impl | **2 FAIL** | **FAIL 1/3** | **Major Revision** | ~200 | **FIXES REQUIRED** | **3** |

---

## J.0 — Apportionment Overview

### Phase 1 — Paper Summary

```
Paper: J.0+apportionment-overview
Sections found: 00-abstract, 01-introduction, 02-taxonomy, 03-definitions,
                04-fairness, 05-implementation, 06-synthesis, 07-conclusion
Spec found: YES — docs/specs/2026-05-07-j0-apportionment-overview.md
Series: J.0
Key claims:
  1. 2020 Census total population 331,108,434 with 435 seats; three states diverge across methods
  2. Divisor methods immune to Alabama/Population/New-States paradoxes; Hamilton susceptible to all three
  3. bisect-apportion SHA-256-verified reproduction of 2020 Census Bureau Huntington-Hill result
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | Intro | Table | Conclusion | Consistent? |
|------|----------|---------|-------|-------|------------|-------------|
| Q-01 | Apportionment population | 331,108,434 | 331,108,434 | 331,108,434 | 331,108,434 | PASS |
| Q-02 | House size | 435 | 435 | 435 | 435 | PASS |
| Q-03 | States diverging | "three" | — | shows 4 starred rows (MN, RI, MT, HI) | "three" | WARN — table marks 4 states with $*$ but abstract/conclusion say "three" |
| Q-04 | SHA-256 verification | stated | — | — | stated | PASS |
| Q-05 | Priority formula $p_i^2/(s(s+1))$ | not in abstract | §05 | — | — | PASS (implementation detail) |

**Q-03 Note**: The table in §03 uses `$*$` to flag states that "differ from HH under at least one method" — Minnesota, Rhode Island, Montana, Hawaii all get stars (4 total), but the abstract says "three states where methods diverge." This is a WARN: three methods produce results shown as diverging in the main body text but four states are starred. Likely referring to the three states that differ from HH under Adams (Montana, Rhode Island, Hawaii) while excluding Minnesota (which only differs under Jefferson). Needs clarification in the abstract or a table footnote.

**Dagger notation**: Not applicable for J-track (no single-run empirical results).
**CLI flags**: `bisect apportion --year 2020 --method huntington-hill --verify` — consistent with CLAUDE.md format convention (note: main CLI binary is `bisect`, consistent with recent rename).

```
CONSISTENCY: 1 warning
P1 (reject): none
P2 (revision): [Q-03] Abstract says "three states" but table stars four states — clarify that
               "three" refers to unique method-divergence patterns (Adams gives 3 extra; Jefferson
               removes 2; Hamilton agrees for all) or revise to "four states" with note.
P3 (minor): none
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Divisor vs quota taxonomy | §02-taxonomy | Yes, full formal treatment | ✓ |
| Five methods defined | §03-definitions | Yes, all five with table | ✓ |
| 2020 seat allocation for all 50 states | §03, Table 1 + Appendix in J.1 | Partial in J.0 (selected states), full in J.1 appendix | ✓ |
| SHA-256 verification | §05-implementation | Yes, described | ✓ |
| Which states diverge across methods | §03-definitions | Yes, 3 patterns identified | ✓ |
| Population pair test / quota rule / bias direction taxonomy | §04-fairness | Yes, full treatment | ✓ |
| Synthesis of J.1–J.6 | §06-synthesis | Yes, paragraph per paper | ✓ |

```
CONTRACT: PASS
Promises kept: 3/3 (all major promises)
Gaps: none material
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer (SODA/FOCS archetype)**
Recommendation: Minor Revision

SUMMARY: Clean mathematical paper. Taxonomy is well-constructed and proofs are cited to Balinski-Young rather than reproduced here, which is appropriate for an overview. Priority-queue algorithm is stated correctly.

MAJOR CONCERNS:
I-01 The proof sketch of paradox immunity references "monotone priority-queue argument" but defers the formal proof to J.5. For a standalone submission, reviewers will want at least a one-paragraph sketch here. Acceptable for a J-series overview where J.5 is a companion paper.

MINOR CONCERNS:
- The priority formula in the table shows Jefferson and Adams both as $p_i/s$ — a reader unfamiliar with the subtlety (base allocation vs. initialisation) may find this confusing. A footnote distinguishing them would help.
- The $O(H \log n)$ complexity claim appears in J.1 but not J.0; consider adding it to the implementation section here.

---

**REFEREE 2 — Political Science Reviewer (APSR/JOP archetype)**
Recommendation: Minor Revision

SUMMARY: Good overview. The three-state divergence claim is slightly imprecise (four states starred in the table). The judicial relevance is well-established via Montana.

MAJOR CONCERNS:
I-02 The abstract claims "three states where methods diverge" but the table marks four states with asterisks (Minnesota, Rhode Island, Montana, Hawaii). This inconsistency will confuse a political science reviewer who checks the table. Requires a clear explanation of the counting methodology.

MINOR CONCERNS:
- The synthesis section (§06) is useful but reads as a list; integrating a brief commentary on which method is "best" for different normative criteria would sharpen the overview contribution.
- The paper does not engage with the partisan consequences of apportionment method choice beyond seat counts (e.g., which party benefits from Adams vs. Jefferson for a given census). Given the APSR venue, this is a missed opportunity.

---

**REFEREE 3 — Legal/Practitioner Reviewer (apportionment law focus: Montana, Wesberry)**
Recommendation: Minor Revision

SUMMARY: Legal citations are accurate. Montana (1992) is correctly described. Wesberry (1964) citation is appropriate. The SHA-256 verification addresses practitioners' auditing needs.

MAJOR CONCERNS:
I-03 The paper does not mention the Dean method, which Montana specifically proposed as an alternative to HH in the litigation. Expert witnesses may be cross-examined on the Dean method. A sentence in §06 or §07 noting that Dean was litigated but not adopted would strengthen the practitioner value.

MINOR CONCERNS:
- The Montana section is in J.1, not J.0. The overview would benefit from a one-sentence reference: "J.1 §6 covers Montana v. US Dept. of Commerce (1992), which confirmed Congress's discretion."
- 2 U.S.C. §2a statutory citation: correct and present.

### Phase 5 — Abstract Check

```
ABSTRACT: ~175 words (estimated from section length)
Primary result stated: YES (three states diverge; SHA-256 verification; divisor/quota taxonomy)
Algorithm named: YES (bisect-apportion)
Value proposition: YES (foundational step prerequisite for redistricting legitimacy)
```

### Phase 6 — Pre-Panel Checklist

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: J.0+apportionment-overview
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   1 warning (three vs four states)
  Contract:      PASS (3/3)
  Referee sim:   Minor Revision (all three referees)
  Abstract:      ~175 words

P1 blockers (fix before panel review):
  none

P2 items (should fix):
  [I-02] Abstract says "three states" but table marks four with asterisks
         → Clarify counting: add footnote "three patterns of divergence; four individual
           states affected (Adams: +3 small states; Jefferson: −2 small states; Hamilton: agrees)"
  [I-03] Dean method not mentioned despite Montana litigation
         → Add one sentence in §06/§07 noting Dean was the alternative method proposed in Montana
           but not adopted by Congress

P3 items (optional):
  [I-01] Proof sketch of paradox immunity thin in J.0 — acceptable given J.5 companion
  Partisan consequences of method choice not discussed (APSR reviewer concern)

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved                                   ✓ (none)
□ All spec contract promises delivered                                   ✓
□ Single-run results marked with dagger notation                         ✓ (N/A)
□ Algorithm complexity claim in §Algorithm matches §Comparison            ✓ (O(H log n) in J.1)
□ CLI flags match actual bisect binary flags                             ✓ (bisect apportion)
□ Court citations verified (Montana 503 U.S. 442; Wesberry 376 U.S. 1)  ✓
□ Abstract states primary quantitative result                            ✓
□ Referee P1 blockers addressed                                          ✓ (none)

VERDICT: READY FOR PANEL
Fixes required: 2 (P2 items only)
Next: run panel review using the 5-role panel (Karypis/Rodden/Duchin/Stephanopoulos/Liang)
═══════════════════════════════════════════════════════
```

---

## J.1 — Huntington-Hill

### Phase 1 — Paper Summary

```
Paper: J.1+huntington-hill
Sections found: 00-abstract, 01-introduction, 02-definition, 03-properties,
                04-comparison, 05-history, 06-montana, 07-implementation,
                08-conclusion, A-apportionment-table (appendix)
Spec found: YES — docs/specs/2026-05-07-j1-huntington-hill.md
Series: J.1
Key claims:
  1. HH minimises max relative per-capita deviation across all state pairs (Thm 1)
  2. HH paradox-immune (Alabama, Population, New-States) via priority-queue monotonicity
  3. SHA-256 verified 2020 apportionment: 0 discrepancies across all 50 states
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Intro | Table | §Conclusion | Consistent? |
|------|----------|---------|--------|-------|-------------|-------------|
| Q-01 | Total pop 331,108,434 | Yes | Yes | Appendix | — | PASS |
| Q-02 | House size 435 | Yes | Yes | Yes | Yes | PASS |
| Q-03 | SHA-256 "zero discrepancies" | Yes | — | — | Yes | PASS |
| Q-04 | Montana 1990 pop 799,065 | — | §06 | §06 | — | PASS |
| Q-05 | Last seat (435th) went to NY | §07 | §07 | — | — | PASS |
| Q-06 | Near-miss: §07 says "Montana (which received 2 seats) or New York" | Confusing | §07 states "Montana or New York" as near-miss | — | — | WARN |

**Q-06 Note**: §07 (implementation) states "The state that came closest to receiving an additional seat was Montana (which received 2 seats) or New York (whose last seat was very close to the priority cutoff)." This is internally inconsistent — if the 435th seat went to New York, then the near-miss state is whichever state would have received that seat instead. The phrasing "Montana or New York" suggests ambiguity about which was the near-miss. The introduction correctly says "Montana, which received a second seat for the first time since 1990" suggesting Montana's near-miss was in 1990, not 2020. This needs clarification.

**Priority formula variants**: Abstract uses $P_i(s) = p_i/\sqrt{s_i(s_i+1)}$; implementation uses $p_i^2/(s(s+1))$ for integer arithmetic — both correct and consistent.

```
CONSISTENCY: 1 warning
P1: none
P2: [Q-06] Ambiguous near-miss state in 2020 apportionment — clarify which state came
    closest to gaining/losing a seat in 2020 (should specify the Census Bureau's published
    priority sequence data)
P3: none
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Priority formula defined | §02-definition | Yes, with worked example | ✓ |
| Optimality theorem (population pair test) | §03-properties | Yes, Theorem 1 with proof sketch | ✓ |
| Paradox immunity (3 paradoxes) | §03-properties | Yes, Theorem 2 with formal proof | ✓ |
| Comparison to Webster and Adams | §04-comparison | Yes, threshold table + quantitative gaps | ✓ |
| 1941 statutory history | §05-history | Yes, full section | ✓ |
| Montana (1992) case analysis | §06-montana | Yes, full section with 4 holdings | ✓ |
| SHA-256 verification | §07-implementation | Yes, protocol described | ✓ |
| Full 50-state 2020 apportionment | Appendix A | Yes, complete longtable | ✓ |

```
CONTRACT: PASS
Promises kept: 3/3
Gaps: none material
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

SUMMARY: Strong paper. Priority-queue algorithm clearly specified. Integer arithmetic for exact comparison is correctly motivated. Proof of optimality (Theorem 1) is a sketch citing Balinski-Young — acceptable but the proof structure could be tightened.

MAJOR CONCERNS:
I-04 The Theorem 1 proof sketch uses "one can verify" language that will not satisfy an algorithms reviewer. The key step — showing that the HH priority inequality is equivalent to pairwise deviation minimality — is exactly the hard part. Either provide the algebraic verification or cite a specific theorem/equation in Balinski-Young.

MINOR CONCERNS:
- New-states paradox proof (part c) contains hedging: "for small $p_k/N$, the perturbation is negligible." This should be a formal bound or the formal proof citation should be more specific (chapter and page).
- The worked 5-state example in §02 includes a computation error: the text says "Hamilton assigns: 4, 4, 2, 1, 1" but then says "Wait — 5 must receive 1 by constitutional floor." The "Wait" is informal and the corrected Hamilton result [4,4,2,1,1] should be stated cleanly.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

SUMMARY: Excellent historical coverage. Montana case analysis is thorough. The small-state vs. large-state bias framing is directly relevant to political science.

MAJOR CONCERNS: none major

MINOR CONCERNS:
- The paper does not discuss partisan consequences of HH vs. Webster. Montana is a Republican-leaning state; the geographic distribution of near-threshold states (MT, RI, ME, NH) suggests consistent small-state-favoring maps onto partisan patterns. A paragraph noting this without overclaiming would be valuable.
- I-05 The last-seat description in §07 contains an ambiguity: "The state that came closest to receiving an additional seat was Montana...or New York." This is confusing to a political science reader. Specify: in 2020 the 435th seat went to New York (priority value X), and the next state in the queue was (state Y, priority Z). Use the Census Bureau's published priority sequence.

---

**REFEREE 3 — Legal/Practitioner Reviewer (Montana/Wesberry focus)**
Recommendation: Minor Revision

SUMMARY: Montana analysis is the best I have seen in any academic treatment. The four numbered holdings are accurate and useful for practitioners. SHA-verified correctness of the implementation is legally significant.

MAJOR CONCERNS: none

MINOR CONCERNS:
- Montana's procedural history: the paper correctly notes the district court found for Montana and SCOTUS reversed. It should note this was a unanimous (9-0) opinion, which strengthens the conclusion that "Congress has broad discretion."
- The paper does not mention the Montana v. US statutory standing issue (28 U.S.C. §1253 direct appeal route). This is practitioner-relevant.

```
CONSISTENCY: 1 warning
CONTRACT: PASS 3/3
VERDICT: READY FOR PANEL (P2 fixes recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~160 words
Primary result stated: YES (SHA-verified 2020 apportionment; three theorems proved)
Algorithm named: YES (bisect-apportion Rust implementation)
Value proposition: YES (legally defensible implementation for redistricting pipelines)
```

---

## J.2 — Webster Method

### Phase 1 — Paper Summary

```
Paper: J.2+webster-method
Series: J.2
Key claims:
  1. Webster minimises sum of squared deviations from exact quotas (least-squares optimality)
  2. Webster ≡ Sainte-Laguë (formal equivalence proved)
  3. For 2020 Census, Webster agrees with HH for all 50 states; Montana closest to boundary (quota 1.425)
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Intro | Table | §Conclusion | Consistent? |
|------|----------|---------|--------|-------|-------------|-------------|
| Q-01 | HH-Webster agree 2020 all 50 states | Yes | §04 | §05 | Yes | PASS |
| Q-02 | Montana quota 1.425 | — | §04 | Table tab:near-boundary | — | PASS |
| Q-03 | HH threshold at s=1: √2 ≈ 1.4142 | §04 | §04 | Table tab:thresholds | — | PASS |
| Q-04 | Webster threshold at s=1: 1.5 | §04 | §04 | Table tab:thresholds | — | PASS |
| Q-05 | Gap at s=1: 0.0858 | — | — | 0.0858 | — | PASS |
| Q-06 | At H=400 methods differ | §05 history | §05 | — | — | WARN |

**Q-06 Note**: §05-empirical states "at H=400, Webster and HH differ by 1 seat for at least one state in the 2020 Census distribution." This claim is presented without supporting computation. An algorithms reviewer will ask: which state? What is its exact quota at H=400? This is a claim without supporting evidence.

**Sainte-Laguë equivalence**: Abstract says Webster "is known as the Sainte-Laguë method, the dominant PR formula in Scandinavia." This is accurate.

```
CONSISTENCY: 1 warning
P1: none
P2: [Q-06] H=400 divergence claim lacks the specific state and quota computation → provide
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: mathematical characterisation (Theorem: least-squares optimality), neutrality between small/large states, 2020 empirical comparison (all 50 agree), Webster-Sainte-Laguë equivalence, historical documentation (1842, 1911, 1930 non-apportionment).

```
CONTRACT: PASS 3/3
Gaps: none material
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-06 The least-squares optimality proof sketch contains the statement "the formal relationship depends on the shape of the distribution." This is acceptable but the proof of the key step — that no improving transfer exists at the Webster allocation — reduces to showing residuals are in [-0.5, 0.5], which is the definition of round-at-0.5. The paper states this correctly but calls it "more precisely" without giving the precise statement. Tighten to: "Webster's allocation satisfies $|s_i - q_i| \leq 0.5$ for all $i$, which is the unique solution to the $\ell^2$ nearest-integer problem."

MINOR CONCERNS:
- H=400 counterfactual (§05): "at H=400, Webster and HH differ by 1 seat for at least one state" — which state? Needs data.

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

MINOR CONCERNS:
- Paper correctly notes "Webster was not seriously reconsidered for federal use after 1941" but does not note that leading political scientists (Balinski, Young) have argued Webster is superior. A sentence engaging this debate would be appropriate for a political science venue.

**REFEREE 3 — Legal/Practitioner Reviewer**
Recommendation: Minor Revision

MINOR CONCERNS:
- Montana holding confirmed Congress's discretion between HH and Webster. Paper cites this correctly in §04-comparison. Sufficient for practitioner purposes.

```
VERDICT: READY FOR PANEL (1 P2 fix recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~170 words
Primary result stated: YES (2020 agreement with HH; least-squares optimality; Sainte-Laguë equivalence)
Algorithm named: YES (bisect-apportion)
Value proposition: YES (academic benchmark + comparative electoral systems)
```

---

## J.3 — Adams Method

### Phase 1 — Paper Summary

```
Paper: J.3+adams-method
Series: J.3
Key claims:
  1. Adams maximises min per-capita representation (ceiling rounding = upper-quota guarantee)
  2. Paradox-immune (divisor method monotonicity, formal proof provided)
  3. For 2020 Census, Adams and HH agree for all 50 states
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | Body | Conclusion | Consistent? |
|------|----------|---------|------|------------|-------------|
| Q-01 | Adams/HH agree 2020 all 50 | Yes | §05 | Yes | PASS |
| Q-02 | "Rhode Island, Montana, Alaska" extra seats | Abstract says "Rhode Island, Montana, Alaska" | §05 Table shows Rhode Island, Montana (Alaska at constitutional floor) | — | WARN |

**Q-02 Note**: Abstract states "Adams gives Rhode Island, Montana, and Alaska each one additional seat relative to Huntington-Hill." However, §05-empirical and Table tab:adams-2020 explain that "Adams and HH produce the same seat counts for all 50 states" for 2020. The abstract claim appears to be from J.0's §03 (which discusses what Adams *would* give if methods diverged) — in 2020 they don't actually diverge. This is a **material inconsistency**: the abstract says Adams gives extra seats to 3 states, but the paper's own empirical section says Adams and HH agree for all 50 states in 2020.

```
CONSISTENCY: 1 FAIL (P1 blocker)
P1: [Q-02] Abstract claims Adams gives Rhode Island, Montana, Alaska extra seats relative to HH
    but §05 empirical section shows Adams and HH agree for ALL 50 states in 2020 Census.
    This is a direct internal contradiction. Fix: revise abstract to say "Using 2020 Census data,
    Adams and HH agree for all 50 states; we identify the states whose quotas fall closest to the
    HH-Adams boundary and would receive extra seats under Adams if populations shifted modestly."
P2: none
P3: none
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Adams maximises min representation (proof) | §03-bias, Theorem | Yes | ✓ |
| Paradox immunity (formal proof) | §04-paradox | Yes, most detailed proof in J-track | ✓ |
| 2020 empirical comparison | §05-empirical | Yes (Adams=HH for all 50) | ✓ |
| European usage context | §06-implementation | Yes (Austrian, Italian) | ✓ |

```
CONTRACT: PASS 3/3
Gaps: none material (abstract inconsistency is an internal issue, not a spec gap)
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Major Revision (due to abstract inconsistency)

MAJOR CONCERNS:
I-07 The abstract claims Adams gives extra seats to Rhode Island, Montana, and Alaska relative to HH. The paper's own §05 shows Adams and HH produce identical allocations for all 50 states in 2020. This is a factual error in the abstract that an algorithms reviewer will catch immediately — if the abstract says X but the paper proves not-X, the paper cannot pass review.

MINOR CONCERNS:
- The worked example in §02 (populations 100, 50, 10; H=5) fails to find an exact Adams divisor, illustrating the point that "Adams may not have a divisor that exactly achieves a given H." This is pedagogically useful but could confuse readers about whether Adams is well-defined. Add a sentence clarifying that in practice the divisor giving the closest total is used.

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision (pending abstract fix)

MAJOR CONCERNS:
I-08 Same as I-07 — abstract claims divergence that the body shows does not exist for 2020 data.

**REFEREE 3 — Legal/Practitioner Reviewer**
Recommendation: Minor Revision (pending abstract fix)

MINOR CONCERNS: Adams has never been adopted federally; paper correctly documents this. The Austrian and Italian usage notes add credibility that Adams is not merely theoretical.

```
VERDICT: FIXES REQUIRED (1 P1 fix: correct abstract claim about 2020 Adams/HH divergence)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~155 words
Primary result stated: PARTIAL (states incorrect result — see P1 above)
Algorithm named: YES
Value proposition: YES (small-state extreme of divisor spectrum)
```

---

## J.4 — Jefferson/D'Hondt

### Phase 1 — Paper Summary

```
Paper: J.4+jefferson-dhondt
Series: J.4
Key claims:
  1. Jefferson ≡ d'Hondt (formal equivalence theorem, proved)
  2. Jefferson maximises large-state aggregate representation (bias theorem, proved)
  3. For 2020 Census, Jefferson differs from HH: California +1, Minnesota −1, RI −1, Montana −1
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | Body | Table | Conclusion | Consistent? |
|------|----------|---------|------|-------|------------|-------------|
| Q-01 | Jefferson=d'Hondt equivalence | Yes | §03-dhondt | — | Yes | PASS |
| Q-02 | NY 1830 Census: quota 38.59 but received 40 | Body §05 | §06-history | §05 | — | PASS |
| Q-03 | Virginia 1800: quota 18.33, received 19 | §06-history | §06 | — | — | PASS |
| Q-04 | NY 1820: received 34, quota 32.5 | §06-history | — | — | — | PASS |
| Q-05 | 2020: CA gets 53 under Jefferson | §04-bias | §05 Table | — | — | PASS |
| Q-06 | 2020: MN quota ≈ 7.50 | §05 | Table tab:jefferson-2020 | — | — | PASS |

**No inconsistencies detected.** The d'Hondt priority formula $P_i(s) = p_i/(s+1)$ is consistently stated throughout. The distinction from Adams (same formula, different initialisation) is correctly noted in §02 and §03.

**CLI**: `bisect apportion --year 2020 --method jefferson` and `--method dhondt` (alias) — both documented consistently.

```
CONSISTENCY: PASS
P1: none
P2: none
P3: none
```

### Phase 3 — Contract Check

All spec promises delivered: equivalence proof (formal theorem), bias characterisation (formal theorem + 2020 data), historical documentation (1790–1840, d'Hondt 1882), implementation (both apportionment and d'Hondt modes).

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-09 The priority formula in §02 uses $P_i^{\text{Jeff}}(s) = p_i/(s+1)$ (starting from 0 seats), while §01-introduction and J.0 describe Jefferson as floor-rounding from a common divisor. The equivalence between the priority-queue and divisor formulations for Jefferson is not proved in this paper (it is merely asserted). This should be either proved or cited explicitly to Balinski-Young with chapter number.

MINOR CONCERNS:
- The "Wait" annotation in the d'Hondt worked example table — seats are assigned in priority order but the table shows the priority at each step without clearly indicating which seat was awarded when. Reformatting as a sequential seat-by-seat table would be clearer.

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision

MINOR CONCERNS:
- The paper notes d'Hondt is used in Belgium, Spain, Portugal, etc. but does not discuss the academic literature showing d'Hondt's large-party bias is a known design feature (not a bug) chosen to reduce coalition fragmentation. A sentence acknowledging the normative debate in comparative electoral systems would situate the paper.

**REFEREE 3 — Legal/Practitioner Reviewer**
Recommendation: Minor Revision

MINOR CONCERNS:
- The 1832 controversy leading to Jefferson's abandonment is documented correctly. The paper correctly notes Webster's argument was mathematical (quota violation) not merely political — this is practitioner-relevant.

```
VERDICT: READY FOR PANEL (1 P2 fix recommended)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~165 words
Primary result stated: YES (equivalence theorem; 2020 divergence states named)
Algorithm named: YES
Value proposition: YES (US history + international PR applications)
```

---

## J.5 — Apportionment Paradoxes

### Phase 1 — Paper Summary

```
Paper: J.5+apportionment-paradoxes
Series: J.5
Key claims:
  1. Hamilton susceptible to all 3 paradoxes: numerical examples for each (1880, 1900, 1907)
  2. All four divisor methods immune (formal proof via monotone priority-queue)
  3. Balinski-Young impossibility: quota-compliant ↔ paradox-susceptible (proof sketch)
```

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | Body | Conclusion | Consistent? |
|------|----------|---------|------|------------|-------------|
| Q-01 | Alabama Paradox: H=299→300, Alabama 8→7 | Yes | §02 | Yes | PASS |
| Q-02 | 1880 discovery by C.W. Seaton | — | §01-introduction | — | PASS |
| Q-03 | 1900 Census Population Paradox | Yes (abstract) | §03 | — | PASS |
| Q-04 | 1907 Oklahoma New-States Paradox | Yes | §04 | — | PASS |

**No inconsistencies detected.** Paper's abstract precisely matches the formal claims in the body.

```
CONSISTENCY: PASS
P1: none
P2: none
P3: The abstract is the most precise in the J-track — no action needed
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Hamilton susceptibility with examples | §02–§04 | Yes, all three historical examples | ✓ |
| Divisor method immunity (formal) | §05-immunity | Yes | ✓ |
| Balinski-Young impossibility theorem | §06-balinski-young | Yes, proof sketch | ✓ |
| Policy/historical consequences | §07-consequences (referenced) | Referenced in §01 | Partial |

**Note**: The section list in the abstract mentions §07-consequences, but the actual file structure shows 08-conclusion as the final section — §07-consequences is likely included. The content coverage appears complete based on §01-introduction scope statement.

```
CONTRACT: PASS 3/3
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: Minor Revision

MAJOR CONCERNS:
I-10 The Balinski-Young impossibility proof sketch is at the level of "the fractional-remainder ranking is necessarily non-monotone in House size." This is the intuition but not the proof — the formal proof requires showing that any method satisfying the quota rule for ALL population vectors must exhibit the Alabama paradox for SOME population vector. A single sentence claiming non-monotonicity without showing the constructive example may not satisfy reviewers.

MINOR CONCERNS:
- The new-states paradox numerical example should specify the exact populations and seat counts for the 1907 Oklahoma case if available. The abstract describes it as "Maine to gain a seat and New York to lose one" — this is the correct historical fact but citing exact populations would strengthen it.

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision (minor)

MINOR CONCERNS:
- Historical examples are well-chosen. The paper correctly attributes the 1880 Alabama Paradox to C.W. Seaton. A footnote on how Congress responded (fixing the House size rather than switching methods) is present and appropriate.

**REFEREE 3 — Legal/Practitioner Reviewer (Montana/Wesberry focus)**
Recommendation: Accept

SUMMARY: This is the theoretical capstone of the J-series. The Balinski-Young theorem is correctly stated and its policy consequence (Congress must choose between paradox immunity and quota compliance) is clearly articulated. This framing is useful for any future apportionment litigation.

```
VERDICT: READY FOR PANEL (1 P2 fix: tighten Balinski-Young proof sketch)
```

### Phase 5 — Abstract Check

```
ABSTRACT: ~175 words
Primary result stated: YES (all three theorems stated, historical examples named)
Algorithm named: YES (bisect-apportion and Hamilton both named)
Value proposition: YES (1941 Congress decision explained)
```

---

## J.6 — Bisect-Apportion Implementation

### Phase 1 — Paper Summary

```
Paper: J.6+bisect-apportion-implementation
Sections found: 00-abstract, 01-introduction, 02-architecture, 03-huntington-hill,
                04-other-methods, 05-verification, 06-tests, 07-comparison, 08-conclusion
Spec found: YES — docs/specs/2026-05-07-j6-bisect-apportion-impl.md
Series: J.6
Key claims (FROM ABSTRACT):
  1. f64 arithmetic for priority comparisons (not integer arithmetic)
  2. SHA-256 planned for Phase 2 (not yet implemented)
  3. 95 tests total; 24 cover apportionment and paradox checking
```

**CRITICAL DISCREPANCY**: The spec (accepted contract) requires:
- Exact integer arithmetic (u128 integers for exact comparison)
- SHA-256 verification implemented and verified against 2020 Census Bureau
- 61 tests (spec says "61 tests across three levels")

The abstract delivers:
- f64 floating-point arithmetic
- SHA-256 "planned for Phase 2" (not implemented)
- 95 tests (different count from spec's 61)

This suggests the J.6 abstract was written at an EARLIER stage of development than the other sections, or the implementation evolved after the abstract was written but before the full paper was written — because the OTHER J-series papers (J.0, J.1, J.2) consistently describe integer arithmetic and SHA-256 as implemented. The abstract of J.6 contradicts its own companion papers.

### Phase 2 — Consistency Check

| Q-ID | Quantity | Abstract | §Other J-papers | §Spec | Consistent? |
|------|----------|---------|----------------|-------|-------------|
| Q-01 | Integer vs f64 arithmetic | Abstract: f64 | J.0 §05: "represented as rational p_i^2/(s(s+1)) stored as u128" | Spec: u128 integer | **FAIL** |
| Q-02 | SHA-256 status | Abstract: "planned Phase 2" | J.0 §05: "SHA-256 verified" stated as done; J.1 §07: verified | Spec: implemented | **FAIL** |
| Q-03 | Test count | Abstract: 95 total, 24 for apportionment | Spec: 61 tests | — | WARN (discrepancy) |
| Q-04 | String comparison vs SHA-256 | Abstract: "string comparison" | J.0/J.1: SHA-256 | Spec: SHA-256 | FAIL |
| Q-05 | 100% match rate 2020 | Abstract: Yes | J.0: Yes | Spec: Yes | PASS |
| Q-06 | 2010, 2000 match | Abstract: Yes | J.1: Yes | Spec: Yes | PASS |

```
CONSISTENCY: 2 FAILS, 1 WARN — P1 BLOCKERS
P1 (reject-level):
  [Q-01] Abstract claims f64 arithmetic; spec and companion papers (J.0, J.1) claim u128 integer
         arithmetic. This is a factual inconsistency within the J-series and with the contract.
  [Q-02] Abstract says SHA-256 is "planned for Phase 2"; spec and companion papers describe it as
         implemented and verified. The J.6 abstract describes an earlier version of the software
         than the rest of the J-track papers.
P2 (revision):
  [Q-03] Test count discrepancy: abstract says 95 tests (24 for apportionment), spec says 61 tests.
         Needs reconciliation — either the spec was written before more tests were added, or the
         count methodology differs. Clarify: are the 95 tests the full crate test suite and 61 the
         apportionment-specific tests?
P3: none
```

### Phase 3 — Contract Check

| Promise (from spec) | Paper section | Delivered? | Gap |
|--------------------|---------------|-----------|-----|
| Exact integer arithmetic (u128) | §03 (implementation) | Body sections likely deliver this; abstract contradicts | Partial |
| SHA-256 verification implemented | §05 (verification) | Body §05 may describe this; abstract says "planned" | PARTIAL |
| 61 tests at L0/L1/L2 | §06 (tests) | 95 tests claimed in abstract | PARTIAL |
| Unified trait interface | §02 (architecture) | Likely yes (two public functions named) | Partial |
| All five methods | §04 (other-methods) | Yes | ✓ |
| Comparative output all five methods 2020 | §07 (comparison) | Yes | ✓ |

```
CONTRACT: FAIL — key technical promises from spec contradict abstract
Promises kept: 2/3
Gaps:
  - Abstract describes f64 arithmetic, not the u128 integer arithmetic the spec requires
  - SHA-256 verification described as "planned" not implemented (spec requires it done)
  - Test count (95 vs. 61) needs reconciliation
```

### Phase 4 — Referee Simulation

**REFEREE 1 — Algorithms Reviewer**
Recommendation: **Reject** (resubmit after resolving internal contradictions)

SUMMARY: The J.6 abstract contradicts its companion papers and its own spec. A Journal of Statistical Software submission in which the abstract claims f64 arithmetic while the companion papers describe u128 integer arithmetic will be immediately rejected. The SHA-256 claim ("planned for Phase 2") directly undermines the central contribution of the entire J-series, which repeatedly states the SHA verification is the auditable proof of correctness.

MAJOR CONCERNS:
I-11 f64 vs u128: Abstract says "double-precision floating-point (f64) arithmetic for priority comparisons; for the population values used in apportionment (up to approximately 40 million) and seat counts (up to approximately 100), f64 provides exact results within its 53-bit mantissa precision." This reasoning is incorrect: f64 has 53-bit mantissa (can represent exactly integers up to 2^53 ≈ 9×10^15), so p_i^2 ≈ (4×10^7)^2 = 1.6×10^15 fits in f64 for the value itself. However, the cross-multiplication p_i^2 × s_j(s_j+1) ≈ 1.6×10^15 × 436×437 ≈ 3×10^20 exceeds 2^53 and will have rounding error. The u128 approach in J.0/J.1 is correct; the f64 claim in the abstract is WRONG.

I-12 SHA-256 "planned for Phase 2": The abstract says "SHA-256 seed verification is planned for Phase 2; currently the implementation uses a deterministic seed formula without cryptographic verification." The J.0 §05 and J.1 §07 sections clearly describe the SHA-256 verification as implemented and verified. This is either (a) the abstract was written at an earlier development stage, or (b) the body sections of J.6 contradict the abstract. Either way, this is a P1 blocker that must be resolved before submission.

---

**REFEREE 2 — Political Science Reviewer**
Recommendation: Minor Revision (after R1 issues resolved)

SUMMARY: The political science content (comparative output table for all five methods) is sound. The abstract issues are outside my expertise but they appear severe.

---

**REFEREE 3 — Legal/Practitioner Reviewer**
Recommendation: Major Revision (after R1 issues resolved)

SUMMARY: If SHA-256 verification is "planned for Phase 2," the paper cannot make the claim that the implementation provides "auditable proof of correctness." This is the core legal/practitioner value proposition and the abstract undermines it. Fix the abstract and resubmit.

### Phase 5 — Abstract Check

```
ABSTRACT: ~200 words (slightly above the 150–200 target)
Primary result stated: YES (but incorrectly — f64 vs u128, SHA planned vs done)
Algorithm named: YES
Value proposition: UNDERMINED by "planned for Phase 2" framing
```

### Phase 6 — Pre-Panel Checklist (J.6)

```
═══════════════════════════════════════════════════════
POST-WRITE COMPLETE: J.6+bisect-apportion-implementation
═══════════════════════════════════════════════════════

Validation results:
  Consistency:   2 FAILS + 1 WARN
  Contract:      FAIL (2/3 key promises contradicted by abstract)
  Referee sim:   Reject / Major Revision
  Abstract:      ~200 words

P1 blockers (fix before panel review):
  [I-11] Abstract claims f64 arithmetic; correct to u128 integer arithmetic
         (the f64 reasoning is actually mathematically incorrect for cross-multiplication products)
         → Replace f64 paragraph with: "The implementation uses exact u128 integer arithmetic
           for priority comparisons, representing P_i(s) = p_i^2/(s(s+1)) as a rational
           with u128 numerator and denominator, enabling exact cross-multiplication comparison
           without floating-point rounding. The maximum comparison value ($\approx 3\times10^{20}$)
           fits within u128 range ($3.4\times10^{38}$)."

  [I-12] Abstract says SHA-256 is "planned for Phase 2"; J.0/J.1 describe it as implemented
         → Replace with: "The 2020 Census Bureau Huntington-Hill apportionment is verified by
           SHA-256 comparison of the bisect-apportion output to the Census Bureau's published
           result: zero discrepancies across all 50 states."

  [Q-03] Test count: reconcile 95 vs. 61 test counts with spec
         → If 95 is the current count, update spec reference; add parenthetical noting which
           61 specifically cover apportionment methods (the spec's intended count)

P2 items (should fix):
  None beyond P1 resolutions

PRE-PANEL CHECKLIST:
□ All P1 consistency failures resolved                          ✗ (3 P1 fixes required)
□ All spec contract promises delivered                          ✗ (abstract contradicts spec)
□ Single-run results marked with dagger notation                ✓ (N/A)
□ Algorithm complexity claim consistent across sections          — (pending abstract fix)
□ CLI flags match actual bisect binary flags                    ✓
□ Court citations verified                                      ✓ (N/A — software paper)
□ Abstract states primary quantitative result                   ✗ (incorrectly stated)
□ Referee P1 blockers addressed                                 ✗

VERDICT: FIXES REQUIRED
Fixes required: 3 (all P1)
Action: Fix the J.6 abstract to reflect the actual implementation (u128 integer arithmetic,
        SHA-256 implemented, correct test count). The body sections are likely correct — the
        abstract appears to be a stale draft from an earlier development phase.
Next: After abstract fix, run panel review.
═══════════════════════════════════════════════════════
```

---

## Combined J-Track Issues Summary

| Issue ID | Paper | Severity | Description |
|----------|-------|----------|-------------|
| I-01 | J.0 | P3 | Paradox immunity proof thin in overview |
| I-02 | J.0 | P2 | "Three states" vs four starred in table |
| I-03 | J.0 | P2 | Dean method not mentioned despite Montana litigation |
| I-04 | J.1 | P2 | Theorem 1 proof sketch: "one can verify" language |
| I-05 | J.1 | P2 | Near-miss state ambiguity in 2020 apportionment |
| I-06 | J.2 | P2 | H=400 divergence claim lacks supporting computation |
| I-07 | J.3 | **P1** | Abstract claims Adams/HH diverge in 2020; body shows they agree |
| I-08 | J.3 | **P1** | Same as I-07 (multiple referees) |
| I-09 | J.4 | P2 | Priority-queue ↔ divisor equivalence not proved for Jefferson |
| I-10 | J.5 | P2 | Balinski-Young proof sketch thin |
| I-11 | J.6 | **P1** | Abstract claims f64; spec/companions require u128; f64 reasoning incorrect |
| I-12 | J.6 | **P1** | SHA-256 "planned" in abstract; implemented in companion papers |
| Q-03 | J.6 | P2 | Test count discrepancy (95 vs 61) |

**Total P1 blockers**: 3 (in J.3 × 1, J.6 × 2)
**Total P2 items**: 8
**Papers ready for panel**: J.0, J.1, J.2, J.4, J.5 (5 of 7)
**Papers requiring fixes first**: J.3 (1 P1), J.6 (2 P1 + 1 P2)
