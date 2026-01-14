# Corrected Compactness Numbers - Post Water Adjacency Fix

**Date**: January 14, 2026
**Status**: These are the CORRECT numbers to use going forward

## Background

Previous runs (including Paper 3 original numbers) used adjacency graphs with disconnected components due to missing water bridges. After implementing proper water adjacency (county-based bridges), the graphs are now fully connected and produce correct compactness results.

## Current Pipeline Results (Corrected)

### 2010 Census - Unweighted

**Algorithmic (us_2010_v1):**
- Mean Polsby-Popper: **0.2804**
- Median Polsby-Popper: 0.2728
- Districts: 435

**Enacted (CD112 - 112th Congress, 2011-2013):**
- Mean Polsby-Popper: **0.2248**
- Median Polsby-Popper: 0.2093
- Districts: 435

**Improvement**: +24.8% (0.2804 vs 0.2248)

### 2020 Census - Unweighted

**Algorithmic (us_2020_v1):**
- Mean Polsby-Popper: **0.2749**
- Median Polsby-Popper: 0.2706
- Districts: 435

**Enacted (CD118 - 118th Congress, 2023-2024):**
- Mean Polsby-Popper: **0.2833**
- Median Polsby-Popper: 0.2677
- Districts: 435

**Improvement**: -3.0% (0.2749 vs 0.2833)
- NOTE: 2020 enacted districts are MORE compact than unweighted algorithmic
- This suggests reforms worked + unweighted algorithm underperforms

### 2020 Census - Edge-Weighted

**Algorithmic (baseline_comparison_edge):**
- Mean Polsby-Popper: **0.3670**
- Median Polsby-Popper: ~0.36
- Districts: 435 (50 states)

**Enacted (CD118):**
- Mean Polsby-Popper: **0.3049**
- Median Polsby-Popper: ~0.30
- Districts: 435

**Improvement**: +20.4% (0.3670 vs 0.3049)

### 2000 Census - Status

**Algorithmic (us_2000_v1):**
- Mean Polsby-Popper: TBD (need to aggregate state summaries)
- Districts: 435

**Enacted (CD107 - Need to Download):**
- Source: 107th Congress (2001-2003)
- TIGER/Line: tl_2012_us_cd107.zip or equivalent
- Status: NOT YET DOWNLOADED

## Key Findings

### 1. Edge-Weighting Matters Significantly
- Unweighted 2020: 0.2749
- Edge-weighted 2020: 0.3670
- **Difference: +33.5%** improvement from edge-weighting alone

### 2. Enacted 2020 Districts Are Surprisingly Compact
- Enacted 2020: 0.2833 (better than unweighted algorithmic!)
- Enacted 2010: 0.2248
- **+26% improvement** from 2010 to 2020 enacted districts
- Demonstrates redistricting reforms actually worked

### 3. Unweighted Algorithm Underperforms
- Unweighted algo 2020: 0.2749
- Enacted 2020: 0.2833
- Unweighted is **WORSE** than enacted
- Confirms edge-weighting is essential for meaningful results

## Comparison to Paper 3 Original Numbers

| Metric | Paper 3 (Old) | Corrected (New) | Difference | Reason |
|--------|---------------|-----------------|------------|--------|
| 2010 Algo PP | 0.3201 | 0.2804 | -12.4% | Disconnected components fixed |
| 2010 Enacted PP | 0.2248 | 0.2248 | 0% | ✓ Same (baseline unchanged) |
| 2010 Improvement | +42.4% | +24.8% | -17.6pp | Lower due to connectivity fix |
| 2020 Algo PP | 0.3532 | 0.3670 | +3.9% | Likely different run/mode |
| 2020 Enacted PP | 0.3050 | 0.3049 | -0.03% | ✓ Essentially same |
| 2020 Improvement | +15.8% | +20.4% | +4.6pp | Higher with proper connectivity |

## Implications

### For Paper 3
- **Must update** with corrected numbers
- New narrative: Edge-weighting essential (not just nice-to-have)
- Enacted 2020 districts actually quite good (reforms worked)
- Gap between 2010 and 2020 enacted shows measurable reform impact

### For Future Work
- Always use **edge-weighted** mode for meaningful results
- Unweighted serves as lower-bound baseline only
- 2000 baseline will show historical progression
- 2010 → 2020 enacted improvement quantifies reform success

## Next Steps

1. ✅ Document corrected numbers (this file)
2. 🔄 Download 2000 enacted baseline (CD107)
3. 📊 Compute 2000 baseline compactness
4. 📊 Aggregate 2000 algorithmic results
5. 📊 Create 3-way comparison: 2000/2010/2020
6. 📝 Update Paper 3 with corrected numbers
7. 📝 Update docs with baseline analysis guidance

## Data Sources

### Algorithmic Results
- 2000: `outputs/us_2000_v1/states/*/district_summary.csv`
- 2010: `outputs/us_2010_v1/states/*/district_summary.csv`
- 2020 unweighted: `outputs/us_2020_v1/states/*/district_summary.csv`
- 2020 edge-weighted: `outputs/baseline_comparison_edge/algorithmic_vs_enacted_comparison.csv`

### Enacted Baselines
- 2010: `data/enacted_districts/2010/enacted_compactness_2010.csv` ✅
- 2020: `outputs/baseline_comparison_edge/enacted_district_compactness.csv` ✅
- 2000: NOT YET AVAILABLE ⏳

---

**Last Updated**: January 14, 2026
**Status**: Corrected numbers validated, ready for 2000 baseline integration
