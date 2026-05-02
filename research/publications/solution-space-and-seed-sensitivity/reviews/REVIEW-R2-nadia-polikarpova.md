# Review Round 2: R-37 Nadia Polikarpova
**Paper**: Solution Space of Minimum-Edge-Cut Redistricting
**Date**: 2026-05-01
**Score**: 3.5 / 4

---

## P1 Item Resolution

**P1-A (Cheeger formula)**: Fixed and correct.

**P1-B (Proposition 3.1 proof)**: Fixed. The proof is complete and correct: the 4-line chain from EC(L,R) ≥ λ₂n/4 through P(L) lower bound to PP(L) upper bound, and the product giving the GMPP bound. The Proposition + Corollary structure is cleaner than the original.

**P1-C (Termination)**: Not fully addressed. The Fiedler-certified CompactBisect (Definition 3.3) still lacks a termination specification. The definition says "run seeds until ratio ≥ 1−δ" but:
1. No maximum seed budget B is specified
2. No fallback behavior when the ratio is unachievable (and we now know the ratio is never achieved in practice — Fiedler bounds are 1.6-4.8% of achieved cuts)
3. No correctness proof that the algorithm terminates

Given that the Fiedler certificate is now acknowledged as non-certifying in practice (§3.3), the Fiedler-certified CompactBisect definition is effectively theoretical. This should be explicitly stated: "Definition 3.3 is a theoretical ideal; the practical certification uses the empirical convergence criterion of §3.2."

---

## New Strengths

**S-1**: The honest disclosure that λ₂ = 7-46m for all tested states, making the Fiedler bound non-certifying in practice, is scientifically rigorous. Many papers would have buried this finding. Keeping it as theoretical context while using empirical convergence for practical certification is the correct approach.

**S-2**: Proposition 3.1 is now a proper proposition with a complete proof. The corollary structure correctly separates the mathematical bound from the practical implication.

---

## Remaining Concerns

**R-1 (Definition 3.3 needs practical annotation)**: Since the Fiedler-certified CompactBisect is not practically achievable for US graphs, the definition should note this: "In practice for US census-tract graphs, the ratio (1−δ) is not achievable (see §3.3.2); the practical certification protocol is the empirical tail criterion (§3.2)." Without this note, readers will be confused about when to use Definition 3.3 vs. the empirical criterion.

**R-2 (CompactBisect termination for balance)**: The REQUIRES clause is still missing from CompactBisect: what happens if no METIS seed satisfies both population balance and high GMPP? The algorithm silently falls back (in the implementation) but the definition doesn't say so.

---

## Verdict

The mathematical core is now correct and the honest treatment of Fiedler tightness is admirable. Two annotation issues remain but are editorial-level fixes. Score rises to 3.5. Ready for submission after fixing R-1.
