# Review: R-37 Nadia Polikarpova
**Paper**: The Solution Space of Minimum-Edge-Cut Redistricting: Seed Sensitivity and Partisan Variance
**Date**: 2026-05-01
**Score**: 3.0 / 4

---

## Summary

The Fiedler certificate is a genuinely original legal-technical contribution: a mathematical proof object that converts empirical convergence claims into verifiable guarantees. The formal structure is mostly sound but Proposition 3.1 as stated is not proven, and the correctness specification for CompactBisect (termination with valid population balance) is incomplete. With targeted fixes, this could be the paper's load-bearing legal contribution.

---

## Strong Points

1. **The certificate mechanism is formally elegant.** The chain λ₂ → EC lower bound → PP upper bound → GMPP upper bound is algebraically valid (modulo the formula issue raised by other reviewers). The key insight — that a verifier can check the bound in O(1) but a challenger cannot improve without O(N) work — is the correct structure for a legally-admissible certificate.

2. **The independence of the certificate from partisan data is stated precisely.** The four-point list in §3.3.4 (λ₂ depends only on Laplacian → TIGER data → no partisan signal; δ is statutory) correctly identifies what makes the certificate legally durable. This is the kind of specification that a court can actually evaluate.

3. **The CompactBisect definition (Def 3.4)** is formal enough to implement — and it has been implemented, which is stronger evidence of specification quality than most papers provide.

---

## Concerns and Weaknesses

### P1: Critical

**P1.1 — Proposition 3.1 is not proven.**

The proposition asserts Certificate Immunity: that a challenger cannot produce a bisection with GMPP exceeding (1−δ)·GMPP_upper(G). The stated "proof" is: "since GMPP_upper is a mathematical upper bound on any bisection, no challenge can succeed when δ=0." This conflates the definition of an upper bound with a proof that the bound is correct.

The formal claim requires:

For all bisections (L, R) of G with |L| ≤ |R|:
  sqrt(PP(L)·PP(R)) ≤ sqrt(PP_upper(L)·PP_upper(R))

This requires proving that PP(L) ≤ PP_upper(L) AND PP(R) ≤ PP_upper(R) for the SAME bisection. The derivation in eq. (3.1)-(3.2) bounds PP(L) via the Cheeger bound EC_min ≥ λ₂·n/4, but this bound is for the minimum bisection, not for the bisection (L, R) under consideration. For a SPECIFIC bisection, EC(L,R) ≥ EC_min ≥ λ₂·n/4, so P(L) ≥ λ₂·n/4 + P_ext(L). This does give PP(L) ≤ 4πA(L)/(λ₂·n/4 + P_ext(L))². So the bound DOES hold for each individual half.

But the second half R also needs the bound. R uses the same EC(L,R) in its perimeter, so P(R) ≥ λ₂·n/4 + P_ext(R). This holds by the same argument. So the proposition IS provable — it just needs to be written down. Please add the 4-line proof.

**P1.2 — CompactBisect termination is not specified for the Fiedler-certified variant.**

Definition 3.3 (Fiedler-Certified CompactBisect) runs seeds until `ratio ≥ 1 − δ`. There is no bound on how many seeds are needed, and no fallback when the ratio is not achievable. For graphs where the GMPP upper bound is very tight and the achieved GMPP is far from the bound (e.g., a state with a very elongated tract that dominates the external perimeter), the algorithm may run indefinitely.

The specification needs: (a) a finite seed budget B (e.g., B = 1,000), (b) a fallback: if ratio < 1−δ after B seeds, publish the best achieved plan with the certificate marked as "unachieved: ratio = r/GMPP_upper", and (c) a claim about the probability that the ratio is achievable given B seeds (this is the convergence analysis).

### P2: Significant

**P2.1 — The specification of "valid partition" is missing.**

CompactBisect must produce a partition satisfying: (a) every tract assigned to exactly one district, (b) population balance within 0.5%, (c) contiguity. The definition says "return {V} as a single district" for k=1 and recurses, but never specifies that the recursion preserves population balance. If the PP-maximising split violates balance (METIS's balance constraint is per-level, not global), the algorithm silently produces an invalid plan. The specification needs a REQUIRES clause: "Precondition: METIS candidates satisfying population balance are available."

**P2.2 — The external perimeter approximation (2√(πA) − Σ edges) should be formally stated as an assumption.**

The implementation computes external perimeters via the circular approximation. The Fiedler certificate proof (Proposition 3.1) implicitly assumes that P_ext is exact. If the approximation underestimates P_ext, then PP_upper is tighter than it should be (making the certificate harder to achieve). If it overestimates, PP_upper is looser (making the certificate easier to achieve but less meaningful). The paper should either: (a) state the approximation as an assumption and bound its error, or (b) use exact TIGER perimeters (from the shapefile geometry, not the circular approximation). Given that TIGER shapefiles contain exact polygon coordinates, exact perimeters are computable with negligible extra cost.

### P3: Minor

**P3.1 —** The Certification Protocol (Def 3.3) uses δ = 0.05 as a threshold but doesn't specify how δ is published or who controls it. §3.3.3 says "the certification threshold δ is a statutory parameter set in the parameter table." This should be formalized: δ is immutable once published (not a runtime parameter), analogous to the fixed seed in the current proposal.

**P3.2 —** The proof of Proposition 3.1 needs to handle the degenerate case where P_ext(L) = 0 (a subgraph whose external perimeter is zero — possible if a subset of tracts is entirely interior with no boundary touching the state border). In this case, PP_upper = 4πA/(λ₂·n/4)² which is finite and the bound still holds, but it should be stated explicitly.

---

## Verdict

The Fiedler certificate is the right kind of contribution for this domain: a verifiable proof object rather than just an empirical claim. The gaps are fixable: Proposition 3.1 needs a 4-line proof (which the paper almost contains already), and CompactBisect needs a termination specification. I would accept this paper after minor-to-moderate revisions. The formal specification quality is above average for a systems/algorithms paper targeting a political science venue.
