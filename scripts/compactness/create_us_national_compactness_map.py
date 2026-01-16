#!/usr/bin/env python3
"""
Create national compactness map showing all congressional districts colored by Polsby-Popper score.

This script combines data from all 50 states to create a single map showing
district compactness across the nation.
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


def main():
    parser = argparse.ArgumentParser(
        description='Create national compactness map'
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

    # Output file
    output_file = base_dir / f'us_national_compactness_{args.year}.png'

    # Print-only mode
    if args.print_only:
        if is_standalone:
            print("=" * 70)
            print(f"[PRINT-ONLY] National Compactness Map - {args.year} Census")
            print("=" * 70)
            print("
WOULD EXECUTE:")
            print("
  STEP 1: Load all state data")
            print(f"    - Load tracts: data/raw/{{state}}_tracts_{args.year}.parquet")
            print(f"    - Load assignments: {base_dir}/states/{{state}}/final_assignments.pkl")
            print(f"    - Load compactness data: {base_dir}/states/{{state}}/district_summary.csv")
            print("    - Create district geometries by dissolving tracts")
            print("")
            print("  STEP 2: Create map with compactness gradient")
            print("    - 50 states (all districts)")
            print("    - Color by Polsby-Popper score (0-1 scale)")
            print("    - Add district boundaries")
            print("    - Add state boundaries")
            print("    - Add legend and statistics")
            print("")
            print("  STEP 3: Save map")
            print(f"    - Output: {output_file}")
            print("
" + "-" * 70)
            print("ESTIMATED OUTPUT:")
            print("-" * 70)
            print("  Total districts: 435")
            print("  Map size: 28x18 inches")
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
    report_progress("Creating national compactness map - Loading data")

    if is_standalone:
        print(f"
Creating national compactness map for {args.year} census...")
        print(f"Output: {output_file}")

    # Load all states with districts and compactness data
    try:
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

            # Load assignments
            assignments_file = state_dir / 'final_assignments.pkl'
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

            # Load compactness data if available
            summary_file = state_dir / 'district_summary.csv'
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
                    if is_standalone:
                        print(f"  Warning: Skipping {state_name} - error loading compactness data: {e}")
                    continue
            else:
                tracts['polsby_popper'] = np.nan
                tracts['reock'] = np.nan

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

        # Create single-panel figure
        fig = plt.figure(figsize=(28, 18))
        ax_main = fig.add_subplot(111)

        # Plot tracts colored by Polsby-Popper score
        import matplotlib.cm as cm
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
        fig.suptitle(f'United States Congressional Districts - Compactness
Polsby-Popper Scores ({args.year} Census, {total_districts} Districts)',
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
        textstr = f'Average: {avg_compactness:.3f}
'
        textstr += f'Median: {median_compactness:.3f}
'
        textstr += f'Total Districts: {total_districts}'
        props = dict(boxstyle='round', facecolor='white', alpha=0.9, edgecolor='black', linewidth=2)
        ax_main.text(0.98, 0.98, textstr, transform=ax_main.transAxes, fontsize=14,
                    verticalalignment='top', horizontalalignment='right', bbox=props, fontweight='bold')

        plt.tight_layout(rect=[0, 0.05, 1, 0.97])

        # Save
        output_file.parent.mkdir(parents=True, exist_ok=True)
        plt.savefig(output_file, dpi=args.dpi, bbox_inches='tight')
        plt.close(fig)

        if is_standalone:
            print(f"
Saved: {output_file}")

        report_progress("Creating national compactness map - Complete")
        return 0

    except Exception as e:
        if is_standalone:
            print(f"
ERROR: {e}")
            import traceback
            traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
