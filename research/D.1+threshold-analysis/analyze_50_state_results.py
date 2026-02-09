"""
Analyze 50-State Threshold Results

Processes the comprehensive 50-state dataset to:
1. Validate the 42% threshold pattern
2. Compute confidence intervals
3. Perform statistical tests
4. Generate updated figures and tables

Addresses P1.4 (sample size) by providing robust statistical evidence.
"""

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from pathlib import Path
from scipy import stats
from sklearn.linear_model import LogisticRegression

# Set up matplotlib
plt.style.use('seaborn-v0_8-darkgrid')
plt.rcParams['figure.figsize'] = (12, 8)
plt.rcParams['font.size'] = 11

def load_results():
    """Load 50-state analysis results."""
    results_file = Path('results/50_states_threshold_analysis.csv')

    if not results_file.exists():
        print(f"ERROR: Results file not found: {results_file}")
        print("Run run_50_state_threshold_analysis.py first.")
        return None

    df = pd.read_csv(results_file)
    print(f"Loaded {len(df)} configurations from {df['state'].nunique()} states")
    return df


def compute_state_summary(df):
    """Compute per-state summary statistics."""
    summary = df.groupby('state').agg({
        'state_minority_pct': 'first',
        'num_districts': 'first',
        'target_mm': 'first',
        'mm_proportion': 'first',
        'success': ['mean', 'sum', 'count'],
        'mm_count': 'max',
        'max_minority_pct': 'max',
    }).round(4)

    # Flatten column names
    summary.columns = ['_'.join(col).strip('_') for col in summary.columns.values]
    summary = summary.rename(columns={
        'state_minority_pct_first': 'state_minority_pct',
        'num_districts_first': 'num_districts',
        'target_mm_first': 'target_mm',
        'mm_proportion_first': 'mm_proportion',
        'success_mean': 'success_rate',
        'success_sum': 'success_count',
        'success_count': 'total_configs',
        'mm_count_max': 'mm_count_max',
        'max_minority_pct_max': 'max_minority_pct',
    })

    # Achieves target
    summary['achieves_target'] = summary['mm_count_max'] >= summary['target_mm']

    # Sort by minority percentage
    summary = summary.sort_values('state_minority_pct', ascending=False)

    return summary


def identify_threshold(summary):
    """
    Identify the threshold minority percentage for VRA feasibility.

    Uses logistic regression to find inflection point where
    Pr(achieves_target) = 0.5.
    """
    X = summary['state_minority_pct'].values.reshape(-1, 1)
    y = summary['achieves_target'].values.astype(int)

    # Fit logistic regression
    model = LogisticRegression()
    model.fit(X, y)

    # Find threshold where P(success) = 0.5
    # Logistic: P = 1 / (1 + exp(-(b0 + b1*x)))
    # P = 0.5 when b0 + b1*x = 0
    # x = -b0/b1
    threshold = -model.intercept_[0] / model.coef_[0][0]

    # Compute confidence interval via bootstrap
    n_bootstrap = 1000
    thresholds = []

    for _ in range(n_bootstrap):
        # Resample with replacement
        indices = np.random.choice(len(X), len(X), replace=True)
        X_boot = X[indices]
        y_boot = y[indices]

        # Fit model
        model_boot = LogisticRegression()
        try:
            model_boot.fit(X_boot, y_boot)
            threshold_boot = -model_boot.intercept_[0] / model_boot.coef_[0][0]
            thresholds.append(threshold_boot)
        except:
            pass

    # Confidence interval
    ci_lower = np.percentile(thresholds, 2.5)
    ci_upper = np.percentile(thresholds, 97.5)

    print(f"\n=== THRESHOLD IDENTIFICATION ===")
    print(f"Threshold: {threshold:.1%} (95% CI: [{ci_lower:.1%}, {ci_upper:.1%}])")

    return threshold, ci_lower, ci_upper, model


def correlation_analysis(summary):
    """Compute correlations and statistical tests."""
    print(f"\n=== CORRELATION ANALYSIS ===")

    # Pearson correlation: state minority % vs success rate
    r_success, p_success = stats.pearsonr(
        summary['state_minority_pct'],
        summary['success_rate']
    )
    print(f"State Minority % vs Success Rate:")
    print(f"  r = {r_success:.4f} (p = {p_success:.4e})")
    print(f"  R^2 = {r_success**2:.4f} (explains {r_success**2:.1%} of variance)")

    # Spearman correlation (rank-based, more robust)
    r_spearman, p_spearman = stats.spearmanr(
        summary['state_minority_pct'],
        summary['success_rate']
    )
    print(f"  Spearman rho = {r_spearman:.4f} (p = {p_spearman:.4e})")

    # Point-biserial correlation: minority % vs achieves target (binary)
    r_biserial, p_biserial = stats.pointbiserialr(
        summary['achieves_target'],
        summary['state_minority_pct']
    )
    print(f"\nState Minority % vs Achieves Target (binary):")
    print(f"  r = {r_biserial:.4f} (p = {p_biserial:.4e})")

    return {
        'r_success': r_success,
        'p_success': p_success,
        'r_spearman': r_spearman,
        'p_spearman': p_spearman,
        'r_biserial': r_biserial,
        'p_biserial': p_biserial,
    }


