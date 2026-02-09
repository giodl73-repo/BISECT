# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Moon Duchin (Rutgers)
**Expertise**: Gerrymandering, metric geometry, fairness
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper makes a valuable contribution to automated redistricting by incorporating geometric information into graph partitioning. The core insight—that boundary length minimization directly optimizes compactness—is mathematically sound and empirically validated. The 56% improvement over unweighted baseline and 20% over enacted districts demonstrates clear practical value.

From a metric geometry perspective, the work is interesting but incomplete. Polsby-Popper is just one compactness metric, and optimizing it doesn't necessarily produce districts that satisfy normative redistricting goals. The paper gestures toward "gerrymandering resistance" (Illinois +174%) but doesn't rigorously define or measure this property. Compactness and fairness are distinct concepts—compact districts can still produce partisan bias, especially in geographically sorted populations.

More critically, the paper's claim of "partisan neutrality" is oversold. The algorithm is *politically blind* (uses no partisan data), but this doesn't make outcomes neutral. Geographic determinism means compact districts may systematically advantage one party (as Rodden's work shows, Democrats cluster in cities while Republicans spread across suburbs/rural areas). The paper acknowledges this in passing but doesn't grapple with the implications.

The gerrymandering analysis (Section 4.5) is the paper's strongest contribution to redistricting theory. The argument that "gerrymandering requires elongation" is intuitive and supported by the Illinois case study. However, this needs more rigorous treatment: What exactly is the relationship between perimeter and partisan advantage? Under what conditions does compactness guarantee fairness?

For a computational venue (KDD, AAAI), this is a strong empirical paper with clear algorithmic contributions. For redistricting/political science venues, it needs deeper engagement with fairness, representation, and Voting Rights Act compliance.

## Score

**Score**: 3/4 — **Accept**

## Major Issues (Blocking)

### M1: Compactness ≠ Fairness, But Paper Conflates Them

The paper repeatedly implies compact districts are "fair" or "neutral":
- "partisan neutrality" (abstract, multiple sections)
- "neutral baseline for evaluating gerrymandering claims" (conclusion)
- "geometric optimization implicitly promotes fairness" (conclusion)

This is incorrect. Compact districts can exhibit severe partisan bias due to geographic sorting. The paper must:
- Explicitly distinguish political blindness (no partisan data) from political neutrality (no partisan advantage)
- Measure partisan metrics (efficiency gap, partisan symmetry, seats-votes curves) for algorithmic plans
- Show whether compact plans favor Democrats or Republicans and by how much
- Discuss Rodden's "geography is destiny" problem directly

Without partisan analysis, claims of "neutrality" or "fairness" are unsubstantiated.

### M2: Voting Rights Act Compliance Not Addressed

The Voting Rights Act requires majority-minority districts in many states. The paper mentions VRA once in passing (Section 2.4) but never evaluates compliance:
- Do algorithmic plans create majority-minority districts where required?
- How do compactness and VRA compliance trade off?
- Can edge weighting be modified to respect racial fairness constraints?

For states like Alabama (requiring Black-majority district), Georgia, North Carolina, Texas (Hispanic-majority districts), VRA compliance is constitutionally mandatory. The paper can't claim to provide "practical redistricting method" while ignoring this.

At minimum:
- Analyze majority-minority district counts for algorithmic vs enacted plans
- Discuss tension between compactness maximization and VRA compliance
- Propose how edge weighting could incorporate demographic constraints

### M3: Single Metric Optimization Is Insufficient

The paper optimizes Polsby-Popper exclusively. Other compactness metrics (Reock, convex hull) improve but less dramatically. More importantly, compactness is just *one* redistricting criterion:
- County preservation (minimize county splits)
- Communities of interest (keep cities/regions together)
- Competitive districts (avoid safe seats)
- Proportional representation (seat share ≈ vote share)

Real redistricting requires balancing multiple objectives. The paper's single-objective approach is a toy problem. Include:
- Multi-objective formulation with weighted combinations
- Pareto frontier analysis (compactness vs other criteria)
- Demonstration that edge weighting can incorporate other objectives

## Minor Issues

### m1: "Gerrymandering Resistance" Undefined

The paper claims boundary minimization "resists gerrymandering" (Section 5.2) but never defines what this means quantitatively. Gerrymandering detection methods include:
- Outlier analysis (ensemble comparison)
- Efficiency gap thresholds
- Mean-median difference
- Declination measures

Show that algorithmic plans fall within "reasonable" bounds on these metrics, or acknowledge that compactness alone doesn't guarantee non-gerrymandering.

### m2: Geographic Sorting Not Fully Addressed

Section 5.2 acknowledges "any redistricting plan has partisan effects due to geographic sorting" but doesn't quantify this. Key questions:
- What's the expected Democratic vs Republican advantage from compact districts?
- Which states show pro-D vs pro-R bias?
- Is there correlation between compactness and partisan bias?

Include partisan analysis for representative states showing geographic sorting effects.

### m3: Indiana Outlier Underexplored

Indiana's commission achieves 0.478 Polsby-Popper (vs algorithmic 0.353, +35%). The paper dismisses this as "exceptional human performance" but doesn't investigate:
- What did Indiana's commission do differently?
- Can we reverse-engineer their approach?
- What tradeoffs did they make (county splits, communities of interest)?

Analyzing best-case human performance could improve the algorithm.

### m4: Communities of Interest Ignored

Real redistricting commissions preserve "communities of interest"—cities, neighborhoods, cultural regions. Pure compactness optimization may split these. Example: A compact district that bisects a city is geometrically optimal but normatively poor. Discuss this tension and how it might be addressed.

### m5: Competitive Districts Not Evaluated

Some redistricting reforms prioritize competitive districts (swing seats). Do compact districts tend to be competitive or safe? Analyze margin of victory distributions for algorithmic vs enacted plans.

## Strengths

1. **Mathematically rigorous**: Polsby-Popper optimization via perimeter minimization is elegant and theoretically sound.

2. **Strong empirical results**: 56% improvement, 37/50 states surpass enacted plans demonstrates practical effectiveness.

3. **Gerrymandering mechanism insight**: "Elongation required for partisan advantage" is valuable theoretical contribution, especially Illinois case study.

4. **Honest about limitations**: Discussion acknowledges single-objective optimization and political neutrality vs outcomes distinction (though needs more depth).

5. **Computational efficiency**: 2-3 hours for 50 states enables practical deployment and rapid scenario exploration.

6. **Reproducibility**: Open-source commitment and detailed methodology enable verification and extension.

## Questions for Authors

1. **Partisan analysis**: What are efficiency gap, mean-median difference, and seats-votes curves for algorithmic plans? Do compact districts systematically favor Democrats or Republicans?

2. **VRA compliance**: How many majority-minority districts do algorithmic plans create vs enacted plans? Which states lose required minority representation?

3. **Multi-objective optimization**: Can edge weighting incorporate other objectives (county preservation, racial fairness)? What's the Pareto tradeoff?

4. **Gerrymandering definition**: How do you formally define "gerrymandering resistance"? What metrics quantify this beyond compactness?

5. **Geographic sorting quantification**: Can you measure expected partisan bias from geography alone (no gerrymandering)? How much of enacted plans' partisan effects are geographic vs intentional?

6. **Indiana reverse engineering**: What techniques did Indiana's commission use to achieve 0.478 compactness? Can these be algorithmically incorporated?

7. **Communities of interest**: How often do compact districts split cities or neighborhoods? Can this be measured?

## Recommendations

- **Add partisan analysis**: Compute efficiency gap, partisan symmetry, mean-median difference, seats-votes curves for algorithmic plans. Show partisan effects and acknowledge that compactness ≠ neutrality.

- **VRA evaluation**: Count majority-minority districts, compare to enacted plans, discuss compliance challenges.

- **Multi-objective formulation**: Extend to weighted objectives (compactness + county preservation + racial fairness). Show Pareto frontiers.

- **Gerrymandering metrics**: Compute outlier scores, efficiency gap, declination for algorithmic plans; show they fall within reasonable bounds.

- **Soften neutrality claims**: Replace "partisan neutrality" with "political blindness" or "algorithmic neutrality." Acknowledge geographic bias.

- **Indiana deep dive**: Analyze Indiana's commission process, identify techniques, discuss whether they're algorithmically replicable.

- **Normative discussion**: Engage with redistricting principles beyond compactness (representation, fairness, VRA, communities of interest).

---

**Verdict**: Accept with Major Revisions

**Confidence**: High — I have extensive experience with redistricting mathematics, gerrymandering detection, and electoral fairness. This paper makes a solid algorithmic contribution (edge weighting improves compactness) but overclaims on fairness/neutrality without partisan analysis. The work would be substantially strengthened by: (1) measuring partisan effects, (2) evaluating VRA compliance, (3) distinguishing compactness from fairness explicitly, (4) extending to multi-objective optimization. As currently written, the paper is valuable for its computational contribution but incomplete as a redistricting solution. For KDD/AAAI, the algorithmic content justifies acceptance with revisions addressing these normative issues. For political science venues, major additions would be required.
