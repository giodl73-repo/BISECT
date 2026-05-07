"""
Threshold Definition Analysis - Comparing Different Success Metrics

Analyzes the 50-state data under three different threshold definitions:
1. Feasibility: Creates at least 1 MM district (any success)
2. Effectiveness: Average MM count > 0 (improvement over baseline)
3. Proportionality: Achieves proportional representation target (current)
"""

import pandas as pd
import numpy as np
from pathlib import Path
from scipy import stats
from sklearn.linear_model import LogisticRegression

# Load data
df = pd.read_csv('results/50_states_threshold_analysis.csv')

print("=" * 80)
print("THRESHOLD DEFINITION COMPARISON - 50 STATES")
print("=" * 80)
print()

# Define three success metrics
df['success_feasibility'] = df['mm_count'] >= 1  # Creates at least 1 MM district
df['success_effectiveness'] = df['mm_count'] > 0  # Same as feasibility for this case
df['success_proportionality'] = df['success']  # Already defined (mm_count >= target)

# Compute state-level summaries for each metric
metrics = {
    'Feasibility (>=1 MM)': 'success_feasibility',
    'Proportionality (>=target)': 'success_proportionality'
}

results = {}

for metric_name, metric_col in metrics.items():
    print(f"\n{'='*80}")
    print(f"METRIC: {metric_name}")
    print(f"{'='*80}\n")

    # Compute per-state success rates
    state_summary = df.groupby('state').agg({
        'state_minority_pct': 'first',
        'num_districts': 'first',
        'target_mm': 'first',
        metric_col: 'mean',
        'mm_count': 'max'
    }).reset_index()

    state_summary.columns = ['state', 'state_minority_pct', 'num_districts',
                             'target_mm', 'success_rate', 'max_mm']
    state_summary['achieves_any'] = state_summary['max_mm'] >= 1
    state_summary['achieves_target'] = state_summary['max_mm'] >= state_summary['target_mm']

    # Sort by minority %
    state_summary = state_summary.sort_values('state_minority_pct', ascending=False)

    # Categorize states
    state_summary['category'] = pd.cut(
        state_summary['state_minority_pct'],
        bins=[0, 0.37, 0.42, 1.0],
        labels=['Below (<37%)', 'Borderline (37-42%)', 'Above (>=42%)']
    )

    # Category statistics
    by_category = state_summary.groupby('category', observed=True).agg({
        'state': 'count',
        'success_rate': 'mean',
        'achieves_any': 'sum',
        'achieves_target': 'sum'
    })

    print("Success by Category:")
    print(by_category)
    print()

    # Logistic regression for threshold
    X = state_summary['state_minority_pct'].values.reshape(-1, 1)
    y = (state_summary['success_rate'] > 0.5).astype(int)

    model = LogisticRegression()
    model.fit(X, y)

    threshold = -model.intercept_[0] / model.coef_[0][0]

    # Bootstrap confidence interval
    thresholds = []
    for _ in range(1000):
        indices = np.random.choice(len(X), len(X), replace=True)
        X_boot = X[indices]
        y_boot = y[indices]

        try:
            model_boot = LogisticRegression()
            model_boot.fit(X_boot, y_boot)
            threshold_boot = -model_boot.intercept_[0] / model_boot.coef_[0][0]
            if 0 < threshold_boot < 1:  # Reasonable bounds
                thresholds.append(threshold_boot)
        except:
            pass

    if thresholds:
        ci_lower = np.percentile(thresholds, 2.5)
        ci_upper = np.percentile(thresholds, 97.5)
    else:
        ci_lower, ci_upper = threshold, threshold

    print(f"Threshold (P(success>50%) = 0.5): {threshold:.1%}")
    print(f"95% Confidence Interval: [{ci_lower:.1%}, {ci_upper:.1%}]")
    print()

    # Correlation
    r, p = stats.pearsonr(state_summary['state_minority_pct'], state_summary['success_rate'])
    print(f"Correlation: r = {r:.4f} (p = {p:.2e})")
    print(f"R^2 = {r**2:.4f}")
    print()

    # T-test: above vs below 42%
    above_42 = state_summary[state_summary['state_minority_pct'] >= 0.42]
    below_37 = state_summary[state_summary['state_minority_pct'] < 0.37]

    if len(above_42) > 0 and len(below_37) > 0:
        t_stat, p_value = stats.ttest_ind(above_42['success_rate'], below_37['success_rate'])
        print(f"t-test (Above >=42% vs Below <37%):")
        print(f"  Above: {above_42['success_rate'].mean():.1%} (n={len(above_42)})")
        print(f"  Below: {below_37['success_rate'].mean():.1%} (n={len(below_37)})")
        print(f"  t = {t_stat:.4f}, p = {p_value:.2e}")

    results[metric_name] = {
        'threshold': threshold,
        'ci_lower': ci_lower,
        'ci_upper': ci_upper,
        'correlation': r,
        'r_squared': r**2,
        'p_value': p
    }

# Summary comparison
print("\n" + "="*80)
print("SUMMARY COMPARISON")
print("="*80)
print()

comparison = pd.DataFrame(results).T
comparison['threshold_pct'] = comparison['threshold'].apply(lambda x: f"{x:.1%}")
comparison['ci'] = comparison.apply(lambda row: f"[{row['ci_lower']:.1%}, {row['ci_upper']:.1%}]", axis=1)
comparison['correlation_str'] = comparison['correlation'].apply(lambda x: f"{x:.4f}")

print(comparison[['threshold_pct', 'ci', 'correlation_str', 'r_squared']])
print()

# Recommendation
print("="*80)
print("RECOMMENDATION")
print("="*80)
print()
print("For VRA compliance research:")
print()
print("1. FEASIBILITY THRESHOLD (~42-50%)")
print("   - Question: 'When can we create MM districts at all?'")
print("   - Use case: Initial VRA compliance assessment")
print("   - More lenient, focuses on possibility")
print()
print("2. PROPORTIONALITY THRESHOLD (~80%)")
print("   - Question: 'When can we achieve full proportional representation?'")
print("   - Use case: Strong VRA compliance, Gingles prong satisfaction")
print("   - More stringent, focuses on full target achievement")
print()
print("PAPER SHOULD CLARIFY:")
print("- Which threshold is legally relevant for VRA Section 2?")
print("- Is 'creating MM districts' sufficient, or must they be proportional?")
print("- How does this relate to Gingles three-prong test?")
print()

# Save detailed breakdown
state_summary_all = df.groupby('state').agg({
    'state_minority_pct': 'first',
    'num_districts': 'first',
    'target_mm': 'first',
    'mm_count': 'max',
    'success_feasibility': 'mean',
    'success_proportionality': 'mean',
}).reset_index()

state_summary_all.columns = ['state', 'minority_pct', 'num_districts', 'target_mm',
                              'max_mm', 'feasibility_rate', 'proportionality_rate']
state_summary_all = state_summary_all.sort_values('minority_pct', ascending=False)

output_file = Path('results/threshold_comparison_50states.csv')
state_summary_all.to_csv(output_file, index=False, float_format='%.4f')
print(f"\nDetailed comparison saved to: {output_file}")
