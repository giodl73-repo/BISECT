# Review: Slice-Based Cross-Census Validation for Congressional Redistricting Algorithms

**Reviewer**: Moon Duchin (Rutgers)
**Expertise**: Gerrymandering, metric geometry, fairness
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper makes a valuable methodological contribution to redistricting research by proposing a rigorous cross-census validation framework. The observation that within-slice variance across time is less than between-slice variance within a census is geometrically intuitive and, if properly substantiated, has implications for how we understand redistricting stability and fairness.

My primary concern is that the paper focuses narrowly on algorithm validation without engaging with the broader normative questions that make redistricting consequential. The abstract claims the algorithm is "neutral to political considerations," but neutrality is not a well-defined mathematical property—it's a contestable normative claim. The paper would benefit from acknowledging this tension and discussing what kinds of neutrality (partisan, racial, socioeconomic) the validation framework can and cannot assess.

Additionally, the compactness metrics discussion is insufficient. Polsby-Popper and Reock are mentioned but not critiqued. These metrics have known problems (Polsby-Popper is scale-variant, Reock is easily gamed) and more sophisticated measures exist (Roeck-Reock, convex hull ratio, moment of inertia).

## Score

**Score**: 3/4 — **Accept**

## Major Issues (Blocking)

### M1: "Neutrality" Claim Needs Nuance
The abstract states the algorithm "remains neutral to political considerations." But what does this mean? If districts follow geography and demographics are correlated with partisanship (which they are), the result is not politically neutral—it's geography-induced sorting. The paper should distinguish between: (1) Process neutrality (no partisan data used), (2) Outcome neutrality (no systematic partisan advantage), (3) Intent neutrality (no gerrymandering motive). These are different concepts with different legal and normative implications.

### M2: Compactness Metrics Uncritically Applied
The paper uses compactness as a proxy for quality but doesn't discuss: (1) Which compactness measure is used (PP? Reock? Both?)? (2) Why this measure vs alternatives? (3) How does slice size affect compactness distributions? (4) What is the null distribution of compactness for random partitions with equal population constraints? Without this context, compactness scores are hard to interpret.

### M3: Fairness Implications Not Discussed
Cross-census validation is valuable, but what does it tell us about fairness? If an algorithm consistently produces districts with efficiency gaps of 8% across all census years, is that "valid" or problematic? The paper needs a section connecting validation results to fairness outcomes (even if the conclusion is that validation is orthogonal to fairness).

## Minor Issues

### m1: Voting Rights Act Compliance Not Mentioned
The VRA requires majority-minority districts in certain states. Does the algorithm produce these? Are they stable across census years? This is legally required and affects ~40-50 districts nationwide.

### m2: Geometric Measures Beyond Compactness
Moment of inertia, area-perimeter ratio, convex hull deficit—these are established measures in metric geometry that could enrich the analysis.

### m3: Ensemble Methods Comparison Missing
The Metric Geometry and Gerrymandering Group (MGGG) has published ensemble methods that generate distributions of compliant maps. How does single-algorithm validation relate to ensemble-based validation?

### m4: Definition of "Persistent Tract Centroid" Ambiguous
Are centroids stable in geographic space or in feature space (e.g., population density, urbanization level)? Clarify whether "persistence" is spatial, demographic, or both.

## Strengths

1. **Methodological rigor**: The slice-based approach is more sophisticated than naive cross-census comparison.
2. **Scale**: 50 states × 3 census years is comprehensive.
3. **Insight potential**: The variance decomposition (geographic vs temporal) is a genuinely interesting finding.
4. **Practical relevance**: State legislatures and courts need validation methodologies for redistricting algorithms.

## Questions for Authors

1. Have you compared your METIS results to human-drawn maps (actual legislative districts) for the same census years?
2. Can the slice-based framework detect gerrymandering if partisan data is introduced?
3. How do slices interact with communities of interest (COI), which are often legally protected?
4. Does algorithm stability across census years imply fairness, or can stable unfairness persist?
5. What happens in states where slices cut across urban-rural divides (known to correlate with partisanship)?

## Recommendations

- Replace "neutral to political considerations" with more precise language (e.g., "does not use partisan data")
- Add a "Limitations" section discussing what the validation framework does NOT assess (partisan fairness, VRA compliance, COI preservation)
- Include multiple compactness metrics and discuss their trade-offs
- Provide null model comparisons: what is the expected compactness distribution for random partitions?
- Cite and engage with MGGG ensemble methods and discuss how your single-algorithm approach complements or differs from ensemble-based validation

---

**Verdict**: **Accept with Minor Revisions**

**Confidence**: High — My work focuses on mathematical fairness in redistricting and I am confident these revisions would clarify the paper's scope and claims.
