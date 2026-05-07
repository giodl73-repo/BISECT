---
reviewer: Moon Duchin
round: 2
score: 4
date: 2026-05-05
---

## Summary

All three P1 items from my Round 1 review are resolved. The EC_norm definition is now precise for both the recursive bisection and full k-way partition contexts. The Theorem 3.1 proof error is corrected. The Gumbel independence assumption is explicitly acknowledged with a KS goodness-of-fit test. The paper is ready for acceptance.

## P1 Resolution

**P1.1 — EC_norm definition inconsistency: RESOLVED.**
The paper now provides the correct two-case definition that I required:
- Recursive bisection context: EC_norm is the sum of level-specific normalised cuts with denominator sqrt(min(i, k_ℓ - i)) at each bisection level ℓ.
- Full k-way partition context: EC_norm = EC(Π) / sqrt(k/2).

The new text correctly notes that "i ranges over all bisection levels in the recursive tree" was undefined for direct k-way METIS calls, and resolves the ambiguity by defining a flat normalisation for that case. The table legend specifying which normalisation applies for which states (direct-k-way for Nebraska and New Mexico, recursive bisection for all other states) is the right implementation-level specification.

I note that the choice of sqrt(k/2) as the denominator for the flat k-way case is a design choice, not a unique mathematical derivation. The paper would benefit from a brief justification (which it partially provides: "k/2 is the balance parameter that would apply to a fully balanced k-way split"). This is acceptable as a Round 2 state; a future version could derive the normalisation more formally.

**P1.2 — Theorem 3.1 proof: RESOLVED.**
The corrected theorem states O(n^{2(k-1)}) with the correct argument: k-1 bisection cuts in a recursive tree, O(n²) choices per cut, so (O(n²))^{k-1} = O(n^{2(k-1)}) total. The proof now also correctly characterises the O(n²) bound as arising from the planar separator theorem and Euler formula (via the citation to karypis1998 for the cut-space discussion). The exponent correction eliminates the mathematical debt I identified in Round 1.

**P1.3 — Gumbel independence assumption: RESOLVED.**
The paper now includes:
1. An explicit statement that the 50 observations are not i.i.d. (different n, k, geographic structure).
2. A KS goodness-of-fit test: D = 0.11, p ≈ 0.52, no significant departure from Gumbel at 5%.
3. A characterisation of the Gumbel fit as a "descriptive envelope" rather than a structural model.
4. The correct epistemic ordering: "the statutory recommendation rests primarily on the empirical bound and secondarily on the parametric bound."

This is exactly what I asked for. The p-value of 0.52 is reassuring (the Gumbel model is not rejected by the data), and the visual inspection via PP and QQ plots is described as showing "no systematic curvature in the bulk or upper tail." I am satisfied.

## Minor Observation

The version-string domain-separation byte order (Duchin R1 P2a) was raised as a P2 item. The paper does not address it in Round 2, which is acceptable for a P2 item. I note the paper correctly specifies the concatenation order in Algorithm 1 (`census_release_id || "DIA_SEED_V1"`) and I would recommend a clarifying footnote in a future revision noting that this specific order is canonical for DIA V1.

## Score: 4 / 4 — Accept

All P1 items resolved. Mathematical rigour restored. The paper is ready for publication.
