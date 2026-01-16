#!/usr/bin/env python3
"""
Create national demographic map showing all 435 congressional districts colored by majority group.

This script combines data from all 50 states to create a single map showing
the majority demographic group in each district.
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


# Demographic color mapping
DEMOGRAPHIC_COLORS = {
    'White': '#4A90E2',       # Blue
    'Hispanic': '#E27A3F',    # Orange
    'Black': '#50C878',       # Green
    'Asian': '#9B59B6',       # Purple
    'Other': '#F39C12'        # Yellow
}

DEMOGRAPHIC_ORDER = ['White', 'Hispanic', 'Black', 'Asian', 'Other']

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
        description='Create national demographic map'
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

    # Create maps/demographic directory if it doesn't exist
    maps_dir = base_dir / 'maps' / 'demographic'
    maps_dir.mkdir(parents=True, exist_ok=True)

    # Output file (no year suffix)
    output_file = maps_dir / 'majority_demographics.png'

    # Print-only mode
    if args.print_only:
        if is_standalone:
            print("=" * 70)
            print(f"[PRINT-ONLY] National Demographic Map - {args.year} Census")
            print("=" * 70)
            print("\nWOULD EXECUTE:")
            print("\n  STEP 1: Load all state data")
            print(f"    - Load tracts: data/raw/{{state}}_tracts_{args.year}.parquet")
            print(f"    - Load assignments: {base_dir}/states/{{state}}/data/final_assignments.pkl")
            print(f"    - Load demographic data: {base_dir}/states/{{state}}/demographic/district_demographics.csv")
            print("    - Create district geometries by dissolving tracts")
            print("")
            print("  STEP 2: Separate contiguous US from Alaska/Hawaii")
            print("    - 48 contiguous states -> main map")
            print("    - Alaska -> inset (lower-left)")
            print("    - Hawaii -> inset (lower-left)")
            print("")
            print("  STEP 3: Create map with demographic colors")
            print("    - Color by majority group (5 categories)")
            print("    - Add state boundaries")
            print("    - Add legend with district counts")
            print("    - Add summary statistics")
            print("")
            print("  STEP 4: Save map")
            print(f"    - Output: {output_file}")
            print("\n" + "-" * 70)
            print("ESTIMATED OUTPUT:")
            print("-" * 70)
            print("  Total districts: 435")
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
    report_progress("Creating national demographic map - Loading data")

    if is_standalone:
        print(f"\nCreating national demographic map for {args.year} census...")
        print(f"Output: {output_file}")

    # Load all states with districts and demographic data
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

            # Load demographic data if available
            demographic_file = state_dir / 'demographic' / 'district_demographics.csv'
            if demographic_file.exists():
                try:
                    demographic_df = pd.read_csv(demographic_file)
                    # Rename majority_race to majority_group for consistency with color mapping
                    demographic_df = demographic_df.rename(columns={'majority_race': 'majority_group'})
                    # Merge demographic data
                    tracts = tracts.merge(
                        demographic_df[['district', 'majority_group', 'white_pct', 'hispanic_pct', 'black_pct', 'asian_pct']],
                        on='district',
                        how='left'
                    )
                except Exception as e:
                    # Skip states with problematic demographic data
                    if is_standalone:
                        print(f"  Warning: Skipping {state_name} - error loading demographic data: {e}")
                    continue
            else:
                tracts['majority_group'] = 'White'
                tracts['white_pct'] = 0.0
                tracts['hispanic_pct'] = 0.0
                tracts['black_pct'] = 0.0
                tracts['asian_pct'] = 0.0

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

        # Separate continental US, Alaska, and Hawaii
        continental = us_tracts[~us_tracts['state_code'].isin(['AK', 'HI'])].copy()
        alaska = us_tracts[us_tracts['state_code'] == 'AK'].copy()
        hawaii = us_tracts[us_tracts['state_code'] == 'HI'].copy()

        # Create figure with 3 panels
        fig = plt.figure(figsize=(28, 18))
        ax_main = plt.subplot2grid((4, 4), (0, 0), colspan=4, rowspan=3)
        ax_alaska = plt.subplot2grid((4, 4), (3, 0), colspan=1, rowspan=1)
        ax_hawaii = plt.subplot2grid((4, 4), (3, 1), colspan=1, rowspan=1)

        # Plot function
        def plot_demographic_region(tracts_data, ax, region_name):
            if len(tracts_data) == 0:
                return

            # Plot each demographic group
            from collections import OrderedDict
            DEMOGRAPHIC_ORDER = ['White', 'Hispanic', 'Black', 'Asian', 'Other']

            for group in DEMOGRAPHIC_ORDER:
                if group in tracts_data['majority_group'].values:
                    data = tracts_data[tracts_data['majority_group'] == group]
                    data.plot(
                        ax=ax,
                        color=DEMOGRAPHIC_COLORS[group],
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

        # Plot all regions
        plot_demographic_region(continental, ax_main, "Continental US")
        plot_demographic_region(alaska, ax_alaska, "Alaska")
        plot_demographic_region(hawaii, ax_hawaii, "Hawaii")

        ax_alaska.set_title('Alaska', fontsize=10, fontweight='bold')
        ax_hawaii.set_title('Hawaii', fontsize=10, fontweight='bold')

        # Count districts by majority group
        district_counts = us_tracts.drop_duplicates('unique_district_id')['majority_group'].value_counts()

        # Title
        fig.suptitle(f'United States Congressional Districts - Demographic Composition\nMajority Group by District ({args.year} Census)',
                     fontsize=22, fontweight='bold', y=0.98)

        # Legend with counts
        from matplotlib.patches import Patch
        legend_elements = [
            Patch(facecolor=DEMOGRAPHIC_COLORS[group], edgecolor='black',
                  label=f'{group}: {district_counts.get(group, 0)} districts')
            for group in DEMOGRAPHIC_ORDER
            if group in us_tracts['majority_group'].values
        ]
        ax_main.legend(handles=legend_elements, loc='lower right', fontsize=10,
                      title='Majority Demographic Group', title_fontsize=11, framealpha=0.9)

        # Add summary statistics
        textstr = f'Total Districts: 435\n'
        textstr += f'White Majority: {district_counts.get("White", 0)}\n'
        textstr += f'Hispanic Majority: {district_counts.get("Hispanic", 0)}\n'
        textstr += f'Black Majority: {district_counts.get("Black", 0)}\n'
        textstr += f'Asian Majority: {district_counts.get("Asian", 0)}\n'
        textstr += f'Other: {district_counts.get("Other", 0)}'
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

        report_progress("Creating national demographic map - Complete")
        return 0

    except Exception as e:
        if is_standalone:
            print(f"\nERROR: {e}")
            import traceback
            traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
