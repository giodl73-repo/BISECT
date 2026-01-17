#!/usr/bin/env python3
"""
Visualize district compactness scores.

Creates maps showing Polsby-Popper and Reock compactness scores
for districts at state or national scope.

Usage:
    # State scope (single state)
    python scripts/pipeline/visualize_compactness.py \
        --scope state \
        --state-dir outputs/us_2020_v1/states/vermont \
        --census-year 2020

    # National scope (all states)
    python scripts/pipeline/visualize_compactness.py \
        --scope national \
        --output-dir outputs/us_2020_v1 \
        --version v1 \
        --census-year 2020
"""

import sys
import os
import argparse
from pathlib import Path
import pandas as pd
import geopandas as gpd
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.cm as cm
from matplotlib.colors import Normalize
import pickle
import numpy as np


# State name to abbreviation mapping
STATE_ABBREV = {
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

# Districts per state (2020 apportionment)
DISTRICTS_PER_STATE = {
    'alabama': 7, 'alaska': 1, 'arizona': 9, 'arkansas': 4, 'california': 52,
    'colorado': 8, 'connecticut': 5, 'delaware': 1, 'florida': 28, 'georgia': 14,
    'hawaii': 2, 'idaho': 2, 'illinois': 17, 'indiana': 9, 'iowa': 4,
    'kansas': 4, 'kentucky': 6, 'louisiana': 6, 'maine': 2, 'maryland': 8,
    'massachusetts': 9, 'michigan': 13, 'minnesota': 8, 'mississippi': 4, 'missouri': 8,
    'montana': 2, 'nebraska': 3, 'nevada': 4, 'new_hampshire': 2, 'new_jersey': 12,
    'new_mexico': 3, 'new_york': 26, 'north_carolina': 14, 'north_dakota': 1, 'ohio': 15,
    'oklahoma': 5, 'oregon': 6, 'pennsylvania': 17, 'rhode_island': 2, 'south_carolina': 7,
    'south_dakota': 1, 'tennessee': 9, 'texas': 38, 'utah': 4, 'vermont': 1,
    'virginia': 11, 'washington': 10, 'west_virginia': 2, 'wisconsin': 8, 'wyoming': 1
}


def create_compactness_map(tracts_gdf, metric_name, metric_col, output_file, dpi=150):
    """Create a compactness visualization map"""
    
    fig, ax = plt.subplots(1, 1, figsize=(12, 10))

    # Color by compactness score (red = low, green = high)
    # Use realistic range (0.05-0.45) instead of theoretical (0-1)
    # to show meaningful variation among real-world districts
    cmap = cm.get_cmap('RdYlGn')
    norm = Normalize(vmin=0.05, vmax=0.45)
    
    tracts_gdf.plot(
        ax=ax,
        column=metric_col,
        cmap=cmap,
        norm=norm,
        edgecolor='white',
        linewidth=0.1,
        alpha=0.9,
        missing_kwds={'color': 'lightgray'}
    )
    
    # Add district boundaries
    districts_gdf = tracts_gdf.dissolve(by='district')
    districts_gdf.boundary.plot(ax=ax, linewidth=1.0, edgecolor='black', alpha=0.7)
    
    ax.set_axis_off()
    
    # Add title
    state_name = tracts_gdf['state'].iloc[0].replace('_', ' ').title()
    num_districts = len(tracts_gdf['district'].unique())
    avg_score = tracts_gdf.drop_duplicates('district')[metric_col].mean()

    plt.title(f'{state_name} - {metric_name} Compactness\n'
              f'{num_districts} Districts | Average: {avg_score:.3f}',
              fontsize=16, fontweight='bold', pad=20)
    
    # Add colorbar
    sm = cm.ScalarMappable(cmap=cmap, norm=norm)
    sm.set_array([])
    cbar = plt.colorbar(sm, ax=ax, orientation='horizontal', pad=0.05, aspect=50, shrink=0.8)
    cbar.set_label(f'{metric_name} Score (0.05 = gerrymandered, 0.45 = highly compact)', fontsize=12)
    
    plt.tight_layout()
    output_file.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()


def visualize_state_compactness(state_dir, state_code, census_year, dpi=150):
    """Create compactness visualizations for a single state."""
    state_dir = Path(state_dir)

    # Map state code to state name
    state_code_to_name = {v.upper(): k for k, v in STATE_ABBREV.items()}
    state_name = state_code_to_name.get(state_code.upper())

    if not state_name:
        print(f"ERROR: Unknown state code: {state_code}")
        return 1

    # Load district_summary.csv for compactness data (from data/ subdirectory)
    summary_file = state_dir / 'data' / 'district_summary.csv'
    if not summary_file.exists():
        print(f"ERROR: {summary_file} not found")
        return 1

    summary_df = pd.read_csv(summary_file)

    # Check if compactness columns exist
    if 'polsby_popper' not in summary_df.columns:
        print(f"ERROR: polsby_popper column not found in {summary_file}")
        return 1

    # Load tract geometries
    state_abbrev_map = {
        'california': 'ca', 'texas': 'tx', 'florida': 'fl', 'new_york': 'ny',
        'pennsylvania': 'pa', 'illinois': 'il', 'ohio': 'oh', 'georgia': 'ga',
        'north_carolina': 'nc', 'michigan': 'mi', 'new_jersey': 'nj', 'virginia': 'va',
        'washington': 'wa', 'arizona': 'az', 'massachusetts': 'ma', 'tennessee': 'tn',
        'indiana': 'in', 'maryland': 'md', 'missouri': 'mo', 'wisconsin': 'wi',
        'colorado': 'co', 'minnesota': 'mn', 'south_carolina': 'sc', 'alabama': 'al',
        'louisiana': 'la', 'kentucky': 'ky', 'oregon': 'or', 'oklahoma': 'ok',
        'connecticut': 'ct', 'utah': 'ut', 'iowa': 'ia', 'nevada': 'nv',
        'arkansas': 'ar', 'mississippi': 'ms', 'kansas': 'ks', 'new_mexico': 'nm',
        'nebraska': 'ne', 'idaho': 'id', 'west_virginia': 'wv', 'hawaii': 'hi',
        'new_hampshire': 'nh', 'maine': 'me', 'rhode_island': 'ri', 'montana': 'mt',
        'delaware': 'de', 'south_dakota': 'sd', 'north_dakota': 'nd', 'alaska': 'ak',
        'vermont': 'vt', 'wyoming': 'wy'
    }

    state_code = state_abbrev_map.get(state_name, state_name[:2]).lower()

    # Load tracts (unified directory structure)
    tracts_file = Path(f'data/tracts/{census_year}/{state_code}_tracts_{census_year}.parquet')

    if not tracts_file.exists():
        print(f"ERROR: {tracts_file} not found")
        return 1

    tracts = gpd.read_parquet(tracts_file)

    # Load assignments (from data/ subdirectory)
    assignments_file = state_dir / 'data' / 'final_assignments.pkl'
    if not assignments_file.exists():
        print(f"ERROR: {assignments_file} not found")
        return 1

    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    tracts['district'] = tracts.index.map(assignments)
    tracts['state'] = state_name

    # Merge compactness data
    tracts = tracts.merge(
        summary_df[['district', 'polsby_popper', 'reock']],
        on='district',
        how='left'
    )

    # Create output directory
    output_dir = state_dir / 'compactness' / 'maps'
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Creating compactness visualizations for {state_name.title()}...")

    # Create Polsby-Popper map (no year suffix)
    pp_output = output_dir / 'polsby_popper.png'
    create_compactness_map(tracts, 'Polsby-Popper', 'polsby_popper', pp_output, dpi)
    print(f"  Saved: {pp_output}")

    # Create Reock map (no year suffix)
    reock_output = output_dir / 'reock.png'
    create_compactness_map(tracts, 'Reock', 'reock', reock_output, dpi)
    print(f"  Saved: {reock_output}")

    return 0


def visualize_national_compactness(output_dir, version, census_year, dpi=150, position=-1, force=False):
    """Create national compactness visualization combining all states."""
    base_dir = Path(output_dir)

    # Progress bar protocol
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    if not base_dir.exists():
        if is_standalone:
            print(f"ERROR: Base directory not found: {base_dir}")
        return 1

    # Create maps/compactness directory if it doesn't exist
    maps_dir = base_dir / 'maps' / 'compactness'
    maps_dir.mkdir(parents=True, exist_ok=True)

    # Output file (no year suffix)
    output_file = maps_dir / 'polsby_popper.png'

    # Check if output exists
    if not force and output_file.exists():
        if is_standalone:
            print(f"Output already exists: {output_file}")
            print("Use --force to regenerate")
        return 0

    report_progress("Creating national compactness map - Loading data")

    if is_standalone:
        print(f"\nCreating national compactness map for {census_year} census...")
        print(f"Output: {output_file}")

    # Load all states with districts and compactness data
    try:
        all_tracts = []
        total_states = len(DISTRICTS_PER_STATE)
        processed = 0

        for state_name in DISTRICTS_PER_STATE.keys():
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

            # Load assignments (from data/ subdirectory)
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

            # Load compactness data if available (from data/ subdirectory)
            summary_file = state_dir / 'data' / 'district_summary.csv'
            if summary_file.exists():
                try:
                    summary_df = pd.read_csv(summary_file)
                    if 'polsby_popper' in summary_df.columns:
                        # Merge compactness data
                        tracts = tracts.merge(
                            summary_df[['district', 'polsby_popper', 'reock']],
                            on='district',
                            how='left'
                        )
                    else:
                        tracts['polsby_popper'] = np.nan
                        tracts['reock'] = np.nan
                except Exception as e:
                    print(f"  Warning: Skipping {state_name} - error loading compactness data: {e}")
                    continue
            else:
                tracts['polsby_popper'] = np.nan
                tracts['reock'] = np.nan

            all_tracts.append(tracts)
            processed += 1

            if processed % 10 == 0:
                if is_standalone:
                    print(f"  Loaded {processed}/{total_states} states...")

        if not all_tracts:
            if is_standalone:
                print("ERROR: No state data found")
            return 1

        us_tracts = gpd.GeoDataFrame(pd.concat(all_tracts, ignore_index=True))

        if is_standalone:
            print(f"  Loaded {processed} states with {len(us_tracts):,} tracts")

        report_progress("Creating national compactness map - Rendering")

        # Create single-panel figure
        fig = plt.figure(figsize=(28, 18))
        ax_main = fig.add_subplot(111)

        # Plot tracts colored by Polsby-Popper score
        cmap = cm.get_cmap('RdYlGn')  # Red (low) to Green (high)

        us_tracts.plot(
            ax=ax_main,
            column='polsby_popper',
            cmap=cmap,
            edgecolor='white',
            linewidth=0.05,
            alpha=0.9,
            vmin=0.05,
            vmax=0.45,
            missing_kwds={'color': 'lightgray'}
        )

        # Add district boundaries (dissolved tracts)
        region_districts = us_tracts.dissolve(by='unique_district_id')
        region_districts.boundary.plot(ax=ax_main, linewidth=1.5, edgecolor='black', alpha=0.8)

        # Add state boundaries
        region_states = us_tracts.dissolve(by='state_code')
        region_states.boundary.plot(ax=ax_main, linewidth=0.8, edgecolor='black', alpha=0.6)
        ax_main.set_axis_off()

        # Calculate statistics
        district_compactness = us_tracts.drop_duplicates('unique_district_id')['polsby_popper'].dropna()
        avg_compactness = district_compactness.mean()
        median_compactness = district_compactness.median()
        total_districts = len(us_tracts['unique_district_id'].unique())

        # Title
        fig.suptitle(f'United States Congressional Districts - Compactness\n'
                     f'Polsby-Popper Scores ({census_year} Census, {total_districts} Districts)',
                     fontsize=22, fontweight='bold', y=0.98)

        # Add colorbar with realistic range (0.05-0.45)
        from matplotlib.colors import Normalize
        norm = Normalize(vmin=0.05, vmax=0.45)
        sm = cm.ScalarMappable(cmap='RdYlGn', norm=norm)
        sm.set_array([])
        cbar = plt.colorbar(sm, ax=ax_main, orientation='horizontal',
                           pad=0.02, aspect=50, shrink=0.8)
        cbar.set_label('Polsby-Popper Score (0 = irregular, 1 = circular)', fontsize=12, fontweight='bold')

        # Add statistics box
        textstr = f'Average: {avg_compactness:.3f}\n'
        textstr += f'Median: {median_compactness:.3f}\n'
        textstr += f'Total Districts: {total_districts}'
        props = dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black', linewidth=2)
        ax_main.text(0.98, 0.98, textstr, transform=ax_main.transAxes, fontsize=14,
                    verticalalignment='top', horizontalalignment='right', bbox=props, fontweight='bold')

        plt.tight_layout(rect=[0, 0.05, 1, 0.97])

        # Save
        output_file.parent.mkdir(parents=True, exist_ok=True)
        plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
        plt.close(fig)

        if is_standalone:
            print(f"\nSaved: {output_file}")

        report_progress("Creating national compactness map - Complete")
        return 0

    except Exception as e:
        if is_standalone:
            print(f"\nERROR: {e}")
            import traceback
            traceback.print_exc()
        return 1


def main():
    parser = argparse.ArgumentParser(
        description='Visualize district compactness at state or national scope'
    )

    # Scope-based design - defaults to national (backward compatible with wrapper scripts)
    parser.add_argument('--scope', choices=['state', 'national'], default='national',
                       help='Scope: state (single state) or national (all states, default)')
    parser.add_argument('--census-year', type=str, required=True,
                       choices=['2000', '2010', '2020'],
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
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    parser.add_argument('--position', type=int, default=-1,
                       help='Progress bar position (for parent coordination)')

    args = parser.parse_args()

    # Get position from args or environment
    position = args.position if args.position >= 0 else int(os.environ.get('TQDM_POSITION', '-1'))

    # Validate scope-specific requirements
    if args.scope == 'state':
        if not args.state or not args.state_dir:
            parser.error("--state and --state-dir required when scope=state")
        return visualize_state_compactness(args.state_dir, args.state, args.census_year, args.dpi)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")
        return visualize_national_compactness(args.output_dir, args.version, args.census_year,
                                            args.dpi, position, args.force)


if __name__ == '__main__':
    sys.exit(main())
