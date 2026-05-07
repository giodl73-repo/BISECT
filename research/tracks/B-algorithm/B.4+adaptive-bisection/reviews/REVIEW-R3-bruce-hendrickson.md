> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Review Round 3: Edge-Weighting Makes Method Selection Irrelevant

**Reviewer**: Bruce Hendrickson (Sandia National Laboratories)
**Expertise**: Graph partitioning theory, recursive bisection algorithms, spectral methods
**Round**: 3 (Final revision review)
**Date**: 2026-05-05

## Summary of Round 2 Remaining Issues

In Round 2, I gave this paper 3.5/4 and noted three minor issues: (m1) the α_crit formula requires knowing P* in advance, limiting practical utility without estimation guidance; (m2) METIS parameters still not fully reported; (m3) the discrepancy between α_crit theoretical prediction and empirical ablation finding needs cleaner discussion. I also noted that the paper needed to be consistent about the α_crit claim (the earlier [3,5] claim versus the Corollary's [11,38]).

## Assessment of Revisions

### α_crit Practical Estimation — Addressed

The paper now adds one sentence in the Theorem 1 discussion: "In practice, |E_r| (total regular boundary edges in the input graph) is a conservative upper bound for |E_r*|, giving a computable overestimate of α_crit that can be computed in O(|E|) time before partitioning." This is exactly what I requested — it makes the formula usable by practitioners without first solving the problem.

### METIS Parameters — Addressed

Section 5 now fully specifies: METIS 5.1.0, -ufactor=5, -contig, -minconn, -seed=42, -ncuts=1, deterministic mode. Combined with the experimental protocol appendix (P3.1), the reproduction specification is complete.

### α_crit Reconciliation — Addressed

The Discussion now contains a synthesizing paragraph (as requested by Teng as well) that presents three α values consistently: (1) structural threshold α_crit ∈ [11,38] from the Corollary; (2) empirical phase transition to zero variance at α ∈ [20,50] from the ablation; (3) working value α = 5 achieving near-zero variance on the five test states. The paper explicitly retracts the earlier informal [3,5] claim as an underestimate based on limited experiments, replacing it with the empirically validated [20,50]. This is scientifically honest and resolves the inconsistency I flagged.

### One Remaining Issue (Minor)

The Proposition Part 4 (near-optimality bound Obj ≤ Obj*(1 + C₁/α)) still contains the assumption that population balance can be achieved by perturbing at the Fiedler sign boundary. Teng flagged this in Round 2 as an assumption not implied by C1-C3 alone. The paper now adds the sentence: "For the census tract graphs studied here, this perturbation is always feasible due to the fine population granularity of census tracts; in general, the bound holds modulo a feasibility assumption on balance-adjusting moves." This is the appropriate hedge — it correctly limits the claim to the studied instances without invalidating the practical result.

## Strengths (Maintained)

The eigenvalue gap proposition remains the paper's most technically elegant contribution. Parts 1-4 of the proof are sound, and the two-indicator construction for the λ₃ upper bound is exactly the argument that explains why the eigenvalue gap is structural (a consequence of the minority subgraph's connected components) rather than accidental. This result will be of independent interest to the graph partitioning theory community.

## Score

**Score: 4/4 — Accept**

Upgraded from 3.5/4 in Round 2. All three minor issues are addressed. The α_crit practical estimation note, the complete METIS specification, and the Discussion synthesis paragraph together make the paper ready for SIAM SISC or INFORMS Journal on Computing. The Proposition Part 4 assumption hedge is appropriately scoped. I am satisfied with the theoretical contributions and the empirical validation.

**Recommendation**: Accept. The paper is publication-ready.
