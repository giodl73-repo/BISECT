#!/usr/bin/env python3
"""
Visualize demographic characteristics of districts.

Creates three maps:
1. Gender balance map (Male vs Female %)
2. Majority race map (colored by dominant demographic group)
3. Diversity index map (showing heterogeneity of districts)
"""

import pandas as pd
import geopandas as gpd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np
import argparse
from pathlib import Path
import os


# Color schemes
LEAN_COLORS = {
    'Male-leaning': '#4A90E2',      # Blue
    'Female-leaning': '#E24A90',    # Pink
    'Balanced': '#9B59B6'            # Purple
}

RACE_COLORS = {
    'White': '#E8F4F8',             # Light blue
    'Black': '#FFF4E6',             # Light orange
    'Asian': '#E8F8E8',             # Light green
    'Hispanic': '#FFF0F0',          # Light red
    'Other': '#F0F0F0'              # Light gray
}

DIVERSITY_COLORS = {
    'Very Homogeneous': '#fee5d9',   # Light
    'Homogeneous': '#fcae91',
    'Moderate': '#fb6a4a',
    'Diverse': '#de2d26',
    'Very Diverse': '#a50f15'        # Dark
}


def create_gender_map(tracts_gdf, district_stats, state_name, output_file, dpi=150):
    """Create map showing gender balance by district."""

    # Merge district stats with tracts
    tracts = tracts_gdf.copy()

    # Classify districts by gender lean
    district_stats = district_stats.copy()
    district_stats['gender_lean'] = 'Balanced'
    district_stats.loc[district_stats['male_pct'] > 51, 'gender_lean'] = 'Male-leaning'
    district_stats.loc[district_stats['female_pct'] > 51, 'gender_lean'] = 'Female-leaning'

    # Merge with tracts
    tracts = tracts.merge(
        district_stats[['district', 'male_pct', 'female_pct', 'gender_lean']],
        on='district',
        how='left'
    )

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(14, 10))

    # Plot each gender lean category
    for lean in ['Male-leaning', 'Balanced', 'Female-leaning']:
        data = tracts[tracts['gender_lean'] == lean]
        if len(data) > 0:
            data.plot(
                ax=ax,
                color=LEAN_COLORS[lean],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.8
            )

    # Add thick district boundaries
    districts_dissolved = tracts.dissolve(by='district', as_index=False)
    districts_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    ax.set_axis_off()
    ax.set_title(f'{state_name} Congressional Districts - Gender Balance',
                 fontsize=16, fontweight='bold', pad=20)

    # Create legend
    legend_elements = []
    for lean in ['Male-leaning', 'Balanced', 'Female-leaning']:
        count = len(district_stats[district_stats['gender_lean'] == lean])
        if count > 0:
            legend_elements.append(
                mpatches.Patch(facecolor=LEAN_COLORS[lean], edgecolor='black',
                             label=f'{lean} ({count} districts)')
            )

    ax.legend(handles=legend_elements, loc='lower right', frameon=True,
              fancybox=True, shadow=True, fontsize=11)

    plt.tight_layout()
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  Created: {output_file.name}")


def create_majority_race_map(tracts_gdf, district_stats, state_name, output_file, dpi=150):
    """Create map showing majority race/ethnicity by district."""

    # Merge district stats with tracts
    tracts = tracts_gdf.copy()
    tracts = tracts.merge(
        district_stats[['district', 'majority_race', 'majority_race_pct']],
        on='district',
        how='left'
    )

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(14, 10))

    # Plot each race category
    for race in ['White', 'Black', 'Asian', 'Hispanic', 'Other']:
        data = tracts[tracts['majority_race'] == race]
        if len(data) > 0:
            data.plot(
                ax=ax,
                color=RACE_COLORS[race],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.9
            )

    # Add thick district boundaries
    districts_dissolved = tracts.dissolve(by='district', as_index=False)
    districts_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    ax.set_axis_off()
    ax.set_title(f'{state_name} Congressional Districts - Majority Race/Ethnicity',
                 fontsize=16, fontweight='bold', pad=20)

    # Create legend with counts
    legend_elements = []
    for race in ['White', 'Black', 'Asian', 'Hispanic', 'Other']:
        count = len(district_stats[district_stats['majority_race'] == race])
        if count > 0:
            legend_elements.append(
                mpatches.Patch(facecolor=RACE_COLORS[race], edgecolor='black',
                             label=f'{race} majority ({count} districts)')
            )

    ax.legend(handles=legend_elements, loc='lower right', frameon=True,
              fancybox=True, shadow=True, fontsize=11)

    plt.tight_layout()
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  Created: {output_file.name}")


