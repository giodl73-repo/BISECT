# Round 2 Review - Moon Duchin (Rutgers)
**Date**: 2026-02-07
**Round**: 2
**Paper**: Recursive Bisection for Congressional Redistricting

---

## Summary Assessment

**Score**: 3.5/4.0 (Strong Accept with Minor Revisions)
**Change from Round 1**: +1.0 points

The authors have substantially strengthened the mathematical rigor of this paper. The addition of Section 4.5 with 400-run parameter sensitivity analysis provides exceptional empirical evidence for algorithmic determinism. The new Section 3.9 on edge-weighted optimization demonstrates proper implementation of geometric optimization principles. These additions transform the paper from a primarily conceptual contribution to one with strong empirical and mathematical foundations. One remaining concern about compactness metric choice prevents a perfect score.

---

## What Changed My Score

### 1. **Rigorous Parameter Sensitivity Analysis** (Section 4.5)
**Impact**: +0.5 points

Round 1 concern:
> "Mathematical rigor needs strengthening. Claims about neutrality require formal proof or comprehensive empirical validation."

Round 2 response:
- **404 redistricting runs**: Systematic exploration across 4 geographically diverse states
- **Perfect reproducibility**: Coefficient of variation = 0.000000% for all metrics
- **Comprehensive testing**: Random seeds, METIS parameters (ufactor, niter, objtype)
- **Partisan stability**: Democratic seat share identical across all 100 seeds per state

**Why this satisfies my concern**:
1. Empirical validation is as rigorous as one can achieve without formal proof
2. Sample size (n=400) far exceeds typical standards for computational experiments
3. Zero variation is the strongest possible result—proves geographic determinism
4. Tests the specific claims about non-manipulability

From a mathematical perspective, this is exceptional empirical work. The coefficient of variation being exactly zero (not just "very small") indicates true determinism, not approximate stability.

### 2. **Edge-Weighted Optimization Implementation** (Section 3.9)
**Impact**: +0.5 points

Round 1 concern:
> "Compactness optimization implementation unclear. How exactly are tract boundaries weighted?"

Round 2 response:
- **Mathematical formulation**: Edge weight w(e) = shared boundary length between adjacent tracts
- **Geometric justification**: Minimizing weighted edge cuts = minimizing district perimeter
- **Empirical comparison**: 6-state analysis showing +3.2% Polsby-Popper improvement
- **Partisan stability verification**: 0.0% change in partisan outcomes (weighted vs. unweighted)

**Why this is important**:
1. Demonstrates proper understanding of geometric optimization principles
2. Edge weighting by boundary length is mathematically sound approach to compactness
3. Modest improvement (+3.2%) is realistic given geographic constraints
4. Zero partisan change confirms optimization is purely geometric

**Mathematical note**: The correlation between edge counts and boundary lengths (r=0.68) explains why unweighted approach already performs reasonably well—the graph structure partially encodes geometric information.

---

## Strengths of Revised Paper

### 1. **Mathematical Rigor**
The paper now demonstrates strong mathematical foundations:
- Systematic parameter exploration with comprehensive documentation
- Edge-weighted optimization with proper mathematical formulation
- Statistical analysis with appropriate metrics (CV, correlation coefficients)
- Honest quantification of improvements (e.g., +3.2%, not inflated claims)

### 2. **Empirical Depth**
- 404 runs for parameter sensitivity (exceptional sample size)
- 50-state VRA demographic analysis
- Process type disaggregation for compactness (11 commission, 4 court, 35 legislative)
- 6-state edge-weighting comparison

This level of empirical work is publication-quality.

### 3. **Honest Assessment**
The authors acknowledge limitations clearly:
- Edge weighting provides modest (+3.2%) not dramatic improvement
- Commission maps achieve higher compactness (Michigan +125%)
- Geographic constraints dominate (Texas case study)
- Localized VRA deficits exist in 5 states

This honesty strengthens rather than weakens the paper.

---

## Remaining Concern

### **Compactness Metric Justification** (UNRESOLVED)
**Severity**: Moderate (prevents 4.0 score)

The paper uses Polsby-Popper as the primary compactness metric but provides limited justification for this choice. This matters because:

1. **Different metrics capture different geometries**: Polsby-Popper favors circles, Reock favors convexity, etc.
2. **Metric choice affects optimization**: METIS minimizes edge cuts, which correlates with but doesn't directly optimize Polsby-Popper
3. **No sensitivity analysis**: Would results be similar with other metrics (Reock, convex hull, etc.)?

**What I need to see**:
- Comparison of district compactness across multiple metrics (PP, Reock, convex hull ratio)
- Correlation analysis: How strongly do different metrics agree?
- Discussion: Why Polsby-Popper is appropriate benchmark for geometric optimization

