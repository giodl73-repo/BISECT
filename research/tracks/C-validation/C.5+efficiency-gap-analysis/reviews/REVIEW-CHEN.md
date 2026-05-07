# Review: Measuring Partisan Fairness in Algorithmic Redistricting

**Reviewer**: Jowei Chen (University of Michigan, Political Science)
**Expertise**: Computational redistricting, simulations, neutral benchmarks
**Date**: 2026-02-08
**Venue**: American Political Science Review

---

## Overall Assessment

This paper employs computational redistricting at impressive scale (50 states, 3 election years) to establish efficiency gap benchmarks for neutral algorithms. The methodology is generally sound, and the finding that algorithmic plans exhibit -3.2% EG provides an important empirical baseline. However, the paper needs substantial strengthening on methodological transparency, sensitivity analysis, and legal defensibility of the algorithmic approach.

**Verdict**: Accept with major revisions
**Score**: **3/4**

---

## Major Issues

### 1. Insufficient Algorithmic Transparency (P1)

You describe using "recursive bisection via METIS" but provide minimal detail on implementation choices that affect partisan outcomes. As someone who has testified in redistricting cases, I can tell you courts ask:

**Critical missing details:**
- Edge weight specifications (are tracts weighted by population? Geographic proximity? Both?)
- Tiebreaking rules when multiple partitions have similar cut weights
- Starting seed initialization (random? Deterministic?)
- How many algorithmic maps did you generate per state? (You show one "algorithmic plan" but best practice is to generate ensembles of 1000-10000 maps)

**Problem:** Without these details, I cannot evaluate whether your -3.2% baseline is:
a) A stable property of neutral algorithms, or
b) An artifact of specific METIS parameterization

**Required revision:**
1. Add detailed "Algorithm Specification" section (likely in Section 3 or in online appendix)
2. Report sensitivity to edge weight specifications (try geographic-only vs population-weighted edges)
3. Generate ensemble of 100+ algorithmic maps per state, report mean ± std dev of EG across ensemble
4. Clarify whether your "algorithmic plan" is single map or average across ensemble

### 2. Compactness-Partisan Correlation Unexplored (P1)

METIS optimizes for edge cut minimization, which produces compact districts. But compactness and partisan outcomes are correlated—compact urban districts pack Democrats. You need to show:

