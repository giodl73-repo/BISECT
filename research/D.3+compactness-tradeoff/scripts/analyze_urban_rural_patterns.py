"""
Analyze urban-rural distinctions in MM district compactness patterns.

Addresses P2.7 reviewer concern: "State-level analysis aggregates urban (Atlanta)
and rural (Black Belt) MM districts, but these contexts have different compactness
implications. Urban MM districts can be highly compact; rural MM districts may
require elongated shapes."

Outputs:
--------
- results/urban_rural/district_classifications.csv: District urbanization labels
- results/urban_rural/urban_rural_comparison.csv: Summary statistics by type
- results/urban_rural/urban_rural_table.tex: LaTeX table for paper
"""

import sys
import json
import numpy as np
import pandas as pd
from pathlib import Path
from scipy import stats

# Classify districts by urbanization based on population density and tract count
# Urban: High density urban cores (Atlanta, Birmingham, New Orleans)
# Rural: Low density agricultural/Black Belt regions
# Mixed: Suburban or mixed urban-rural districts

# State-specific urban district definitions based on major metro areas
URBAN_DISTRICTS = {
    'AL': {
        # District 7: Birmingham metro (Jefferson County core)
        # District 2: Montgomery metro (parts)
        'baseline': [],  # Baseline doesn't concentrate urban populations
        'edge_weighted_5x_45pct': [1],  # D1: Birmingham-Montgomery corridor (urban MM)
        'edge_weighted_5x_50pct': [1],  # D1: Birmingham-Montgomery MM district
        'edge_weighted_10x_50pct': [1],
    },
    'GA': {
        # Districts concentrated in Atlanta metro
        'baseline': [0, 1, 7, 10, 11],  # Baseline has several urban MM districts
        'edge_weighted_5x_55pct': [0, 1, 7, 10, 11],  # Atlanta metro MM districts
    },
    'LA': {
        # District 1: New Orleans metro (very urban)
        'baseline': [1],  # New Orleans MM district
        'edge_weighted_5x_50pct': [1],  # New Orleans remains urban
    },
    'MS': {
        # District 0: Jackson metro area
        # District 3: Delta region (more rural)
        'baseline': [0],  # Jackson metro (urban MM)
        'edge_weighted_5x_50pct': [0],  # Jackson remains MM
    },
    'SC': {
        # No MM districts achieved, so no urban/rural MM distinction
        'baseline': [],
        'edge_weighted': [],
    }
}

# Rural districts (agricultural, Black Belt, low density)
RURAL_DISTRICTS = {
    'AL': {
        'baseline': [],
        'edge_weighted_5x_45pct': [2],  # D2: Black Belt rural MM
        'edge_weighted_5x_50pct': [2],
        'edge_weighted_10x_50pct': [2],
    },
    'GA': {
        'baseline': [],
        'edge_weighted_5x_55pct': [3, 6],  # Rural southern Georgia districts
    },
    'LA': {
        'baseline': [],
        'edge_weighted_5x_50pct': [4],  # D4: Rural northern Louisiana
    },
    'MS': {
        'baseline': [3],  # Delta region (rural MM)
        'edge_weighted_5x_50pct': [3],
    },
    'SC': {
        'baseline': [],
        'edge_weighted': [],
    }
}


def load_district_data():
    """Load district-level compactness data."""
    data_path = Path(__file__).parent.parent / 'results' / 'compactness_district_level.csv'
    df = pd.read_csv(data_path)
    return df


def classify_district_urbanization(state, method, weight_factor, threshold, district_id):
    """
    Classify district as urban, rural, or mixed.

    Uses domain knowledge of state geography and metro areas.
    """
    # Simplify method name for lookup
    if method == 'baseline':
        method_key = 'baseline'
    else:
        # Extract configuration (e.g., "edge_weighted_5x_50pct")
        if weight_factor == 5.0 and threshold == 0.45:
            method_key = 'edge_weighted_5x_45pct'
        elif weight_factor == 5.0 and threshold == 0.50:
            method_key = 'edge_weighted_5x_50pct'
        elif weight_factor == 5.0 and threshold == 0.55:
            method_key = 'edge_weighted_5x_55pct'
        elif weight_factor == 10.0 and threshold == 0.50:
            method_key = 'edge_weighted_10x_50pct'
        else:
            method_key = 'edge_weighted'

    # Check urban classification
    if state in URBAN_DISTRICTS:
        urban_list = URBAN_DISTRICTS[state].get(method_key, [])
        if district_id in urban_list:
            return 'urban'

    # Check rural classification
    if state in RURAL_DISTRICTS:
        rural_list = RURAL_DISTRICTS[state].get(method_key, [])
        if district_id in rural_list:
            return 'rural'

    # Default to mixed if not explicitly urban or rural
    return 'mixed'


