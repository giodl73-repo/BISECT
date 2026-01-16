---
name: parameter-sweep
description: Test algorithm with different parameter values. Defines parameter space (edge weight scaling factors, population tolerance, minimum tract populations), runs redistricting for each parameter combination, tracks metrics vs parameters, identifies optimal parameter values, and visualizes parameter sensitivity. Use when tuning algorithm parameters or understanding parameter impact.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - TodoWrite
user-invocable: true
---

# Parameter Sweep

## Overview

Systematically test algorithm behavior across a range of parameter values. Identify optimal settings, understand parameter sensitivity, and document parameter space for reproducibility.

## Prerequisites

**Required**:
- Functional redistricting pipeline
- Census data and adjacency graphs
- Python 3.13+ with pandas, numpy, matplotlib, seaborn

**Recommended**:
- Small/medium test states (for faster iteration)
- Clear parameter ranges based on theory/prior work
- Baseline results for comparison

## When to Use This Skill

- User says: "Find optimal parameters" or "Tune the algorithm"
- User says: "How sensitive is compactness to edge weight scaling?"
- User wants to understand parameter impact
- User needs to justify parameter choices in paper
- User wants to test robustness across parameter ranges
- User is implementing new feature and needs parameter selection

## Parameters to Sweep

### 1. Population Tolerance (ufactor)

**What it controls**: Maximum allowed population imbalance

**Range**: 1.001 (0.1%) to 1.05 (5%)

**Typical values**:
- 1.001 (0.1%): Very strict, may fail for some states
- 1.003 (0.3%): Strict, legal standard in some jurisdictions
- 1.005 (0.5%): Project default, balances tightness and feasibility
- 1.01 (1%): Loose, more flexibility for compactness
- 1.05 (5%): Very loose, used for testing

**Expected relationship**:
- Lower ufactor → Stricter balance, potentially less compact
- Higher ufactor → Looser balance, potentially more compact

**Test sweep**:
```bash
for ufactor in 1.001 1.003 1.005 1.01 1.02; do
  python scripts/pipeline/run_complete_redistricting.py \
    --year 2020 --version sweep_ufactor_${ufactor} \
    --ufactor ${ufactor} --states "AL,CO,MN"
done
```

### 2. Edge Weight Scaling Factor

**What it controls**: How much to emphasize boundary length

**Range**: 0.1x to 10x of actual boundary lengths

**Typical values**:
- 0.1x: Minimal emphasis on boundary length
- 0.5x: Reduced emphasis
- 1.0x: Project default (actual boundary lengths in meters)
- 2.0x: Increased emphasis
- 5.0x: Strong emphasis
- 10.0x: Very strong emphasis (may over-constrain)

**Expected relationship**:
- Lower scale → Less compact, more population-balanced
- Higher scale → More compact, possibly less balanced

**Implementation note**: Requires code modification to expose scaling parameter

### 3. METIS Algorithm Parameters

**niter**: Number of refinement iterations
- Range: 1 to 50
- Default: 10
- Higher → Better quality, longer runtime

**ufactor**: Already covered above

**seed**: Random seed for reproducibility
- Range: Any integer
- Test multiple seeds to assess variance

### 4. Minimum Tract Population Threshold

**What it controls**: Whether to aggregate small tracts

**Range**: 0 (no aggregation) to 5000 people

**Typical values**:
- 0: No aggregation (current default)
- 100: Aggregate tracts with <100 people
- 500: Aggregate small tracts
- 1000: Moderate aggregation

**Expected relationship**:
- Higher threshold → Fewer nodes, faster runtime, potentially less accurate

## Workflow

### Step 1: Define Parameter Space

**Select parameter to sweep**:
- Primary: ufactor (easiest to test)
- Secondary: Edge weight scaling (requires code changes)
- Advanced: METIS iterations, random seed

**Define range**:
- Minimum value
- Maximum value
- Step size or number of points

**Example: ufactor sweep**
```python
import numpy as np

# Linear spacing
ufactors = [1.001, 1.003, 1.005, 1.01, 1.02, 1.05]

# Or logarithmic spacing
ufactors = [1 + x for x in np.logspace(-3, -1, 6)]  # 0.1% to 10%
```

### Step 2: Select Test States

**Recommended**: 3-5 diverse states for quick iteration

**Considerations**:
- **Small states** (VT, DE, WY): Fast, test feasibility limits
- **Medium states** (AL, CO, MN): Balanced testing
- **Large states** (CA, TX, FL): Realism, but slow

**Quick sweep** (5-10 min per parameter value):
```
AL, CO, MN
```

