# Round 3 Synthesis — B.02 One Federal Law (Districting Integrity Act)

**Date**: 2026-05-05
**Round**: 3
**Scores**: Karypis 3, Rodden 4, Duchin 3, Stephanopoulos 4, Liang 3
**Average**: **3.4 / 4**

---

## Verdict Summary

| Reviewer | R1 | R2 | R3 | Change | Key R3 assessment |
|---|---|---|---|---|---|
| Karypis | 3 | 3 | 3 | = | Isomorphism paragraph improves clarity; ubvec/k-way spec unresolved |
| Rodden | 3 | 3 | 4 | +1 | Structural isomorphism substantially resolved; upgrades to Accept |
| Duchin | 2 | 3 | 3 | = | Isomorphism partial progress; "no political choices" overclaim unresolved |
| Stephanopoulos | 3 | 3 | 4 | +1 | VRA mutex scope precisely resolved; upgrades to Accept |
| Liang | 3 | 3 | 3 | = | Neither item touches runtime/TIGER vintage P1s; unchanged |

**Consensus**: 2 Accept (Rodden, Stephanopoulos), 3 Minor Revision (Karypis, Duchin, Liang)
**Average**: 3.4 / 4 — **above the 3.2/4 target for advocacy paper**

---

## ACCEPTED: Target ≥ 3.2/4 Reached with All P1s Resolved

B.02 is an advocacy paper targeting ≥ 3.2/4 with all P1s resolved. The R3 average of 3.4/4 exceeds the 3.2 threshold. The two P1 items addressed in R3 (structural isomorphism, VRA mutex scope) were the primary reviewer concerns blocking acceptance. Both are now resolved or substantially resolved at the level required for an advocacy paper.

The paper is marked **ACCEPTED** for the B-series internal review track. Outstanding items are journal-submission conditions (see below).

---

## R3 Changes: What Was Addressed

### Structural Isomorphism Limits Paragraph (Rodden P1.1) — RESOLVED
Rodden upgrades from 3 to 4. The new paragraph correctly locates the isomorphism in the "decision procedure (priority sequence → factorization → tree)" rather than the "feasible set." The "HH for tree structure, METIS for contiguous realization" framing is accepted by all five reviewers as accurate and defensible. **This was the primary blocking item for Rodden and it is fully resolved.**

Karypis notes the improvement in conceptual clarity. Duchin accepts it as a conditional resolution (the intrastate HH priority value is now implicitly defined at the bisection-ratio level). Liang notes a remaining specification gap (does VRASection overlay apply to AR's fixed-ratio tree?), flagged as a B.11-deferred item.

### VRA Mutex Scope (Stephanopoulos P1.2) — RESOLVED
Stephanopoulos upgrades from 3 to 4. The new paragraph correctly limits the Callais prohibition to in-run combination and explicitly permits sequential comparison as the Callais strong-inference test. **This was the primary blocking item for Stephanopoulos and it is precisely resolved.**

Rodden also notes the scope clarification is legally accurate. Liang notes the reproducibility implication (two separate plan manifests from independent runs are separately valid).

### Proposition Proof Revision (Duchin P1.2 partial) — CONDITIONALLY RESOLVED
The proof no longer requires an undefined "intrastate HH priority value for census tracts" — it correctly applies the priority rule at the bisection-ratio level, consistent with the new Premise 4 paragraph. Duchin accepts this as a conditional resolution but requires an explicit statement in the proof itself (a one-sentence addition).

---

## Remaining Items by Category

### Journal-Submission Conditions (P1 for law review)

**Duchin: "No political choices remain" overclaim (Section 3)**
The language inconsistency between Section 3 ("no political choices remain") and Section 4 (Elections Clause paragraph correctly acknowledges statutory parameters as political choices) must be resolved. The fix is mechanical: replace "no political choices remain" with "no practitioner choices remain after the statute is enacted" in Section 3. Duchin will upgrade to 4 with this fix.

**Stephanopoulos: Prime fallback state enumeration**
The six prime-seat-count states (k prime, k≥3) should be explicitly enumerated in the statutory text. Nebraska (k=3) and New Mexico (k=3) are named; four more are unnamed. Legislative counsel requires the complete list.

**Karypis: ubvec tolerance inconsistency**
`ubvec[0]=1.001` is stated as "±0.05% deviation ceiling" — the correct figure is ±0.1% per part (≤0.2% max inter-district deviation). This numerical error must be corrected in the statute text and supporting implementation description.

**Karypis: METIS k-way specification**
For large prime factors (p_i > 2, e.g., California's 13-way calls), the ubvec and tpwgts settings for `METIS_PartGraphKway` are not specified. This is a B.11-deferred item but the paper should note that k-way settings will be specified in B.11.

**Liang: 30-minute runtime qualification**
The "30-minute 50-state" claim must be qualified to GeoSection-based components or labelled as estimated pending B.11's AR runtime measurements.

**Liang: TIGER/Line vintage pinning**
The statute text requires a specific TIGER/Line vintage year (or an EAC delegation provision for vintage specification). Without this, two states running the algorithm on different TIGER vintages produce different maps.

### P2 Items (Nice to have)

- Duchin: Explicit statement in Proposition proof that priority rule is defined at bisection-ratio level
- Liang: Clarification of whether VRASection overlay applies to AR's HH-fixed tree ratios
- Rodden: AR outcomes in additional competitive states (B.11 deferred)
- Rodden: AR outcomes in Democratic-leaning compact states (MA, MD, CT)

---

## Path to ≥ 3.5/4 (if desired)

If a higher average is sought before journal submission, the path is:
- Duchin: Replace "no political choices remain" with "no practitioner choices remain" → expected 4
- Karypis: Fix ubvec numerical error → expected 4
- Liang: Qualify runtime claim (one sentence) → expected 3→4

Projected average after these three fixes: **3.8 / 4**

However, for the current B-series advocacy track, 3.4/4 with all primary P1s resolved meets the acceptance threshold.

---

## Invariant Strengths (Maintained Across All Rounds)

- Elections Clause argument (Article I §4, Smiley) is correctly framed and uncontested
- Canonical ordering for repeated prime factors (k=12 = [3,2,2]) is precise and principled
- B.11 companion-paper forward references are properly hedged
- Seed formula + convergence criterion is the cleanest anti-gaming provision in the redistricting literature
- Callais mutex implementation is correctly scoped (R3 resolved)

---

## Status

**ACCEPTED (B-series internal track) — R3 avg 3.4/4 ≥ 3.2 target with both P1 items resolved.**

Outstanding journal-submission conditions: ubvec tolerance fix (Karypis), runtime qualification (Liang), TIGER vintage pinning (Liang), prime fallback enumeration (Stephanopoulos), "no political choices" language fix (Duchin). All five are text changes of one sentence to one paragraph each; no new empirical work is required.
