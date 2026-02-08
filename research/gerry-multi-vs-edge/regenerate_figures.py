"""
Regenerate all figures with P1-3 balanced comparison data.

Uses:
- results/edge_weighted_results.csv (from Paper 1 - 140 configs)
- results/multi_constraint_results_v2.csv (P1-3 - 140 configs, balanced)
"""

import sys
from pathlib import Path
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.gridspec import GridSpec
import seaborn as sns

# Set style
plt.style.use('seaborn-v0_8-paper')
sns.set_palette("husl")

# Load data
print("Loading data...")
df_edge = pd.read_csv('../gerry-vra-compliance/results/edge_weighting_ablation_study.csv')
df_multi = pd.read_csv('results/multi_constraint_results_v2.csv')

# Rename multi-constraint columns for consistency with edge-weighted
df_multi = df_multi.rename(columns={'num_mm': 'mm_count'})

# Rename edge-weighted columns for consistency
df_edge = df_edge.rename(columns={
    'weight_factor': 'edge_weight_scale_factor',
    'edge_cut_unweighted': 'edge_cut'
})

STATES = {
    'AL': 'Alabama',
    'GA': 'Georgia',
    'LA': 'Louisiana',
    'MS': 'Mississippi',
    'SC': 'South Carolina'
}

print(f"Loaded {len(df_edge)} edge-weighted configs and {len(df_multi)} multi-constraint configs")
print()

# ==============================================================================
# Figure 1: Success Rates
# ==============================================================================

def generate_figure1():
    """Success rate comparison"""
    print("Generating Figure 1: Success Rates...")

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))

    # Panel A: Configuration-level success
    ax = axes[0]

    edge_success = (df_edge['success'].sum() / len(df_edge)) * 100
    multi_success = (df_multi['success'].sum() / len(df_multi)) * 100

    x = [0, 1]
    heights = [edge_success, multi_success]
    colors = ['#3498db', '#e74c3c']

    bars = ax.bar(x, heights, color=colors, width=0.6, alpha=0.8)

    # Add value labels
    for i, (bar, val) in enumerate(zip(bars, heights)):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                f'{val:.1f}%', ha='center', va='bottom', fontsize=11, fontweight='bold')

    ax.set_ylabel('Configuration Success Rate (%)', fontsize=11)
    ax.set_xticks(x)
    ax.set_xticklabels(['Edge-Weighted', 'Multi-Constraint'], fontsize=11)
    ax.set_ylim(0, 60)
    ax.set_title('(A) Configuration-Level Success', fontsize=12, fontweight='bold')
    ax.grid(axis='y', alpha=0.3)

    # Panel B: State-level success
    ax = axes[1]

    edge_states = len(df_edge[df_edge['success'] == True]['state'].unique())
    multi_states = len(df_multi[df_multi['success'] == True]['state'].unique())

    x = [0, 1]
    heights = [(edge_states/5)*100, (multi_states/5)*100]

    bars = ax.bar(x, heights, color=colors, width=0.6, alpha=0.8)

    # Add value labels
    for i, (bar, val, count) in enumerate(zip(bars, heights, [edge_states, multi_states])):
        ax.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 2,
                f'{val:.0f}%\n({count}/5)', ha='center', va='bottom', fontsize=11, fontweight='bold')

    ax.set_ylabel('State Success Rate (%)', fontsize=11)
    ax.set_xticks(x)
    ax.set_xticklabels(['Edge-Weighted', 'Multi-Constraint'], fontsize=11)
    ax.set_ylim(0, 100)
    ax.set_title('(B) State-Level Success', fontsize=12, fontweight='bold')
    ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig('results/figure1_success_rates.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure1_success_rates.png', dpi=300, bbox_inches='tight')
    plt.close()

    print(f"  Edge-weighted: {edge_success:.1f}% config, {edge_states}/5 states")
    print(f"  Multi-constraint: {multi_success:.1f}% config, {multi_states}/5 states")
    print("  Saved: figure1_success_rates.pdf/png")
    print()


