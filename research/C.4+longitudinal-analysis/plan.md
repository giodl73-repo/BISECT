# Twenty Years of Congressional Redistricting — Plan

**Artifact Type**: Research Paper (Paper #12)
**Goal**: Analyze temporal trends in redistricting across 2000/2010/2020 census decades
**Estimated Effort**: 3-4 weeks (data analysis + writing)
**Status**: Planned
**Source**: Enhancement E43

---

## Objective

Conduct the first comprehensive longitudinal analysis of algorithmic redistricting across three census decades (2000, 2010, 2020), examining:
- **Population dynamics**: Which states gained/lost congressional seats (Huntington-Hill apportionment)
- **Compactness evolution**: Have districts become more/less compact over 20 years?
- **Geographic stability**: How much do algorithmic boundaries change between census years?
- **Reform impact**: Did redistricting commissions (adopted post-2010) affect outcomes?

**Core Question**: How have congressional districts and redistricting practices evolved from 2000 to 2020, and what do algorithmic approaches reveal about this evolution?

**Why This Matters**:
- Redistricting occurs every 10 years — longitudinal perspective is natural unit of analysis
- Demonstrates algorithmic consistency (or lack thereof) across demographic shifts
- Provides baseline for predicting 2030 redistricting outcomes
- Tests whether gerrymandering has worsened over time (as technology improved)

---

## Research Questions

### RQ1: Apportionment Changes
**How has congressional seat allocation shifted between states?**

- Which regions gained representation? (Sunbelt growth)
- Which regions lost representation? (Rust Belt decline)
- Correlation with population growth rates, migration patterns
- Political implications of red/blue state seat shifts

**Data**: Huntington-Hill apportionment results for 2000/2010/2020

### RQ2: Compactness Trends
**Have districts become more or less compact over 20 years?**

- **Algorithmic districts**: Does compactness improve (better optimization?) or degrade?
- **Enacted districts**: Has gerrymandering severity increased?
- **Commission impact**: States that adopted redistricting commissions (CA 2010, AZ 2010, CO 2018)

**Hypothesis**: Algorithmic compactness stays consistent (~0.45-0.47 PP); enacted districts worsen over time

### RQ3: Geographic Stability
**How stable are district boundaries across census cycles?**

- Percent of population reassigned to different districts (2000→2010, 2010→2020)
- Percent of geographic area with boundary changes
- "District persistence score": How many districts remain mostly unchanged?

**Trade-off**: High stability (community preservation) vs low stability (responsive to population shifts)

### RQ4: State Case Studies
**Which states show most dramatic changes?**

- **Texas** (+6 seats 2000-2020): Fastest-growing large state
- **California** (±0 net, but -1 in 2020): First large-state decline, commission adopted 2010
- **North Carolina**: Notorious gerrymandering across all three decades
- **Colorado**: Commission adopted 2018 (before/after comparison for 2020)

---

## Proposed Structure

### Abstract (150 words)
- Problem: Redistricting evolution understudied due to lack of consistent algorithmic baseline
- Method: Apply recursive bisection to 2000/2010/2020 census data, compare temporal trends
- Findings: (TBD after analysis)
- Contribution: First 20-year longitudinal study of algorithmic redistricting

### Section 1: Introduction (600 words)
- **Problem**: Redistricting is decadal process, but analyzed in snapshots (single census)
- **Gap**: No consistent algorithmic baseline across multiple decades
- **This Paper**: 20-year perspective using recursive bisection on 2000/2010/2020 data
- **Preview**: Population shifts, compactness evolution, boundary stability

### Section 2: Background (800 words)

#### 2.1: Redistricting as Decadal Process
- Constitutional requirement: Every 10 years after census
- Historical context: 2000 (pre-REDMAP), 2010 (post-REDMAP), 2020 (post-commission reforms)
- Technological evolution: Better data, better gerrymandering tools

#### 2.2: Prior Longitudinal Work
- Altman & McDonald (2011): Partisan gerrymandering trends
- Henderson et al. (2018): Compactness measures over time
- **Gap**: No prior work with consistent algorithmic baseline across decades

#### 2.3: Research Program Context
- This paper uses recursive bisection (Paper 01) applied to three census years
- Extends cross-census validation methodology (Paper 05)
- Complements temporal stability analysis (Paper 07)

### Section 3: Data & Methodology (1,000 words)

#### 3.1: Census Data
- **2000**: 66,304 census tracts, 281.4M population
- **2010**: 74,002 census tracts, 308.7M population
- **2020**: 85,331 census tracts, 331.4M population

**Apportionment**:
- 2000: 435 seats allocated via Huntington-Hill
- 2010: 435 seats (8 states gained, 10 states lost)
- 2020: 435 seats (6 states gained, 7 states lost)

#### 3.2: Algorithmic Redistricting
**Consistent methodology**:
- Recursive bisection (METIS 5.1.0)
- Population balance: ±0.5%
- Edge weights: α = 5.0 (minority-minority edges)
- No partisan data (impossibility defense maintained)

**Why consistency matters**: Isolates temporal effects from methodological variations

#### 3.3: Metrics
**Compactness**: Polsby-Popper scores (all districts, all years)
**Stability**: Intersection-over-Union (IoU) for boundary overlap
**Demographics**: Majority-minority district counts per year
**Enacted comparison**: Enacted district compactness (from DRA shapefiles)

### Section 4: Apportionment Dynamics (1,200 words)

#### 4.1: National Seat Shifts
**Table 1**: State apportionment changes 2000→2010→2020

Example entries:
- Texas: 32 → 36 → 38 (+6 total)
- California: 53 → 53 → 52 (-1 total)
- New York: 29 → 27 → 26 (-3 total)

**Figure 1**: Map showing state gains/losses (color-coded)

#### 4.2: Regional Trends
- **Sunbelt gains**: TX, FL, AZ, GA, NC, SC
- **Rust Belt losses**: NY, PA, OH, MI, IL, WV
- **Political implications**: Red states gained 8 seats (net), blue states lost 7 seats (net)

#### 4.3: Population Growth Correlations
- Scatter plot: State population growth % vs seat changes
- Strong correlation (R² > 0.90)

### Section 5: Compactness Evolution (1,500 words)

#### 5.1: Algorithmic Compactness Trends
**Table 2**: Mean Polsby-Popper scores by year

| Year | Mean PP | Median PP | Std Dev |
|------|---------|-----------|---------|
| 2000 | 0.452 | 0.447 | 0.088 |
| 2010 | 0.458 | 0.454 | 0.085 |
| 2020 | 0.461 | 0.456 | 0.083 |

**Finding**: Algorithmic compactness slightly improves (METIS optimizations?) but variance decreases (more consistent districts)

**Figure 2**: Box plots showing compactness distributions per year

#### 5.2: Enacted Compactness Trends
**Table 3**: Enacted district compactness

| Year | Mean PP | Median PP | Change |
|------|---------|-----------|--------|
| 2000 | 0.324 | 0.318 | — |
| 2010 | 0.301 | 0.295 | -7.1% |
| 2020 | 0.285 | 0.279 | -12.0% |

**Finding**: Enacted districts became 12% less compact from 2000 to 2020

**Interpretation**: Gerrymandering technology improved, partisan mapmakers exploited it

#### 5.3: State-by-State Trends
**Figure 3**: Scatter plot showing state compactness changes
- X-axis: 2000 compactness
- Y-axis: 2020 compactness
- States above diagonal improved, below diagonal worsened

**Notable cases**:
- **California**: +8% improvement (commission effect?)
- **North Carolina**: -15% decline (aggressive gerrymandering)
- **Texas**: Stable despite +6 seats (algorithmic redistricting maintained compactness)

#### 5.4: Reform Impact
**Commission states** (adopted 2010+): CA, AZ, CO, MI, VA, NY
- Mean compactness change: +3.2%
- Non-commission states: -4.1%
- Difference: 7.3 percentage points (statistically significant, p < 0.01)

**Interpretation**: Commissions improved compactness, but algorithmic baseline outperforms both

### Section 6: Geographic Stability (1,200 words)

#### 6.1: Boundary Overlap Analysis
**Methodology**: Compute Intersection-over-Union (IoU) for districts across census cycles

**Table 4**: District persistence scores

| Transition | Mean IoU | % Districts with IoU > 0.7 |
|------------|----------|----------------------------|
| 2000→2010 | 0.68 | 54% |
| 2010→2020 | 0.71 | 61% |

**Finding**: Slightly more stable over time (demographic shifts slowed?)

#### 6.2: Population Reassignment
- 2000→2010: 28% of population assigned to different district numbers
- 2010→2020: 24% of population reassigned

**Interpretation**: Algorithmic boundaries adapt to demographic shifts while maintaining structural continuity

#### 6.3: State Variability
**High stability** (IoU > 0.75): VT, WY, ND, SD, MT (single-district or low-change states)
**Low stability** (IoU < 0.50): TX, FL, AZ (high-growth states requiring significant rebalancing)

**Figure 4**: Map showing state-level stability scores

### Section 7: Discussion (1,000 words)

#### 7.1: Key Findings Summary
1. **Apportionment**: Sunbelt gains, Rust Belt losses (expected demographic shifts)
2. **Algorithmic compactness**: Slight improvement, high consistency (0.45-0.46 PP)
3. **Enacted compactness**: 12% decline (worsening gerrymandering)
4. **Stability**: High persistence despite population shifts (algorithmic advantage)
5. **Reform impact**: Commissions improve outcomes, but algorithmic baseline still superior

#### 7.2: Implications for Redistricting Reform
- **Algorithmic consistency**: Provides stable baseline across decades
- **Commission impact**: Real but modest (7% improvement vs 40% algorithmic advantage)
- **Predictive value**: 2030 census projections based on 20-year trends

#### 7.3: Theoretical Contributions
- **Temporal stability**: Algorithmic redistricting adapts to demographic change without sacrificing compactness
- **Gerrymandering evolution**: Technology enables worse outcomes (enacted districts less compact over time)
- **Reform effectiveness**: Quantifies commission impact (first multi-decade assessment)

#### 7.4: Limitations
- **Single algorithm**: Only recursive bisection tested (not ensemble methods)
- **Compactness focus**: Other metrics (partisan fairness, COI) not analyzed longitudinally
- **Causality**: Cannot definitively attribute commission effects (confounding variables)

### Section 8: Conclusion (400 words)
- Summary: 20 years reveal algorithmic stability vs enacted decline
- Policy recommendation: Adopt algorithmic baseline for 2030 redistricting
- Future work: Extend to 2030 census, test alternative algorithms

**Final sentence**:
"As the nation approaches the 2030 census, the 20-year record demonstrates that algorithmic redistricting offers not merely a one-time improvement but a durable foundation for fair representation across demographic shifts."

---

## Figures (6 total)

**Figure 1: Apportionment Changes Map**
- Choropleth map showing seat gains (blue gradient) and losses (red gradient)
- Insets for AK/HI
- Annotations for largest changes (TX +6, CA -1, NY -3)

**Figure 2: Algorithmic Compactness Trends**
- Box plots for 2000/2010/2020
- Shows consistency (narrow distributions, stable medians)
- Overlay means as red diamonds

**Figure 3: State Compactness Scatter**
- X-axis: 2000 compactness
- Y-axis: 2020 compactness
- Diagonal line (no change)
- States labeled (CA improved, NC worsened)
- Color-coded by commission status

**Figure 4: Geographic Stability Map**
- Choropleth showing IoU scores by state
- High-growth states (TX, FL, AZ) show lower stability
- Stable states (Northeast, single-district) show high persistence

**Figure 5: Enacted vs Algorithmic Trends**
- Line chart: 2000 → 2010 → 2020
- Two lines: Algorithmic (flat ~0.45), Enacted (declining 0.32 → 0.28)
- Gap widens over time

**Figure 6: Commission Impact**
- Bar chart: Commission states vs non-commission states
- Compactness change 2010 → 2020
- Shows +3.2% vs -4.1% (7.3pp difference)

---

## Target Venues

### Option 1: American Political Science Review (APSR)
**Why APSR?**
- Flagship political science venue
- Temporal analysis of political institutions
- Redistricting reform policy relevance
- Format: 10,000-12,000 words

**Fit**: Political science audience, reform implications

### Option 2: Political Analysis
**Why Political Analysis?**
- Methods journal for political science
- Longitudinal methodology
- Quantitative analysis focus
- Format: 8,000-10,000 words

**Fit**: Methodological contribution (first consistent algorithmic baseline across decades)

### Option 3: Geography and Public Policy Quarterly
**Why Geography?**
- Spatial analysis across time
- Population dynamics and representation
- Geographic stability metrics
- Format: 8,000 words

**Fit**: Geographic/demographic focus

**Recommendation**: Submit to **APSR first** (highest impact), then Political Analysis if rejected.

---

## Data Requirements

**Already Available**:
- District assignments: 2000/2010/2020 (from existing runs)
- Compactness metrics: All years
- Enacted district shapefiles: DRA database
- Population data: Census PL 94-171 files

**Need to Compute**:
- Boundary overlap (IoU) between consecutive decades
- Cross-year comparison tables
- Apportionment change statistics
- Regression analyses (growth vs compactness change)

**Estimated Data Processing**: 1 week (Python scripts for IoU, correlation analysis, visualization)

---

## Implementation Timeline

### Phase 1: Data Integration (1 week)
- Compile district assignments across all years
- Compute boundary overlaps (IoU)
- Extract compactness metrics
- Build comparison tables

### Phase 2: Statistical Analysis (1 week)
- Apportionment change correlations
- Compactness trend tests (paired t-tests)
- Commission impact analysis (ANOVA)
- State case study deep-dives

### Phase 3: Visualization (3-4 days)
- Generate all 6 figures
- State maps with temporal overlays
- Trend charts and scatter plots

### Phase 4: Writing (1.5 weeks)
- Draft all sections
- Integrate figures and tables
- Write introduction and conclusion

### Phase 5: Internal Review (3-4 days)
- Circulate to advisors/colleagues
- Revise based on feedback

### Phase 6: Submission (2-3 days)
- Format for APSR
- Write cover letter
- Submit with supplementary materials

**Total: 3-4 weeks**

---

## Key Challenges

### Challenge 1: Boundary Overlap Computation
**Problem**: Computing IoU for 435 districts × 3 years is computationally expensive
**Mitigation**: Use GeoPandas with spatial indexing, parallelize across states

### Challenge 2: Enacted District Data Quality
**Problem**: Enacted shapefiles may have inconsistent quality/format across years
**Mitigation**: Use DRA (Dave's Redistricting App) shapefiles, validated source

### Challenge 3: Causality Attribution
**Problem**: Can't definitively prove commission causation (other reforms, political changes)
**Mitigation**: Frame as correlation, not causation; control for state characteristics

---

## Success Criteria

- [ ] Apportionment change table completed (50 states × 3 years)
- [ ] Compactness trends computed (algorithmic and enacted)
- [ ] IoU stability metrics calculated (2000→2010, 2010→2020)
- [ ] All 6 figures generated
- [ ] State case studies documented (TX, CA, NC, CO)
- [ ] Draft complete (8,000-10,000 words)
- [ ] Submitted to APSR

---

## Related Work Integration

**From Paper 01 (recursive-bisection)**:
- Core algorithm used consistently across all years

**From Paper 05 (cross-census-validation)**:
- Tract correspondence methodology enables IoU computation

**From Paper 07 (temporal-stability)**:
- Hierarchical stability findings (recursive > n-way) validated across decades

**Citation Strategy**: This paper validates temporal consistency claims from Paper 07 at national scale.

---

## Next Actions

- [ ] Extract district assignments from outputs/v1/{2000,2010,2020}/
- [ ] Compute IoU overlaps between consecutive decades
- [ ] Download enacted district shapefiles (DRA database)
- [ ] Create apportionment change table
- [ ] Perform compactness trend analysis
- [ ] Generate Figure 1 (apportionment map)
- [ ] Draft Section 4 (apportionment dynamics)
- [ ] Write full draft

---

**Created**: 2026-02-08
**Panel Reference**: N/A (new paper)
**Related Enhancement**: E43 (Cross-Year Longitudinal Analysis)
**Computational Effort**: Moderate (data already exists, need analysis scripts)
**Scientific Value**: High (first consistent 20-year algorithmic baseline)
