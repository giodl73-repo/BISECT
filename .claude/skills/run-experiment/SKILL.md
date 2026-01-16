---
name: run-experiment
description: Test algorithm variants and compare results. Defines experiment parameters (states, variants, metrics), runs redistricting for each variant, collects metrics (compactness, partisan lean, computation time), and performs statistical comparison with paired t-tests, effect sizes, and confidence intervals. Use when comparing edge-weighted vs unweighted, testing new algorithms, or validating improvements.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - TodoWrite
user-invocable: true
---

# Run Experiment

## Overview

Design and execute controlled experiments to compare redistricting algorithm variants. Systematically test different configurations, collect quantitative metrics, and perform statistical analysis to validate improvements.

## Prerequisites

**Required**:
- Full redistricting pipeline functional
- Census data and adjacency graphs for target year/states
- Python 3.13+ with pandas, numpy, scipy, matplotlib

**Recommended**:
- Multiple states selected for testing (small, medium, large)
- Baseline results available for comparison
- Clear hypothesis about expected improvements

## When to Use This Skill

- User says: "Compare edge-weighted vs unweighted mode"
- User says: "Test algorithm changes" or "Validate improvements"
- User wants to compare different METIS parameters
- User needs quantitative evidence for paper/presentation
- User wants to test new feature against baseline
- User needs statistical significance testing

## Experiment Types

### 1. Mode Comparison (Edge-Weighted vs Unweighted)

**Hypothesis**: Edge-weighted mode produces more compact districts

**Variables**:
- **Independent**: Mode (edge-weighted vs unweighted)
- **Dependent**: Polsby-Popper compactness, Reock compactness, perimeter length
- **Controlled**: Census year, states, population tolerance

**Example**:
```bash
# Baseline: Unweighted mode
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version exp_unweighted --mode unweighted \
  --states "AL,CA,TX,FL,NY,IL,PA,OH,GA,NC"

# Treatment: Edge-weighted mode
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version exp_weighted --mode weighted \
  --states "AL,CA,TX,FL,NY,IL,PA,OH,GA,NC"
```

### 2. Parameter Sensitivity

**Hypothesis**: Population tolerance affects compactness

**Variables**:
- **Independent**: Population tolerance (ufactor)
- **Dependent**: Compactness, max population deviation
- **Controlled**: Mode, states, census year

**Example**:
```bash
# Test different ufactor values
for ufactor in 1.001 1.005 1.01 1.02; do
  python scripts/pipeline/run_complete_redistricting.py \
    --year 2020 --version exp_ufactor_${ufactor} \
    --ufactor ${ufactor} --states "AL,GA,NC"
done
```

### 3. Multi-Year Comparison

**Hypothesis**: 2020 census data produces more compact districts than 2010

**Variables**:
- **Independent**: Census year (2010 vs 2020)
- **Dependent**: Compactness, population balance
- **Controlled**: Mode, algorithm, states (where districts unchanged)

**Example**:
```bash
# 2010 census
python scripts/pipeline/run_complete_redistricting.py \
  --year 2010 --version exp_2010 --states "AL,AZ,CO,IN,KY"

# 2020 census
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version exp_2020 --states "AL,AZ,CO,IN,KY"
```

### 4. State Selection

**Hypothesis**: Algorithm performance consistent across state sizes

**Variables**:
- **Independent**: State size (small, medium, large)
- **Dependent**: Compactness, runtime
- **Controlled**: Mode, census year, parameters

**Test states**:
- **Small**: Vermont (1), Delaware (1), Wyoming (1), Alaska (1)
- **Medium**: Alabama (7), Minnesota (8), Missouri (8)
- **Large**: California (52), Texas (38), Florida (28), New York (26)

## Workflow

### Step 1: Define Experiment

**Research Question**: What are you testing?

**Example**: "Does edge-weighted mode significantly improve compactness compared to unweighted mode?"

**Hypothesis**: Edge-weighted > Unweighted for mean Polsby-Popper score