# ==============================================================================
# Figure 2: Compactness-Concentration Tradeoff
# ==============================================================================

def generate_figure2():
    """Compactness vs concentration tradeoff"""
    print("Generating Figure 2: Compactness-Concentration Tradeoff...")

    fig, ax = plt.subplots(figsize=(8, 6))

    # Best config per state/method
    for state_code in STATES.keys():
        # Edge-weighted best
        edge_state = df_edge[df_edge['state'] == state_code]
        edge_best = edge_state.sort_values(['mm_count', 'max_minority_pct'], ascending=[False, False]).iloc[0]

        # Multi-constraint best
        multi_state = df_multi[df_multi['state'] == state_code]
        multi_best = multi_state.sort_values(['mm_count', 'max_minority_pct'], ascending=[False, False]).iloc[0]

        # Plot
        marker = 'o' if edge_best['success'] else 'o'
        face = '#3498db' if edge_best['success'] else 'white'
        ax.scatter(edge_best['edge_cut'], edge_best['max_minority_pct']*100,
                  s=150, c=face, edgecolors='#3498db', linewidths=2, marker=marker,
                  label='Edge-Weighted' if state_code == 'AL' else '', zorder=3)

        marker = 's' if multi_best['success'] else 's'
        face = '#e74c3c' if multi_best['success'] else 'white'
        ax.scatter(multi_best['edge_cut'], multi_best['max_minority_pct']*100,
                  s=150, c=face, edgecolors='#e74c3c', linewidths=2, marker=marker,
                  label='Multi-Constraint' if state_code == 'AL' else '', zorder=3)

        # Connect with line
        ax.plot([edge_best['edge_cut'], multi_best['edge_cut']],
               [edge_best['max_minority_pct']*100, multi_best['max_minority_pct']*100],
               'k--', alpha=0.3, linewidth=1, zorder=1)

        # Add state label
        mid_x = (edge_best['edge_cut'] + multi_best['edge_cut']) / 2
        mid_y = (edge_best['max_minority_pct'] + multi_best['max_minority_pct']) * 50
        ax.text(mid_x, mid_y + 1.5, state_code, fontsize=9, ha='center',
               bbox=dict(boxstyle='round,pad=0.3', facecolor='white', alpha=0.7))

    # 50% threshold line
    ax.axhline(50, color='red', linestyle='--', linewidth=1.5, alpha=0.5, label='50% Threshold')

    ax.set_xlabel('Edge Cut (Lower = More Compact)', fontsize=11)
    ax.set_ylabel('Maximum Minority Percentage (%)', fontsize=11)
    ax.set_title('Compactness-Concentration Tradeoff', fontsize=12, fontweight='bold')
    ax.legend(loc='best', framealpha=0.9)
    ax.grid(alpha=0.3)

    plt.tight_layout()
    plt.savefig('results/figure2_compactness_tradeoff.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure2_compactness_tradeoff.png', dpi=300, bbox_inches='tight')
    plt.close()

    print("  Saved: figure2_compactness_tradeoff.pdf/png")
    print()


# ==============================================================================
# Figure 3: Constraint Conflict (Alabama)
# ==============================================================================

