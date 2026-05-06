# REVISION PLAN — F.0: Algorithmic State Legislative Redistricting: A Research Program
**Round**: R1 → R2
**Date**: 2026-05-05

## Score Summary

| Reviewer | Score | Recommendation |
|----------|-------|----------------|
| Karypis | 3/4 | Accept with minor revisions |
| Rodden | 3/4 | Accept with minor revisions |
| Duchin | 3/4 | Revise and resubmit |
| Stephanopoulos | 3/4 | Accept with revisions |
| Liang | 2/4 | Major revision required |
| **Mean** | **2.8/4** | |

## Critical Issues (Must Fix Before Resubmission)

### C1 — Nesting count inconsistency (Duchin C2, Karypis implicit)
**Problem**: F.0 claims "42 states" support compatible NestSection spine. F.2's Table 1 lists 9 states with gcd=1, giving 40 compatible states. The abstract, body, and F.2 are inconsistent (42 vs. 40 vs. "7 incompatible" vs. "9 listed").
**Action**: Count states in F.2's Table 1 directly. Reconcile to one consistent number across F.0 and F.2. If 40 is correct (9 incompatible), update F.0 abstract and key-findings section.

### C2 — California nesting constitutional claim (Stephanopoulos C3)
**Problem**: Paper states California's constitution "requires that 'each Senate district shall consist of two complete, contiguous Assembly districts.'" This is pre-Proposition 11 (2008) text; the CRC draws chambers independently under current law.
**Action**: Remove or correct the California quote. Replace with accurate current description: CRC draws house and senate maps independently under priority ordering of criteria.

### C3 — Karcher citation in population balance section (Stephanopoulos C1)
**Problem**: Karcher v. Daggett (1983) addresses the Wesberry standard for congressional districts, not the Reynolds ±5% tolerance for state legislative plans.
**Action**: Remove Karcher citation from the Reynolds/state-legislative context. Add correct citations: Mahan v. Howell (1973) and Brown v. Thomson (1983) for the state legislative tolerance standard.

### C4 — Block-level deferral not quantified for NH (Karypis C1)
**Problem**: NH at k/n_bg = 0.57 violates the 20-units-per-district heuristic by a factor of ~11 but results are presented without caveat.
**Action**: Add explicit note in Section 5.2 (compactness findings) that NH, WY, VT, SD, ND results should be interpreted with caution because block-group resolution is itself suboptimal for these states.

## Important Issues (Should Fix)

### I1 — TN parenthetical self-correction in text (Karypis C implicit)
**Problem**: Section 4.2 contains "TN 99:33 ... correction: TN 99:33 is 3:1" as in-text self-correction.
**Action**: Remove parenthetical. State directly: Tennessee (H=99, S=33, 3:1 ratio, gcd=33).

### I2 — No partisan outcome preview in Section 5 (Rodden C1)
**Problem**: Section 5 previews compactness, balance, runtime, nesting — but not partisan outcomes.
**Action**: Add brief paragraph in Section 5 noting that partisan outcomes are examined in F.2 (Section 6) using Kuriwaki precinct data; summarise the key finding (nesting changes outcomes by ≤0.3 seats; independent maps show modest Democratic seat advantages in most states).

### I3 — PP scale mechanism confusion (Duchin C1)
**Problem**: Section 5.1 conflates scale-invariance of PP with the source of the compactness advantage. The mechanism (boundary fraction decreasing as O(1/√k)) is not stated.
**Action**: Add one-sentence citation to F.5's O(1/√k) result: "As F.5 derives, the fraction of boundary districts decreases as O(1/√k), which is the operative mechanism for the observed compactness advantage."

### I4 — VRA preview absent from Section 5 (Stephanopoulos C4)
**Problem**: Section 5 does not preview F.6's VRA findings.
**Action**: Add brief paragraph: 42% threshold holds at state house scale in 4 of 5 (or 5 of 5 — pending C1 resolution) covered states; compactness penalty for majority-minority districts is smaller at state house scale (5.7% vs 10.3% at congressional).

### I5 — Vermont missing from Table 1 (Karypis C4)
**Problem**: Vermont (k/n_tract = 0.81, a prominent edge case) absent from Table 1.
**Action**: Add Vermont row to Table 1 (VT, House, 150, ~186, 0.81).

## Minor Issues (Can Fix in Proofing)

### M1 — Trifecta count temporal reference (Rodden C3)
**Problem**: "In 2020, Republicans held trifectas in 23 states; Democrats in 15" refers to pre-election 2020 figures.
**Action**: Clarify: "As of November 2020 elections, before the 2020-cycle redistricting took effect..."

### M2 — Data availability statement needs specificity (Liang C3)
**Problem**: "Block-group adjacency data for the states studied in F.1--F.3 are available at the project repository" is vague.
**Action**: Specify which states, which resolution, approximate file sizes, and the command required to generate them.

### M3 — Runtime scaling degenerate-regime claim needs citation (Karypis C2)
**Problem**: O(n^2·log n) claim when k/n → 1 stated without citation.
**Action**: Add footnote: "Full derivation deferred to F.3 [Section X]; result consistent with METIS documentation for near-degenerate partitions."

## Priorities for R2

1. Fix nesting count inconsistency (C1) — fundamental numerical error
2. Fix California constitutional claim (C2) — legal error
3. Fix Karcher citation (C3) — legal citation error
4. Add NH caveat (C4) — methodological transparency
5. Address issues I1--I5
6. Minor issues M1--M3 as time allows
