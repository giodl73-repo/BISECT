"""
Validate Paper 3 cross-census compactness numbers against current pipeline outputs.

This script:
1. Aggregates district_summary.csv files from all states for each census year
2. Loads enacted baseline compactness data
3. Computes means and generates comparison tables
4. Reports any discrepancies with Paper 3 published numbers

Usage:
    python scripts/baseline/validate_paper3_numbers.py
"""

import pandas as pd
from pathlib import Path
import glob

# Paper 3 published numbers
PAPER3_NUMBERS = {
    '2010': {
        'algorithmic_pp': 0.3201,
        'enacted_pp': 0.2248,
        'improvement_pct': 42.4
    },
    '2020': {
        'algorithmic_pp': 0.3532,
        'enacted_pp': 0.3050,
        'improvement_pct': 15.8
    }
}

def aggregate_state_summaries(output_dir):
    """Aggregate district_summary.csv files from all states."""
    pattern = f"{output_dir}/states/*/district_summary.csv"
    files = glob.glob(pattern)

    if not files:
        return None, f"No district_summary.csv files found in {output_dir}/states/"

    dfs = []
    for file in files:
        df = pd.read_csv(file)
        # Extract state name from path
        state = Path(file).parent.name
        df['state'] = state
        dfs.append(df)

    return pd.concat(dfs, ignore_index=True), None

def load_enacted_baseline(year):
    """Load enacted district compactness baseline."""
    if year == '2000':
        file = 'data/enacted_districts/2000/enacted_compactness_2000.csv'
    elif year == '2010':
        file = 'data/enacted_districts/2010/enacted_compactness_2010.csv'
    elif year == '2020':
        # Check if we have the per-state files or aggregated file
        files = glob.glob('outputs/baseline_comparison*/enacted_district_compactness.csv')
        if files:
            file = files[0]  # Use most recent
        else:
            return None, "No 2020 enacted baseline found"
    else:
        return None, f"Unknown year: {year}"

    if not Path(file).exists():
        return None, f"Baseline file not found: {file}"

    df = pd.read_csv(file)
    return df, None

def compute_comparison(year, output_dir):
    """Compute algorithmic vs enacted comparison for a census year."""
    print(f"\n{'='*70}")
    print(f"  Validating {year} Census Compactness")
    print(f"{'='*70}")

    # Load algorithmic results
    print(f"\n[1/3] Loading algorithmic results from {output_dir}...")
    algo_df, error = aggregate_state_summaries(output_dir)
    if error:
        print(f"  ERROR: {error}")
        return None

    algo_mean_pp = algo_df['polsby_popper'].mean()
    algo_median_pp = algo_df['polsby_popper'].median()
    algo_count = len(algo_df)
    print(f"  [OK] Loaded {algo_count} districts")
    print(f"       Mean PP: {algo_mean_pp:.4f}")
    print(f"       Median PP: {algo_median_pp:.4f}")

    # Load enacted baseline
    print(f"\n[2/3] Loading enacted baseline...")
    enacted_df, error = load_enacted_baseline(year)
    if error:
        print(f"  ERROR: {error}")
        return None

    enacted_mean_pp = enacted_df['polsby_popper'].mean()
    enacted_median_pp = enacted_df['polsby_popper'].median()
    enacted_count = len(enacted_df)
    print(f"  [OK] Loaded {enacted_count} districts")
    print(f"    Mean PP: {enacted_mean_pp:.4f}")
    print(f"    Median PP: {enacted_median_pp:.4f}")

    # Compute improvement
    improvement_pct = ((algo_mean_pp / enacted_mean_pp) - 1) * 100

    print(f"\n[3/3] Comparison:")
    print(f"  Algorithmic PP:  {algo_mean_pp:.4f}")
    print(f"  Enacted PP:      {enacted_mean_pp:.4f}")
    print(f"  Improvement:     +{improvement_pct:.1f}%")

    # Compare to Paper 3 numbers if available
    if year in PAPER3_NUMBERS:
        print(f"\n  Paper 3 Published:")
        paper3 = PAPER3_NUMBERS[year]
        print(f"    Algorithmic PP:  {paper3['algorithmic_pp']:.4f}")
        print(f"    Enacted PP:      {paper3['enacted_pp']:.4f}")
        print(f"    Improvement:     +{paper3['improvement_pct']:.1f}%")

        # Check discrepancies
        algo_diff = abs(algo_mean_pp - paper3['algorithmic_pp'])
        enacted_diff = abs(enacted_mean_pp - paper3['enacted_pp'])

        print(f"\n  Validation:")
        if algo_diff < 0.001:
            print(f"    [OK] Algorithmic PP matches (diff: {algo_diff:.5f})")
        else:
            print(f"    [WARN] Algorithmic PP differs (diff: {algo_diff:.5f})")

        if enacted_diff < 0.001:
            print(f"    [OK] Enacted PP matches (diff: {enacted_diff:.5f})")
        else:
            print(f"    [WARN] Enacted PP differs (diff: {enacted_diff:.5f})")

    return {
        'year': year,
        'algo_mean_pp': algo_mean_pp,
        'algo_median_pp': algo_median_pp,
        'enacted_mean_pp': enacted_mean_pp,
        'enacted_median_pp': enacted_median_pp,
        'improvement_pct': improvement_pct,
        'algo_count': algo_count,
        'enacted_count': enacted_count
    }

