"""
FINAL HEAD-TO-HEAD COMPARISON
N-way vs Recursive Bisection - Complete 50-state results
"""

import pandas as pd
import numpy as np
from pathlib import Path

# Load both complete datasets
results_dir = Path(__file__).parent / "results"
nway_df = pd.read_csv(results_dir / "50_states_ablation_test.csv")
recursive_df = pd.read_csv(results_dir / "50_states_recursive_ablation.csv")

print("=" * 80)
print("FINAL N-WAY VS RECURSIVE BISECTION COMPARISON")
print("Complete 50-State Ablation Study Results")
print("=" * 80)
print()

# Filter to tested states
nway_tested = nway_df[~nway_df['skipped']].copy()
recursive_tested = recursive_df[~recursive_df['skipped']].copy()

print("DATASET SUMMARY")
print("-" * 80)
print(f"{'Method':<20} | {'Total Configs':>12} | {'States':>6} | {'Success Rate':>13}")
print("-" * 80)
print(f"{'N-way':<20} | {len(nway_tested):>12} | {nway_tested['state'].nunique():>6} | "
      f"{nway_tested['success'].mean()*100:>12.1f}%")
print(f"{'Recursive':<20} | {len(recursive_tested):>12} | {recursive_tested['state'].nunique():>6} | "
      f"{recursive_tested['success'].mean()*100:>12.1f}%")
print()

# Runtime comparison
print("RUNTIME COMPARISON")
print("-" * 80)
print(f"{'Method':<20} | {'Mean':>8} | {'Median':>8} | {'Min':>8} | {'Max':>8} | {'Total Hrs':>10}")
print("-" * 80)
print(f"{'N-way':<20} | {nway_tested['runtime'].mean():>7.2f}s | "
      f"{nway_tested['runtime'].median():>7.2f}s | "
      f"{nway_tested['runtime'].min():>7.2f}s | "
      f"{nway_tested['runtime'].max():>7.2f}s | "
      f"{nway_tested['runtime'].sum()/3600:>9.2f}h")
print(f"{'Recursive':<20} | {recursive_tested['runtime'].mean():>7.2f}s | "
      f"{recursive_tested['runtime'].median():>7.2f}s | "
      f"{recursive_tested['runtime'].min():>7.2f}s | "
      f"{recursive_tested['runtime'].max():>7.2f}s | "
      f"{recursive_tested['runtime'].sum()/3600:>9.2f}h")
print()
speedup = (recursive_tested['runtime'].mean() - nway_tested['runtime'].mean()) / recursive_tested['runtime'].mean() * 100
print(f"N-way is {speedup:.1f}% faster than recursive")
print()

# Parameter sensitivity for both methods
print("=" * 80)
print("PARAMETER SENSITIVITY")
print("=" * 80)
print()

print("WEIGHT FACTOR PERFORMANCE:")
print("-" * 80)
print(f"{'Weight':<10} | {'N-way Success':>14} | {'Recursive Success':>18} | {'Winner':>10}")
print("-" * 80)
for wf in sorted(nway_tested['weight_factor'].unique()):
    nway_perf = nway_tested[nway_tested['weight_factor'] == wf]['success'].mean() * 100
    rec_perf = recursive_tested[recursive_tested['weight_factor'] == wf]['success'].mean() * 100
    winner = "N-way" if nway_perf > rec_perf else "Recursive" if rec_perf > nway_perf else "Tie"
    print(f"{wf:<10} | {nway_perf:>13.1f}% | {rec_perf:>17.1f}% | {winner:>10}")
print()

print("MINORITY THRESHOLD PERFORMANCE:")
print("-" * 80)
print(f"{'Threshold':<10} | {'N-way Success':>14} | {'Recursive Success':>18} | {'Winner':>10}")
print("-" * 80)
for mt in sorted(nway_tested['minority_threshold'].unique()):
    nway_perf = nway_tested[nway_tested['minority_threshold'] == mt]['success'].mean() * 100
    rec_perf = recursive_tested[recursive_tested['minority_threshold'] == mt]['success'].mean() * 100
    winner = "N-way" if nway_perf > rec_perf else "Recursive" if rec_perf > nway_perf else "Tie"
    print(f"{mt:<10.2f} | {nway_perf:>13.1f}% | {rec_perf:>17.1f}% | {winner:>10}")
