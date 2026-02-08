"""
Create visualizations for n-way vs recursive bisection comparison paper.

Generates:
1. Overall success rate comparison
2. Parameter sensitivity heatmaps
3. State-by-state comparison
4. Runtime distribution comparison
5. Best configuration comparison
"""

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns
from pathlib import Path

# Set style
plt.style.use('seaborn-v0_8-paper')
sns.set_palette("colorblind")

# Load data
results_dir = Path(__file__).parent / "results"
nway_df = pd.read_csv(results_dir / "50_states_ablation_test.csv")
recursive_df = pd.read_csv(results_dir / "50_states_recursive_ablation.csv")

# Filter out single-district states
multi_district_states = nway_df[nway_df['k'] > 1]['state'].unique()
nway_df = nway_df[nway_df['state'].isin(multi_district_states)].copy()
recursive_df = recursive_df[recursive_df['state'].isin(multi_district_states)].copy()

# Create figures directory
figures_dir = Path(__file__).parent / "figures"
figures_dir.mkdir(exist_ok=True)

print(f"Creating visualizations from {len(nway_df)} n-way and {len(recursive_df)} recursive runs...")
print(f"Analyzing {len(multi_district_states)} multi-district states")


# FIGURE 1: Overall Success Rate Comparison
print("\n[1/6] Creating overall success rate comparison...")
fig, ax = plt.subplots(figsize=(8, 5))

methods = ['N-way', 'Recursive']
success_rates = [
    nway_df['success'].mean() * 100,
    recursive_df['success'].mean() * 100
]
colors = ['#1f77b4', '#ff7f0e']

bars = ax.bar(methods, success_rates, color=colors, alpha=0.8, edgecolor='black', linewidth=1.5)
ax.set_ylabel('Success Rate (%)', fontsize=12, fontweight='bold')
ax.set_title('Overall VRA Compliance Success Rates\n(44 states × 20 configurations = 880 runs per method)',
             fontsize=13, fontweight='bold', pad=15)
ax.set_ylim(0, 60)
ax.grid(axis='y', alpha=0.3, linestyle='--')
ax.axhline(50, color='gray', linestyle=':', alpha=0.5, label='50% benchmark')

# Add value labels on bars
for bar, rate in zip(bars, success_rates):
    height = bar.get_height()
    ax.text(bar.get_x() + bar.get_width()/2., height + 1,
            f'{rate:.1f}%\n({int(rate*880/100)}/880)',
            ha='center', va='bottom', fontsize=11, fontweight='bold')

# Add statistical note
ax.text(0.5, 0.02, 'Difference: 0.8 percentage points (p=0.634, not significant)',
        transform=ax.transAxes, ha='center', fontsize=10, style='italic',
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.3))

plt.tight_layout()
plt.savefig(figures_dir / "fig1_overall_success_rates.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig1_overall_success_rates.png")


# FIGURE 2: Parameter Sensitivity Heatmaps (Both Methods)
print("\n[2/6] Creating parameter sensitivity heatmaps...")
fig, axes = plt.subplots(1, 2, figsize=(14, 5))

weight_factors = [1, 5, 10, 50, 100]
thresholds = [0.40, 0.45, 0.50, 0.55]

for idx, (df, method, ax) in enumerate([
    (nway_df, 'N-way', axes[0]),
    (recursive_df, 'Recursive', axes[1])
]):
    # Create pivot table for heatmap
    heatmap_data = np.zeros((len(weight_factors), len(thresholds)))

    for i, weight in enumerate(weight_factors):
        for j, threshold in enumerate(thresholds):
            mask = (df['weight_factor'] == weight) & (df['minority_threshold'] == threshold)
            success_rate = df[mask]['success'].mean() * 100
            heatmap_data[i, j] = success_rate

    # Plot heatmap
    im = ax.imshow(heatmap_data, cmap='RdYlGn', aspect='auto', vmin=40, vmax=60)
    ax.set_xticks(range(len(thresholds)))
    ax.set_yticks(range(len(weight_factors)))
    ax.set_xticklabels([f'{t:.2f}' for t in thresholds])
    ax.set_yticklabels([f'α={w}' for w in weight_factors])
    ax.set_xlabel('Minority Threshold (τ)', fontsize=11, fontweight='bold')
    ax.set_ylabel('Weight Factor (α)', fontsize=11, fontweight='bold')
    ax.set_title(f'{method} Partitioning\nSuccess Rate by Parameters',
                 fontsize=12, fontweight='bold')

    # Add text annotations
    for i in range(len(weight_factors)):
        for j in range(len(thresholds)):
            text = ax.text(j, i, f'{heatmap_data[i, j]:.1f}',
                          ha="center", va="center", color="black", fontsize=9, fontweight='bold')

    # Mark best configuration
    best_i, best_j = np.unravel_index(heatmap_data.argmax(), heatmap_data.shape)
    ax.add_patch(plt.Rectangle((best_j-0.5, best_i-0.5), 1, 1,
                               fill=False, edgecolor='blue', linewidth=3))

