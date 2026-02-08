"""
Generate publication-quality figures for Paper 4: Multi-Constraint vs Edge-Weighted
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# Set publication-quality style
plt.rcParams.update({
    'font.size': 11,
    'font.family': 'serif',
    'axes.labelsize': 12,
    'axes.titlesize': 13,
    'xtick.labelsize': 10,
    'ytick.labelsize': 10,
    'legend.fontsize': 10,
    'figure.titlesize': 14,
    'figure.dpi': 300,
    'savefig.dpi': 300,
    'savefig.bbox': 'tight',
})

# Load data
results_dir = Path(__file__).parent / "results"
multi_df = pd.read_csv(results_dir / "multi_constraint_results.csv")
edge_df = pd.read_csv("research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv")

# State info
STATE_INFO = {
    'AL': {'name': 'Alabama', 'k': 7, 'target': 2, 'minority_pct': 36.9},
    'GA': {'name': 'Georgia', 'k': 14, 'target': 5, 'minority_pct': 49.9},
    'LA': {'name': 'Louisiana', 'k': 6, 'target': 2, 'minority_pct': 44.2},
    'MS': {'name': 'Mississippi', 'k': 4, 'target': 2, 'minority_pct': 44.6},
    'SC': {'name': 'South Carolina', 'k': 7, 'target': 3, 'minority_pct': 37.9},
}

# ============================================================================
# Figure 1: Success Rate Comparison
# ============================================================================

def create_figure1():
    """Success rate comparison - configuration level and state level"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    # Panel A: Configuration-Level Success Rate
    methods = ['Edge-Weighted', 'Multi-Constraint']
    config_success_rates = [
        (edge_df['success'].sum() / len(edge_df)) * 100,  # Edge-weighted
        (multi_df['success'].sum() / len(multi_df)) * 100   # Multi-constraint
    ]

    colors = ['#2E86AB', '#A23B72']  # Professional blue and red
    bars1 = ax1.bar(methods, config_success_rates, color=colors, alpha=0.8, edgecolor='black', linewidth=1.5)

    # Add value labels on bars
    for bar, rate in zip(bars1, config_success_rates):
        height = bar.get_height()
        ax1.text(bar.get_x() + bar.get_width()/2., height,
                f'{rate:.1f}%',
                ha='center', va='bottom', fontweight='bold', fontsize=12)

    ax1.set_ylabel('Configuration Success Rate (%)', fontweight='bold')
    ax1.set_title('(A) Configuration-Level Success Rate', fontweight='bold', pad=10)
    ax1.set_ylim(0, 100)
    ax1.grid(axis='y', alpha=0.3, linestyle='--')
    ax1.set_axisbelow(True)

    # Add sample sizes
    ax1.text(0, -15, f'n={len(edge_df)}', ha='center', fontsize=9, style='italic')
    ax1.text(1, -15, f'n={len(multi_df)}', ha='center', fontsize=9, style='italic')

    # Panel B: State-Level Success Rate (at least one successful config)
    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    edge_state_success = []
    multi_state_success = []

    for state in states:
        # Edge-weighted: at least one config achieves target
        edge_success = edge_df[edge_df['state'] == state]['success'].any()
        edge_state_success.append(1 if edge_success else 0)

        # Multi-constraint: at least one config achieves target
        multi_success = multi_df[multi_df['state'] == state]['success'].any()
        multi_state_success.append(1 if multi_success else 0)

    x = np.arange(len(states))
    width = 0.35

    bars2a = ax2.bar(x - width/2, edge_state_success, width, label='Edge-Weighted',
                     color=colors[0], alpha=0.8, edgecolor='black', linewidth=1.5)
    bars2b = ax2.bar(x + width/2, multi_state_success, width, label='Multi-Constraint',
                     color=colors[1], alpha=0.8, edgecolor='black', linewidth=1.5)

    ax2.set_ylabel('Success (0=Fail, 1=Pass)', fontweight='bold')
    ax2.set_xlabel('State', fontweight='bold')
    ax2.set_title('(B) State-Level Success (≥1 Config Succeeds)', fontweight='bold', pad=10)
    ax2.set_xticks(x)
    ax2.set_xticklabels([STATE_INFO[s]['name'] for s in states], rotation=45, ha='right')
    ax2.set_ylim(0, 1.3)
    ax2.legend(loc='upper right', framealpha=0.9)
    ax2.grid(axis='y', alpha=0.3, linestyle='--')
    ax2.set_axisbelow(True)

    # Add target MM counts as text
    for i, state in enumerate(states):
        target = STATE_INFO[state]['target']
        ax2.text(i, 1.15, f'{target} MM', ha='center', fontsize=8, style='italic')

    plt.tight_layout()
    plt.savefig(results_dir / 'figure1_success_rates.png')
    plt.savefig(results_dir / 'figure1_success_rates.pdf')
    print(f"[OK] Figure 1 saved: {results_dir / 'figure1_success_rates.png'}")
    plt.close()