print()

# State-by-state comparison
print("=" * 80)
print("STATE-BY-STATE COMPARISON")
print("=" * 80)
print()

nway_states = nway_tested.groupby('state').agg({
    'success': 'mean',
    'k': 'first',
    'target_mm': 'first',
    'state_minority_pct': 'first',
    'runtime': 'mean'
}).round(4)
nway_states['success_pct'] = nway_states['success'] * 100

recursive_states = recursive_tested.groupby('state').agg({
    'success': 'mean',
    'k': 'first',
    'target_mm': 'first',
    'state_minority_pct': 'first',
    'runtime': 'mean'
}).round(4)
recursive_states['success_pct'] = recursive_states['success'] * 100

# Merge for comparison
comparison = nway_states.join(recursive_states, lsuffix='_nway', rsuffix='_rec', how='outer')

# Count wins
nway_wins = (comparison['success_pct_nway'] > comparison['success_pct_rec']).sum()
rec_wins = (comparison['success_pct_rec'] > comparison['success_pct_nway']).sum()
ties = (comparison['success_pct_nway'] == comparison['success_pct_rec']).sum()

print(f"Overall winner by state:")
print(f"  N-way wins: {nway_wins} states")
print(f"  Recursive wins: {rec_wins} states")
print(f"  Ties: {ties} states")
print()

# Show biggest differences
comparison['difference'] = comparison['success_pct_nway'] - comparison['success_pct_rec']
comparison_sorted = comparison.sort_values('difference', ascending=False)

print("Top 10 states where N-way beats Recursive (by largest margin):")
print("-" * 80)
print(f"{'State':<20} | {'N-way':>10} | {'Recursive':>10} | {'Difference':>11}")
print("-" * 80)
for state, row in comparison_sorted.head(10).iterrows():
    print(f"{state:<20} | {row['success_pct_nway']:>9.1f}% | {row['success_pct_rec']:>9.1f}% | "
          f"{row['difference']:>+10.1f}%")
print()

print("Top 10 states where Recursive beats N-way (by largest margin):")
print("-" * 80)
print(f"{'State':<20} | {'N-way':>10} | {'Recursive':>10} | {'Difference':>11}")
print("-" * 80)
for state, row in comparison_sorted.tail(10).iterrows():
    print(f"{state:<20} | {row['success_pct_nway']:>9.1f}% | {row['success_pct_rec']:>9.1f}% | "
          f"{row['difference']:>+10.1f}%")
print()

# Success tier analysis
print("=" * 80)
print("SUCCESS TIER ANALYSIS")
print("=" * 80)
print()

nway_perfect = (nway_states['success_pct'] == 100).sum()
nway_high = ((nway_states['success_pct'] >= 50) & (nway_states['success_pct'] < 100)).sum()
nway_medium = ((nway_states['success_pct'] > 0) & (nway_states['success_pct'] < 50)).sum()
nway_failed = (nway_states['success_pct'] == 0).sum()

rec_perfect = (recursive_states['success_pct'] == 100).sum()
rec_high = ((recursive_states['success_pct'] >= 50) & (recursive_states['success_pct'] < 100)).sum()
rec_medium = ((recursive_states['success_pct'] > 0) & (recursive_states['success_pct'] < 50)).sum()
rec_failed = (recursive_states['success_pct'] == 0).sum()

print(f"{'Success Tier':<30} | {'N-way':>10} | {'Recursive':>10}")
print("-" * 60)
print(f"{'100% (always succeeds)':<30} | {nway_perfect:>10} | {rec_perfect:>10}")
print(f"{'50-99% (usually succeeds)':<30} | {nway_high:>10} | {rec_high:>10}")
print(f"{'1-49% (rarely succeeds)':<30} | {nway_medium:>10} | {rec_medium:>10}")
print(f"{'0% (never succeeds)':<30} | {nway_failed:>10} | {rec_failed:>10}")
print()

