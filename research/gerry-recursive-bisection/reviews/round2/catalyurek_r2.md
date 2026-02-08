# Round 2 Review - Ümit V. Çatalyürek (Georgia Tech)
**Date**: 2026-02-07
**Round**: 2
**Paper**: Recursive Bisection for Congressional Redistricting

---

## Summary Assessment

**Score**: 3.5/4.0 (Strong Accept with Minor Revisions)
**Change from Round 1**: +0.5 points

The authors have made solid progress addressing computational concerns. The addition of Section 3.9 on edge-weighted optimization demonstrates proper implementation of geometric weighting and provides meaningful performance improvement (+3.2% average compactness). The parameter sensitivity analysis (Section 4.5) validates algorithmic stability. However, opportunities remain for further computational optimization through hypergraph formulation and more sophisticated objective functions, preventing a perfect score.

---

## What Changed My Score

### **Edge-Weighted Optimization Implementation** (Section 3.9)
**Impact**: Resolved my primary concern

Round 1 concern:
> "Edge weighting implementation unclear. Standard METIS uses unit weights—how are geographic features incorporated?"

Round 2 response:
- **Explicit edge weighting**: w(e) = shared boundary length between adjacent tracts
- **Geometric optimization**: Minimizing weighted edge cuts ≈ minimizing district perimeter
- **Empirical validation**: 6-state comparison showing +3.2% Polsby-Popper improvement
- **METIS integration**: Proper use of `adjwgt` parameter

**Why this satisfies my concern**:
1. Demonstrates understanding of METIS's edge-weighting capabilities
2. Provides meaningful (though modest) performance improvement
3. Shows that standard METIS formulation can be enhanced with domain-specific information
4. Validates approach empirically across multiple states

**Technical assessment**: Implementation is correct. The +3.2% improvement is reasonable given that unweighted METIS already captures some geometric structure through graph topology.

---

## Strengths of Revised Paper

### 1. **Proper Edge Weighting**
The edge-weighted optimization demonstrates:
- Correct understanding of METIS API
- Appropriate weight computation (boundary lengths)
- Meaningful performance improvement
- Validation across multiple states

### 2. **Computational Rigor**
- 404 runs for parameter sensitivity (comprehensive testing)
- Systematic parameter exploration (ufactor, niter, objtype)
- Perfect reproducibility finding validates algorithmic stability
- Clear documentation of implementation details

### 3. **Scalability Demonstrated**
- 50-state national scale (75,000+ tracts)
- Largest states (CA, TX) with ~8,000 tracts each
- Efficient execution time (minutes per state)
- Validates METIS's O(n log n) complexity

---

## Remaining Opportunities for Improvement

### 1. **Hypergraph Formulation** (NOT REQUIRED, but would strengthen)
**Potential benefit**: 5-10% additional compactness improvement

The current approach uses standard graph partitioning (pairwise edges). A hypergraph formulation where hyperedges connect all tracts sharing a common boundary point could potentially:

1. Capture higher-order geographic relationships
2. Better optimize perimeter (not just pairwise boundary minimization)
3. Provide more natural representation of contiguity constraints

**Implementation**: Use hMETIS (hypergraph METIS) instead of standard METIS

**Why I'm not requiring this**:
1. Would significantly increase implementation complexity
2. Benefit is uncertain (5-10% is estimate, not proven)
3. Current results are already strong
4. Standard graph formulation is appropriate for this application

**Future work recommendation**: If authors extend to block-level redistricting (10M units), hypergraph formulation might provide more significant benefits.

### 2. **Multi-Objective Optimization** (NOT REQUIRED, but would strengthen)
**Current approach**: Single-objective (edge-cut minimization)
**Alternative**: Multi-objective optimization balancing:
- Compactness (edge cuts)
- Population equality (deviation from target)
- County/municipal preservation (boundary crossing minimization)

**Potential implementation**:
- Use PaToH or Zoltan libraries (support multi-objective optimization)
- Define weighted objective function combining criteria
- Pareto frontier exploration

**Why I'm not requiring this**:
1. Would require switching from METIS to other partitioning libraries
2. Sequential optimization (population first, then compactness) is reasonable approach
3. Paper already demonstrates adequate results

**Note for authors**: If extending work, multi-objective formulation could address COI preservation concerns raised by Rodden.

---

## Technical Observations