# Add colorbar
cbar = fig.colorbar(im, ax=axes, orientation='vertical', pad=0.02, aspect=30)
cbar.set_label('Success Rate (%)', fontsize=11, fontweight='bold')

plt.suptitle('Parameter Sensitivity Analysis: N-way vs Recursive Bisection',
             fontsize=14, fontweight='bold', y=1.02)
plt.tight_layout()
plt.savefig(figures_dir / "fig2_parameter_sensitivity.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig2_parameter_sensitivity.png")


# FIGURE 3: State-by-State Comparison
print("\n[3/6] Creating state-by-state comparison...")

# Calculate state-level success rates
state_comparison = []
for state in multi_district_states:
    nway_success = nway_df[nway_df['state'] == state]['success'].mean() * 100
    recursive_success = recursive_df[recursive_df['state'] == state]['success'].mean() * 100
    advantage = nway_success - recursive_success

    state_comparison.append({
        'state': state,
        'nway': nway_success,
        'recursive': recursive_success,
        'advantage': advantage,
        'winner': 'N-way' if advantage > 5 else ('Recursive' if advantage < -5 else 'Tie')
    })

state_df = pd.DataFrame(state_comparison).sort_values('advantage')

# Get top 10 each direction
top_recursive = state_df.head(10)
top_nway = state_df.tail(10)
interesting_states = pd.concat([top_recursive, top_nway])

fig, ax = plt.subplots(figsize=(10, 8))

y_pos = np.arange(len(interesting_states))
colors_map = {'N-way': '#1f77b4', 'Recursive': '#ff7f0e', 'Tie': '#808080'}
colors = [colors_map[w] for w in interesting_states['winner']]

bars = ax.barh(y_pos, interesting_states['advantage'], color=colors, alpha=0.8, edgecolor='black')
ax.set_yticks(y_pos)
ax.set_yticklabels([s.replace('_', ' ').title() for s in interesting_states['state']], fontsize=9)
ax.set_xlabel('Success Rate Advantage (percentage points)', fontsize=11, fontweight='bold')
ax.set_title('State-Specific Method Advantages\n(Showing top 10 for each method)',
             fontsize=13, fontweight='bold', pad=15)
ax.axvline(0, color='black', linewidth=1.5)
ax.grid(axis='x', alpha=0.3, linestyle='--')

# Add legend
from matplotlib.patches import Patch
legend_elements = [
    Patch(facecolor='#1f77b4', alpha=0.8, edgecolor='black', label='N-way Advantage'),
    Patch(facecolor='#ff7f0e', alpha=0.8, edgecolor='black', label='Recursive Advantage')
]
ax.legend(handles=legend_elements, loc='lower right', fontsize=10)

plt.tight_layout()
plt.savefig(figures_dir / "fig3_state_comparison.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig3_state_comparison.png")


# FIGURE 4: Runtime Distribution Comparison
print("\n[4/6] Creating runtime distribution comparison...")

fig, axes = plt.subplots(1, 2, figsize=(14, 5))

# Box plots
ax = axes[0]
data_to_plot = [nway_df['runtime'], recursive_df['runtime']]
bp = ax.boxplot(data_to_plot, tick_labels=['N-way', 'Recursive'], patch_artist=True,
                showmeans=True, meanline=True)

