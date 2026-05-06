---
reviewer: George Karypis
round: 2
score: 4
date: 2026-05-05
---

## Summary

The authors have addressed all three P1 items I raised in Round 1. The Theorem 3.1 proof is corrected, the Gumbel model is now properly hedged with a KS goodness-of-fit test and the independence limitation acknowledged, and the EC_norm definition is resolved with separate formulae for the recursive bisection context and the full k-way partition context. The revision is satisfactory. I am upgrading my score to 4.

## P1 Resolution

**P1.1 — Theorem 3.1 proof error (O(n^{2k}) → O(n^{2(k-1)})): RESOLVED.**
The corrected theorem now states O(n^{2(k-1)}) and the proof sketch has been revised to match. The proof correctly identifies that a recursive bisection tree has k-1 internal nodes, each corresponding to a bisection cut, and the ordered (k-1)-tuple of cuts has at most O(n^{2(k-1)}) choices. The Euler-formula-based O(n²) bound on distinct cuts per level is still stated in a slightly informal way (the citation to karypis1998 for the cut-space discussion is appropriate but not a primary reference for this result), but the exponent correction is what matters and it is now correct. The remark that the O(n^{2(k-1)}) bound is astronomically large for US state graphs (n ≤ 10,000, k ≤ 52) and the empirical evidence from B.7 is more informative remains the correct framing.

**P1.2 — Gumbel model justification: RESOLVED.**
The paper now includes a goodness-of-fit assessment: KS statistic D = 0.11 with p ≈ 0.52, indicating no significant departure from the Gumbel model at the 5% level. More importantly, the paper now explicitly acknowledges that the 50 observations are heterogeneous (different n, k, geographic structure) and not i.i.d., and frames the Gumbel fit as a "descriptive envelope" rather than a structural model. The paragraph noting that the statutory recommendation rests primarily on the empirical 89-seed margin and only secondarily on the parametric bound is exactly the right epistemic ordering. I am satisfied with this resolution.

**P1.3 — EC_norm definition for full k-way partition: RESOLVED.**
The paper now provides separate definitions for the recursive bisection context (sum of level-specific EC_norm values with sqrt(min(i, k_ℓ - i)) denominators) and the full k-way partition context (EC(Π) / sqrt(k/2)). The note that results in Table 1 identify which normalisation applies for which states is the correct implementation detail. This resolves the ambiguity that would have prevented independent implementation of Algorithm 1.

## Remaining Minor Issues

The METIS single-threaded requirement is now clearly stated in the Proposition 2.1 remark with the specific flag `METIS_OPTION_NTHREADS=1` and the explanation for why parallel METIS breaks determinism (non-deterministic work-stealing scheduler). This is an important addition that R1 missed flagging.

The j* column is now fully populated. The values for states beyond Georgia and Wisconsin are described as "read from the same sweep data," which is appropriate since j* is extractable from the B.7 sweep log. The table caption now correctly describes j* as confirmed for Georgia, Wisconsin, Florida, and Texas and drawn from the same sweep for remaining states. The single-district states (k=1) correctly show j* = 0 with the explanation that the first seed is trivially optimal.

## Score: 4 / 4 — Accept

All three P1 items are resolved. The paper's central empirical contribution (Georgia's 511-seed tail, T=500 insufficiency, T=600 adequacy) was strong in Round 1 and remains unchanged. The theoretical improvements make the paper publishable in its current form.