def calculate_diversity_index(row):
    """Calculate diversity index (entropy-based) for a district."""
    # Get percentages as proportions
    proportions = [
        row['white_pct'] / 100,
        row['black_pct'] / 100,
        row['asian_pct'] / 100,
        row['hispanic_pct'] / 100,
        row['other_pct'] / 100
    ]

    # Calculate Shannon entropy (normalized)
    entropy = 0
    for p in proportions:
        if p > 0:
            entropy -= p * np.log(p)

    # Normalize to 0-1 scale (max entropy for 5 groups is log(5))
    max_entropy = np.log(5)
    return entropy / max_entropy


def create_diversity_map(tracts_gdf, district_stats, state_name, output_file, dpi=150):
    """Create map showing diversity index by district."""

    # Calculate diversity index for each district
    district_stats = district_stats.copy()
    district_stats['diversity_index'] = district_stats.apply(calculate_diversity_index, axis=1)

    # Classify diversity levels
    district_stats['diversity_level'] = 'Moderate'
    district_stats.loc[district_stats['diversity_index'] < 0.3, 'diversity_level'] = 'Very Homogeneous'
    district_stats.loc[(district_stats['diversity_index'] >= 0.3) &
                      (district_stats['diversity_index'] < 0.5), 'diversity_level'] = 'Homogeneous'
    district_stats.loc[(district_stats['diversity_index'] >= 0.5) &
                      (district_stats['diversity_index'] < 0.7), 'diversity_level'] = 'Moderate'
    district_stats.loc[(district_stats['diversity_index'] >= 0.7) &
                      (district_stats['diversity_index'] < 0.85), 'diversity_level'] = 'Diverse'
    district_stats.loc[district_stats['diversity_index'] >= 0.85, 'diversity_level'] = 'Very Diverse'

    # Merge with tracts
    tracts = tracts_gdf.copy()
    tracts = tracts.merge(
        district_stats[['district', 'diversity_index', 'diversity_level']],
        on='district',
        how='left'
    )

    # Create figure
    fig, ax = plt.subplots(1, 1, figsize=(14, 10))

    # Plot each diversity level
    for level in ['Very Homogeneous', 'Homogeneous', 'Moderate', 'Diverse', 'Very Diverse']:
        data = tracts[tracts['diversity_level'] == level]
        if len(data) > 0:
            data.plot(
                ax=ax,
                color=DIVERSITY_COLORS[level],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.9
            )

    # Add thick district boundaries
    districts_dissolved = tracts.dissolve(by='district', as_index=False)
    districts_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    ax.set_axis_off()
    ax.set_title(f'{state_name} Congressional Districts - Diversity Index',
                 fontsize=16, fontweight='bold', pad=20)

    # Create legend
    legend_elements = []
    for level in ['Very Homogeneous', 'Homogeneous', 'Moderate', 'Diverse', 'Very Diverse']:
        count = len(district_stats[district_stats['diversity_level'] == level])
        if count > 0:
            legend_elements.append(
                mpatches.Patch(facecolor=DIVERSITY_COLORS[level], edgecolor='black',
                             label=f'{level} ({count} districts)')
            )

    ax.legend(handles=legend_elements, loc='lower right', frameon=True,
              fancybox=True, shadow=True, fontsize=11)

    plt.tight_layout()
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    print(f"  Created: {output_file.name}")