def generate_figure3():
    """Alabama constraint conflict test"""
    print("Generating Figure 3: Alabama Constraint Conflict...")

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))

    # Filter Alabama multi-constraint
    al_multi = df_multi[df_multi['state'] == 'AL'].sort_values('ubvec_minority')

    ubvec_labels = ['1.3\n(±30%)', '1.5\n(±50%)', '2.0\n(±100%)', '5.0\n(±400%)']

    # Panel A: MM district count
    ax = axes[0]

    target = al_multi.iloc[0]['target_mm']
    mm_counts = al_multi['mm_count'].values

    ax.bar(range(len(mm_counts)), mm_counts, color='#e74c3c', alpha=0.7, width=0.6)
    ax.axhline(target, color='red', linestyle='--', linewidth=2, label=f'Target ({target} MM)')

    ax.set_xlabel('Minority Tolerance (ubvec)', fontsize=11)
    ax.set_ylabel('MM Districts Achieved', fontsize=11)
    ax.set_xticks(range(len(ubvec_labels)))
    ax.set_xticklabels(ubvec_labels, fontsize=10)
    ax.set_ylim(0, 3)
    ax.set_title('(A) MM District Count vs Tolerance', fontsize=12, fontweight='bold')
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    # Panel B: Maximum minority percentage
    ax = axes[1]

    max_pcts = al_multi['max_minority_pct'].values * 100

    ax.bar(range(len(max_pcts)), max_pcts, color='#e74c3c', alpha=0.7, width=0.6)
    ax.axhline(50, color='red', linestyle='--', linewidth=2, label='50% Threshold')

    ax.set_xlabel('Minority Tolerance (ubvec)', fontsize=11)
    ax.set_ylabel('Maximum Minority Percentage (%)', fontsize=11)
    ax.set_xticks(range(len(ubvec_labels)))
    ax.set_xticklabels(ubvec_labels, fontsize=10)
    ax.set_ylim(40, 55)
    ax.set_title('(B) Max Minority % vs Tolerance', fontsize=12, fontweight='bold')
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig('results/figure3_constraint_conflict.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure3_constraint_conflict.png', dpi=300, bbox_inches='tight')
    plt.close()

    print("  Alabama ubvec results:")
    for _, row in al_multi.iterrows():
        print(f"    [{row['ubvec_pop']}, {row['ubvec_minority']}]: {row['mm_count']}/2 MM, {row['max_minority_pct']:.1%}")
    print("  Saved: figure3_constraint_conflict.pdf/png")
    print()


# ==============================================================================
# Figure 5: Parameter Sensitivity
# ==============================================================================

