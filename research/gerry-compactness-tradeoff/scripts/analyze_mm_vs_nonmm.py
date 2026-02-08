"""
Analyze MM vs Non-MM district compactness tradeoff.

Key Question: Do non-MM districts pay the compactness cost when we create MM districts?
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Load results
results_dir = Path(__file__).parent.parent / 'results'
state_df = pd.read_csv(results_dir / 'compactness_state_level.csv')
district_df = pd.read_csv(results_dir / 'compactness_district_level.csv')

print("=" * 80)
print("MM vs NON-MM DISTRICT COMPACTNESS ANALYSIS")
print("=" * 80)

# Get baseline
baseline = state_df[state_df['method'] == 'baseline'].iloc[0]
print(f"\nBASELINE (No VRA Optimization):")
print(f"  Edge cut: {baseline['edge_cut_unweighted']}")
print(f"  Avg Polsby-Popper: {baseline['avg_polsby_popper']:.3f}")
print(f"  MM districts: {baseline['mm_count']}/{baseline['target_mm']}")

# Get successful configurations
successful = state_df[
    (state_df['method'] == 'edge_weighted') &
    (state_df['success'] == True)
].copy()

print(f"\n" + "=" * 80)
print(f"SUCCESSFUL CONFIGURATIONS ({len(successful)} total)")
print("=" * 80)

for idx, row in successful.iterrows():
    print(f"\nConfig: {row['weight_factor']:.0f}x @ {row['minority_threshold']*100:.0f}%")
    print(f"  Edge cut: {row['edge_cut_unweighted']} ({(row['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100:+.1f}%)")
    print(f"  Avg PP: {row['avg_polsby_popper']:.3f} ({(row['avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}%)")
    print(f"  MM districts: {row['mm_count']}/{row['target_mm']}")

    if pd.notna(row['mm_avg_polsby_popper']) and pd.notna(row['non_mm_avg_polsby_popper']):
        mm_pp = row['mm_avg_polsby_popper']
        non_mm_pp = row['non_mm_avg_polsby_popper']
        baseline_pp = baseline['avg_polsby_popper']

        print(f"\n  DISTRICT BREAKDOWN:")
        print(f"    MM districts PP:     {mm_pp:.3f} ({(mm_pp/baseline_pp-1)*100:+.1f}% vs baseline)")
        print(f"    Non-MM districts PP: {non_mm_pp:.3f} ({(non_mm_pp/baseline_pp-1)*100:+.1f}% vs baseline)")
        print(f"    Gap: {(non_mm_pp - mm_pp):.3f} ({(non_mm_pp/mm_pp-1)*100:+.1f}%)")

        if mm_pp > non_mm_pp:
            print(f"    -> MM districts are MORE compact than non-MM")
        else:
            print(f"    -> Non-MM districts are MORE compact than MM")

# Find best configuration (minimum edge cut among successful)
if len(successful) > 0:
    best = successful.loc[successful['edge_cut_unweighted'].idxmin()]

    print("\n" + "=" * 80)
    print("BEST CONFIGURATION (Lowest Edge Cut with VRA Success)")
    print("=" * 80)
    print(f"\nConfig: {best['weight_factor']:.0f}x @ {best['minority_threshold']*100:.0f}%")
    print(f"  Edge cut: {best['edge_cut_unweighted']} ({(best['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100:+.1f}%)")
    print(f"  Avg PP: {best['avg_polsby_popper']:.3f} ({(best['avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}%)")
    print(f"  MM districts: {best['mm_count']}/{best['target_mm']}")

    print(f"\nDISTRICT BREAKDOWN:")
    print(f"  Baseline avg PP:     {baseline['avg_polsby_popper']:.3f}")
    print(f"  MM districts PP:     {best['mm_avg_polsby_popper']:.3f} ({(best['mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}% vs baseline)")
    print(f"  Non-MM districts PP: {best['non_mm_avg_polsby_popper']:.3f} ({(best['non_mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:+.1f}% vs baseline)")

    print(f"\nKEY FINDING:")
    if best['mm_avg_polsby_popper'] > baseline['avg_polsby_popper']:
        print(f"  MM districts IMPROVED compactness (+{(best['mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:.1f}%)")
    else:
        print(f"  MM districts REDUCED compactness ({(best['mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:.1f}%)")

    if best['non_mm_avg_polsby_popper'] > baseline['avg_polsby_popper']:
        print(f"  Non-MM districts IMPROVED compactness (+{(best['non_mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:.1f}%)")
    else:
        print(f"  Non-MM districts REDUCED compactness ({(best['non_mm_avg_polsby_popper']/baseline['avg_polsby_popper']-1)*100:.1f}%)")

# Create visualization
print("\n" + "=" * 80)
print("GENERATING VISUALIZATIONS...")
print("=" * 80)

# Figure 1: Edge Cut vs MM Count (Pareto Frontier)
fig, axes = plt.subplots(2, 2, figsize=(14, 10))

# Plot 1: Edge Cut vs MM Districts
ax = axes[0, 0]
edge_weighted = state_df[state_df['method'] == 'edge_weighted']

# Color by success
colors = edge_weighted['success'].map({True: 'green', False: 'red'})
ax.scatter(edge_weighted['edge_cut_unweighted'], edge_weighted['mm_count'],
           c=colors, s=100, alpha=0.6, edgecolors='black', linewidths=1)

# Baseline
ax.scatter([baseline['edge_cut_unweighted']], [baseline['mm_count']],
           c='blue', s=200, marker='*', edgecolors='black', linewidths=2,
           label='Baseline', zorder=10)

ax.set_xlabel('Edge Cut (lower = more compact)', fontsize=12)
ax.set_ylabel('MM Districts Created', fontsize=12)
ax.set_title('Compactness-VRA Tradeoff: Edge Cut vs MM Districts', fontsize=14, fontweight='bold')
ax.grid(alpha=0.3)
ax.legend()

# Plot 2: Polsby-Popper vs MM Districts
ax = axes[0, 1]
ax.scatter(edge_weighted['avg_polsby_popper'], edge_weighted['mm_count'],
           c=colors, s=100, alpha=0.6, edgecolors='black', linewidths=1)

ax.scatter([baseline['avg_polsby_popper']], [baseline['mm_count']],
           c='blue', s=200, marker='*', edgecolors='black', linewidths=2,
           label='Baseline', zorder=10)

ax.set_xlabel('Avg Polsby-Popper (higher = more compact)', fontsize=12)
ax.set_ylabel('MM Districts Created', fontsize=12)
ax.set_title('Geometric Compactness vs MM Districts', fontsize=14, fontweight='bold')
ax.grid(alpha=0.3)
ax.legend()

# Plot 3: MM vs Non-MM Compactness
ax = axes[1, 0]

# Only successful configurations
for idx, row in successful.iterrows():
    if pd.notna(row['mm_avg_polsby_popper']) and pd.notna(row['non_mm_avg_polsby_popper']):
        label = f"{row['weight_factor']:.0f}x@{row['minority_threshold']*100:.0f}%"
        ax.scatter([row['mm_avg_polsby_popper']], [row['non_mm_avg_polsby_popper']],
                   s=150, alpha=0.7, edgecolors='black', linewidths=1.5)
        ax.annotate(label, (row['mm_avg_polsby_popper'], row['non_mm_avg_polsby_popper']),
                    fontsize=9, ha='right', va='bottom')

# Add diagonal line (equal compactness)
lims = [0, 0.35]
ax.plot(lims, lims, 'k--', alpha=0.3, linewidth=2, label='Equal Compactness')

# Add baseline reference lines
ax.axhline(y=baseline['avg_polsby_popper'], color='blue', linestyle=':', alpha=0.5, label='Baseline PP')
ax.axvline(x=baseline['avg_polsby_popper'], color='blue', linestyle=':', alpha=0.5)

ax.set_xlabel('MM Districts Polsby-Popper', fontsize=12)
ax.set_ylabel('Non-MM Districts Polsby-Popper', fontsize=12)
ax.set_title('MM vs Non-MM District Compactness (Successful Configs)', fontsize=14, fontweight='bold')
ax.grid(alpha=0.3)
ax.legend()
ax.set_xlim(lims)
ax.set_ylim(lims)

# Plot 4: Weight Factor vs Compactness Cost
ax = axes[1, 1]

# Group by weight factor
for weight in sorted(successful['weight_factor'].unique()):
    configs = successful[successful['weight_factor'] == weight]
    ax.plot(configs['minority_threshold']*100,
            (configs['edge_cut_unweighted']/baseline['edge_cut_unweighted']-1)*100,
            marker='o', linewidth=2, markersize=8, label=f'{weight:.0f}x')

ax.axhline(y=0, color='black', linestyle='-', alpha=0.3, linewidth=1)
ax.set_xlabel('Minority Threshold (%)', fontsize=12)
ax.set_ylabel('Edge Cut Change vs Baseline (%)', fontsize=12)
ax.set_title('Compactness Cost by Weight Factor & Threshold', fontsize=14, fontweight='bold')
ax.grid(alpha=0.3)
ax.legend(title='Weight Factor', loc='best')

plt.tight_layout()
plt.savefig(results_dir / 'mm_vs_nonmm_analysis.png', dpi=300, bbox_inches='tight')
print(f"\nSaved: {results_dir / 'mm_vs_nonmm_analysis.png'}")

# Figure 2: District-Level Comparison (Best Config)
if len(successful) > 0:
    fig, ax = plt.subplots(figsize=(10, 6))

    # Get district-level data for baseline and best config
    baseline_districts = district_df[district_df['method'] == 'baseline']
    best_districts = district_df[
        (district_df['weight_factor'] == best['weight_factor']) &
        (district_df['minority_threshold'] == best['minority_threshold'])
    ]

    # Sort by district ID
    baseline_districts = baseline_districts.sort_values('district_id')
    best_districts = best_districts.sort_values('district_id')

    x = np.arange(len(baseline_districts))
    width = 0.35

    # Plot baseline
    ax.bar(x - width/2, baseline_districts['polsby_popper'].values,
           width, label='Baseline (No VRA)', alpha=0.8, color='steelblue')

    # Plot best config - color by MM status
    colors = ['darkgreen' if is_mm else 'coral' for is_mm in best_districts['is_mm'].values]
    ax.bar(x + width/2, best_districts['polsby_popper'].values,
           width, label=f'Edge-Weighted ({best["weight_factor"]:.0f}x@{best["minority_threshold"]*100:.0f}%)',
           alpha=0.8, color=colors)

    # Add MM labels
    for i, (idx, row) in enumerate(best_districts.iterrows()):
        if row['is_mm']:
            ax.text(i + width/2, row['polsby_popper'] + 0.01, 'MM',
                    ha='center', va='bottom', fontsize=8, fontweight='bold')

    ax.set_xlabel('District ID', fontsize=12)
    ax.set_ylabel('Polsby-Popper Compactness', fontsize=12)
    ax.set_title(f'District-Level Compactness: Baseline vs Best Edge-Weighted\n'
                 f'Green = MM Districts (>50% minority), Coral = Non-MM Districts',
                 fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([f'D{i+1}' for i in range(len(baseline_districts))])
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(results_dir / 'district_level_comparison.png', dpi=300, bbox_inches='tight')
    print(f"Saved: {results_dir / 'district_level_comparison.png'}")

print("\n" + "=" * 80)
print("ANALYSIS COMPLETE!")
print("=" * 80)
