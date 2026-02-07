---
uuid: 45a1b5
slug: baseline-data-organization
name: 45: Baseline Data Organization and Analysis
wave_uuid: f3a1b3
created: '2026-01-25'
status: PLANNED
---
# E45: Baseline Data Organization and Analysis

**Status**: Proposed
**Priority**: Critical
**Created**: January 17, 2026
**Commits**: [ad542e9](https://github.com/giodl_microsoft/redistricting/commit/ad542e9efb96bd2ee745b6ab6e77f2ee721b1789)
**Size**: M - 1,191 lines changed (6 files)

## Problem Statement

Baseline comparison data has been gathered for Paper 03 (combined recursive bisection), but it's not systematically organized, analyzed, or integrated into the project structure.

**User Feedback (January 17, 2026):**
> "we have gathered baseline data for papers\03_ but i take your point it isnt organized enough"

**Current State:**
- E11 (Baseline Comparison) marked as completed
- Baseline data exists somewhere in `artifacts/papers/03_combined_recursive_bisection/`
- Data not integrated into pipeline or dashboard
- No systematic analysis scripts
- Results not readily accessible

**Missing:**
- Structured data directory for baseline districts
- Automated analysis comparing algorithmic vs. enacted
- Integration into dashboard (show both side-by-side)
- Statistical tests and summary tables
- Clear documentation of findings

## Goals

1. **Organize baseline data systematically**
2. **Create automated comparison pipeline**
3. **Generate publication-ready tables/figures**
4. **Integrate into dashboard**
5. **Document findings clearly**

## Proposed Directory Structure

```
data/baseline/
├── enacted_districts/
│   ├── 2000/
│   │   ├── shapefiles/
│   │   │   ├── alabama.shp
│   │   │   ├── california.shp
│   │   │   └── ...
│   │   └── metadata.json
│   ├── 2010/
│   └── 2020/
│       ├── shapefiles/
│       └── metadata.json
├── compactness/
│   ├── enacted_compactness_2000.csv
│   ├── enacted_compactness_2010.csv
│   ├── enacted_compactness_2020.csv
│   └── comparison_summary.csv
└── README.md

outputs/{version}/{year}/baseline_comparison/
├── state_by_state_comparison.csv
├── summary_statistics.csv
├── visualization/
│   ├── compactness_comparison.png
│   ├── state_bar_chart.png
│   └── side_by_side_maps/
│       ├── california_comparison.png
│       ├── texas_comparison.png
│       └── ...
└── statistical_tests.json
```

## Implementation Plan

### Phase 1: Data Audit and Organization

**Tasks:**
- [ ] Locate all baseline data currently in `artifacts/papers/03_*/`
- [ ] Move to structured `data/baseline/` directory
- [ ] Create inventory: Which states/years do we have?
- [ ] Document data sources and download dates
- [ ] Create `data/baseline/README.md` with provenance

**Deliverable:** Clean, documented baseline data directory

### Phase 2: Automated Analysis Pipeline

**Create Analysis Scripts:**

1. **`scripts/baseline/compute_enacted_compactness.py`**
   - Load enacted district shapefiles
   - Compute Polsby-Popper and Reock for each district
   - Save to `data/baseline/compactness/enacted_compactness_{year}.csv`

2. **`scripts/baseline/compare_algorithmic_vs_enacted.py`**
   - Load algorithmic districts from pipeline output
   - Load enacted districts from `data/baseline/`
   - Compute metrics for both
   - Generate comparison tables (state-by-state, district-by-district)
   - Output: `outputs/{version}/{year}/baseline_comparison/state_by_state_comparison.csv`

3. **`scripts/baseline/statistical_analysis.py`**
   - Paired t-tests (algorithmic vs. enacted compactness)
   - Effect sizes (Cohen's d)
   - Confidence intervals
   - Output: `statistical_tests.json`

4. **`scripts/baseline/generate_comparison_visualizations.py`**
   - Bar charts: State-by-state compactness comparison
   - Scatter plots: Algorithmic vs. enacted (with diagonal reference line)
   - Box plots: Distribution comparison
   - Maps: Side-by-side algorithmic vs. enacted for key states

**Deliverable:** Automated, reproducible analysis pipeline

### Phase 3: Integration into Main Pipeline

**Modify Existing Scripts:**
- [ ] Add `--baseline-comparison` flag to `run_complete_redistricting.py`
- [ ] Automatically run comparison analysis if baseline data exists
- [ ] Include baseline comparison in process_nation.py summary

**Example Usage:**
```bash
python scripts/pipeline/run_complete_redistricting.py \
    --year 2020 --version v1 --baseline-comparison
```

**Output:**
- All standard outputs (maps, CSVs, dashboard)
- Plus: `baseline_comparison/` directory with comparison analysis

### Phase 4: Dashboard Integration

**Add Baseline Comparison Tab:**
- Side-by-side map viewer (algorithmic | enacted)
- State selector
- Metrics table showing both approaches
- Highlight improvements/regressions

**Example Layout:**
```
[State: California ▼]

Algorithmic Districts          Enacted Districts
[Map with 52 districts]        [Map with 52 enacted]

Metrics Comparison:
                    Algorithmic    Enacted    Improvement
Mean Compactness:      0.47          0.32       +47%
Min Compactness:       0.35          0.18       +94%
Max Compactness:       0.58          0.48       +21%
```

**Files to Modify:**
- `web/dashboard.html` - Add "Baseline Comparison" tab
- `scripts/web/generate_dashboard.py` - Bake in baseline data

### Phase 5: Publication-Ready Tables and Figures

**Generate for Paper:**

**Table 1: State-by-State Compactness Comparison**
```
State          Algorithmic  Enacted   Δ      p-value
Alabama           0.45       0.31    +45%    <0.001
California        0.47       0.32    +47%    <0.001
Texas             0.42       0.28    +50%    <0.001
...
National Mean     0.45       0.30    +50%    <0.001
```

**Table 2: Summary Statistics**
```
Metric                  Algorithmic    Enacted    Improvement
Mean Compactness           0.45          0.30       +50%
Median Compactness         0.46          0.31       +48%
Std Deviation              0.08          0.09       -11%
Min Compactness            0.28          0.12      +133%
Max Compactness            0.62          0.51       +22%
```

**Figure 1: Scatter Plot (all 435 districts)**
- X-axis: Enacted district compactness
- Y-axis: Algorithmic district compactness
- Diagonal reference line (y=x)
- Most points above line → algorithmic better

**Figure 2: State Bar Chart**
- States on X-axis (sorted by improvement)
- Bars showing compactness (algorithmic vs. enacted)
- Clearly shows which states benefit most

**Figure 3: Box Plot Comparison**
- Side-by-side box plots (algorithmic vs. enacted)
- Shows distribution differences

### Phase 6: Documentation

**Create Documentation:**
- [ ] `data/baseline/README.md` - Data sources and structure
- [ ] `docs/BASELINE_COMPARISON.md` - Analysis methodology
- [ ] Update `artifacts/papers/03_*/README.md` with findings
- [ ] Add to `ARCHITECTURE.md` - Baseline comparison workflow

**Key Findings Document:**
Write summary answering:
- How much more compact are algorithmic districts?
- Which states show largest improvements?
- Are improvements consistent across all states?
- Statistical significance of findings?

## Expected Findings

**Hypothetical Results (to be validated):**
- "Algorithmic districts are 50% more compact on average (PP: 0.45 vs. 0.30)"
- "All 50 states show improvement (range: +25% to +90%)"
- "Largest improvements in heavily gerrymandered states (MD, NC, PA)"
- "Smallest improvements in compact/commission states (IA, CO, CA)"
- "Effect is highly significant (p < 0.001, Cohen's d = 2.1)"

## Success Criteria

- [ ] All baseline data in `data/baseline/` with clear organization
- [ ] Automated analysis scripts run without errors
- [ ] State-by-state comparison table generated
- [ ] Statistical tests complete (t-test, effect sizes)
- [ ] Publication-ready figures generated (3+ figures)
- [ ] Dashboard includes baseline comparison tab
- [ ] Documentation complete (README, methodology)
- [ ] Findings summarized in 1-page document

## Files to Create

### Data
- `data/baseline/README.md`
- `data/baseline/compactness/` (CSVs with computed metrics)

### Scripts
- `scripts/baseline/compute_enacted_compactness.py`
- `scripts/baseline/compare_algorithmic_vs_enacted.py`
- `scripts/baseline/statistical_analysis.py`
- `scripts/baseline/generate_comparison_visualizations.py`
- `scripts/baseline/README.md`

### Documentation
- `docs/BASELINE_COMPARISON.md`
- `outputs/{version}/{year}/baseline_comparison/FINDINGS.md`

### Visualizations
- `outputs/{version}/{year}/baseline_comparison/visualization/` (PNG files)

## Related Enhancements

- [E11: Baseline Comparison](11_baseline_comparison.md) - Data collection (completed)
- [E42: Research Narrative](42_research_narrative_policy_questions.md) - Uses baseline data to answer research questions
- [E43: Longitudinal Analysis](43_cross_year_longitudinal_analysis.md) - Compare baseline trends over time

## Notes

**Priority Justification:**
- User confirmed baseline data exists but needs organization
- Critical for publication (reviewers will ask for this)
- Relatively straightforward (data exists, just needs systematization)
- High impact (answers key research question)

**Quick Win:**
This could be completed quickly since data already gathered. Focus on:
1. Organization (move files to standard locations)
2. Automation (scripts to generate tables/figures)
3. Documentation (write up findings)

**Estimated Effort:** Medium (2-4 hours)
- Mostly organization and scripting
- Data collection already done (E11)
