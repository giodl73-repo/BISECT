#!/usr/bin/env python3
"""
VRA Demographic Analysis for P1.2

Analyzes majority-minority districts in algorithmic vs enacted plans.
Identifies potential Section 2 Voting Rights Act compliance issues.

Usage:
    python vra_demographic_analysis.py --output research/gerry-recursive-bisection/data/vra/
"""

import json
import pandas as pd
import numpy as np
from pathlib import Path
import argparse
import sys

# State configurations (from config_2020)
STATE_FIPS = {
    'AL': ('01', 'Alabama'), 'AK': ('02', 'Alaska'), 'AZ': ('04', 'Arizona'),
    'AR': ('05', 'Arkansas'), 'CA': ('06', 'California'), 'CO': ('08', 'Colorado'),
    'CT': ('09', 'Connecticut'), 'DE': ('10', 'Delaware'), 'FL': ('12', 'Florida'),
    'GA': ('13', 'Georgia'), 'HI': ('15', 'Hawaii'), 'ID': ('16', 'Idaho'),
    'IL': ('17', 'Illinois'), 'IN': ('18', 'Indiana'), 'IA': ('19', 'Iowa'),
    'KS': ('20', 'Kansas'), 'KY': ('21', 'Kentucky'), 'LA': ('22', 'Louisiana'),
    'ME': ('23', 'Maine'), 'MD': ('24', 'Maryland'), 'MA': ('25', 'Massachusetts'),
    'MI': ('26', 'Michigan'), 'MN': ('27', 'Minnesota'), 'MS': ('28', 'Mississippi'),
    'MO': ('29', 'Missouri'), 'MT': ('30', 'Montana'), 'NE': ('31', 'Nebraska'),
    'NV': ('32', 'Nevada'), 'NH': ('33', 'New Hampshire'), 'NJ': ('34', 'New Jersey'),
    'NM': ('35', 'New Mexico'), 'NY': ('36', 'New York'), 'NC': ('37', 'North Carolina'),
    'ND': ('38', 'North Dakota'), 'OH': ('39', 'Ohio'), 'OK': ('40', 'Oklahoma'),
    'OR': ('41', 'Oregon'), 'PA': ('42', 'Pennsylvania'), 'RI': ('44', 'Rhode Island'),
    'SC': ('45', 'South Carolina'), 'SD': ('46', 'South Dakota'), 'TN': ('47', 'Tennessee'),
    'TX': ('48', 'Texas'), 'UT': ('49', 'Utah'), 'VT': ('50', 'Vermont'),
    'VA': ('51', 'Virginia'), 'WA': ('53', 'Washington'), 'WV': ('54', 'West Virginia'),
    'WI': ('55', 'Wisconsin'), 'WY': ('56', 'Wyoming')
}

# VRA-sensitive states (historical Section 2 litigation)
VRA_PRIORITY_STATES = {
    'CRITICAL': ['AL', 'GA', 'LA', 'MS'],  # Recent Section 2 cases
    'HIGH': ['SC', 'NC', 'TX', 'FL'],       # Significant minority populations
    'MEDIUM': ['NY', 'CA', 'IL', 'VA', 'MD', 'NJ']  # Large minority populations
}


def load_demographic_data(data_dir):
    """Load algorithmic and enacted district demographics."""
    algo_file = Path(data_dir) / 'district_demographics_2020_algorithmic.json'
    enacted_file = Path(data_dir) / 'district_demographics_2020_enacted.json'

    with open(algo_file, 'r') as f:
        algo_data = json.load(f)

    with open(enacted_file, 'r') as f:
        enacted_data = json.load(f)

    return algo_data, enacted_data


def classify_district(district_demographics):
    """
    Classify district as majority-minority based on demographics.

    Returns:
        dict: Classification with type and percentages
    """
    pct_white = district_demographics.get('pct_white', 0)
    pct_black = district_demographics.get('pct_black', 0)
    pct_hispanic = district_demographics.get('pct_hispanic', 0)
    pct_asian = district_demographics.get('pct_asian', 0)
    pct_other = district_demographics.get('pct_other', 0)

    # Non-white percentage
    pct_nonwhite = 1.0 - pct_white

    # Majority-minority: >50% non-white
    is_mm = pct_nonwhite > 0.50

    # Determine primary minority group
    minority_groups = {
        'Black': pct_black,
        'Hispanic': pct_hispanic,
        'Asian': pct_asian,
        'Other': pct_other
    }

    primary_minority = max(minority_groups, key=minority_groups.get)
    primary_pct = minority_groups[primary_minority]

    # Coalition district: Minority majority but no single group >50%
    is_coalition = is_mm and primary_pct < 0.50

    return {
        'is_mm': is_mm,
        'is_coalition': is_coalition,
        'pct_nonwhite': pct_nonwhite,
        'pct_white': pct_white,
        'pct_black': pct_black,
        'pct_hispanic': pct_hispanic,
        'pct_asian': pct_asian,
        'primary_minority': primary_minority if is_mm else None,
        'primary_minority_pct': primary_pct if is_mm else 0
    }


