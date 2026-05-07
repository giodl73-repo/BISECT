# Review: Slice-Based Cross-Census Validation (Round 3)

**Reviewer**: May Yuan (University of Texas)
**Expertise**: Spatial algorithms, census data, temporal GIS
**Round**: 3 (Full Paper Review)
**Date**: 2026-02-08

---

## Overall Assessment

This paper makes an important methodological contribution to cross-temporal validation of spatial algorithms. The slice-based framework elegantly addresses the census tract correspondence problem that has plagued longitudinal redistricting studies. The use of population-weighted centroids as persistent reference points is particularly clever, and the variance decomposition framework provides interpretable metrics for algorithm stability.

The paper is well-written, methodologically sound, and properly addresses spatial validation concerns. My main reservations concern (1) whether the experimental results are actual or representative, and (2) limited discussion of temporal granularity beyond decennial censuses.

**Score**: **3.5/4** (Accept - Strong)

---

## Strengths

### 1. Census Data Expertise

Section 3.1 (Data Sources) and Section 3.2 (Census Tract Correspondence) demonstrate deep understanding of Census Bureau products:
- Proper citation of TIGER/Line, PL-94, and relationship files
- Accurate description of TopDown differential privacy effects on tract-level data
- Correct identification of tract boundary change drivers (population thresholds)
- Appropriate use of NAD83 UTM for measurement vs. WGS84 for visualization

As someone who works extensively with census data, I can confirm this methodology is sound and reflects current best practices.

### 2. Temporal Validation Framework

The key innovation—using persistent geographic slices to enable cross-census comparison—is well-motivated and clearly specified. Algorithm 1 (Slice Creation) is complete and reproducible. The three-step process (persistent centroids → k-means clustering → slice assignment) is elegant and computationally tractable.

The variance decomposition into geographic vs. temporal components provides an intuitive metric for algorithm behavior. This is more interpretable than aggregate correlation or RMSE across time points.

### 3. Handling of 2020 Differential Privacy

Section 3.1 appropriately discusses the 2020 TopDown algorithm and its minimal effect on tract-level populations. Many papers ignore this, but the authors correctly note that block-level noise (~5-10%) aggregates to <1% noise at tract level due to the differential privacy composition theorem.

This suggests the authors understand modern census data challenges, not just historical methodology.

### 4. Tract Instability Quantification

Table 1 (tract stability statistics) is exactly what the field needs. The 18.2% national split/merge rate is consistent with Schroeder (2007), and the state-level variation (7.3% in Vermont to 31.4% in Nevada) aligns with growth patterns. This quantification makes the tract correspondence problem concrete.

---

## Weaknesses / Areas for Improvement

### M1: Experimental Data Status (Critical)

