#!/usr/bin/env python3
"""
Visualize partisan lean of redistricting results.

This script creates maps showing the political characteristics of districts
and intermediate rounds, color-coded by partisan lean.
"""

import warnings
import os
os.environ['MPLBACKEND'] = 'Agg'
warnings.filterwarnings('ignore')

import pandas as pd
import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
plt.rcParams['figure.max_open_warning'] = 0
import matplotlib.patches as mpatches
import matplotlib.patheffects as path_effects
import argparse
from pathlib import Path
import pickle
import json
import numpy as np


# Political lean color mapping
LEAN_COLORS = {
    'Strong D': '#0015BC',    # Dark blue
    'Lean D': '#3A5FCD',      # Medium blue
    'Tilt D': '#6495ED',      # Light blue
    'Tossup': '#9370DB',      # Purple
    'Tilt R': '#FF6B6B',      # Light red
    'Lean R': '#DC143C',      # Medium red
    'Strong R': '#8B0000',    # Dark red
    'No Data': '#CCCCCC'      # Gray
}

LEAN_ORDER = ['Strong D', 'Lean D', 'Tilt D', 'Tossup', 'Tilt R', 'Lean R', 'Strong R', 'No Data']


def visualize_final_districts(run_dir, analysis_dir, tracts_gdf, state_name, year, dpi=150):
    """Create map of final districts colored by partisan lean."""

    # Load political analysis
    political_file = analysis_dir / f'district_political_{year}.csv'
    if not political_file.exists():
        print(f"Political analysis not found: {political_file}")
        return

    political_df = pd.read_csv(political_file)

    # Load assignments
    assignments_file = run_dir / 'final_assignments.pkl'
    if not assignments_file.exists():
        print(f"Assignments not found: {assignments_file}")
        return

    with open(assignments_file, 'rb') as f:
        assignments_by_index = pickle.load(f)

    # Map tract index to district
    tracts_gdf['district'] = tracts_gdf.index.map(assignments_by_index)

    # Join with political data
    tracts_gdf = tracts_gdf.merge(
        political_df[['district', 'lean', 'dem_margin']],
        on='district',
        how='left'
    )

    # Create figure with space for table
    fig = plt.figure(figsize=(20, 14))
    # Map takes 75% width, table takes 25%
    ax_map = plt.subplot2grid((1, 4), (0, 0), colspan=3)
    ax_table = plt.subplot2grid((1, 4), (0, 3))

    # Plot each lean category
    for lean in LEAN_ORDER:
        if lean in tracts_gdf['lean'].values:
            data = tracts_gdf[tracts_gdf['lean'] == lean]
            data.plot(
                ax=ax_map,
                color=LEAN_COLORS[lean],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.9
            )

    # Add thick district boundaries on top
    districts_dissolved = tracts_gdf.dissolve(by='district', as_index=False)
    districts_dissolved.boundary.plot(
        ax=ax_map,
        edgecolor='black',
        linewidth=1.5,
        zorder=10
    )

    # Add district numbers (just numbers, no margins)
    num_districts = political_df['district'].nunique()
    if num_districts <= 100:
        # Match fontsize logic from visualize_all_rounds.py
        if num_districts <= 4:
            fontsize = 40
        elif num_districts <= 8:
            fontsize = 28
        elif num_districts <= 16:
            fontsize = 18
        elif num_districts <= 32:
            fontsize = 12
        else:  # 52+ districts
            fontsize = 8

        for district in political_df['district']:
            district_data = tracts_gdf[tracts_gdf['district'] == district]
            if len(district_data) > 0:
                try:
                    centroid = district_data.geometry.union_all().representative_point()
                    text = ax_map.text(centroid.x, centroid.y, str(district),
                            fontsize=fontsize, fontweight='bold', ha='center', va='center',
                            color='white', zorder=10)
                    text.set_path_effects([
                        path_effects.Stroke(linewidth=2, foreground='black'),
                        path_effects.Normal()
                    ])
                except:
                    pass

    ax_map.set_axis_off()

    # Title
    title = f'{state_name} - {num_districts} Congressional Districts\n'
    title += f'2020 Presidential Election Results by District'
    ax_map.set_title(title, fontsize=18, fontweight='bold', pad=20)

    # Legend
    legend_elements = [
        mpatches.Patch(facecolor=LEAN_COLORS[lean], edgecolor='black', label=lean)
        for lean in LEAN_ORDER
        if lean in tracts_gdf['lean'].values
    ]

    ax_map.legend(handles=legend_elements, loc='lower right', fontsize=10,
                 title='Partisan Lean', title_fontsize=11, framealpha=0.9)

    # Create table showing districts with margins
    ax_table.axis('off')

    # Sort by district number
    table_data = political_df[['district', 'dem_margin', 'lean']].copy()
    table_data = table_data.sort_values('district')

    # Format margin as D+X or R+X
    def format_margin(margin):
        if margin >= 0:
            return f'D+{margin:.0f}'
        else:
            return f'R+{abs(margin):.0f}'

    table_data['Margin'] = table_data['dem_margin'].apply(format_margin)

    # Create table (show all districts in multiple columns if many)
    if num_districts <= 20:
        # Single column
        cell_text = [[f"{row['district']}", row['Margin']]
                    for _, row in table_data.iterrows()]
        col_labels = ['#', 'Margin']
        table = ax_table.table(cellText=cell_text, colLabels=col_labels,
                              cellLoc='left', loc='upper left',
                              colWidths=[0.3, 0.7])
    else:
        # Split into two columns for many districts
        mid = (len(table_data) + 1) // 2
        left_half = table_data.iloc[:mid]
        right_half = table_data.iloc[mid:]

        cell_text = []
        for i in range(max(len(left_half), len(right_half))):
            row = []
            if i < len(left_half):
                row.extend([f"{left_half.iloc[i]['district']}", left_half.iloc[i]['Margin']])
            else:
                row.extend(['', ''])
            if i < len(right_half):
                row.extend([f"{right_half.iloc[i]['district']}", right_half.iloc[i]['Margin']])
            else:
                row.extend(['', ''])
            cell_text.append(row)

        col_labels = ['#', 'Margin', '#', 'Margin']
        table = ax_table.table(cellText=cell_text, colLabels=col_labels,
                              cellLoc='left', loc='upper left',
                              colWidths=[0.12, 0.28, 0.12, 0.28])

    table.auto_set_font_size(False)
    table.set_fontsize(7 if num_districts > 30 else 9)
    table.scale(1, 1.5)

    # Color header
    for i in range(len(col_labels)):
        table[(0, i)].set_facecolor('#E0E0E0')
        table[(0, i)].set_text_props(weight='bold')

    # Color-code each row by partisan lean
    if num_districts <= 20:
        # Single column layout
        for idx, (_, row) in enumerate(table_data.iterrows()):
            cell_row = idx + 1  # +1 for header
            lean = row['lean']
            color = LEAN_COLORS.get(lean, '#CCCCCC')
            table[(cell_row, 0)].set_facecolor(color)
            table[(cell_row, 1)].set_facecolor(color)
            table[(cell_row, 0)].set_text_props(weight='bold', color='white')
            table[(cell_row, 1)].set_text_props(weight='bold', color='white')
    else:
        # Two column layout
        mid = (len(table_data) + 1) // 2
        for i in range(max(mid, len(table_data) - mid)):
            cell_row = i + 1  # +1 for header
            # Left side
            if i < mid:
                lean = table_data.iloc[i]['lean']
                color = LEAN_COLORS.get(lean, '#CCCCCC')
                table[(cell_row, 0)].set_facecolor(color)
                table[(cell_row, 1)].set_facecolor(color)
                table[(cell_row, 0)].set_text_props(weight='bold', color='white')
                table[(cell_row, 1)].set_text_props(weight='bold', color='white')
            # Right side
            if mid + i < len(table_data):
                lean = table_data.iloc[mid + i]['lean']
                color = LEAN_COLORS.get(lean, '#CCCCCC')
                table[(cell_row, 2)].set_facecolor(color)
                table[(cell_row, 3)].set_facecolor(color)
                table[(cell_row, 2)].set_text_props(weight='bold', color='white')
                table[(cell_row, 3)].set_text_props(weight='bold', color='white')

    # Add D/R seat count annotation to map
    # Count seats where D has majority (dem_margin >= 0)
    d_seats = len(political_df[political_df['dem_margin'] >= 0])
    r_seats = len(political_df[political_df['dem_margin'] < 0])

    # Add text box in upper-right corner
    ax_map.text(0.98, 0.98, f'D: {d_seats} | R: {r_seats}',
                transform=ax_map.transAxes,
                fontsize=16,
                fontweight='bold',
                verticalalignment='top',
                horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black', linewidth=2),
                zorder=100)

    plt.tight_layout()

    # Save
    output_dir = analysis_dir / 'maps'
    output_dir.mkdir(parents=True, exist_ok=True)
    map_file = output_dir / f'partisan_lean_districts_{year}.png'
    plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
    plt.close(fig)

    print(f"Saved: {map_file}")


