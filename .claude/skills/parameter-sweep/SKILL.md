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

Systematically test algorithm across parameter value ranges. Identify optimal settings, understand parameter sensitivity, document parameter space for reproducibility.

## Prerequisites
**Required**: Functional redistricting pipeline, census data + adjacency graphs, Python 3.13+ with pandas, numpy, matplotlib, seaborn
**Recommended**: Small/medium test states (faster iteration), clear parameter ranges (theory/prior work), baseline results for comparison

## When to Use
User says "Find optimal parameters/Tune algorithm/How sensitive is compactness to edge weight scaling", understand parameter impact, justify parameter choices in paper, test robustness across parameter ranges, implementing new feature needing parameter selection

## Parameters to Sweep

**1. Population Tolerance (ufactor)**: Controls max allowed population imbalance, range 1.001 (0.1%) to 1.05 (5%)
- 1.001 (0.1%): Very strict, may fail some states
- 1.003 (0.3%): Strict, legal standard some jurisdictions
- 1.005 (0.5%): Project default, balances tightness + feasibility
- 1.01 (1%): Loose, more flexibility for compactness
- 1.05 (5%): Very loose, testing
- **Expected**: Lower ufactor → stricter balance, potentially less compact | Higher → looser balance, potentially more compact
```bash
for ufactor in 1.001 1.003 1.005 1.01 1.02; do
  python scripts/pipeline/run_complete_redistricting.py --year 2020 --version sweep_ufactor_${ufactor} --ufactor ${ufactor} --states "AL"
done
```

**2. Edge Weight Scaling**: Multiplier for boundary lengths, range 1.0 (no scaling) to 100.0 (very strong)
- 1.0: No scaling (unweighted mode effectively)
- 10.0: Moderate emphasis on shared boundaries
- 50.0: Strong emphasis (default)
- 100.0: Very strong, may overweight compactness vs balance
- **Expected**: Higher scaling → more compact districts, diminishing returns >50
```bash
for scale in 1 10 25 50 75 100; do
  python scripts/pipeline/run_complete_redistricting.py --year 2020 --version sweep_scale_${scale} --edge-weight-scale ${scale} --states "AL"
done
```

**3. Minimum Tract Population**: Threshold for excluding low-pop tracts, range 0 (include all) to 1000 (exclude small)
- 0: Include all tracts (may create tiny districts)
- 50: Exclude uninhabited/nearly uninhabited
- 100: Standard threshold (default)
- 500: Exclude rural/sparse areas
- **Expected**: Higher threshold → fewer tracts, potentially more compact but may exclude valid areas

**4. METIS Recursive Bisection Depth**: Max depth for recursive bisection, affects algorithm termination
**5. Random Seed**: For reproducibility testing (check if results consistent across seeds)

## Workflow

### Step 1: Define Parameter Space
**Select parameters**: Which to test (1-3 parameters typical)
**Define ranges**: Min/max values + step size (log-scale for large ranges)
**Sample points**: Grid search (all combinations) or random sampling (large spaces)
**Example grid**: ufactor [1.001, 1.003, 1.005, 1.01] × edge_scale [10, 25, 50, 75] = 16 combinations

### Step 2: Select Test States
**Small states** (AL, VT, DE): Fast iteration (~30s each)
**Medium states** (GA, NC, AZ): Balance speed + representativeness
**Large states** (CA, TX): Use sparingly (expensive)
**Recommendation**: 2-3 small states for initial sweep, 1-2 medium for validation

### Step 3: Run Parameter Sweep
Use TodoWrite: Track each combination, mark in_progress/completed
```bash
# Grid search example
for ufactor in 1.001 1.005 1.01; do
  for edge_scale in 10 50 100; do
    python scripts/pipeline/run_complete_redistricting.py \
      --year 2020 --version sweep_u${ufactor}_e${edge_scale} \
      --ufactor ${ufactor} --edge-weight-scale ${edge_scale} \
      --states "AL,VT"
  done
done
```
**Monitor**: Runtime per combination, check for errors/failures, verify outputs created

### Step 4: Extract Metrics
**Collect**: Read `compactness.csv`, `district_summary.csv` for each combination
**Aggregate**: Mean PP, mean Reock, max pop deviation, runtime, METIS iterations
**Format**: DataFrame (rows=combinations, cols=params + metrics)
```python
results = pd.DataFrame({
  'ufactor': [1.001, 1.005, ...],
  'edge_scale': [10, 50, ...],
  'mean_pp': [0.38, 0.42, ...],
  'max_deviation': [0.3, 0.4, ...],
  'runtime_sec': [45, 52, ...]
})
```

