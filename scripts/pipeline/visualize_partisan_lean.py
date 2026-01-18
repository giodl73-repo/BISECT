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

import sys
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
from shapely.ops import unary_union

# Import shared constants
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.constants import STATE_ABBREV
from scripts.utils import build_state_name_to_districts_map


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

    # Load political analysis (no year suffix)
    political_file = analysis_dir / 'political' / 'district_political.csv'
    if not political_file.exists():
        print(f"Political analysis not found: {political_file}")
        return

    political_df = pd.read_csv(political_file)

    # Skip if empty (AK/HI - no election data)
    if len(political_df) == 0:
        print(f"Political data empty for {state_name} (no tract-level election data) - skipping visualization")
        return

    # Load assignments from data/ subdirectory
    assignments_file = run_dir / 'data' / 'final_assignments.pkl'
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

    # Save to political/maps subdirectory (no year suffix)
    output_dir = analysis_dir / 'political' / 'maps'
    output_dir.mkdir(parents=True, exist_ok=True)
    map_file = output_dir / 'partisan_lean.png'
    plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
    plt.close(fig)

    print(f"Saved: {map_file}")


def visualize_intermediate_rounds(run_dir, analysis_dir, tracts_gdf, state_name, year, dpi=150):
    """Create maps of intermediate rounds colored by partisan lean."""

    # Load political analysis (no year suffix)
    rounds_file = analysis_dir / 'political' / 'rounds_political.csv'
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

    output_dir = analysis_dir / 'political' / 'maps' / 'rounds'
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

        # Save with zero-padded round number (no year/region count suffix)
        map_file = output_dir / f'round_{round_num:02d}.png'
        plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
        plt.close(fig)

        print(f"Saved: {map_file}")

        # Clean up temporary columns
        tracts_gdf.drop(columns=['region_1based', 'region_political', 'lean', 'dem_margin'], inplace=True, errors='ignore')


def visualize_state_political(state_dir, state_code, election_year, census_year, dpi=150, skip_rounds=False, force=False):
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

    # Check political directory exists (no year suffix)
    analysis_dir = state_dir
    political_file = state_dir / 'political' / 'district_political.csv'

    if not political_file.exists():
        print(f"ERROR: Political analysis not found: {political_file}")
        print(f"Run analyze_districts.py first")
        return 1

    # CHECK IF OUTPUTS EXIST BEFORE LOADING DATA
    political_maps_dir = state_dir / 'political' / 'maps'
    required_maps = [
        political_maps_dir / 'partisan_lean.png'
    ]

    if not force and all(m.exists() for m in required_maps):
        print(f"Political visualization already exists for {state_name} - skipping")
        return 0

    # Now load tract geometries (only if we need to generate)
    state_code_lower = state_code.lower()
    tracts_file = Path(f'data/tracts/{census_year}/{state_code_lower}_tracts_{census_year}.parquet')

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