# Statistical summary
print("=" * 80)
print("STATISTICAL SUMMARY")
print("=" * 80)
print()

from scipy import stats

# Paired t-test on state-level success rates
common_states = set(nway_states.index) & set(recursive_states.index)
nway_rates = [nway_states.loc[s, 'success_pct'] for s in common_states]
rec_rates = [recursive_states.loc[s, 'success_pct'] for s in common_states]

t_stat, p_value = stats.ttest_rel(nway_rates, rec_rates)

print(f"Paired t-test (state-level success rates):")
print(f"  t-statistic: {t_stat:.4f}")
print(f"  p-value: {p_value:.6f}")
if p_value < 0.05:
    print(f"  Result: Statistically significant difference (p < 0.05)")
else:
    print(f"  Result: No significant difference (p >= 0.05)")
print()

# Effect size (Cohen's d)
mean_diff = np.mean(nway_rates) - np.mean(rec_rates)
pooled_std = np.sqrt((np.std(nway_rates)**2 + np.std(rec_rates)**2) / 2)
cohens_d = mean_diff / pooled_std

print(f"Effect size (Cohen's d): {cohens_d:.4f}")
if abs(cohens_d) < 0.2:
    effect = "negligible"
elif abs(cohens_d) < 0.5:
    effect = "small"
elif abs(cohens_d) < 0.8:
    effect = "medium"
else:
    effect = "large"
print(f"  Interpretation: {effect} effect")
print()

# Best configurations
print("=" * 80)
print("BEST PARAMETER COMBINATIONS")
print("=" * 80)
print()

nway_best = nway_tested.groupby(['weight_factor', 'minority_threshold'])['success'].agg(['sum', 'count', 'mean'])
nway_best['rate'] = nway_best['mean'] * 100
nway_best = nway_best.sort_values('rate', ascending=False)

rec_best = recursive_tested.groupby(['weight_factor', 'minority_threshold'])['success'].agg(['sum', 'count', 'mean'])
rec_best['rate'] = rec_best['mean'] * 100
rec_best = rec_best.sort_values('rate', ascending=False)

print("Top 5 N-way configurations:")
print("-" * 80)
for i, ((wf, mt), row) in enumerate(nway_best.head(5).iterrows(), 1):
    print(f"  {i}. Weight={wf:>3}x, Threshold={mt:.2f}: {row['rate']:>5.1f}% ({int(row['sum'])}/{int(row['count'])})")

print()

print("Top 5 Recursive configurations:")
print("-" * 80)
for i, ((wf, mt), row) in enumerate(rec_best.head(5).iterrows(), 1):
    print(f"  {i}. Weight={wf:>3}x, Threshold={mt:.2f}: {row['rate']:>5.1f}% ({int(row['sum'])}/{int(row['count'])})")

print()

# Final verdict
print("=" * 80)
print("FINAL VERDICT")
print("=" * 80)
print()

print("OVERALL WINNER: ", end="")
if nway_tested['success'].mean() > recursive_tested['success'].mean():
    print("N-WAY PARTITIONING")
    margin = (nway_tested['success'].mean() - recursive_tested['success'].mean()) * 100
    print(f"  Margin: {margin:.1f} percentage points")
else:
    print("RECURSIVE BISECTION")
    margin = (recursive_tested['success'].mean() - nway_tested['success'].mean()) * 100
    print(f"  Margin: {margin:.1f} percentage points")

print()
print(f"State-by-state: N-way wins {nway_wins}/{len(common_states)} states ({100*nway_wins/len(common_states):.1f}%)")
print(f"Speed advantage: N-way is {speedup:.1f}% faster")
print(f"Statistical significance: p = {p_value:.6f}")
print()

print("=" * 80)
