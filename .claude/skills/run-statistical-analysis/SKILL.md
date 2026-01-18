---
name: run-statistical-analysis
description: Perform quantitative analysis of redistricting results. Computes statistics (means, medians, standard deviations), comparisons across years/modes, generates comparison tables, and creates statistical plots. Use when analyzing algorithm performance, compactness improvements, partisan fairness, or demographic representation.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Run Statistical Analysis

Perform comprehensive quantitative analysis of redistricting results. Generates statistical summaries, comparison tables, visualizations for academic papers/research.

## Prerequisites
**Data**: Pipeline outputs (`outputs/us_{year}_{version}/`), district assignments (50 states), analysis data (political_lean, demographic_composition, compactness)
**Software**: Python 3.13+ with pandas, numpy, scipy, matplotlib
**Check**: `ls outputs/us_2020_v1/states/*/data/{districts,district_summary}.csv`

## When to Use
User says "Analyze redistricting results/Generate statistics/Compare edge-weighted vs unweighted/Calculate compactness improvements", needs quantitative metrics for paper, wants statistical comparison across census years, validate algorithm performance

## Analysis Types

**1. Population Deviation**: Max deviation per state, mean absolute deviation, standard deviation, % districts within ±0.5% → `paper/data/population_stats.csv` + summary (mean/median/min/max)
**2. Compactness**: Polsby-Popper scores (0-1 higher=compact), Reock scores (0-1), mean/median/std per state, distribution histograms → `paper/data/compactness_stats.csv` + comparison tables
**3. Political** (2020 only): Partisan lean per district (% D vs R), seat totals by party, efficiency gap, mean-median difference, partisan bias metrics → `paper/data/political_stats.csv` + comparison to current maps
**4. Demographic**: Racial/ethnic composition per district, majority-minority districts count, representation ratios, diversity indices → `paper/data/demographic_stats.csv`

## Workflow

### Step 1: Identify Data Source
Ask: Year (2000/2010/2020), Version (v1/v2/test), Mode (edge-weighted/unweighted for comparisons)

### Step 2: Run Analysis Script
```bash
python scripts/analysis/statistical_analysis.py --year 2020 --version v1
```
**With comparisons**: `--compare-versions v1,v2` or `--compare-years 2010,2020` or `--compare-modes weighted,unweighted`

### Step 3: Monitor Execution
```
[1/4] Loading data: 51 states
[2/4] Computing population stats: Mean dev 0.12%, Max 0.48%
[3/4] Computing compactness: Mean PP 0.42 (↑15% vs unweighted)
[4/4] Generating tables: 4 CSV files, 6 plots
Complete! Results in paper/data/
```
**Runtime**: ~30-60s

### Step 4: Review Outputs
```bash
ls paper/data/
```
**Expected**: `population_stats.csv`, `compactness_stats.csv`, `political_stats.csv` (2020), `demographic_stats.csv`, `comparison_*.csv` (if comparisons), `plots/*.png`

## Statistical Methods