def visualize_national_political(output_dir, version, election_year, census_year, dpi=150, force=False, position=-1):
    """Create national political map showing all 435 congressional districts colored by partisan lean."""

    # Get position from args or environment
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Determine base directory
    base_dir = Path(output_dir)

    if not base_dir.exists():
        if is_standalone:
            print(f"ERROR: Base directory not found: {base_dir}")
        return 1

    # Create maps/political directory if it doesn't exist
    maps_dir = base_dir / 'maps' / 'political'
    maps_dir.mkdir(parents=True, exist_ok=True)

    # Output file (no year suffix)
    output_file = maps_dir / 'partisan_lean.png'

    # Check if output exists
    if not force and output_file.exists():
        if is_standalone:
            print(f"Output already exists: {output_file}")
            print("Use --force to regenerate")
        return 0

    # Main execution
    report_progress("Creating national political map - Loading data")

    if is_standalone:
        print(f"\nCreating national political map for {census_year} census...")
        print(f"Output: {output_file}")

    # Load all states with districts and political data
    try:
        # Build state name -> districts mapping from year-specific config
        districts_per_state = build_state_name_to_districts_map(census_year)

        all_tracts = []
        total_states = len(districts_per_state)
        processed = 0

        for state_name, num_districts in districts_per_state.items():
            # Skip Alaska and Hawaii (no 2020 presidential election data available)
            if state_name in ['alaska', 'hawaii']:
                continue

            state_dir = base_dir / 'states' / state_name

            # Skip if state not processed
            if not state_dir.exists():
                continue

            # Load tracts (use state abbreviation for filename)
            state_abbrev = STATE_ABBREV[state_name].lower()
            tracts_file = Path(f'data/tracts/{census_year}/{state_abbrev}_tracts_{census_year}.parquet')

            if not tracts_file.exists():
                continue

            tracts = gpd.read_parquet(tracts_file)

            # Load assignments from data/ subdirectory
            assignments_file = state_dir / 'data' / 'final_assignments.pkl'
            if not assignments_file.exists():
                continue

            with open(assignments_file, 'rb') as f:
                assignments = pickle.load(f)

            tracts['district'] = tracts.index.map(assignments)
            tracts['state'] = state_name
            tracts['state_code'] = STATE_ABBREV[state_name]
            tracts['unique_district_id'] = tracts.apply(
                lambda row: f"{row['state_code']}-{row['district']:02d}", axis=1
            )

            # Load political data if available (no year suffix)
            political_file = state_dir / 'political' / 'district_political.csv'
            if political_file.exists():
                try:
                    political_df = pd.read_csv(political_file)
                    # Merge political data
                    tracts = tracts.merge(
                        political_df[['district', 'lean', 'dem_margin']],
                        on='district',
                        how='left'
                    )
                except Exception as e:
                    # Skip states with problematic political data
                    if is_standalone:
                        print(f"  Warning: Skipping {state_name} - error loading political data: {e}")
                    continue
            else:
                tracts['lean'] = 'No Data'
                tracts['dem_margin'] = 0.0

            all_tracts.append(tracts)
            processed += 1

            if is_standalone and processed % 10 == 0:
                print(f"  Loaded {processed}/{total_states} states...")

        if not all_tracts:
            if is_standalone:
                print("ERROR: No state data found")
            return 1

        us_tracts = gpd.GeoDataFrame(pd.concat(all_tracts, ignore_index=True))

        if is_standalone:
            print(f"  Loaded {processed} states with {len(us_tracts):,} tracts")

        # Create single-panel figure for continental US only
        fig = plt.figure(figsize=(28, 18))
        ax_main = fig.add_subplot(111)

        # Plot function
        def plot_political_region(tracts_data, ax, region_name):
            if len(tracts_data) == 0:
                return

            # Plot each lean category
            for lean in LEAN_ORDER:
                if lean in tracts_data['lean'].values:
                    data = tracts_data[tracts_data['lean'] == lean]
                    data.plot(
                        ax=ax,
                        color=LEAN_COLORS[lean],
                        edgecolor='white',
                        linewidth=0.05,
                        alpha=0.9
                    )

            # Add district boundaries (dissolved tracts)
            region_districts = tracts_data.dissolve(by='unique_district_id')
            region_districts.boundary.plot(ax=ax, linewidth=1.5, edgecolor='black', alpha=0.8)

            # Add state boundaries
            region_states = tracts_data.dissolve(by='state_code')
            region_states.boundary.plot(ax=ax, linewidth=0.8, edgecolor='black', alpha=0.6)
            ax.set_axis_off()

        # Plot continental US (48 states)
        plot_political_region(us_tracts, ax_main, "Continental US")

        # Add D/R seat counts
        d_seats = len(us_tracts[us_tracts['dem_margin'] >= 0].drop_duplicates('unique_district_id'))
        r_seats = len(us_tracts[us_tracts['dem_margin'] < 0].drop_duplicates('unique_district_id'))
        total_districts = len(us_tracts['unique_district_id'].unique())

        # Title
        fig.suptitle(f'United States Congressional Districts - Political Lean\n'
                     f'2020 Presidential Election Results (48 States, {total_districts} Districts)',
                     fontsize=22, fontweight='bold', y=0.98)

        # Legend
        from matplotlib.patches import Patch
        legend_elements = [
            Patch(facecolor=LEAN_COLORS[lean], edgecolor='black', label=lean)
            for lean in ['Strong D', 'Lean D', 'Tilt D', 'Tossup', 'Tilt R', 'Lean R', 'Strong R']
            if lean in us_tracts['lean'].values
        ]
        ax_main.legend(handles=legend_elements, loc='lower right', fontsize=10,
                      title='Partisan Lean', title_fontsize=11, framealpha=0.9)

        # Add seat count
        textstr = f'D: {d_seats} | R: {r_seats}\n'
        textstr += f'Total Districts: {total_districts}'
        props = dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black', linewidth=2)
        ax_main.text(0.98, 0.98, textstr, transform=ax_main.transAxes, fontsize=14,
                    verticalalignment='top', horizontalalignment='right', bbox=props, fontweight='bold')

        plt.tight_layout(rect=[0, 0, 1, 0.97])

        # Save
        output_file.parent.mkdir(parents=True, exist_ok=True)
        plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
        plt.close(fig)

        if is_standalone:
            print(f"\nSaved: {output_file}")

        report_progress("Creating national political map - Complete")
        return 0

    except Exception as e:
        if is_standalone:
            print(f"\nERROR: {e}")
            import traceback
            traceback.print_exc()
        return 1


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
                                        args.census_year, args.dpi, args.skip_rounds, args.force)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")

        # Call the national visualization function directly (merged from create_us_national_political_map.py)
        return visualize_national_political(
            args.output_dir,
            args.version,
            args.election_year,
            args.census_year,
            args.dpi,
            args.force,
            args.position
        )




if __name__ == '__main__':
    exit(main())
