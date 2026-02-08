# Review: Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning

**Reviewer**: Moon Duchin (Rutgers University)
**Expertise**: Gerrymandering, metric geometry, fairness metrics
**Date**: February 7, 2026

---

## Overall Assessment

This paper tackles an important problem—VRA compliance through algorithmic redistricting—and proposes an interesting edge-weighting approach that shows empirical improvement. The systematic testing across states and methods is commendable. However, the paper has significant mathematical and conceptual weaknesses: (1) edge-cut minimization is not a principled compactness measure from a geometric perspective, (2) the "fairness" of these plans is not evaluated beyond demographics, (3) the optimization landscape is poorly understood (local vs global optima, uniqueness, stability), and (4) the paper treats VRA compliance as demographic threshold achievement rather than representation fairness.

The edge-weighting idea is clever but essentially amounts to a heuristic for preferentially keeping minority tracts together. Without theoretical analysis of when/why this works, the 80% success rate is empirically interesting but mathematically unsatisfying. The paper would benefit from deeper engagement with metric geometry, ensemble analysis, and fairness measurement beyond demographic percentages.

**Score**: 3/4 (Accept with major revisions)

---

## Major Issues

### M1: Edge-Cut is Not a Principled Compactness Measure

The paper uses edge-cut minimization as the sole compactness metric, but from a metric geometry perspective, this is problematic:

**Why edge-cut is insufficient**:

1. **Graph-dependent**: Edge count depends on how you construct the graph (rook vs queen adjacency, resolution of units). Two identical geometric shapes can have different edge cuts depending on discretization.

2. **Not scale-invariant**: A large district and a small district with identical shapes will have different edge cuts (more tracts → more edges). Comparing edge cuts across states with different tract counts is dubious.

3. **No geometric interpretation**: What does "edge cut = 276" mean geometrically? How compact is this in real space? Edge-cut is a graph-theoretic quantity, not a geometric one.

**Standard compactness metrics** (which this paper ignores):
- **Polsby-Popper**: 4π × Area / Perimeter² (measures shape roundness, scale-invariant)
- **Reock**: Area / Area of minimum bounding circle (measures convexity)
- **Convex hull**: Area / Convex hull area (measures indentation)
- **Isoperimetric quotient**: Generalization of Polsby-Popper to higher dimensions

These metrics have clear geometric interpretations and are invariant under scaling/rotation. Edge-cut is not.

**Example of the problem**: The paper claims Alabama edge-weighted plan "actually improves compactness (-1.4%)" (p.12, line 316). But:
- Baseline: 280 edge cut, 585.1 avg internal edges
- Edge-weighted: 276 edge cut, 585.7 avg internal edges

How can overall edge cut decrease while average internal edges increase? These metrics are telling different stories. Without geometric compactness measures, I can't assess actual district shapes.

**Recommendation**:
- Compute Polsby-Popper, Reock, and convex hull ratios for all districts in all plans
- Show distributions (not just averages) to assess variation
- Compare to "typical" compactness in neutral ensembles (MGGG ReCom chains)
- Acknowledge that edge-cut is a graph property, not a geometric property

### M2: No Ensemble Analysis—Are These Plans Outliers?

The paper presents single districting plans (one per method/state/configuration) but provides no context for how typical or unusual these plans are. In the MGGG approach, we generate ensembles of valid plans and ask: **where does this plan sit in the distribution?**

**Questions requiring ensemble analysis**:

1. **VRA compliance distribution**: In a neutral ensemble for Alabama, what percentage of plans achieve 2 MM districts at 50%+ minority? Is edge-weighting finding a rare outlier or a typical plan?

2. **Compactness distribution**: Is edge cut = 276 unusually compact, typical, or non-compact for 7-district Alabama plans?

3. **Stability**: How sensitive are results to METIS randomness? If you run METIS 100 times with different seeds, do you always get 2 MM districts, or is this configuration fragile?

