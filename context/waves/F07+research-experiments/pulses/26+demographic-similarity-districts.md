---
uuid: 26a1b6
slug: demographic-similarity-districts
name: 26: Demographic Similarity Districts
wave_uuid: f7a1b7
created: '2026-01-25'
status: PLANNED
---
# E26: Demographic Similarity Districts

**Status**: 📋 PLANNED
**Priority**: Research
**Proposed**: January 15, 2026
**Estimated Complexity**: Very High (18-22 hours)
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

The current redistricting system:
- Optimizes for geographic compactness and equal population
- Districts contain demographically diverse populations
- Geographic proximity determines district membership
- Demographics may vary widely within a district

## Goal

**Research Experiment**: Implement "demographic similarity districts" where district membership is determined by demographic characteristics rather than geography:

1. **Similarity-based clustering**: Group census tracts by demographic similarity (age, race, income, education)
2. **Non-geographic districts**: Districts are "weather bands" of demographically homogeneous populations
3. **Equal population**: Maintain 435 equal-population districts, but ignore geography entirely
4. **Multiple dimensions**: Create separate district maps for different demographic axes:
   - Age cohort districts (Gen Z, Millennials, Gen X, Boomers, Silent)
   - Racial cohort districts (majority White, majority Black, majority Hispanic, majority Asian, diverse)
   - Income cohort districts (low, lower-middle, middle, upper-middle, high)
   - Education cohort districts (no HS, HS, some college, bachelor's, graduate)

**Example**: Age-Based Districts (2020)
- District 1: All census tracts with median age 20-30 (Gen Z + young Millennials) across all states
- District 2: All census tracts with median age 30-40 (Millennials) across all states
- ...
- District 435: Mixed age tracts to balance population

**Research questions**:
- What happens when districts are demographically homogeneous?
- Does geographic contiguity matter for representation?
- How do "like with like" districts affect governance?
- What are compactness implications when ignoring geography?

## Implementation Plan

### Phase 1: Demographic Feature Engineering
- Load tract-level demographics for all census tracts nationwide
- Compute demographic features per tract:
  - Age distribution (median age, age cohort percentages)
  - Racial composition (White NH%, Black NH%, Hispanic%, Asian NH%, Other%)
  - Income statistics (median household income, poverty rate)
  - Education attainment (% bachelor's+, % graduate degree)
- Normalize features for clustering
- Files to create:
  - `scripts/experimental/compute_tract_demographics.py` - Feature computation
  - `data/experimental/tract_demographics_2020.csv` - Demographic features

### Phase 2: Similarity Metric Definition
- Define distance metric for demographic similarity
- Options:
  - Euclidean distance in feature space
  - Cosine similarity
  - Manhattan distance
  - Custom weighted metric
- Compute pairwise similarity matrix (expensive: 74K × 74K)
- Files to create:
  - `scripts/experimental/demographic_similarity.py` - Distance functions

### Phase 3: Demographic Clustering
- Cluster census tracts into 435 groups by demographic similarity
- Algorithm options:
  - K-means clustering (fast, may not balance population)
  - Hierarchical clustering (slow, better control)
  - Spectral clustering
  - Custom constraint-based clustering (equal population)
- Constraint: Each cluster must have equal population (~769K for 2020)
- Output: District assignments ignoring geography
- Files to create:
  - `scripts/experimental/demographic_clustering.py` - Clustering algorithm
  - `scripts/experimental/constrained_clustering.py` - Equal-population constraint

### Phase 4: Dimension-Specific Districts
- Create separate district maps for each demographic dimension:
  - Age-based districts
  - Race-based districts
  - Income-based districts
  - Education-based districts
- For each dimension:
  - Cluster tracts by similarity on that dimension only
  - Balance population across clusters
  - Output district assignments
- Files to create:
  - `scripts/experimental/age_based_districts.py` - Age clustering
  - `scripts/experimental/race_based_districts.py` - Race clustering
  - `scripts/experimental/income_based_districts.py` - Income clustering
  - `scripts/experimental/education_based_districts.py` - Education clustering

### Phase 5: Visualization
- Generate maps showing demographic districts:
  - Color tracts by assigned district (will be geographically fragmented)
  - Show "weather bands" pattern (similar demographics across geography)
  - National map with district boundaries (non-contiguous)
- Create demographic profile visualizations:
  - Per-district demographic composition
  - Homogeneity scores per district
- Files to create:
  - `scripts/experimental/visualize_demographic_districts.py` - Maps
  - `scripts/experimental/visualize_district_profiles.py` - Charts

### Phase 6: Analysis & Comparison
- Compute metrics:
  - **Homogeneity**: Within-district demographic variance
  - **Geographic fragmentation**: Number of disconnected components per district
  - **Compactness**: Polsby-Popper scores (will be extremely low)
  - **Population balance**: Verify ±0.5% target
- Compare to standard geographic approach:
  - Homogeneity improvement
  - Compactness cost
  - Representation trade-offs
- Files to create:
  - `scripts/experimental/analyze_demographic_districts.py` - Metrics

### Phase 7: Documentation
- Document demographic clustering methodology
- Explain non-geographic representation concept
- Clarify constitutional challenges (massive violations)
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `scripts/experimental/compute_tract_demographics.py` - Feature engineering
2. `scripts/experimental/demographic_similarity.py` - Distance metrics
3. `scripts/experimental/demographic_clustering.py` - General clustering algorithm
4. `scripts/experimental/constrained_clustering.py` - Equal-population constraint
5. `scripts/experimental/age_based_districts.py` - Age-specific districts
6. `scripts/experimental/race_based_districts.py` - Race-specific districts
7. `scripts/experimental/income_based_districts.py` - Income-specific districts
8. `scripts/experimental/education_based_districts.py` - Education-specific districts
9. `scripts/experimental/visualize_demographic_districts.py` - Fragmented maps
10. `scripts/experimental/visualize_district_profiles.py` - Demographic charts
11. `scripts/experimental/analyze_demographic_districts.py` - Comparative metrics
12. `data/experimental/tract_demographics_2020.csv` - Feature data

### Modified Files
1. `ARCHITECTURE.md` - Document experimental demographic system

## Testing Plan

1. **Demographic feature validation**: Check feature ranges, distributions
   ```bash
   python scripts/experimental/compute_tract_demographics.py --year 2020 --validate
   ```

2. **Similarity metric test**: Verify distance function produces reasonable results
   ```bash
   python scripts/experimental/demographic_similarity.py --sample 1000 --visualize
   ```

3. **Small-scale clustering**: Test on subset (e.g., single state)
   ```bash
   python scripts/experimental/demographic_clustering.py --state CA --n-clusters 52 --year 2020
   ```

4. **Population constraint validation**: Verify all clusters balanced
   ```bash
   python scripts/experimental/constrained_clustering.py --validate --year 2020
   ```

5. **Full national run**: All 435 districts, all dimensions
   ```bash
   python scripts/experimental/demographic_clustering.py --year 2020 --version demo_v1
   ```

6. **Quantitative validation**:
   - Verify population balance (±0.5%)
   - Measure homogeneity improvement vs geographic districts
   - Count disconnected components per district
   - Compute compactness scores (expect near-zero)

## Benefits

- **Research contribution**: Novel non-geographic representation model
- **Homogeneity analysis**: Quantify effects of demographically uniform districts
- **Representation theory**: Explore "descriptive representation" vs "geographic representation"
- **Academic publication**: Unique thought experiment
- **Policy discussion**: Inform debate on identity-based representation

## Success Criteria

- [ ] Demographic features computed for all 74K census tracts (2020)
- [ ] Similarity metric defined and validated
- [ ] Clustering produces 435 equal-population districts
- [ ] All districts within ±0.5% of target population
- [ ] Dimension-specific districts created (age, race, income, education)
- [ ] Visualization maps show geographic fragmentation
- [ ] Homogeneity metrics computed and compared to baseline
- [ ] Geographic fragmentation quantified (components per district)
- [ ] Documentation explains massive constitutional violations

## Estimated Complexity

**Effort**: 18-22 hours
- Phase 1 (Feature Engineering): 3-4 hours (data loading, computation)
- Phase 2 (Similarity Metric): 2-3 hours (distance function testing)
- Phase 3 (Clustering): 5-6 hours (algorithm selection, equal-population constraint)
- Phase 4 (Dimension-Specific): 3-4 hours (per-dimension clustering)
- Phase 5 (Visualization): 3-4 hours (fragmented maps are complex)
- Phase 6 (Analysis): 2-3 hours (homogeneity metrics)
- Phase 7 (Documentation): 1 hour

**Risk**: Very High
- **Computational complexity**: Pairwise similarity for 74K tracts = 5.5 billion comparisons
- **Population constraint**: Hard to enforce in clustering (may need custom algorithm)
- **Convergence**: Clustering with constraints may not converge
- **Visualization**: Fragmented districts difficult to display clearly
- **Conceptual novelty**: No prior art, uncertain if feasible

**Dependencies**:
- Complete demographic data for all census tracts (2020)
- Efficient clustering implementation (scikit-learn or custom)

## Implementation Notes

### Expected District Characteristics

**Age-Based Districts**:
- District 1-50: Tracts with median age 20-30 (young urban areas, college towns)
- District 51-150: Tracts with median age 30-45 (suburban families)
- District 151-350: Tracts with median age 45-65 (established neighborhoods)
- District 351-435: Tracts with median age 65+ (retirement communities)

**Geographic pattern**: "Weather bands" across the country
- Young districts: Seattle, San Francisco, Austin, Boston, NYC (urban cores + college towns)
- Middle-age districts: Suburban rings around major cities
- Older districts: Rural areas, Florida, Arizona retirement communities

**Race-Based Districts**:
- District 1-250: Majority White NH tracts (rural + suburban areas nationwide)
- District 251-300: Majority Hispanic tracts (Southwest, urban areas)
- District 301-350: Majority Black tracts (South, urban areas)
- District 351-380: Majority Asian tracts (West Coast, Northeast urban)
- District 381-435: Diverse/plurality tracts (mixed urban areas)

**Income-Based Districts**:
- District 1-87: Low income (median HH income <$40K)
- District 88-261: Middle income ($40K-$80K)
- District 262-348: Upper-middle income ($80K-$150K)
- District 349-435: High income (>$150K)

### Homogeneity Example

**Geographic district** (current system):
- California District 1 (Northern CA):
  - Median age: 42 (range 25-75)
  - White: 65%, Hispanic: 20%, Asian: 10%, Black: 3%, Other: 2%
  - Median income: $65K (range $30K-$200K)
  - **Homogeneity score**: 0.45 (moderate diversity)

**Age-based demographic district**:
- District 100 (Median age 40-45 cohort):
  - Tracts from CA, TX, FL, NY, IL, OH, NC, etc.
  - Median age: 42.5 (range 40-45) ← very narrow
  - White: 60%, Hispanic: 18%, Asian: 12%, Black: 8%, Other: 2% (similar to national)
  - Median income: $70K (range $35K-$180K) (similar to national)
  - **Homogeneity score (age)**: 0.95 (very homogeneous on age dimension)
  - **Geographic contiguity**: 0 (hundreds of disconnected fragments nationwide)

### Geographic Fragmentation

**Expected pattern**:
- Each district will have 100-1000 disconnected components
- "District" is purely demographic label, not geographic entity
- Visualization will show speckled map with similar demographics scattered nationwide

**Example visualization**:
```
District 1 (Young Urban):
- San Francisco downtown
- Manhattan downtown
- Boston Back Bay
- Chicago Loop
- Seattle Capitol Hill
[...scattered fragments in 200+ locations]
```

### Computational Challenges

**Pairwise similarity**:
- 74,000 tracts × 74,000 tracts = 5.5 billion pairwise comparisons
- If 1 comparison = 1 microsecond → 5,500 seconds = 92 minutes
- **Solution**: Dimensionality reduction (PCA), approximate nearest neighbors

**Population-constrained clustering**:
- Standard k-means doesn't enforce equal population
- **Solution**: Custom algorithm with population awareness:
  1. Initialize clusters randomly
  2. Assign tracts to nearest cluster
  3. If cluster too large, split; if too small, merge
  4. Iterate until population balanced

### Constitutional Issues

**Massive violations**:
- **Geographic representation**: Article I, Section 2 requires representatives from states
- **Contiguity**: Districts not geographically contiguous (100+ fragments each)
- **State boundaries**: Ignores state boundaries entirely
- **Equal Protection**: May violate 14th Amendment (racial classifications)
- **Practical governance**: Representatives cannot serve non-contiguous fragments

**This is the most constitutionally problematic enhancement** - purely theoretical thought experiment.

### Governance Implications

**Advantages**:
- **Descriptive representation**: Representatives demographically match constituents
- **Homogeneity**: Constituents have shared interests/concerns
- **Identity politics**: Explicitly acknowledges demographic-based representation

**Disadvantages**:
- **No constituent services**: How does representative serve 1000 disconnected fragments?
- **Geographic issues**: Local concerns (schools, roads, zoning) have no representative
- **Fragmentation**: Impossible to hold town halls, visit constituents
- **Balkanization**: Entrenches demographic divisions

### Research Questions

1. **Does homogeneity improve representation quality?**
   - Hypothesis: Homogeneous districts make representatives more responsive
   - Test: Survey hypothetical constituent satisfaction

2. **What is the cost of ignoring geography?**
   - Measure: Compactness scores (expect 0.01-0.05 Polsby-Popper)
   - Compare: Geographic baseline (0.30-0.40)

3. **Which demographic dimension matters most?**
   - Compare: Age-based vs race-based vs income-based districts
   - Test: Which produces most homogeneous clusters?

4. **How many disconnected components per district?**
   - Hypothesis: 100-1000 fragments per district
   - Measure: Connected components in geographic graph

### International Comparisons

**No international parallel exists**. Closest concepts:
- **Descriptive representation theory**: Representatives should "look like" constituents
- **Proportional representation by party**: Similar non-geographic representation
- **Ethnic/religious quotas**: Some countries reserve seats for groups (Lebanon, India)

This model takes descriptive representation to logical extreme.

## Related Enhancements

- E22: National Redistricting (ignores state boundaries, still geographic)
- E24: Party-Based District Allocation (non-geographic, party-based)

## Research Value

This model explores:
- **Descriptive representation**: Extreme version of "like with like"
- **Geographic vs demographic**: Trade-offs between physical proximity and demographic similarity
- **Homogeneity effects**: What happens when districts perfectly homogeneous?
- **Fragmentation cost**: Quantify value of geographic contiguity
- **Identity politics**: Explicit demographic-based representation