def visualize_intermediate_rounds(run_dir, analysis_dir, tracts_gdf, state_name, year, dpi=150):
    """Create maps of intermediate rounds colored by partisan lean."""

    # Load political analysis
    rounds_file = analysis_dir / f'rounds_political_{year}.csv'
    if not rounds_file.exists():
        print(f"Rounds analysis not found: {rounds_file}")
        return

    rounds_df = pd.read_csv(rounds_file)

    intermediate_dir = run_dir / 'intermediate'
    if not intermediate_dir.exists():
        print(f"Intermediate directory not found: {intermediate_dir}")
        return

    # Get unique rounds
    unique_rounds = sorted(rounds_df['round'].unique())

    output_dir = analysis_dir / 'maps' / 'rounds'
    output_dir.mkdir(parents=True, exist_ok=True)

    for round_num in unique_rounds:
        round_data = rounds_df[rounds_df['round'] == round_num]
        num_regions = round_data['num_regions'].iloc[0]

        # Load assignments for this round
        assignments_file = intermediate_dir / f'round_{round_num}_{num_regions}_regions_assignments.json'
        if not assignments_file.exists():
            continue

        with open(assignments_file, 'r') as f:
            assignments_by_index_str = json.load(f)

        # Map tract index to region
        region_map = {}
        for idx_str, region in assignments_by_index_str.items():
            idx = int(idx_str)
            if idx < len(tracts_gdf):
                region_map[idx] = region

        tracts_gdf['region'] = tracts_gdf.index.map(region_map)

        # Join with political data (region is 0-based in assignments, 1-based in CSV)
        tracts_gdf['region_1based'] = tracts_gdf['region'] + 1
        tracts_gdf = tracts_gdf.merge(
            round_data[['region', 'lean', 'dem_margin']],
            left_on='region_1based',
            right_on='region',
            how='left',
            suffixes=('', '_political')
        )

        # Create figure with space for table
        fig = plt.figure(figsize=(20, 14))
        ax_map = plt.subplot2grid((1, 4), (0, 0), colspan=3)
        ax_table = plt.subplot2grid((1, 4), (0, 3))

        # Plot each lean category
        for lean in LEAN_ORDER:
            if lean in tracts_gdf['lean'].values:
                data = tracts_gdf[tracts_gdf['lean'] == lean]
                data.plot(
                    ax=ax_map,
                    color=LEAN_COLORS[lean],
                    edgecolor='white',
                    linewidth=0.1,
                    alpha=0.9
                )

        # Add thick region boundaries on top
        regions_dissolved = tracts_gdf.dissolve(by='region', as_index=False)
        regions_dissolved.boundary.plot(
            ax=ax_map,
            edgecolor='black',
            linewidth=1.5,
            zorder=10
        )

        # Add region numbers (just numbers, no margins)
        if num_regions <= 100:
            # Match fontsize logic from visualize_all_rounds.py
            if num_regions <= 4:
                fontsize = 40
            elif num_regions <= 8:
                fontsize = 28
            elif num_regions <= 16:
                fontsize = 18
            elif num_regions <= 32:
                fontsize = 12
            else:  # 52+ regions
                fontsize = 8

            for region_id in range(num_regions):
                region_tracts = tracts_gdf[tracts_gdf['region'] == region_id]
                if len(region_tracts) > 0:
                    try:
                        centroid = region_tracts.geometry.union_all().representative_point()
                        text = ax_map.text(centroid.x, centroid.y, str(region_id + 1),
                                fontsize=fontsize, fontweight='bold', ha='center', va='center',
                                color='white', zorder=10)
                        text.set_path_effects([
                            path_effects.Stroke(linewidth=2, foreground='black'),
                            path_effects.Normal()
                        ])
                    except:
                        pass

        ax_map.set_axis_off()

        # Title
        title = f'{state_name} - Round {round_num}: {num_regions} Regions\n'
        title += f'2020 Presidential Election Results by Region'
        ax_map.set_title(title, fontsize=18, fontweight='bold', pad=20)

        # Legend
        legend_elements = [
            mpatches.Patch(facecolor=LEAN_COLORS[lean], edgecolor='black', label=lean)
            for lean in LEAN_ORDER
            if lean in tracts_gdf['lean'].values
        ]

        ax_map.legend(handles=legend_elements, loc='lower right', fontsize=10,
                     title='Partisan Lean', title_fontsize=11, framealpha=0.9)

        # Create table showing regions with margins
        ax_table.axis('off')

        # Sort by region number
        table_data = round_data[['region', 'dem_margin', 'lean']].copy()
        table_data = table_data.sort_values('region')

        # Format margin as D+X or R+X
        def format_margin(margin):
            if margin >= 0:
                return f'D+{margin:.0f}'
            else:
                return f'R+{abs(margin):.0f}'

        table_data['Margin'] = table_data['dem_margin'].apply(format_margin)

        # Create table (adjust columns based on number of regions)
        if num_regions <= 20:
            # Single column
            cell_text = [[f"{row['region']}", row['Margin']]
                        for _, row in table_data.iterrows()]
            col_labels = ['#', 'Margin']
            table = ax_table.table(cellText=cell_text, colLabels=col_labels,
                                  cellLoc='left', loc='upper left',
                                  colWidths=[0.3, 0.7])
        else:
            # Split into two columns for many regions
            mid = (len(table_data) + 1) // 2
            left_half = table_data.iloc[:mid]
            right_half = table_data.iloc[mid:]

            cell_text = []
            for i in range(max(len(left_half), len(right_half))):
                row = []
                if i < len(left_half):
                    row.extend([f"{left_half.iloc[i]['region']}", left_half.iloc[i]['Margin']])
                else:
                    row.extend(['', ''])
                if i < len(right_half):
                    row.extend([f"{right_half.iloc[i]['region']}", right_half.iloc[i]['Margin']])
                else:
                    row.extend(['', ''])
                cell_text.append(row)

            col_labels = ['#', 'Margin', '#', 'Margin']
            table = ax_table.table(cellText=cell_text, colLabels=col_labels,
                                  cellLoc='left', loc='upper left',
                                  colWidths=[0.15, 0.35, 0.15, 0.35])

        table.auto_set_font_size(False)
        table.set_fontsize(7 if num_regions > 30 else 9)
        table.scale(1, 1.5)

        # Color header
        for i in range(len(col_labels)):
            table[(0, i)].set_facecolor('#E0E0E0')
            table[(0, i)].set_text_props(weight='bold')

        # Color-code each row by partisan lean
        if num_regions <= 20:
            # Single column layout
            for idx, (_, row) in enumerate(table_data.iterrows()):
                cell_row = idx + 1  # +1 for header
                lean = row['lean']
                color = LEAN_COLORS.get(lean, '#CCCCCC')
                table[(cell_row, 0)].set_facecolor(color)
                table[(cell_row, 1)].set_facecolor(color)
                table[(cell_row, 0)].set_text_props(weight='bold', color='white')
                table[(cell_row, 1)].set_text_props(weight='bold', color='white')
        else:
            # Two column layout
            mid = (len(table_data) + 1) // 2
            for i in range(max(mid, len(table_data) - mid)):
                cell_row = i + 1  # +1 for header
                # Left side
                if i < mid:
                    lean = table_data.iloc[i]['lean']
                    color = LEAN_COLORS.get(lean, '#CCCCCC')
                    table[(cell_row, 0)].set_facecolor(color)
                    table[(cell_row, 1)].set_facecolor(color)
                    table[(cell_row, 0)].set_text_props(weight='bold', color='white')
                    table[(cell_row, 1)].set_text_props(weight='bold', color='white')
                # Right side
                if mid + i < len(table_data):
                    lean = table_data.iloc[mid + i]['lean']
                    color = LEAN_COLORS.get(lean, '#CCCCCC')
                    table[(cell_row, 2)].set_facecolor(color)
                    table[(cell_row, 3)].set_facecolor(color)
                    table[(cell_row, 2)].set_text_props(weight='bold', color='white')
                    table[(cell_row, 3)].set_text_props(weight='bold', color='white')

        # Add D/R region count annotation to map
        # Count regions where D has majority (dem_margin >= 0)
        d_regions = len(round_data[round_data['dem_margin'] >= 0])
        r_regions = len(round_data[round_data['dem_margin'] < 0])

        # Add text box in upper-right corner
        ax_map.text(0.98, 0.98, f'D: {d_regions} | R: {r_regions}',
                    transform=ax_map.transAxes,
                    fontsize=16,
                    fontweight='bold',
                    verticalalignment='top',
                    horizontalalignment='right',
                    bbox=dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black', linewidth=2),
                    zorder=100)

        plt.tight_layout()

        # Save
        map_file = output_dir / f'partisan_lean_round_{round_num}_{num_regions}_regions_{year}.png'
        plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
        plt.close(fig)

        print(f"Saved: {map_file}")

        # Clean up temporary columns
        tracts_gdf.drop(columns=['region_1based', 'region_political', 'lean', 'dem_margin'], inplace=True, errors='ignore')