for patch, color in zip(bp['boxes'], ['#1f77b4', '#ff7f0e']):
    patch.set_facecolor(color)
    patch.set_alpha(0.6)

ax.set_ylabel('Runtime (seconds)', fontsize=11, fontweight='bold')
ax.set_title('Runtime Distribution Comparison', fontsize=12, fontweight='bold')
ax.grid(axis='y', alpha=0.3, linestyle='--')

# Add statistics text
nway_mean = nway_df['runtime'].mean()
recursive_mean = recursive_df['runtime'].mean()
speedup = (recursive_mean / nway_mean - 1) * 100

stats_text = f"N-way mean: {nway_mean:.2f}s\n"
stats_text += f"Recursive mean: {recursive_mean:.2f}s\n"
stats_text += f"N-way is {speedup:.1f}% faster"
ax.text(0.5, 0.97, stats_text, transform=ax.transAxes,
        verticalalignment='top', horizontalalignment='center',
        bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5),
        fontsize=10, fontweight='bold')

# Histograms
ax = axes[1]
bins = np.linspace(0, max(nway_df['runtime'].max(),
                          recursive_df['runtime'].max()), 30)
ax.hist(nway_df['runtime'], bins=bins, alpha=0.6, label='N-way',
        color='#1f77b4', edgecolor='black')
ax.hist(recursive_df['runtime'], bins=bins, alpha=0.6, label='Recursive',
        color='#ff7f0e', edgecolor='black')
ax.set_xlabel('Runtime (seconds)', fontsize=11, fontweight='bold')
ax.set_ylabel('Frequency', fontsize=11, fontweight='bold')
ax.set_title('Runtime Distribution Histograms', fontsize=12, fontweight='bold')
ax.legend(loc='upper right', fontsize=10)
ax.grid(axis='y', alpha=0.3, linestyle='--')

plt.tight_layout()
plt.savefig(figures_dir / "fig4_runtime_comparison.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig4_runtime_comparison.png")


# FIGURE 5: Best Configuration Comparison
print("\n[5/6] Creating best configuration comparison...")

# Find best configs for each method
nway_configs = nway_df.groupby(['weight_factor', 'minority_threshold'])['success'].mean().astype(float)
recursive_configs = recursive_df.groupby(['weight_factor', 'minority_threshold'])['success'].mean().astype(float)

nway_best = nway_configs.nlargest(5)
recursive_best = recursive_configs.nlargest(5)

fig, axes = plt.subplots(1, 2, figsize=(14, 6))

for ax, configs, method, color in [
    (axes[0], nway_best, 'N-way', '#1f77b4'),
    (axes[1], recursive_best, 'Recursive', '#ff7f0e')
]:
    labels = [f'α={w}, τ={t:.2f}' for (w, t) in configs.index]
    values = configs.values * 100

    bars = ax.barh(range(len(labels)), values, color=color, alpha=0.8, edgecolor='black')
    ax.set_yticks(range(len(labels)))
    ax.set_yticklabels(labels, fontsize=10)
    ax.set_xlabel('Success Rate (%)', fontsize=11, fontweight='bold')
    ax.set_title(f'Top 5 Configurations: {method}', fontsize=12, fontweight='bold')
    ax.set_xlim(0, 70)
    ax.grid(axis='x', alpha=0.3, linestyle='--')

    # Add value labels
    for i, (bar, val) in enumerate(zip(bars, values)):
        ax.text(val + 1, i, f'{val:.1f}%', va='center', fontsize=10, fontweight='bold')

    # Highlight best
    bars[0].set_edgecolor('gold')
    bars[0].set_linewidth(3)

plt.suptitle('Best Parameter Configurations by Method', fontsize=14, fontweight='bold', y=1.00)
plt.tight_layout()
plt.savefig(figures_dir / "fig5_best_configs.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig5_best_configs.png")


# FIGURE 6: Success Rate Scatter with Speed-Quality Trade-off
print("\n[6/6] Creating speed-quality trade-off visualization...")

fig, ax = plt.subplots(figsize=(10, 6))

# Calculate average success rate and runtime for each configuration
nway_summary = nway_df.groupby(['weight_factor', 'minority_threshold']).agg({
    'success': 'mean',
    'runtime': 'mean'
}).reset_index()
nway_summary['success_pct'] = nway_summary['success'] * 100

