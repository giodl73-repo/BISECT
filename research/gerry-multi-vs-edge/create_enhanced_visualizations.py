"""
Create enhanced visualizations from existing CSV results
(No need for raw census data)
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
import seaborn as sns

# Publication style
plt.rcParams.update({
    'font.size': 11,
    'font.family': 'serif',
    'axes.labelsize': 12,
    'axes.titlesize': 13,
    'legend.fontsize': 10,
    'figure.dpi': 300,
    'savefig.dpi': 300,
    'savefig.bbox': 'tight',
})

def load_results():
    """Load existing results"""
    multi_df = pd.read_csv("research/gerry-multi-vs-edge/results/multi_constraint_results.csv")
    edge_df = pd.read_csv("research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv")
    return multi_df, edge_df

def create_parameter_sensitivity_figure(multi_df, edge_df, output_path):
    """
    Figure 5: Parameter sensitivity showing how results vary with configuration
    """
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(14, 12))

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = {
        'AL': 'Alabama',
        'GA': 'Georgia',
        'LA': 'Louisiana',
        'MS': 'Mississippi',
        'SC': 'South Carolina'
    }

    # Panel 1: Multi-constraint ubvec sensitivity
    for state in states:
        state_data = multi_df[multi_df['state'] == state].sort_values('ubvec_minority')
        ax1.plot(state_data['ubvec_minority'], state_data['max_minority_pct'] * 100,
                marker='o', label=state_names[state], linewidth=2, markersize=6)

    ax1.axhline(y=50, color='red', linestyle='--', linewidth=1.5, alpha=0.5, label='50% MM Threshold')
    ax1.set_xlabel('Minority Constraint Tolerance (ubvec[1])', fontweight='bold')
    ax1.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold')
    ax1.set_title('(A) Multi-Constraint: Minority % vs Tolerance', fontweight='bold')
    ax1.legend(loc='best', fontsize=9)
    ax1.grid(alpha=0.3)
    ax1.set_xlim(1, 5.5)

    # Panel 2: Edge-weighted weight factor sensitivity (for Alabama)
    al_edge = edge_df[edge_df['state'] == 'AL']
    for threshold in [0.40, 0.45, 0.50, 0.55]:
        thresh_data = al_edge[al_edge['minority_threshold'] == threshold].sort_values('weight_factor')
        ax2.plot(thresh_data['weight_factor'], thresh_data['max_minority_pct'] * 100,
                marker='s', label=f'{int(threshold*100)}% threshold', linewidth=2, markersize=6)

    ax2.axhline(y=50, color='red', linestyle='--', linewidth=1.5, alpha=0.5, label='50% MM Threshold')
    ax2.set_xlabel('Weight Factor (α)', fontweight='bold')
    ax2.set_ylabel('Maximum Minority Percentage (%)', fontweight='bold')
    ax2.set_title('(B) Edge-Weighted (Alabama): Minority % vs Weight', fontweight='bold')
    ax2.legend(loc='best', fontsize=9)
    ax2.grid(alpha=0.3)
    ax2.set_xscale('log')

    # Panel 3: Success rate vs ubvec (multi-constraint)
    ubvec_success = multi_df.groupby('ubvec_minority')['success'].agg(['sum', 'count'])
    ubvec_success['rate'] = (ubvec_success['sum'] / ubvec_success['count']) * 100

    ax3.bar(range(len(ubvec_success)), ubvec_success['rate'],
           color='#A23B72', alpha=0.8, edgecolor='black', linewidth=1.5)
    ax3.set_xticks(range(len(ubvec_success)))
    ax3.set_xticklabels([f'{x:.1f}' for x in ubvec_success.index])
    ax3.set_xlabel('Minority Constraint Tolerance (ubvec[1])', fontweight='bold')
    ax3.set_ylabel('Success Rate (%)', fontweight='bold')
    ax3.set_title('(C) Multi-Constraint: Success Rate vs Tolerance', fontweight='bold')
    ax3.grid(axis='y', alpha=0.3)
    ax3.set_ylim(0, 100)

    # Add value labels
    for i, v in enumerate(ubvec_success['rate']):
        ax3.text(i, v + 2, f'{v:.0f}%', ha='center', va='bottom', fontweight='bold')

    # Panel 4: Success rate vs weight factor (edge-weighted, Alabama)
    al_edge_success = al_edge.groupby('weight_factor')['success'].agg(['sum', 'count'])
    al_edge_success['rate'] = (al_edge_success['sum'] / al_edge_success['count']) * 100

    ax4.bar(range(len(al_edge_success)), al_edge_success['rate'],
           color='#2E86AB', alpha=0.8, edgecolor='black', linewidth=1.5)
    ax4.set_xticks(range(len(al_edge_success)))
    ax4.set_xticklabels([f'{int(x)}' for x in al_edge_success.index])
    ax4.set_xlabel('Weight Factor (α)', fontweight='bold')
    ax4.set_ylabel('Success Rate (%)', fontweight='bold')
    ax4.set_title('(D) Edge-Weighted (Alabama): Success Rate vs Weight', fontweight='bold')
    ax4.grid(axis='y', alpha=0.3)
    ax4.set_ylim(0, 100)

    # Add value labels
    for i, v in enumerate(al_edge_success['rate']):
        ax4.text(i, v + 2, f'{v:.0f}%', ha='center', va='bottom', fontweight='bold')

    fig.suptitle('Parameter Sensitivity Analysis', fontsize=16, fontweight='bold', y=0.995)
    plt.tight_layout(rect=[0, 0, 1, 0.99])

    plt.savefig(output_path)
    plt.savefig(output_path.with_suffix('.pdf'))
    print(f"[OK] Parameter sensitivity figure saved: {output_path}")
    plt.close()

def create_detailed_state_comparison(multi_df, edge_df, output_path):
    """
    Figure 6: Detailed state-by-state comparison with multiple metrics
    """
    fig, axes = plt.subplots(5, 1, figsize=(12, 16))

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = {
        'AL': 'Alabama',
        'GA': 'Georgia',
        'LA': 'Louisiana',
        'MS': 'Mississippi',
        'SC': 'South Carolina'
    }

    for idx, state in enumerate(states):
        ax = axes[idx]

        # Get best configs
        multi_best = multi_df[multi_df['state'] == state].sort_values(
            ['mm_count', 'max_minority_pct'], ascending=False).iloc[0]

        edge_best = edge_df[edge_df['state'] == state].sort_values(
            ['mm_count', 'max_minority_pct'], ascending=False).iloc[0]

        # Metrics to compare
        metrics = ['MM Count', 'Max\nMinority %', 'Edge Cut']
        multi_vals = [multi_best['mm_count'], multi_best['max_minority_pct'] * 100, multi_best['edge_cut']]
        edge_vals = [edge_best['mm_count'], edge_best['max_minority_pct'] * 100, edge_best['edge_cut_unweighted']]

        # Normalize for visualization
        max_mm = max(multi_vals[0], edge_vals[0])
        max_pct = max(multi_vals[1], edge_vals[1])
        max_cut = max(multi_vals[2], edge_vals[2])

        x = np.arange(len(metrics))
        width = 0.35

        bars1 = ax.bar(x - width/2, [multi_vals[0]/max_mm*100, multi_vals[1]/max_pct*100, multi_vals[2]/max_cut*100],
                       width, label='Multi-Constraint', color='#A23B72', alpha=0.8, edgecolor='black', linewidth=1.5)

        bars2 = ax.bar(x + width/2, [edge_vals[0]/max_mm*100, edge_vals[1]/max_pct*100, edge_vals[2]/max_cut*100],
                       width, label='Edge-Weighted', color='#2E86AB', alpha=0.8, edgecolor='black', linewidth=1.5)

        # Add actual values as text
        for i, (m_val, e_val) in enumerate(zip(multi_vals, edge_vals)):
            ax.text(i - width/2, (m_val / [max_mm, max_pct, max_cut][i] * 100) + 3,
                   f'{m_val:.1f}' if i < 2 else f'{int(m_val)}',
                   ha='center', va='bottom', fontsize=9, fontweight='bold')
            ax.text(i + width/2, (e_val / [max_mm, max_pct, max_cut][i] * 100) + 3,
                   f'{e_val:.1f}' if i < 2 else f'{int(e_val)}',
                   ha='center', va='bottom', fontsize=9, fontweight='bold')

        ax.set_ylabel('Normalized Score', fontweight='bold')
        ax.set_title(f'{state_names[state]} (Target: {multi_best["target_mm"]} MM Districts)',
                    fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels(metrics)
        ax.set_ylim(0, 120)
        ax.legend(loc='upper right')
        ax.grid(axis='y', alpha=0.3)
        ax.set_axisbelow(True)

    fig.suptitle('Detailed State-by-State Comparison', fontsize=16, fontweight='bold', y=0.995)
    plt.tight_layout(rect=[0, 0, 1, 0.99])

    plt.savefig(output_path)
    plt.savefig(output_path.with_suffix('.pdf'))
    print(f"[OK] Detailed state comparison saved: {output_path}")
    plt.close()

def create_configuration_robustness_analysis(multi_df, edge_df, output_path):
    """
    Figure 7: Robustness analysis - how many configurations succeed per state
    """
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = {
        'AL': 'Alabama',
        'GA': 'Georgia',
        'LA': 'Louisiana',
        'MS': 'Mississippi',
        'SC': 'South Carolina'
    }

    # Multi-constraint robustness
    multi_success = []
    for state in states:
        state_data = multi_df[multi_df['state'] == state]
        success_rate = state_data['success'].mean() * 100
        multi_success.append(success_rate)

    # Edge-weighted robustness
    edge_success = []
    for state in states:
        state_data = edge_df[edge_df['state'] == state]
        success_rate = state_data['success'].mean() * 100
        edge_success.append(success_rate)

    x = np.arange(len(states))
    width = 0.35

    bars1 = ax1.barh(x + width/2, multi_success, width, label='Multi-Constraint',
                     color='#A23B72', alpha=0.8, edgecolor='black', linewidth=1.5)
    bars2 = ax1.barh(x - width/2, edge_success, width, label='Edge-Weighted',
                     color='#2E86AB', alpha=0.8, edgecolor='black', linewidth=1.5)

    ax1.set_yticks(x)
    ax1.set_yticklabels([state_names[s] for s in states])
    ax1.set_xlabel('Configuration Success Rate (%)', fontweight='bold')
    ax1.set_title('(A) Robustness by State', fontweight='bold')
    ax1.legend(loc='lower right')
    ax1.grid(axis='x', alpha=0.3)
    ax1.set_xlim(0, 100)

    # Add value labels
    for i, (m_val, e_val) in enumerate(zip(multi_success, edge_success)):
        ax1.text(m_val + 2, i + width/2, f'{m_val:.0f}%', ha='left', va='center', fontsize=9)
        ax1.text(e_val + 2, i - width/2, f'{e_val:.0f}%', ha='left', va='center', fontsize=9)

    # Panel B: Distribution of configurations
    ax2.hist([edge_df['mm_count'], multi_df['mm_count']],
            bins=range(0, 10), label=['Edge-Weighted', 'Multi-Constraint'],
            color=['#2E86AB', '#A23B72'], alpha=0.7, edgecolor='black', linewidth=1.5)

    ax2.set_xlabel('MM Districts Created', fontweight='bold')
    ax2.set_ylabel('Number of Configurations', fontweight='bold')
    ax2.set_title('(B) Distribution of MM District Counts', fontweight='bold')
    ax2.legend()
    ax2.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_path)
    plt.savefig(output_path.with_suffix('.pdf'))
    print(f"[OK] Robustness analysis saved: {output_path}")
    plt.close()

def main():
    print("\n" + "="*70)
    print("Creating Enhanced Visualizations from Existing Results")
    print("="*70)

    # Load results
    print("\nLoading results...")
    multi_df, edge_df = load_results()
    print(f"  Multi-constraint: {len(multi_df)} experiments")
    print(f"  Edge-weighted: {len(edge_df)} experiments")

    # Create output directory
    output_dir = Path("research/gerry-multi-vs-edge/results")
    output_dir.mkdir(exist_ok=True, parents=True)

    # Generate figures
    print("\nGenerating enhanced visualizations...")

    print("  [1/3] Parameter sensitivity analysis...")
    create_parameter_sensitivity_figure(
        multi_df, edge_df,
        output_dir / "figure5_parameter_sensitivity.png"
    )

    print("  [2/3] Detailed state comparison...")
    create_detailed_state_comparison(
        multi_df, edge_df,
        output_dir / "figure6_state_details.png"
    )

    print("  [3/3] Robustness analysis...")
    create_configuration_robustness_analysis(
        multi_df, edge_df,
        output_dir / "figure7_robustness.png"
    )

    print("\n" + "="*70)
    print("DONE! Enhanced visualizations created")
    print("="*70)

    print("\nGenerated figures:")
    print("  - figure5_parameter_sensitivity.{png,pdf}")
    print("  - figure6_state_details.{png,pdf}")
    print("  - figure7_robustness.{png,pdf}")

if __name__ == '__main__':
    main()
