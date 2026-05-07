> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Cross-Census Temporal Stability
## Reviewer: Dr. Justin Solomon (MIT)
**Expertise**: Computational geometry, optimal transport, hierarchical structures
**Date**: 2026-02-08
**Score**: 3.0/4.0 (Accept)

---

## Overall Assessment

This paper investigates whether hierarchical partitioning structures provide better temporal stability than flat k-way partitioning—a question with both theoretical and practical merit. From a computational geometry perspective, the hierarchical stability advantage makes intuitive sense: coarse-level geometric structure should be more robust to local perturbations than fine-grained optimization.

**Strengths**:
1. **Geometric intuition**: Hierarchical splits follow natural geographic divisions
2. **Proper methodology**: Compares true hierarchical bisection vs flat k-way
3. **Reproducible**: Clear description of geometric data and algorithms
4. **Practical relevance**: Real-world redistricting application

**Concerns**:
1. **Weak geometric analysis**: Doesn't analyze shape, compactness, or geometry
2. **No optimal transport perspective**: Natural framework for comparing partitions
3. **Missing hierarchical structure verification**: Claims hierarchy without proving it
4. **Compactness-stability tradeoff unexplored**

---

## Major Issues (P1 - Blocking)

### P1.1: Hierarchical Structure Not Verified
**Issue**: Paper claims recursive bisection creates hierarchical structure but provides no evidence:
- No dendrograms showing tree structure
- No verification that parent-child relationships preserve
- No analysis of whether top-level splits remain stable

**Fix Required**: Add Section 3.5 "Hierarchical Structure Validation":
1. **Dendrogram visualization**: Show full binary tree for Alabama (7 districts)
2. **Parent-child stability**: Measure how many 2010 parent-child pairs remain in 2020
3. **Level-wise analysis**: Show stability at each tree level (root → leaves)

**Example analysis needed**:
```
Alabama 7 districts = 3-level tree:
Level 0: State → [North, South]
Level 1: North → [NW, NE], South → [SW, South-central]
Level 2: Final 7 districts

Measure: Do 2010's [North, South] split match 2020's top split?
```

Without this, "hierarchical" claim is unsubstantiated.

---

### P1.2: Geometric Analysis Completely Missing
**Issue**: This is fundamentally a geometric problem (partitioning 2D space) but paper ignores geometry:
- No analysis of district shapes
- No compactness metrics (Polsby-Popper, Reock, Convex Hull)
- No discussion of geographic features (rivers, mountains, urban boundaries)

**Fix Required**: Add Section 4.3 "Geometric Analysis":
1. **Compactness comparison**: Do stable districts maintain better shapes?
2. **Shape evolution**: Measure shape similarity 2010 → 2020
3. **Geographic alignment**: Do hierarchical splits follow natural boundaries?

**Hypothesis**: Stable districts should be more compact (less gerrymandered).

---

## Major Issues (P2 - Important)

### P2.1: Optimal Transport Perspective Missing
**Issue**: Comparing partitions across time is naturally framed as optimal transport problem:
- 2010 partition = source distribution
- 2020 partition = target distribution
- Stability = Wasserstein distance between distributions

**Benefits of OT framework**:
1. Principled metric (Wasserstein distance)
2. Accounts for geographic distance (tracts moving far = worse)
3. Provides visual flow maps
4. Connects to geometric measure theory

**Recommendation**: Add Section 3.4 "Optimal Transport Formulation":
- Define partition as probability measure on 2D space
- Measure stability as W₂ distance
- Show that recursive bisection minimizes transport cost

---

### P2.2: No Analysis of Spatial Autocorrelation
**Issue**: Geographic data has spatial structure (neighboring tracts are similar). Paper doesn't analyze:
- Moran's I statistic for spatial autocorrelation
- Whether stable regions exhibit stronger spatial clustering
- Local indicators of spatial association (LISA)

**Recommendation**: Add spatial statistics showing:
- High spatial autocorrelation → easier to maintain stability
- Low autocorrelation → inherently unstable regardless of method

---

### P2.3: Compactness-Stability Tradeoff Unexplored
**Issue**: Paper mentions "performance-stability tradeoff" but doesn't analyze geometric tradeoffs:
- Do more compact districts provide better stability?
- Does optimizing for stability sacrifice compactness?
- Can you jointly optimize both?

**Recommendation**: Add multi-objective analysis:
- Plot Pareto frontier: compactness vs stability
- Show whether recursive bisection dominates n-way on both metrics
- Discuss whether there's inherent tradeoff

---

### P2.4: Census Boundary Changes Create Geometric Artifacts
**Issue**: 26% of tracts have boundary changes. This creates:
- Artifacts in stability measurement (tract splits/merges)
- Invalid comparisons (2010 tract ≠ 2020 "same" tract)
- Geographic discontinuities

**Recommendation**: Use geometric approach:
- Represent tracts as polygons, not discrete IDs
- Measure overlap area between 2010 and 2020 districts
- Account for tract boundary changes via area-weighted stability

---

## Minor Issues (P3 - Nice to Have)

