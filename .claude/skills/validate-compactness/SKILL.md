---
name: validate-compactness
description: Validate redistricting maintains/improves compactness. Loads district geometries, computes compactness metrics (Polsby-Popper, Reock), compares to baselines (current congressional districts, historical districts, random partitions), reports improvements/regressions, and identifies outlier districts. Use when verifying algorithm produces compact districts.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Validate Compactness

Verify algorithmically-generated districts are compact via standard metrics + benchmark comparisons (current congressional, historical, random partitions, theoretical bounds).

## Prerequisites
**Required**: Pipeline outputs with district assignments, census tract geometries, Python 3.13+ with geopandas, shapely, pandas, numpy, matplotlib
**Optional** (comparisons): Current congressional boundaries, historical district boundaries (2010/2000), random partition baseline

## When to Use
User says "Check if districts compact/Validate compactness/Compare to current congressional", verify algorithm improves over baseline, needs quantitative evidence for paper, suspects non-compact districts (verification), wants to identify problematic districts (outliers)

## Compactness Metrics

**1. Polsby-Popper (PP)**: `(4π × Area) / Perimeter²`, range [0,1] (0=extremely non-compact, 1=perfect circle)
- PP <0.10: Highly gerrymandered (Illinois 4th)
- PP ~0.20-0.30: Typical current US districts
- PP ~0.40-0.50: Good compactness
- PP >0.60: Excellent compactness
- **Advantages**: Intuitive geometric, easy compute, widely used
- **Limitations**: Sensitive to boundary irregularities (coastlines/rivers), penalizes necessary geographic features

**2. Reock**: `District Area / Minimum Bounding Circle Area`, range [0,1]
- Reock <0.20: Very non-compact
- Reock ~0.30-0.40: Typical current US
- Reock ~0.50-0.60: Good
- Reock >0.70: Excellent
- **Advantages**: Less sensitive to boundary irregularities, captures dispersion
- **Limitations**: More expensive (requires minimum bounding circle), can be high for elongated districts fitting circle

**3. Convex Hull Ratio (CHR)**: `District Area / Convex Hull Area`, range [0,1] (1=convex no concavities)
- CHR ~0.80-1.00: Good convexity
- CHR ~0.60-0.80: Moderate concavities
- CHR <0.60: Highly concave
- **Use**: Identifies districts with "arms" reaching out

**4. Cut Edges Ratio**: `Cut Edges / Total Edges`, range [0,1] (lower=better, fewer boundaries cut)
- **Use**: Graph-based compactness (algorithm directly minimizes this)

## Workflow

### Step 1: Run Compactness Validation
```bash
python scripts/validation/validate_compactness.py --year 2020 --version v1
```
**With baseline comparison**: `--baseline current` or `--baseline historical_2010` or `--baseline random`
**Output**: `outputs/us_2020_v1/validation/compactness_report.txt` + `compactness_comparison.csv` + plots

### Step 2: Review Validation Report
```
Compactness Validation Report
Year: 2020, Version: v1
States: 51, Total Districts: 435

Polsby-Popper:
  Mean: 0.42 (↑38% vs current congressional 0.30)
  Median: 0.40
  Std: 0.12
  Min: 0.18 (california-12)
  Max: 0.78 (wyoming-1)
  <0.20: 2 districts (0.5%)
  >0.40: 235 districts (54%)

Reock:
  Mean: 0.53 (↑25% vs current 0.42)
  Median: 0.51
  Std: 0.09
  Min: 0.28 (florida-5)
  Max: 0.89 (wyoming-1)

Statistical Test (vs Current Congressional):
  Paired t-test: t=15.3, p<0.001
  Effect size: d=1.02 (large)
  95% CI for mean diff: [0.10, 0.14]

Outliers Detected (PP < 0.20):
  1. california-12 (PP=0.18, Reock=0.31) - Urban coastal
  2. florida-5 (PP=0.19, Reock=0.28) - Rural panhandle

Recommendation: PASS - Algorithm produces significantly more compact districts
```

