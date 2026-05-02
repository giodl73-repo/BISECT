# Review Round 2: R-50 Jeffrey D. Ullman
**Paper**: Solution Space of Minimum-Edge-Cut Redistricting
**Date**: 2026-05-01
**Score**: 3.0 / 4

---

## P1 Item Resolution

**P1-A (Cheeger formula)**: Fixed. The paper now correctly states EC_min ≥ λ₂×n/4. The code was always correct; the paper now matches. The derivation via h(G) ≥ λ₂/2 → EC_min ≥ λ₂n/4 is sound.

**P1-B (Proposition 3.1 proof)**: Fixed. The 4-line proof is present and correct. The chain: EC(L,R) ≥ EC_min ≥ λ₂n/4 → P(L) ≥ λ₂n/4 + P_ext(L) → PP(L) ≤ PP_upper(L), same for R, product gives GMPP bound. Reformulation as Proposition + Corollary is cleaner.

**P1-C (Algorithm termination)**: Not fully addressed. CompactBisect still has no finite seed budget bound in the Fiedler-certified variant (Definition 3.3). The definition says "run seeds until ratio ≥ 1−δ" but the termination fallback is undocumented. A maximum seed budget B with a documented fallback is required.

---

## New Concerns

**N-1 (PA evaluation section stale)**: The main PA table (Table 2) shows 100-seed data (seeds 1-100, last improvement at seed 34, EC=2529km). But the paper's conclusion and Introduction cite 1,100-seed PA results (last improvement seed 181, EC=2441km). This inconsistency will confuse readers. Update Table 2 to the 1,100-seed result.

**N-2 (Fiedler bound non-certifying)**: The revised §3.3 now honestly acknowledges that λ₂ = 7-46m for US census-tract graphs, giving bounds of 6-26km against achieved cuts of 275-925km. This is mathematically correct and scientifically honest, but it substantially weakens the paper's key "certificate" contribution. The Fiedler certificate is now presented as theoretical context for future work, not as a practical contribution. The abstract and introduction should be updated to reflect that the certificate is a theoretical contribution, not an empirical one.

**N-3 (Complexity claim)**: §3.3 says λ₂ can be computed "in O(n²) time via the Lanczos algorithm". With the shift to scipy ARPACK, the practical complexity is O(n × k) per Arnoldi step for k eigenvalues. "O(n²)" is now inaccurate — delete or replace with "O(n × k) ARPACK steps, milliseconds for census-tract graphs."

---

## Verdict

Substantial improvement. The two critical math errors (formula, proof) are fixed. The Fiedler limitation is honestly disclosed. Two remaining issues: stale PA table (easy fix) and termination specification (P2-B from last round). Score rises to 3.0. Will accept at 3.5 if N-1 and P1-C are addressed.
