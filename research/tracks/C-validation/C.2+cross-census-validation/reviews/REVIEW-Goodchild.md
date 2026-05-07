> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Slice-Based Cross-Census Validation for Congressional Redistricting Algorithms

**Reviewer**: Michael Goodchild (UC Santa Barbara)
**Expertise**: GIS theory, spatial analysis, geography
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper addresses a fundamental challenge in geospatial algorithm validation: how to evaluate spatial partitioning methods across temporal datasets when the underlying geographic units are non-stationary. The slice-based validation framework is conceptually sound and addresses a real methodological gap in redistricting research. The approach of using persistent tract centroids to define stable validation regions is clever and well-motivated for SIGSPATIAL's audience.

However, the paper suffers from insufficient treatment of critical geographic considerations. The abstract claims to control for "geographic and demographic confounds" but provides no detail on how spatial autocorrelation, boundary effects, or MAUP (Modifiable Areal Unit Problem) are addressed. The methodology section must demonstrate that the slicing process itself doesn't introduce spatial artifacts that confound the validation results. Additionally, the paper needs stronger engagement with the spatial statistics and GIS methodology literature.

## Score

**Score**: 2.5/4 — **Accept with Major Revisions**

## Major Issues (Blocking)

### M1: Spatial Autocorrelation Not Addressed
The paper validates an algorithm that produces spatially contiguous districts, but the validation slices are themselves spatial regions. This creates inherent spatial autocorrelation in the validation metrics—districts within a slice are not independent observations. The paper must address: (1) How is spatial autocorrelation quantified? (2) How does it affect statistical inference? (3) Are appropriate spatial statistical methods (Moran's I, spatial regression) used to account for this?

### M2: MAUP (Modifiable Areal Unit Problem) Analysis Missing
The choice of slice size and configuration directly affects the results (MAUP). The paper claims slices are created via "spatial clustering of persistent tract centroids" but provides no analysis of: (1) Sensitivity to number of slices, (2) Alternative slicing schemes, (3) Scale effects. This is fundamental to any spatial validation framework and must be demonstrated.

### M3: Boundary Effects and Edge Cases
Districts that cross slice boundaries are not discussed. How are these handled? Are they excluded, split, or assigned to a primary slice? Boundary effects are a major confound in spatial analysis and must be explicitly addressed in the methodology.

## Minor Issues

### m1: Insufficient GIS Literature Engagement
The paper cites redistricting and algorithm papers but largely ignores spatial validation methodology literature. Key missing references: Openshaw's MAUP work, spatial cross-validation methods (Roberts et al. 2017), temporal geographic analysis (Peuquet 2001).

### m2: Tract Boundary Changes Underspecified
The abstract mentions "tract boundary changes" but doesn't quantify how many tracts split, merged, or disappeared between 2000-2010-2020. This is critical context for evaluating the claim that persistent centroids provide stable validation units.

### m3: Coordinate Reference System Not Specified
What CRS is used for centroid calculation and distance metrics? Different projections yield different spatial relationships. This should be specified in the methodology.

### m4: Visualization Quality
Figures showing the slice boundaries overlaid on state boundaries with district results would greatly enhance comprehension. Current description is text-heavy.

## Strengths

1. **Novel methodology**: Slice-based cross-temporal validation is genuinely new and addresses a real gap in redistricting research.
2. **Comprehensive scope**: 50 states across 3 census years is ambitious and valuable for generalizability.
3. **Clear motivation**: The paper articulates why cross-census validation is important and why naive approaches fail.
4. **Appropriate venue**: SIGSPATIAL is the right target—this is fundamentally a spatial algorithms problem.

## Questions for Authors

1. Have you considered using spatial k-fold cross-validation methods (e.g., spatial blocking) instead of slices?
2. How do Alaska and Hawaii (non-contiguous states) fit into the validation framework?
3. What is the typical number of slices per state, and how does this relate to the number of congressional districts?
4. Can you provide quantitative measures of slice stability (e.g., how much do slice boundaries shift if the clustering seed changes)?

## Recommendations

- Add a "Spatial Validation Methodology" subsection that explicitly addresses MAUP, spatial autocorrelation, and boundary effects
- Include a figure showing example slices for 2-3 representative states
- Provide sensitivity analysis for number of slices (e.g., try K=3, 5, 7 slices per state)
- Cite and engage with spatial cross-validation literature from GIScience
- Specify all geographic data processing details: CRS, distance metrics, tract matching criteria

---

**Verdict**: **Major Revisions Required**

**Confidence**: High — My expertise is directly in spatial analysis methodology and I am highly confident these issues are fundamental to the paper's validity.
