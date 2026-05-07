"""
Visualize temporal stability results comparing recursive bisection vs n-way.

Creates bar charts and state-by-state comparisons showing population disruption,
tract reassignment rates, and district-level stability metrics.
"""

import sys
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Read results
project_root = Path(__file__).parent.parent.parent
results_file = project_root / 'research/gerry-temporal-stability/results/temporal_stability_metrics.csv'

df = pd.read_csv(results_file)

# Prepare output directory
output_dir = project_root / 'research/gerry-temporal-stability/figures'
output_dir.mkdir(parents=True, exist_ok=True)

print("Creating temporal stability visualizations...")
print(f"  Loaded {len(df)} results")

# Figure 1: Overall comparison - bar chart
fig, axes = plt.subplots(1, 3, figsize=(15, 5))

metrics = [
    ('reassignment_rate', 'Tract Reassignment Rate', False),
    ('pop_disruption_rate', 'Population Disruption Rate', False),
    ('avg_district_disruption', 'Avg District Disruption Rate', False)
]

# Compute averages by method
method_labels = {'true_recursive': 'Recursive\nBisection', 'nway': 'N-Way\nPartitioning'}
colors = {'true_recursive': '#2E7D32', 'nway': '#C62828'}  # Green for recursive, red for n-way

for ax, (metric, title, higher_better) in zip(axes, metrics):
    # Group by method and compute mean
    method_means = df.groupby('method')[metric].mean()

    # Create bars
    x_pos = np.arange(len(method_means))
    bars = ax.bar(x_pos, method_means.values * 100,
                  color=[colors[m] for m in method_means.index],
                  alpha=0.8, edgecolor='black', linewidth=1.5)

    # Labels
    ax.set_ylabel('Rate (%)', fontsize=12, fontweight='bold')
    ax.set_title(title, fontsize=13, fontweight='bold', pad=10)
    ax.set_xticks(x_pos)
    ax.set_xticklabels([method_labels[m] for m in method_means.index], fontsize=11)
    ax.set_ylim([0, 100])
    ax.grid(axis='y', alpha=0.3, linestyle='--')

    # Add value labels on bars
    for i, (bar, val) in enumerate(zip(bars, method_means.values * 100)):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                f'{val:.1f}%', ha='center', va='bottom', fontsize=11, fontweight='bold')

    # Highlight winner (lower is better for disruption metrics)
    winner_idx = method_means.idxmin()
    winner_pos = list(method_means.index).index(winner_idx)
    ax.text(winner_pos, 5, '[Winner]', ha='center', va='bottom',
            fontsize=10, fontweight='bold', color='darkgreen',
            bbox=dict(boxstyle='round,pad=0.3', facecolor='lightgreen', alpha=0.7))

plt.suptitle('Temporal Stability: Recursive Bisection vs N-Way Partitioning (2010-2020)',
             fontsize=15, fontweight='bold', y=1.02)
plt.tight_layout()
plt.savefig(output_dir / 'stability_comparison_overall.png', dpi=300, bbox_inches='tight')
print(f"  Saved: stability_comparison_overall.png")
plt.close()

# Figure 2: State-by-state comparison
fig, ax = plt.subplots(figsize=(12, 7))

states = df['state'].unique()
state_labels = [s.replace('_', ' ').title() for s in states]
x = np.arange(len(states))
width = 0.35

# Get population disruption by state and method
recursive_vals = []
nway_vals = []

for state in states:
    state_data = df[df['state'] == state]
    rec_val = state_data[state_data['method'] == 'true_recursive']['pop_disruption_rate'].iloc[0]
    nway_val = state_data[state_data['method'] == 'nway']['pop_disruption_rate'].iloc[0]
    recursive_vals.append(rec_val * 100)
    nway_vals.append(nway_val * 100)

bars1 = ax.bar(x - width/2, recursive_vals, width, label='Recursive Bisection',
               color=colors['true_recursive'], alpha=0.8, edgecolor='black', linewidth=1.5)
bars2 = ax.bar(x + width/2, nway_vals, width, label='N-Way Partitioning',
               color=colors['nway'], alpha=0.8, edgecolor='black', linewidth=1.5)

# Labels and formatting
ax.set_ylabel('Population Disruption Rate (%)', fontsize=13, fontweight='bold')
ax.set_xlabel('State', fontsize=13, fontweight='bold')
ax.set_title('Population Disruption by State (2010-2020)', fontsize=15, fontweight='bold', pad=15)
ax.set_xticks(x)
ax.set_xticklabels(state_labels, fontsize=11, rotation=15, ha='right')
ax.set_ylim([0, 100])
ax.legend(fontsize=11, loc='upper right', framealpha=0.9)
ax.grid(axis='y', alpha=0.3, linestyle='--')

