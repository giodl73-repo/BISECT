"""
Generate ablation study heatmap.

Shows MM districts achieved across all 140 configurations:
- Rows: 5 states × 4 thresholds (20 rows)
- Columns: 7 weight factors
- Cell color: Success (green) vs Fail (red)
- Cell text: MM count / target
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
from pathlib import Path

# Load results
data_file = Path(__file__).parent.parent / 'results' / 'edge_weighting_ablation_study.csv'
df = pd.read_csv(data_file)

# State ordering (by minority percentage, descending)
state_order = ['MS', 'GA', 'LA', 'AL', 'SC']
state_names = {
    'MS': 'Mississippi (46.1%)',
    'GA': 'Georgia (42.4%)',
    'LA': 'Louisiana (41.6%)',
    'AL': 'Alabama (36.9%)',
    'SC': 'South Carolina (35.1%)'
}

# Weight factor and threshold ordering
weight_factors = [1, 5, 10, 50, 100, 500, 1000]
thresholds = [0.40, 0.45, 0.50, 0.55]

# Create pivot tables
mm_counts = df.pivot_table(
    index=['state', 'minority_threshold'],
    columns='weight_factor',
    values='mm_count',
    aggfunc='first'
)

success = df.pivot_table(
    index=['state', 'minority_threshold'],
    columns='weight_factor',
    values='success',
    aggfunc='first'
)

targets = df.groupby('state')['target_mm'].first()

# Prepare data matrices for heatmap
n_states = len(state_order)
n_thresholds = len(thresholds)
n_weights = len(weight_factors)

# Create matrices
heatmap_data = np.zeros((n_states * n_thresholds, n_weights))
annotations = [['' for _ in range(n_weights)] for _ in range(n_states * n_thresholds)]
row_labels = []

row_idx = 0
for state in state_order:
    for threshold in thresholds:
        row_labels.append(f"{state} @ {int(threshold*100)}%")

        for col_idx, weight in enumerate(weight_factors):
            try:
                mm = mm_counts.loc[(state, threshold), weight]
                target = targets[state]
                is_success = success.loc[(state, threshold), weight]

                # Color: 1 for success, 0 for fail
                heatmap_data[row_idx, col_idx] = 1.0 if is_success else 0.0

                # Annotation: MM count / target
                annotations[row_idx][col_idx] = f"{int(mm)}/{int(target)}"
            except KeyError:
                heatmap_data[row_idx, col_idx] = np.nan
                annotations[row_idx][col_idx] = '--'

        row_idx += 1

# Create figure
fig, ax = plt.subplots(figsize=(12, 14))

# Custom colormap: red for fail, green for success
cmap = sns.diverging_palette(10, 130, as_cmap=True)

# Create heatmap
sns.heatmap(heatmap_data,
            annot=np.array(annotations),
            fmt='',
            cmap=cmap,
            center=0.5,
            vmin=0,
            vmax=1,
            linewidths=0.5,
            linecolor='gray',
            cbar_kws={'label': 'Success', 'ticks': [0, 1]},
            ax=ax)

# Customize
ax.set_xlabel('Weight Factor (α)', fontsize=12, fontweight='bold')
ax.set_ylabel('State @ Minority Threshold (τ)', fontsize=12, fontweight='bold')
ax.set_title('Edge-Weighting Ablation Study: VRA Compliance Across 140 Configurations',
             fontsize=14, fontweight='bold', pad=20)

# Set tick labels
ax.set_xticklabels([f"{w}x" for w in weight_factors], rotation=0)
ax.set_yticklabels(row_labels, rotation=0)

# Add state separators
for i in range(1, n_states):
    ax.axhline(i * n_thresholds, color='black', linewidth=2)

# Colorbar labels
cbar = ax.collections[0].colorbar
cbar.set_ticks([0.25, 0.75])
cbar.set_ticklabels(['Fail', 'Success'])

plt.tight_layout()

# Save
output_file = Path(__file__).parent.parent / 'figures' / 'ablation_heatmap.png'
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print(f"Saved to: {output_file}")

plt.show()
