"""
Compute feasibility metric and validate against empirical results.

Feasibility Metric:
    F = (state_minority_pct × num_districts / target_mm) × clustering_coefficient

Where clustering_coefficient = (Moran's I + 1) / 2 (normalized to [0, 1])

Thresholds:
- F >= 1.2: Likely feasible (high success probability)
- F in [1.0, 1.2): Borderline (uncertain outcome)
- F < 1.0: Likely infeasible (low success probability)
"""

import pandas as pd
from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np

def compute_feasibility_metric(state_minority_pct, num_districts, target_mm, morans_i):
    """
    Compute feasibility metric for VRA compliance.

    Parameters
    ----------
    state_minority_pct : float
        State-level minority percentage (0-1)
    num_districts : int
        Total number of congressional districts
    target_mm : int
        Target number of majority-minority districts
    morans_i : float
        Moran's I spatial autocorrelation coefficient (-1 to +1)

    Returns
    -------
    float
        Feasibility metric F
    str
        Category: "Likely Feasible", "Borderline", or "Likely Infeasible"
    """
    # Normalize Moran's I to [0, 1]
    clustering_coef = (morans_i + 1) / 2

    # Compute feasibility metric
    F = (state_minority_pct * num_districts / target_mm) * clustering_coef

    # Categorize
    if F >= 1.2:
        category = "Likely Feasible"
    elif F >= 1.0:
        category = "Borderline"
    else:
        category = "Likely Infeasible"

    return F, category


def load_and_merge_data():
    """Load consolidated VRA data and clustering metrics."""
    # Load consolidated VRA results
    vra_df = pd.read_csv('results/consolidated_vra_data.csv')

    # Load clustering metrics
    clustering_df = pd.read_csv('results/geographic_clustering_metrics.csv')

    # Merge on state_code
    merged = vra_df.merge(
        clustering_df[['state_code', 'morans_i', 'clustering_index_50', 'mm_tract_pct']],
        on='state_code',
        how='left'
    )

    return merged


def compute_all_feasibility_metrics():
    """Compute feasibility metrics for all states and methods."""
    # Load data
    df = load_and_merge_data()

    # Compute feasibility metric for each row
    results = []
    for _, row in df.iterrows():
        F, category = compute_feasibility_metric(
            state_minority_pct=row['state_minority_pct'],
            num_districts=row['num_districts'],
            target_mm=row['target_mm'],
            morans_i=row['morans_i']
        )

        results.append({
            **row.to_dict(),
            'feasibility_metric': F,
            'feasibility_category': category,
        })

    results_df = pd.DataFrame(results)

    return results_df


def analyze_feasibility_validation():
    """Validate feasibility metric against empirical success rates."""
    df = compute_all_feasibility_metrics()

    # Save results
    output_path = Path('results')
    output_file = output_path / 'feasibility_analysis.csv'
    df.to_csv(output_file, index=False, float_format='%.4f')

    print(f"Saved feasibility analysis to: {output_file}")

    # Print analysis
    print("\n=== FEASIBILITY METRIC VALIDATION ===\n")

    # Group by method
    for method in ['edge_weighted', 'multi_constraint']:
        method_df = df[df['method'] == method].sort_values('state_minority_pct', ascending=False)

        print(f"\n{method.replace('_', ' ').upper()}:")
        print(f"{'State':<5} {'Minority%':<11} {'Moran I':<9} {'F Score':<8} {'Category':<20} "
              f"{'Success Rate':<13} {'Achieves Target':<10}")
        print("-" * 105)

        for _, row in method_df.iterrows():
            achieves = "YES" if row['achieves_target'] else "NO"
            print(f"{row['state_code']:<5} {row['state_minority_pct']:<11.1%} "
                  f"{row['morans_i']:<9.4f} {row['feasibility_metric']:<8.3f} "
                  f"{row['feasibility_category']:<20} {row['success_rate']:<13.1%} {achieves:<10}")

    # Correlation analysis
    print("\n=== CORRELATION ANALYSIS ===\n")

    edge_df = df[df['method'] == 'edge_weighted']

    # Pearson correlation
    corr_f_success = edge_df[['feasibility_metric', 'success_rate']].corr().iloc[0, 1]
    corr_minority_success = edge_df[['state_minority_pct', 'success_rate']].corr().iloc[0, 1]
    corr_morans_success = edge_df[['morans_i', 'success_rate']].corr().iloc[0, 1]

    print(f"Feasibility Metric vs Success Rate: r = {corr_f_success:.4f}")
    print(f"State Minority % vs Success Rate:   r = {corr_minority_success:.4f}")
    print(f"Moran's I vs Success Rate:          r = {corr_morans_success:.4f}")

    # Threshold analysis
    print("\n=== THRESHOLD PATTERNS ===\n")

    print("Edge-Weighted Method:")
    print("  F >= 1.2 states:")
    high_f = edge_df[edge_df['feasibility_metric'] >= 1.2]
    if len(high_f) > 0:
        print(f"    {', '.join(high_f['state_code'].values)}")
        print(f"    Avg success rate: {high_f['success_rate'].mean():.1%}")
        print(f"    Achieve target: {high_f['achieves_target'].sum()}/{len(high_f)}")

    print("\n  F in [1.0, 1.2) states:")
    border_f = edge_df[(edge_df['feasibility_metric'] >= 1.0) & (edge_df['feasibility_metric'] < 1.2)]
    if len(border_f) > 0:
        print(f"    {', '.join(border_f['state_code'].values)}")
        print(f"    Avg success rate: {border_f['success_rate'].mean():.1%}")
        print(f"    Achieve target: {border_f['achieves_target'].sum()}/{len(border_f)}")

    print("\n  F < 1.0 states:")
    low_f = edge_df[edge_df['feasibility_metric'] < 1.0]
    if len(low_f) > 0:
        print(f"    {', '.join(low_f['state_code'].values)}")
        print(f"    Avg success rate: {low_f['success_rate'].mean():.1%}")
        print(f"    Achieve target: {low_f['achieves_target'].sum()}/{len(low_f)}")

    # Key insight
    print("\n=== KEY INSIGHT ===\n")
    print("The 42% Threshold Pattern:")
    print("  States with >=42% minority (GA, MS, LA):")
    high_minority = edge_df[edge_df['state_minority_pct'] >= 0.42]
    print(f"    Average F score: {high_minority['feasibility_metric'].mean():.3f}")
    print(f"    Average success rate: {high_minority['success_rate'].mean():.1%}")
    print(f"    Achieve target: {high_minority['achieves_target'].sum()}/{len(high_minority)}")

    print("\n  States with <37% minority (AL, SC):")
    low_minority = edge_df[edge_df['state_minority_pct'] < 0.37]
    print(f"    Average F score: {low_minority['feasibility_metric'].mean():.3f}")
    print(f"    Average success rate: {low_minority['success_rate'].mean():.1%}")
    print(f"    Achieve target: {low_minority['achieves_target'].sum()}/{len(low_minority)}")

    return df


def main():
    """Main analysis pipeline."""
    print("Computing feasibility metrics...")

    df = analyze_feasibility_validation()

    print(f"\nTotal configurations analyzed: {len(df)}")
    print(f"States: {len(df['state_code'].unique())}")
    print(f"Methods: {', '.join(df['method'].unique())}")


if __name__ == '__main__':
    main()