def generate_figure5():
    """Parameter sensitivity analysis"""
    print("Generating Figure 5: Parameter Sensitivity...")

    fig = plt.figure(figsize=(14, 10))
    gs = GridSpec(2, 2, figure=fig, hspace=0.3, wspace=0.3)

    # Panel A: Multi-constraint minority % by ubvec
    ax = fig.add_subplot(gs[0, :])

    ubvec_values = sorted(df_multi['ubvec_minority'].unique())

    for state_code, state_name in STATES.items():
        state_data = df_multi[df_multi['state'] == state_code].sort_values('ubvec_minority')
        ax.plot(state_data['ubvec_minority'], state_data['max_minority_pct'] * 100,
               marker='o', linewidth=2, markersize=8, label=state_code, alpha=0.8)

    ax.axhline(50, color='red', linestyle='--', linewidth=1, alpha=0.5, label='50% Threshold')
    ax.set_xlabel('Minority Tolerance (ubvec)', fontsize=11)
    ax.set_ylabel('Maximum Minority Percentage (%)', fontsize=11)
    ax.set_title('(A) Multi-Constraint: Maximum Minority % by Tolerance', fontsize=12, fontweight='bold')
    ax.legend(ncol=6, fontsize=9, loc='upper left')
    ax.grid(alpha=0.3)
    ax.set_xticks(ubvec_values)

    # Panel B: Edge-weighted Alabama by weight factor
    ax = fig.add_subplot(gs[1, 0])

    al_edge = df_edge[df_edge['state'] == 'AL']
    weight_factors = sorted(al_edge['edge_weight_scale_factor'].unique())
    thresholds = sorted(al_edge['minority_threshold'].unique())

    for threshold in thresholds:
        thresh_data = al_edge[al_edge['minority_threshold'] == threshold]
        grouped = thresh_data.groupby('edge_weight_scale_factor')['max_minority_pct'].max() * 100
        ax.plot(grouped.index, grouped.values, marker='o', linewidth=2,
               markersize=6, label=f'{int(threshold*100)}% thresh', alpha=0.7)

    ax.axhline(50, color='red', linestyle='--', linewidth=1, alpha=0.5)
    ax.set_xlabel('Edge Weight Scale Factor', fontsize=10)
    ax.set_ylabel('Max Minority % (Alabama)', fontsize=10)
    ax.set_title('(B) Edge-Weighted: Alabama Parameter Sweep', fontsize=11, fontweight='bold')
    ax.legend(fontsize=8)
    ax.grid(alpha=0.3)
    ax.set_xscale('log')

    # Panel C: Multi-constraint success rate by ubvec
    ax = fig.add_subplot(gs[1, 1])

    success_rates = []
    for ubvec in ubvec_values:
        subset = df_multi[df_multi['ubvec_minority'] == ubvec]
        rate = (subset['success'].sum() / len(subset)) * 100
        success_rates.append(rate)

    ax.bar(range(len(ubvec_values)), success_rates, color='#e74c3c', alpha=0.7, width=0.6)
    ax.set_xlabel('Minority Tolerance (ubvec)', fontsize=10)
    ax.set_ylabel('Success Rate (%)', fontsize=10)
    ax.set_title('(C) Multi-Constraint: Success Rate by Tolerance', fontsize=11, fontweight='bold')
    ax.set_xticks(range(len(ubvec_values)))
    ax.set_xticklabels([f'{v}' for v in ubvec_values], fontsize=9)
    ax.set_ylim(0, 100)
    ax.grid(axis='y', alpha=0.3)

    plt.savefig('results/figure5_parameter_sensitivity.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure5_parameter_sensitivity.png', dpi=300, bbox_inches='tight')
    plt.close()

    print("  Multi-constraint success by ubvec:")
    for ubvec, rate in zip(ubvec_values, success_rates):
        print(f"    ubvec={ubvec}: {rate:.0f}%")
    print("  Saved: figure5_parameter_sensitivity.pdf/png")
    print()


# ==============================================================================
# Figure 6: State Details
# ==============================================================================

