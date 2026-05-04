> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review Round 2: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Reviewer**: William J. Cook (University of Waterloo)
**Expertise**: Exact optimization algorithms, integer programming, algorithm selection, computational complexity
**Round**: 2 (Revision Review)
**Date**: 2026-05-02

## Summary of Revisions Received

In Round 1 I scored this paper 3/4. My concerns were: (1) no bounds analysis or optimality assessment; (2) algorithm selection framework lacks objectivity; (3) reproducibility gaps; and (4) overgeneralized claims. The revision addresses the P1 items comprehensively, while my P2 concerns are only partially resolved.

## P1 Resolution Assessment

### P1-1: Implementation Bug — RESOLVED

The corrected `tpwgts` implementation confirms the bug and the corrected results (30.0% MC success, down from 35.0%) now widen the performance gap. This is the outcome I hoped for: implementation correction that validates rather than undermines the paper's claim. Accepted.

### P1-2: Theory Section — RESOLVED

The formal tightness definition and 60–800× quantification replace the confused 129%/258% calculation. The constraint conflict is now framed as a hypothesis with empirical support rather than a mathematical proof. This is honest and appropriate.

### P1-3: Balanced Comparison — RESOLVED

140-vs-140 is a fair comparison. The state-level result (80% vs 40%) is now the primary claim and the configuration-level result is properly contextualized. I note that the new finding — MC achieves 0% across all 28 configurations in Alabama and Louisiana — is a strong empirical result that the original 4-configuration design could not have revealed. The expanded design was worth doing.

### P1-4: Statistical Rigor — RESOLVED (within scope)

Section 5.6 provides appropriate statistical support: Wilson CIs, χ² test, per-state 30-seed variance, and Phase 2 population estimates. The SD=0 finding is notable: it means the per-state outcomes are fully determined by algorithm and parameter, not by random initialization. This is exactly the kind of variance quantification I asked for, and the result (deterministic outcomes) is more informative than I expected.

I do note that the χ² test (p=0.039) is statistically weak in the absolute sense. The authors should be careful not to overstate this as strong evidence; the state-level pattern (80% vs 40%, complete MC failure in 3/5 states) is far more persuasive.

## Remaining Concerns

### P2-7: Bounds Analysis Still Absent

This was my most idiosyncratic concern and it remains unaddressed. The paper still compares two heuristics without establishing how close either comes to the optimum. For South Carolina, the paper now makes a plausibility argument — "3 × 60% × 1/7 ≈ 25.7%, close to available 28.5%" — but this is an arithmetic sketch, not an upper bound computation. I would accept even a simple LP relaxation bound for one or two states, or a direct citation showing the state-level target is infeasible under contiguous equal-population constraints.

I recognize this is the most demanding item on my list and that the authors have done substantial new work. I will accept a formal statement in the paper that says: "We do not claim optimality; our results are heuristic comparisons. For South Carolina, we conjecture that 3 MM districts are infeasible under equal-population contiguous partitioning, but formal verification via IP or upper-bound analysis is left for future work." This is honest and does not require additional computation.

### P2-5: Algorithm Selection Criteria Still Informal

Section 6.2 still presents the decision criteria in qualitative terms. The tightness ratio concept is now defined (τ_c = ε) but the decision threshold — "when should I use edge-weighting?" — is still phrased as "constraints differ by 60–800×." A practitioner facing a new problem still must run both methods to find out which works. I asked for a scoring function or decision tree with quantitative thresholds. A minimal version would be: "If τ_tight / τ_loose > T, prefer edge-weighting; we find T ≈ 60 sufficient in our experiments." One table, not a new section.

### P2-1: Reproducibility Gap

Complete METIS command-line specifications are still not in supplementary material. I cannot verify the results as published. This should be an easy fix: paste the exact invocations used.

## Score and Recommendation

**Score: 3.5/4 — Accept with Minor Revisions**

The revision is substantive and credible. All four P1 items are addressed. The statistical section provides the variance quantification I required. The balanced experimental design is a genuine improvement, and the state-dependency finding that emerged from it (complete MC failure in AL/LA/SC) is a stronger result than the original paper contained.

My remaining concerns — bounds analysis, decision threshold formalization, command-line reproducibility — are all achievable without new experiments. I recommend accepting the paper after a final revision pass addressing these three items. If the authors add (a) a one-paragraph limitation statement on optimality/feasibility of SC, (b) a threshold table for the algorithm selection criterion, and (c) a supplementary METIS invocation appendix, I will support acceptance without further review.

**Verdict**: Accept with minor revisions (no re-review required if the three items above are addressed in the author response letter).
