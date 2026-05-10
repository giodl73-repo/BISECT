---
reviewer: Nicholas Stephanopoulos
round: 1
score: 3
date: 2026-05-05
---

## Summary

B.6 provides the complexity-theoretic foundation for the DIA's use of METIS as the statutory partitioner. Its legal argument — that NP-hardness justifies a heuristic and that approximation guarantees bound worst-case deviation from the optimal — is well-framed and directly applicable to legal challenges. As an election law reader, I cannot evaluate the technical correctness of the proofs, but I can assess whether the legal applications of the theoretical results are correctly drawn. My main concerns are about the scope of the legal claims and the precision of the "approximation guarantee" argument as it would be applied in court.

## Strengths

- **The "heuristic is necessary" argument is the paper's most valuable legal contribution.** The argument that NP-hardness means no polynomial-time exact algorithm can exist (unless P=NP), and therefore the DIA's use of METIS is not a design weakness but a theoretical necessity, directly answers the most obvious legal challenge: "why didn't you use the optimal algorithm?" This is a strong argument that will hold up in expert witness testimony.
- **The approximation ratio legal framing.** The paper argues that the O(sqrt(log n)) approximation ratio "bounds how far METIS can deviate from the optimal compact map, providing a worst-case hedge for legal challenges." This framing is correct and useful. Courts evaluating redistricting algorithms have asked questions of this form in the GerryChain literature context.
- **The 3 MB peak memory result.** For courts and legislative bodies that may question whether the DIA's algorithm is computationally feasible for state and local government implementation, the concrete memory requirement (<3 MB for California) provides a clear and compelling answer.

## Weaknesses / P1 Items (Required Fixes)

- **The "P=NP" framing needs qualification for non-technical audiences.** Section 5.1 states "no polynomial-time algorithm can guarantee an optimal redistricting map unless P = NP." For legal practitioners, this sentence requires unpacking: P=NP is an unproven hypothesis, and the contingent framing ("unless P = NP") could be read as leaving open the possibility that such an algorithm exists. The paper should add a sentence clarifying that "P = NP is universally believed to be false among computer scientists, making the NP-hardness result effectively unconditional for practical purposes."
- **The approximation ratio 4.1 claim will not survive cross-examination.** Section 5.2 computes the O(sqrt(log n)) approximation ratio for n = 74,000 as "approximately sqrt(log 74,000) ≈ 4.1." This means METIS's output could be up to 4.1 times worse than the optimal. In a deposition, opposing counsel would immediately ask: has METIS ever produced a redistricting map that was 4.1 times worse than optimal? If the answer is no (and empirically, B.7 shows it's within 3% of the best seed found), then the 4.1 figure is misleading. The paper should explicitly distinguish the worst-case theoretical bound (4.1) from the empirical performance (within 3% of the best of 10,000 seeds), and explain that the theoretical bound is a guarantee, not a prediction.
- **The paper does not address whether the redistricting problem has a feasibility gap.** The NP-hardness result covers the decision version of the problem: "does a partition with edge cut ≤ C exist?" But the redistricting problem has an additional feasibility requirement: the partition must exist (a valid partition always exists for connected graphs with sufficient district count). The paper should clarify that the hardness is in the optimisation (finding the minimum-cut partition), not in feasibility (finding any valid partition). Legal audiences may conflate these.

## P2 Items (Suggestions)

- **Add a paragraph connecting the complexity result to the due process question.** In redistricting litigation, defendants sometimes argue that the adopted plan was chosen in good faith from among feasible alternatives. NP-hardness supports this narrative: because finding the optimal plan is computationally intractable, any enacted plan (including the DIA's) is a good-faith approximation, not a strategic manipulation.
- **Cross-reference B.7's empirical approximation finding.** The paper should explicitly note in Section 5.2 that B.7 provides an empirical approximation result (2.9% above best seed found in 10,000 trials) that complements the theoretical O(sqrt(log n)) bound. Together, the two results create a credible picture of METIS's actual performance range.

## Score: 3 — Minor Revision

The legal framing is generally sound, with the three P1 issues representing communication problems (qualifying the P=NP statement, distinguishing theoretical from empirical bound, clarifying feasibility vs. optimisation) rather than substantive errors. These fixes are straightforward.