def visualize_state_political(state_dir, state_code, election_year, census_year, dpi=150, skip_rounds=False):
    """Visualize political lean for a single state."""
    state_dir = Path(state_dir)

    # State code to name mapping
    STATE_CODE_TO_NAME = {
        'AL': 'alabama', 'AK': 'alaska', 'AZ': 'arizona', 'AR': 'arkansas', 'CA': 'california',
        'CO': 'colorado', 'CT': 'connecticut', 'DE': 'delaware', 'FL': 'florida', 'GA': 'georgia',
        'HI': 'hawaii', 'ID': 'idaho', 'IL': 'illinois', 'IN': 'indiana', 'IA': 'iowa',
        'KS': 'kansas', 'KY': 'kentucky', 'LA': 'louisiana', 'ME': 'maine', 'MD': 'maryland',
        'MA': 'massachusetts', 'MI': 'michigan', 'MN': 'minnesota', 'MS': 'mississippi', 'MO': 'missouri',
        'MT': 'montana', 'NE': 'nebraska', 'NV': 'nevada', 'NH': 'new_hampshire', 'NJ': 'new_jersey',
        'NM': 'new_mexico', 'NY': 'new_york', 'NC': 'north_carolina', 'ND': 'north_dakota', 'OH': 'ohio',
        'OK': 'oklahoma', 'OR': 'oregon', 'PA': 'pennsylvania', 'RI': 'rhode_island', 'SC': 'south_carolina',
        'SD': 'south_dakota', 'TN': 'tennessee', 'TX': 'texas', 'UT': 'utah', 'VT': 'vermont',
        'VA': 'virginia', 'WA': 'washington', 'WV': 'west_virginia', 'WI': 'wisconsin', 'WY': 'wyoming'
    }

    state_name_lower = STATE_CODE_TO_NAME.get(state_code.upper())
    if not state_name_lower:
        print(f"ERROR: Unknown state code: {state_code}")
        return 1

    state_name = state_name_lower.replace('_', ' ').title()

    # Check analysis directory exists
    analysis_dir = state_dir / 'political_analysis'
    political_file = analysis_dir / f'district_political_{election_year}.csv'

    if not political_file.exists():
        print(f"ERROR: Political analysis not found: {political_file}")
        print(f"Run analyze_districts.py first")
        return 1

    # Load tract geometries
    tracts_file = Path(f'data/raw/{state_code.lower()}_tracts_{census_year}.parquet')
    if not tracts_file.exists():
        print(f"ERROR: Tract geometries not found: {tracts_file}")
        return 1

    tracts_gdf = gpd.read_parquet(tracts_file)

    print(f"Creating political visualizations for {state_name}...")

    # Visualize final districts
    print("  Creating final districts map...")
    visualize_final_districts(state_dir, analysis_dir, tracts_gdf.copy(), state_name, election_year, dpi)

    # Visualize intermediate rounds
    if not skip_rounds:
        print("  Creating intermediate rounds maps...")
        visualize_intermediate_rounds(state_dir, analysis_dir, tracts_gdf.copy(), state_name, election_year, dpi)

    print(f"  Political visualizations complete for {state_name}")
    return 0


