"""
Analyze 50-state ablation test results for n-way vs recursive bisection.
"""

import pandas as pd
import numpy as np
from pathlib import Path

# Load results
results_file = Path(__file__).parent / "results" / "50_states_ablation_test.csv"
df = pd.read_csv(results_file)

print("=" * 80)
print("50-STATE N-WAY ABLATION TEST ANALYSIS")
print("=" * 80)
print()

# Basic stats
print("DATASET OVERVIEW")
print("-" * 80)
print(f"Total rows: {len(df)}")
print(f"Unique states: {df['state'].nunique()}")
print(f"Skipped states: {df['skipped'].sum()} (single-district)")
print(f"Tested configurations: {len(df[~df['skipped']])}")
print()

# Filter to non-skipped states
df_tested = df[~df['skipped']].copy()
print(f"States tested: {df_tested['state'].nunique()}")
print(f"Total test configurations: {len(df_tested)}")
print()

# Success analysis
print("=" * 80)
print("SUCCESS RATE ANALYSIS")
print("=" * 80)
print()

overall_success = df_tested['success'].sum()
overall_total = len(df_tested)
print(f"Overall success rate: {overall_success}/{overall_total} ({100*overall_success/overall_total:.1f}%)")
print()

# By weight factor
print("Success by Weight Factor:")
print("-" * 80)
for wf in sorted(df_tested['weight_factor'].unique()):
    subset = df_tested[df_tested['weight_factor'] == wf]
    successes = subset['success'].sum()
    total = len(subset)
    print(f"  Weight {wf:>3}: {successes:>3}/{total:>3} ({100*successes/total:>5.1f}%) - Avg runtime: {subset['runtime'].mean():.2f}s")
print()

# By minority threshold
print("Success by Minority Threshold:")
print("-" * 80)
for mt in sorted(df_tested['minority_threshold'].unique()):
    subset = df_tested[df_tested['minority_threshold'] == mt]
    successes = subset['success'].sum()
    total = len(subset)
    print(f"  Threshold {mt:.2f}: {successes:>3}/{total:>3} ({100*successes/total:>5.1f}%)")
print()

# Combined parameters
print("Success by Combined Parameters:")
print("-" * 80)
print(f"{'Weight':>6} | {'Threshold':>9} | {'Success':>7} | {'Rate':>6} | {'Avg Runtime':>12}")
print("-" * 60)
for wf in sorted(df_tested['weight_factor'].unique()):
    for mt in sorted(df_tested['minority_threshold'].unique()):
        subset = df_tested[(df_tested['weight_factor'] == wf) &
                          (df_tested['minority_threshold'] == mt)]
        successes = subset['success'].sum()
        total = len(subset)
        rate = 100*successes/total if total > 0 else 0
        avg_runtime = subset['runtime'].mean()
        print(f"{wf:>6} | {mt:>9.2f} | {successes:>3}/{total:<3} | {rate:>5.1f}% | {avg_runtime:>11.2f}s")
print()

# State-level analysis
print("=" * 80)
print("STATE-LEVEL ANALYSIS")
print("=" * 80)
print()

state_success = df_tested.groupby('state').agg({
    'success': ['sum', 'count', lambda x: 100*x.sum()/len(x)],
    'runtime': 'mean',
    'k': 'first',
    'state_minority_pct': 'first',
    'target_mm': 'first'
}).round(2)

state_success.columns = ['successes', 'total', 'success_rate', 'avg_runtime', 'districts', 'minority_pct', 'target_mm']
state_success = state_success.sort_values('success_rate')

print("States with 0% success (never achieved target):")
print("-" * 80)
zero_success = state_success[state_success['success_rate'] == 0]
if len(zero_success) > 0:
    print(f"{'State':<20} | {'Districts':>9} | {'Target MM':>9} | {'Minority %':>11} | {'Tested':>6}")
    print("-" * 70)
    for state, row in zero_success.iterrows():
        print(f"{state:<20} | {row['districts']:>9.0f} | {row['target_mm']:>9.0f} | {row['minority_pct']:>10.1%} | {row['total']:>6.0f}")
else:
    print("  None! All states achieved target in at least one configuration.")
print()

print("States with 100% success (always achieved target):")
print("-" * 80)
full_success = state_success[state_success['success_rate'] == 100]
if len(full_success) > 0:
    print(f"{'State':<20} | {'Districts':>9} | {'Target MM':>9} | {'Minority %':>11} | {'Tested':>6}")
    print("-" * 70)
    for state, row in full_success.iterrows():
        print(f"{state:<20} | {row['districts']:>9.0f} | {row['target_mm']:>9.0f} | {row['minority_pct']:>10.1%} | {row['total']:>6.0f}")
else:
    print("  None.")
print()

print("States with partial success (50-99%):")
print("-" * 80)
partial_success = state_success[(state_success['success_rate'] >= 50) &
                                (state_success['success_rate'] < 100)]