**Metrics to track**:
- Primary: Mean Polsby-Popper compactness
- Secondary: Reock compactness, perimeter length
- Control: Population deviation, runtime

### Step 2: Select States

**Considerations**:
- **Sample size**: At least 5-10 states for statistical power
- **Diversity**: Mix of geographic regions, sizes, complexities
- **Practical**: Small states for quick tests, large states for realism
- **Apportionment**: Use states where district count unchanged (for temporal comparisons)

**Recommended test sets**:

**Quick test** (5 states, ~5-10 minutes):
```
VT, DE, WY, AL, MN
```

**Medium test** (10 states, ~20-30 minutes):
```
VT, DE, WY, AK, AL, GA, MN, MO, CO, AZ
```

**Full test** (20 states, ~1-2 hours):
```
All states except CA, TX, FL (to reduce runtime)
```

### Step 3: Run Baseline

```bash
# Create TodoWrite task list
# Mark experiment setup as in_progress

# Run baseline (control condition)
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version exp_baseline \
  --mode unweighted \
  --states "AL,GA,NC,MN,MO,CO,AZ,VA,IN,KY"

# Check outputs
ls outputs/us_2020_exp_baseline/states/*/data/district_summary.csv
```

### Step 4: Run Treatment

```bash
# Run treatment (experimental condition)
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version exp_treatment \
  --mode weighted \
  --states "AL,GA,NC,MN,MO,CO,AZ,VA,IN,KY"

# Check outputs
ls outputs/us_2020_exp_treatment/states/*/data/district_summary.csv
```

### Step 5: Collect Metrics

**Extract compactness data**:
```python
import pandas as pd
from pathlib import Path

states = ['alabama', 'georgia', 'north_carolina', 'minnesota', 'missouri',
          'colorado', 'arizona', 'virginia', 'indiana', 'kentucky']

# Baseline
baseline_pp = []
for state in states:
    df = pd.read_csv(f'outputs/us_2020_exp_baseline/states/{state}/data/compactness.csv')
    baseline_pp.extend(df['polsby_popper'].tolist())

# Treatment
treatment_pp = []
for state in states:
    df = pd.read_csv(f'outputs/us_2020_exp_treatment/states/{state}/data/compactness.csv')
    treatment_pp.extend(df['polsby_popper'].tolist())

# Compute aggregate statistics
baseline_mean = pd.Series(baseline_pp).mean()
treatment_mean = pd.Series(treatment_pp).mean()
improvement = (treatment_mean - baseline_mean) / baseline_mean * 100

print(f"Baseline mean PP: {baseline_mean:.4f}")
print(f"Treatment mean PP: {treatment_mean:.4f}")
print(f"Improvement: {improvement:.2f}%")
```

### Step 6: Statistical Analysis

**Paired t-test** (if comparing same states):
```python
from scipy import stats
import numpy as np

# Per-state means
baseline_state_means = []
treatment_state_means = []

for state in states:
    df_b = pd.read_csv(f'outputs/us_2020_exp_baseline/states/{state}/data/compactness.csv')
    df_t = pd.read_csv(f'outputs/us_2020_exp_treatment/states/{state}/data/compactness.csv')

    baseline_state_means.append(df_b['polsby_popper'].mean())
    treatment_state_means.append(df_t['polsby_popper'].mean())

# Paired t-test
t_stat, p_value = stats.ttest_rel(treatment_state_means, baseline_state_means)

print(f"t-statistic: {t_stat:.3f}")
print(f"p-value: {p_value:.6f}")

if p_value < 0.05:
    print("*** Statistically significant improvement (p < 0.05) ***")
else:
    print("Not statistically significant (p >= 0.05)")
```

