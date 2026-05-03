# Review (Round 2): Edge-Weighting Makes Method Selection Irrelevant

**Reviewer**: Shang-Hua Teng (University of Southern California)
**Expertise**: Algorithm design, graph algorithms, smoothed analysis, computational theory
**Round**: 2 (Recheck after major revision)
**Date**: 2026-05-02

## Overview of Revision

In Round 1, I requested (M1) a rigorous theoretical framework with convergence theorems and phase transition characterization, (M2) empirical validation of theoretical predictions via an α sweep, and (M3) a smoothed analysis framework. I also raised minor issues on complexity theory connections (m1), general algorithm design principles (m2), experimental reproducibility (m3), and writing/presentation (m4). My key question was: "What is the phase transition point between method-dependent and method-independent regimes?"

The revision directly answers this question. The new theory section is a substantial improvement.

---

## Response to Major Issues

### M1: Theory Formalization — SUBSTANTIALLY RESOLVED

The four-part structure of the revised theory (Definition, Theorem 1, Theorem 2 + Corollary, Proposition + Remarks) is the right architecture. Let me assess each component.

**Theorem 1 (Method Equivalence Conditions)**: The three conditions C1–C3 are well-chosen. C1 and C2 are graph-structural conditions that are verifiable in polynomial time from the input. C3 (balance feasibility — the existence of a partition P* that places each minority component in a single part) is the decisive condition; it asserts that VRA compliance is achievable without splitting any minority cluster. The proof sketch is correct: the KL refinement argument that any move cutting a minority edge costs ≥ α while the entire regular-edge budget is O(|E_r*|) ≤ |E_r| directly gives the O(1/α) convergence bound.

**Remark rem:nphard**: This is exactly the clarification I needed. I raised (m1 in Round 1) that the paper should connect to complexity theory and avoid any suggestion of polynomial-time tractability. The remark correctly states that the problem remains NP-complete even with edge weights (by reduction from unweighted balanced bisection), and that Theorem 1 establishes a *landscape property* (unique global minimum, provably good local optimum) rather than polynomial-time solvability. The connection to smoothed complexity and "easy instances of hard problems" is exactly the framing I recommended. I am satisfied with this treatment.

One technical note on the NP-hardness reduction: the remark cites Garey (1979) for the reduction, which is appropriate. However, the reduction "replace each minority edge with an edge of weight α" needs one additional word of care: the NP-hardness of *weighted* balanced bisection for general graphs does not follow from simply scaling edge weights, because the decision problem depends on whether a partition with weighted cut value ≤ C exists, and C scales with α. The standard reduction would construct the weighted instance and set C = α (requiring that the partition cut *no* minority edge), at which point the problem reduces to asking whether the minority components can be balanced — which is NP-complete by the standard argument. The remark's conclusion is correct; the sentence about the reduction could be slightly more careful.

**Proposition (Eigenvalue Gap Under Edge Weighting)**: Parts 1–4 are technically sound. Part 1 (λ_2 = Θ(α)) uses the Weyl inequality L_w ≥ L_r + (α-1)L_m, which follows directly from the semidefinite decomposition. Part 2 (λ_3 = O(1) via difference-of-indicators) is the clean argument I was hoping for — the key insight is that the indicator vector z supported on two separate minority components has z^T L_m z = 0 by C2, so the Rayleigh quotient for L_w reduces to L_r. Part 3 (Davis–Kahan) is correctly cited.

Part 4 (Obj ≤ Obj*(1 + C1/α)) is the near-optimality bound. My concern here: the constant C1 depends on |E_r| and k (stated in the proposition), but the proof sketch says "any deviation from the sign boundary to satisfy population balance cuts at most O(|E_r|/k) additional regular edges." This last step assumes that population balance can always be achieved by perturbing the Fiedler partition at the minority/non-minority boundary — which is true for graphs with sufficient connectivity but is not proven for the general case. For the specific instances studied (census tract graphs with tract populations), this is empirically true, but a sentence flagging this assumption would be appropriate.

### M2: Empirical Validation — RESOLVED