# ============================================================================
# Figure 2: Compactness vs Minority Concentration Tradeoff
# ============================================================================

def create_figure2():
    """Scatter plot: edge cut vs minority concentration, colored by method"""
    fig, ax = plt.subplots(figsize=(10, 7))

    # Prepare data - get best config per state for each method
    best_results = []

    for state in ['AL', 'GA', 'LA', 'MS', 'SC']:
        target = STATE_INFO[state]['target']

        # Edge-weighted: best config (highest MM count, then highest max_minority_pct)
        edge_state = edge_df[edge_df['state'] == state].copy()
        edge_state = edge_state.sort_values(['mm_count', 'max_minority_pct'], ascending=False)
        edge_best = edge_state.iloc[0]

        best_results.append({
            'state': state,
            'method': 'Edge-Weighted',
            'mm_count': edge_best['mm_count'],
            'target_mm': target,
            'max_minority_pct': edge_best['max_minority_pct'] * 100,
            'edge_cut': edge_best['edge_cut_unweighted'],
            'success': edge_best['success']
        })

        # Multi-constraint: best config (highest MM count, then highest max_minority_pct)
        multi_state = multi_df[multi_df['state'] == state].copy()
        multi_state = multi_state.sort_values(['mm_count', 'max_minority_pct'], ascending=False)
        multi_best = multi_state.iloc[0]

        best_results.append({
            'state': state,
            'method': 'Multi-Constraint',
            'mm_count': multi_best['mm_count'],
            'target_mm': target,
            'max_minority_pct': multi_best['max_minority_pct'] * 100,
            'edge_cut': multi_best['edge_cut'],
            'success': multi_best['success']
        })

    df = pd.DataFrame(best_results)

    # Plot
    colors = {'Edge-Weighted': '#2E86AB', 'Multi-Constraint': '#A23B72'}
    markers = {'Edge-Weighted': 'o', 'Multi-Constraint': 's'}

    for method in ['Edge-Weighted', 'Multi-Constraint']:
        method_df = df[df['method'] == method]

        # Separate successful and failed configs
        success_df = method_df[method_df['success']]
        fail_df = method_df[~method_df['success']]

        # Plot successful (filled markers)
        if len(success_df) > 0:
            ax.scatter(success_df['edge_cut'], success_df['max_minority_pct'],
                      c=colors[method], marker=markers[method], s=200, alpha=0.8,
                      edgecolors='black', linewidth=1.5, label=f'{method} (Success)',
                      zorder=3)

        # Plot failed (hollow markers)
        if len(fail_df) > 0:
            ax.scatter(fail_df['edge_cut'], fail_df['max_minority_pct'],
                      facecolors='none', edgecolors=colors[method], marker=markers[method],
                      s=200, linewidth=2, label=f'{method} (Fail)', alpha=0.6, zorder=2)

    # Add state labels
    for _, row in df.iterrows():
        offset_x = 15 if row['method'] == 'Edge-Weighted' else -15
        offset_y = 1.5 if row['method'] == 'Edge-Weighted' else -1.5
        ax.annotate(row['state'],
                   (row['edge_cut'], row['max_minority_pct']),
                   xytext=(offset_x, offset_y), textcoords='offset points',
                   fontsize=8, alpha=0.7, style='italic')

    # 50% threshold line
    ax.axhline(y=50, color='red', linestyle='--', linewidth=1.5, alpha=0.5,
               label='50% Majority-Minority Threshold', zorder=1)

    ax.set_xlabel('Edge Cut (Compactness)', fontweight='bold', fontsize=12)
    ax.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold', fontsize=12)
    ax.set_title('Compactness vs Minority Concentration Tradeoff',
                 fontweight='bold', fontsize=14, pad=15)
    ax.legend(loc='best', framealpha=0.95, fontsize=9)
    ax.grid(alpha=0.3, linestyle='--')
    ax.set_axisbelow(True)

    plt.tight_layout()
    plt.savefig(results_dir / 'figure2_compactness_tradeoff.png')
    plt.savefig(results_dir / 'figure2_compactness_tradeoff.pdf')
    print(f"[OK] Figure 2 saved: {results_dir / 'figure2_compactness_tradeoff.png'}")
    plt.close()