**Effect size (Cohen's d)**:
```python
mean_diff = np.mean(treatment_state_means) - np.mean(baseline_state_means)
pooled_std = np.sqrt((np.std(treatment_state_means)**2 + np.std(baseline_state_means)**2) / 2)
cohens_d = mean_diff / pooled_std

print(f"Cohen's d: {cohens_d:.3f}")
print(f"Effect size: {'small' if abs(cohens_d) < 0.5 else 'medium' if abs(cohens_d) < 0.8 else 'large'}")
```

### Step 7: Visualize Results

**Box plot comparison**:
```python
import matplotlib.pyplot as plt

fig, ax = plt.subplots(figsize=(8, 6))

positions = [1, 2]
data = [baseline_state_means, treatment_state_means]
labels = ['Unweighted (Baseline)', 'Edge-Weighted (Treatment)']

bp = ax.boxplot(data, positions=positions, labels=labels, patch_artist=True)
bp['boxes'][0].set_facecolor('lightcoral')
bp['boxes'][1].set_facecolor('lightblue')

ax.set_ylabel('Mean Polsby-Popper Score')
ax.set_title('Compactness Comparison: Edge-Weighted vs Unweighted')
ax.grid(axis='y', alpha=0.3)

# Add statistical annotation
if p_value < 0.05:
    y_max = max(max(baseline_state_means), max(treatment_state_means))
    ax.plot([1, 2], [y_max * 1.05, y_max * 1.05], 'k-', linewidth=1)
    ax.text(1.5, y_max * 1.07, f'p = {p_value:.4f}', ha='center', fontweight='bold')

plt.tight_layout()
plt.savefig('experiment_comparison.png', dpi=150)
print("Saved: experiment_comparison.png")
```

### Step 8: Document Results

**Create experiment report**:
```markdown
# Experiment: Edge-Weighted vs Unweighted Mode

## Hypothesis
Edge-weighted mode produces significantly more compact districts than unweighted mode.

## Method
- States: AL, GA, NC, MN, MO, CO, AZ, VA, IN, KY (N=10)
- Census year: 2020
- Metric: Mean Polsby-Popper compactness score
- Test: Paired t-test (two-tailed, α=0.05)

## Results
- Baseline mean PP: 0.3421 ± 0.0623 (SD)
- Treatment mean PP: 0.5234 ± 0.0512 (SD)
- Improvement: +52.98%
- t(9) = 8.234, p < 0.001
- Cohen's d = 1.67 (large effect size)

## Conclusion
Edge-weighted mode significantly improved compactness (p < 0.001) with a large effect size (d = 1.67). Results support hypothesis.
```

## Experiment Design Best Practices

### 1. Control Variables

**Keep constant across conditions**:
- Census year
- Population tolerance (ufactor)
- METIS algorithm (recursive bisection)
- State selection
- Random seed (if applicable)

**Only vary**:
- Independent variable (mode, parameter, etc.)

### 2. Sample Size

**Minimum recommended**:
- **Pilot test**: 3-5 states
- **Full experiment**: 10-20 states
- **Publication**: 20-50 states (or all 50)

**Power analysis**: For detecting medium effect (d=0.5) with 80% power:
- Need N ≥ 34 states (paired design)
- Need N ≥ 64 states per group (independent design)

### 3. Replication

**Recommended**:
- Run each condition 2-3 times if randomness involved
- Use fixed seed for reproducibility
- Document all parameters

### 4. Validation

**Check assumptions**:
- Normality: Shapiro-Wilk test
- Equal variances: Levene's test
- Independence: No overlap in state selection

## Common Experiments

### Experiment 1: Mode Comparison

**Research question**: Does edge-weighted mode improve compactness?

**Design**:
- **Conditions**: Unweighted vs Edge-weighted
- **States**: 10 diverse states
- **Metric**: Mean Polsby-Popper
- **Test**: Paired t-test

**Expected runtime**: ~30-45 minutes (10 states × 2 conditions)

### Experiment 2: Population Tolerance

**Research question**: How does ufactor affect compactness?

**Design**:
- **Conditions**: ufactor = 1.001, 1.005, 1.01, 1.02, 1.05
- **States**: 5 medium-sized states
- **Metric**: Mean PP, max deviation
- **Test**: ANOVA + post-hoc Tukey

**Expected runtime**: ~20-30 minutes (5 states × 5 conditions)

### Experiment 3: Algorithm Scaling

**Research question**: Does algorithm scale to large states?

**Design**:
- **Conditions**: State size (small, medium, large)
- **States**: 3 per category (9 total)
- **Metric**: Runtime, compactness
- **Test**: Correlation analysis

**Expected runtime**: ~45-60 minutes (includes CA, TX, FL)

## Troubleshooting

### Inconsistent Results

**Symptom**: Metrics vary between runs

**Causes**:
- METIS has random component (initial partition)
- Floating point precision differences
- Different data files used

**Solutions**:
- Use fixed seed: `--seed 42`
- Run multiple replicates (3-5)
- Report mean ± SD across replicates

### Statistical Non-Significance

**Symptom**: p > 0.05 despite visible improvement

**Causes**:
- Sample size too small
- High variance in state metrics
- Effect size truly small

**Solutions**:
- Increase N (more states)
- Use non-parametric test (Wilcoxon signed-rank)
- Report effect size (practical significance)
- Consider district-level analysis (more power)

### Runtime Too Long

**Symptom**: Experiment takes hours

**Solutions**:
- Start with pilot (3-5 states)
- Use small/medium states only
- Run overnight for full 50-state test
- Use `--print-only` to validate parameters first

### Missing Data

**Symptom**: Some states missing outputs

**Solutions**:
- Check pipeline logs for errors
- Re-run failed states individually
- Use `--force` to regenerate
- Exclude incomplete states from analysis

## Advanced Techniques

### Stratified Sampling

**Group states by characteristic, sample within groups**:

```python
# Stratify by district count
small = [s for s in states if districts[s] <= 3]      # Sample 3
medium = [s for s in states if 4 <= districts[s] <= 10]  # Sample 4
large = [s for s in states if districts[s] > 10]      # Sample 3

import random
sample = random.sample(small, 3) + random.sample(medium, 4) + random.sample(large, 3)
```

### Regression Analysis

**Model compactness as function of predictors**:

```python
import statsmodels.api as sm

# Predictors: mode, district count, state area, population density
X = pd.DataFrame({
    'mode': [0, 1] * len(states),  # 0 = unweighted, 1 = weighted
    'num_districts': districts_list * 2,
    'area': area_list * 2,
})
X = sm.add_constant(X)

# Response: mean compactness
y = baseline_pp + treatment_pp

# Fit model
model = sm.OLS(y, X).fit()
print(model.summary())
```

### Bootstrapping Confidence Intervals

**Robust CI without normality assumption**:

```python
from scipy.stats import bootstrap

def mean_diff(x, y):
    return np.mean(x) - np.mean(y)

# Bootstrap CI for improvement
res = bootstrap((baseline_pp, treatment_pp), mean_diff, n_resamples=10000, method='percentile')
ci_low, ci_high = res.confidence_interval

print(f"95% CI for improvement: [{ci_low:.4f}, {ci_high:.4f}]")
```

## Related Skills

- `/run-redistricting` - Execute pipeline for experiment conditions
- `/run-statistical-analysis` - Compute detailed metrics
- `/parameter-sweep` - Test multiple parameter values systematically
- `/validate-compactness` - Verify compactness improvements

## Performance

**Typical experiment runtimes**:

| States | Conditions | Total Runtime |
|--------|-----------|--------------|
| 5 (small) | 2 | ~10-15 min |
| 10 (mixed) | 2 | ~30-45 min |
| 20 (mixed) | 2 | ~2-3 hours |
| 50 (all) | 2 | ~6-8 hours |

**Bottlenecks**:
- Large states (CA, TX, FL): 10-20 min each
- Multiple conditions: Linear scaling
- Full pipeline: Adjacency + redistricting + analysis

**Optimization**:
- Use `--skip-analysis` for redistricting-only tests
- Reuse adjacency graphs across conditions
- Run parallel (separate terminals for each condition)

## Next Steps

After experiment:
- Document methodology in paper methods section
- Include statistical tests in results
- Create comparison visualizations for figures
- Archive experiment outputs for reproducibility
- Share results in presentation or publication
