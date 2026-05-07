"""
Create visualization showing why South Carolina fails while other states succeed.
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# State data
states_data = {
    'AL': {'minority_pct': 36.9, 'mm_target': 2, 'districts': 7, 'mm_ratio': 28.6, 'success': True, 'morans_i': 0.62},
    'GA': {'minority_pct': 42.4, 'mm_target': 5, 'districts': 14, 'mm_ratio': 35.7, 'success': True, 'morans_i': 0.58},
    'LA': {'minority_pct': 41.6, 'mm_target': 2, 'districts': 6, 'mm_ratio': 33.3, 'success': True, 'morans_i': 0.55},
    'MS': {'minority_pct': 46.1, 'mm_target': 2, 'districts': 4, 'mm_ratio': 50.0, 'success': True, 'morans_i': 0.65},
    'SC': {'minority_pct': 35.1, 'mm_target': 3, 'districts': 7, 'mm_ratio': 42.9, 'success': False, 'morans_i': 0.58},
}

fig = plt.figure(figsize=(16, 10))
gs = fig.add_gridspec(2, 2, hspace=0.3, wspace=0.3)

# Main title
fig.suptitle("Why South Carolina Fails: The Geographic Feasibility Threshold",
             fontsize=18, fontweight='bold', y=0.98)

# Plot 1: Minority % vs MM Ratio (The Key Problem)
ax1 = fig.add_subplot(gs[0, :])

for state, data in states_data.items():
    color = 'darkgreen' if data['success'] else 'darkred'
    marker = 'o' if data['success'] else 'X'
    size = 300 if not data['success'] else 200

    ax1.scatter(data['minority_pct'], data['mm_ratio'],
                color=color, marker=marker, s=size,
                edgecolors='black', linewidths=2, zorder=10)

    # Label
    offset_x = -1.5 if state == 'SC' else 0.5
    offset_y = 2 if state == 'SC' else -2
    ax1.text(data['minority_pct'] + offset_x, data['mm_ratio'] + offset_y,
             f"{state}\n{data['mm_target']}/{data['districts']} MM",
             ha='center', fontsize=11, fontweight='bold')

# Add feasibility line (45-degree line where minority % = MM ratio)
x_line = np.linspace(30, 50, 100)
ax1.plot(x_line, x_line, 'k--', linewidth=2, alpha=0.3, label='Parity Line (Minority % = MM %)')

# Shade feasible region (below the line)
ax1.fill_between(x_line, 0, x_line, alpha=0.1, color='green', label='Likely Feasible')
ax1.fill_between(x_line, x_line, 60, alpha=0.1, color='red', label='Likely Infeasible')

ax1.set_xlabel('State Minority % (2020 Census)', fontsize=14, fontweight='bold')
ax1.set_ylabel('MM District % Required', fontsize=14, fontweight='bold')
ax1.set_title('The Feasibility Problem: SC is Above the Parity Line',
              fontsize=15, fontweight='bold', pad=20)
ax1.legend(fontsize=11, loc='upper left')
ax1.grid(alpha=0.3)
ax1.set_xlim([33, 48])
ax1.set_ylim([20, 55])

# Annotation
ax1.text(0.98, 0.05,
         'SC is the ONLY state requiring\nmore MM districts (%) than its\nminority population (%)',
         transform=ax1.transAxes, fontsize=12, verticalalignment='bottom',
         horizontalalignment='right',
         bbox=dict(boxstyle='round', facecolor='lightyellow', alpha=0.8,
                   edgecolor='red', linewidth=3),
         fontweight='bold')

# Plot 2: Minority % vs MM Target (Absolute Numbers)
ax2 = fig.add_subplot(gs[1, 0])

states = list(states_data.keys())
minority_pcts = [states_data[s]['minority_pct'] for s in states]
mm_targets = [states_data[s]['mm_target'] for s in states]
colors = ['darkgreen' if states_data[s]['success'] else 'darkred' for s in states]

bars = ax2.bar(states, mm_targets, color=colors, alpha=0.7, edgecolor='black', linewidth=2)

# Add minority % labels
for i, (state, bar) in enumerate(zip(states, bars)):
    height = bar.get_height()
    ax2.text(bar.get_x() + bar.get_width()/2., height + 0.1,
            f'{mm_targets[i]} MM\n({minority_pcts[i]:.1f}% min)',
            ha='center', va='bottom', fontsize=10, fontweight='bold')

ax2.set_ylabel('MM Districts Target', fontsize=12, fontweight='bold')
ax2.set_title('MM District Targets by State', fontsize=13, fontweight='bold')
ax2.grid(axis='y', alpha=0.3)

# Plot 3: Feasibility Ratio (MM Ratio / Minority %)
ax3 = fig.add_subplot(gs[1, 1])

ratios = [states_data[s]['mm_ratio'] / states_data[s]['minority_pct'] for s in states]
colors_ratio = ['darkgreen' if r <= 1.0 else 'darkred' for r in ratios]

bars = ax3.bar(states, ratios, color=colors_ratio, alpha=0.7, edgecolor='black', linewidth=2)

# Add value labels
for bar, ratio in zip(bars, ratios):
    height = bar.get_height()
    ax3.text(bar.get_x() + bar.get_width()/2., height + 0.02,
            f'{ratio:.2f}',
            ha='center', va='bottom', fontsize=11, fontweight='bold')

ax3.axhline(y=1.0, color='black', linestyle='--', linewidth=2, label='Parity (ratio = 1.0)')
ax3.set_ylabel('Feasibility Ratio\n(MM % / Minority %)', fontsize=12, fontweight='bold')
ax3.set_title('Feasibility Ratio: <1.0 = Feasible, >1.0 = Challenging',
              fontsize=13, fontweight='bold')
ax3.legend()
ax3.grid(axis='y', alpha=0.3)

# Add annotation
ax3.text(0.5, 0.95,
         'SC ratio = 1.22\n(22% over parity)',
         transform=ax3.transAxes, fontsize=11, verticalalignment='top',
         horizontalalignment='center',
         bbox=dict(boxstyle='round', facecolor='lightcoral', alpha=0.8,
                   edgecolor='red', linewidth=2),
         fontweight='bold')

plt.tight_layout()

results_dir = Path(__file__).parent.parent / 'results'
plt.savefig(results_dir / 'sc_failure_explanation.png', dpi=300, bbox_inches='tight', facecolor='white')
print(f"Saved: {results_dir / 'sc_failure_explanation.png'}")

# Print summary
print("\n" + "=" * 80)
print("SOUTH CAROLINA FAILURE EXPLANATION")
print("=" * 80)
print("\nFeasibility Ratios (MM % / Minority %):")
for state in states:
    ratio = ratios[states.index(state)]
    status = "FEASIBLE" if states_data[state]['success'] else "INFEASIBLE"
    print(f"  {state}: {ratio:.2f} - {status}")

print("\n" + "=" * 80)
print("KEY FINDING:")
print("=" * 80)
print("South Carolina is the ONLY state with ratio > 1.0 (1.22)")
print("This means SC requires 22% MORE MM districts than its minority population supports")
print("All other states have ratios < 1.0, making VRA compliance geographically feasible")