if len(partial_success) > 0:
    print(f"{'State':<20} | {'Districts':>9} | {'Target MM':>9} | {'Minority %':>11} | {'Success Rate':>13} | {'Tested':>6}")
    print("-" * 90)
    for state, row in partial_success.iterrows():
        print(f"{state:<20} | {row['districts']:>9.0f} | {row['target_mm']:>9.0f} | {row['minority_pct']:>10.1%} | {row['success_rate']:>12.1f}% | {row['total']:>6.0f}")
else:
    print("  None.")
print()

print("States with low success (1-49%):")
print("-" * 80)
low_success = state_success[(state_success['success_rate'] > 0) &
                            (state_success['success_rate'] < 50)]
if len(low_success) > 0:
    print(f"{'State':<20} | {'Districts':>9} | {'Target MM':>9} | {'Minority %':>11} | {'Success Rate':>13} | {'Tested':>6}")
    print("-" * 90)
    for state, row in low_success.iterrows():
        print(f"{state:<20} | {row['districts']:>9.0f} | {row['target_mm']:>9.0f} | {row['minority_pct']:>10.1%} | {row['success_rate']:>12.1f}% | {row['total']:>6.0f}")
else:
    print("  None.")
print()

# Runtime analysis
print("=" * 80)
print("RUNTIME ANALYSIS")
print("=" * 80)
print()
print(f"Mean runtime: {df_tested['runtime'].mean():.2f}s")
print(f"Median runtime: {df_tested['runtime'].median():.2f}s")
print(f"Min runtime: {df_tested['runtime'].min():.2f}s")
print(f"Max runtime: {df_tested['runtime'].max():.2f}s")
print(f"Std dev: {df_tested['runtime'].std():.2f}s")
print()

# Slowest states
print("Top 10 slowest states (by average runtime):")
print("-" * 80)
slowest = state_success.nlargest(10, 'avg_runtime')[['districts', 'avg_runtime', 'success_rate']]
for state, row in slowest.iterrows():
    print(f"  {state:<20}: {row['avg_runtime']:>6.2f}s (k={row['districts']:.0f}, {row['success_rate']:.1f}% success)")
print()

# Fastest states
print("Top 10 fastest states (by average runtime):")
print("-" * 80)
fastest = state_success.nsmallest(10, 'avg_runtime')[['districts', 'avg_runtime', 'success_rate']]
for state, row in fastest.iterrows():
    print(f"  {state:<20}: {row['avg_runtime']:>6.2f}s (k={row['districts']:.0f}, {row['success_rate']:.1f}% success)")
print()

# Key findings
print("=" * 80)
print("KEY FINDINGS")
print("=" * 80)
print()

# Best parameter combinations
best_combo = df_tested.groupby(['weight_factor', 'minority_threshold']).agg({
    'success': ['sum', 'count']
}).round(2)
best_combo.columns = ['successes', 'total']
best_combo['rate'] = 100 * best_combo['successes'] / best_combo['total']
best_combo = best_combo.sort_values('rate', ascending=False)

print("Top 3 parameter combinations:")
print("-" * 80)
for i, ((wf, mt), row) in enumerate(best_combo.head(3).iterrows(), 1):
    print(f"{i}. Weight={wf}, Threshold={mt:.2f}: {row['rate']:.1f}% success ({int(row['successes'])}/{int(row['total'])})")
print()

# Correlation analysis
if df_tested['success'].sum() > 0 and df_tested['success'].sum() < len(df_tested):
    print("Correlation with success:")
    print("-" * 80)

    # Weight factor correlation
    weight_corr = df_tested.groupby('weight_factor')['success'].mean()
    print("  Weight factor trend:")
    for wf in sorted(weight_corr.index):
        print(f"    {wf:>3}: {100*weight_corr[wf]:.1f}% success")

    print()

    # Threshold correlation
    threshold_corr = df_tested.groupby('minority_threshold')['success'].mean()
    print("  Minority threshold trend:")
    for mt in sorted(threshold_corr.index):
        print(f"    {mt:.2f}: {100*threshold_corr[mt]:.1f}% success")

    print()

# Summary statistics
print("=" * 80)
print("SUMMARY STATISTICS")
print("=" * 80)
print()
print(f"Total states tested: {df_tested['state'].nunique()}")
print(f"Total configurations: {len(df_tested)}")
print(f"Overall success rate: {100*df_tested['success'].mean():.1f}%")
print(f"States with 100% success: {len(state_success[state_success['success_rate'] == 100])}")
print(f"States with 0% success: {len(state_success[state_success['success_rate'] == 0])}")
print(f"Average runtime: {df_tested['runtime'].mean():.2f}s")
print(f"Total runtime: {df_tested['runtime'].sum()/3600:.2f} hours")
print()

print("=" * 80)
print("ANALYSIS COMPLETE")
print("=" * 80)
