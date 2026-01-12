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


def main():
    parser = argparse.ArgumentParser(description='Visualize partisan lean of districts')
    parser.add_argument('state', type=str, nargs='?', default=None,
                       help='State name or code (e.g., california or CA), or full run_dir path')
    parser.add_argument('--election-year', type=str, default='2020', choices=['2020', '2016'],
                       help='Election year for political data (default: 2020)')
    parser.add_argument('--census-year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year for tract data (default: 2020)')
    parser.add_argument('--version', type=str, default='v1',
                       help='Version identifier (default: v1)')
    parser.add_argument('--output-dir', type=str, default=None,
                       help='Override output directory (default: outputs/us_{census_year}_{version}/states/{state})')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--skip-rounds', action='store_true',
                       help='Skip visualizing intermediate rounds')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    # State name to code mapping
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
    STATE_CODE_TO_NAME = {v.lower(): k for k, v in STATE_NAME_TO_CODE.items()}

    # Determine run directory
    if args.output_dir:
        # Use explicit output directory override
        run_dir = Path(args.output_dir)
        # Extract state from directory name
        dir_name = run_dir.name
        state_code = STATE_NAME_TO_CODE.get(dir_name)
        if not state_code:
            raise ValueError(f"Could not detect state from directory name: {dir_name}")
        state_name = dir_name.replace('_', ' ').title()
    elif args.state:
        # Check if state is a full path or just state name/code
        state_path = Path(args.state)
        if state_path.exists() and state_path.is_dir():
            # Full path provided
            run_dir = state_path
            dir_name = run_dir.name
            state_code = STATE_NAME_TO_CODE.get(dir_name)
            if not state_code:
                raise ValueError(f"Could not detect state from directory name: {dir_name}")
            state_name = dir_name.replace('_', ' ').title()
        else:
            # State name or code provided - construct path
            state_input = args.state.lower()
            if state_input in STATE_NAME_TO_CODE:
                # State name
                state_code = STATE_NAME_TO_CODE[state_input]
                state_name = state_input.replace('_', ' ').title()
                dir_name = state_input
            elif state_input in STATE_CODE_TO_NAME:
                # State code
                state_code = state_input.upper()
                dir_name = STATE_CODE_TO_NAME[state_input]
                state_name = dir_name.replace('_', ' ').title()
            else:
                raise ValueError(f"Unknown state: {args.state}")

            # Construct run directory
            run_dir = Path(f'outputs/us_{args.census_year}_{args.version}/states/{dir_name}')
    else:
        print("ERROR: Must provide state name/code or use --output-dir")
        return 1

    run_dir = Path(run_dir)

    if not run_dir.exists():
        print(f"ERROR: Run directory not found: {run_dir}")
        return 1

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

    # Check if political analysis exists
    analysis_dir = run_dir / 'political_analysis'
    if not analysis_dir.exists():
        print(f"ERROR: Political analysis not found: {analysis_dir}")
        print(f"Run analyze_districts.py first")
        return 1

    # Load tract file
    tracts_file = Path(f'data/raw/{state_code.lower()}_tracts_{args.census_year}.parquet')
    if not tracts_file.exists():
        print(f"ERROR: Tract file not found: {tracts_file}")
        return 1

    # Check if being called from parent pipeline
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0

    def report_progress(msg):
        """Report progress to parent pipeline."""
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    is_standalone = not send_status

    if is_standalone:
        print("="*70)
        print("VISUALIZING PARTISAN LEAN")
        print("="*70)
        print(f"State: {state_name} ({state_code})")
        print(f"Run: {run_dir}")
        print(f"Census Year: {args.census_year}")
        print(f"Election Year: {args.election_year}")
        print("="*70)
        print()

    # Check if maps already exist
    maps_dir = analysis_dir / 'maps'
    district_map = maps_dir / f'partisan_lean_districts_{args.election_year}.png'
    rounds_dir = maps_dir / 'rounds'

    if not args.force and district_map.exists():
        # Check if we should skip
        skip = True
        if not args.skip_rounds:
            # Also check if round maps exist
            if rounds_dir.exists():
                round_maps = list(rounds_dir.glob(f'partisan_lean_round_*_{args.election_year}.png'))
                # If there are some round maps, assume it's complete
                skip = len(round_maps) > 0
            else:
                skip = False

        if skip:
            print("Partisan lean maps already exist - skipping")
            print(f"  {district_map}")
            if not args.skip_rounds and rounds_dir.exists():
                print(f"  {rounds_dir}/*.png")
            print("\nUse --force to regenerate")
            return 0

    try:
        report_progress(f"Visualizing {state_name} - Loading tract data...")
        if is_standalone:
            print("Loading tract data...")
        tracts_gdf = gpd.read_parquet(tracts_file)
        if is_standalone:
            print(f"Loaded {len(tracts_gdf):,} tracts")
            print()

        # Visualize final districts
        report_progress(f"Visualizing {state_name} - Creating final districts map...")
        if is_standalone:
            print("Creating final districts map...")
        visualize_final_districts(run_dir, analysis_dir, tracts_gdf.copy(), state_name, args.election_year, args.dpi)
        if is_standalone:
            print()

        # Visualize intermediate rounds
        if not args.skip_rounds:
            report_progress(f"Visualizing {state_name} - Creating intermediate rounds maps...")
            if is_standalone:
                print("Creating intermediate rounds maps...")
            visualize_intermediate_rounds(run_dir, analysis_dir, tracts_gdf.copy(), state_name, args.election_year, args.dpi)
        else:
            if is_standalone:
                print("Skipping intermediate rounds (--skip-rounds)")
        if is_standalone:
            print()

        if is_standalone:
            print("="*70)
            print("VISUALIZATION COMPLETE!")
            print("="*70)

        return 0

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    exit(main())