def analyze_state(state_name, algo_districts, enacted_districts):
    """
    Analyze majority-minority districts for a single state.

    Returns:
        dict: State-level analysis with MM district counts
    """
    # Analyze algorithmic districts
    algo_mm = []
    for dist in algo_districts:
        classification = classify_district(dist)
        if classification['is_mm']:
            algo_mm.append({
                'district': dist['district'],
                'total_pop': dist['total_pop'],
                **classification
            })

    # Analyze enacted districts
    enacted_mm = []
    for dist in enacted_districts:
        classification = classify_district(dist)
        if classification['is_mm']:
            enacted_mm.append({
                'district': dist['district'],
                'total_pop': dist['total_pop'],
                **classification
            })

    # Compare
    algo_mm_count = len(algo_mm)
    enacted_mm_count = len(enacted_mm)
    deficit = enacted_mm_count - algo_mm_count

    # Identify retrogression (VRA violation if deficit > 0)
    is_retrogression = deficit > 0

    return {
        'state': state_name,
        'total_districts': len(algo_districts),
        'algo_mm_count': algo_mm_count,
        'enacted_mm_count': enacted_mm_count,
        'deficit': deficit,
        'is_retrogression': is_retrogression,
        'algo_mm_districts': algo_mm,
        'enacted_mm_districts': enacted_mm
    }


def generate_state_comparison_table(analyses, output_dir):
    """Generate CSV table comparing algorithmic vs enacted MM districts."""
    rows = []

    for analysis in analyses:
        # Determine priority
        priority = 'LOW'
        state_code = [k for k, v in STATE_FIPS.items() if v[1].lower() == analysis['state'].lower()]
        if state_code:
            state_code = state_code[0]
            if state_code in VRA_PRIORITY_STATES['CRITICAL']:
                priority = 'CRITICAL'
            elif state_code in VRA_PRIORITY_STATES['HIGH']:
                priority = 'HIGH'
            elif state_code in VRA_PRIORITY_STATES['MEDIUM']:
                priority = 'MEDIUM'

        rows.append({
            'State': analysis['state'].title(),
            'State Code': state_code[0] if state_code else '',
            'Priority': priority,
            'Total Districts': analysis['total_districts'],
            'Algorithmic MM': analysis['algo_mm_count'],
            'Enacted MM': analysis['enacted_mm_count'],
            'Deficit': analysis['deficit'],
            'Retrogression': 'YES' if analysis['is_retrogression'] else 'NO'
        })

    df = pd.DataFrame(rows)

    # Sort by priority and deficit
    priority_order = {'CRITICAL': 0, 'HIGH': 1, 'MEDIUM': 2, 'LOW': 3}
    df['priority_rank'] = df['Priority'].map(priority_order)
    df = df.sort_values(['priority_rank', 'Deficit'], ascending=[True, False])
    df = df.drop(columns=['priority_rank'])

    # Save
    output_file = Path(output_dir) / 'vra_mm_district_comparison.csv'
    df.to_csv(output_file, index=False)
    print(f"Saved: {output_file}")

    return df


def generate_detailed_district_table(analyses, output_dir):
    """Generate detailed CSV with all MM districts."""
    rows = []

    for analysis in analyses:
        state = analysis['state']

        # Algorithmic MM districts
        for dist in analysis['algo_mm_districts']:
            rows.append({
                'State': state.title(),
                'Plan': 'Algorithmic',
                'District': dist['district'],
                'Total Population': dist['total_pop'],
                'Pct Non-White': f"{dist['pct_nonwhite']:.1%}",
                'Pct Black': f"{dist['pct_black']:.1%}",
                'Pct Hispanic': f"{dist['pct_hispanic']:.1%}",
                'Pct Asian': f"{dist['pct_asian']:.1%}",
                'Primary Minority': dist['primary_minority'],
                'Coalition': 'Yes' if dist['is_coalition'] else 'No'
            })

        # Enacted MM districts
        for dist in analysis['enacted_mm_districts']:
            rows.append({
                'State': state.title(),
                'Plan': 'Enacted',
                'District': dist['district'],
                'Total Population': dist['total_pop'],
                'Pct Non-White': f"{dist['pct_nonwhite']:.1%}",
                'Pct Black': f"{dist['pct_black']:.1%}",
                'Pct Hispanic': f"{dist['pct_hispanic']:.1%}",
                'Pct Asian': f"{dist['pct_asian']:.1%}",
                'Primary Minority': dist['primary_minority'],
                'Coalition': 'Yes' if dist['is_coalition'] else 'No'
            })

    df = pd.DataFrame(rows)
    output_file = Path(output_dir) / 'vra_mm_districts_detailed.csv'
    df.to_csv(output_file, index=False)
    print(f"Saved: {output_file}")

    return df