### Step 5: Analyze Parameter Effects
**Univariate**: Plot each parameter vs metrics (line plots, one param at a time)
**Interactions**: 2D heatmaps (two params, metric as color)
**Correlations**: Pearson correlation matrix (params vs metrics)
**Optimal point**: Identify combination maximizing objective (e.g., max PP while max_deviation <0.5%)

### Step 6: Visualize Parameter Space
**Line plots**: X=parameter value, Y=metric, multiple lines for different states/conditions
**Heatmaps**: X=param1, Y=param2, color=metric (2D parameter space)
**Contour plots**: Contours of equal metric value (shows optimal region)
**Scatter plots**: Parameter vs metric with trend line

### Step 7: Identify Optimal Parameters
**Objective function**: Define what "optimal" means (max PP? min runtime? balance both?)
**Constraints**: Max pop deviation <0.5%, runtime <X minutes
**Pareto frontier**: Trade-offs (compactness vs runtime, balance vs compactness)
**Recommendation**: Document optimal values + rationale

## Output Files

`parameter_sweeps/{sweep_name}/design.txt` (parameter ranges + states), `parameter_sweeps/{sweep_name}/results.csv` (all combinations + metrics), `parameter_sweeps/{sweep_name}/plots/param_vs_metric.png` (univariate plots), `parameter_sweeps/{sweep_name}/plots/heatmap_{param1}_{param2}.png` (interactions), `parameter_sweeps/{sweep_name}/optimal_params.txt` (recommended values + justification)

## Analysis Methods

**Sensitivity Analysis**: Measure how much metric changes per unit parameter change, compute partial derivatives (∂PP/∂ufactor), rank parameters by sensitivity (which has largest impact)
**Robustness Testing**: Check if optimal values stable across states, test on validation set (different states), ensure not overfitting to test states
**Trade-off Analysis**: Pareto frontier (compactness vs balance, compactness vs runtime), identify knee point (diminishing returns), document trade-offs for user choice

## Example Sweep

**Goal**: Find optimal ufactor + edge_scale
**Parameters**: ufactor [1.001, 1.003, 1.005, 1.01], edge_scale [10, 25, 50, 75, 100] → 20 combinations
**States**: AL, VT (fast testing)
**Metrics**: Mean PP, max pop deviation, runtime
**Results**:
- Optimal ufactor=1.005 (balances tightness + feasibility)
- Optimal edge_scale=50 (diminishing returns >50)
- Mean PP improves 0.30 → 0.42 (40% increase)
- Runtime increases 30s → 45s (acceptable)
**Conclusion**: Use ufactor=1.005, edge_scale=50 as defaults

## Performance

**Grid search cost**: n_params¹ × n_values¹ × ... × runtime_per_state × n_states
**Example**: 2 params × 5 values each × 40s/state × 2 states = 2×5×5×40×2 = 4,000s (~1.1 hours)
**Optimization**: Use small states for initial sweep, log-scale for large parameter ranges (fewer points needed), random sampling for >3 parameters (grid too expensive), parallelize runs (separate terminals/machines)

## Best Practices

Start small (1-2 params, 2-3 states), use domain knowledge for ranges (don't test implausible values), log-scale for orders of magnitude (e.g., edge_scale 1, 10, 100 not 1, 2, 3), validate on different states (check generalization), document assumptions (why these ranges?), report sensitivity (which params matter most), consider interactions (params may not be independent), archive all results (reproducibility)

## What You'll Get

Parameter space documentation (ranges + rationale), comprehensive results CSV (all combinations), sensitivity analysis (which params matter), optimal parameter values (justified recommendation), trade-off analysis (Pareto frontiers), visualization plots (line/heatmap/contour), validation results (different states), reproducible methodology (document for paper)

## Related Skills
`/run-experiment` (structured experiments with controls), `/run-redistricting` (execute parameter combinations), `/run-statistical-analysis` (analyze results), `/validate-compactness` (check optimal params produce valid results)

## Next Steps
Document optimal parameters in paper methods, include parameter sweep plots in supplementary materials, justify parameter choices with sensitivity analysis, test optimal params on all 50 states, consider user-configurable parameters (let users tune for their use case)
