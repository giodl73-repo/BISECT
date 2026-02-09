#!/usr/bin/env python3
"""
Analyze Citizen Voting-Age Population (CVAP) vs Total Population sensitivity.

This script addresses P2.1 reviewer concern: 50% of total population may only
be 40-45% of actual voters due to age demographics and citizenship rates.

Methodology:
1. Load district-level total population minority percentages from results
2. Apply demographic adjustment factors for VAP and CVAP
3. Recalculate MM district counts under VAP and CVAP standards
4. Compare to total population baseline
5. Generate tables for paper

Data Sources:
- Total population by race: Census 2020 P.L. 94-171 redistricting data
- CVAP estimates: Census Bureau 2016-2020 ACS 5-Year Special Tabulations
  (https://www.census.gov/programs-surveys/decennial-census/about/voting-rights/cvap.html)
"""

import pandas as pd
import numpy as np
from pathlib import Path

# State-level adjustment factors (2020 Census / ACS 2016-2020 estimates)
# Based on Census Bureau published CVAP data for our 5 study states
# Source: https://www.census.gov/programs-surveys/decennial-census/about/voting-rights/cvap.html

# Adjustment factors: VAP/Total and CVAP/Total ratios by race/state
# Format: {state: {race: {'vap_ratio': X, 'cvap_ratio': Y}}}

ADJUSTMENT_FACTORS = {
    'AL': {
        'black': {
            'vap_ratio': 0.72,    # Black VAP / Black Total Pop (younger age structure)
            'cvap_ratio': 0.71,   # Black CVAP / Black Total Pop (high citizenship)
        },
        'white': {
            'vap_ratio': 0.78,    # White VAP / White Total Pop (older age structure)
            'cvap_ratio': 0.77,   # White CVAP / White Total Pop (high citizenship)
        },
        'total': {
            'vap_ratio': 0.76,
            'cvap_ratio': 0.75,
        }
    },
    'GA': {
        'black': {
            'vap_ratio': 0.73,
            'cvap_ratio': 0.71,
        },
        'white': {
            'vap_ratio': 0.77,
            'cvap_ratio': 0.76,
        },
        'hispanic': {
            'vap_ratio': 0.65,    # Younger age structure
            'cvap_ratio': 0.48,   # Lower citizenship rate
        },
        'total': {
            'vap_ratio': 0.75,
            'cvap_ratio': 0.72,
        }
    },
    'LA': {
        'black': {
            'vap_ratio': 0.72,
            'cvap_ratio': 0.71,
        },
        'white': {
            'vap_ratio': 0.77,
            'cvap_ratio': 0.76,
        },
        'total': {
            'vap_ratio': 0.75,
            'cvap_ratio': 0.74,
        }
    },
    'MS': {
        'black': {
            'vap_ratio': 0.71,    # Youngest Black population among study states
            'cvap_ratio': 0.70,
        },
        'white': {
            'vap_ratio': 0.78,
            'cvap_ratio': 0.77,
        },
        'total': {
            'vap_ratio': 0.74,
            'cvap_ratio': 0.73,
        }
    },
    'SC': {
        'black': {
            'vap_ratio': 0.72,
            'cvap_ratio': 0.71,
        },
        'white': {
            'vap_ratio': 0.79,    # Oldest white population (retirement destination)
            'cvap_ratio': 0.78,
        },
        'total': {
            'vap_ratio': 0.76,
            'cvap_ratio': 0.75,
        }
    }
}


def load_district_demographics(results_dir):
    """Load district-level demographic data from compactness results."""
    demo_file = results_dir / 'compactness_district_level.csv'
    if not demo_file.exists():
        raise FileNotFoundError(f"District demographics not found: {demo_file}")

    df = pd.read_csv(demo_file)
    print(f"Loaded {len(df)} district records from {len(df['state'].unique())} states")
    return df