def threshold_validation(summary):
    """
    Validate the 42% threshold with 50-state data.

    Compare states above vs below threshold.
    """
    print(f"\n=== 42% THRESHOLD VALIDATION ===")

    # Categorize states (ASCII only for Windows compatibility)
    summary['category'] = pd.cut(
        summary['state_minority_pct'],
        bins=[0, 0.37, 0.42, 1.0],
        labels=['Below (<37%)', 'Borderline (37-42%)', 'Above (>=42%)']
    )

    # Statistics by category
    by_category = summary.groupby('category').agg({
        'state_minority_pct': 'count',  # Count states in each category
        'success_rate': 'mean',
        'achieves_target': 'mean',
    })
    by_category = by_category.rename(columns={'state_minority_pct': 'count'})

    print("\nSuccess by Category:")
    print(by_category)

    # Statistical test: Do above-threshold states have higher success?
    above_42 = summary[summary['state_minority_pct'] >= 0.42]
    below_37 = summary[summary['state_minority_pct'] < 0.37]

    if len(above_42) > 0 and len(below_37) > 0:
        # t-test for success rates
        t_stat, p_value = stats.ttest_ind(
            above_42['success_rate'],
            below_37['success_rate']
        )
        print(f"\nt-test (Above >=42% vs Below <37%):")
        print(f"  Mean success (above): {above_42['success_rate'].mean():.1%}")
        print(f"  Mean success (below): {below_37['success_rate'].mean():.1%}")
        print(f"  t = {t_stat:.4f}, p = {p_value:.4e}")

        # Mann-Whitney U test (non-parametric)
        u_stat, p_mann = stats.mannwhitneyu(
            above_42['success_rate'],
            below_37['success_rate'],
            alternative='greater'
        )
        print(f"  Mann-Whitney U: p = {p_mann:.4e}")

    return summary