**Descriptive**: Mean, median, std, min/max, percentiles (25th/75th), IQR
**Inferential**: Paired t-tests (compare modes/years), effect sizes (Cohen's d), confidence intervals (95%)
**Correlation**: Pearson/Spearman correlation matrices (compactness vs political lean)
**Distribution**: Histograms, box plots, violin plots, Q-Q plots (normality check)

## Comparison Analysis

### Compare Modes (Edge-weighted vs Unweighted)
```bash
--compare-modes weighted,unweighted --years 2020 --versions v1
```
**Output**: `comparison_modes.csv` (side-by-side metrics), `comparison_modes_plot.png` (visual), statistical tests (paired t-test for compactness improvement, effect size Cohen's d)

### Compare Years (2000 vs 2010 vs 2020)
```bash
--compare-years 2000,2010,2020 --version v1
```
**Output**: `comparison_years.csv`, `comparison_years_plot.png`, trend analysis (linear regression for compactness over time)

### Compare Versions (Algorithm iterations)
```bash
--compare-versions v1,v2,v3 --year 2020
```
**Output**: `comparison_versions.csv`, `comparison_versions_plot.png`, identify best version (highest compactness, lowest population deviation)

## Output Tables

### Population Stats Table
**Columns**: State, Districts, Mean Dev (%), Max Dev (%), Std Dev (%), Within ±0.5% (%), Target Pop
**Example**: `california, 52, 0.08, 0.42, 0.15, 100.0, 762,000`

### Compactness Stats Table
**Columns**: State, Mean PP, Median PP, Std PP, Mean Reock, Median Reock, Std Reock, Min PP, Max PP
**Example**: `california, 0.38, 0.36, 0.12, 0.51, 0.49, 0.09, 0.18, 0.72`

### Political Stats Table (2020)
**Columns**: State, D Seats, R Seats, D Vote %, R Vote %, Efficiency Gap, Mean-Median, Bias
**Example**: `california, 42, 10, 63.2, 34.5, -0.12, 0.08, D+2.1`

### Demographic Stats Table
**Columns**: State, Majority-Minority Districts, White %, Black %, Hispanic %, Asian %, Other %, Diversity Index
**Example**: `california, 15, 41.2, 5.8, 38.7, 13.2, 1.1, 0.72`

## Plots Generated

**1. Compactness Distribution**: Histograms PP/Reock scores (all districts), violin plots by state (top 10), comparison box plots (modes/years)
**2. Population Deviation**: Histogram of deviations, box plot by state, scatter (target pop vs max deviation)
**3. Political Lean**: Histogram partisan lean (2020), scatter (vote % vs seat %), efficiency gap plot
**4. Demographic Composition**: Stacked bar charts (racial composition by state), scatter (diversity index vs compactness)

## Statistical Tests

**Compactness Improvement** (edge-weighted vs unweighted): Paired t-test (H₀: no difference), effect size Cohen's d, 95% CI for mean improvement
**Typical result**: Mean PP improvement 15.2% (t=12.4, p<0.001, d=0.89 large effect, CI [12.8%, 17.6%])

**Population Balance**: One-sample t-test (H₀: mean deviation = 0), check if max deviations within ±0.5%, binomial test for % compliant districts
**Typical result**: Mean deviation 0.12% (t=8.2, p<0.001 but practically negligible), 98.4% districts within ±0.5%

## Troubleshooting

**Missing data**: `Error: No data found for california` → Run redistricting or analysis first
**Import errors**: `ModuleNotFoundError: scipy` → `pip install scipy matplotlib`
**Comparison fails**: Not enough data → Ensure all compared versions/years/modes exist
**Plot generation fails**: Memory error → Reduce plot count or simplify visualizations

## Interpretation Guidelines

**Compactness PP**: 0.0-0.2 (very irregular), 0.2-0.3 (irregular), 0.3-0.4 (moderate), 0.4-0.5 (compact), >0.5 (very compact). Typical: Algorithmic districts 0.35-0.45, current congressional 0.25-0.35
**Population Deviation**: <0.1% (excellent), 0.1-0.3% (good), 0.3-0.5% (acceptable), >0.5% (non-compliant). Legal requirement: ±0.5%
**Efficiency Gap**: -7% to +7% (fair), outside (potential bias). Positive = R advantage, negative = D advantage
**Effect Size Cohen's d**: 0.2 (small), 0.5 (medium), 0.8 (large)

## Best Practices

Use paired comparisons (same states across modes/years), report effect sizes (not just p-values), include confidence intervals, validate normality assumptions (Q-Q plots), correct for multiple comparisons (Bonferroni if needed), visualize distributions (not just means), document statistical choices

## What You'll Get

Comprehensive statistics tables (population/compactness/political/demographic), comparison analyses (modes/years/versions), statistical plots (distributions/comparisons/trends), hypothesis tests (t-tests/effect sizes/CIs), LaTeX-ready tables (copy to paper), publication-quality figures (300 DPI)

## Related Skills
`/run-redistricting` (generate data), `/validate-compactness` (validate metrics), `/run-experiment` (test algorithm variants), `/parameter-sweep` (parameter sensitivity)

## Next Steps
Include tables in paper (LaTeX format provided), interpret statistical significance (practical vs statistical), discuss compactness improvements, compare to current congressional districts, publish figures in manuscript
