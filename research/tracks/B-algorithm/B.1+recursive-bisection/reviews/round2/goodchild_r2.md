# Round 2 Review - Michael Goodchild (UC Santa Barbara)
**Date**: 2026-02-07
**Round**: 2
**Paper**: Recursive Bisection for Congressional Redistricting

---

## Summary Assessment

**Score**: 3.0/4.0 (Accept with Minor Revisions)
**Change from Round 1**: 0.0 points (no change)

The authors have made substantial improvements to the paper's empirical rigor, legal analysis, and computational methodology. However, my primary Round 1 concerns about GIS methodology—specifically coordinate projection impacts on geometric calculations and spatial data quality assessment—remain unaddressed. The paper continues to treat census tract geometries as "given" without discussing map projection distortions, area calculation accuracy, or boundary length measurement uncertainties. These gaps prevent a score increase.

---

## Why Score Unchanged

### My Round 1 Concerns (Still Unaddressed)

#### 1. **Map Projection and Coordinate System** (UNRESOLVED)
**Severity**: Moderate

The paper computes geometric quantities (boundary lengths, areas, Polsby-Popper scores) from census tract geometries but never discusses:

**Missing methodological details**:
- Which coordinate reference system (CRS) is used for each state?
- Are calculations performed in geographic (lat/lon) or projected coordinates?
- If projected, which projection (Albers Equal Area, State Plane, Web Mercator)?
- How are area and distance distortions accounted for?

**Why this matters**:
1. **Area distortions**: Geographic coordinates (lat/lon) produce severe area distortions, especially at northern latitudes (Alaska, Minnesota, etc.)
2. **Distance distortions**: Boundary length calculations in geographic coordinates are inaccurate
3. **Compactness impacts**: Polsby-Popper scores depend on area/perimeter ratios—projection choice affects results
4. **Multi-state comparisons**: Comparing compactness across states requires consistent projection methodology

**What I need to see**:
- Statement of CRS used for each state
- Justification of projection choice (e.g., "Albers Equal Area chosen to minimize area distortion")
- Sensitivity analysis: How much do compactness scores vary across projection choices?
- Discussion of projection impact on multi-state comparisons

**Current status**: Section 3 mentions "GeoPandas" but doesn't specify CRS or discuss projection methodology.

#### 2. **Spatial Data Quality** (UNRESOLVED)
**Severity**: Moderate

The paper uses Census Bureau TIGER/Line tract boundaries but doesn't discuss data quality issues:

**Missing considerations**:
- Boundary precision (TIGER/Line uses generalized boundaries, not survey-quality)
- Topological consistency (do adjacent tracts share exact boundaries, or are there gaps/overlaps?)
- Temporal consistency (2000 vs. 2010 vs. 2020 tract boundary changes)
- Resolution impacts (simplified vs. detailed geometries)

**Why this matters**:
1. **Boundary length accuracy**: Generalized boundaries underestimate true perimeter
2. **Compactness measurement**: PP scores depend on perimeter accuracy
3. **Adjacency determination**: Topological inconsistencies can create false adjacencies or missed connections
4. **Reproducibility**: Others using same data may get slightly different results due to geometry processing differences

**What I need to see**:
- Discussion of TIGER/Line geometry resolution and accuracy
- Statement about topological processing (are gaps/overlaps corrected?)
- Sensitivity analysis: How do compactness scores change with geometry simplification/resolution?

**Current status**: Paper mentions "Census Bureau data" but doesn't discuss geometric data quality.

---

## What Improved (Other Dimensions)

While my GIS-specific concerns remain unaddressed, I note substantial improvements in other areas:

### 1. **Empirical Rigor**
- Parameter sensitivity analysis (404 runs) is exceptional
- 50-state VRA analysis is comprehensive
- Process type disaggregation shows analytical sophistication

### 2. **Legal Analysis**
- VRA section is now comprehensive (~3,500 words)
- *Rucho* analysis demonstrates constitutional law sophistication
- Post-*Rucho* pathway is well-articulated

### 3. **Computational Methodology**
- Edge-weighted optimization properly implemented
- METIS usage validated comprehensively
- Scalability demonstrated nationally

**These improvements are substantial**—they just don't address my specific GIS methodology concerns.

---

## Detailed GIS Concerns

### 1. **Coordinate Reference Systems**

**Issue**: Geometric calculations depend critically on CRS choice

**Example**: Minnesota tract area calculation
- **Geographic CRS** (EPSG:4269, lat/lon): 1,000 km² tract → distorted area at 45°N latitude
- **Projected CRS** (EPSG:26915, UTM Zone 15N): Same tract → accurate area
- **Different projection** (EPSG:3857, Web Mercator): Same tract → different area

**For this paper**:
- Boundary length calculations for edge weighting (Section 3.9)
- Area calculations for Polsby-Popper scores (Section 4)
- Multi-state compactness comparisons

**All depend on CRS choice**, but paper never states which is used.

**Recommendation**: Add methodological subsection stating:
1. Which CRS used for each state (or if using state-specific equal-area projections)
2. Why that choice is appropriate for geometric calculations
3. How multi-state comparisons account for projection differences

**Estimated effort**: 2-3 hours to document current practice + 1-2 days for sensitivity analysis if not already using appropriate projections

### 2. **Geometry Processing Pipeline**

**Issue**: Census geometries require processing before analysis

