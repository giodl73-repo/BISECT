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

## Overview

Perform comprehensive quantitative analysis of redistricting results. Generates statistical summaries, comparison tables, and visualizations for academic papers and research presentations.

## Prerequisites

**Required Data**:
- Redistricting pipeline outputs in `outputs/us_{year}_{version}/`
- District assignments for all 50 states
- Analysis data (political_lean, demographic_composition, compactness)

**Required Software**:
- Python 3.13+ with pandas, numpy, scipy, matplotlib installed

**Check prerequisites**:
```bash
ls outputs/us_2020_v1/states/*/data/districts.csv
ls outputs/us_2020_v1/states/*/data/district_summary.csv
```

## When to Use This Skill

- User says: "Analyze the redistricting results" or "Generate statistics"
- User says: "Compare edge-weighted vs unweighted mode"
- User says: "Calculate compactness improvements"
- User needs quantitative metrics for a paper
- User wants statistical comparison across census years
- User needs to validate algorithm performance

## Analysis Types

### 1. Population Deviation Statistics

**What it measures**:
- Maximum population deviation per state
- Mean absolute deviation
- Standard deviation
- Percentage of districts within ±0.5% target

**Output**:
- `paper/data/population_stats.csv` - Per-state population statistics
- Summary table with mean, median, min, max across all states

### 2. Compactness Statistics

**What it measures**:
- **Polsby-Popper** scores (0-1, higher = more compact)
- **Reock** scores (0-1, higher = more compact)
- Mean, median, standard deviation per state
- Distribution histograms

**Output**:
- `paper/data/compactness_stats.csv` - Per-state compactness statistics
- Comparison tables across modes/years

### 3. Political Statistics (2020 only)

**What it measures**:
- Partisan lean per district (% Democratic vs Republican)
- Seat totals by party
- Efficiency gap
- Mean-median difference
- Partisan bias metrics

**Output**:
- `paper/data/political_stats.csv` - Per-state partisan metrics
- Comparison to current congressional maps

### 4. Demographic Statistics

**What it measures**:
- Racial/ethnic composition per district
- Majority-minority districts count
- Representation ratios
- Diversity indices

**Output**:
- `paper/data/demographic_stats.csv` - Per-state demographic statistics

## Workflow

### Step 1: Identify Data Source

Ask user if not specified:
- **Year**: Census year (2000, 2010, 2020)
- **Version**: Pipeline version (v1, v2, test, etc.)
- **Mode**: Edge-weighted or unweighted (for comparisons)

**Example**:
- Source: `outputs/us_2020_v1/` (2020 census, version v1, edge-weighted)
- Compare to: `outputs/us_2020_v1_unweighted/` (same year, unweighted mode)

### Step 2: Navigate to Analysis Directory

```bash
cd papers/01_recursive_bisection/analysis
```

### Step 3: Run Statistical Analysis

**Generate all statistics**:
```bash
python generate_all_statistics.py --output-dir ../../../outputs/us_2020_v1
```

**This runs in sequence**:
1. `compute_population_stats.py` - Population deviation analysis
2. `compute_compactness_stats.py` - Compactness metrics
3. `compute_political_stats.py` - Partisan lean analysis (2020 only)
4. `compute_demographic_stats.py` - Demographic composition
5. `select_example_state.py` - Pick representative state for figures

**Run individual analysis**:
```bash
# Just compactness
python compute_compactness_stats.py --output-dir ../../../outputs/us_2020_v1

# Just political
python compute_political_stats.py --output-dir ../../../outputs/us_2020_v1
```

### Step 4: Review Generated Files

**Data files** (CSV tables):
```bash
ls ../paper/data/
# population_stats.csv
# compactness_stats.csv
# political_stats.csv
# demographic_stats.csv
```

**Figure files** (PNG visualizations):
```bash
ls ../paper/figures/
# compactness_histogram.png
# partisan_lean_distribution.png
# population_deviation_boxplot.png
```

### Step 5: Include in Paper

**LaTeX table example**:
```latex
\begin{table}
\centering
\caption{Compactness Statistics by State}
\input{data/compactness_stats_table.tex}
\end{table}
```

**LaTeX figure example**:
```latex
\begin{figure}
\centering
\includegraphics[width=0.8\textwidth]{figures/compactness_histogram.png}
\caption{Distribution of Polsby-Popper compactness scores across 435 districts}
\end{figure}
```

## Comparison Analysis

### Compare Two Modes

**Scenario**: Compare edge-weighted vs unweighted mode

