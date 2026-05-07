"""
Create Pareto frontier visualizations for paper figures.

Generates:
1. pareto_frontiers_multistate.png - 4-panel comparison across states
2. alabama_pareto_detailed.png - Detailed Alabama configuration analysis
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Paths
RESULTS_DIR = Path(__file__).parent.parent / "results"
STATE_LEVEL_CSV = RESULTS_DIR / "compactness_state_level.csv"

# Load data
print(f"Loading data from {STATE_LEVEL_CSV}...")
df = pd.read_csv(STATE_LEVEL_CSV)

# Filter to successful states (exclude SC)
successful_states = ['AL', 'GA', 'LA', 'MS']

print(f"Loaded {len(df)} configurations")
print(f"States: {df['state'].unique()}")
print(f"Methods: {df['method'].unique()}")

# ==============================================================================
# Figure 1: Multi-State Pareto Frontiers (4 panels)
# ==============================================================================

fig, axes = plt.subplots(2, 2, figsize=(14, 12))
fig.suptitle('State-Dependent Pareto Frontiers for VRA-Compactness Tradeoffs',
             fontsize=16, fontweight='bold', y=0.995)

state_names = {'AL': 'Alabama', 'GA': 'Georgia', 'LA': 'Louisiana', 'MS': 'Mississippi'}
state_targets = {'AL': 2, 'GA': 5, 'LA': 2, 'MS': 2}

for idx, state in enumerate(successful_states):
    ax = axes[idx // 2, idx % 2]

    # Filter state data
    state_df = df[df['state'] == state].copy()

    # Identify baseline
    baseline = state_df[state_df['method'] == 'baseline'].iloc[0]

    # Edge-weighted configs
    edge_weighted = state_df[state_df['method'] == 'edge_weighted']

    # Identify Pareto frontier (simple version: for each MM count, best edge cut)
    pareto_points = []
    for mm_count in edge_weighted['mm_count'].unique():
        mm_configs = edge_weighted[edge_weighted['mm_count'] == mm_count]
        best_idx = mm_configs['edge_cut_unweighted'].idxmin()
        pareto_points.append(best_idx)

    pareto_df = edge_weighted.loc[pareto_points]
    non_pareto = edge_weighted[~edge_weighted.index.isin(pareto_points)]

    # Plot all configurations
    ax.scatter(non_pareto['mm_count'], non_pareto['edge_cut_unweighted'],
              c='lightgray', s=80, alpha=0.6, label='Other configs', zorder=2)

    # Plot Pareto-optimal points
    ax.scatter(pareto_df['mm_count'], pareto_df['edge_cut_unweighted'],
              c='red', s=150, edgecolors='darkred', linewidths=2,
              label='Pareto-optimal', zorder=4)

    # Plot baseline
    ax.scatter(baseline['mm_count'], baseline['edge_cut_unweighted'],
              c='blue', marker='^', s=200, edgecolors='darkblue',
              linewidths=2, label='Baseline', zorder=3)

    # Connect Pareto points with line
    pareto_sorted = pareto_df.sort_values('mm_count')
    ax.plot(pareto_sorted['mm_count'], pareto_sorted['edge_cut_unweighted'],
           'r--', linewidth=1.5, alpha=0.7, zorder=1)

    # Formatting
    ax.set_xlabel('Majority-Minority Districts', fontsize=12, fontweight='bold')
    ax.set_ylabel('Edge Cut', fontsize=12, fontweight='bold')
    ax.set_title(f'{state_names[state]} (Target: {state_targets[state]} MM)',
                fontsize=13, fontweight='bold')
    ax.grid(True, alpha=0.3, linestyle='--')
    ax.legend(loc='best', fontsize=9)

    # Add percentage change annotations for key points
    if state == 'AL':
        best_config = pareto_df[pareto_df['mm_count'] == 2].iloc[0]
        pct_change = ((best_config['edge_cut_unweighted'] - baseline['edge_cut_unweighted']) / baseline['edge_cut_unweighted']) * 100
        ax.annotate(f'{pct_change:.1f}%',
                   xy=(best_config['mm_count'], best_config['edge_cut_unweighted']),
                   xytext=(10, -15), textcoords='offset points',
                   fontsize=10, fontweight='bold', color='darkgreen',
                   bbox=dict(boxstyle='round,pad=0.3', facecolor='lightgreen', alpha=0.7))

plt.tight_layout()
output_path = RESULTS_DIR / "pareto_frontiers_multistate.png"
plt.savefig(output_path, dpi=300, bbox_inches='tight')
print(f"-> Saved: {output_path}")
plt.close()

# ==============================================================================
# Figure 2: Alabama Detailed Pareto Frontier
# ==============================================================================

fig, ax = plt.subplots(figsize=(10, 8))
fig.suptitle('Alabama Pareto Frontier: Detailed Configuration Analysis',
            fontsize=16, fontweight='bold')

# Filter Alabama data
al_df = df[df['state'] == 'AL'].copy()
baseline_al = al_df[al_df['method'] == 'baseline'].iloc[0]
edge_weighted_al = al_df[al_df['method'] == 'edge_weighted']

# Identify Pareto points
pareto_indices = []
for mm_count in edge_weighted_al['mm_count'].unique():
    mm_configs = edge_weighted_al[edge_weighted_al['mm_count'] == mm_count]
    best_idx = mm_configs['edge_cut_unweighted'].idxmin()
    pareto_indices.append(best_idx)

pareto_al = edge_weighted_al.loc[pareto_indices]
non_pareto_al = edge_weighted_al[~edge_weighted_al.index.isin(pareto_indices)]

# Plot non-Pareto configs
scatter = ax.scatter(non_pareto_al['mm_count'], non_pareto_al['edge_cut_unweighted'],
                    c='lightgray', s=120, alpha=0.6, label='Dominated configs', zorder=2)

# Plot Pareto-optimal configs
ax.scatter(pareto_al['mm_count'], pareto_al['edge_cut_unweighted'],
          c='red', s=200, edgecolors='darkred', linewidths=2.5,
          label='Pareto-optimal', zorder=4)

# Plot baseline
ax.scatter(baseline_al['mm_count'], baseline_al['edge_cut_unweighted'],
          c='blue', marker='^', s=250, edgecolors='darkblue',
          linewidths=2.5, label='Baseline (0 MM)', zorder=3)

# Connect Pareto frontier
pareto_sorted_al = pareto_al.sort_values('mm_count')
ax.plot(pareto_sorted_al['mm_count'], pareto_sorted_al['edge_cut_unweighted'],
       'r--', linewidth=2, alpha=0.7, zorder=1)

# Annotate key configurations
for _, row in pareto_al.iterrows():
    if pd.notna(row.get('weight_factor')) and pd.notna(row.get('threshold')):
        weight = int(row['weight_factor']) if row['weight_factor'] >= 1 else 1
        threshold = int(row['threshold'] * 100) if row['threshold'] < 1 else int(row['threshold'])
        label = f"{weight}×@{threshold}%"
        ax.annotate(label,
                   xy=(row['mm_count'], row['edge_cut_unweighted']),
                   xytext=(10, 5), textcoords='offset points',
                   fontsize=9, fontweight='bold',
                   bbox=dict(boxstyle='round,pad=0.4', facecolor='lightyellow', edgecolor='orange', alpha=0.8),
                   arrowprops=dict(arrowstyle='->', color='black', lw=1))

# Add regions
ax.axhline(y=baseline_al['edge_cut_unweighted'], color='blue', linestyle=':', alpha=0.5, linewidth=1.5)
ax.text(1.5, baseline_al['edge_cut_unweighted'] - 15, 'Baseline edge cut',
       fontsize=10, color='blue', fontweight='bold', ha='center',
       bbox=dict(boxstyle='round,pad=0.3', facecolor='lightblue', alpha=0.7))

# Formatting
ax.set_xlabel('Majority-Minority (MM) Districts Achieved', fontsize=13, fontweight='bold')
ax.set_ylabel('Edge Cut (Lower is Better)', fontsize=13, fontweight='bold')
ax.grid(True, alpha=0.3, linestyle='--')
ax.legend(loc='upper right', fontsize=11, framealpha=0.9)

# Set limits
ax.set_xlim(-0.3, 2.5)
ax.set_ylim(al_df['edge_cut_unweighted'].min() - 20, al_df['edge_cut_unweighted'].max() + 20)

plt.tight_layout()
output_path = RESULTS_DIR / "alabama_pareto_detailed.png"
plt.savefig(output_path, dpi=300, bbox_inches='tight')
print(f"-> Saved: {output_path}")
plt.close()

print("\nPareto frontier figures created successfully!")
print(f"1. {RESULTS_DIR / 'pareto_frontiers_multistate.png'}")
print(f"2. {RESULTS_DIR / 'alabama_pareto_detailed.png'}")