# ============================================================================
# Figure 3: Constraint Conflict Test (Alabama ubvec sweep)
# ============================================================================

def create_figure3():
    """Constraint conflict test: Alabama with varying minority tolerance"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    # Alabama only
    al_df = multi_df[multi_df['state'] == 'AL'].copy()
    al_df = al_df.sort_values('ubvec_minority')

    ubvec_labels = ['1.3\n(±30%)', '1.5\n(±50%)', '2.0\n(±100%)', '5.0\n(±400%)']

    # Panel A: MM Count vs Minority Tolerance
    colors_gradient = ['#8B0000', '#CD5C5C', '#F08080', '#FFA07A']
    bars1 = ax1.bar(range(len(al_df)), al_df['mm_count'],
                    color=colors_gradient, alpha=0.8, edgecolor='black', linewidth=1.5)

    # Add target line
    ax1.axhline(y=2, color='green', linestyle='--', linewidth=2,
                label='Target (2 MM Districts)', alpha=0.7)

    # Value labels
    for i, (bar, count) in enumerate(zip(bars1, al_df['mm_count'])):
        height = bar.get_height()
        label = f'{int(count)}/2'
        color = 'green' if count >= 2 else 'red'
        ax1.text(bar.get_x() + bar.get_width()/2., height + 0.05,
                label, ha='center', va='bottom', fontweight='bold',
                fontsize=11, color=color)

    ax1.set_xticks(range(len(al_df)))
    ax1.set_xticklabels(ubvec_labels)
    ax1.set_xlabel('Minority Constraint (ubvec[1])', fontweight='bold')
    ax1.set_ylabel('Majority-Minority Districts Achieved', fontweight='bold')
    ax1.set_title('(A) MM District Count vs Minority Tolerance', fontweight='bold', pad=10)
    ax1.set_ylim(0, 2.5)
    ax1.legend(loc='upper left')
    ax1.grid(axis='y', alpha=0.3, linestyle='--')
    ax1.set_axisbelow(True)

    # Panel B: Max Minority % vs Minority Tolerance
    max_minority_pcts = al_df['max_minority_pct'] * 100

    ax2.plot(range(len(al_df)), max_minority_pcts, marker='o', markersize=10,
             linewidth=2.5, color='#2E86AB', markerfacecolor='#2E86AB',
             markeredgecolor='black', markeredgewidth=1.5, alpha=0.8)

    # 50% threshold line
    ax2.axhline(y=50, color='red', linestyle='--', linewidth=2,
                label='50% MM Threshold', alpha=0.7)

    # Value labels
    for i, pct in enumerate(max_minority_pcts):
        ax2.text(i, pct + 1, f'{pct:.1f}%', ha='center', va='bottom',
                fontsize=9, fontweight='bold')

    ax2.set_xticks(range(len(al_df)))
    ax2.set_xticklabels(ubvec_labels)
    ax2.set_xlabel('Minority Constraint (ubvec[1])', fontweight='bold')
    ax2.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold')
    ax2.set_title('(B) Minority Concentration vs Tolerance', fontweight='bold', pad=10)
    ax2.set_ylim(45, 52)
    ax2.legend(loc='lower right')
    ax2.grid(alpha=0.3, linestyle='--')
    ax2.set_axisbelow(True)

    # Overall title
    fig.suptitle('Alabama Constraint Conflict Test: Even 400% Minority Tolerance Fails',
                 fontsize=14, fontweight='bold', y=1.02)

    plt.tight_layout()
    plt.savefig(results_dir / 'figure3_constraint_conflict.png')
    plt.savefig(results_dir / 'figure3_constraint_conflict.pdf')
    print(f"[OK] Figure 3 saved: {results_dir / 'figure3_constraint_conflict.png'}")
    plt.close()


# ============================================================================
# Figure 4: State-by-State Heatmap
# ============================================================================

def create_figure4():
    """Heatmap showing MM count for all state × method combinations"""
    fig, ax = plt.subplots(figsize=(10, 6))

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = [STATE_INFO[s]['name'] for s in states]

    # Create matrix: rows = states, columns = methods
    data = []
    annotations = []

    for state in states:
        target = STATE_INFO[state]['target']

        # Edge-weighted best
        edge_state = edge_df[edge_df['state'] == state]
        edge_best_mm = edge_state['mm_count'].max()

        # Multi-constraint best
        multi_state = multi_df[multi_df['state'] == state]
        multi_best_mm = multi_state['mm_count'].max()

        data.append([edge_best_mm, multi_best_mm])
        annotations.append([
            f'{edge_best_mm}/{target}',
            f'{multi_best_mm}/{target}'
        ])

    data = np.array(data)

    # Create heatmap
    im = ax.imshow(data, cmap='RdYlGn', aspect='auto', vmin=0, vmax=8)

    # Set ticks
    ax.set_xticks([0, 1])
    ax.set_xticklabels(['Edge-Weighted', 'Multi-Constraint'], fontweight='bold')
    ax.set_yticks(range(len(states)))
    ax.set_yticklabels(state_names, fontweight='bold')

    # Add text annotations
    for i in range(len(states)):
        for j in range(2):
            text = ax.text(j, i, annotations[i][j],
                          ha="center", va="center", color="black",
                          fontsize=14, fontweight='bold')

    # Colorbar
    cbar = plt.colorbar(im, ax=ax)
    cbar.set_label('MM Districts Achieved', rotation=270, labelpad=20, fontweight='bold')

    ax.set_title('Best MM District Count by State and Method',
                 fontweight='bold', fontsize=14, pad=15)

    plt.tight_layout()
    plt.savefig(results_dir / 'figure4_heatmap.png')
    plt.savefig(results_dir / 'figure4_heatmap.pdf')
    print(f"[OK] Figure 4 saved: {results_dir / 'figure4_heatmap.png'}")
    plt.close()


# ============================================================================
# Main
# ============================================================================

if __name__ == '__main__':
    print("\nGenerating figures for Paper 4: Multi-Constraint vs Edge-Weighted\n")
    print("=" * 70)

    # Create results directory if needed
    results_dir.mkdir(exist_ok=True)

    # Generate all figures
    create_figure1()
    create_figure2()
    create_figure3()
    create_figure4()

    print("=" * 70)
    print("\nAll figures generated successfully!")
    print(f"\nOutput directory: {results_dir}/")
    print("\nFigures created:")
    print("  - figure1_success_rates.png/pdf")
    print("  - figure2_compactness_tradeoff.png/pdf")
    print("  - figure3_constraint_conflict.png/pdf")
    print("  - figure4_heatmap.png/pdf")
