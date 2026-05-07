# Review 3 — Moon Duchin
**Paper**: F.2: NestSection at Scale — Spine-Compatible Bicameral Redistricting for All 50 States
**Round**: R1
**Score**: 2/4

## Summary

F.2 is the most mathematically interesting paper in the F track. The NestSection construction is an elegant approach to the bicameral redistricting problem. My review focuses on the mathematical claims about the spine factorization, the PP penalty scaling, and whether the results are correctly presented.

## Strengths

The Proposition on spine triviality when g=1 is correctly stated: when gcd(H,S)=1, the spine degenerates to a single super-region (the whole state), and NestSection provides no factorization benefit. This is a clean mathematical result. The spine definition (Definition 1) is precise and the algorithm description in Section 2.1 correctly separates the spine construction from the within-spine redistricting.

The edge-cut penalty scaling argument (Section 5.3) — that the senate penalty scales approximately linearly with the H/S ratio at +0.8% per additional house district per senate — is the right conceptual framework. Each additional house-district boundary incorporated into a senate district adds perimeter without adding area.

## Concerns

**C1 — Count inconsistency is a mathematical error.** Table 1 lists 9 states with gcd=1 (Missouri, Oklahoma, Texas, Hawaii, Pennsylvania, Connecticut, Rhode Island, Maine, Delaware). 49 - 9 = 40 compatible states. But the abstract says "42 states support a compatible ratio," the abstract says "the remaining 8," and Section 3.2 says "7 incompatible states." These three different counts (7, 8, 9 incompatible; 40, 41, 42 compatible) cannot all be correct. Counting from Table 1 directly gives 9 gcd=1 states, yielding 40 compatible states. This needs to be corrected to a single consistent count throughout the paper.

**C2 — Senate penalty linear scaling claim needs proof or simulation.** The claim that "the senate penalty scales approximately linearly with the H/S ratio at +0.8% per additional house district per senate" is asserted without derivation or regression. The data in Table 3 show: Vermont (5:1, senate penalty 5.2%), Massachusetts (4:1, senate penalty 4.5%), Ohio/Florida (3:1, senate penalties 3.9% and 4.1%), Washington (2:1, senate penalty 2.9%). Fitting a line through these points: 5.2-2.9=2.3 additional penalty for 3 additional H/S units, giving approximately +0.77% per unit. This is consistent with the stated +0.8% claim, but the paper should report the regression rather than asserting it from inspection of five data points.

**C3 — PP versus edge-cut asymmetry.** Section 5.4 notes that the PP penalty from nesting (-0.008 house, -0.028 senate) is smaller than the edge-cut penalty (+2.1% house, +3.4% senate). The paper attributes this to the nature of the metrics: "PP measures the shape of individual districts, while edge-cut measures total boundary length." This is correct but incomplete. PP and edge-cut are not directly comparable in magnitude (PP is dimensionless, edge-cut is in boundary-length units). A more rigorous treatment would convert edge-cut to an implied PP reduction using the relationship PP ≈ 4πA/P^2, where an x% increase in P decreases PP by approximately 2x% (for small x). For a 3.4% edge-cut penalty, the implied PP reduction would be approximately 6.8% of PP, or approximately 0.4 × 0.402 × 0.068 ≈ 0.011 — which is approximately the observed 0.028 senate PP reduction. This calculation should be shown to validate the consistency between the two metrics.

**C4 — Nesting within spine: independence assumption.** Section 2.1 states that "within each spine super-region, the house sub-map is the (H/g)-way partition of the super-region" and that this independently optimises house and senate maps. However, if H/g is not an integer multiple of S/g (i.e., if H/S is not an integer), the within-spine redistricting faces a simultaneous constraint: H/g house districts and S/g senate districts must all have the same area, and the senate districts must be exact unions of house districts. For ratios like Colorado (13:7, g=5) and Kansas (25:8, g=5), H/g = 13 and S/g = 7, and 13/7 is not an integer — so senate districts cannot each contain the same number of house districts. How does NestSection handle this case?

## Recommendation

Revise and resubmit. C1 (count inconsistency) is a mathematical error. C4 (non-integer H:S ratio within spine) is a gap in the algorithm description that may indicate a real limitation of NestSection for non-integer-ratio states.