def generate_figure6():
    """Detailed state-by-state comparison"""
    print("Generating Figure 6: State Details...")

    fig, axes = plt.subplots(1, 5, figsize=(16, 4))

    metrics = ['MM Count', 'Max Minority %', 'Edge Cut']

    for idx, (state_code, state_name) in enumerate(STATES.items()):
        ax = axes[idx]

        # Best configs
        edge_best = df_edge[df_edge['state'] == state_code].sort_values(
            ['mm_count', 'max_minority_pct'], ascending=[False, False]).iloc[0]
        multi_best = df_multi[df_multi['state'] == state_code].sort_values(
            ['mm_count', 'max_minority_pct'], ascending=[False, False]).iloc[0]

        # Normalize values for display
        target = multi_best['target_mm']
        edge_vals = [
            edge_best['mm_count'] / target,
            edge_best['max_minority_pct'],
            1.0 - (edge_best['edge_cut'] / max(edge_best['edge_cut'], multi_best['edge_cut']))
        ]
        multi_vals = [
            multi_best['mm_count'] / target,
            multi_best['max_minority_pct'],
            1.0 - (multi_best['edge_cut'] / max(edge_best['edge_cut'], multi_best['edge_cut']))
        ]

        x = np.arange(len(metrics))
        width = 0.35

        ax.bar(x - width/2, edge_vals, width, label='Edge-Weighted', color='#3498db', alpha=0.8)
        ax.bar(x + width/2, multi_vals, width, label='Multi-Constraint', color='#e74c3c', alpha=0.8)

        ax.set_ylabel('Normalized Score' if idx == 0 else '', fontsize=10)
        ax.set_title(f'{state_code}', fontsize=11, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels(metrics, fontsize=8, rotation=45, ha='right')
        ax.set_ylim(0, 1.2)
        ax.grid(axis='y', alpha=0.3)

        if idx == 0:
            ax.legend(fontsize=8)

    plt.tight_layout()
    plt.savefig('results/figure6_state_details.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure6_state_details.png', dpi=300, bbox_inches='tight')
    plt.close()

    print("  Saved: figure6_state_details.pdf/png")
    print()


# ==============================================================================
# Figure 7: Configuration Robustness
# ==============================================================================

def generate_figure7():
    """Configuration robustness analysis"""
    print("Generating Figure 7: Configuration Robustness...")

    fig, axes = plt.subplots(1, 2, figsize=(12, 4))

    # Panel A: Success rate by state
    ax = axes[0]

    states = list(STATES.keys())
    edge_rates = []
    multi_rates = []

    for state_code in states:
        edge_state = df_edge[df_edge['state'] == state_code]
        multi_state = df_multi[df_multi['state'] == state_code]

        edge_rate = (edge_state['success'].sum() / len(edge_state)) * 100
        multi_rate = (multi_state['success'].sum() / len(multi_state)) * 100

        edge_rates.append(edge_rate)
        multi_rates.append(multi_rate)

    x = np.arange(len(states))
    width = 0.35

    ax.bar(x - width/2, edge_rates, width, label='Edge-Weighted', color='#3498db', alpha=0.8)
    ax.bar(x + width/2, multi_rates, width, label='Multi-Constraint', color='#e74c3c', alpha=0.8)

    ax.set_ylabel('Configuration Success Rate (%)', fontsize=11)
    ax.set_xlabel('State', fontsize=11)
    ax.set_title('(A) Success Rate by State', fontsize=12, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(states, fontsize=10)
    ax.set_ylim(0, 110)
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    # Panel B: Distribution of MM counts
    ax = axes[1]

    edge_mm = df_edge['mm_count'].values
    multi_mm = df_multi['mm_count'].values

    bins = np.arange(0, max(edge_mm.max(), multi_mm.max()) + 2) - 0.5

    ax.hist(edge_mm, bins=bins, alpha=0.6, label='Edge-Weighted', color='#3498db', edgecolor='black')
    ax.hist(multi_mm, bins=bins, alpha=0.6, label='Multi-Constraint', color='#e74c3c', edgecolor='black')

    ax.set_xlabel('MM Districts Achieved', fontsize=11)
    ax.set_ylabel('Number of Configurations', fontsize=11)
    ax.set_title('(B) Distribution of MM District Counts', fontsize=12, fontweight='bold')
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig('results/figure7_robustness.pdf', dpi=300, bbox_inches='tight')
    plt.savefig('results/figure7_robustness.png', dpi=300, bbox_inches='tight')
    plt.close()

    print("  Success rates by state:")
    for state, edge, multi in zip(states, edge_rates, multi_rates):
        print(f"    {state}: Edge={edge:.0f}%, Multi={multi:.0f}%")
    print("  Saved: figure7_robustness.pdf/png")
    print()


# ==============================================================================
# Main
# ==============================================================================

if __name__ == '__main__':
    print("="*70)
    print("REGENERATING ALL FIGURES WITH CORRECTED DATA")
    print("="*70)
    print()

    generate_figure1()
    generate_figure2()
    generate_figure3()
    generate_figure5()
    generate_figure6()
    generate_figure7()

    print("="*70)
    print("ALL FIGURES REGENERATED SUCCESSFULLY")
    print("="*70)
    print()
    print("Generated files:")
    print("  - figure1_success_rates.pdf/png")
    print("  - figure2_compactness_tradeoff.pdf/png")
    print("  - figure3_constraint_conflict.pdf/png")
    print("  - figure5_parameter_sensitivity.pdf/png")
    print("  - figure6_state_details.pdf/png")
    print("  - figure7_robustness.pdf/png")