**Why this matters**:
From a mathematical perspective, the choice of compactness metric is not neutral—it encodes geometric assumptions about "ideal" district shapes. Without demonstrating robustness across metrics, we cannot be confident the approach optimizes "compactness" in general (as opposed to Polsby-Popper specifically).

**Note**: This is not a fundamental flaw—I expect metrics would be highly correlated. But mathematical rigor requires demonstrating this empirically.

---

## Detailed Section Comments

### Section 3.9: Edge-Weighted Optimization
**Quality**: Very good

**Strengths**:
- Clear mathematical formulation of edge weighting
- Proper geometric interpretation (boundary length)
- Empirical validation with 6 states
- Honest assessment of modest improvement

**Suggestions**:
- Consider adding a figure showing weighted vs. unweighted district boundaries for one state (visual comparison)
- The finding that edge counts correlate with boundary lengths (r=0.68) could be explored more—what explains the 0.32 residual variance?

**Mathematical note**: The approach is sound. The modest +3.2% improvement is expected given that METIS's edge-cut minimization already implicitly favors compact districts.

### Section 4.5: Parameter Sensitivity Analysis
**Quality**: Outstanding

**Strengths**:
- Exceptional sample size (404 runs)
- Perfect reproducibility finding (CV = 0.00%)
- Comprehensive parameter testing
- Clear statistical documentation

**Minor suggestion**: Consider adding a theoretical explanation for *why* perfect determinism occurs. Is it because:
1. METIS's multilevel algorithm converges to unique optimum?
2. Geographic constraints eliminate all degrees of freedom?
3. Both?

Understanding the mechanism would strengthen the mathematical narrative.

### Section 4.3: Compactness Gap Analysis
**Quality**: Good analytical contribution

The process type disaggregation is rigorous and provides valuable insights. The Michigan trade-off analysis (compactness vs. splits) shows understanding of multi-objective optimization constraints.

**Mathematical observation**: The redistricting reform trilemma is an interesting conceptual contribution but could be formalized. Are these truly mutually exclusive objectives, or is there a Pareto frontier? A formal treatment would strengthen this section.

---

## Comparison to Round 1

| Dimension | Round 1 | Round 2 |
|-----------|---------|---------|
| **Mathematical rigor** | Weak (major concern) | Strong (404-run validation) |
| **Empirical evidence** | Good | Exceptional |
| **Optimization clarity** | Vague | Clear (edge-weighted formulation) |
| **Statistical analysis** | Minimal | Comprehensive |
| **Metric justification** | Weak | Still weak (remaining concern) |

**Overall**: From "interesting concept, weak validation" → "strong empirical contribution"

---

## Scoring Rationale

**Score**: 3.5/4.0 (Strong Accept with Minor Revisions)

### Why not 3.0?
The additions are exceptional:
- Perfect reproducibility (0.00% CV) is extraordinary validation
- Edge-weighted optimization shows mathematical sophistication
- Statistical rigor substantially improved
- Honest assessment of limitations demonstrates maturity

### Why not 4.0?
One gap remains: compactness metric robustness. The paper needs to demonstrate that results hold across multiple compactness definitions, not just Polsby-Popper.

**Effort to reach 4.0**: 2-3 days
- Compute Reock, convex hull ratio, average radial distance for all states
- Correlation analysis across metrics
- Brief discussion of metric choice justification

---

## Publication Recommendation

**Recommendation**: Strong Accept with Minor Revisions

**Conditional on**: Either (1) multi-metric compactness analysis, or (2) expanded justification for Polsby-Popper as appropriate metric

**Venue suitability**:
- **APSR**: Yes—now has sufficient mathematical rigor
- **JOP**: Yes—strong fit
- **Geometry/topology journals**: Possibly—with expanded mathematical treatment

---

## Additional Observations

### Methodological Contribution
The perfect reproducibility finding is significant beyond this specific application. It suggests that for problems with strong geographic constraints, deterministic optimization may be *more* appropriate than stochastic sampling (contra recent trends toward ensemble methods).

### Philosophical Note
The paper's argument that "geographic determinism eliminates manipulation" is mathematically coherent. If algorithm has zero degrees of freedom (proven empirically), then manipulation is structurally impossible. This is a stronger claim than "manipulation is difficult" or "manipulation is unlikely"—it's a claim of impossibility, backed by empirical proof.

---

## Summary for Authors

You've substantially strengthened the mathematical and empirical foundations of this paper. The perfect reproducibility finding is exceptional validation of your core claims. The edge-weighted optimization demonstrates proper implementation of geometric principles.

One remaining gap (compactness metric robustness) prevents a perfect score, but this is addressable with modest additional analysis.

**Priority**: Multi-metric compactness validation (2-3 days of work would move score from 3.5 to 4.0)

With that addition, this would be an exceptional contribution to the computational redistricting literature.