def main():
    parser = argparse.ArgumentParser(description='Visualize district demographics')
    parser.add_argument('run_dir', type=str,
                       help='Redistricting run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Output directory (default: run_dir/demographic_analysis/maps)')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    run_dir = Path(args.run_dir)

    if not run_dir.exists():
        print(f"ERROR: Run directory not found: {run_dir}")
        return 1

    # Set output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = run_dir / 'demographic_analysis' / 'maps'

    output_dir.mkdir(parents=True, exist_ok=True)

    # Check if progress reporting is needed
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    is_standalone = not send_status

    if is_standalone:
        print("=" * 70)
        print("DEMOGRAPHIC VISUALIZATION")
        print("=" * 70)
        print(f"Run: {run_dir}")
        print(f"Output: {output_dir}")
        print("=" * 70)
        print()

    # Define output files
    gender_map = output_dir / 'gender_balance.png'
    majority_map = output_dir / 'majority_race.png'
    diversity_map = output_dir / 'diversity_index.png'

    # Check if all outputs exist
    if not args.force and gender_map.exists() and majority_map.exists() and diversity_map.exists():
        if is_standalone:
            print("Demographic maps already exist - skipping")
            print(f"  {gender_map.name}")
            print(f"  {majority_map.name}")
            print(f"  {diversity_map.name}")
            print("\nUse --force to regenerate")
        return 0

    try:
        # Detect state from directory name
        dir_name = run_dir.name
        STATE_NAME_TO_CODE = {
            'alabama': 'AL', 'alaska': 'AK', 'arizona': 'AZ', 'arkansas': 'AR', 'california': 'CA',
            'colorado': 'CO', 'connecticut': 'CT', 'delaware': 'DE', 'florida': 'FL', 'georgia': 'GA',
            'hawaii': 'HI', 'idaho': 'ID', 'illinois': 'IL', 'indiana': 'IN', 'iowa': 'IA',
            'kansas': 'KS', 'kentucky': 'KY', 'louisiana': 'LA', 'maine': 'ME', 'maryland': 'MD',
            'massachusetts': 'MA', 'michigan': 'MI', 'minnesota': 'MN', 'mississippi': 'MS', 'missouri': 'MO',
            'montana': 'MT', 'nebraska': 'NE', 'nevada': 'NV', 'new_hampshire': 'NH', 'new_jersey': 'NJ',
            'new_mexico': 'NM', 'new_york': 'NY', 'north_carolina': 'NC', 'north_dakota': 'ND', 'ohio': 'OH',
            'oklahoma': 'OK', 'oregon': 'OR', 'pennsylvania': 'PA', 'rhode_island': 'RI', 'south_carolina': 'SC',
            'south_dakota': 'SD', 'tennessee': 'TN', 'texas': 'TX', 'utah': 'UT', 'vermont': 'VT',
            'virginia': 'VA', 'washington': 'WA', 'west_virginia': 'WV', 'wisconsin': 'WI', 'wyoming': 'WY'
        }

        state_code = STATE_NAME_TO_CODE.get(dir_name)
        if not state_code:
            raise ValueError(f"Could not detect state from directory name: {dir_name}")

        state_name = dir_name.replace('_', ' ').title()

        # Load tract file with district assignments
        tracts_file = Path(f'data/raw/{state_code.lower()}_tracts_2020.parquet')
        if not tracts_file.exists():
            raise FileNotFoundError(f"Tract file not found: {tracts_file}")

        report_progress(f"Visualizing {state_name} demographics - Loading data...")
        tracts_gdf = gpd.read_parquet(tracts_file)

        # Load district assignments
        import pickle
        assignments_file = run_dir / 'final_assignments.pkl'
        if not assignments_file.exists():
            raise FileNotFoundError(f"Assignments not found: {assignments_file}")

        with open(assignments_file, 'rb') as f:
            assignments_by_index = pickle.load(f)

        # Map indices to GEOIDs and add district column
        geoid_to_district = {}
        for idx, district in assignments_by_index.items():
            if idx < len(tracts_gdf):
                geoid = str(tracts_gdf.iloc[idx]['GEOID']).zfill(11)
                geoid_to_district[geoid] = district

        tracts_gdf['district'] = tracts_gdf['GEOID'].astype(str).str.zfill(11).map(geoid_to_district)

        # Load demographic statistics
        demo_file = run_dir / 'demographic_analysis' / 'district_demographics.csv'
        if not demo_file.exists():
            raise FileNotFoundError(f"Demographic statistics not found: {demo_file}\n"
                                  f"Run analyze_district_demographics.py first.")

        district_stats = pd.read_csv(demo_file)

        # Create maps
        report_progress(f"Visualizing {state_name} demographics - Creating gender map...")
        create_gender_map(tracts_gdf, district_stats, state_name, gender_map, args.dpi)

        report_progress(f"Visualizing {state_name} demographics - Creating majority race map...")
        create_majority_race_map(tracts_gdf, district_stats, state_name, majority_map, args.dpi)

        report_progress(f"Visualizing {state_name} demographics - Creating diversity map...")
        create_diversity_map(tracts_gdf, district_stats, state_name, diversity_map, args.dpi)

        if is_standalone:
            print("\n" + "=" * 70)
            print("VISUALIZATION COMPLETE!")
            print("=" * 70)
            print("Created 3 demographic maps:")
            print(f"  1. Gender Balance: {gender_map}")
            print(f"  2. Majority Race: {majority_map}")
            print(f"  3. Diversity Index: {diversity_map}")
            print("=" * 70)

        return 0

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    exit(main())
