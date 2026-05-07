"""
Generate VRA success comparison visualization.

Compares three approaches:
1. Multi-constraint (tpwgts only)
2. Multi-constraint (ubvec)
3. Edge-weighting (optimal config)
"""

import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# VRA state data
states = {
    'MS': {'name': 'Mississippi', 'minority_pct': 46.1, 'target_mm': 2},
    'GA': {'name': 'Georgia', 'minority_pct': 42.4, 'target_mm': 5},
    'LA': {'name': 'Louisiana', 'minority_pct': 41.6, 'target_mm': 2},
    'AL': {'name': 'Alabama', 'minority_pct': 36.9, 'target_mm': 2},
    'SC': {'name': 'South Carolina', 'minority_pct': 35.1, 'target_mm': 3},
}

# Results (MM districts achieved)
multi_constraint_tpwgts = {'MS': 2, 'GA': 5, 'LA': 1, 'AL': 0, 'SC': 0}
multi_constraint_ubvec = {'MS': 2, 'GA': 5, 'LA': 1, 'AL': 0, 'SC': 0}
edge_weighting = {'MS': 2, 'GA': 5, 'LA': 2, 'AL': 2, 'SC': 0}

# Set up figure
fig, ax = plt.subplots(figsize=(12, 7))

# Sort states by minority percentage (descending)
sorted_states = sorted(states.keys(), key=lambda s: states[s]['minority_pct'], reverse=True)

x = np.arange(len(sorted_states))
width = 0.25

# Create bars
bars1 = ax.bar(x - width, [multi_constraint_tpwgts[s] for s in sorted_states],
               width, label='Multi-constraint (tpwgts)', color='#e74c3c', alpha=0.8)
bars2 = ax.bar(x, [multi_constraint_ubvec[s] for s in sorted_states],
               width, label='Multi-constraint (ubvec)', color='#e67e22', alpha=0.8)
bars3 = ax.bar(x + width, [edge_weighting[s] for s in sorted_states],
               width, label='Edge-weighting (optimal)', color='#27ae60', alpha=0.8)

# Add target MM lines
for i, state in enumerate(sorted_states):
    target = states[state]['target_mm']
    ax.plot([i - width*1.5, i + width*1.5], [target, target],
            'k--', linewidth=1.5, alpha=0.5)

# Customize plot
ax.set_xlabel('State (sorted by minority %)', fontsize=12, fontweight='bold')
ax.set_ylabel('MM Districts Achieved', fontsize=12, fontweight='bold')
ax.set_title('VRA Compliance: Multi-Constraint vs Edge-Weighting',
             fontsize=14, fontweight='bold', pad=20)

# X-axis labels with minority percentages
labels = [f"{states[s]['name']}\n({states[s]['minority_pct']}% minority)"
          for s in sorted_states]
ax.set_xticks(x)
ax.set_xticklabels(labels)

ax.legend(loc='upper right', frameon=True, shadow=True)
ax.grid(axis='y', alpha=0.3, linestyle='--')

# Add success annotations
for i, state in enumerate(sorted_states):
    target = states[state]['target_mm']

    # Multi-constraint tpwgts
    val1 = multi_constraint_tpwgts[state]
    color1 = 'green' if val1 >= target else 'red'
    ax.text(i - width, val1 + 0.1, f"{val1}/{target}",
            ha='center', va='bottom', fontsize=9, color=color1, fontweight='bold')

    # Edge-weighting
    val3 = edge_weighting[state]
    color3 = 'green' if val3 >= target else 'red'
    ax.text(i + width, val3 + 0.1, f"{val3}/{target}",
            ha='center', va='bottom', fontsize=9, color=color3, fontweight='bold')

plt.tight_layout()

# Save figure
output_dir = Path(__file__).parent.parent / 'figures'
output_dir.mkdir(exist_ok=True)
output_file = output_dir / 'approach_comparison.png'
plt.savefig(output_file, dpi=300, bbox_inches='tight')
print(f"Saved to: {output_file}")

plt.show()