def main():
    print("\n" + "="*70)
    print("  Paper 3 Cross-Census Validation")
    print("  Verifying Compactness Numbers")
    print("="*70)

    results = []

    # Validate 2000
    result_2000 = compute_comparison('2000', 'outputs/us_2000_v1')
    if result_2000:
        results.append(result_2000)

    # Validate 2010
    result_2010 = compute_comparison('2010', 'outputs/us_2010_v1')
    if result_2010:
        results.append(result_2010)

    # Validate 2020 unweighted
    print(f"\n\n{'='*70}")
    print(f"  2020 Unweighted (for reference)")
    print(f"{'='*70}")
    result_2020_unweighted = compute_comparison('2020', 'outputs/us_2020_v1')

    # Validate 2020 edge-weighted
    # Note: Need to check which directory has the edge-weighted results
    # The paper mentions us_2020_v1_edge but that directory is empty
    # Check if edge-weighted results are in baseline_comparison_edge

    print(f"\n\n{'='*70}")
    print(f"  2020 Edge-Weighted Analysis")
    print(f"{'='*70}")
    print(f"\n  Checking for edge-weighted results...")

    edge_comparison_file = 'outputs/baseline_comparison_edge/algorithmic_vs_enacted_comparison.csv'
    if Path(edge_comparison_file).exists():
        print(f"  Found: {edge_comparison_file}")
        df = pd.read_csv(edge_comparison_file)
        edge_mean_pp = df['algo_pp'].mean()
        enacted_mean_pp = df['enacted_pp'].mean()
        improvement = ((edge_mean_pp / enacted_mean_pp) - 1) * 100

        print(f"\n  From comparison file:")
        print(f"    Algorithmic PP:  {edge_mean_pp:.4f}")
        print(f"    Enacted PP:      {enacted_mean_pp:.4f}")
        print(f"    Improvement:     +{improvement:.1f}%")

        print(f"\n  Paper 3 Published:")
        print(f"    Algorithmic PP:  {PAPER3_NUMBERS['2020']['algorithmic_pp']:.4f}")
        print(f"    Enacted PP:      {PAPER3_NUMBERS['2020']['enacted_pp']:.4f}")
        print(f"    Improvement:     +{PAPER3_NUMBERS['2020']['improvement_pct']:.1f}%")

        diff = abs(edge_mean_pp - PAPER3_NUMBERS['2020']['algorithmic_pp'])
        print(f"\n  Validation:")
        if diff < 0.01:
            print(f"    [OK] Close match (diff: {diff:.4f})")
        else:
            print(f"    [WARN] Significant difference (diff: {diff:.4f})")
            print(f"\n  NOTE: Edge-weighted results may need recomputation from")
            print(f"        per-state district_summary.csv files if available")
    else:
        print(f"  ERROR: Edge-weighted comparison file not found")
        print(f"         Expected: {edge_comparison_file}")

    # Summary table
    if results:
        print(f"\n\n{'='*70}")
        print(f"  Summary Table: Algorithmic Consistency")
        print(f"{'='*70}")
        print(f"\n  {'Year':<6} {'Algo PP':<10} {'Enacted PP':<12} {'Improvement':<12} {'Districts':<10}")
        print(f"  {'-'*60}")
        for r in results:
            print(f"  {r['year']:<6} {r['algo_mean_pp']:<10.4f} {r['enacted_mean_pp']:<12.4f} {r['improvement_pct']:<12.1f}% {r['algo_count']:<10}")

        # Cross-census algorithmic variation
        if len(results) >= 2:
            pp_values = [r['algo_mean_pp'] for r in results]
            variation = (max(pp_values) - min(pp_values)) / min(pp_values) * 100
            print(f"\n  Cross-census algorithmic variation: {variation:.1f}%")
            print(f"  (demonstrates algorithm is driven by geography, not politics)")

    print(f"\n{'='*70}\n")

if __name__ == '__main__':
    main()