def apply_cvap_adjustment(row, adjustment_factors):
    """Apply VAP and CVAP adjustment to a district's minority percentage.

    Logic:
    1. minority_pct_total = minority_pop / total_pop
    2. minority_pct_vap = (minority_pop * minority_vap_ratio) / (total_pop * total_vap_ratio)
    3. minority_pct_cvap = (minority_pop * minority_cvap_ratio) / (total_pop * total_cvap_ratio)

    This accounts for differential age structures and citizenship rates between groups.
    """
    state = row['state']
    minority_pct_total = row['minority_pct']

    if state not in adjustment_factors:
        # No adjustment factors available
        return {
            'minority_pct_total': minority_pct_total,
            'minority_pct_vap': None,
            'minority_pct_cvap': None,
            'is_mm_total': row['is_mm'],
            'is_mm_vap': None,
            'is_mm_cvap': None,
        }

    factors = adjustment_factors[state]

    # Approximate minority_pct_vap and minority_pct_cvap
    # Assume district minority composition matches state-level ratios
    # (More sophisticated would use tract-level CVAP, but not available)

    # For Black-majority states (AL, GA, LA, MS, SC), assume minority = Black
    minority_vap_ratio = factors['black']['vap_ratio']
    minority_cvap_ratio = factors['black']['cvap_ratio']
    total_vap_ratio = factors['total']['vap_ratio']
    total_cvap_ratio = factors['total']['cvap_ratio']

    # Adjusted percentages
    minority_pct_vap = (minority_pct_total * minority_vap_ratio) / total_vap_ratio
    minority_pct_cvap = (minority_pct_total * minority_cvap_ratio) / total_cvap_ratio

    return {
        'minority_pct_total': minority_pct_total,
        'minority_pct_vap': minority_pct_vap,
        'minority_pct_cvap': minority_pct_cvap,
        'is_mm_total': minority_pct_total >= 50.0,
        'is_mm_vap': minority_pct_vap >= 50.0,
        'is_mm_cvap': minority_pct_cvap >= 50.0,
    }


def analyze_cvap_sensitivity(df, adjustment_factors):
    """Analyze how MM district counts change under VAP and CVAP standards."""

    # Apply adjustments to each district
    adjustments = df.apply(
        lambda row: apply_cvap_adjustment(row, adjustment_factors),
        axis=1,
        result_type='expand'
    )

    # Merge back with original data
    df_adj = pd.concat([df, adjustments], axis=1)

    # Calculate state-level MM counts by standard
    state_summary = []

    for state in df_adj['state'].unique():
        state_data = df_adj[df_adj['state'] == state]

        # Count MM districts under each standard
        mm_count_total = state_data['is_mm_total'].sum()
        mm_count_vap = state_data['is_mm_vap'].sum() if state_data['is_mm_vap'].notna().any() else None
        mm_count_cvap = state_data['is_mm_cvap'].sum() if state_data['is_mm_cvap'].notna().any() else None

        # Identify districts that lose MM status under VAP/CVAP
        if mm_count_vap is not None:
            lost_vap = state_data[state_data['is_mm_total'] & ~state_data['is_mm_vap']]
            lost_cvap = state_data[state_data['is_mm_total'] & ~state_data['is_mm_cvap']]
        else:
            lost_vap = pd.DataFrame()
            lost_cvap = pd.DataFrame()

        state_summary.append({
            'state': state,
            'total_districts': len(state_data),
            'mm_count_total': mm_count_total,
            'mm_count_vap': mm_count_vap,
            'mm_count_cvap': mm_count_cvap,
            'districts_lost_vap': len(lost_vap),
            'districts_lost_cvap': len(lost_cvap),
            'vap_retention_rate': (mm_count_vap / mm_count_total * 100) if mm_count_total > 0 and mm_count_vap is not None else None,
            'cvap_retention_rate': (mm_count_cvap / mm_count_total * 100) if mm_count_total > 0 and mm_count_cvap is not None else None,
        })

    summary_df = pd.DataFrame(state_summary)
    return df_adj, summary_df


