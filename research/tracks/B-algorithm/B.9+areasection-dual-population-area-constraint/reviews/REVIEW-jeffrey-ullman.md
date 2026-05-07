# Review R-1: Jeffrey Ullman
**Paper**: AreaSection: Simultaneous Population and Land-Area Balance in Minimum-Edge-Cut Redistricting
**Date**: 2026-05-03
**Score**: 2.5 / 4

---

## Summary

The paper introduces AreaSection, an extension of METIS-based redistricting that simultaneously enforces population balance and land-area balance via a two-constraint formulation (`ncon=2`, `ubvec=[1.001, 1.10]`). The central claim is that the area constraint suppresses urban-peeling artifacts without sacrificing the minimum-edge-cut objective. A Lorenz curve pre-filter identifies feasible ratio windows. A 50-state empirical sweep is presented, along with partisan analysis on Wisconsin and North Carolina.

The empirical scope is commendable. The Lorenz curve analysis is clean. However, the paper makes a number of claims that outrun its formal foundations. Several parameters are introduced without sensitivity analysis or principled justification. Six large states fail the stated balance tolerance, and the authors' treatment of this failure is insufficient. I recommend major revision.

---

## Strengths

**S1. Lorenz Curve Pre-Filter Is a Genuine Contribution.**
The use of the population-area Lorenz curve to identify feasibility windows is a clean and novel idea. The visualization communicates, at a glance, why certain ratio values produce infeasible bisections. This is the most technically crisp contribution in the paper.

**S2. Empirical Scope Is Serious.**
Running the full 50-state pipeline on actual Census data is non-trivial work. 44/50 states completing successfully with dramatic improvements in area ratios (IL 1:16 → 8:9, NY 1:25 → 13:13, FL 1:27 → 14:14) constitutes a meaningful empirical contribution.

**S3. Partisan Stability Finding Has Policy Value.**
The finding that partisan seat counts are stable under AreaSection (WI and NC both yield 3D/5R) is a strong result. It directly addresses the most likely objection from practitioners — that adding a geometric constraint would introduce partisan distortion through the back door.

**S4. Implementation on Real Data.**
The paper is grounded in Census tract geometries and FIPS-coded state boundaries. This distinguishes it from a large fraction of the redistricting literature, which works on synthetic planar graphs or toy examples.

---

## Weaknesses

**W1. The Lorenz Condition Is Necessary, Not Sufficient — Is This Stated Clearly?**
The Lorenz feasibility window identifies ratio values for which area balance is *not obviously impossible*. It does not guarantee that METIS will find a balanced cut within that window. The paper must state explicitly, and repeatedly where relevant, that the Lorenz condition is a necessary filter, not a sufficient one. What is the empirical rate of Lorenz-passing ratios that still produce infeasible METIS solutions?

**W2. The `ubvec=[1.001, 1.10]` Parameter Is Unjustified.**
The 10% area imbalance tolerance is the central design parameter of the entire method, and it receives no principled justification. The authors report area splits reaching 62.9% when the target is 50% ± 10%. The paper does not explain how often this occurs across the 44 completing states. Without a sensitivity analysis on this parameter, the claim that "area balance" is being achieved is not well-supported.

**W3. The Six Large-State Failures Are a Fundamental Issue, Not a Footnote.**
FL, IL, MI, NY, PA, and TX all fail the 1.5% population balance tolerance. Together they account for roughly 37% of congressional districts. The paper does not answer whether the failure is *fundamental* (constraint incompatibility in recursive bisection) or *incidental* (rounding, fixable by post-processing).

**W4. The Ratio Selection Procedure Lacks Formal Analysis.**
The isoperimetric normalization EC/sqrt(min(i, k-i)) is inherited from B.8 (GeoSection) and justified by analogy to the isoperimetric inequality. This is not a proof. The paper should either cite a theorem or label this explicitly as an empirically-motivated heuristic.

**W5. The Partisan Analysis Is Underpowered.**
The Rodden effect observation — 8/8 competitive states show Republican over-representation, mean gap -6.2pp — does not specify the methodology for competitive-state classification, does not report variance, and does not control for state-level demographic variation.

**W6. METIS Constraint Violation Is Not Quantified.**
`ubvec` is a soft tolerance in METIS, not a hard constraint. The paper reports a 62.9% area split in at least one case when the target is 50% ± 10%. How often does this happen? What is the distribution of actual area imbalances across all bisections in the 44-state sweep?

---

## P1 Items (Must Fix Before Acceptance)

**P1-1. Formal status of the Lorenz condition.**
Add a theorem or clearly labeled proposition stating that the Lorenz feasibility window is a necessary but not sufficient condition for a balanced METIS cut. Report the empirical rate of Lorenz-passing ratios that nonetheless produce infeasible solutions.

**P1-2. Sensitivity analysis on `ubvec`.**
Report results for at least three values of the area tolerance parameter (e.g., 1.05, 1.10, 1.20). Show the effect on completion rate, distribution of actual achieved area imbalances, and the urban-rural ratio metrics that are the paper's headline result.

**P1-3. Characterize the large-state failures.**
For the six failing states: determine whether the failure is architectural or incidental. Report this determination, with evidence. If architectural, the paper must substantially revise its claims about applicability.

**P1-4. Quantify METIS constraint violations.**
Report the distribution of actual area imbalances across all bisections in the 44-state sweep, not just headline cases.

---

## P2 Items (Should Fix)

**P2-1.** Clarify the theoretical status of EC/sqrt(min(i,k-i)) normalization.

**P2-2.** Strengthen the partisan analysis: narrow the claim or provide full methodology for the 8-state competitive comparison, including variance on the -6.2pp figure.

**P2-3.** Report area-balance metrics alongside population-balance metrics throughout.

**P2-4.** Discuss the relationship between ncon=2 and METIS's internal multilevel coarsening.

---

## Verdict

The paper addresses a real and important limitation of minimum-edge-cut redistricting. The Lorenz pre-filter is novel and well-executed. The 50-state sweep is the most comprehensive in the redistricting algorithms literature. The paper's weaknesses are primarily ones of rigor and completeness: the central parameter is unjustified, the formal status of the Lorenz condition is unclear, the method fails on 37% of congressional districts without adequate explanation, and METIS constraint violations are reported anecdotally rather than systematically.

These are correctable. I recommend major revision with the four P1 items as conditions for re-review. A revised version addressing P1-1 through P1-4 would be substantially stronger and could reasonably receive 3.5 or better.

**Score: 2.5 / 4**