**Typical pipeline**:
1. Download TIGER/Line shapefiles
2. Load into GIS software
3. Check/fix topological errors (gaps, overlaps, invalid polygons)
4. Simplify geometries (if needed for performance)
5. Reproject to analysis CRS
6. Compute geometric properties (area, perimeter)

**Paper should document**:
- Which steps are performed?
- What topology corrections are applied?
- Is geometry simplified? If so, what tolerance?
- At what stage is CRS transformation performed?

**Why this matters**: Others attempting to reproduce results need to know exact processing steps.

**Recommendation**: Add "Spatial Data Processing" subsection documenting:
- Geometry loading and validation procedures
- Topology correction methodology
- CRS transformation procedures
- Geometric computation methods

**Estimated effort**: 3-4 hours to document existing pipeline

### 3. **Sensitivity to Geometric Resolution**

**Issue**: Compactness scores depend on geometry detail level

**Example**: Illinois district with highly detailed boundary
- **Original resolution** (10,000 vertices): PP = 0.240
- **Simplified (1,000 vertices)**: PP = 0.255 (+6.3%)
- **Highly simplified (100 vertices)**: PP = 0.280 (+16.7%)

**For this paper**:
- Are all analyses using same geometric resolution?
- How would results change with different resolution?
- Is TIGER/Line resolution adequate for compactness analysis?

**Recommendation**: Brief sensitivity analysis showing compactness score stability across resolution levels.

**Estimated effort**: 2-3 days for sensitivity analysis

---

## Minor GIS Observations

### 1. **Alaska and Hawaii**
The paper mentions these states but doesn't discuss special projection considerations:
- **Alaska**: Requires Albers Alaska projection due to extreme latitude distortions
- **Hawaii**: Island geography requires careful distance measurement
- **Multi-state visualization**: Both require special handling for national maps

**Minor suggestion**: Brief note about special projection handling for these states.

### 2. **Edge Length Computation**
Section 3.9 uses boundary lengths for edge weighting but doesn't specify:
- Geodesic distance vs. planar distance?
- Haversine formula (for lat/lon) vs. Euclidean (for projected)?
- Accuracy of shared boundary detection?

**Minor suggestion**: Add sentence clarifying distance computation method.

---

## What Would Increase Score to 3.5/4.0

**Requirements for +0.5 points**:
1. **Document CRS methodology**: State which projections used, justify choices (~500 words)
2. **Spatial data quality discussion**: TIGER/Line accuracy, topology processing (~300 words)
3. **One of the following**:
   - Projection sensitivity analysis (show compactness stability across projections)
   - OR Geometry resolution sensitivity (show PP score stability across simplification levels)

**Estimated total effort**: 3-4 days

**Why this would justify 3.5**: Would demonstrate understanding of GIS best practices and ensure results are robust to geometric processing choices.

---

## Comparison to Round 1

| Dimension | Round 1 | Round 2 |
|-----------|---------|---------|
| **GIS methodology** | Minimal | Still minimal ⚠️ |
| **Projection discussion** | None | None ⚠️ |
| **Spatial data quality** | None | None ⚠️ |
| **Empirical rigor** | Good | Exceptional ✓ |
| **Legal analysis** | Weak | Strong ✓ |
| **Computational methodology** | Adequate | Strong ✓ |

**Overall**: Substantial improvements in most areas, but GIS-specific concerns remain unaddressed.

---

## Scoring Rationale

**Score**: 3.0/4.0 (Accept with Minor Revisions)

### Why not 2.5?
The paper's core methodology is sound:
- Graph partitioning approach is appropriate
- METIS implementation is correct
- Scalability is demonstrated
- Results are plausible

GIS methodology gaps are **documentation issues**, not fundamental flaws. I have no reason to believe results are wrong—I just can't verify geometric calculation accuracy without methodology documentation.

### Why not 3.5?
GIS methodology documentation remains insufficient:
- No CRS specification
- No projection justification
- No spatial data quality discussion
- No sensitivity analysis for geometric choices

These gaps prevent full confidence in geometric calculation accuracy and reproducibility.

---

## Publication Recommendation

**Recommendation**: Accept with Minor Revisions

**Conditional on**: Adding GIS methodology documentation (CRS, spatial data quality, one sensitivity analysis)

**Venue suitability**:
- **APSR/JOP**: Yes—after GIS methodology documentation
- **Cartography and GIS journals**: Not suitable without expanded GIS analysis
- **Geography journals**: Would require more extensive spatial analysis discussion

**Note**: My concerns are specific to GIS methodology. For political science journals (APSR/JOP), current level may be acceptable to other reviewers, but I believe proper GIS methodology documentation would strengthen the paper and improve reproducibility.

---

## Summary for Authors

You've made substantial improvements to empirical rigor, legal analysis, and computational methodology. However, my primary concerns about GIS methodology remain unaddressed. These are not fundamental flaws—they're documentation gaps that affect reproducibility and confidence in geometric calculations.

**Priority additions**:
1. **CRS documentation** (required): State which projections used for each state
2. **Spatial data quality** (required): Discuss TIGER/Line accuracy and topology processing
3. **Sensitivity analysis** (required): Show results are robust to geometric processing choices

**Estimated effort**: 3-4 days total

With these additions, I would increase score from 3.0 to 3.5. Without them, I maintain my Round 1 score of 3.0 despite improvements in other areas.

**The paper is publishable as-is for political science venues**, but GIS methodology documentation would strengthen it and improve reproducibility for researchers attempting to replicate your work.
