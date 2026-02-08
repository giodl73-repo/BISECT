"""
Cross-State Analysis: MM vs Non-MM District Compactness

Analyzes all 5 VRA states to identify patterns in the compactness-VRA tradeoff.
Key question: Does the pattern hold across different states?
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Load results
results_dir = Path(__file__).parent.parent / 'results'
state_df = pd.read_csv(results_dir / 'compactness_state_level.csv')
district_df = pd.read_csv(results_dir / 'compactness_district_level.csv')

# State names
STATE_NAMES = {
    'AL': 'Alabama',
    'GA': 'Georgia',
    'LA': 'Louisiana',
    'MS': 'Mississippi',
    'SC': 'South Carolina'
}

print("=" * 80)
print("CROSS-STATE COMPACTNESS-VRA TRADEOFF ANALYSIS")
print("=" * 80)

# Summary table for all states
summary_data = []

for state_code in sorted(state_df['state'].unique()):
    state_name = STATE_NAMES[state_code]

    # Get baseline
    baseline = state_df[
        (state_df['state'] == state_code) &
        (state_df['method'] == 'baseline')
    ].iloc[0]

    # Get successful configurations
    successful = state_df[
        (state_df['state'] == state_code) &
        (state_df['method'] == 'edge_weighted') &
        (state_df['success'] == True)
    ]

    print(f"\n{'=' * 80}")
    print(f"{state_name.upper()} ({state_code})")
    print(f"{'=' * 80}")

    print(f"\nBASELINE:")
    print(f"  Edge cut: {baseline['edge_cut_unweighted']}")
    print(f"  Avg PP: {baseline['avg_polsby_popper']:.3f}")
    print(f"  MM districts: {baseline['mm_count']}/{baseline['target_mm']}")

    if len(successful) > 0:
        # Find best (minimum edge cut)
        best = successful.loc[successful['edge_cut_unweighted'].idxmin()]

        print(f"\nBEST CONFIGURATION:")
        print(f"  Weight: {best['weight_factor']:.0f}x @ {best['minority_threshold']*100:.0f}%")
        print(f"  Edge cut: {best['edge_cut_unweighted']} ({(best['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100:+.1f}%)")
        print(f"  Avg PP: {best['avg_polsby_popper']:.3f} ({(best['avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}%)")
        print(f"  MM districts: {best['mm_count']}/{best['target_mm']}")

        if pd.notna(best['mm_avg_polsby_popper']) and pd.notna(best['non_mm_avg_polsby_popper']):
            mm_pp = best['mm_avg_polsby_popper']
            non_mm_pp = best['non_mm_avg_polsby_popper']
            baseline_pp = baseline['avg_polsby_popper']

            print(f"\n  DISTRICT BREAKDOWN:")
            print(f"    Baseline PP:         {baseline_pp:.3f}")
            print(f"    MM districts PP:     {mm_pp:.3f} ({(mm_pp/baseline_pp-1)*100:+.1f}%)")
            print(f"    Non-MM districts PP: {non_mm_pp:.3f} ({(non_mm_pp/baseline_pp-1)*100:+.1f}%)")

            # Determine who pays the cost
            mm_change = (mm_pp/baseline_pp - 1) * 100
            non_mm_change = (non_mm_pp/baseline_pp - 1) * 100

            if mm_change < 0 and non_mm_change > 0:
                pattern = "MM sacrifice, Non-MM gain"
            elif mm_change > 0 and non_mm_change < 0:
                pattern = "MM gain, Non-MM sacrifice"
            elif mm_change < 0 and non_mm_change < 0:
                pattern = "Both sacrifice"
            else:
                pattern = "Both gain"

            print(f"\n  PATTERN: {pattern}")

            # Store for summary
            summary_data.append({
                'state': state_code,
                'state_name': state_name,
                'target_mm': baseline['target_mm'],
                'baseline_edge_cut': baseline['edge_cut_unweighted'],
                'baseline_pp': baseline_pp,
                'best_weight': best['weight_factor'],
                'best_threshold': best['minority_threshold'],
                'best_edge_cut': best['edge_cut_unweighted'],
                'edge_cut_change_pct': (best['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100,
                'best_pp': best['avg_polsby_popper'],
                'pp_change_pct': (best['avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100,
                'mm_pp': mm_pp,
                'mm_change_pct': mm_change,
                'non_mm_pp': non_mm_pp,
                'non_mm_change_pct': non_mm_change,
                'pattern': pattern
            })
    else:
        print(f"\n  NO SUCCESSFUL CONFIGURATIONS FOUND!")
        summary_data.append({
            'state': state_code,
            'state_name': state_name,
            'target_mm': baseline['target_mm'],
            'baseline_edge_cut': baseline['edge_cut_unweighted'],
            'baseline_pp': baseline['avg_polsby_popper'],
            'best_weight': None,
            'best_threshold': None,
            'best_edge_cut': None,
            'edge_cut_change_pct': None,
            'best_pp': None,
            'pp_change_pct': None,
            'mm_pp': None,
            'mm_change_pct': None,
            'non_mm_pp': None,
            'non_mm_change_pct': None,
            'pattern': 'No success'
        })

# Create summary table
summary_df = pd.DataFrame(summary_data)

print("\n" + "=" * 80)
print("CROSS-STATE SUMMARY TABLE")
print("=" * 80)
print(summary_df.to_string(index=False))

# Save summary
summary_df.to_csv(results_dir / 'cross_state_summary.csv', index=False)
print(f"\nSaved: {results_dir / 'cross_state_summary.csv'}")

# Pattern frequency
print("\n" + "=" * 80)
print("PATTERN FREQUENCY")
print("=" * 80)
pattern_counts = summary_df['pattern'].value_counts()
print(pattern_counts)

# Average changes
print("\n" + "=" * 80)
print("AVERAGE CHANGES ACROSS STATES")
print("=" * 80)
successful_states = summary_df[summary_df['pattern'] != 'No success']
if len(successful_states) > 0:
    print(f"\nEdge Cut Change: {successful_states['edge_cut_change_pct'].mean():+.1f}% (avg)")
    print(f"Overall PP Change: {successful_states['pp_change_pct'].mean():+.1f}% (avg)")
    print(f"MM Districts PP Change: {successful_states['mm_change_pct'].mean():+.1f}% (avg)")
    print(f"Non-MM Districts PP Change: {successful_states['non_mm_change_pct'].mean():+.1f}% (avg)")

# Create visualizations
print("\n" + "=" * 80)
print("GENERATING CROSS-STATE VISUALIZATIONS...")
print("=" * 80)

# Figure 1: Cross-State Comparison
fig, axes = plt.subplots(2, 3, figsize=(18, 12))

# Plot 1: Edge Cut Change by State
ax = axes[0, 0]
if len(successful_states) > 0:
    colors = ['green' if x < 0 else 'red' for x in successful_states['edge_cut_change_pct']]
    ax.bar(successful_states['state'], successful_states['edge_cut_change_pct'],
           color=colors, alpha=0.7, edgecolor='black')
ax.axhline(y=0, color='black', linestyle='-', linewidth=1)
ax.set_ylabel('Edge Cut Change (%)', fontsize=11)
ax.set_title('Edge Cut: Baseline vs Best Config', fontsize=12, fontweight='bold')
ax.grid(axis='y', alpha=0.3)

# Plot 2: Overall PP Change by State
ax = axes[0, 1]
if len(successful_states) > 0:
    colors = ['green' if x > 0 else 'red' for x in successful_states['pp_change_pct']]
    ax.bar(successful_states['state'], successful_states['pp_change_pct'],
           color=colors, alpha=0.7, edgecolor='black')
ax.axhline(y=0, color='black', linestyle='-', linewidth=1)
ax.set_ylabel('Polsby-Popper Change (%)', fontsize=11)
ax.set_title('Overall Compactness: Baseline vs Best Config', fontsize=12, fontweight='bold')
ax.grid(axis='y', alpha=0.3)

# Plot 3: MM vs Non-MM PP Change
ax = axes[0, 2]
if len(successful_states) > 0:
    x = np.arange(len(successful_states))
    width = 0.35
    ax.bar(x - width/2, successful_states['mm_change_pct'],
           width, label='MM Districts', alpha=0.7, color='darkgreen', edgecolor='black')
    ax.bar(x + width/2, successful_states['non_mm_change_pct'],
           width, label='Non-MM Districts', alpha=0.7, color='coral', edgecolor='black')
    ax.set_xticks(x)
    ax.set_xticklabels(successful_states['state'])
ax.axhline(y=0, color='black', linestyle='-', linewidth=1)
ax.set_ylabel('Polsby-Popper Change (%)', fontsize=11)
ax.set_title('MM vs Non-MM District Compactness Change', fontsize=12, fontweight='bold')
ax.legend()
ax.grid(axis='y', alpha=0.3)

# Plot 4: MM vs Non-MM Scatter (All States)
ax = axes[1, 0]
if len(successful_states) > 0:
    for idx, row in successful_states.iterrows():
        ax.scatter([row['mm_pp']], [row['non_mm_pp']], s=200, alpha=0.7,
                   edgecolors='black', linewidths=1.5)
        ax.annotate(row['state'], (row['mm_pp'], row['non_mm_pp']),
                    fontsize=10, ha='right', va='bottom', fontweight='bold')

    # Diagonal line
    lims = [0, max(successful_states['mm_pp'].max(), successful_states['non_mm_pp'].max()) * 1.1]
    ax.plot(lims, lims, 'k--', alpha=0.3, linewidth=2, label='Equal Compactness')

ax.set_xlabel('MM Districts Polsby-Popper', fontsize=11)
ax.set_ylabel('Non-MM Districts Polsby-Popper', fontsize=11)
ax.set_title('MM vs Non-MM Compactness (All States)', fontsize=12, fontweight='bold')
ax.legend()
ax.grid(alpha=0.3)

# Plot 5: Best Weight Factor by State
ax = axes[1, 1]
if len(successful_states) > 0:
    ax.bar(successful_states['state'], successful_states['best_weight'],
           alpha=0.7, color='steelblue', edgecolor='black')
ax.set_ylabel('Weight Factor', fontsize=11)
ax.set_title('Optimal Weight Factor by State', fontsize=12, fontweight='bold')
ax.grid(axis='y', alpha=0.3)

# Plot 6: Best Threshold by State
ax = axes[1, 2]
if len(successful_states) > 0:
    ax.bar(successful_states['state'], successful_states['best_threshold']*100,
           alpha=0.7, color='orange', edgecolor='black')
ax.set_ylabel('Minority Threshold (%)', fontsize=11)
ax.set_title('Optimal Threshold by State', fontsize=12, fontweight='bold')
ax.grid(axis='y', alpha=0.3)

plt.tight_layout()
plt.savefig(results_dir / 'cross_state_analysis.png', dpi=300, bbox_inches='tight')
print(f"Saved: {results_dir / 'cross_state_analysis.png'}")

# Figure 2: Per-State District Breakdown
states_with_success = successful_states['state'].values
if len(states_with_success) > 0:
    fig, axes = plt.subplots(len(states_with_success), 1,
                             figsize=(12, 4*len(states_with_success)))

    if len(states_with_success) == 1:
        axes = [axes]

    for idx, state_code in enumerate(states_with_success):
        ax = axes[idx]
        state_name = STATE_NAMES[state_code]

        # Get baseline and best config for this state
        baseline_districts = district_df[
            (district_df['state'] == state_code) &
            (district_df['method'] == 'baseline')
        ].sort_values('district_id')

        best_config = successful_states[successful_states['state'] == state_code].iloc[0]
        best_districts = district_df[
            (district_df['state'] == state_code) &
            (district_df['weight_factor'] == best_config['best_weight']) &
            (district_df['minority_threshold'] == best_config['best_threshold'])
        ].sort_values('district_id')

        x = np.arange(len(baseline_districts))
        width = 0.35

        # Plot baseline
        ax.bar(x - width/2, baseline_districts['polsby_popper'].values,
               width, label='Baseline', alpha=0.8, color='steelblue')

        # Plot best config - color by MM status
        colors = ['darkgreen' if is_mm else 'coral'
                  for is_mm in best_districts['is_mm'].values]
        ax.bar(x + width/2, best_districts['polsby_popper'].values,
               width, label=f'Best ({best_config["best_weight"]:.0f}x@{best_config["best_threshold"]*100:.0f}%)',
               alpha=0.8, color=colors)

        # Add MM labels
        for i, (_, row) in enumerate(best_districts.iterrows()):
            if row['is_mm']:
                ax.text(i + width/2, row['polsby_popper'] + 0.01, 'MM',
                        ha='center', va='bottom', fontsize=8, fontweight='bold')

        ax.set_ylabel('Polsby-Popper', fontsize=11)
        ax.set_title(f'{state_name} District-Level Compactness',
                     fontsize=12, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels([f'D{i+1}' for i in range(len(baseline_districts))])
        ax.legend()
        ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(results_dir / 'per_state_districts.png', dpi=300, bbox_inches='tight')
    print(f"Saved: {results_dir / 'per_state_districts.png'}")

print("\n" + "=" * 80)
print("ANALYSIS COMPLETE!")
print("=" * 80)
