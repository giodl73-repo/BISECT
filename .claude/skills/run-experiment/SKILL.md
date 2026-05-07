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

Design + execute controlled experiments to compare redistricting algorithm variants. Systematically test configurations, collect quantitative metrics, perform statistical analysis to validate improvements.

## Prerequisites
**Required**: Full pipeline functional, census data + adjacency graphs for target year/states, Python 3.13+ with pandas, numpy, scipy, matplotlib
**Recommended**: Multiple states (small/medium/large), baseline results available, clear hypothesis about expected improvements

## When to Use
User says "Compare edge-weighted vs unweighted/Test algorithm changes/Validate improvements", compare different METIS parameters, needs quantitative evidence for paper/presentation, test new feature against baseline, needs statistical significance testing

## Experiment Types

**1. Mode Comparison** (Edge-weighted vs Unweighted): Hypothesis edge-weighted produces more compact districts, Independent var (mode), Dependent vars (PP compactness, Reock, perimeter), Controlled (year, states, population tolerance)
```bash
# Baseline: Unweighted
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version exp_unweighted --mode unweighted --states "AL,CA,TX,FL,NY"
# Treatment: Edge-weighted
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version exp_weighted --mode weighted --states "AL,CA,TX,FL,NY"
```

**2. Parameter Sensitivity**: Hypothesis population tolerance affects compactness, Independent (ufactor), Dependent (compactness, max pop deviation), Controlled (mode, states, year)
```bash
for ufactor in 1.001 1.005 1.01 1.02; do
  python scripts/pipeline/run_complete_redistricting.py --year 2020 --version exp_ufactor_${ufactor} --ufactor ${ufactor} --states "AL,GA,NC"
done
```

**3. Algorithm Variant**: Hypothesis new algorithm improves compactness/speed, Independent (algorithm version), Dependent (compactness, runtime, memory), Controlled (data, parameters)

**4. State Selection**: Hypothesis large vs small states behave differently, Independent (state size/complexity), Dependent (compactness, runtime, convergence), Controlled (algorithm, parameters)

## Workflow

### Step 1: Define Experiment
**Research question**: What are you testing?
**Hypothesis**: Expected outcome (directional: weighted > unweighted by 15%)
**Variables**: Independent (what you change), dependent (what you measure), controlled (what stays same)
**Sample size**: States to test (5-10 minimum for statistical power)
**Significance level**: α=0.05 typical

### Step 2: Design Experiment
**States selection**: Small (AL, VT, DE), Medium (GA, NC, AZ), Large (CA, TX, FL) → Mix for generalizability
**Conditions**: Baseline (control), treatment (variant to test), block design if needed (group by state size)
**Metrics to collect**: Compactness (PP, Reock), population balance (mean/max deviation), runtime (seconds), memory (GB peak), convergence (METIS iterations)

### Step 3: Run Conditions
Use TodoWrite to track: Create experiment todo list, one todo per condition × state combo, mark in_progress/completed systematically
```bash
# Run all conditions systematically
for condition in control treatment; do
  for state in AL CA TX FL NY IL PA OH GA NC; do
    python scripts/pipeline/run_complete_redistricting.py --year 2020 --version exp_${condition} --states ${state} [parameters]
  done
done
```
**Monitor**: Progress bars, check for errors, verify outputs created, record any anomalies

### Step 4: Collect Metrics
**Extract data**: Read `district_summary.csv` for each state/condition, parse `compactness.csv`, measure runtime from logs, aggregate into experiment dataframe
**Format**: Columns (State, Condition, PP_mean, PP_median, Reock_mean, Runtime_sec, Max_deviation_pct), rows (one per state × condition)
**Save**: `experiments/exp_{name}/results.csv`

### Step 5: Perform Statistical Analysis
**Paired t-test** (same states across conditions): H₀ no difference, compute t-statistic + p-value, calculate effect size Cohen's d, determine 95% CI for mean difference
**Example**: Mean PP improvement 15.2% (t=12.4, p<0.001, d=0.89 large effect, CI [12.8%, 17.6%])
**Wilcoxon signed-rank** (if not normally distributed): Non-parametric alternative
**ANOVA** (>2 conditions): F-test for overall difference, post-hoc pairwise comparisons