### Step 3: Examine Outliers
Check identified outliers: `ls outputs/us_2020_v1/states/{state}/maps/district_{num}.png`
**Common causes**: Coastlines (irregular boundaries), rivers (natural barriers), urban areas (irregular shapes), island connections (Hawaii)
**Determine if acceptable**: Geographic necessity (coastal/islands), still >0.15 (not extreme), better than current congressional for that district

### Step 4: Generate Comparison Plots
**Histogram**: PP/Reock distributions (algorithmic vs current), box plots (by state), scatter (PP vs Reock correlation), outlier visualization (geographic map with low-PP districts highlighted)

## Baseline Comparisons

**Current Congressional** (2020 enacted):
- Source: US Census Bureau TIGER/Line
- Expected: Algorithmic ~30-50% more compact
- Comparison: Paired t-test (same states), identify states with largest improvement

**Historical** (2010/2000):
- Source: Historical TIGER/Line
- Use: Track compactness trends over time
- Comparison: Linear regression (compactness over decades)

**Random Partitions**:
- Generate: 100 random contiguous partitions per state
- Compute: Mean/std PP across random samples
- Compare: Algorithm should significantly exceed random (proves optimization working)

**Theoretical Bounds**:
- Circle grid: Hexagonal packing (PP ~0.95)
- Square grid: Grid partitioning (PP ~0.64)
- Realistic upper bound: ~0.60-0.70 (accounting for geographic constraints)

## Validation Criteria

**Pass Criteria**:
- Mean PP >0.35 (significantly above current ~0.30)
- <5% districts with PP <0.20 (minimal gerrymandering)
- Statistically significant improvement (p<0.05, effect size >0.5)
- No extreme outliers (<0.10 PP) unless geographically justified

**Fail Criteria**:
- Mean PP <0.25 (worse than current)
- >10% districts with PP <0.20
- No statistical improvement over baseline
- Unjustified extreme outliers

## Troubleshooting

**Geometry errors**: `Invalid geometry for district X` → Repair with `buffer(0)` in district aggregation
**Missing baseline**: Can't find current congressional → Download from Census TIGER/Line, convert to same CRS
**Outlier false positive**: District flagged but looks reasonable → Check if coastal/island, compare to current congressional for same area, verify perimeter calculation correct
**Statistical test fails**: No significant difference → Check if using correct baseline, verify data loaded properly, consider edge-weighted vs unweighted comparison

## Output Files

`validation/compactness_report.txt` (summary statistics + outliers + recommendations), `validation/compactness_comparison.csv` (per-district metrics + baselines), `validation/plots/pp_histogram.png` (distribution comparison), `validation/plots/pp_by_state.png` (box plots), `validation/plots/outliers_map.png` (geographic outlier locations), `validation/plots/pp_vs_reock.png` (metric correlation scatter)

## Interpretation Guidelines

**PP vs Reock**: Both should improve together, if PP↑ but Reock→ check for elongated districts, if PP→ but Reock↑ check for convexity improvements
**State variation**: Some states naturally lower (coastlines, islands, irregular geography), compare within-state (algorithmic vs current), not across states
**Outliers**: <3% outliers acceptable (geography), >10% suggests algorithm issue
**Statistical significance**: p-value <0.05 + effect size >0.5 required, practical significance matters (mean improvement >0.05 PP meaningful)

## Best Practices

Always compare to baseline (current congressional minimum), report both PP and Reock (different aspects), identify outlier causes (geography vs algorithm), validate statistical significance (not just means), visualize distributions (not just summary stats), document geographic constraints (coastlines/islands explain lower scores), test on multiple years (verify consistency)

## What You'll Get

Validation report (pass/fail + statistics), comparison to baselines (current/historical/random), outlier identification (geographic causes), statistical tests (t-tests/effect sizes/CIs), visualization plots (distributions/box plots/scatter/maps), per-district CSV (all metrics for analysis), quantitative evidence (for papers/presentations)

## Related Skills
`/run-statistical-analysis` (comprehensive statistics), `/run-redistricting` (generate districts to validate), `/create-state-map` (visualize outliers), `/run-experiment` (test algorithm variants)

## Next Steps
Include validation in paper methods section, discuss outliers (geographic justification), compare to literature (cite typical PP ranges), identify states with largest improvements, investigate any fails (algorithm issues vs data issues)