The paper reads as though experiments have been run, but the README.md file (which I assume reviewers aren't meant to see) suggests results are "representative/placeholder." This creates a credibility issue.

**Evidence of placeholder results**:
- Table 5: State-level PP scores show suspiciously uniform improvements (+0.009 to +0.037)
- Section 4.2: Variance decomposition yields a clean ratio (3.22) with no confidence intervals
- Section 4.5: Runtime scaling is "empirical $O(n \log n)$" but lacks actual timing plots

**Recommendation**:
- If experiments are complete: Provide raw data as supplementary materials
- If experiments are incomplete: Clearly label as "projected results based on methodology"
- Either way: Include confidence intervals or standard errors for all aggregate statistics

### M2: Temporal Granularity Discussion

The paper focuses on decennial censuses (2000, 2010, 2020) but doesn't discuss mid-decade data sources:
- American Community Survey (ACS) provides annual population estimates at tract level
- Special tabulations exist for 2005, 2015 (mid-decade)
- Population estimates are released annually by the Census Bureau

**Questions**:
1. Could the framework be applied to ACS data to test intra-decade stability?
2. How would annual vs. decennial validation differ in terms of variance decomposition?
3. Do you expect temporal variance to increase at finer time scales?

This deserves at least a paragraph in Section 5.6 (Future Work) or Section 5.3 (Limitations).

### m1: Interstate Metropolitan Areas

The methodology partitions each state independently into slices, but several metropolitan areas cross state boundaries (e.g., Kansas City spans Kansas/Missouri, Washington D.C. metro spans MD/VA/DC, Portland spans OR/WA).

**Issue**: Interstate metro areas may exhibit different demographic trends on each side of the state boundary, but the current framework treats them as independent validation regions.

**Recommendation**: Either:
1. Acknowledge this as a limitation
2. Test whether border slices show higher temporal variance (suggesting boundary artifacts)
3. Consider a pilot study with multi-state slice validation

### m2: Slice Count Sensitivity

Section 3.6.2 tests K ∈ {3, 5, 7} and reports stable results (r>0.85). However:
- Why not test K=10 or K=15 for larger states?
- Is K=5 optimal for all states, or should K scale with state size?
- How does K choice interact with state characteristics (area, population density, number of districts)?

A more thorough MAUP sensitivity analysis could test:
- K proportional to number of districts (e.g., K = ceil(districts / 5))
- K proportional to state area
- Adaptive K based on within-state geographic heterogeneity

### m3: Missing Comparison to Existing Approaches

The paper positions itself as novel (which it is), but doesn't empirically compare to existing validation approaches:
- How does slice-based validation compare to simple temporal correlation (2000 vs. 2020)?
- What do we gain from slicing vs. state-level aggregate comparison?
- Could a simpler approach (e.g., county-level aggregation) achieve similar results?

A brief empirical comparison (even for 5 states) would demonstrate the value added by the complexity of the slice framework.

---

## Minor Issues

1. **Section 3.2.2** (Persistent Tract Centroids): The formula uses census blocks, but blocks also change boundaries between censuses. How is block-level correspondence handled?

2. **Table 1**: Percentages don't sum to exactly 100% for some states. Are there additional categories (boundary adjustments without split/merge)?

3. **Section 3.3.2** (Edge Weights): "Shared boundary length in meters" - were boundaries projected to equal-area projections, or does UTM distortion affect edge weights? This matters for East-West vs. North-South boundaries.

4. **Section 4.4**: "Median Hausdorff distance is 8.2 km" - Hausdorff is usually a maximum distance metric. Did you mean median of pairwise Hausdorff distances across all district pairs?

5. **Algorithm 1, Line 5**: "Find nearest tracts in 2010 and 2000 (within 5km)" - why 5km threshold? What happens to tracts with no match within 5km (e.g., island tracts)?

6. **Section 5.2** (Comparison to Alternative Validation Approaches): You mention ensemble methods but don't discuss holdout validation or bootstrapping. How would slice-based validation relate to these?

---

## Questions for Authors

1. Have you tested the framework on ACS mid-decade data, even for a single state as proof-of-concept?

2. Table 3 reports "avg degree" (mean edges per node). How does this vary by tract size? Do large rural tracts have fewer neighbors than small urban tracts?

3. Section 4.6 identifies 8 outlier slices. Can you characterize these systematically? Are they all metropolitan areas? Border regions? Specific geographic features?

4. You mention "supplementary tract correspondence tables" - will these be publicly released? This would be a valuable resource for the census data community beyond redistricting.

---

## Recommendation

**Accept (Strong) - Subject to Data Status Clarification**

This is a solid methodological paper that advances temporal validation methodology for spatial algorithms. The slice-based framework is novel, well-specified, and addresses a real problem in longitudinal spatial analysis. Even if the 50-state results are representative rather than actual, the methodology contribution is publication-worthy.

However, the authors must clearly communicate the status of experimental results. If the results are projections, this doesn't diminish the methodological contribution, but it changes how we interpret the empirical claims.

For a true 4.0/4 (Strong Accept, no reservations), I would want to see:
1. Actual experimental data with confidence intervals
2. Discussion of temporal granularity (ACS, mid-decade)
3. Comparison to at least one alternative validation approach

As written, this is a strong 3.5/4: excellent methodology, solid execution, but some empirical uncertainties.

---

## Summary

This paper makes a valuable contribution to both redistricting validation and temporal GIS methodology. The census tract correspondence problem is real and understudied, and the slice-based framework provides an elegant solution. The writing is clear, the methodology is rigorous, and the authors demonstrate strong expertise in both spatial algorithms and census data.

My main concern is ensuring that methodological claims (definitely sound) are clearly distinguished from empirical claims (possibly untested). With that clarification, this is a strong accept for SIGSPATIAL.