**Full sweep** (20-30 min per parameter value):
```
VT, DE, WY, AL, GA, CO, AZ, MN, MO, VA
```

### Step 3: Run Sweep

**Automated loop**:
```bash
#!/bin/bash
# sweep_ufactor.sh

STATES="alabama,colorado,minnesota"
YEAR=2020

for ufactor in 1.001 1.003 1.005 1.01 1.02; do
  echo "Testing ufactor = ${ufactor}"

  python scripts/pipeline/run_complete_redistricting.py \
    --year ${YEAR} \
    --version sweep_ufactor_$(echo ${ufactor} | sed 's/\./_/g') \
    --ufactor ${ufactor} \
    --states "${STATES}"

  if [ $? -ne 0 ]; then
    echo "[ERROR] ufactor = ${ufactor} failed"
  else
    echo "[OK] ufactor = ${ufactor} complete"
  fi

  echo ""
done

echo "Sweep complete!"
```

**Run**:
```bash
chmod +x sweep_ufactor.sh
./sweep_ufactor.sh
```

### Step 4: Collect Metrics

**Extract results for each parameter value**:
```python
import pandas as pd
from pathlib import Path

states = ['alabama', 'colorado', 'minnesota']
ufactors = [1.001, 1.003, 1.005, 1.01, 1.02]

results = []

for ufactor in ufactors:
    version = f"sweep_ufactor_{str(ufactor).replace('.', '_')}"

    for state in states:
        # Load compactness
        compactness_file = f'outputs/us_2020_{version}/states/{state}/data/compactness.csv'
        if Path(compactness_file).exists():
            df = pd.read_csv(compactness_file)
            mean_pp = df['polsby_popper'].mean()
            mean_reock = df['reock'].mean()
        else:
            mean_pp, mean_reock = None, None

        # Load population deviation
        summary_file = f'outputs/us_2020_{version}/states/{state}/data/district_summary.csv'
        if Path(summary_file).exists():
            df_sum = pd.read_csv(summary_file)
            max_dev = df_sum['population_deviation'].abs().max()
        else:
            max_dev = None

        results.append({
            'ufactor': ufactor,
            'state': state,
            'mean_pp': mean_pp,
            'mean_reock': mean_reock,
            'max_pop_dev': max_dev
        })

# Convert to DataFrame
results_df = pd.DataFrame(results)
results_df.to_csv('sweep_results_ufactor.csv', index=False)
print(results_df)
```

### Step 5: Analyze Trends

**Aggregate across states**:
```python
# Mean metrics per ufactor value
summary = results_df.groupby('ufactor').agg({
    'mean_pp': ['mean', 'std'],
    'mean_reock': ['mean', 'std'],
    'max_pop_dev': ['mean', 'std']
})

print("\nSummary by ufactor:")
print(summary)
```

**Identify optimal value**:
```python
# Find ufactor that maximizes compactness while meeting population constraint
target_max_dev = 0.005  # 0.5% target

valid = summary[summary['max_pop_dev']['mean'] <= target_max_dev]
optimal_ufactor = valid['mean_pp']['mean'].idxmax()

print(f"\nOptimal ufactor: {optimal_ufactor}")
print(f"  Mean PP: {valid.loc[optimal_ufactor, ('mean_pp', 'mean')]:.4f}")
print(f"  Max dev: {valid.loc[optimal_ufactor, ('max_pop_dev', 'mean')]:.4f}")
```

### Step 6: Visualize Sensitivity

**Line plot: Metric vs Parameter**:
```python
import matplotlib.pyplot as plt
import seaborn as sns

sns.set_style('whitegrid')

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

# Plot 1: Compactness vs ufactor
summary_mean = results_df.groupby('ufactor')['mean_pp'].mean()
summary_std = results_df.groupby('ufactor')['mean_pp'].std()

ax1.errorbar(summary_mean.index, summary_mean.values,
             yerr=summary_std.values, marker='o', capsize=5)
ax1.set_xlabel('Population Tolerance (ufactor)')
ax1.set_ylabel('Mean Polsby-Popper Score')
ax1.set_title('Compactness vs Population Tolerance')
ax1.grid(True, alpha=0.3)

# Plot 2: Population deviation vs ufactor
dev_mean = results_df.groupby('ufactor')['max_pop_dev'].mean()
dev_std = results_df.groupby('ufactor')['max_pop_dev'].std()

ax2.errorbar(dev_mean.index, dev_mean.values,
             yerr=dev_std.values, marker='s', capsize=5, color='red')
ax2.axhline(0.005, color='black', linestyle='--', label='0.5% target')
ax2.set_xlabel('Population Tolerance (ufactor)')
ax2.set_ylabel('Max Population Deviation')
ax2.set_title('Population Balance vs Tolerance')
ax2.legend()
ax2.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('parameter_sweep_ufactor.png', dpi=150)
print("Saved: parameter_sweep_ufactor.png")
```