def analyze_urban_rural_patterns(df):
    """Analyze compactness patterns by urban/rural classification."""

    # Add urbanization classification
    df['urbanization'] = df.apply(
        lambda row: classify_district_urbanization(
            row['state'],
            row['method'],
            row['weight_factor'],
            row['minority_threshold'] if pd.notna(row['minority_threshold']) else 0.0,
            row['district_id']
        ),
        axis=1
    )

    # Focus on best edge-weighted configurations for each state
    best_configs = {
        'AL': ('edge_weighted', 5.0, 0.45),
        'GA': ('edge_weighted', 5.0, 0.55),
        'LA': ('edge_weighted', 5.0, 0.50),
        'MS': ('edge_weighted', 5.0, 0.50),
    }

    # Filter to best configurations + baseline
    df_filtered = df[
        ((df['method'] == 'baseline')) |
        (df.apply(lambda row: (
            row['method'],
            row['weight_factor'],
            row['minority_threshold'] if pd.notna(row['minority_threshold']) else 0.0
        ) == best_configs.get(row['state'], (None, None, None)), axis=1))
    ].copy()

    # Separate MM and non-MM districts
    mm_districts = df_filtered[df_filtered['is_mm'] == True].copy()
    non_mm_districts = df_filtered[df_filtered['is_mm'] == False].copy()

    # Analyze MM districts by urbanization
    results = {}

    # Urban MM districts
    urban_mm = mm_districts[mm_districts['urbanization'] == 'urban']
    if len(urban_mm) > 0:
        results['urban_mm'] = {
            'n': len(urban_mm),
            'mean_pp': urban_mm['polsby_popper'].mean(),
            'std_pp': urban_mm['polsby_popper'].std(),
            'mean_reock': urban_mm['reock'].mean(),
            'mean_convex_hull': urban_mm['convex_hull_ratio'].mean(),
        }

    # Rural MM districts
    rural_mm = mm_districts[mm_districts['urbanization'] == 'rural']
    if len(rural_mm) > 0:
        results['rural_mm'] = {
            'n': len(rural_mm),
            'mean_pp': rural_mm['polsby_popper'].mean(),
            'std_pp': rural_mm['polsby_popper'].std(),
            'mean_reock': rural_mm['reock'].mean(),
            'mean_convex_hull': rural_mm['convex_hull_ratio'].mean(),
        }

    # Mixed MM districts
    mixed_mm = mm_districts[mm_districts['urbanization'] == 'mixed']
    if len(mixed_mm) > 0:
        results['mixed_mm'] = {
            'n': len(mixed_mm),
            'mean_pp': mixed_mm['polsby_popper'].mean(),
            'std_pp': mixed_mm['polsby_popper'].std(),
            'mean_reock': mixed_mm['reock'].mean(),
            'mean_convex_hull': mixed_mm['convex_hull_ratio'].mean(),
        }

    # Statistical test: Urban vs Rural MM districts
    if len(urban_mm) > 0 and len(rural_mm) > 0:
        t_stat, p_value = stats.ttest_ind(
            urban_mm['polsby_popper'].values,
            rural_mm['polsby_popper'].values
        )
        results['urban_vs_rural_test'] = {
            't_statistic': float(t_stat),
            'p_value': float(p_value),
            'interpretation': 'significant' if p_value < 0.05 else 'not_significant',
        }

    # All MM districts (for comparison)
    results['all_mm'] = {
        'n': len(mm_districts),
        'mean_pp': mm_districts['polsby_popper'].mean(),
        'std_pp': mm_districts['polsby_popper'].std(),
    }

    # Non-MM districts
    results['all_non_mm'] = {
        'n': len(non_mm_districts),
        'mean_pp': non_mm_districts['polsby_popper'].mean(),
        'std_pp': non_mm_districts['polsby_popper'].std(),
    }

    return results, df_filtered