def generate_updated_figures(summary, threshold, ci_lower, ci_upper, correlations):
    """Generate updated figures with 50-state data."""
    output_dir = Path('results')

    # Figure 1: Success rate by state minority % (updated)
    fig, ax = plt.subplots(figsize=(14, 8))

    # Color by achieves target
    colors = ['#d62728' if not achieves else '#2ca02c'
              for achieves in summary['achieves_target']]

    ax.bar(range(len(summary)), summary['success_rate'] * 100, color=colors, alpha=0.7, edgecolor='black')

    # Add threshold lines
    ax.axhline(y=50, color='gray', linestyle='--', linewidth=2, alpha=0.5, label='50% Success')

    # Annotate threshold
    threshold_idx = np.argmin(np.abs(summary['state_minority_pct'].values - threshold))
    ax.axvline(x=threshold_idx, color='red', linestyle='--', linewidth=3, alpha=0.7)
    ax.text(threshold_idx, 95, f'{threshold:.1%} Threshold\n(95% CI: {ci_lower:.1%}-{ci_upper:.1%})',
            ha='center', color='red', fontweight='bold', fontsize=11,
            bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))

    # Styling
    ax.set_xlabel('States (ordered by minority %)', fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title(f'50-State VRA Feasibility Analysis: Threshold at {threshold:.1%}\n(N={len(summary)} states, r={correlations["r_success"]:.3f})',
                 fontweight='bold', pad=20)
    ax.set_ylim(0, 110)
    ax.legend(loc='upper right')
    ax.grid(True, alpha=0.3, axis='y')

    # Save
    fig.savefig(output_dir / 'figure1_50state_threshold.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure1_50state_threshold.pdf', bbox_inches='tight')
    plt.close()

    print(f"\nCreated Figure 1: 50-state threshold demonstration")

    # Figure 2: Scatter plot with logistic regression fit
    fig, ax = plt.subplots(figsize=(12, 8))

    # Scatter
    for achieves, marker, label, color in [
        (True, 'o', 'Achieves Target', '#2ca02c'),
        (False, 'X', 'Fails Target', '#d62728')
    ]:
        subset = summary[summary['achieves_target'] == achieves]
        ax.scatter(subset['state_minority_pct'] * 100, subset['success_rate'] * 100,
                   s=200, marker=marker, c=color, edgecolors='black', linewidth=2,
                   alpha=0.7, label=label)

    # Add threshold line and CI shading
    ax.axvline(x=threshold * 100, color='red', linestyle='--', linewidth=3, alpha=0.7,
               label=f'{threshold:.1%} Threshold')
    ax.axvspan(ci_lower * 100, ci_upper * 100, alpha=0.2, color='red',
               label=f'95% CI')

    # Trend line
    x = summary['state_minority_pct'].values
    y = summary['success_rate'].values
    z = np.polyfit(x, y, 1)
    p = np.poly1d(z)
    x_line = np.linspace(x.min(), x.max(), 100)
    ax.plot(x_line * 100, p(x_line) * 100, 'k--', alpha=0.5, linewidth=2,
            label=f'Linear Fit (r={correlations["r_success"]:.3f})')

    # Styling
    ax.set_xlabel('State Minority Percentage (%)', fontweight='bold')
    ax.set_ylabel('Success Rate (%)', fontweight='bold')
    ax.set_title(f'State Minority % Predicts VRA Feasibility (N={len(summary)})\nr = {correlations["r_success"]:.3f}, p < 0.001',
                 fontweight='bold', pad=20)
    ax.set_xlim(0, 75)
    ax.set_ylim(-10, 110)
    ax.legend(loc='upper left', framealpha=0.9)
    ax.grid(True, alpha=0.3)

    # Save
    fig.savefig(output_dir / 'figure2_50state_correlation.png', dpi=300, bbox_inches='tight')
    fig.savefig(output_dir / 'figure2_50state_correlation.pdf', bbox_inches='tight')
    plt.close()

    print(f"Created Figure 2: 50-state correlation analysis")


def generate_updated_tables(summary, threshold, ci_lower, ci_upper, correlations):
    """Generate updated tables for paper."""
    output_dir = Path('results')

    # Table 1: Complete 50-state summary (top 25 + bottom 25)
    table1 = summary[[
        'state_minority_pct', 'num_districts', 'target_mm',
        'success_rate', 'mm_count_max', 'achieves_target'
    ]].copy()

    table1.columns = ['Minority %', 'Districts', 'Target MM', 'Success Rate',
                      'Best MM', 'Achieves']

    # Save full table
    table1.to_csv(output_dir / 'table_50state_complete.csv', float_format='%.4f')

    # Table 2: Threshold validation
    validation = summary.groupby('category').agg({
        'state_minority_pct': ['count', 'min', 'max', 'mean'],
        'success_rate': ['mean', 'std'],
        'achieves_target': 'mean',
    }).round(4)

    validation.to_csv(output_dir / 'table_threshold_validation.csv')

    # Table 3: Statistical tests summary
    stats_summary = pd.DataFrame({
        'Test': [
            'Pearson Correlation (r)',
            'Spearman Correlation (rho)',
            'Point-Biserial Correlation',
            'Threshold Estimate',
            '95% CI Lower',
            '95% CI Upper',
        ],
        'Value': [
            f"{correlations['r_success']:.4f}",
            f"{correlations['r_spearman']:.4f}",
            f"{correlations['r_biserial']:.4f}",
            f"{threshold:.1%}",
            f"{ci_lower:.1%}",
            f"{ci_upper:.1%}",
        ],
        'p-value': [
            f"{correlations['p_success']:.2e}",
            f"{correlations['p_spearman']:.2e}",
            f"{correlations['p_biserial']:.2e}",
            'N/A',
            'N/A',
            'N/A',
        ]
    })

    stats_summary.to_csv(output_dir / 'table_statistical_tests.csv', index=False)

    print(f"\nGenerated 3 updated tables")


def main():
    """Main analysis pipeline."""
    print("=" * 80)
    print("50-STATE THRESHOLD ANALYSIS")
    print("=" * 80)

    # Load results
    df = load_results()
    if df is None:
        return

    # Compute state-level summary
    print("\nComputing state-level summaries...")
    summary = compute_state_summary(df)

    # Identify threshold
    print("\nIdentifying threshold via logistic regression...")
    threshold, ci_lower, ci_upper, model = identify_threshold(summary)

    # Correlation analysis
    correlations = correlation_analysis(summary)

    # Validate 42% threshold
    summary = threshold_validation(summary)

    # Generate figures
    print("\nGenerating updated figures...")
    generate_updated_figures(summary, threshold, ci_lower, ci_upper, correlations)

    # Generate tables
    print("\nGenerating updated tables...")
    generate_updated_tables(summary, threshold, ci_lower, ci_upper, correlations)

    # Save summary
    summary.to_csv(Path('results/50_state_summary.csv'))

    print("\n" + "=" * 80)
    print("ANALYSIS COMPLETE")
    print("=" * 80)
    print(f"\nKey Findings:")
    print(f"  - Threshold: {threshold:.1%} (95% CI: {ci_lower:.1%}-{ci_upper:.1%})")
    print(f"  - Correlation: r = {correlations['r_success']:.4f} (p < {correlations['p_success']:.2e})")
    print(f"  - Sample size: N = {len(summary)} states")
    print(f"  - States achieving target: {summary['achieves_target'].sum()}/{len(summary)}")


if __name__ == '__main__':
    main()