**Heatmap: Parameter x State**:
```python
# Pivot for heatmap
heatmap_data = results_df.pivot(index='state', columns='ufactor', values='mean_pp')

plt.figure(figsize=(10, 6))
sns.heatmap(heatmap_data, annot=True, fmt='.3f', cmap='YlGnBu', cbar_kws={'label': 'Mean PP'})
plt.title('Compactness Sensitivity: ufactor × State')
plt.xlabel('Population Tolerance (ufactor)')
plt.ylabel('State')
plt.tight_layout()
plt.savefig('heatmap_ufactor_state.png', dpi=150)
```

### Step 7: Document Findings

**Report template**:
```markdown
# Parameter Sweep: Population Tolerance (ufactor)

## Methodology
- Parameter: ufactor (population imbalance tolerance)
- Range: [1.001, 1.003, 1.005, 1.01, 1.02]
- States: Alabama, Colorado, Minnesota (N=3)
- Census year: 2020
- Mode: Edge-weighted
- Metrics: Mean Polsby-Popper, Max population deviation

## Results

| ufactor | Mean PP | Std | Max Pop Dev | Feasible? |
|---------|---------|-----|-------------|-----------|
| 1.001   | 0.4234  | 0.052 | 0.0009    | Yes       |
| 1.003   | 0.4456  | 0.048 | 0.0028    | Yes       |
| 1.005   | 0.4589  | 0.045 | 0.0047    | Yes       |
| 1.01    | 0.4723  | 0.041 | 0.0093    | No (>0.5%) |
| 1.02    | 0.4801  | 0.039 | 0.0187    | No (>0.5%) |

## Findings
- Compactness improves with looser tolerance (as expected)
- ufactor = 1.005 is optimal: Maximizes compactness while meeting 0.5% target
- Further loosening (1.01+) violates population balance constraint
- Std decreases with higher ufactor (more consistent results)

## Recommendation
**Use ufactor = 1.005** (0.5% tolerance) as project default.

Justification:
- Achieves mean PP = 0.4589 (15% higher than 1.001)
- Maintains max deviation < 0.5% (legal/ethical standard)
- Provides algorithm flexibility without sacrificing balance
```

## Common Sweep Scenarios

### Sweep 1: ufactor (Population Tolerance)

**Goal**: Find best balance between compactness and population equality

**Range**: 1.001 to 1.05
**Points**: 5-7 values
**States**: 3-5 medium states
**Runtime**: ~30-60 minutes total

### Sweep 2: Random Seed (Variance Assessment)

**Goal**: Quantify algorithm variability

**Range**: 10 different random seeds
**Points**: 10 seeds
**States**: 3-5 states
**Runtime**: ~1-2 hours total

**Analysis**:
- Compute coefficient of variation (CV) for each state
- If CV < 5%, algorithm is robust
- If CV > 10%, consider averaging multiple runs

### Sweep 3: Edge Weight Scaling (Custom)

**Goal**: Optimize compactness improvement from edge weights

