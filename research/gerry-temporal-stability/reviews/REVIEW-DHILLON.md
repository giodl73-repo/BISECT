# Review: Cross-Census Temporal Stability
## Reviewer: Dr. Inderjit Dhillon (UT Austin)
**Expertise**: Graph clustering, scalable algorithms, temporal analysis
**Date**: 2026-02-08
**Score**: 3.5/4.0 (Strong Accept)

---

## Overall Assessment

This paper makes a solid empirical contribution to understanding temporal stability in graph partitioning methods. As someone who works extensively on temporal graph analysis and clustering stability, I find this work technically sound and the research question well-motivated. The finding that hierarchical structure provides modest but consistent stability advantages aligns with theoretical expectations.

**Key Strengths**:
1. **Clean experimental design**: Proper comparison of methods on identical data
2. **Appropriate metrics**: Population disruption rate is well-chosen
3. **Reproducible**: Clear methodology, code provided
4. **Honest reporting**: Doesn't oversell 1.1% effect

**Key Limitations**:
1. **Lack of theoretical analysis**: Why does hierarchy help? Missing graph-theoretic explanation
2. **Limited temporal depth**: Only one decade (2010-2020), need longer timeframe
3. **No spectral analysis**: Could explain stability via eigenvalue/eigenvector stability
4. **Scalability not addressed**: How do findings scale to larger graphs?

---

## Major Issues (P1 - Blocking)

### P1.1: Missing Theoretical Foundation for Hierarchical Stability
**Issue**: Paper empirically shows recursive bisection is more stable but doesn't explain WHY hierarchical structure provides this advantage. From graph clustering theory, we expect:

1. **Modularity preservation**: Top-level cuts should have high modularity, making them robust to perturbations
2. **Spectral stability**: Hierarchical bisection follows eigenvector structure, which evolves smoothly
3. **Optimization landscape**: k-way partitioning has many local optima, different runs can converge to different solutions

**Fix Required**: Add Section 3.6 "Theoretical Analysis":
- Analyze modularity of top-level splits (2010 vs 2020)
- Show that recursive bisection follows 2nd eigenvector (Fiedler vector)
- Prove/argue that hierarchical optimization has smoother objective landscape

**Mathematical Framework Needed**:
```
Let G₂₀₁₀ = (V, E₂₀₁₀) and G₂₀₂₀ = (V, E₂₀₂₀) be graphs over same vertices
Define perturbation: ΔE = E₂₀₂₀ \ E₂₀₁₀

Show: Hierarchical partition P_H has smaller ||P_H(G₂₀₂₀) - P_H(G₂₀₁₀)||
      compared to k-way partition P_K
```

---

## Major Issues (P2 - Important)

### P2.1: Temporal Depth Insufficient
**Issue**: One decade (2010-2020) isn't enough to establish temporal trends. Need:
- 2000-2010-2020 for three-way comparison
- Analysis of whether stability advantage compounds over multiple cycles
- Test whether hierarchies established in 2000 survive to 2020

**Recommendation**: Add 2000 data or clearly label this as "single-decade" study

---

### P2.2: No Spectral Graph Analysis
**Issue**: Temporal stability in graph clustering is often explained via spectral methods. Paper could strengthen theoretical foundation by analyzing:

1. **Eigenvalue stability**: Do top eigenvalues λ₁, λ₂, ..., λ_k stay similar 2010→2020?
2. **Eigenvector alignment**: Does 2nd eigenvector (Fiedler vector) remain similar?
3. **Spectral gap**: Is stability related to algebraic connectivity?

**Recommendation**: Add spectral analysis showing:
- Eigenvalue plots for 2010 vs 2020 graphs
- Eigenvector alignment scores
- Correlation between spectral gap and stability

---

### P2.3: Scalability Analysis Missing
**Issue**: Study uses 5 states with 664-2,796 tracts. How do findings scale?

**Questions**:
- Does recursive advantage hold for California (8,000+ tracts)?
- What about Texas (5,000+ tracts)?
- Are there graph size thresholds where method choice matters more?

**Recommendation**: Add two large-scale validation runs (CA, TX) or discuss scalability limits explicitly.

---

### P2.4: No Analysis of Graph Structure Changes
**Issue**: Paper measures partition stability but doesn't analyze how underlying graph changes:

Metrics needed:
- **Edge addition/deletion**: How much does E₂₀₁₀ differ from E₂₀₂₀?
- **Degree distribution**: Do high-degree nodes stay high-degree?
- **Clustering coefficient**: Does local structure preserve?

These graph-level changes explain partition-level stability.

**Recommendation**: Add Section 4.2 "Graph Evolution Analysis" before discussing partition stability.

---

## Minor Issues (P3 - Nice to Have)

