# REVISION PLAN — B.18: Multi-Reapportionment Stability
**Round 1 Panel Scores:** Karypis 3/4 | Rodden 2/4 | Duchin 3/4 | Stephanopoulos 3/4 | Liang 3/4
**Average:** 2.8/4
**Status:** Accept with revisions (target ≥ 3.0 after R2)

## Critical Issues (Must Fix Before R2)

### C1 — Seat Projection Uncertainty and ±1 Sensitivity [Stephanopoulos, primary; Karypis secondary]
The seat projections (TX: 38→41, CA: 52→50, FL: 28→29, NY: 26→25) are presented as point estimates without uncertainty ranges. The Florida projection is especially sensitive: if FL lands at 30 instead of 29, the paper's "composite-to-prime" characterization is wrong (30 = 2×3×5 is composite), and the d_Ham estimate would be different.
**Action:** For each projected seat count, add ±1 sensitivity analysis showing: (a) the factorization at N-1 and N+1, (b) whether the tree-type characterization (composite/prime) changes at ±1, and (c) the expected d_Ham range. At minimum, provide this analysis for Florida (28→29 vs. 28→30) and California (52→50 vs. 52→51). Add a paragraph acknowledging projection uncertainty in Section 2.3.

### C2 — Multiple Seed Hamming Distance Distributions [Liang, primary; Duchin secondary]
The d_Ham values in Table 3 are single-seed point estimates. Each value has variance across seed pairs; the claimed "d_Ham = 0.23 for Texas" may not be a stable central estimate.
**Action:** Run 10 independent seed pairs for each state with projected seat changes and report mean ± SD Hamming distances. If the budget allows, run 20 pairs for the two most important cases (Texas and California). Add standard deviation columns to Table 3 (or add error bars to a figure).

### C3 — Geographic Distribution of Boundary Disruptions [Rodden, primary]
The paper's claim that algorithmic reapportionment is "less disruptive than political redistricting" ignores the geographic concentration of disruption. For Texas, the 23% tract-level disruption is likely concentrated along the I-35 corridor — the most electorally contested region of the state.
**Action:** For Texas and at minimum one other state, add analysis of where the disrupted tracts are geographically located. Report the fraction of disrupted tracts in: (a) urban cores (top 20% population density), (b) suburban rings, (c) rural areas. Also report whether disrupted tracts are disproportionately located in historically competitive vs. safe districts.

## Moderate Issues (Should Fix Before R2)

### M1 — ReCom Baseline Rigor [Duchin]
The "9-step" ReCom comparison uses the algebraic approximation d_Ham / (1/k), not an empirical measurement of how many ReCom steps produce equivalent disruption.
**Action:** Run a ReCom chain starting from the AR(k=38) Texas plan and measure the number of steps T* at which the average Hamming distance from the start plan equals 0.23. Report T* alongside the algebraic approximation. Even an approximate measurement (chain of 100 steps) would validate the order of magnitude.

### M2 — Political Redistricting Disruption Citation [Liang]
The 35–55% tract disruption figure for political redistricting is a major comparative claim without citation.
**Action:** Add empirical support for the 35–55% figure. Options: (a) cite a systematic study of tract-level changes across political redistricting cycles, (b) compute the figure from 2000→2010 and 2010→2020 redistricting for Texas and Ohio using the same Hamming metric as the algorithmic comparison.

### M3 — Tree Depth Definition [Duchin]
The paper uses "depth=1" for prime seat counts but a prime k actually produces a flat k-way partition (not a binary tree). The depth terminology needs clarification.
**Action:** Add a formal definition of "tree depth" in Section 2.2 that distinguishes between: (a) binary-tree depth (number of bisection levels), and (b) the prime-k case (flat k-way partition at the root, described as "depth=0" or "flat"). Use consistent terminology throughout.

### M4 — Karcher v. Daggett Citation [Stephanopoulos]
The constitutional analysis correctly cites *Wesberry v. Sanders* but omits *Karcher v. Daggett* (1983), which requires a good faith effort at precise equality and addresses deviation justification.
**Action:** Add *Karcher v. Daggett* to the constitutional foundation discussion in Section 6.2 with a sentence explaining how the fresh-recomputation design satisfies the *Karcher* equal-population requirement.

## Minor Issues (Optional for R2)

- Clarify whether the "previous map as a useful starting point" note (Section 5.2) contradicts the DIA fresh-recomputation design (Rodden)
- Add the exact prime count in [3,60] to the prime frequency analysis (Duchin)
- Add the Huntington-Hill calculation for Texas and California projections to strengthen the projection claims (Liang)

## Target R2 Score

After addressing C1-C3 and M1-M4: expected average ≥ 3.2/4.
Rodden's 2/4 should become 3/4 with geographic distribution analysis (C3).
The seat projection sensitivity (C1) and multi-seed d_Ham (C2) will strengthen all reviewers' confidence in the empirical claims.
