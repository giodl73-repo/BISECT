"""
Generate VRA vs Compactness tradeoff scatter plot.

X-axis: Edge cut (compactness - lower is better)
Y-axis: Maximum minority percentage achieved
Points: All 140 configurations
Color: By state
Shape: Success (circle) vs Fail (x)
Size: By weight factor
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Load results
data_file = Path(__file__).parent.parent / 'results' / 'edge_weighting_ablation_study.csv'
df = pd.read_csv(data_file)

# State colors and names
state_colors = {
    'MS': '#e74c3c',  # red
    'GA': '#3498db',  # blue
    'LA': '#2ecc71',  # green
    'AL': '#f39c12',  # orange
    'SC': '#9b59b6'   # purple
}

state_names = {
    'MS': 'Mississippi',
    'GA': 'Georgia',
    'LA': 'Louisiana',
    'AL': 'Alabama',
    'SC': 'South Carolina'
}

# Create figure
fig, ax = plt.subplots(figsize=(14, 8))

# Plot each state
for state in ['MS', 'GA', 'LA', 'AL', 'SC']:
    state_df = df[df['state'] == state]

    # Success points (circles)
    success_df = state_df[state_df['success'] == True]
    if len(success_df) > 0:
        sizes = success_df['weight_factor'].map(lambda x: 50 + np.log10(x+1) * 30)
        ax.scatter(success_df['edge_cut_unweighted'],
                  success_df['max_minority_pct'] * 100,
                  c=state_colors[state],
                  s=sizes,
                  alpha=0.6,
                  marker='o',
                  edgecolors='black',
                  linewidth=0.5,
                  label=f"{state_names[state]} (success)")

    # Fail points (x)
    fail_df = state_df[state_df['success'] == False]
    if len(fail_df) > 0:
        sizes = fail_df['weight_factor'].map(lambda x: 50 + np.log10(x+1) * 30)
        ax.scatter(fail_df['edge_cut_unweighted'],
                  fail_df['max_minority_pct'] * 100,
                  c=state_colors[state],
                  s=sizes,
                  alpha=0.3,
                  marker='x',
                  linewidth=1.5,
                  label=f"{state_names[state]} (fail)")

# Add 50% MM threshold line
ax.axhline(50, color='red', linestyle='--', linewidth=2, alpha=0.7, label='50% MM Threshold')

# Customize
ax.set_xlabel('Edge Cut (Compactness) - Lower is Better', fontsize=12, fontweight='bold')
ax.set_ylabel('Maximum Minority % Achieved', fontsize=12, fontweight='bold')
ax.set_title('VRA Compliance vs Compactness Tradeoff\nEdge-Weighted Optimization (140 Configurations)',
             fontsize=14, fontweight='bold', pad=20)

ax.grid(True, alpha=0.3, linestyle='--')
ax.legend(loc='upper right', frameon=True, shadow=True, ncol=2, fontsize=9)

# Add annotations for key points
# Alabama breakthrough
al_success = df[(df['state'] == 'AL') & (df['success'] == True) & (df['weight_factor'] == 5)]
if len(al_success) > 0:
    row = al_success.iloc[0]
    ax.annotate('Alabama\nBreakthrough\n(5x @ 40%)',
                xy=(row['edge_cut_unweighted'], row['max_minority_pct'] * 100),
                xytext=(row['edge_cut_unweighted'] + 100, row['max_minority_pct'] * 100 + 3),
                arrowprops=dict(arrowstyle='->', color='black', lw=1.5),
                fontsize=9,
                bbox=dict(boxstyle='round,pad=0.5', facecolor='yellow', alpha=0.7))

# Add size legend
for weight in [1, 10, 100, 1000]:
    size = 50 + np.log10(weight+1) * 30
    ax.scatter([], [], c='gray', s=size, alpha=0.6, marker='o',
              label=f'Weight {weight}x', edgecolors='black', linewidth=0.5)

ax.legend(loc='upper right', frameon=True, shadow=True, ncol=2, fontsize=8)

plt.tight_layout()

# Save
output_file = Path(__file__).parent.parent / 'figures' / 'vra_compactness_tradeoff.png'
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print(f"Saved to: {output_file}")

plt.show()
