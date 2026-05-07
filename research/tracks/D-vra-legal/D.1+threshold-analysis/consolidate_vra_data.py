"""
Consolidate VRA state data from multiple experiments.

Combines:
- Edge-weighted results from gerry-vra-compliance
- Multi-constraint results from gerry-multi-vs-edge
- State demographics and success metrics

Output: consolidated_vra_data.csv
"""

import pandas as pd
from pathlib import Path

# VRA states with known demographics
VRA_STATES = {
    'AL': {'name': 'Alabama', 'minority_pct': 0.369, 'k': 7, 'target_mm': 2},
    'GA': {'name': 'Georgia', 'minority_pct': 0.424, 'k': 14, 'target_mm': 5},
    'LA': {'name': 'Louisiana', 'minority_pct': 0.416, 'k': 6, 'target_mm': 2},
    'MS': {'name': 'Mississippi', 'minority_pct': 0.461, 'k': 4, 'target_mm': 2},
    'SC': {'name': 'South Carolina', 'minority_pct': 0.351, 'k': 7, 'target_mm': 3},
}


def load_edge_weighted_data():
    """Load edge-weighted results from VRA compliance paper."""
    path = Path('../gerry-vra-compliance/results/edge_weighting_ablation_study.csv')
    df = pd.read_csv(path)

    # Group by state and compute best results
    results = []
    for state_code, info in VRA_STATES.items():
        state_data = df[df['state'] == state_code]

        # Find best configuration (highest MM count, breaking ties by max_minority_pct)
        best_row = state_data.sort_values(
            ['mm_count', 'max_minority_pct'],
            ascending=[False, False]
        ).iloc[0]

        # Compute success metrics
        success_count = state_data['success'].sum()
        total_configs = len(state_data)
        success_rate = success_count / total_configs

        results.append({
            'state_code': state_code,
            'state_name': info['name'],
            'state_minority_pct': info['minority_pct'],
            'num_districts': info['k'],
            'target_mm': info['target_mm'],
            'mm_proportion': info['target_mm'] / info['k'],
            'method': 'edge_weighted',
            'best_weight_factor': best_row['weight_factor'],
            'best_minority_threshold': best_row['minority_threshold'],
            'best_mm_count': best_row['mm_count'],
            'best_max_minority_pct': best_row['max_minority_pct'],
            'success_count': success_count,
            'total_configs': total_configs,
            'success_rate': success_rate,
            'achieves_target': best_row['mm_count'] >= info['target_mm'],
        })

    return pd.DataFrame(results)


def load_multi_constraint_data():
    """Load multi-constraint results from multi-constraint vs edge paper."""
    path = Path('../gerry-multi-vs-edge/results/multi_constraint_results.csv')
    df = pd.read_csv(path)

    # Group by state and compute best results
    results = []
    for state_code, info in VRA_STATES.items():
        state_data = df[df['state'] == state_code]

        if len(state_data) == 0:
            # No data for this state
            continue

        # Find best configuration (highest MM count, breaking ties by max_minority_pct)
        best_row = state_data.sort_values(
            ['mm_count', 'max_minority_pct'],
            ascending=[False, False]
        ).iloc[0]

        # Compute success metrics
        success_count = state_data['success'].sum()
        total_configs = len(state_data)
        success_rate = success_count / total_configs

        results.append({
            'state_code': state_code,
            'state_name': info['name'],
            'state_minority_pct': info['minority_pct'],
            'num_districts': info['k'],
            'target_mm': info['target_mm'],
            'mm_proportion': info['target_mm'] / info['k'],
            'method': 'multi_constraint',
            'best_weight_factor': best_row['ubvec_minority'],
            'best_minority_threshold': None,  # Not applicable for multi-constraint
            'best_mm_count': best_row['mm_count'],
            'best_max_minority_pct': best_row['max_minority_pct'],
            'success_count': success_count,
            'total_configs': total_configs,
            'success_rate': success_rate,
            'achieves_target': best_row['mm_count'] >= info['target_mm'],
        })

    return pd.DataFrame(results)


def create_summary_table():
    """Create summary comparison table."""
    edge_df = load_edge_weighted_data()
    multi_df = load_multi_constraint_data()

    # Combine both methods
    combined = pd.concat([edge_df, multi_df], ignore_index=True)

    # Sort by state minority % descending
    combined = combined.sort_values('state_minority_pct', ascending=False)

    return combined


def main():
    """Consolidate all VRA data."""
    print("Consolidating VRA state data...")

    # Load and combine data
    df = create_summary_table()

    # Save consolidated data
    output_path = Path('results')
    output_path.mkdir(exist_ok=True)

    output_file = output_path / 'consolidated_vra_data.csv'
    df.to_csv(output_file, index=False, float_format='%.4f')

    print(f"\nSaved consolidated data to: {output_file}")
    print(f"Total rows: {len(df)}")

    # Print summary
    print("\n=== SUMMARY BY STATE ===")
    for state_code in ['MS', 'GA', 'LA', 'AL', 'SC']:
        state_data = df[df['state_code'] == state_code]
        if len(state_data) == 0:
            continue

        row = state_data.iloc[0]
        print(f"\n{row['state_name']} ({state_code}):")
        print(f"  Minority: {row['state_minority_pct']:.1%}")
        print(f"  Target: {row['target_mm']}/{row['num_districts']} MM districts ({row['mm_proportion']:.1%})")

        for method in ['edge_weighted', 'multi_constraint']:
            method_data = state_data[state_data['method'] == method]
            if len(method_data) == 0:
                continue
            row = method_data.iloc[0]
            print(f"  {method.replace('_', ' ').title()}:")
            print(f"    Success rate: {row['success_rate']:.1%} ({row['success_count']}/{row['total_configs']})")
            print(f"    Best result: {row['best_mm_count']:.0f} MM districts ({row['best_max_minority_pct']:.1%} max minority)")
            print(f"    Achieves target: {'YES' if row['achieves_target'] else 'NO'}")

    print("\n=== THRESHOLD PATTERN ===")
    print("Edge-weighted success rates:")
    edge_only = df[df['method'] == 'edge_weighted'].sort_values('state_minority_pct', ascending=False)
    for _, row in edge_only.iterrows():
        status = "[YES]" if row['achieves_target'] else "[NO]"
        print(f"  {status} {row['state_code']}: {row['state_minority_pct']:.1%} minority -> {row['success_rate']:.1%} success")


if __name__ == '__main__':
    main()