4. **Uniqueness**: Are there many plans achieving VRA compliance, or is the edge-weighted plan essentially unique?

**Why this matters**: If edge-weighting finds a plan that's (a) highly unusual in the ensemble and (b) requires specific parameter tuning, it's less compelling than if it finds a typical region of plan-space that reliably achieves VRA compliance.

**Example analysis**: Generate ReCom ensemble for Alabama (100,000 plans). Measure:
- Histogram of "maximum minority percentage in any district" → where does 50.8% sit?
- Histogram of "number of MM districts" → what % of neutral plans achieve 2 MM?
- Scatterplot: edge cut vs MM district count → is there a Pareto frontier?

This would contextualize the edge-weighted plan: is it unusual? Is it on the efficiency frontier? How does it compare to the "typical" plan?

**Recommendation**: Add ensemble analysis using ReCom or other spanning tree methods. Compare edge-weighted METIS plans to ensemble distributions. This is standard practice in gerrymandering analysis (see *Science* paper on partisan gerrymandering, DeFord et al. work).

### M3: VRA Compliance ≠ Fairness

The paper conflates "achieving 50%+ minority demographics" with VRA compliance and fairness, but these are not equivalent:

**What VRA actually requires** (Gingles test):
1. Minority group sufficiently large and geographically compact to form majority in district
2. Minority group politically cohesive
3. Majority votes as bloc to defeat minority-preferred candidates

Only condition (1) is demographic—conditions (2) and (3) are behavioral and require election analysis.