### Step 6: Visualize Results
**Box plots**: Side-by-side comparison (condition on x-axis, metric on y-axis)
**Scatter plots**: Paired comparison (control on x, treatment on y, diagonal=no change)
**Bar charts**: Mean + error bars (condition on x, mean metric on y, 95% CI error bars)
**Tables**: Comparison table (State | Control | Treatment | Δ | % Improvement)

### Step 7: Interpret Results
**Statistical significance**: p<0.05 + large effect size (d>0.5) required
**Practical significance**: Mean improvement meaningful (>5% PP is substantial)
**Consistency**: All/most states improve (not just aggregate)
**Outliers**: Investigate states that don't improve (geographic reasons?)

## Metrics to Collect

**Compactness**: PP mean/median/std/min/max per state, Reock mean/median/std, comparison to baseline (% improvement)
**Population Balance**: Mean absolute deviation (%), max deviation (%), % districts within ±0.5%, target population (for reference)
**Performance**: Total runtime (seconds), per-state runtime, memory peak (GB), METIS iterations
**Political** (2020 only): Efficiency gap, partisan bias, seat distribution
**Convergence**: METIS edge-cut value, iterations to convergence, final objective

## Statistical Tests

**Paired t-test**: Use when same states tested across conditions, assumptions (normality-check with Q-Q plot, paired data), interpretation (p<0.05 = significant, report t-statistic + df)
**Effect Size Cohen's d**: Small (0.2), medium (0.5), large (0.8), compute `d = (mean_diff) / pooled_std`, report alongside p-value
**Confidence Intervals**: 95% CI for mean difference, interpret (if CI excludes 0 → significant), report range [lower, upper]
**Power Analysis**: Ensure enough samples (states) for statistical power, typically need n>5 for paired t-test, n>10 for robust results

## Example Experiment

**Question**: Does edge-weighted mode improve compactness?
**Hypothesis**: Edge-weighted mode increases mean PP by >10%
**Design**: 10 states (mixed sizes), 2 conditions (weighted vs unweighted), metrics (PP compactness, runtime)
**Run**: Execute both conditions for all states (~1 hour total)
**Collect**: Aggregate PP means per state/condition
**Analyze**: Paired t-test (t=8.3, p<0.001, d=0.95 large, mean improvement 15.2%, CI [11%, 19%])
**Conclude**: Edge-weighted significantly improves compactness (ACCEPT hypothesis)

## Output Files

`experiments/{exp_name}/design.txt` (experiment design doc), `experiments/{exp_name}/results.csv` (collected metrics), `experiments/{exp_name}/analysis.txt` (statistical tests), `experiments/{exp_name}/plots/*.png` (visualizations), `experiments/{exp_name}/comparison_table.tex` (LaTeX table for paper)

## Performance

| States | Conditions | Runtime |
|--------|-----------|---------|
| 5 small | 2 | ~10-15min |
| 10 mixed | 2 | ~30-45min |
| 20 mixed | 2 | ~2-3h |
| 50 all | 2 | ~6-8h |

**Bottlenecks**: Large states (CA/TX/FL 10-20min each), multiple conditions (linear scaling), full pipeline (adjacency + redistricting + analysis)
**Optimization**: Use `--skip-analysis` for redistricting-only tests, reuse adjacency across conditions, run parallel (separate terminals)

## Best Practices

Preregister hypothesis (document before running), use adequate sample size (n>5 minimum), control confounds (same data/parameters except variable of interest), randomize order (avoid systematic bias), validate assumptions (check normality for t-tests), report effect sizes (not just p-values), visualize distributions (not just means), document thoroughly (methods for reproducibility)

## What You'll Get

Experiment design document, collected metrics CSV (all states/conditions), statistical analysis (t-tests/effect sizes/CIs), comparison visualizations (box plots/scatter/tables), quantitative evidence (for papers/presentations), reproducible workflow (documented commands), validated conclusions (hypothesis accepted/rejected with evidence)

## Related Skills
`/run-redistricting` (execute conditions), `/run-statistical-analysis` (comprehensive stats), `/validate-compactness` (validate metrics), `/parameter-sweep` (systematic parameter testing)

## Next Steps
Document methodology in paper methods, include statistical tests in results, create comparison visualizations for figures, archive experiment outputs for reproducibility, share results in presentation/publication