### P3.1: Community Detection Perspective Missing
**Issue**: Temporal stability in graph partitioning relates to community detection literature. Paper should cite:
- Rosvall & Bergstrom on map equation stability
- Palla et al. on community evolution
- Mucha et al. on multilayer community detection

**Recommendation**: Add Related Work subsection on temporal community detection.

---

### P3.2: No Confidence Intervals or Significance Tests
**Issue**: With n=5 states, 1.1% difference might not be statistically significant.

**Recommendation**:
- Bootstrap confidence intervals
- Paired t-test (recursive vs n-way per state)
- Report p-values

---

### P3.3: Optimization Convergence Not Analyzed
**Issue**: METIS uses random seeds. Paper doesn't report:
- Variance across multiple runs with different seeds
- Whether results are deterministic or stochastic
- Sensitivity to initialization

**Recommendation**: Run each configuration 10 times with different seeds, report mean ± std.

---

## Detailed Technical Comments

### Algorithmic Implementation
✅ **Correct**: RecursiveBisection class properly implements hierarchical bisection
✅ **Correct**: Edge weights computed consistently
⚠️ **Concern**: niter=100 is unusually high. Standard is 10. At iteration 100, is METIS still improving or has it converged? Show convergence plots.

### Metric Choice

**Population Disruption Rate** is good but could be complemented by:
1. **Normalized Mutual Information (NMI)**: Standard metric for comparing partitions
2. **Adjusted Rand Index (ARI)**: Measures partition similarity
3. **Variation of Information (VI)**: Information-theoretic distance

These metrics from clustering literature would strengthen comparison.

### Missing Baselines

**What's missing**: Comparison to other stability-enhancing approaches:
- Consensus clustering over multiple runs
- Incremental clustering (use 2010 partition as seed for 2020)
- Modularity-optimizing methods (Louvain, Leiden)

### Statistical Power

With n=5 states and 1.1% effect size:
- Cohen's d ≈ 0.15 (very small effect)
- Power analysis suggests need n≈15-20 states for 80% power
- Current study is underpowered for effect this small

**Recommendation**: Either add more states or acknowledge power limitation.

---

## Recommendations for Revision

### Tier 1 (P1 - Must Fix)
1. **Add theoretical foundation** (Section 3.6): Explain WHY hierarchy provides stability
   - Modularity preservation analysis
   - Spectral stability (eigenvector alignment)
   - Optimization landscape smoothness

### Tier 2 (P2 - Strongly Recommended)
1. Add spectral graph analysis (eigenvalues, Fiedler vector)
2. Include graph structure evolution metrics (edge changes, degree distribution)
3. Discuss scalability explicitly or add large-state validation
4. Add statistical significance tests (bootstrap CI, p-values)

### Tier 3 (P3 - Would Strengthen)
1. Connect to community detection literature
2. Add NMI/ARI metrics alongside population disruption
3. Test sensitivity to random seeds
4. Compare to incremental/consensus clustering baselines

---

## Recommendation

**Score: 3.5/4.0 (Strong Accept with Revisions)**

This is a solid empirical study addressing a novel and practical question. The finding that hierarchical partitioning provides modest temporal stability advantages is valuable, and the execution is technically competent.

The major gap is lack of theoretical explanation (P1.1). From graph clustering theory, we can predict that hierarchical methods should be more stable due to:
1. Following eigenvector structure (which evolves smoothly)
2. Optimizing modularity at each level (robust to perturbations)
3. Having simpler optimization landscape (fewer local optima)

Adding this theoretical analysis would elevate the paper from "empirical observation" to "theoretically grounded finding." The spectral analysis (P2.2) would provide concrete evidence for these theoretical claims.

With P1 addressed, this is a strong contribution bridging graph theory and societal applications.

**Venue Fit**: Excellent fit for ACM-KDD. Consider also submitting to:
- SIAM Data Mining (more theory-focused)
- PNAS Applied Math (broader impact audience)
- Journal of Graph Algorithms and Applications (specialized theory venue)

---

## Questions for Author Rebuttal

1. **Theoretical mechanism**: Can you explain via graph Laplacian or spectral methods why hierarchy helps?

2. **Scalability**: Do findings generalize to graphs with 10,000+ nodes (Texas, California)?

3. **Statistical power**: With n=5 and small effect size, is study adequately powered?

4. **Seed sensitivity**: How much variance in results from different random seeds?

5. **Optimization convergence**: At iteration 100, has METIS converged or still improving?

---

## Contribution Assessment

**Novelty**: ★★★★☆ (4/5) - First empirical temporal stability comparison
**Rigor**: ★★★☆☆ (3/5) - Good empirics, weak theory
**Impact**: ★★★★☆ (4/5) - Practical implications for redistricting practice
**Clarity**: ★★★★★ (5/5) - Excellent writing, transparent reporting

**Overall**: Strong empirical contribution that would benefit from theoretical depth. Fix P1.1 and this is a clear accept.