**Missing analysis:**
1. Actual compactness scores (Polsby-Popper, Reock, etc.) for algorithmic vs enacted plans
2. Scatter plot: compactness (x-axis) vs efficiency gap (y-axis) to show correlation
3. Do enacted plans with similar compactness to algorithmic plans still show higher EG? (If yes, that's strong evidence of manipulation)

**Why this matters:** Critics will say "your algorithms produce Democratic bias because they over-optimize for compactness." You need to show enacted plans are LESS compact but MORE Republican-favoring, which proves manipulation.

**Required addition:** Add subsection "Compactness-Partisan Tradeoff" showing compactness scores and correlation with EG.

### 3. Alternative Algorithms Not Tested (P1)

You use only METIS recursive bisection. But legal defensibility requires showing that multiple neutral algorithms produce similar results. What if:
- K-means clustering produces different EG?
- Shortest splitline algorithm produces different EG?
- Voronoi diagrams seeded by population centroids produce different EG?

**Problem:** If different algorithms produce wildly different EGs, then "algorithmic neutrality" is poorly defined. If they all produce similar EGs (~-3% to -4%), that's strong evidence of geographic determinism.

**Required revision:** Add subsection testing 2-3 alternative neutral algorithms. If time/space limited, do this for 5 representative states (PA, TX, CA, FL, NY) and report in online appendix. Main text should discuss whether results are algorithm-specific or general.

---

## Minor Issues

### 4. Election Data Transparency (P2)

You mention "district-level election results" but don't specify:
- Which elections? (House? Presidential? Average across multiple offices?)
- How did you map precinct-level results to algorithmic districts? (Point-in-polygon? Population-weighted aggregation?)
- What about split precincts? (Many precincts span multiple districts)

**Suggested addition:** Data appendix explaining election data processing, especially precinct-to-district mapping methodology.

### 5. Statistical Significance Underreported (P2)

You report p < 0.001 for all regional differences, but no confidence intervals, standard errors, or effect size measures beyond Cohen's d. For publication in APSR, you need full statistical reporting.

**Required additions:**
- Confidence intervals for all mean EG estimates
- Bootstrapped standard errors (since observations aren't independent—same states across years)
- Regression table predicting EG from region, year, plan type

### 6. Communities of Interest Entirely Absent (P2)

Real redistricting must respect communities of interest (cities, counties, regions). Your algorithmic plans likely split communities to optimize compactness. This is legally problematic in many states.

**Suggested addition:** Add paragraph discussing how enforcing community-of-interest constraints (e.g., minimizing county splits) would affect algorithmic EG. My hypothesis: modest increase in EG (from -3.2% to maybe -2.5%) but still substantially below enacted plans.

---

## Positive Aspects

1. **Scale**: 50 states × 3 years = 150 maps is impressive computational undertaking
2. **Neutral baseline**: Using algorithms that can't access partisan data is methodologically sound
3. **Temporal consistency**: Showing EG stability across elections is important
4. **Regional variation**: Revealing Rust Belt vs Sunbelt differences is valuable
5. **Legal relevance**: Clear framing for state constitutional litigation

---

## Specific Recommendations

### Section 3 (Methodology)
- Add "3.4 Algorithm Specification" with complete METIS parameterization
- Add "3.5 Sensitivity Analysis" testing alternative algorithms and edge weights
- Add "3.6 Election Data Processing" explaining precinct aggregation

### Section 4 (Results)
- Add subsection on compactness scores for algorithmic vs enacted plans
- Report confidence intervals for all EG estimates
- Add scatter plots: compactness vs EG, urban density vs EG

### Section 5 (Discussion)
- Add paragraph on communities of interest and how they interact with algorithmic redistricting
- Discuss legal defensibility of algorithmic approach given state constitutional requirements

### Online Appendix (Recommended)
- Complete algorithm pseudo-code
- Sensitivity analysis for 5 representative states
- Robustness checks: alternative algorithms, edge weights, compactness thresholds

---

## Questions for Authors

1. Did you generate single algorithmic plan per state or ensemble? If single, how sensitive is EG to random seed?
2. Have you tested whether enforcing county/municipality constraints significantly affects algorithmic EG?
3. Can you release your algorithmic maps publicly? (Transparency is critical for replication)

---

## Comparison to My Work

In my *Florida* and *North Carolina* analyses, I generated 10,000 simulated maps per state using similar neutral criteria. Key differences from your approach:

1. **Ensemble vs single map**: I report distributions, you report point estimates
2. **Communities of interest**: I enforce county constraints; unclear if you do
3. **Sampling method**: I use MCMC sampling of redistricting space; you use single deterministic algorithm

**Implications:** My ensembles show EG range of -2% to -5% depending on random draws. Your single -3.2% estimate likely falls within this range, but without ensemble you can't quantify uncertainty.

**Recommendation:** Generate ensembles for at least 5 states to validate your single-map approach.

---

## Verdict Justification

This paper makes an important empirical contribution and the computational scale is impressive. The finding that neutral algorithms produce -3.2% EG is valuable for policy and legal debates. However, methodological transparency is insufficient for publication in a top political science journal, and sensitivity analysis is missing.

The paper needs:
1. Full algorithm specification
2. Sensitivity to alternative algorithms
3. Compactness-partisan correlation analysis
4. Ensemble generation to quantify uncertainty

With these additions, this will be a strong methodological contribution demonstrating the limits of algorithmic neutrality. The core findings are likely robust, but the methodology needs more rigorous documentation and testing.

**Recommendation**: Major revisions required, focusing on methodological transparency and robustness.
