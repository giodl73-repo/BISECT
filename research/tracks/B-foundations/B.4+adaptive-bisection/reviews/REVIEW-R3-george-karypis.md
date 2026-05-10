> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied.

---

# Review Round 3: Edge-Weighting Makes Method Selection Irrelevant

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: Graph partitioning algorithms, METIS development, multi-constraint optimization
**Round**: 3 (Final revision review)
**Date**: 2026-05-05

## Summary of Round 2 Remaining Issues

In Round 2, I gave this paper 3.5/4 and identified four minor issues: (m1) METIS random seed and determinism not fully specified; (m2) Fiduccia-Mattheyses (1982) not cited despite the Theorem 1 proof directly using FM-style local search logic; (m3) the coarsening phase is not discussed in the theory — only KL refinement; (m4) the discrepancy between the theoretical α_crit ∈ [11,38] and the empirical transition at [20,50] needs cleaner reconciliation.

## Assessment of Revisions

### METIS Seed Specification — Addressed

Section 5 (Experimental Setup) now explicitly states that METIS is run in deterministic mode with a fixed seed (-seed 42), and explains that this choice is justified by the zero-variance finding: since outcomes are deterministic from (algorithm, parameter, state) inputs, the seed choice does not affect results. This is the correct explanation and resolves my concern.

### Fiduccia-Mattheyses Citation — Addressed

The FM (1982) citation is now present in the Theorem 1 proof sketch, correctly noting that the KL refinement argument is a variant of FM-style vertex moves. The connection is appropriate and accurate: FM partitioning exactly formalizes vertex moves that reduce cut by moving vertices across boundaries, which is the argument used in the Theorem 1 proof.

### Coarsening-Phase Theory — Addressed

The theory now includes a sentence acknowledging that METIS's heavy-edge matching in the coarsening phase also benefits from high edge weights: for α ≥ α_crit, minority-minority connected vertices will preferentially merge in the coarsening phase, making the initial partition already near-optimal before refinement begins. This makes the algorithmic story complete — both phases (coarsening and refinement) are guided by edge weights, not just refinement.

### α_crit Reconciliation — Addressed (with one note)

The paper now includes a Discussion paragraph synthesizing the three α values: (1) theoretical structural threshold α_crit ∈ [11,38] from the Corollary bounds; (2) empirical transition to zero variance at α ∈ [20,50] from the ablation study; (3) working value α = 5, which achieves empirical equivalence on the five original test states but is below the theoretically safe threshold. The paper now explicitly recommends α ≥ 20 for robust production use, noting that α = 5 may be adequate for states satisfying conditions C1-C3 but that α = 5 cannot be guaranteed from theory alone.

This reconciliation is correct and complete. My one remaining observation: the original claim in Section 5 that "empirical α_crit ∈ [3,5]" should be explicitly retracted or updated, since the ablation data shows that [3,5] is not where zero variance occurs — it is where near-zero variance occurs on the original five states. The current text still includes the [3,5] claim in one location.

## Score

**Score: 4/4 — Accept**

Upgraded from 3.5/4 in Round 2. All four minor issues are addressed. The coarsening-phase sentence and the Discussion synthesis paragraph are the most important additions — together they make the theoretical contribution complete and the practical guidance clear. The FM citation is appropriate. The remaining issue (one vestigial [3,5] claim) can be fixed in copy-editing without further review.

**Recommendation**: Accept. The paper is ready for submission to SIAM SISC or INFORMS Journal on Computing.
