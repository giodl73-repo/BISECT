"""
Compare n-way vs recursive bisection for the 5 states we have data for.
"""

import pandas as pd
from pathlib import Path

# Load results
results_dir = Path(__file__).parent / "results"
nway_results = pd.read_csv(results_dir / "50_states_ablation_test.csv")
recursive_results = pd.read_csv(results_dir / "recursive_bisection_results.csv")

print("=" * 80)
print("N-WAY VS RECURSIVE BISECTION COMPARISON")
print("Limited to 5 states with recursive data")
print("=" * 80)
print()

# Filter n-way to just the 5 states we have recursive data for
states_to_compare = ['mississippi', 'louisiana', 'alabama', 'south_carolina', 'georgia']
nway_filtered = nway_results[nway_results['state'].isin(states_to_compare)].copy()

print(f"States compared: {', '.join([s.title() for s in states_to_compare])}")
print()

# For recursive, we only have one configuration per state
print("RECURSIVE BISECTION RESULTS (single configuration per state):")
print("-" * 80)
print(f"{'State':<20} | {'K':>3} | {'Target':>6} | {'Achieved':>8} | {'Success':>7} | {'Runtime':>8}")
print("-" * 80)
for _, row in recursive_results.iterrows():
    if not isinstance(row['state'], str):
        continue
    success = "YES" if row['mm_count'] >= row['target_mm'] else "NO"
    print(f"{row['state']:<20} | {row['k']:>3.0f} | {row['target_mm']:>6.0f} | "
          f"{row['mm_count']:>8.0f} | {success:>7} | {row['runtime']:>7.1f}s")
print()

# N-way summary for these 5 states
print("N-WAY RESULTS (20 configurations per state):")
print("-" * 80)
print(f"{'State':<20} | {'K':>3} | {'Target':>6} | {'Success Rate':>13} | {'Best MM':>7} | {'Avg Runtime':>12}")
print("-" * 80)

for state in states_to_compare:
    state_data = nway_filtered[nway_filtered['state'] == state]
    if len(state_data) == 0:
        continue

    success_rate = state_data['success'].sum() / len(state_data) * 100
    best_mm = state_data['mm_count'].max()
    avg_runtime = state_data['runtime'].mean()
    k = state_data['k'].iloc[0]
    target = state_data['target_mm'].iloc[0]

    print(f"{state:<20} | {k:>3.0f} | {target:>6.0f} | {success_rate:>12.1f}% | "
          f"{best_mm:>7.0f} | {avg_runtime:>11.2f}s")
print()

# Head-to-head comparison
print("=" * 80)
print("HEAD-TO-HEAD COMPARISON")
print("=" * 80)
print()

print(f"{'State':<20} | {'Target':>6} | {'Recursive':>9} | {'N-way Best':>11} | {'N-way %':>8} | {'Winner':>10}")
print("-" * 80)

for _, rec_row in recursive_results.iterrows():
    if not isinstance(rec_row['state'], str):
        continue

    state = rec_row['state']
    state_nway = nway_filtered[nway_filtered['state'] == state]

    if len(state_nway) == 0:
        continue

    rec_mm = rec_row['mm_count']
    target = rec_row['target_mm']
    nway_best_mm = state_nway['mm_count'].max()
    nway_success_rate = state_nway['success'].sum() / len(state_nway) * 100

    # Determine winner
    if rec_mm >= target and nway_success_rate == 0:
        winner = "Recursive"
    elif rec_mm < target and nway_success_rate > 0:
        winner = "N-way"
    elif rec_mm >= target and nway_success_rate > 0:
        winner = "Both"
    else:
        winner = "Neither"

    print(f"{state:<20} | {target:>6.0f} | {rec_mm:>9.0f} | {nway_best_mm:>11.0f} | "
          f"{nway_success_rate:>7.1f}% | {winner:>10}")

print()

# Summary statistics
print("=" * 80)
print("SUMMARY")
print("=" * 80)
print()

rec_successes = (recursive_results['mm_count'] >= recursive_results['target_mm']).sum()
rec_total = len(recursive_results[recursive_results['state'].notna()])

nway_unique_states = nway_filtered['state'].unique()
nway_state_success = []
for state in nway_unique_states:
    state_data = nway_filtered[nway_filtered['state'] == state]
    success_rate = state_data['success'].mean()
    nway_state_success.append(success_rate)

nway_avg_success = sum(nway_state_success) / len(nway_state_success) * 100

print(f"Recursive Bisection:")
print(f"  States achieving target: {rec_successes}/{rec_total} ({100*rec_successes/rec_total:.1f}%)")
print(f"  Average runtime: {recursive_results['runtime'].mean():.2f}s")
print()

print(f"N-way Partitioning:")
print(f"  Average state success rate: {nway_avg_success:.1f}%")
print(f"  States with >0% success: {sum(1 for x in nway_state_success if x > 0)}/{len(nway_state_success)}")
print(f"  Average runtime per config: {nway_filtered['runtime'].mean():.2f}s")
print()

# Performance insights
print("=" * 80)
print("KEY INSIGHTS")
print("=" * 80)
print()

print("1. Success Rate Comparison:")
for state in states_to_compare:
    rec_row = recursive_results[recursive_results['state'] == state]
    nway_state = nway_filtered[nway_filtered['state'] == state]

    if len(rec_row) == 0 or len(nway_state) == 0:
        continue

    rec_success = rec_row.iloc[0]['mm_count'] >= rec_row.iloc[0]['target_mm']
    nway_success_rate = nway_state['success'].mean() * 100

    print(f"   {state.title():<20}: Recursive={'YES' if rec_success else 'NO':>3}  |  "
          f"N-way={nway_success_rate:>5.1f}% ({nway_state['success'].sum()}/20)")

print()
print("2. Runtime Comparison:")
rec_avg = recursive_results['runtime'].mean()
nway_avg = nway_filtered['runtime'].mean()
print(f"   Recursive average: {rec_avg:.2f}s")
print(f"   N-way average: {nway_avg:.2f}s")
print(f"   Difference: {abs(rec_avg - nway_avg):.2f}s ({abs(rec_avg - nway_avg)/rec_avg*100:.1f}%)")

print()
print("3. Method Advantages:")
print(f"   - Recursive achieved target in {rec_successes}/{rec_total} states (single config)")
print(f"   - N-way achieved target in {sum(1 for x in nway_state_success if x > 0)}/{len(nway_state_success)} states (best of 20 configs)")
print()
print("=" * 80)
