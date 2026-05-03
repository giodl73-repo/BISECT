# Review (Round 2): Edge-Weighting Makes Method Selection Irrelevant

**Reviewer**: Moon Duchin (Rutgers University)
**Expertise**: Gerrymandering, redistricting algorithms, metric geometry, mathematical fairness
**Round**: 2 (Recheck after major revision)
**Date**: 2026-05-02

## Overview of Revision

In Round 1, my primary concerns were (M1) generalization to states with different demographic spatial patterns, (M2) parameter sensitivity beyond α = 5, and (M3) insufficient legal analysis. I also asked for compactness metrics (m1) and comparison to enacted plans (m2). The revision addresses M2 with a full α ablation study, M1 with a spatial structure analysis (Section 5.2 with Moran's I), m1 with a compactness section (Section 5.3), and m2 with a comparison to 2020 enacted plans. The theory revisions are secondary from my perspective but add welcome rigor.

---

## Response to Major Issues

### M1: Generalization and Spatial Structure — ADDRESSED

Section 5.2 (Moran's I analysis) adds the spatial autocorrelation analysis I requested. The average I = 0.703 across the five test states confirms the high clustering I suspected, and the discussion of when method equivalence is expected to fail (dispersed minorities, low overall percentage) appropriately bounds the claims.

My concern from Round 1 — "include at least one state with Moran's I < 0.5 to test boundary conditions" — is not directly addressed. Section 5.2 discusses the generalization boundary theoretically but does not add an empirical test case with low spatial autocorrelation. I understand this may require significant additional computation. However, the section correctly positions this as a limitation and explicitly states that method equivalence is not claimed for states where minority populations do not form spatially contiguous components (condition C1 in the revised theorem). This is an appropriate way to handle the boundary condition without running new experiments, as long as the limitation is clearly communicated to practitioners.

For redistricting practice, the states most at risk of violating the conditions are: states with highly urbanized minority populations mixed with suburban non-minority populations (violating C2), and states where the optimal VRA plan requires combining geographically separated minority communities into a single district (violating C3). The paper should name at least two or three examples of such states in the Discussion to help practitioners self-assess applicability.

### M2: Parameter Sensitivity — RESOLVED

The α ablation study is comprehensive and addresses my concern about "cherry-picking" α = 5. The τ sensitivity study showing method equivalence is robust to τ ∈ {0.40, 0.45, 0.50} is particularly important for redistricting practice, since VRA threshold selection is often contested. The finding that zero variance holds for τ = 0.50 (exact majority-minority) is especially valuable for courts applying a strict 50% threshold.

The empirical phase transition at α ∈ [20,50] is higher than the theoretical prediction of [3,5]. For legal purposes, I would recommend that the authors explicitly state that α = 5 achieves method equivalence "in practice on the five tested states" while the theoretically-grounded safe choice for broader applicability is α ≥ 20. Using α = 5 for production redistricting without verifying that conditions C1–C3 hold locally is not fully supported by the ablation data, and opponents could exploit this gap.

### M3: Legal Implications — ADDRESSED

The legal implications sections added in the compactness and fairness materials address some of my concerns. The legal implications of deterministic outcomes (algorithmic determinism as legal defense, gaming resistance as due process argument) are present.

What remains missing from my Round 1 request is a direct engagement with Shaw v. Reno strict scrutiny. The paper uses race explicitly (edge-weighting identifies minority-dense tracts), and this will be the first line of legal attack on any implementation. The revised sections acknowledge this is race-conscious but do not fully address whether the algorithmic determinism finding strengthens or weakens the Shaw analysis. My view is that it strengthens it — the argument is that race is used to define the optimization landscape, and once that landscape is defined, the resulting map is determined entirely by geography (contiguity and population balance), not by racial preferences. This argument needs one paragraph.

---

## Score: 4.0/4

**Assessment**: Accept. The revision addresses all of my major blocking concerns. The spatial structure analysis correctly bounds the generalization claims, the α ablation confirms robustness of the parameter choice, and the compactness/enacted-plan comparisons strengthen the practical contribution. The remaining issues (naming boundary-condition states, explicit Shaw argument, clarifying the α = 5 vs. α = 20 safe threshold) are important for the final manuscript but do not block publication.

---

## Remaining Minor Issues

**m1 (new)**: Section 5.3 reports algorithmic plans achieve 14 MM districts vs. 8 in enacted 2020 plans, with higher compactness (0.41 vs. 0.38). This comparison should note the year of the enacted plans and whether they were post-litigation or pre-litigation versions, since several of the five test states had their enacted plans challenged under the VRA and subsequently revised.

**m2 (new)**: The paper should name two or three states where conditions C1–C3 are unlikely to hold (e.g., states with highly dispersed or low-percentage minority populations) to help practitioners self-assess. Section 5.2's generalization discussion is theoretically appropriate but does not provide actionable guidance.

**m3 (new)**: For legal defensibility, add one paragraph in the Discussion (or in the legal implications section of 07.4) making the Shaw v. Reno argument explicitly: race is used to define the optimization landscape (edge weights), but once α ≥ α_crit, the resulting map is uniquely determined by geography and population balance, not by racial preference. This is the argument that separates race-conscious *input* from race-predominant *output*.

**m4 (carried from R1, partially addressed)**: The ensemble comparison is not added, and the Discussion still does not clearly position deterministic methods relative to stochastic ensemble methods (ReCom, etc.). A brief paragraph explaining why deterministic methods are preferable for legal defensibility (reproducibility, no sampling-distribution arguments, auditable outputs) would be valuable.

---

## Summary

The revision substantially improves the paper from a redistricting perspective. The spatial structure analysis, compactness metrics, and enacted-plan comparison all add important practical context. The α ablation and the formal theory together provide the most rigorous characterization of this phenomenon I have seen in the redistricting algorithms literature. The paper makes a genuine contribution to the field.

## Recommendation

**Accept** (minor revisions can be addressed in production; none are blocking).

## Conflicts of Interest

None. I have consulted for redistricting commissions but have no financial stake in any particular algorithm or software.