The α sweep data (α ∈ {1,2,3,4,5,7,10,20,50,100}, 430 data points) provides the empirical validation I requested. Phase transition confirmed at α ∈ [20,50]. The Corollary's prediction [11,38] for the test states is consistent with the empirical finding within the uncertainty of the simulation.

I want to revisit my key question: "What is the phase transition point?" The answer from the combined theory and experiment is: the structural threshold is α_crit = |E_r*| / (ε · |E_m*| / k), which evaluates to approximately [11,38] for the five test states; the empirical transition to zero variance (as measured by METIS on finite instances) occurs at α ∈ [20,50]; and α = 5 is empirically sufficient on these specific instances but is below the theoretically safe threshold. This is a complete and honest answer, and the paper now contains all the pieces needed to state it clearly. What is missing is a single paragraph in the Discussion synthesizing these three quantities — structural threshold, empirical threshold, and the α = 5 working value — and explaining why practitioners should use α ≥ 20 for robustness.

### M3: Smoothed Analysis — RESOLVED

Section 4.5 (Smoothed Analysis) with Theorem 3 (Smoothed Equivalence) addresses my request. The proof sketch via eigenvalue gap is the right approach: under perturbations of magnitude σ to vertex weights, the eigenvalue gap shrinks by O(σ · max_degree) rather than O(1), so for σ ≪ α / max_degree the gap is preserved and method equivalence persists. The legal implications paragraph (census undercount, O(1%) perturbation) appropriately frames the practical relevance.

---

## Score: 3.5/4

**Assessment**: Accept with minor revisions. My three major concerns are addressed. The NP-hardness remark is the most important improvement — it correctly positions the contribution as a structural/landscape result rather than a tractability result. The Proposition's proof is sound with one minor gap in Part 4 (the assumption about Fiedler-boundary perturbation should be flagged). The empirical validation is complete, and the smoothed analysis adds the robustness argument.

---

## Remaining Minor Issues

**m1 (largely resolved)**: The NP-hardness remark addresses the complexity-theory connection I requested. The additional paragraph on "strong problem structure dominating algorithmic choice" (my m2 from Round 1) is present in the Discussion in the form of the eigenvalue gap explanation and the NP-hardness framing. I am satisfied.

**m2 (new — Part 4 assumption)**: The near-optimality bound (Part 4 of the Proposition) assumes that population balance can be achieved by perturbing at the Fiedler sign boundary. This is true for the studied instances but is not a consequence of C1–C3 alone. Add a sentence flagging this: "For the census tract graphs studied here, this perturbation is always feasible due to the fine population granularity of census tracts; in general, the bound holds modulo a feasibility assumption on balance-adjusting moves."

**m3 (experimental reproducibility, carried from R1)**: METIS version and exact flags are still not reported. This remains a concern for reproducibility.

**m4 (new — Discussion synthesis)**: A paragraph synthesizing the structural α_crit formula, the Corollary's [11,38] estimate, the empirical [20,50] transition, and the working value α = 5 would make the phase transition characterization complete. Currently these are spread across Sections 4.1, 4.2, and 5.1 without a unified summary.

**m5 (new — Corollary bounds)**: The Corollary states α_crit ∈ [11,38] for the test states. The empirical transition is at [20,50]. The overlap is [20,38]. The paper attributes the gap between the theoretical lower bound (11) and the empirical transition (20) to simulation noise without quantifying how much noise is expected. A brief analysis — or at minimum an acknowledgment that this gap is an open problem — would strengthen the paper.

---

## Summary

This revision transforms the paper from "strong empirical work with underdeveloped theory" to "rigorous empirical + theoretical contribution." The NP-hardness remark is technically precise and necessary for a complexity-aware audience. The Eigenvalue Gap Proposition provides the spectral foundation that was missing in Round 1. The phase transition characterization answers my key question with appropriate nuance. With the minor clarifications noted, this paper is ready for publication.

## Recommendation

**Accept with minor revisions** (non-blocking; addressable in production).

## Conflicts of Interest

None.
