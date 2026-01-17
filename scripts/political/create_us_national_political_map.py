#!/usr/bin/env python3
"""
Create national political map showing all 435 congressional districts colored by partisan lean.

This script combines data from all 50 states to create a single map showing
Democratic vs Republican lean across all districts.
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
import argparse
from pathlib import Path
import pickle
from shapely.ops import unary_union


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


def main():
    parser = argparse.ArgumentParser(
        description='Create national political map'
    )
    parser.add_argument('--year', type=str, required=True,
                       choices=['2010', '2020'],
                       help='Census year')
    parser.add_argument('--version', type=str, required=True,
                       help='Version (e.g., v1, v2)')
    parser.add_argument('--output-dir', type=str, required=False,
                       help='Base output directory (default: outputs/us_YEAR_VERSION)')
    parser.add_argument('--dpi', type=int, default=150,
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--position', type=int, default=-1,
                       help='Progress bar position (for parent coordination)')

    args = parser.parse_args()

    # Get position from args or environment
    position = args.position if args.position >= 0 else int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Determine base directory
    if args.output_dir:
        base_dir = Path(args.output_dir)
    else:
        base_dir = Path(f'outputs/us_{args.year}_{args.version}')

    if not base_dir.exists():
        if is_standalone:
            print(f"ERROR: Base directory not found: {base_dir}")
        return 1

    # Create maps/political directory if it doesn't exist
    maps_dir = base_dir / 'maps' / 'political'
    maps_dir.mkdir(parents=True, exist_ok=True)

    # Output file (no year suffix)
    output_file = maps_dir / 'partisan_lean.png'

    # Print-only mode
    if args.print_only:
        if is_standalone:
            print("=" * 70)
            print(f"[PRINT-ONLY] National Political Map - {args.year} Census")
            print("=" * 70)
            print("\nWOULD EXECUTE:")
            print("\n  STEP 1: Load all state data")
            print(f"    - Load tracts: data/raw/{{state}}_tracts_{args.year}.parquet")
            print(f"    - Load assignments: {base_dir}/states/{{state}}/data/final_assignments.pkl")
            print(f"    - Load political data: {base_dir}/states/{{state}}/political/district_political.csv")
            print("    - Create district geometries by dissolving tracts")
            print("")
            print("  STEP 2: Create map with partisan lean colors")
            print("    - 48 contiguous states (excludes AK/HI - no election data)")
            print("    - Color by D/R lean (7 categories)")
            print("    - Add district boundaries")
            print("    - Add state boundaries")
            print("    - Add legend")
            print("    - Add D/R seat totals")
            print("")
            print("  STEP 3: Save map")
            print(f"    - Output: {output_file}")
            print("\n" + "-" * 70)
            print("ESTIMATED OUTPUT:")
            print("-" * 70)
            print("  Total districts: 433 (48 states)")
            print("  Map size: 20x12 inches")
            print(f"  DPI: {args.dpi}")
            print("  Estimated time: 3-5 minutes")
            print("=" * 70)
        return 0

    # Check if output exists
    if not args.force and output_file.exists():
        if is_standalone:
            print(f"Output already exists: {output_file}")
            print("Use --force to regenerate")
        return 0

    # Main execution
    report_progress("Creating national political map - Loading data")

    if is_standalone:
        print(f"\nCreating national political map for {args.year} census...")
        print(f"Output: {output_file}")

    # Load all states with districts and political data
    try:
        import matplotlib
        matplotlib.use('Agg')
        import matplotlib.pyplot as plt
        import matplotlib.patheffects as path_effects
        import geopandas as gpd
        import pandas as pd
        import pickle
        from shapely.ops import unary_union

        all_tracts = []
        total_states = len(DISTRICTS_PER_STATE)
        processed = 0

        for state_name, num_districts in DISTRICTS_PER_STATE.items():
            # Skip Alaska and Hawaii (no 2020 presidential election data available)
            if state_name in ['alaska', 'hawaii']:
                continue

            state_dir = base_dir / 'states' / state_name

            # Skip if state not processed
            if not state_dir.exists():
                continue

            # Load tracts (use state abbreviation for filename)
            state_abbrev = STATE_ABBREV[state_name].lower()
            tracts_file = Path(f'data/tracts/{args.year}/{state_abbrev}_tracts_{args.year}.parquet')

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
            from collections import OrderedDict
            LEAN_ORDER = ['Strong D', 'Lean D', 'Tilt D', 'Tossup', 'Tilt R', 'Lean R', 'Strong R', 'No Data']

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
        plt.savefig(output_file, dpi=args.dpi, bbox_inches='tight')
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


if __name__ == '__main__':
    sys.exit(main())
