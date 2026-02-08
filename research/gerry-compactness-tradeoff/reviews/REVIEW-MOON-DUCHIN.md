# Review: Quantifying the VRA-Compactness Tradeoff

**Reviewer**: Moon Duchin (Rutgers)
**Expertise**: Gerrymandering, Metric Geometry, Fairness
**Date**: 2026-02-08
**Round**: 1

---

## Overall Assessment

This paper makes an important empirical contribution to redistricting scholarship by systematically quantifying what has long been assumed: that Voting Rights Act compliance inherently requires sacrificing district compactness. The finding that non-majority-minority (non-MM) districts generally *gain* compactness (+7.5% average) when MM districts are created is genuinely surprising and challenges conventional wisdom that has shaped three decades of litigation and legislative practice.

The methodological approach—edge-weighted graph partitioning with systematic parameter sweeps across five states—is sound, and the Pareto frontier analysis provides a rigorous framework for characterizing state-dependent tradeoffs. The discovery of geographic feasibility thresholds (South Carolina's ratio 1.22) moves beyond algorithmic benchmarking to identify fundamental geometric constraints, which has clear implications for courts assessing VRA targets.

However, the paper would benefit from deeper engagement with the metric geometry of compactness, more careful treatment of the relationship between population-based and voter-based metrics, and acknowledgment of limitations in generalizing from five Southern states to the broader national redistricting context.

**Score**: 3.5/4 (Strong Accept with Minor Revisions)

---

## Major Strengths

### 1. Novel Empirical Finding with Policy Relevance
The central finding—that non-MM districts benefit rather than sacrifice compactness—directly contradicts the "spreading the pain" narrative used by VRA opponents. This has immediate implications for redistricting litigation and could shift how courts evaluate compactness arguments. The quantification (+7.5% non-MM gain vs -25.3% MM loss) makes the claim concrete and testable.

### 2. Rigorous Multi-State Comparison
Testing 105 configurations across five states with district-level breakdown demonstrates thoroughness. The identification of four distinct patterns (MM sacrifice/non-MM gain, both gain, both sacrifice, no success) shows that tradeoffs are state-dependent rather than universal, which is a significant methodological insight.

### 3. Mechanistic Explanation
The three mechanisms (geographic clustering enables joint optimization, non-MM districts benefit from clearer boundaries, baseline algorithms are suboptimal) provide causal understanding beyond pattern identification. The connection to Moran's I spatial autocorrelation and feasibility ratios grounds the findings in measurable geographic properties.

### 4. Pareto Frontier Framework
Framing the VRA-compactness relationship as a Pareto optimization problem is conceptually clean and operationally useful. The framework provides courts with a tool for assessing whether plans are dominated (suboptimal) or Pareto-efficient, which could strengthen legal challenges to gerrymandered plans.

---

## Major Issues (Must Address)

### M1: Compactness Metric Selection and Justification

**Issue**: The paper uses four compactness metrics (edge cut, Polsby-Popper, Reock, convex hull) but provides limited theoretical justification for why these specific measures capture the normative goals of compactness. From a metric geometry perspective, different metrics emphasize different aspects of shape regularity—Polsby-Popper penalizes perimeter irregularity, Reock emphasizes dispersion, convex hull measures deviation from convexity. The paper shows these metrics are correlated (r = -0.67 to +0.82) but doesn't address *why* we should care about compactness in the first place or which metric best captures the underlying normative concern.

**Recommendation**: Add a subsection in the Background (Section 2) discussing the normative foundations of compactness requirements. Why do state constitutions mandate compactness? Is it about minimizing travel distance (moment of inertia), preventing bizarre shapes (perimeter/area), or maximizing cohesion (graph-theoretic measures)? Justify your metric choices by linking them to these normative goals. The Polsby-Popper focus is reasonable given its legal prevalence, but the paper should acknowledge that different normative frameworks might prioritize different metrics.

**Severity**: P1 (Blocking) - The paper's central claim requires clarity on what "compactness" means normatively, not just operationally.

---

### M2: Voting-Eligible Population vs Total Population

**Issue**: Section 6.4 (Limitations) acknowledges that optimizing on total population may overestimate minority electoral strength due to age, citizenship, registration, and turnout gaps. However, this limitation is more than a methodological caveat—it fundamentally affects the paper's policy claims. A district with 50% minority *population* may have only 40-45% minority *voters*, potentially failing to achieve effective representation despite satisfying the algorithmic definition of "majority-minority."

The Supreme Court's *Evenweil v. Abbott* (2016) permits states to use total population, but Section 2 VRA litigation increasingly considers voter dilution, not just population dilution. If your Alabama "2 MM districts" actually have minority *voting* majorities below 50%, the VRA compliance claims become tenuous.

**Recommendation**:
1. Add empirical analysis of voting-eligible population (VEP) for your study states using Census CVAP data. Calculate the VEP minority percentage for each "MM district" you identify.
2. If VEP drops MM counts below targets (e.g., Alabama's 2 MM districts become 1.5 effective MM districts), acknowledge this explicitly and adjust claims about "VRA compliance" to "VRA compliance under population-based definitions."
3. In Discussion (Section 5), add a subsection on "Population vs Voting Power Tradeoffs" discussing how age/citizenship gaps affect the feasibility thresholds you identify.

**Severity**: P1 (Blocking) - This affects the validity of your VRA compliance metrics and policy recommendations.

---

### M3: Generalizability Beyond Southern States

**Issue**: All five study states (AL, GA, LA, MS, SC) are former Section 5 preclearance jurisdictions with histories of racial vote dilution and Black-White demographic patterns. The paper's mechanisms (geographic clustering, feasibility thresholds) are derived exclusively from this context. However, redistricting in states with multi-group populations (Black + Latino + Asian in CA/TX), dispersed minority populations (Native American in Western states), or different settlement patterns may exhibit fundamentally different tradeoff structures.

The paper's title and abstract make universal claims ("Non-Majority-Minority Districts Generally Benefit") without qualifying that "generally" means "in five Southern states with large, spatially autocorrelated Black populations." Extending these findings to California (where coalition districts require Black + Latino cooperation), New Mexico (tri-ethnic Anglo-Latino-Native American), or Hawaii (Asian-Pacific Islander majority) is speculative.

**Recommendation**:
1. Retitle the paper to clarify scope: "Quantifying the VRA-Compactness Tradeoff in the American South" or add "Evidence from Five Southern States" to the subtitle.
2. In the Limitations section, add a subsection on "Geographic and Demographic Scope" explicitly discussing which contexts your findings generalize to and which require further research.
3. In the Conclusion, frame future research extensions (multi-group coalitions, Western states) as necessary tests of your mechanisms, not just "nice to have" extensions.

**Severity**: P2 (Important) - Doesn't invalidate findings but affects how readers interpret their applicability.

---

## Minor Issues

### m1: Baseline Algorithm Characterization
You claim baseline METIS is "locally but not globally optimal" (page 49, Mechanism 3). This is imprecise—METIS uses multilevel refinement with multiple random starts, so it's not strictly trapped in local optima. More accurate: "Baseline METIS optimizes for raw edge cut without regard for demographic clustering, potentially missing Pareto-optimal solutions that jointly optimize compactness and VRA compliance." Clarify this in Discussion Section 5.1.

### m2: South Carolina Moran's I Interpretation
You report Moran's I = 0.581 as "strong spatial autocorrelation" (page 37, Section 4.6). While statistically significant, I = 0.581 is moderate, not strong. Strong clustering would be I > 0.7. Your point still holds (SC's failure isn't due to dispersion), but the language should be tempered: "statistically significant spatial autocorrelation" or "moderate clustering insufficient to overcome feasibility constraints."

### m3: Missing Error Bars
State-level averages (+7.5% non-MM gain, -25.3% MM loss) lack confidence intervals or standard errors. Given that these averages pool across 4 states with heterogeneous patterns (GA: +28.1%, LA: -39.0%), the variance is substantial. Add error bars or report ranges in Table 2 and Figure 2.

### m4: Edge-Weighting Implementation Details
Section 3.3 describes edge-weighting conceptually but omits algorithmic details that would allow replication. Specifically: (1) How are edge weights incorporated into METIS's multilevel coarsening phase? (2) Do you modify the initial partitioning or only the refinement phase? (3) Are weighted edges preserved across coarsening levels? Add a technical appendix with pseudocode or cite your implementation repository.

### m5: Pareto Frontier Identification Algorithm
Section 3.5 describes Pareto frontier identification as O(n²) pairwise dominance checks. For 105 configurations, this is trivial, but for larger parameter spaces, this doesn't scale. Consider mentioning that skyline algorithms (O(n log n)) exist for 2D Pareto problems, though your current approach is fine for this paper's scope.

---

## Recommendations for Revision

### High Priority (Must Address)
1. **Add normative justification for compactness metrics** (M1) - Why these metrics matter for redistricting goals.
2. **Incorporate VEP analysis** (M2) - Calculate voting-eligible population for MM districts, adjust VRA compliance claims if necessary.
3. **Clarify scope and generalizability** (M3) - Retitle or reframe claims to acknowledge Southern state focus.

### Medium Priority (Strengthen Paper)
4. **Add confidence intervals** (m3) - Quantify uncertainty in state-level averages.
5. **Technical appendix for replication** (m4) - Pseudocode or GitHub link for edge-weighting implementation.

### Low Priority (Polish)
6. **Refine local optimality claim** (m1) - More precise characterization of baseline METIS behavior.
7. **Temper Moran's I language** (m2) - "Moderate" not "strong" clustering.
8. **Mention skyline algorithms** (m5) - Note scalability for future work.

---

## Detailed Comments by Section

### Introduction (Section 1)
- **Strength**: Hook is excellent—immediately challenges conventional wisdom with concrete numbers (+7.5%, +22.2%).
- **Suggestion**: Add one sentence acknowledging that your findings apply to single-minority contexts (Black-White) and that multi-group scenarios require further research. This manages reader expectations upfront.

### Background (Section 2)
- **Missing**: Normative foundations of compactness (M1). Why do we care about compact districts beyond legal mandates?
- **Suggestion**: Add subsection "Why Compactness Matters" discussing travel time minimization, community cohesion, and preventing gerrymandering.

### Methodology (Section 3)
- **Strength**: Experimental design is thorough—105 configurations, 4 compactness metrics, 3 VRA metrics, systematic parameter sweeps.
- **Weakness**: Edge-weighting implementation details missing (m4). Readers cannot replicate your approach without more specificity.
- **Suggestion**: Add pseudocode in an appendix showing how METIS edge weights are computed and applied.

### Results (Section 4)
- **Strength**: District-level breakdown (Table 2) is critical—state-level aggregates would obscure the non-MM gain pattern.
- **Strength**: Four-pattern taxonomy (both gain, MM sac/non-MM gain, both sac, no success) is clear and memorable.
- **Weakness**: Missing error bars on averages (m3). Confidence intervals would strengthen claims.
- **Suggestion**: Add standard deviations or ranges to Table 2.

### Discussion (Section 5)
- **Strength**: Three mechanisms provide causal explanation, not just empirical patterns.
- **Strength**: Debunking three myths (Section 5.2) is rhetorically effective—directly addresses opposition arguments.
- **Weakness**: VEP vs population issue (M2) not addressed. This belongs in Discussion, not just Limitations.
- **Suggestion**: Add subsection "Population-Based vs Voter-Based VRA Metrics" discussing implications of age/citizenship gaps.

### Limitations (Section 6)
- **Strength**: Comprehensive—acknowledges single-minority focus, fixed district counts, tract resolution, population-only data, compactness-only objective.
- **Weakness**: Scope generalization (M3) framed as limitation but deserves stronger treatment. This isn't just a caveat—it's a boundary condition for your claims.
- **Suggestion**: Elevate scope discussion to a standalone subsection with more detailed treatment of which contexts your findings apply to.

### Conclusion (Section 8)
- **Strength**: Strong closing—"I-85 district represents algorithm failure, not inevitable cost" is memorable.
- **Suggestion**: Temper universal claims ("VRA compliance and compactness universally conflict") to "in contexts with spatially autocorrelated single-minority populations." Small wording change, big impact on accuracy.

---

## Significance and Impact

This paper has the potential to reshape redistricting litigation and legislative practice by demonstrating that VRA-compactness tradeoffs are state-dependent, not universal. The policy tools (Pareto frontiers, feasibility ratios) are immediately actionable for courts and redistricting commissions. The finding that non-MM districts *benefit* from VRA optimization contradicts 30 years of accepted wisdom and provides empirical ammunition for VRA proponents.

However, the paper's impact will be limited if readers perceive it as overreaching beyond its evidence base. Addressing the scope generalization issue (M3) and VEP analysis (M2) will strengthen rather than weaken the paper by making claims more defensible.

With revisions addressing the major issues, this paper should be a strong accept for APSR or a top-tier venue. The combination of rigorous methods, novel findings, and policy relevance makes it a significant contribution to both redistricting scholarship and computational social science.

---

## Final Recommendation

**Accept with Major Revisions**

The paper makes important contributions but requires addressing:
1. Normative foundations of compactness (M1)
2. Voting-eligible population analysis (M2)
3. Scope and generalizability clarifications (M3)

With these revisions, the paper will be publication-ready for a top venue.

**Estimated revision time**: 2-3 weeks for empirical VEP analysis, 1 week for conceptual additions.