**What fairness requires** (Gingles doesn't address this):
- Proportionality: Does overall representation match population?
- Opportunity: Can minority voters elect candidates proportional to population?
- Influence: Do minority voters affect outcomes in coalitional districts?

**Example from paper**: Georgia achieves 5 MM districts at 60-77% minority from 42.4% state-wide minority. But:
- 5/14 districts = 35.7% MM districts
- State-wide minority = 42.4%
- This is *under-representation* if we care about proportionality

Should Georgia create 6 MM districts (42.9% of seats)? Or accept 5 MM at higher percentages (more secure)?

The paper never addresses this tradeoff: **should we maximize MM district count or maximize minority-preferred candidate victories?** These may not be the same goal (see Lublin on descriptive vs substantive representation).

**Recommendation**:
- Clarify that demographic thresholds are necessary but insufficient for VRA compliance
- Discuss proportionality vs security tradeoff (more districts at 52% vs fewer at 65%)
- Analyze election results: how many seats elect minority-preferred candidates?
- Frame as "opportunity" rather than "compliance" (opportunity to elect is what VRA guarantees)

### M4: Optimization Landscape is Unanalyzed

The paper treats METIS as a black box that produces plans, but provides no analysis of the optimization landscape:

**Critical questions**:

1. **Local vs global optima**: Does METIS find global minimum edge cut, or a local minimum? How do you know?

2. **Multiple optima**: Are there many distinct plans with similar (near-optimal) edge cuts? Or is the solution essentially unique?

3. **Convergence**: How do METIS parameters (niter, ncuts) affect results? Are results stable?

4. **Edge-weighting landscape**: When you increase weight factor from 5x → 10x → 50x, how does the optimization landscape change? Are there phase transitions?

**Example of the problem**: Alabama results show:
- 5x @ 40%: 2 MM at 50.8%, edge cut 276
- 10x @ 45%: 2 MM at 52.0%, edge cut (not reported)
- 50x @ 40%: 2 MM at 51.8%, edge cut (not reported)

What are the edge cuts for these configurations? Are they Pareto-optimal (no other plan achieves better VRA compliance at same compactness)? Or are there plans with better compactness that also achieve 2 MM?

**Recommendation**:
- Plot Pareto frontiers: edge cut vs MM district count for various weight factors
- Analyze METIS sensitivity: run with multiple random seeds, report variance
- Discuss whether solutions are provably optimal or merely "good" heuristics
- Compare to provably optimal solutions from integer programming (if computationally feasible)

---

## Minor Issues

### m1: "Geographic Clustering" Needs Quantification

The paper repeatedly claims success depends on "geographic clustering" but never measures it quantitatively. From a metric geometry perspective, we have tools for this:

**Spatial statistics to compute**:
- **Moran's I**: Global spatial autocorrelation of minority percentage
- **Getis-Ord G***: Local clustering hotspots
- **Ripley's K**: Point pattern analysis of minority population distribution
- **Voronoi tessellation**: Natural clustering structure independent of census units

These would quantify "how clustered" minority populations are in Alabama vs Georgia vs South Carolina, providing explanatory power beyond eyeballing maps.

**Hypothesis to test**: VRA success correlates with Moran's I of minority tract distribution. High Moran's I (clustered) → success, low Moran's I (dispersed) → failure.

### m2: Resolution Dependence Not Discussed

Census tracts are arbitrary administrative units. Would results change with:
- **Block groups** (finer resolution, ~10x more units)
- **Blocks** (finest resolution, ~100x more units)
- **Voronoi cells** (non-administrative, geometric units)

From a metric geometry perspective, the "right" resolution is unclear. The paper uses tracts without justification.

**Concern**: Maybe Alabama could achieve 51% minority if we used blocks instead of tracts? Or maybe it would still fail? Resolution dependence is a fundamental issue in spatial optimization.

### m3: Threshold Analysis Lacks Rigor

The "42%" and "36%" thresholds are identified by eyeballing 5 data points. This is not rigorous:

**Issues**:
- No confidence intervals
- No statistical testing (logistic regression, classification analysis)
- Sample size N=5 is too small for reliable threshold estimation
- Threshold depends on clustering (Mississippi succeeds at 46.1%, Alabama fails at 36.9%—is this 10-point difference due to demographics or geography?)

**Better approach**:
- Model P(success) = f(state minority %, clustering metric, district count)
- Test on larger state set (all 50 states, not just 5 VRA-covered)
- Provide uncertainty quantification

### m4: Contiguity Not Verified

The paper assumes METIS produces contiguous districts, but this is not guaranteed for arbitrary edge-weighted graphs. Did you verify contiguity explicitly?

**Potential problem**: High edge weights on minority-minority edges might incentivize METIS to create disconnected components if it reduces overall weighted edge cut.

**Recommendation**: Add explicit contiguity verification and report any non-contiguous districts encountered.

### m5: Compactness-VRA Pareto Frontier Missing

The most important figure is missing: **scatterplot of VRA compliance vs compactness** showing the tradeoff frontier.

**What I want to see**:
- X-axis: Edge cut (or Polsby-Popper average)
- Y-axis: MM district count (or maximum minority %)
- Points: Each tested configuration (color by method)
- Frontier: Pareto-optimal configurations

This would visually demonstrate whether edge-weighting achieves better VRA-compactness tradeoffs or merely different points on the same frontier.

---

## Methodological Concerns

### MC1: Why METIS?

METIS is designed for parallel computing load balancing, not geographic redistricting. Why use METIS instead of methods designed for spatial partitioning?

**Alternatives not discussed**:
- **Integer programming**: Provably optimal solutions (Shirabe, Oehrlein)
- **Simulated annealing**: Better exploration of solution space
- **Genetic algorithms**: Population-based search
- **ReCom spanning trees**: Unbiased sampling of valid plans

The paper's contribution is "edge-weighting improves METIS for VRA," but maybe METIS is the wrong baseline. How does edge-weighted METIS compare to other methods?

### MC2: Parameter Tuning is Subjective

Choosing weight factor α and threshold τ per-state to maximize VRA compliance is essentially fitting parameters to achieve desired outcomes. This is not "principled" or "algorithmic" in the sense of being free from subjective choices.

**Question**: If a jurisdiction adopts edge-weighted METIS, how do they choose α and τ without knowing in advance what values achieve VRA compliance? The paper's approach is retrospective optimization, not prospective redistricting.

### MC3: Single vs Multi-Objective Framing is Misleading

The paper claims edge-weighting is "single-objective" (minimize weighted edge cut) while multi-constraint is "dual-objective" (balance population + minority). But:

**Edge-weighting is also multi-objective**:
- Objective 1: Minimize weighted edge cut (compactness)
- Objective 2: Preserve minority communities (encoded via weights)

The weight factor α determines the relative importance of these objectives, just like tpwgts determines relative importance in multi-constraint.

**So the real difference is not single vs multi-objective, but rather**:
- Multi-constraint: Explicit dual optimization with target minority percentages
- Edge-weighting: Implicit dual optimization through edge weights

Both involve tradeoff parameters (ubvec vs α). Neither is more "principled" than the other.

---

## Writing and Presentation

### w1: Over-claiming

"Resolves the perceived tension between VRA compliance and algorithmic principles" (abstract) is too strong. South Carolina still fails. The tension is reduced, not resolved.

"Doubles success rate" is accurate but misleading (40% → 80% sounds impressive, but it's 2/5 → 4/5 states, a small sample).

### w2: Technical Precision

- Define "majority-minority district" precisely: 50%+ VAP? CVAP? Total population?
- Specify METIS version and all parameters (not just tpwgts/ubvec)
- Clarify whether "minority" is Black + Hispanic or Black only (some states have both)

### w3: Missing Figures

Key figures not included:
- Maps of minority population distribution (choropleth by tract)
- Maps of resulting districts colored by minority percentage
- Pareto frontier plots (VRA vs compactness)
- Sensitivity analysis plots (success rate vs weight factor, threshold)

---

## Recommendations for Revision

**Critical (required for acceptance)**:
1. Add geometric compactness metrics (Polsby-Popper, Reock) not just edge-cut
2. Perform ensemble analysis—contextualize plans within distribution of valid plans
3. Provide Pareto frontier analysis (VRA vs compactness tradeoffs)
4. Temper over-claiming about "resolving" VRA-compactness tension

**Strongly recommended**:
5. Quantify geographic clustering (Moran's I, Getis-Ord G*)
6. Add maps showing minority distributions and resulting districts
7. Discuss proportionality vs security tradeoff in MM district design
8. Analyze optimization landscape (local optima, stability, uniqueness)

**Nice to have**:
9. Compare to other algorithmic methods (ReCom, IP)
10. Test resolution dependence (blocks vs tracts)
11. Verify contiguity explicitly

---

## Significance

This paper contributes to an important area—algorithmic redistricting for VRA compliance—and the edge-weighting approach is a useful addition to the toolkit. However, the contribution is primarily empirical (showing that edge-weighting improves outcomes on 5 states) rather than theoretical (understanding when/why edge-weighting works).

For a top-tier venue, I'd want to see:
- Deeper mathematical analysis of optimization landscape
- Ensemble-based contextualization of results
- Geometric compactness metrics, not just graph metrics
- Broader comparison to alternative methods

With these additions, this could be a strong paper for *Science* or *Nature* (if framed for general audience) or *Discrete & Computational Geometry* / *Operations Research* (if mathematically rigorous).

As currently written, it's a solid empirical study appropriate for *Political Analysis* or *Election Law Journal*, but lacks the mathematical depth for premier computational geometry venues.

---

**Bottom Line**: Interesting empirical work with a clever heuristic (edge-weighting), but mathematically shallow. The paper needs ensemble analysis, geometric compactness metrics, and deeper engagement with fairness theory to be truly compelling. The edge-weighting idea is promising, but the paper doesn't adequately explain *why* it works or *when* it will succeed/fail—just that it empirically does better on 5 test cases.