**Step 1**: Run both pipelines
```bash
# Edge-weighted (default)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1

# Unweighted
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1_unweighted --mode unweighted
```

**Step 2**: Analyze both
```bash
# Analyze edge-weighted
python generate_all_statistics.py --output-dir outputs/us_2020_v1

# Analyze unweighted
python generate_all_statistics.py --output-dir outputs/us_2020_v1_unweighted
```

**Step 3**: Create comparison table
```python
import pandas as pd

# Load both results
edge_weighted = pd.read_csv('paper/data/compactness_stats.csv')
unweighted = pd.read_csv('paper/data_unweighted/compactness_stats.csv')

# Calculate improvement
improvement = (edge_weighted['mean_pp'] - unweighted['mean_pp']) / unweighted['mean_pp'] * 100

print(f"Mean Polsby-Popper improvement: {improvement.mean():.1f}%")
```

### Compare Across Census Years

**Scenario**: Compare 2010 vs 2020 redistricting

```bash
# Analyze 2010
python generate_all_statistics.py --output-dir outputs/us_2010_v1

# Analyze 2020
python generate_all_statistics.py --output-dir outputs/us_2020_v1

# Compare results
```

**Key differences to check**:
- Population growth impact on compactness
- Apportionment changes (states gaining/losing districts)
- Data quality (2020 has better census tract boundaries)

## Statistical Tests

### Paired T-Test (Edge-Weighted vs Unweighted)

**Test**: Is edge-weighted significantly more compact?

```python
from scipy import stats
import pandas as pd

# Load compactness data for both modes
ew = pd.read_csv('paper/data/compactness_stats_ew.csv')
uw = pd.read_csv('paper/data/compactness_stats_uw.csv')

# Paired t-test (per state)
t_stat, p_value = stats.ttest_rel(ew['mean_pp'], uw['mean_pp'])

print(f"t-statistic: {t_stat:.3f}")
print(f"p-value: {p_value:.4f}")

if p_value < 0.05:
    print("Statistically significant improvement (p < 0.05)")
```

### Effect Size (Cohen's d)

**Measure**: Magnitude of improvement

```python
import numpy as np

# Calculate effect size
mean_diff = ew['mean_pp'].mean() - uw['mean_pp'].mean()
pooled_std = np.sqrt((ew['mean_pp'].std()**2 + uw['mean_pp'].std()**2) / 2)
cohens_d = mean_diff / pooled_std

print(f"Cohen's d: {cohens_d:.3f}")

# Interpretation:
# |d| < 0.2: small effect
# |d| ~ 0.5: medium effect
# |d| > 0.8: large effect
```

### Confidence Intervals

**95% CI for mean compactness**:

```python
from scipy import stats

mean_pp = ew['mean_pp']
ci = stats.t.interval(0.95, len(mean_pp)-1,
                      loc=mean_pp.mean(),
                      scale=stats.sem(mean_pp))

print(f"95% CI: [{ci[0]:.4f}, {ci[1]:.4f}]")
```

## Output File Formats

### Population Statistics CSV

```csv
state,num_districts,target_pop,max_deviation,mean_abs_deviation,pct_within_0.5
alabama,7,704263,0.0032,0.0018,100.0
alaska,1,731545,0.0000,0.0000,100.0
arizona,9,763646,0.0041,0.0021,100.0
...
```

### Compactness Statistics CSV

```csv
state,num_districts,mean_pp,median_pp,std_pp,min_pp,max_pp,mean_reock,median_reock
alabama,7,0.4521,0.4389,0.0821,0.3012,0.5892,0.5623,0.5512
alaska,1,0.0892,0.0892,0.0000,0.0892,0.0892,0.1234,0.1234
arizona,9,0.4123,0.4089,0.0654,0.2987,0.5234,0.5123,0.5089
...
```

### Political Statistics CSV (2020 only)

```csv
state,num_districts,dem_seats,rep_seats,mean_dem_pct,median_dem_pct,efficiency_gap,mean_median_diff
alabama,7,1,6,36.2,34.5,0.142,-1.7
arizona,9,5,4,50.8,50.1,0.012,0.7
california,52,40,12,63.4,64.2,-0.051,0.8
...
```

## Visualization Types

### 1. Histogram: Compactness Distribution

**Shows**: Distribution of Polsby-Popper scores across all 435 districts

**Purpose**: Visualize overall compactness quality

