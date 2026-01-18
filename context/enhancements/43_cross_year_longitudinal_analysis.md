# Enhancement 43: Cross-Year Longitudinal Analysis

**Status**: Proposed
**Priority**: Medium
**Created**: January 17, 2026

## Problem Statement

The pipeline processes 2000, 2010, and 2020 census data independently, but provides no longitudinal analysis of how districts, populations, and compactness evolve over time.

**Current State:**
- Three independent runs for 2000/2010/2020
- Each produces standalone results (maps, CSVs, metrics)
- Separate dashboards for each census year
- No cross-year comparison or trend analysis

**Missing:**
- Population shift analysis (which states gained/lost districts?)
- Compactness trends (are districts getting better or worse?)
- Geographic stability (how much do district boundaries change?)
- Enacted districts evolution (how has gerrymandering evolved?)

## Research Questions

1. **How has U.S. population distribution changed?**
   - Which states gained/lost congressional seats? (Huntington-Hill apportionment)
   - Urban vs. rural population shifts
   - Regional migration patterns (Sunbelt growth, Rust Belt decline)

2. **Have districts become more or less compact over time?**
   - Algorithmic compactness trends (2000 → 2010 → 2020)
   - Enacted districts compactness trends
   - Does redistricting technology lead to worse gerrymandering?

3. **How stable are algorithmic district boundaries?**
   - If we run algorithm on consecutive census years, how much do boundaries change?
   - Useful for: understanding disruption to communities, voter confusion

4. **Which states show largest changes?**
   - Fastest growing states (Texas, Florida, Arizona)
   - Declining states (Illinois, New York, West Virginia)
   - Partisan control changes during redistricting cycles

## Use Cases

### Academic Research
- Study gerrymandering evolution over 20 years
- Population dynamics and political representation
- Impact of redistricting reforms (commissions adopted post-2010)

### Policy Analysis
- Justify redistricting reform by showing trends
- Demonstrate stability (or instability) of algorithmic approach
- Compare states with/without reform

### Visualization
- Animated maps showing district evolution
- "Then vs. Now" comparisons
- State-by-state trend charts

## Proposed Features

### 1. Apportionment Change Analysis

Track seat changes across census years:

```python
# Example output
State        2000  2010  2020  Change 2000-2020
Texas          32    36    38    +6
California     53    53    52    -1
New York       29    27    26    -3
```

**Analysis:**
- Which regions gained/lost representation?
- Correlation with population growth rates
- Political implications (red/blue state seat shifts)

### 2. Compactness Trends

Track Polsby-Popper scores over time:

**Algorithmic Districts:**
- Mean compactness: 2000, 2010, 2020
- Variance reduction over time (better optimization?)

**Enacted Districts:**
- Gerrymandering severity trends
- Impact of redistricting commissions (CA, AZ, CO adopted post-2010)

### 3. Geographic Stability Analysis

**Boundary Change Metrics:**
- Percent of population reassigned to different districts
- Percent of geographic area with boundary changes
- "District persistence score" (how many districts stay mostly the same?)

**Use Case:**
- Argue for/against algorithmic redistricting based on stability
- High stability → less voter confusion, community preservation
- Low stability → responsive to population shifts

### 4. Comparative Dashboard

**Three-Year Master Dashboard:**
- Side-by-side maps (2000 | 2010 | 2020)
- Trend charts (compactness, demographics, political lean)
- State selector with drill-down to year-by-year comparison
- Animated transitions showing boundary evolution

### 5. State Case Studies

**Deep-dive Analysis for Key States:**
- **Texas** (+6 seats): How did growth affect districts?
- **California** (-1 seat): Commission impact (adopted 2010)?
- **North Carolina** (notorious gerrymandering): Trend over time?
- **Colorado** (commission adopted 2018): Before/after comparison?

## Implementation Plan

### Phase 1: Data Integration
- [ ] Create unified data schema spanning all years
- [ ] Generate cross-year comparison tables
- [ ] Compute apportionment changes (Huntington-Hill)
- [ ] Track which states gained/lost seats

### Phase 2: Compactness Trends
- [ ] Extract compactness metrics for all years
- [ ] Compute mean/median/variance trends
- [ ] Statistical tests (is 2020 more/less compact than 2000?)
- [ ] Separate analysis for algorithmic vs. enacted

### Phase 3: Geographic Stability
- [ ] Overlay 2000/2010/2020 district boundaries
- [ ] Compute boundary change metrics (IoU, Hausdorff distance)
- [ ] Identify "stable" vs. "volatile" districts
- [ ] Visualize boundary evolution

### Phase 4: Dashboard Integration
- [ ] Create master longitudinal dashboard
- [ ] Add year selector/slider
- [ ] Implement side-by-side comparison view
- [ ] Generate animated transitions (2000 → 2010 → 2020)

### Phase 5: Paper/Report
- [ ] Write "20 Years of Congressional Redistricting" report
- [ ] Key findings summary
- [ ] Policy recommendations based on trends

## Files to Create/Modify

### New Files
- `scripts/longitudinal/compute_apportionment_changes.py`
- `scripts/longitudinal/analyze_compactness_trends.py`
- `scripts/longitudinal/compute_boundary_stability.py`
- `scripts/longitudinal/generate_comparison_tables.py`
- `scripts/web/generate_longitudinal_dashboard.py`
- `web/longitudinal_dashboard.html`
- `artifacts/papers/04_longitudinal_analysis/` (new paper)

### Modified Files
- `scripts/web/generate_master_dashboard.py` - Add cross-year comparison tab
- `web/master_dashboard.html` - Integrate longitudinal view

## Data Requirements

**Already Available:**
- District assignments for 2000/2010/2020 (algorithmic)
- Compactness metrics for all years
- Enacted district shapefiles (from Enhancement 11)
- Population data from census

**Need to Compute:**
- Cross-year boundary overlaps
- Stability metrics
- Trend statistics

## Success Criteria

- [ ] Dashboard shows 2000/2010/2020 side-by-side for any state
- [ ] Apportionment change table generated (which states gained/lost seats)
- [ ] Compactness trend analysis complete (algorithmic and enacted)
- [ ] Boundary stability metrics computed
- [ ] At least 3 state case studies documented
- [ ] Paper 4 draft or technical report written

## Example Research Findings

**Hypothetical results (to be validated):**
- "Algorithmic districts maintain consistent compactness (0.45-0.47 PP) across all three census years"
- "Enacted districts became 12% less compact from 2000 to 2020 (0.32 → 0.28 PP)"
- "States with redistricting commissions show 8% improvement in compactness post-reform"
- "Texas population growth required +6 seats but algorithmic district boundaries changed by only 23% (high stability)"

## Related Enhancements

- [Enhancement 11: Baseline Comparison](11_baseline_comparison.md) - Provides enacted district data for comparison
- [Enhancement 42: Research Narrative](42_research_narrative_policy_questions.md) - Answers "how has redistricting evolved?"
- [Enhancement 15: Multi-Year Pipeline](15_multi_year_support.md) - Enables processing all years

## Notes

**Why This Matters:**
- Redistricting happens every 10 years - longitudinal view is natural
- Shows algorithmic consistency over time (or lack thereof)
- Provides historical context for current debates
- Enables predictions for 2030 census

**Visualization Ideas:**
- Animated GIF: Texas districts 2000 → 2010 → 2020
- Scatter plot: State population change vs. compactness change
- Timeline: Redistricting reform adoption and impact
