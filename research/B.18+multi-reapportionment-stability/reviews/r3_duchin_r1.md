# Review 3 — Reviewer: Moon Duchin (Metric Geometry / GerryChain)
**Paper:** B.18 — Redistricting Stability Across Reapportionment Cycles
**Round:** 1
**Score:** 3/4

## Summary

This is a technically interesting paper on an underexplored aspect of algorithmic redistricting: what happens to the bisection tree when seat counts change. The prime factorization analysis is correct and the simulation methodology is reasonable. My main concern is with the choice of the GerryChain baseline and whether the Hamming distance metric adequately captures what matters about boundary stability.

## Strengths

The prime factorization analysis is the paper's distinctive contribution. The observation that the algorithmic structure is determined by the arithmetic of the seat count — not by geography or politics — is important and well-documented in the factorization tables. The tree depth and tree type predictions (flat prime, hierarchical composite) are correct and verifiable.

The use of Hamming distance as the stability metric is appropriate for this context. The specific definition (fraction of tracts that change district assignment after canonicalization) is standard and correctly cited. The simulation design (holding geography fixed, varying only seat count) is the right way to isolate the seat-change effect from the population-redistribution effect.

The statutory note (Section 6) is well-reasoned. The argument that fresh recomputation is the correct design — not a bug — is important for the DIA framing, and the three specific DIA drafting implications (no grandfathering clause, no transition period, prime factorization is predictable) are concrete and useful.

## Weaknesses and Concerns

The GerryChain baseline is not well-defined. The paper states that "a full ReCom chain of 10,000 steps has d_Ham ≈ 1 from its start plan (approximately decorrelated)." But the mixing time of a ReCom chain depends strongly on the state, the seed plan, and the chain length. For Texas with k=38, a 10,000-step chain may not be close to decorrelated — the mixing time scales with the state complexity, and there is no reference for the claim that 10,000 steps is sufficient for Texas-scale problems.

The comparison "TX reapportionment = 10 ReCom steps" (from d_Ham = 0.23 and step size ≈ 0.026) is intuitively appealing but algebraically rough. The ReCom step size of 1/k = 1/38 ≈ 0.026 is an expected value for a uniformly random ReCom step — the actual distribution of step sizes has variance, and some steps change substantially more than 1/k tracts. Using the expected step size as if it were a constant produces an integer conversion (0.23/0.026 ≈ 9) that is more precise than the underlying calculation warrants.

A more rigorous approach would be: run a ReCom chain starting from the AR(k=38) plan, and find the number of steps T such that the average Hamming distance from the start plan equals 0.23. This T is the "empirical" equivalent number of steps, as opposed to the algebraic approximation used in the paper.

The paper does not address the distribution of Hamming distances across different starting seeds for the same seat count. The simulation runs AR(k=20) and AR(k=41) with specific seeds and computes one Hamming distance. But the Hamming distance depends on which plan you start from for k=38 and which plan you end up with for k=41, both of which depend on the random seed. The paper should run multiple seed pairs and report the distribution of Hamming distances, not just a single point estimate.

## Minor Issues

- The tree depth column in Table 1 shows depth=1 for primes and depth=2-4 for composites. The paper should be more precise about what "depth" means for a prime-k tree. If k is prime, the tree is a single k-way partition (not a binary tree at all), so "depth=1" is technically incorrect — it should be "depth=0 (flat)" or the definition of depth should be explicitly stated.
- The New York case (26=2×13 → 25=5²) is described as a "two-level (5-5)" tree in 2030. But 5² means a 25-way split at the root? No: 5² = 5×5, which means bisect to 5 halves of 5 each. The paper's description of the tree structure for 25=5² is correct in the factorization table but could be clarified in the text.
- The open question about "prime frequency" (Section 7.3) is interesting but the prime number theorem approximation (~27% near k=40) understates the relevant prime density. For congressional seat counts in the range [3,60], the exact count of prime seat counts is {3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59} = 16 out of 58 numbers, or 27.6% — close to the approximation but worth stating exactly.

## Recommendation

Accept with moderate revisions. Add multiple-seed Hamming distance distributions (not just point estimates), and use an empirical ReCom-step comparison rather than the algebraic approximation. The prime factorization analysis and the policy implications for DIA drafting are solid.