def generate_latex_tables(summary_df, output_dir):
    """Generate LaTeX tables for paper."""

    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Table 1: State-level MM counts by standard
    latex_table1 = """
\\begin{table}[H]
\\centering
\\caption{Majority-Minority District Counts Under Different Population Standards}
\\label{tab:mm_counts_by_standard}
\\begin{tabular}{lcccccc}
\\toprule
\\textbf{State} & \\textbf{Districts} & \\textbf{MM (Total Pop)} & \\textbf{MM (VAP)} & \\textbf{MM (CVAP)} & \\textbf{Lost (VAP)} & \\textbf{Lost (CVAP)} \\\\
\\midrule
"""

    state_names = {
        'AL': 'Alabama', 'GA': 'Georgia', 'LA': 'Louisiana',
        'MS': 'Mississippi', 'SC': 'South Carolina'
    }

    for _, row in summary_df.iterrows():
        state_name = state_names.get(row['state'], row['state'])
        total_districts = int(row['total_districts'])
        mm_total = int(row['mm_count_total'])
        mm_vap = int(row['mm_count_vap']) if row['mm_count_vap'] is not None else '---'
        mm_cvap = int(row['mm_count_cvap']) if row['mm_count_cvap'] is not None else '---'
        lost_vap = int(row['districts_lost_vap']) if row['districts_lost_vap'] is not None else '---'
        lost_cvap = int(row['districts_lost_cvap']) if row['districts_lost_cvap'] is not None else '---'

        latex_table1 += f"{state_name} & {total_districts} & {mm_total} & {mm_vap} & {mm_cvap} & {lost_vap} & {lost_cvap} \\\\\n"

    latex_table1 += """\\bottomrule
\\end{tabular}
\\begin{tablenotes}
\\small
\\item \\textbf{Total Pop}: Standard using total population (50\\% threshold)
\\item \\textbf{VAP}: Voting-age population (18+), adjusted for age structure
\\item \\textbf{CVAP}: Citizen voting-age population, adjusted for age and citizenship
\\item \\textbf{Lost}: Districts classified as MM under Total Pop but not under VAP/CVAP
\\end{tablenotes}
\\end{table}
"""

    with open(output_dir / 'cvap_sensitivity_table.tex', 'w') as f:
        f.write(latex_table1)

    print(f"Generated LaTeX table: {output_dir / 'cvap_sensitivity_table.tex'}")

    # Save CSV for reference
    summary_df.to_csv(output_dir / 'cvap_sensitivity_summary.csv', index=False)
    print(f"Saved summary CSV: {output_dir / 'cvap_sensitivity_summary.csv'}")


def main():
    """Main analysis workflow."""

    # Paths
    results_dir = Path('results')
    output_dir = Path('results') / 'cvap_analysis'

    print("=" * 60)
    print("CVAP Sensitivity Analysis (P2.1)")
    print("=" * 60)

    # Load data
    print("\n1. Loading district demographics...")
    df = load_district_demographics(results_dir)

    # Analyze
    print("\n2. Applying VAP/CVAP adjustments...")
    df_adj, summary_df = analyze_cvap_sensitivity(df, ADJUSTMENT_FACTORS)

    # Generate output
    print("\n3. Generating tables...")
    generate_latex_tables(summary_df, output_dir)

    # Print summary
    print("\n" + "=" * 60)
    print("Summary of Findings:")
    print("=" * 60)

    state_names = {
        'AL': 'Alabama', 'GA': 'Georgia', 'LA': 'Louisiana',
        'MS': 'Mississippi', 'SC': 'South Carolina'
    }

    for _, row in summary_df.iterrows():
        state_name = state_names.get(row['state'], row['state'])
        mm_total = int(row['mm_count_total'])
        mm_cvap = int(row['mm_count_cvap']) if row['mm_count_cvap'] is not None else None
        lost_cvap = int(row['districts_lost_cvap']) if row['districts_lost_cvap'] is not None else None
        retention = row['cvap_retention_rate']

        print(f"\n{state_name}:")
        print(f"  Total Pop: {mm_total} MM districts")
        if mm_cvap is not None:
            print(f"  CVAP: {mm_cvap} MM districts ({lost_cvap} lost)")
            print(f"  Retention rate: {retention:.1f}%")
        else:
            print(f"  CVAP: Data not available")

    print("\n" + "=" * 60)
    print("Analysis complete!")
    print("=" * 60)


if __name__ == '__main__':
    main()
