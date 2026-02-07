# Review: Slice-Based Cross-Census Validation for Congressional Redistricting Algorithms

**Reviewer**: May Yuan (University of Texas)
**Expertise**: Spatial algorithms, census data, temporal GIS
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper tackles a critical and under-addressed problem: validating spatial algorithms across non-stationary temporal datasets. As someone who has worked extensively with census data evolution, I appreciate the authors' recognition that naive cross-census comparisons are confounded by tract boundary changes and demographic shifts. The proposed slice-based validation framework shows promise for isolating algorithm behavior from data evolution.

However, the paper critically underspecifies the temporal aspects of the validation framework. The treatment of census tract changes—splits, merges, and reconfigurations—is mentioned but not detailed. The methodology for identifying "persistent tract centroids" is unclear: do centroids persist spatially, semantically (same GEOID), or both? The paper needs substantial strengthening in its treatment of temporal geographic data processing and change detection.

## Score

**Score**: 2/4 — **Weak Accept**

## Major Issues (Blocking)

### M1: Census Tract Change Methodology Underspecified
The paper claims to use "persistent tract centroids" but doesn't explain how these are identified. Between 2000-2020, approximately 15-20% of census tracts split or merged. The methodology must specify: (1) How are tract correspondences established across census years? (2) What happens to tracts that split into 2+ child tracts? (3) How are population-weighted centroids calculated when tract boundaries change? This is the foundation of the entire validation framework.

### M2: Temporal Validation Design Not Justified
Why is cross-census validation the right approach? The paper should compare against alternative designs: (1) Within-census cross-validation (spatial folds), (2) Bootstrap resampling within a single census, (3) Synthetic data with controlled demographic shifts. Without this comparison, readers cannot assess whether the added complexity of cross-census validation is justified.

### M3: Demographic Shift Quantification Missing
The paper claims geographic characteristics dominate demographic shifts, but provides no quantitative support. What are the magnitudes of population changes within slices across census years? How do racial/ethnic compositions change? Without this context, the claim that algorithm variance is primarily geographic is unsubstantiated.

## Minor Issues

### m1: Census Data Processing Pipeline Not Described
What census products are used? TIGER/Line shapefiles? Summary File 1? How are geometries and demographics joined? What resolution (tract vs block group)? These details matter for reproducibility.

### m2: Missing Discussion of 2020 Census Challenges
The 2020 census had significant differential privacy noise injection and pandemic-related data quality issues. How do these affect the validation framework? Should 2020 be treated differently?

### m3: Slice Temporal Stability Not Measured
Do the slice boundaries themselves change across census years as tract centroids shift? If so, how much? This affects the claim that slices provide "stable validation regions."

### m4: No Discussion of Intercensal Estimates
The Census Bureau publishes annual population estimates between decennial censuses. Could these be used to validate algorithm performance at finer temporal granularity?

## Strengths

1. **Important problem**: Cross-census algorithm validation is genuinely needed but rarely done rigorously.
2. **Appropriate data scope**: Using all three modern censuses (2000, 2010, 2020) is commendable.
3. **Temporal GIS awareness**: The authors recognize that census geography is non-stationary, which many papers ignore.
4. **Potential impact**: If executed well, this framework could become a standard methodology for redistricting research.

## Questions for Authors

1. Have you used the Census Bureau's "tract relationship files" to track tract changes?
2. How do you handle Puerto Rico, which has different census geography (municipios)?
3. What percentage of tract centroids shift by >1km, >5km, >10km between census cycles?
4. Have you validated against known redistricting case studies (e.g., Pennsylvania 2018 court case)?
5. Can this framework be applied to other decadal datasets (e.g., Canadian census, European administrative boundaries)?

## Recommendations

- Add a detailed "Census Data Processing" section describing tract correspondence methodology
- Include a figure showing tract boundary evolution for an example state (e.g., Texas, which has high growth and many tract changes)
- Quantify demographic changes within slices: create a table showing mean/std population change, racial composition change across census years
- Provide reproducibility artifacts: code for tract matching, list of persistent tract GEOIDs used
- Discuss limitations: which states/regions have highest tract instability and how this affects validation

---

**Verdict**: **Major Revisions Required**

**Confidence**: High — I have extensive experience with census data temporal analysis and am confident these issues must be addressed.