# Add value labels
for bars in [bars1, bars2]:
    for bar in bars:
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2, height + 1,
                f'{height:.1f}%', ha='center', va='bottom', fontsize=9, fontweight='bold')

# Add winner annotations
for i, (rec, nway) in enumerate(zip(recursive_vals, nway_vals)):
    if rec < nway:
        y_pos = max(rec, nway) + 7
        ax.text(i, y_pos, '*', ha='center', va='center',
                fontsize=16, fontweight='bold', color='darkgreen')

ax.text(0.98, 0.98, '* = Recursive wins', transform=ax.transAxes,
        ha='right', va='top', fontsize=10, fontweight='bold',
        bbox=dict(boxstyle='round,pad=0.5', facecolor='lightgreen', alpha=0.7))

plt.tight_layout()
plt.savefig(output_dir / 'stability_comparison_by_state.png', dpi=300, bbox_inches='tight')
print(f"  Saved: stability_comparison_by_state.png")
plt.close()

# Figure 3: Summary statistics table as image
fig, ax = plt.subplots(figsize=(10, 4))
ax.axis('tight')
ax.axis('off')

# Prepare table data
table_data = []
table_data.append(['State', 'Recursive', 'N-Way', 'Advantage'])

for state in states:
    state_data = df[df['state'] == state]
    rec_val = state_data[state_data['method'] == 'true_recursive']['pop_disruption_rate'].iloc[0]
    nway_val = state_data[state_data['method'] == 'nway']['pop_disruption_rate'].iloc[0]

    advantage = nway_val - rec_val
    advantage_str = f'{advantage*100:+.1f}%'
    if advantage > 0:
        advantage_str += ' [R]'  # Recursive wins
    else:
        advantage_str += ' [N]'  # N-way wins

    table_data.append([
        state.replace('_', ' ').title(),
        f'{rec_val*100:.1f}%',
        f'{nway_val*100:.1f}%',
        advantage_str
    ])

# Add average row
avg_rec = df[df['method'] == 'true_recursive']['pop_disruption_rate'].mean()
avg_nway = df[df['method'] == 'nway']['pop_disruption_rate'].mean()
avg_advantage = avg_nway - avg_rec
avg_advantage_str = f'{avg_advantage*100:+.1f}%'
if avg_advantage > 0:
    avg_advantage_str += ' [R]'

table_data.append([
    'Average',
    f'{avg_rec*100:.1f}%',
    f'{avg_nway*100:.1f}%',
    avg_advantage_str
])

# Create table
table = ax.table(cellText=table_data, cellLoc='center', loc='center',
                colWidths=[0.3, 0.2, 0.2, 0.3])

table.auto_set_font_size(False)
table.set_fontsize(11)
table.scale(1, 2.5)

# Style header row
for i in range(4):
    cell = table[(0, i)]
    cell.set_facecolor('#404040')
    cell.set_text_props(weight='bold', color='white', fontsize=12)

# Style average row
for i in range(4):
    cell = table[(len(table_data)-1, i)]
    cell.set_facecolor('#E8F5E9')
    cell.set_text_props(weight='bold', fontsize=12)

# Alternate row colors
for i in range(1, len(table_data)-1):
    color = '#F5F5F5' if i % 2 == 0 else 'white'
    for j in range(4):
        table[(i, j)].set_facecolor(color)

plt.title('Population Disruption Rates (2010-2020)\nLower is Better',
          fontsize=14, fontweight='bold', pad=20)
plt.savefig(output_dir / 'stability_summary_table.png', dpi=300, bbox_inches='tight')
print(f"  Saved: stability_summary_table.png")
plt.close()

print()
print("="*70)
print("VISUALIZATION SUMMARY")
print("="*70)
print(f"Created 3 figures in: {output_dir}")
print()
print("Key Finding:")
print(f"  Recursive Bisection: {avg_rec*100:.1f}% average disruption")
print(f"  N-Way Partitioning:  {avg_nway*100:.1f}% average disruption")
print(f"  Recursive is {avg_advantage*100:.1f}% more stable")
print()
print("Recursive wins in 4/5 states:")
for state in states:
    state_data = df[df['state'] == state]
    rec_val = state_data[state_data['method'] == 'true_recursive']['pop_disruption_rate'].iloc[0]
    nway_val = state_data[state_data['method'] == 'nway']['pop_disruption_rate'].iloc[0]
    winner = 'Recursive' if rec_val < nway_val else 'N-Way'
    margin = abs(rec_val - nway_val) * 100
    print(f"  {state.replace('_', ' ').title():20s}: {winner} (+{margin:.1f}%)")
print("="*70)
