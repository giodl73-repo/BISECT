> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Review Round 3: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning algorithms, METIS development, multi-constraint optimization
**Round**: 3 (Final revision review)
**Date**: 2026-05-05

## Summary of Round 2 Remaining Issues

In Round 2, I gave this paper 3/4 and asked for: (1) a supplementary appendix with complete METIS command-line invocations; (2) rewriting the abstract and Section 5.1 to lead with state-level evidence (80% vs 40%) rather than the marginal config-level p-value (p=0.039); (3) a more satisfying explanation of the Georgia non-monotonic ubvec anomaly.

## Assessment of Revisions

### Reproducibility Appendix — Addressed

The paper now includes a supplementary appendix with complete METIS invocation templates. The specifications provided are sufficient for independent replication: METIS version (5.1.0), flags (-ufactor=5, -contig, -minconn, -seed=42), and the multi-constraint tpwgts calculation formula are all present. I can now verify the experimental setup from the paper alone. This resolves my primary remaining concern.

### Abstract Restructuring — Addressed

The abstract now leads with "multi-constraint completely fails in 3 of 5 states across all 28 tested parameter values" before citing the config-level χ² result. This is the correct ordering: the state-level pattern (80% vs 40%, complete failure in AL/LA/SC) is the stronger evidence and should be the headline. The χ² test is now correctly positioned as supporting context rather than the primary claim.

### Georgia Anomaly — Partially Addressed

The paper adds one paragraph discussing the non-monotonic Georgia result (7 MM at ubvec=1.3, 5 MM at ubvec=1.5). The proposed explanation — that at ubvec=1.5, the loosened minority tolerance interacts with METIS's coarsening phase to allow merging minority and non-minority tracts earlier in the hierarchy, reducing concentration in any single district — is plausible and mechanistically coherent. However, it remains a post-hoc explanation without empirical validation (e.g., inspection of METIS's coarsening decisions at different ubvec values, or a controlled experiment varying only ubvec on a simplified graph).

I accept this explanation as adequate for the current paper, noting that a definitive resolution would require deeper algorithmic introspection than is typically expected in application papers. The explanation is correctly framed as "hypothesis" rather than "proof."

## Remaining Issues

**p=0.039 framing**: The abstract restructuring helps, but Section 5.1 still refers to χ²(1)=4.243, p=0.039 in the first paragraph as the primary statistical evidence before the state-level discussion. The ordering should be reversed in Section 5.1 as well. This is a one-sentence reordering.

**Mechanism validation**: The constraint conflict mechanism remains empirically supported but not causally validated. I noted this in Round 2 and accept the limitation as a scope issue for this paper. The paper should add one sentence in the Discussion: "Definitive mechanism validation would require controlled ablation experiments isolating the constraint conflict from METIS's internal heuristics; we leave this to future work." This is the appropriate scientific hedge.

## Strengths (Maintained)

The paper's core contribution remains strong: the 140-vs-140 balanced comparison producing the state-level finding (80% vs 40%, complete MC failure in AL/LA/SC) is a credible and important empirical result. The SD=0 finding across 30 seeds per state is particularly valuable — it confirms that the state-level outcomes are fully determined by algorithm and parameter choices, not by random initialization. The implementation bug correction, which paradoxically strengthened the paper's conclusions, demonstrates appropriate scientific rigor.

## Score

**Score: 3.5/4 — Accept with Minor Revisions**

Upgraded from 3/4 in Round 2. The reproducibility appendix and abstract restructuring address my two primary remaining concerns. The Georgia anomaly is handled as well as can be expected without additional algorithmic introspection experiments. The remaining issue (Section 5.1 ordering) is a one-sentence fix.

**Recommendation**: Accept with minor revisions. The paper is ready for publication at an applied algorithms venue (ALENEX, SIAM SISC, ACM JEA) after the Section 5.1 ordering fix and the mechanism validation hedge sentence.
