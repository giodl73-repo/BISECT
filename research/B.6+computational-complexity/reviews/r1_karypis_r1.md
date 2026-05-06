---
reviewer: George Karypis
round: 1
score: 2
date: 2026-05-05
---

## Summary

B.6 attempts to establish the complexity-theoretic foundation for METIS recursive bisection in congressional redistricting. It claims NP-hardness of the balanced k-cut redistricting problem on planar graphs, derives an O(sqrt(log n)) approximation ratio, and characterises empirical runtime scaling as O(n^1.07). The paper's goals are important and timely. However, the theoretical proofs as presented contain significant errors and omissions that prevent acceptance in their current form. The hardness reduction is incomplete, the approximation bound derivation is not self-contained, and the claimed O(sqrt(log n)) guarantee requires qualifications that the paper does not provide.

## Strengths

- **The problem formulation is correct and precise.** Problem 1 (Balanced k-Cut Redistricting) is well-stated with all three requirements (minimum edge cut, population balance, contiguity) formally defined. This is the correct formal model for the redistricting problem.
- **The empirical scaling characterisation is the paper's strongest contribution.** Table 1 provides clean, reproducible runtime data across a 40x range in n and 52x range in k, with consistent exponent b = 1.07 ± 0.03 across three census years. The OLS fit on log-transformed data is the correct approach, and R^2 = 0.984 is convincing.
- **The DIA policy connection is well-argued.** Section 5.1's argument that NP-hardness makes heuristic approximation "not merely acceptable but necessary" is legally and technically correct. This framing is useful for courts that might otherwise ask why the DIA does not specify an exact algorithm.

## Weaknesses / P1 Items (Required Fixes)

- **The NP-hardness reduction is incomplete.** Theorem 1's proof sketch reduces from Planar Graph Bisection. But the paper claims NP-hardness for the case k = 2 and unit vertex weights. The issue is that the redistricting problem requires contiguous parts (each P_i induces a connected subgraph), while Planar Graph Bisection as defined in Garey and Johnson 1976 does not require connectivity of the parts. If the parts in the bisection instance are not required to be connected, the reduction is trivially invalid for the redistricting case. The paper must either (a) cite a version of Planar Graph Bisection that requires connected parts, or (b) add a gadget to the reduction that enforces connectivity, or (c) prove that the hardness of the unconnected case implies hardness of the connected case on planar graphs. Garey, Johnson, and Stockmeyer (1976) do not prove NP-hardness for the connected variant on planar graphs — this is a non-trivial gap.
- **The approximation ratio derivation is incorrect.** Proposition 1 claims an O(sqrt(log n)) approximation via the Lipton-Tarjan separator theorem and the Arora-Rao-Vazirani (ARV) framework. The ARV framework gives O(sqrt(log n)) approximation for Sparsest Cut, not for Minimum Bisection. For Minimum Bisection on planar graphs, the Lipton-Tarjan separator gives an O(sqrt(n))-size separator, which implies an O(sqrt(n)/OPT) approximation ratio — not O(sqrt(log n)). The O(sqrt(log n)) ratio for Minimum Bisection on planar graphs requires the ARV rounding theorem applied to the planar structure, which is a non-trivial result that should be cited precisely (e.g., Arora, Rao, Vazirani 2009 Theorem 3.1) rather than stated as following directly from the separator theorem. The current proof sketch is incorrect as written.
- **The runtime theorem conflates theoretical and empirical claims.** Theorem 2 states O(n log n log k) runtime, but the empirical fit gives O(n^1.07). For n = 74,000 (all U.S. tracts), n^1.07 ≈ n * n^0.07, which for n = 74,000 gives a multiplicative factor of approximately 3.4. The O(log n log k) factor for n = 74,000 and k = 52 gives approximately log(74,000) * log(52) ≈ 16.8 * 5.7 ≈ 96. The discrepancy between the theoretical constant (96) and the empirical extra factor (3.4) is not discussed. This suggests that either the theoretical bound is loose (FM refinement does not actually run for O(log n) iterations in practice) or the empirical fit conflates n and k growth. The paper must reconcile these.

## P2 Items (Suggestions)

- **Provide full proofs in an appendix.** The current "proof sketch" format is appropriate for a short paper but not for a complexity paper making novel hardness claims. An appendix with complete proofs of Theorem 1 and Proposition 1 would allow reviewers and readers to verify the claims.
- **Test the O(n^1.07) claim at block-group resolution.** The empirical scaling is fit on tract-level data (n up to ~8,000 per state). Block-group resolution (n up to ~25,000 per state) would extend the n range by 3x and provide a stronger test of the near-linear claim.

## Score: 2 — Major Revision

The NP-hardness reduction and the approximation ratio derivation both have substantive errors. These are the paper's central theoretical claims. The empirical section is strong and would be acceptable as a standalone contribution, but the theoretical claims require significant revision.
