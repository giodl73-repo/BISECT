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
import pickle
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_tract_file

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


def visualize_state_demographics(state_dir, state_code, census_year, dpi=150, force=False):
    """Visualize demographics for a single state."""
    state_dir = Path(state_dir)

    # Check demographic analysis exists
    analysis_file = state_dir / 'demographic' / 'district_demographics.csv'
    if not analysis_file.exists():
        print(f"ERROR: Demographic analysis not found: {analysis_file}")
        print(f"Run analyze_district_demographics.py first")
        return 1

    # CHECK IF OUTPUTS EXIST BEFORE LOADING DATA
    output_dir = state_dir / 'demographic' / 'maps'
    required_maps = [
        output_dir / 'gender_balance.png',
        output_dir / 'majority_race.png',
        output_dir / 'diversity_index.png'
    ]

    if not force and all(m.exists() for m in required_maps):
        print(f"Demographic visualization already exists for {state_code} - skipping")
        return 0

    output_dir.mkdir(parents=True, exist_ok=True)

    # Now load data (only if we need to generate)
    state_code_lower = state_code.lower()
    tracts_file = get_tract_file(state_code, str(census_year))

    if not tracts_file.exists():
        print(f"ERROR: Tract geometries not found: {tracts_file}")
        return 1

    # Load demographic data
    demo_df = pd.read_csv(analysis_file)

    # Load tract data and assignments
    tracts_gdf = gpd.read_parquet(tracts_file)
    assignments_file = state_dir / 'data' / 'final_assignments.pkl'
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    tracts_gdf['district'] = tracts_gdf.index.map(assignments)

    # Create maps (functions expect tracts_gdf, district_stats, state_name, output, dpi)
    state_name = state_code  # Just use state code as name for now

    print("  Creating gender balance map...")
    create_gender_map(tracts_gdf.copy(), demo_df, state_name, output_dir / 'gender_balance.png', dpi)

    print("  Creating majority race map...")
    create_majority_race_map(tracts_gdf.copy(), demo_df, state_name, output_dir / 'majority_race.png', dpi)

    print("  Creating diversity index map...")
    create_diversity_map(tracts_gdf.copy(), demo_df, state_name, output_dir / 'diversity_index.png', dpi)

    print(f"  Demographic visualizations complete")
    return 0


def main():
    parser = argparse.ArgumentParser(description='Visualize district demographics at state or national scope')

    # Scope-based design
    parser.add_argument('--scope', choices=['state', 'national'], default='national',
                       help='Scope: state (single state) or national (all states, default)')
    parser.add_argument('--census-year', type=str, default='2020', choices=['2020', '2010'],
                       help='Census year')

    # State scope arguments
    parser.add_argument('--state', type=str,
                       help='State code (2-letter, required if scope=state)')
    parser.add_argument('--state-dir', type=str,
                       help='State directory (required if scope=state)')

    # National scope arguments
    parser.add_argument('--output-dir', type=str,
                       help='Base output directory (required if scope=national)')
    parser.add_argument('--version', type=str,
                       help='Version (required if scope=national)')

    # Common arguments
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    parser.add_argument('--position', type=int, default=-1,
                       help='Progress bar position (for parent coordination)')

    args = parser.parse_args()

    # Validate scope-specific requirements
    if args.scope == 'state':
        if not args.state or not args.state_dir:
            parser.error("--state and --state-dir required when scope=state")
        return visualize_state_demographics(args.state_dir, args.state, args.census_year, args.dpi, args.force)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")

        # TODO: Implement national demographic visualization logic
        # (Similar to visualize_partisan_lean.py::visualize_national_political)
        # For now, print message and skip
        print("[SKIP] National demographic visualization - not yet implemented in consolidated script")
        print("       Will be implemented in follow-up enhancement")
        return 0




if __name__ == '__main__':
    exit(main())
