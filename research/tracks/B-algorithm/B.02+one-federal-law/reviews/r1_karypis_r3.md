---
reviewer: George Karypis
round: 3
score: 3
date: 2026-05-05
---

## Summary

Round 3 adds a structural isomorphism limits paragraph and updates the VRA mutex scope. Neither change directly addresses my carry-forward P1 items (ubvec tolerance inconsistency, METIS k-way specification). Score remains at 3. The new isomorphism paragraph improves conceptual clarity.

## R3 Changes: Assessment

**Structural isomorphism paragraph (Section 2, Premise 4).**
The new paragraph correctly identifies the key distinction: HH's priority rule in the intrastate context applies to the bisection tree structure, not to the identification of specific census tracts. The "HH for tree structure, METIS for contiguous realization" division of labor is a clear and accurate description of how AR works. The statement that "the isomorphism is in the decision procedure (priority sequence → factorization → tree), not in the feasible set (states vs. contiguous connected subgraphs)" is a precise characterisation that addresses Rodden's P1.1 concern at the conceptual level.

From an implementation standpoint, this also makes clear that the intrastate HH priority rule is not applied at the census-tract level (P_tract(n) = ?) but at the region level (which split ratio to use at each level of the tree). This is computationally well-defined and does not require defining a census-tract priority value. The "HH for tree structure" interpretation is what makes AR computable.

**Proposition proof revision (Section 3).**
The revised proof adds: "The adjacency constraint does not introduce a practitioner choice at the level of tree selection — it constrains the feasible set of realizations within which METIS finds the minimum-edge-cut partition." This is a correct and important clarification. The proof now correctly separates two levels of decision-making: the tree structure (HH-determined, no practitioner choice) and the contiguous realization (METIS minimum-edge-cut, no partisan input). The chain from "HH priority rule → bisection tree → METIS contiguous realization" is now logically complete without the circularity Duchin identified.

**VRA mutex scope (Section 5, Component 5).**
The new paragraph on scope correctly distinguishes Callais's in-run prohibition from sequential comparison. The statement that "Callais prohibits combining VRASection and ProportionalSection in a single run" while "the sequential, independent comparison of a VRASection map against a ProportionalSection map is expressly contemplated by the majority opinion as the Callais strong-inference test" is the right legal interpretation. The implementation (single-run mutex gate) is correctly scoped.

## Remaining P1 Items (unchanged from R2)

**P1.3 — ubvec tolerance inconsistency: NOT ADDRESSED.**
The paper still states `ubvec[0]=1.001` imposes a "±0.05% deviation ceiling." The correct interpretation is ±0.1% per part, giving a maximum inter-district deviation of ≤0.2%. This numerical inconsistency must be corrected before the statute can be adopted — legislative drafting errors in population-balance specifications have litigation consequences. This is a P1 carry-forward.

**P1.2 (Karypis original) — METIS k-way specification: NOT ADDRESSED.**
The implementation description still does not specify ubvec and tpwgts settings for the k-way METIS calls (p_i > 2). For California (4-way first split, then 13-way), the 13-way call dominates runtime and its settings are unspecified. P1 carry-forward for journal submission.

## Score: 3 / 4 — Minor Revision

The isomorphism paragraph and proof revision are genuine improvements that address Rodden's structural objection at the conceptual level. My two remaining P1 items are technical specification issues that require precise numerical corrections; they do not affect the paper's constitutional argument but matter for statutory drafting accuracy.