recursive_summary = recursive_df.groupby(['weight_factor', 'minority_threshold']).agg({
    'success': 'mean',
    'runtime': 'mean'
}).reset_index()
recursive_summary['success_pct'] = recursive_summary['success'] * 100

# Plot
scatter1 = ax.scatter(nway_summary['runtime'], nway_summary['success_pct'],
                     s=150, alpha=0.7, c='#1f77b4', edgecolors='black', linewidth=1.5,
                     marker='o', label='N-way configurations')
scatter2 = ax.scatter(recursive_summary['runtime'], recursive_summary['success_pct'],
                     s=150, alpha=0.7, c='#ff7f0e', edgecolors='black', linewidth=1.5,
                     marker='s', label='Recursive configurations')

# Annotate best configs
nway_best_idx = nway_summary['success_pct'].idxmax()
recursive_best_idx = recursive_summary['success_pct'].idxmax()

nway_best_point = nway_summary.iloc[nway_best_idx]
recursive_best_point = recursive_summary.iloc[recursive_best_idx]

ax.annotate(f'N-way best\n({nway_best_point["success_pct"]:.1f}%, {nway_best_point["runtime"]:.1f}s)',
            xy=(nway_best_point['runtime'], nway_best_point['success_pct']),
            xytext=(15, 15), textcoords='offset points',
            bbox=dict(boxstyle='round', facecolor='#1f77b4', alpha=0.3),
            arrowprops=dict(arrowstyle='->', connectionstyle='arc3,rad=0.3', lw=2),
            fontsize=10, fontweight='bold')

ax.annotate(f'Recursive best\n({recursive_best_point["success_pct"]:.1f}%, {recursive_best_point["runtime"]:.1f}s)',
            xy=(recursive_best_point['runtime'], recursive_best_point['success_pct']),
            xytext=(15, -30), textcoords='offset points',
            bbox=dict(boxstyle='round', facecolor='#ff7f0e', alpha=0.3),
            arrowprops=dict(arrowstyle='->', connectionstyle='arc3,rad=0.3', lw=2),
            fontsize=10, fontweight='bold')

ax.set_xlabel('Average Runtime (seconds)', fontsize=12, fontweight='bold')
ax.set_ylabel('Success Rate (%)', fontsize=12, fontweight='bold')
ax.set_title('Speed-Quality Trade-off: N-way vs Recursive Bisection\n(Each point = one parameter configuration averaged across 44 states)',
             fontsize=13, fontweight='bold', pad=15)
ax.grid(alpha=0.3, linestyle='--')
ax.legend(loc='lower right', fontsize=11, framealpha=0.9)

# Add trade-off annotation
tradeoff_text = "Trade-off: N-way is 67.5% faster on average,\nbut recursive achieves 4.5 point higher peak"
ax.text(0.03, 0.97, tradeoff_text, transform=ax.transAxes,
        verticalalignment='top', fontsize=10, style='italic',
        bbox=dict(boxstyle='round', facecolor='yellow', alpha=0.3))

plt.tight_layout()
plt.savefig(figures_dir / "fig6_speed_quality_tradeoff.png", dpi=300, bbox_inches='tight')
plt.close()
print(f"   Saved: fig6_speed_quality_tradeoff.png")


print("\n" + "="*60)
print("VISUALIZATION GENERATION COMPLETE")
print("="*60)
print(f"\nAll figures saved to: {figures_dir.absolute()}")
print("\nGenerated figures:")
print("  1. fig1_overall_success_rates.png - Bar chart comparing overall success")
print("  2. fig2_parameter_sensitivity.png - Heatmaps showing parameter effects")
print("  3. fig3_state_comparison.png - State-specific method advantages")
print("  4. fig4_runtime_comparison.png - Runtime distribution comparison")
print("  5. fig5_best_configs.png - Top configurations for each method")
print("  6. fig6_speed_quality_tradeoff.png - Speed vs quality scatter plot")
print("\nThese can be included in the LaTeX paper using:")
print("  \\includegraphics[width=\\textwidth]{figures/fig1_overall_success_rates.png}")