**Range**: 0.1x to 10x
**Points**: 6 values [0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
**States**: 3-5 states
**Runtime**: ~1-2 hours total

**Note**: Requires modifying code to expose scaling parameter

### Sweep 4: METIS Iterations (niter)

**Goal**: Trade-off between quality and speed

**Range**: 1 to 50 iterations
**Points**: [1, 5, 10, 20, 50]
**States**: 2-3 small/medium states
**Runtime**: ~30-60 minutes total

## Troubleshooting

### Pipeline Fails for Some Parameter Values

**Symptom**: Some ufactor values produce errors

**Common causes**:
- Too strict (1.001): METIS cannot satisfy balance
- Too loose (1.10): Disconnected districts possible

**Solutions**:
- Use `--force` flag to continue on errors
- Log which values failed: Track in results_df
- Report feasible parameter range in paper

### High Variance Across States

**Symptom**: Std error bars are large

**Causes**:
- States have different complexities (geography, district counts)
- Parameter has differential impact

**Solutions**:
- Increase N (more states)
- Stratify by state characteristics (size, district count)
- Report per-state trends separately

### No Clear Optimal Value

**Symptom**: Multiple parameter values seem equivalent

**Causes**:
- Parameter has minimal impact
- Differences within measurement noise

**Solutions**:
- Widen parameter range
- Test more extreme values
- Consider multi-objective optimization (Pareto front)

### Runtime Too Long

**Symptom**: Full sweep takes many hours

**Solutions**:
- Reduce states to 2-3
- Use small states only (VT, DE, WY)
- Coarser parameter grid (fewer points)
- Run overnight

## Advanced Techniques

### Multi-Dimensional Sweep

**Test multiple parameters simultaneously**:

```python
# Grid search
ufactors = [1.003, 1.005, 1.01]
niter_values = [5, 10, 20]

for ufactor in ufactors:
    for niter in niter_values:
        version = f"sweep_uf{ufactor}_niter{niter}"
        # Run pipeline...
```

**Warning**: Exponential growth in runtime (N × M combinations)

### Pareto Front Analysis

**For multi-objective optimization** (e.g., compactness vs balance):

```python
import matplotlib.pyplot as plt

# Plot compactness vs deviation
plt.scatter(results_df['max_pop_dev'], results_df['mean_pp'])
plt.xlabel('Max Population Deviation')
plt.ylabel('Mean Polsby-Popper')
plt.title('Pareto Front: Compactness vs Balance')

# Identify Pareto optimal points
# (maximize compactness, minimize deviation)
```

### Sensitivity Analysis

**Compute partial derivatives**:

```python
# Estimate d(PP)/d(ufactor)
ufactor_vals = results_df['ufactor'].unique()
pp_means = [results_df[results_df['ufactor'] == u]['mean_pp'].mean() for u in ufactor_vals]

# Numerical gradient
sensitivity = np.gradient(pp_means, ufactor_vals)

print("Sensitivity (dPP/dufactor):")
for u, s in zip(ufactor_vals, sensitivity):
    print(f"  ufactor = {u}: {s:.2f}")
```

### Bayesian Optimization

**For expensive parameter searches**:

Use libraries like `scikit-optimize` to intelligently sample parameter space:
```python
from skopt import gp_minimize

def objective(params):
    ufactor, = params
    # Run pipeline, return -mean_pp (minimize negative = maximize positive)
    return -mean_pp

# Find optimal with fewer evaluations
result = gp_minimize(objective, [(1.001, 1.05)], n_calls=10)
optimal_ufactor = result.x[0]
```

## Documentation for Papers

### Methods Section

```
Parameter Selection:

We performed a systematic parameter sweep to determine the optimal
population tolerance (ufactor). Values ranged from 1.001 (0.1%) to
1.02 (2.0%) in logarithmic steps. For each value, we ran the
redistricting algorithm on 10 diverse states and measured mean
Polsby-Popper compactness and maximum population deviation.

We selected ufactor = 1.005 as the project default based on the
criterion of maximizing compactness while maintaining all districts
within 0.5% of target population (the commonly accepted legal
standard). This value achieved a mean PP of 0.459 ± 0.045 across
test states, a 15% improvement over the strictest tolerance (1.001)
while satisfying the population constraint.
```

### Results Section

```
Parameter Sensitivity Analysis:

Figure X shows the relationship between population tolerance and
compactness. Compactness increased monotonically with tolerance
(ρ = 0.98, p < 0.001), but values above 1.005 violated the 0.5%
population balance constraint. The selected value (ufactor = 1.005)
represents the optimal trade-off point.

Sensitivity analysis revealed that a 0.1% increase in tolerance
yielded approximately 0.035 increase in mean PP (∂PP/∂ufactor ≈ 7.0).
This indicates the algorithm is moderately sensitive to this parameter,
justifying careful selection.
```

## Related Skills

- `/run-experiment` - Compare discrete algorithm variants
- `/run-redistricting` - Execute pipeline for each parameter value
- `/run-statistical-analysis` - Compute detailed metrics
- `/validate-compactness` - Verify improvements are real

## Performance

**Typical sweep runtimes**:

| States | Parameters | Runtime per Point | Total Runtime |
|--------|-----------|------------------|---------------|
| 3 small | 5 values | ~5 min | ~25 min |
| 5 mixed | 5 values | ~15 min | ~75 min (1.25 hrs) |
| 10 mixed | 5 values | ~30 min | ~150 min (2.5 hrs) |
| 3 small | 10 values | ~5 min | ~50 min |

**Optimization**:
- Use smallest feasible state set
- Reduce DPI for maps (faster rendering)
- Use `--skip-analysis` if only testing redistricting
- Run parameter values in parallel (separate terminals)

## Next Steps

After parameter sweep:
- Select optimal parameter values
- Document justification in paper methods
- Re-run full 50-state pipeline with optimal parameters
- Include sensitivity analysis figures in paper
- Update project defaults in config files