### 1. **Computational Performance**
**Quality**: Good

Current performance (minutes per state) is adequate for redistricting application. For reference:
- Tract-level (5,000-8,000 units): Seconds to minutes (current)
- Block-level (100,000-500,000 units): Minutes to hours (projected)
- Parcel-level (1M-10M units): Hours (would need optimization)

**Scalability recommendations** (if extending to larger problems):
1. Consider ParMETIS for parallelization (10-100× speedup)
2. Hierarchical coarsening for very large problems
3. Hypergraph formulation for better quality at scale

### 2. **Perfect Reproducibility**
**Observation**: Interesting finding

The 0.00% variation across 400 runs indicates problem is highly constrained. From computational perspective, this means:
- METIS's randomized coarsening has no effect on final solution
- Geographic/population constraints uniquely determine optimal partition
- Deterministic behavior achieved despite randomized algorithm

**Implication**: For this application class, deterministic algorithms (like recursive bisection) may be more appropriate than stochastic methods. This is noteworthy from computational perspective.

---

## Comparison to Round 1

| Dimension | Round 1 | Round 2 |
|-----------|---------|---------|
| **Edge weighting** | Missing (major concern) | Implemented + validated |
| **Parameter justification** | Weak | Strong (400-run validation) |
| **Scalability** | Demonstrated | Demonstrated + validated |
| **Optimization quality** | Adequate | Good (improved with weighting) |
| **Implementation details** | Sparse | Comprehensive |

**Overall**: From "adequate implementation" → "solid computational contribution"

---

## Scoring Rationale

**Score**: 3.5/4.0 (Strong Accept with Minor Revisions)

### Why not 3.0?
The revisions are substantial:
- Edge-weighted optimization implemented correctly
- +3.2% performance improvement achieved
- Comprehensive validation (400 runs)
- Scalability demonstrated nationally

### Why not 4.0?
Opportunities remain for computational improvement:
- Hypergraph formulation could provide additional benefit
- Multi-objective optimization could address broader criteria
- Parallel implementation (ParMETIS) would enable larger-scale problems

**Note**: These are enhancements, not deficiencies. Current work is publication-quality. A 4.0 score would require pushing computational boundaries further.

---

## Detailed Section Comments

### Section 3.9: Edge-Weighted Optimization
**Quality**: Good implementation

**Strengths**:
- Correct METIS API usage
- Appropriate weight computation
- Empirical validation
- Honest assessment of modest improvement

**Suggestions**:
- Consider testing nonlinear weight functions (e.g., w(e) = length^α for α > 1) to penalize long boundaries more heavily
- Analyze correlation between boundary length and edge counts more deeply—0.68 correlation suggests opportunity for better encoding

### Section 4.5: Parameter Sensitivity
**Quality**: Comprehensive validation

The 400-run analysis is thorough and validates algorithmic stability well.

**Computational note**: The perfect reproducibility finding suggests that for this problem class, METIS's randomization provides no benefit. Authors might consider using deterministic coarsening (if METIS supports it) to reduce unnecessary randomness.

---

## Publication Recommendation

**Recommendation**: Strong Accept with Minor Revisions

**Conditional on**: Minor discussion additions (see below)

**Additions that would strengthen paper**:
1. Brief discussion of hypergraph formulation as future work (2-3 sentences)
2. Mention of ParMETIS for large-scale problems (1-2 sentences)
3. Note about multi-objective optimization opportunities (2-3 sentences)

**Estimated effort**: 1-2 hours to add these notes

**Venue suitability**:
- **APSR/JOP**: Yes—computational rigor is sufficient
- **SIAM journals**: Possibly with expanded computational analysis
- **PPoPP/IPDPS**: Short paper focusing on perfect reproducibility finding

---

## Summary for Authors

Good progress on computational aspects. Edge-weighted optimization is correctly implemented and provides meaningful improvement. Parameter sensitivity analysis is comprehensive.

**Strengths**:
- Correct METIS usage
- Meaningful performance improvement (+3.2%)
- Comprehensive validation
- National-scale demonstration

**Opportunities** (not requirements):
- Hypergraph formulation (future work)
- Multi-objective optimization (future work)
- Parallel implementation for larger problems (future work)

With minor discussion additions noting these opportunities, the paper would be strengthened. Current work is already publication-quality.

**Recommendation**: Accept with minor revisions (add future work discussion).