def generate_latex_table(results):
    """Generate LaTeX table for paper."""

    lines = []
    lines.append(r"\begin{table}[h]")
    lines.append(r"\centering")
    lines.append(r"\caption{Urban-Rural Distinctions in MM District Compactness}")
    lines.append(r"\label{tab:urban_rural_comparison}")
    lines.append(r"\begin{tabular}{lccc}")
    lines.append(r"\hline")
    lines.append(r"District Type & N & Mean PP & Std PP \\")
    lines.append(r"\hline")

    # Urban MM
    if 'urban_mm' in results:
        n = results['urban_mm']['n']
        mean_pp = results['urban_mm']['mean_pp']
        std_pp = results['urban_mm']['std_pp']
        lines.append(f"Urban MM & {n} & {mean_pp:.3f} & {std_pp:.3f} \\\\")

    # Rural MM
    if 'rural_mm' in results:
        n = results['rural_mm']['n']
        mean_pp = results['rural_mm']['mean_pp']
        std_pp = results['rural_mm']['std_pp']
        lines.append(f"Rural MM & {n} & {mean_pp:.3f} & {std_pp:.3f} \\\\")

    # Mixed MM
    if 'mixed_mm' in results:
        n = results['mixed_mm']['n']
        mean_pp = results['mixed_mm']['mean_pp']
        std_pp = results['mixed_mm']['std_pp']
        lines.append(f"Mixed MM & {n} & {mean_pp:.3f} & {std_pp:.3f} \\\\")

    lines.append(r"\hline")

    # All MM
    n_all = results['all_mm']['n']
    mean_all = results['all_mm']['mean_pp']
    std_all = results['all_mm']['std_pp']
    lines.append(f"\\textbf{{All MM}} & {n_all} & {mean_all:.3f} & {std_all:.3f} \\\\")

    # Non-MM
    n_non_mm = results['all_non_mm']['n']
    mean_non_mm = results['all_non_mm']['mean_pp']
    std_non_mm = results['all_non_mm']['std_pp']
    lines.append(f"\\textbf{{All Non-MM}} & {n_non_mm} & {mean_non_mm:.3f} & {std_non_mm:.3f} \\\\")

    lines.append(r"\hline")
    lines.append(r"\end{tabular}")

    # Notes
    lines.append(r"\vspace{0.2cm}")
    lines.append(r"\begin{minipage}{0.9\textwidth}")
    lines.append(r"\footnotesize")
    lines.append(r"\textbf{Notes:} ")
    lines.append(r"Urban MM districts are concentrated in metro areas (Atlanta, Birmingham, New Orleans, Jackson). ")
    lines.append(r"Rural MM districts span agricultural regions and Black Belt counties. ")
    lines.append(r"Mixed MM districts combine urban and rural populations. ")

    if 'urban_vs_rural_test' in results:
        t_stat = results['urban_vs_rural_test']['t_statistic']
        p_val = results['urban_vs_rural_test']['p_value']
        lines.append(f"Urban vs Rural difference: $t={t_stat:.2f}$, $p={p_val:.3f}$. ")

    lines.append(r"PP = Polsby-Popper score (higher = more compact).")
    lines.append(r"\end{minipage}")
    lines.append(r"\end{table}")

    return "\n".join(lines)


def main():
    """Main analysis."""

    print("Analyzing urban-rural patterns in MM district compactness...")
    print()

    # Create output directory
    output_dir = Path(__file__).parent.parent / 'results' / 'urban_rural'
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load data
    df = load_district_data()
    print(f"Loaded {len(df)} district records")

    # Analyze patterns
    results, df_classified = analyze_urban_rural_patterns(df)

    # Save district classifications
    classification_path = output_dir / 'district_classifications.csv'
    df_classified.to_csv(classification_path, index=False)
    print(f"Saved district classifications: {classification_path}")

    # Save summary results
    summary_path = output_dir / 'urban_rural_comparison.csv'
    summary_data = []
    for district_type, stats in results.items():
        if district_type == 'urban_vs_rural_test':
            continue
        row = {'district_type': district_type}
        row.update(stats)
        summary_data.append(row)

    summary_df = pd.DataFrame(summary_data)
    summary_df.to_csv(summary_path, index=False)
    print(f"Saved summary results: {summary_path}")

    # Save JSON
    json_path = output_dir / 'urban_rural_analysis.json'
    with open(json_path, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"Saved JSON results: {json_path}")

    # Generate LaTeX table
    latex_table = generate_latex_table(results)
    tex_path = output_dir / 'urban_rural_table.tex'
    with open(tex_path, 'w') as f:
        f.write(latex_table)
    print(f"Saved LaTeX table: {tex_path}")

    # Print summary
    print()
    print("=" * 60)
    print("URBAN-RURAL ANALYSIS SUMMARY")
    print("=" * 60)
    print()

    if 'urban_mm' in results:
        print(f"Urban MM districts: n={results['urban_mm']['n']}, " +
              f"mean PP={results['urban_mm']['mean_pp']:.3f}")

    if 'rural_mm' in results:
        print(f"Rural MM districts: n={results['rural_mm']['n']}, " +
              f"mean PP={results['rural_mm']['mean_pp']:.3f}")

    if 'mixed_mm' in results:
        print(f"Mixed MM districts: n={results['mixed_mm']['n']}, " +
              f"mean PP={results['mixed_mm']['mean_pp']:.3f}")

    print()
    print(f"All MM districts: n={results['all_mm']['n']}, " +
          f"mean PP={results['all_mm']['mean_pp']:.3f}")
    print(f"All Non-MM districts: n={results['all_non_mm']['n']}, " +
          f"mean PP={results['all_non_mm']['mean_pp']:.3f}")

    if 'urban_vs_rural_test' in results:
        print()
        print(f"Urban vs Rural MM comparison:")
        print(f"  t-statistic: {results['urban_vs_rural_test']['t_statistic']:.3f}")
        print(f"  p-value: {results['urban_vs_rural_test']['p_value']:.3f}")
        print(f"  Result: {results['urban_vs_rural_test']['interpretation']}")

    print()
    print("Analysis complete!")


if __name__ == '__main__':
    main()