**Example**:
```python
import matplotlib.pyplot as plt
import pandas as pd

# Load all district compactness scores
districts = []
for state in states:
    df = pd.read_csv(f'outputs/us_2020_v1/states/{state}/data/compactness.csv')
    districts.extend(df['polsby_popper'].tolist())

# Plot histogram
plt.hist(districts, bins=20, edgecolor='black')
plt.xlabel('Polsby-Popper Score')
plt.ylabel('Number of Districts')
plt.title('Distribution of Compactness Scores (435 Districts)')
plt.axvline(np.mean(districts), color='red', linestyle='--', label=f'Mean: {np.mean(districts):.3f}')
plt.legend()
plt.savefig('figures/compactness_histogram.png', dpi=150)
```

### 2. Box Plot: Population Deviation by State

**Shows**: Population balance variability across states

**Purpose**: Show algorithm achieves tight population balance

### 3. Scatter Plot: Compactness vs District Size

**Shows**: Relationship between district geographic size and compactness

**Purpose**: Identify outliers (e.g., Alaska, Montana at-large districts)

### 4. Bar Chart: Partisan Seat Totals

**Shows**: D vs R seats per state

**Purpose**: Show partisan fairness compared to vote share

## Troubleshooting

### Missing Data Files

```
FileNotFoundError: [Errno 2] No such file or directory: 'outputs/us_2020_v1/states/alabama/data/compactness.csv'
```

**Solution**: Run analysis first
```bash
/run-analysis-only --year 2020 --version v1
```

### Political Analysis Fails (2010/2000)

```
ValueError: No 2020 election data available for 2010 census
```

**Expected**: Political analysis only available for 2020
- Skip political analysis for 2010/2000
- Or use historical election data (requires manual setup)

### Import Errors

```
ModuleNotFoundError: No module named 'scipy'
```

**Solution**: Install scientific Python packages
```bash
pip install pandas numpy scipy matplotlib seaborn
```

### Division by Zero Warnings

```
RuntimeWarning: invalid value encountered in double_scalars
```

**Cause**: Some districts may have 0 area (data issue) or undefined metrics
**Solution**: Filter out invalid values before computing statistics

## Best Practices

1. **Run full pipeline first**: Ensure all analysis data available
2. **Check data completeness**: Verify all 50 states present
3. **Document parameters**: Record year, version, mode used
4. **Save intermediate results**: Keep CSV files for reproducibility
5. **Version control data**: Track which pipeline version generated statistics
6. **Compare to baselines**: Include current congressional districts for context
7. **Report effect sizes**: Not just p-values (statistical significance vs practical importance)
8. **Check assumptions**: Normality, independence for parametric tests

## Related Skills

- `/run-redistricting` - Generate data to analyze
- `/run-analysis-only` - Regenerate analysis without redistricting
- `/run-experiment` - Compare algorithm variants systematically
- `/parameter-sweep` - Test parameter sensitivity
- `/compile-latex` - Include statistics in paper

## Common Use Cases

### Paper: Compactness Improvement

**Goal**: Show edge-weighted mode improves compactness

**Workflow**:
1. Run both modes: `/run-redistricting` with `--mode weighted` and `--mode unweighted`
2. Analyze both: `/run-statistical-analysis` for each
3. Compute improvements: Compare mean Polsby-Popper scores
4. Statistical test: Paired t-test for significance
5. Report results: "Edge-weighted mode improved compactness by 52.8% (p < 0.001)"

### Paper: Population Balance

**Goal**: Show algorithm achieves ±0.5% population balance

**Workflow**:
1. Run pipeline: `/run-redistricting --year 2020 --version v1`
2. Analyze: `/run-statistical-analysis`
3. Report: "99.8% of districts within ±0.5% of target population"

### Presentation: Algorithm Comparison

**Goal**: Visual comparison of different approaches

**Workflow**:
1. Generate data: Run multiple modes/parameters
2. Analyze all: `/run-statistical-analysis` for each
3. Create comparison plots: Box plots, bar charts
4. Include in slides: `/compile-latex --type presentation`

## Performance

**Typical runtime**:
- Population statistics: ~10 seconds (all 50 states)
- Compactness statistics: ~30 seconds (geometry calculations)
- Political statistics: ~20 seconds (2020 data aggregation)
- Demographic statistics: ~15 seconds
- **Total**: ~1-2 minutes for all analyses

**Bottlenecks**:
- Reading 50 state CSV files
- Computing geometric metrics (Reock circles)
- Aggregating district-level to national-level statistics

## Next Steps

After generating statistics:
- Review CSV files for accuracy
- Include tables in LaTeX paper
- Generate comparison plots
- Document quantitative improvements
- Prepare statistical significance tests
- Share results with collaborators