def generate_summary_statistics(analyses):
    """Generate summary statistics for paper."""
    total_states = len(analyses)

    # Aggregate counts
    total_algo_mm = sum(a['algo_mm_count'] for a in analyses)
    total_enacted_mm = sum(a['enacted_mm_count'] for a in analyses)
    total_deficit = total_enacted_mm - total_algo_mm

    # States with retrogression
    retrogression_states = [a for a in analyses if a['is_retrogression']]
    num_retrogression = len(retrogression_states)

    # Critical states analysis
    critical_analyses = []
    for state_code in VRA_PRIORITY_STATES['CRITICAL']:
        state_name = STATE_FIPS[state_code][1].lower()
        analysis = next((a for a in analyses if a['state'].lower() == state_name), None)
        if analysis:
            critical_analyses.append(analysis)

    critical_algo_mm = sum(a['algo_mm_count'] for a in critical_analyses)
    critical_enacted_mm = sum(a['enacted_mm_count'] for a in critical_analyses)
    critical_deficit = critical_enacted_mm - critical_algo_mm

    summary = {
        'total_states': total_states,
        'total_algo_mm': total_algo_mm,
        'total_enacted_mm': total_enacted_mm,
        'total_deficit': total_deficit,
        'deficit_pct': (total_deficit / total_enacted_mm * 100) if total_enacted_mm > 0 else 0,
        'num_retrogression_states': num_retrogression,
        'pct_retrogression_states': (num_retrogression / total_states * 100),
        'critical_states_analyzed': len(critical_analyses),
        'critical_algo_mm': critical_algo_mm,
        'critical_enacted_mm': critical_enacted_mm,
        'critical_deficit': critical_deficit,
        'retrogression_states': [a['state'] for a in retrogression_states]
    }

    return summary


def print_summary(summary):
    """Print summary statistics to console."""
    print("\n" + "="*80)
    print("VRA MAJORITY-MINORITY DISTRICT ANALYSIS")
    print("="*80)
    print(f"\nNational Summary:")
    print(f"  Total States Analyzed: {summary['total_states']}")
    print(f"  Algorithmic MM Districts: {summary['total_algo_mm']}")
    print(f"  Enacted MM Districts: {summary['total_enacted_mm']}")
    print(f"  Deficit: {summary['total_deficit']} ({summary['deficit_pct']:.1f}%)")
    print(f"\nRetrogression Analysis:")
    print(f"  States with Retrogression: {summary['num_retrogression_states']} ({summary['pct_retrogression_states']:.1f}%)")

    print(f"\nCritical VRA States (AL, GA, LA, MS):")
    print(f"  States Analyzed: {summary['critical_states_analyzed']}")
    print(f"  Algorithmic MM Districts: {summary['critical_algo_mm']}")
    print(f"  Enacted MM Districts: {summary['critical_enacted_mm']}")
    print(f"  Deficit: {summary['critical_deficit']}")

    if summary['retrogression_states']:
        print(f"\nStates with MM District Deficit:")
        for state in summary['retrogression_states'][:10]:  # Show top 10
            print(f"  - {state.title()}")

    print("="*80 + "\n")


def main():
    parser = argparse.ArgumentParser(description='VRA demographic analysis')
    parser.add_argument('--data-dir', type=str,
                       default='outputs/data/2020/demographics',
                       help='Directory with demographic JSON files')
    parser.add_argument('--output', type=str,
                       default='research/gerry-recursive-bisection/data/vra',
                       help='Output directory for analysis results')

    args = parser.parse_args()

    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("Loading demographic data...")
    algo_data, enacted_data = load_demographic_data(args.data_dir)

    print(f"Analyzing {len(algo_data)} states...")

    # Analyze each state
    analyses = []
    for state_name in sorted(algo_data.keys()):
        if state_name not in enacted_data:
            print(f"  WARNING: {state_name} not in enacted data, skipping")
            continue

        analysis = analyze_state(
            state_name,
            algo_data[state_name],
            enacted_data[state_name]
        )
        analyses.append(analysis)

    # Generate outputs
    print("\nGenerating comparison tables...")
    comparison_df = generate_state_comparison_table(analyses, output_dir)
    detailed_df = generate_detailed_district_table(analyses, output_dir)

    # Generate summary
    summary = generate_summary_statistics(analyses)

    # Save summary JSON
    summary_file = output_dir / 'vra_summary.json'
    with open(summary_file, 'w') as f:
        json.dump(summary, f, indent=2)
    print(f"Saved: {summary_file}")

    # Print summary
    print_summary(summary)

    # Print top retrogression states
    print("\nTop 10 States by MM District Deficit:")
    print(comparison_df[comparison_df['Deficit'] > 0].head(10).to_string(index=False))

    print(f"\n[OK] Analysis complete. Results saved to: {output_dir}")


if __name__ == '__main__':
    main()