def main():
    parser = argparse.ArgumentParser(description='Visualize partisan lean of districts at state or national scope')

    # Scope-based design
    parser.add_argument('--scope', choices=['state', 'national'], default='national',
                       help='Scope: state (single state) or national (all states, default)')
    parser.add_argument('--election-year', type=str, default='2020', choices=['2020', '2016'],
                       help='Election year for political data')
    parser.add_argument('--census-year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year for tract data')

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
    parser.add_argument('--skip-rounds', action='store_true',
                       help='Skip visualizing intermediate rounds (state scope only)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    parser.add_argument('--position', type=int, default=-1,
                       help='Progress bar position (for parent coordination)')

    args = parser.parse_args()

    # Validate scope-specific requirements
    if args.scope == 'state':
        if not args.state or not args.state_dir:
            parser.error("--state and --state-dir required when scope=state")
        return visualize_state_political(args.state_dir, args.state, args.election_year,
                                        args.census_year, args.dpi, args.skip_rounds)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")

        # Delegate to existing national map script for now
        # TODO: Merge logic directly into this script
        import subprocess
        cmd = [
            sys.executable,
            str(Path(__file__).parent / 'create_us_national_political_map.py'),
            '--year', args.election_year,
            '--version', args.version,
            '--output-dir', args.output_dir,
            '--dpi', str(args.dpi)
        ]
        if args.force:
            cmd.append('--force')
        if args.position >= 0:
            cmd.extend(['--position', str(args.position)])

        result = subprocess.run(cmd)
        return result.returncode




if __name__ == '__main__':
    exit(main())