### P3.1: No Visualization of Actual Districts
**Missing**: Maps showing 2010 vs 2020 districts
**Need**: Side-by-side maps for one state with stability coloring
**Would strengthen**: Visual understanding of what "stable" means geometrically

---

### P3.2: Edge Weighting Scheme Not Geometrically Justified
**Issue**: Paper uses "5x weight at 40% minority threshold" but doesn't explain geometric rationale:
- Why this threshold?
- How does weighting affect geometric properties?
- Is there optimal weighting for stability?

**Recommendation**: Analyze weighting sensitivity geometrically.

---

### P3.3: No Comparison to Geometric Clustering Methods
**Issue**: Paper only compares METIS modes. What about geometric algorithms?
- k-means clustering on tract centroids
- Spectral clustering with geographic distance
- Voronoi-based partitioning

**Recommendation**: Add geometric baseline showing METIS vs pure geometric methods.

---

## Detailed Technical Comments

### Spatial Data Representation
⚠️ **Concern**: Paper treats tracts as graph nodes but doesn't discuss:
- Tract sizes (area variability)
- Shape complexity (some tracts are highly irregular)
- Boundary topology (holes, enclaves)

These geometric properties affect partitioning behavior.

### Graph Construction
✅ **Good**: Adjacency via shared boundaries (queen contiguity)
⚠️ **Concern**: Edge weights based solely on demographics, ignore:
- Geographic distance
- Boundary length (long boundaries = weaker connection)
- Natural barriers (rivers, highways)

### Metric Choice
**Population Disruption Rate**: Treats all tracts equally regardless of:
- Geographic size (large rural tract = small urban tract)
- Distance moved (tract shifts to adjacent district vs far district)
- Compactness impact

**Better geometric metric**: Area-weighted Hausdorff distance between district shapes.

---

## Recommendations for Revision

### Tier 1 (P1 - Must Fix)
1. **Verify hierarchical structure** (dendrograms, level-wise stability)
2. **Add geometric analysis** (compactness, shape similarity)

### Tier 2 (P2 - Strongly Recommended)
1. Add optimal transport formulation (Wasserstein distance)
2. Include spatial autocorrelation analysis (Moran's I)
3. Analyze compactness-stability tradeoff
4. Handle census boundary changes geometrically (area-weighted)

### Tier 3 (P3 - Would Strengthen)
1. Add district maps showing 2010 vs 2020
2. Justify edge weighting scheme geometrically
3. Compare to pure geometric clustering methods

---

## Recommendation

**Score: 3.0/4.0 (Accept)**

This paper addresses an interesting question about hierarchical vs flat partitioning stability. The finding that hierarchical structure provides modest stability advantages is valuable, though the effect size (1.1%) is smaller than geometric intuition might suggest.

The major limitation is inadequate geometric analysis (P1.2). This is fundamentally a problem about partitioning 2D space, yet paper ignores:
- Shape and compactness of districts
- Geographic features and natural boundaries
- Spatial autocorrelation and clustering

Adding geometric analysis would:
1. Explain WHY hierarchical splits are more stable (follow geographic structure)
2. Show WHETHER stable districts are more compact
3. Validate that hierarchical structure actually exists (dendrograms)

The hierarchical structure verification (P1.1) is essential. Without dendrograms or tree-level analysis, we can't verify that "hierarchical" methods actually create hierarchies that persist over time.

**Venue Fit**: ACM-KDD is reasonable but consider:
- **SIGSPATIAL**: Focus on geometric/spatial aspects
- **Computational Geometry**: If geometric theory strengthened
- **SODA**: If theoretical analysis added

With P1 items fixed and stronger geometric analysis, this is a solid accept.

---

## Questions for Author Rebuttal

1. **Hierarchical verification**: Can you show dendrograms proving hierarchical structure exists and persists?

2. **Geometric properties**: How do district shapes (compactness) compare between methods and over time?

3. **Optimal transport**: Have you considered measuring stability via Wasserstein distance?

4. **Spatial clustering**: Is stability advantage related to spatial autocorrelation of demographics?

5. **Boundary artifacts**: How do census tract boundary changes affect stability measurement?

---

## Theoretical Implications

From computational geometry perspective, expect:

**Hierarchical advantage should arise from**:
1. Coarse structure (state → regions) more geometrically stable than fine structure
2. Recursive splits follow principal axes (PCA-like), which evolve smoothly
3. Top-level cuts maximize modularity, robust to local perturbations

**Geometric hypothesis**:
```
Hierarchical partitions minimize ∫∫ ||x - c_i||² dx over district interiors,
where c_i are district centroids. This objective evolves smoothly with
demographic changes, unlike discrete k-way optimization.
```

Testing this hypothesis would elevate empirical observation to theoretical understanding.

---

## Overall Assessment

Good empirical work that needs stronger geometric foundation. Fix hierarchical verification and add geometric analysis, and this becomes a solid contribution bridging graph partitioning and computational geometry.

The finding that hierarchy provides stability is intuitive from geometry perspective, but needs quantitative geometric evidence beyond population statistics.
