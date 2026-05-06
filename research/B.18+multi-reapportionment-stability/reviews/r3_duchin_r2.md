# Review 3 — Reviewer: Moon Duchin (Metric Geometry / GerryChain)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 2
**Score:** 3/4

## Summary

The revision addresses my two principal concerns: seed variance is now reported (Section 4.4), and the tree depth definition for prime-k cases is now correct (flat single-level partition, not "depth=1"). I maintain my score at 3/4 — the paper's empirical foundation is now sounder, but the ReCom comparison remains algebraically derived rather than empirically measured, and this is a genuine limitation.

## Addressed Issues

The seed variance analysis (Section 4.4) directly addresses my central concern. The paper now reports that the Hamming distances in Table 3 are measured at seed=0 with median variance of ±0.8pp across all T=600 seeds. The claim that the qualitative ordering is stable across seeds is the key result for the paper's conclusions, and it is now supported. I accept this as an adequate response to my request for multi-seed distributions, given that the ±0.8pp stability bound is tight relative to the differences between states (d_Ham ranges from 0.08 to 0.23).

The tree depth definition for prime-k cases is now corrected. The paper uses "flat" to distinguish prime-k single-level partitions from hierarchical composite-k trees. The footnote clarifying that depth=0 (or "flat") is distinct from depth=1 binary trees is technically precise and resolves my confusion concern.

The prime count in [3,60] is now exactly stated (16 primes, 27.6% density), addressing my minor issue.

## Remaining Concerns

The ReCom baseline comparison (Texas reapportionment ≈ 9 ReCom steps) is still algebraically derived rather than empirically measured. My Round 1 request was to run a ReCom chain starting from the AR(k=38) Texas plan and measure the number of steps T* at which average Hamming distance equals 0.23. This would convert the algebraic approximation into an empirical measurement. The paper still uses 0.23/0.026 ≈ 9, which treats the expected step size as a constant.

I understand this is computationally expensive (a 100-step ReCom chain for Texas at k=38 would require significant time). The paper does not provide the empirical measurement. I accept the algebraic approximation as adequate for the paper's purposes, but a footnote should acknowledge that the actual number of steps depends on the ReCom step size distribution for k=38 Texas, which has variance.

The New York case (26=2×13 → 25=5²) is described as "two-level (5-5)" tree in 2030. The explanation is correct but could be clearer: 5² means the 25-seat tree bisects to 5 sub-problems of 5 seats each, and each 5-seat sub-problem is a flat prime partition. The paper's description is technically accurate but the 5² notation in Table 3 is slightly ambiguous about whether this is five halves of five or something else. A clarifying sentence in Section 3.4 would help.

## Minor Issues

- The open question on prime frequency (Section 7.3) is now appropriately brief.
- The abbreviations k₂₀ and k₃₀ are now defined at first use in Table 3's caption, resolving my Round 1 minor concern.

## Recommendation

Accept. The seed variance analysis and the tree depth clarification are the critical improvements. The ReCom comparison remains algebraic, which is a limitation I accept as honest rather than blocking.
