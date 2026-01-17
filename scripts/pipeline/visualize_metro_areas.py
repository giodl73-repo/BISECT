#!/usr/bin/env python3
"""
Create focused maps for major metro areas showing congressional districts.

This script generates detailed maps for the top 20 MSAs (Metropolitan Statistical Areas)
showing how congressional districts are configured within major urban areas.

Example usage:
    python scripts/pipeline/visualize_metro_areas.py --year 2020 --version v2
    python scripts/pipeline/visualize_metro_areas.py --year 2020 --version v2 --metros "New York" "Los Angeles"
"""

import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.patheffects as path_effects
import pickle
from pathlib import Path
import argparse
import sys
import os
from tqdm import tqdm

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# Import configuration files
try:
    from scripts.config_2020 import STATE_CONFIG_2020
    STATE_CONFIG = STATE_CONFIG_2020
except ImportError:
    STATE_CONFIG = None

# Top 20 MSAs by population (2020 Census)
# Each metro pinned to its primary state for organization
TOP_METROS = {
    # State: [(metro_name, short_name), ...]
    'NY': [("New York-Newark-Jersey City, NY-NJ-PA", "new_york")],
    'CA': [
        ("Los Angeles-Long Beach-Anaheim, CA", "los_angeles"),
        ("San Francisco-Oakland-Berkeley, CA", "san_francisco"),
        ("Riverside-San Bernardino-Ontario, CA", "riverside"),
        ("San Diego-Chula Vista-Carlsbad, CA", "san_diego"),
    ],
    'IL': [("Chicago-Naperville-Elgin, IL-IN-WI", "chicago")],
    'TX': [
        ("Dallas-Fort Worth-Arlington, TX", "dallas"),
        ("Houston-The Woodlands-Sugar Land, TX", "houston"),
    ],
    'VA': [("Washington-Arlington-Alexandria, DC-VA-MD-WV", "washington")],
    'PA': [("Philadelphia-Camden-Wilmington, PA-NJ-DE-MD", "philadelphia")],
    'FL': [
        ("Miami-Fort Lauderdale-Pompano Beach, FL", "miami"),
        ("Tampa-St. Petersburg-Clearwater, FL", "tampa"),
    ],
    'GA': [("Atlanta-Sandy Springs-Alpharetta, GA", "atlanta")],
    'MA': [("Boston-Cambridge-Newton, MA-NH", "boston")],
    'AZ': [("Phoenix-Mesa-Chandler, AZ", "phoenix")],
    'MI': [("Detroit-Warren-Dearborn, MI", "detroit")],
    'WA': [("Seattle-Tacoma-Bellevue, WA", "seattle")],
    'MN': [("Minneapolis-St. Paul-Bloomington, MN-WI", "minneapolis")],
    'CO': [("Denver-Aurora-Lakewood, CO", "denver")],
    'MO': [("St. Louis, MO-IL", "st_louis")],
}


def load_single_state_with_districts(state_code, state_dir, year='2020'):
    """Load a single state with district assignments.

    Args:
        state_code: Two-letter state code (e.g., 'CA')
        state_dir: Path to state output directory
        year: Census year

    Returns:
        GeoDataFrame with tracts and district assignments
    """
    state_dir = Path(state_dir)
    tracts_file = f'data/tracts/{year}/{state_code.lower()}_tracts_{year}.parquet'
    assignments_file = state_dir / 'data' / 'final_assignments.pkl'

    if not Path(tracts_file).exists():
        raise FileNotFoundError(f"Tracts file not found: {tracts_file}")

    tracts = gpd.read_parquet(tracts_file)

    if assignments_file.exists():
        with open(assignments_file, 'rb') as f:
            assignments = pickle.load(f)
        tracts['district'] = [assignments[i] for i in range(len(tracts))]
    else:
        # Single-district state
        tracts['district'] = 1

    tracts['state_code'] = state_code
    tracts['state_name'] = STATE_CONFIG[state_code]['name']

    return tracts


def load_all_states_with_districts(us_dir, year='2020'):
    """Load all states with their district assignments."""
    us_dir = Path(us_dir)

    all_tracts = []

    # Get position for progress reporting
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    if send_status:
        report_progress("Loading state district data")

    # Load state config
    if STATE_CONFIG is None:
        raise ValueError("Could not load STATE_CONFIG. Import from scripts.config_2020 failed.")

    # Iterate through states
    for state_code, config in STATE_CONFIG.items():
        state_name = config['name']

        state_dir = us_dir / 'states' / state_name.lower().replace(' ', '_')
        tracts_file = f'data/tracts/{year}/{state_code.lower()}_tracts_{year}.parquet'
        assignments_file = state_dir / 'data' / 'final_assignments.pkl'

        if not Path(tracts_file).exists():
            continue

        tracts = gpd.read_parquet(tracts_file)

        if assignments_file.exists():
            with open(assignments_file, 'rb') as f:
                assignments = pickle.load(f)
            tracts['district'] = [assignments[i] for i in range(len(tracts))]
        else:
            # Single-district state
            tracts['district'] = 1

        tracts['state_code'] = state_code
        tracts['state_name'] = state_name

        all_tracts.append(tracts)

    us_tracts = pd.concat(all_tracts, ignore_index=True)

    # Create unique district IDs across all states
    us_tracts['unique_district_id'] = (
        us_tracts.groupby(['state_code', 'district']).ngroup() + 1
    )

    return us_tracts


def create_metro_map(metro_name, metro_geometry, us_tracts, places_gdf, output_file, dpi=150):
    """
    Create a focused map for a single metro area.

    Args:
        metro_name: Name of the MSA
        metro_geometry: Shapely geometry of the MSA boundary
        us_tracts: GeoDataFrame with all tracts and district assignments
        places_gdf: GeoDataFrame with city/place boundaries
        output_file: Path to save the map
        dpi: Resolution for output image
    """

    # Find tracts within or intersecting the metro boundary
    intersecting_tracts = us_tracts[us_tracts.intersects(metro_geometry)].copy()

    if len(intersecting_tracts) == 0:
        print(f"  WARNING: No tracts found in {metro_name}")
        return False

    # Filter to only show districts that are substantially within the metro area
    # Calculate what percentage of each district's population is in the metro
    district_pop_in_metro = intersecting_tracts.groupby(['state_code', 'district'])['population'].sum()

    # Get total population for each district
    district_total_pop = us_tracts.groupby(['state_code', 'district'])['population'].sum()

    # Calculate percentage in metro by population
    district_pct_pop_in_metro = (district_pop_in_metro / district_total_pop * 100).fillna(0)

    # Calculate what percentage of each district's land area is in the metro
    # Project to equal-area CRS for accurate area calculation
    intersecting_tracts_proj = intersecting_tracts.to_crs('EPSG:5070')
    us_tracts_proj = us_tracts.to_crs('EPSG:5070')

    district_area_in_metro = intersecting_tracts_proj.groupby(['state_code', 'district']).apply(
        lambda x: x.geometry.area.sum(), include_groups=False
    )

    district_total_area = us_tracts_proj.groupby(['state_code', 'district']).apply(
        lambda x: x.geometry.area.sum(), include_groups=False
    )

    # Calculate percentage in metro by area
    district_pct_area_in_metro = (district_area_in_metro / district_total_area * 100).fillna(0)

    # Only include districts with >50% of BOTH population AND area in the metro
    # (This ensures we only show core districts, not ones that are partially cut off)
    threshold = 50.0
    districts_to_include = district_pct_pop_in_metro[
        (district_pct_pop_in_metro > threshold) &
        (district_pct_area_in_metro > threshold)
    ].index

    # Get ALL tracts from qualifying districts (not just the ones in metro boundary)
    # This ensures we show complete districts, not cropped ones
    metro_tracts = us_tracts[
        us_tracts.apply(
            lambda row: (row['state_code'], row['district']) in districts_to_include,
            axis=1
        )
    ].copy()

    # Get unique districts in this metro
    metro_districts = metro_tracts.groupby(['state_code', 'district']).first().reset_index()

    print(f"  {metro_name}: {len(metro_districts)} districts, {len(metro_tracts)} tracts (>{threshold}% pop AND area)")

    # Dissolve tracts into districts
    districts_gdf = metro_tracts.dissolve(by=['state_code', 'district'], as_index=False)
    districts_gdf['district_label'] = districts_gdf.apply(
        lambda row: f"{row['state_code']}-{row['district']}", axis=1
    )

    # Project to appropriate CRS for plotting (Albers Equal Area)
    districts_proj = districts_gdf.to_crs('EPSG:5070')

    # Get extent with 10% margin
    minx, miny, maxx, maxy = districts_proj.total_bounds
    margin = 0.10
    width = maxx - minx
    height = maxy - miny
    extent = [
        minx - margin * width,
        maxx + margin * width,
        miny - margin * height,
        maxy + margin * height
    ]

    # Create figure
    fig, ax = plt.subplots(figsize=(14, 10), dpi=dpi)

    # Create color mapping for districts
    n_districts = len(districts_gdf)
    cmap = plt.colormaps.get_cmap('tab20').resampled(n_districts)

    # Create district ID to color mapping
    district_to_color = {}
    for idx, (_, district) in enumerate(districts_gdf.iterrows()):
        district_key = (district['state_code'], district['district'])
        district_to_color[district_key] = cmap(idx)

    # Plot individual tracts with white boundaries to show detail
    metro_tracts_proj = metro_tracts.to_crs('EPSG:5070')
    for _, tract in metro_tracts_proj.iterrows():
        district_key = (tract['state_code'], tract['district'])
        tract_geom = gpd.GeoSeries([tract.geometry], crs='EPSG:5070')
        tract_geom.plot(
            ax=ax,
            color=district_to_color[district_key],
            edgecolor='white',
            linewidth=0.1,
            alpha=0.8
        )

    # Add thick black boundaries around districts
    districts_proj.boundary.plot(
        ax=ax,
        edgecolor='black',
        linewidth=1.2,
        zorder=10
    )

    # Add district labels at centroids
    for _, district in enumerate(districts_proj.iterrows()):
        _, district_data = district
        centroid = district_data.geometry.centroid
        if centroid.within(district_data.geometry):
            ax.text(
                centroid.x, centroid.y,
                district_data['district_label'],
                fontsize=10,
                fontweight='bold',
                ha='center',
                va='center',
                color='white',
                zorder=11
            ).set_path_effects([
                path_effects.Stroke(linewidth=2, foreground='black'),
                path_effects.Normal()
            ])

    # Add cities/places labels if available
    if places_gdf is not None and len(places_gdf) > 0:
        # Filter places within metro area
        metro_places = places_gdf[places_gdf.intersects(metro_geometry)].copy()

        if len(metro_places) > 0:
            # Show top 10 cities by population
            metro_places = metro_places.nlargest(10, 'population')
            metro_places_proj = metro_places.to_crs('EPSG:5070')

            for _, place in metro_places_proj.iterrows():
                centroid = place.geometry.centroid
                ax.text(
                    centroid.x, centroid.y,
                    place['NAME'],
                    fontsize=7,
                    ha='center',
                    va='top',
                    color='black',
                    fontweight='normal',
                    alpha=0.7,
                    zorder=5
                )

    # Set extent and styling
    ax.set_xlim(extent[0], extent[1])
    ax.set_ylim(extent[2], extent[3])
    ax.set_aspect('equal')
    ax.axis('off')

    # Title
    plt.title(
        f"{metro_name}\n"
        "Congressional Districts",
        fontsize=16,
        fontweight='bold',
        pad=20
    )

    # Save
    output_file.parent.mkdir(parents=True, exist_ok=True)
    plt.tight_layout()
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

    return True


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Create focused maps for major metro areas showing congressional districts'
    )
    parser.add_argument(
        '--scope',
        type=str,
        default='all',
        choices=['state', 'national', 'all'],
        help='Scope: state (per-state metros), national (overview map), all (batch mode - legacy)'
    )
    parser.add_argument(
        '--state',
        type=str,
        default=None,
        help='State code (required when scope=state, e.g., CA, NY, TX)'
    )
    parser.add_argument(
        '--state-dir',
        type=str,
        default=None,
        help='State output directory (e.g., outputs/us_2020_v1/states/california)'
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--version',
        type=str,
        default='v2',
        help='Output version (default: v2)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default=None,
        help='Output directory (default: outputs/us_{year}_{version})'
    )
    parser.add_argument(
        '--dpi',
        type=int,
        default=150,
        help='Image resolution (default: 150)'
    )
    parser.add_argument(
        '--metros',
        type=str,
        nargs='*',
        default=None,
        help='Specific metros to generate (default: all top 20)'
    )
    parser.add_argument(
        '--force',
        action='store_true',
        help='Force regeneration even if outputs exist'
    )

    args = parser.parse_args()

    # Validate scope parameters
    if args.scope == 'state':
        if not args.state:
            parser.error("--state required when scope=state")
        args.state = args.state.upper()
    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")

    # Set output directory
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'outputs/us_{args.year}_{args.version}')

    # Get position for progress reporting
    position = int(os.environ.get('TQDM_POSITION', '-1'))
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    #==========================================================================
    # SCOPE: NATIONAL - No aggregation needed, metros created per-state
    #==========================================================================
    if args.scope == 'national':
        report_progress("Metro maps complete (created per-state)")
        if is_standalone:
            print(f"\n{'='*80}")
            print(f"National Metro Maps")
            print(f"{'='*80}")
            print("Metro area maps are created during per-state processing.")
            print("No national aggregation map needed.")
            print(f"View individual metro maps in web dashboard.")
            print(f"{'='*80}\n")
        return 0

    #==========================================================================
    # SCOPE: STATE - Process metros for single state
    #==========================================================================
    if args.scope == 'state':
        # Census 2000 did not use CBSA classification - gracefully skip
        if args.year == 2000:
            report_progress(f"Metro maps not available for 2000 census (skipped)")
            if is_standalone:
                print(f"Metro area maps not available for Census 2000")
                print(f"CBSA (Core Based Statistical Area) classification was introduced after 2000.")
            return 0

        # Check if this state has metros
        if args.state not in TOP_METROS:
            report_progress(f"{args.state} - No major metros (skipped)")
            if is_standalone:
                print(f"{args.state} has no major metropolitan areas in TOP_METROS list - skipping")
            return 0

        metros_list = TOP_METROS[args.state]
        state_name = STATE_CONFIG[args.state]['name']

        if is_standalone:
            print(f"\n{'='*80}")
            print(f"Creating Metro Area Maps for {state_name}")
            print(f"{'='*80}")
            print(f"Metros: {len(metros_list)}")
            print(f"Year: {args.year}")
            print(f"DPI: {args.dpi}")
            print(f"{'='*80}\n")

        # Determine state directory
        if args.state_dir:
            state_dir = Path(args.state_dir)
        else:
            state_dir = output_dir / 'states' / state_name.lower().replace(' ', '_')

        # Load CBSA boundaries
        cbsa_file = f'data/raw/us_cbsa_{args.year}.parquet'
        if not Path(cbsa_file).exists():
            print(f"ERROR: CBSA boundaries not found at {cbsa_file}")
            print(f"Run: python scripts/data/geography/download_metro_boundaries.py --year {args.year}")
            return 1

        report_progress(f"{state_name} - Loading MSA boundaries")
        cbsa_gdf = gpd.read_parquet(cbsa_file)
        msa_gdf = cbsa_gdf[cbsa_gdf['LSAD'] == 'M1'].copy()

        # Load state tracts and districts
        report_progress(f"{state_name} - Loading tract data")
        state_tracts = load_single_state_with_districts(args.state, state_dir, year=str(args.year))

        # Load places data for this state
        places_file = f'data/places/{args.year}/{args.state.lower()}_places_{args.year}.parquet'
        if Path(places_file).exists():
            places_gdf = gpd.read_parquet(places_file)
            places_gdf['state_code'] = args.state
        else:
            places_gdf = None

        # Process metros for this state
        total_metros = len(metros_list)
        successful = 0
        failed = 0

        output_map_dir = state_dir / 'maps' / 'metros'

        for idx, (metro_name, short_name) in enumerate(metros_list, 1):
            output_file = output_map_dir / f"{short_name}.png"

            # Skip if already exists (unless --force)
            if output_file.exists() and not args.force:
                successful += 1
                if is_standalone:
                    print(f"  [{idx}/{total_metros}] SKIP: {short_name} (already exists)")
                continue

            report_progress(f"{state_name} - Creating metro map ({idx}/{total_metros}): {short_name}")

            # Find metro in MSA dataset
            metro_match = msa_gdf[msa_gdf['NAME'] == metro_name]
            if len(metro_match) == 0:
                # Try partial match on metro name (before comma)
                metro_short = metro_name.split(',')[0]
                metro_match = msa_gdf[msa_gdf['NAME'].str.startswith(metro_short)]

            if len(metro_match) == 0:
                # Try fuzzy match on primary city (metro names vary by year)
                # e.g., "Los Angeles-Long Beach-Anaheim" vs "Los Angeles-Long Beach-Santa Ana"
                primary_city = metro_name.split('-')[0]  # "Los Angeles"
                metro_match = msa_gdf[msa_gdf['NAME'].str.startswith(primary_city)]

                # If multiple matches, prefer one containing the state code
                if len(metro_match) > 1:
                    state_code = metro_name.split(',')[-1].strip()  # "CA"
                    metro_match = metro_match[metro_match['NAME'].str.contains(f', {state_code}')]

            if len(metro_match) == 0:
                print(f"  [{idx}/{total_metros}] ERROR: MSA not found: {metro_name}")
                failed += 1
                continue

            metro_row = metro_match.iloc[0]
            metro_geometry = metro_row.geometry

            try:
                success = create_metro_map(
                    metro_name=metro_name,
                    metro_geometry=metro_geometry,
                    us_tracts=state_tracts,
                    places_gdf=places_gdf,
                    output_file=output_file,
                    dpi=args.dpi
                )

                if success:
                    successful += 1
                    if is_standalone:
                        print(f"  [{idx}/{total_metros}] OK: {short_name}")
                else:
                    failed += 1

            except Exception as e:
                print(f"  [{idx}/{total_metros}] ERROR creating {short_name}: {e}")
                failed += 1

        # Summary for state scope
        if is_standalone:
            print(f"\n{'='*80}")
            print(f"Metro Maps Complete for {state_name}")
            print(f"{'='*80}")
            print(f"Total: {total_metros}, Successful: {successful}, Failed: {failed}")
            print(f"Output: {output_map_dir}")
            print(f"{'='*80}\n")

        report_progress(f"{state_name} - Metro maps complete ({successful}/{total_metros})")
        return 0 if failed == 0 else 1

    #==========================================================================
    # SCOPE: ALL (LEGACY BATCH MODE) - Process all metros
    #==========================================================================
    # Census 2000 did not use CBSA classification - gracefully skip
    if args.year == 2000:
        report_progress(f"Metro maps not available for 2000 census (skipped)")
        if is_standalone:
            print(f"\n{'='*80}")
            print(f"Metro Area Maps - Not Available for Census 2000")
            print(f"{'='*80}")
            print(f"CBSA (Core Based Statistical Area) classification was")
            print(f"introduced after 2000. Metro area maps are only available")
            print(f"for 2010 and 2020 census data.")
            print(f"{'='*80}\n")
        return 0

    # Banner
    if is_standalone:
        print(f"\n{'='*80}")
        print(f"Creating Metro Area District Maps (Batch Mode)")
        print(f"{'='*80}")
        print(f"Year: {args.year}")
        print(f"Version: {args.version}")
        print(f"Output: {output_dir}/states/*/maps/metros/*.png")
        print(f"DPI: {args.dpi}")
        print(f"{'='*80}\n")

    # Load CBSA boundaries
    cbsa_file = f'data/raw/us_cbsa_{args.year}.parquet'
    if not Path(cbsa_file).exists():
        print(f"ERROR: CBSA boundaries not found at {cbsa_file}")
        print(f"Run: python scripts/data/geography/download_metro_boundaries.py --year {args.year}")
        return 1

    report_progress("Loading MSA boundaries")
    cbsa_gdf = gpd.read_parquet(cbsa_file)

    # Filter to MSAs only (exclude Micropolitan)
    msa_gdf = cbsa_gdf[cbsa_gdf['LSAD'] == 'M1'].copy()
    print(f"Loaded {len(msa_gdf)} Metropolitan Statistical Areas")

    # Load all state districts
    report_progress("Loading state districts")
    us_tracts = load_all_states_with_districts(output_dir, year=str(args.year))
    print(f"Loaded {len(us_tracts)} tracts with district assignments")

    # Load places data for city labels
    places_all = []
    for state_code in STATE_CONFIG.keys():
        places_file = f'data/places/{args.year}/{state_code.lower()}_places_{args.year}.parquet'
        if Path(places_file).exists():
            places = gpd.read_parquet(places_file)
            places['state_code'] = state_code
            places_all.append(places)

    if places_all:
        places_gdf = pd.concat(places_all, ignore_index=True)
        print(f"Loaded {len(places_gdf)} cities/places")
    else:
        places_gdf = None
        print("WARNING: No places data found. Maps will not include city labels.")

    # Process metros by state
    total_metros = 0
    successful = 0
    failed = 0

    for state_code, metros_list in TOP_METROS.items():
        state_name = STATE_CONFIG[state_code]['name']
        state_dir = output_dir / 'states' / state_name.lower().replace(' ', '_') / 'maps' / 'metros'

        print(f"\nProcessing {state_code} ({state_name}): {len(metros_list)} metro(s)")

        for metro_name, short_name in metros_list:
            total_metros += 1

            # Create output filename
            output_file = state_dir / f"{short_name}.png"

            # Skip if already exists (unless --force)
            if output_file.exists() and not args.force:
                successful += 1
                print(f"  [SKIP] Already exists: {output_file}")
                continue

            report_progress(f"Creating metro map {total_metros}: {metro_name}")

            # Find metro in MSA dataset
            metro_match = msa_gdf[msa_gdf['NAME'] == metro_name]

            if len(metro_match) == 0:
                # Try partial match on metro name (before comma)
                metro_short = metro_name.split(',')[0]
                metro_match = msa_gdf[msa_gdf['NAME'].str.startswith(metro_short)]

            if len(metro_match) == 0:
                # Try fuzzy match on primary city (metro names vary by year)
                # e.g., "Los Angeles-Long Beach-Anaheim" vs "Los Angeles-Long Beach-Santa Ana"
                primary_city = metro_name.split('-')[0]  # "Los Angeles"
                metro_match = msa_gdf[msa_gdf['NAME'].str.startswith(primary_city)]

                # If multiple matches, prefer one containing the state code
                if len(metro_match) > 1:
                    state_code = metro_name.split(',')[-1].strip()  # "CA"
                    metro_match = metro_match[metro_match['NAME'].str.contains(f', {state_code}')]

            if len(metro_match) == 0:
                print(f"  [X] MSA not found in dataset: {metro_name}")
                failed += 1
                continue

            metro_row = metro_match.iloc[0]
            metro_geometry = metro_row.geometry

            try:
                success = create_metro_map(
                    metro_name=metro_name,
                    metro_geometry=metro_geometry,
                    us_tracts=us_tracts,
                    places_gdf=places_gdf,
                    output_file=output_file,
                    dpi=args.dpi
                )

                if success:
                    successful += 1
                    print(f"  [OK] Saved: {output_file}")
                else:
                    failed += 1

            except Exception as e:
                print(f"  [ERROR] creating map for {metro_name}: {e}")
                import traceback
                traceback.print_exc()
                failed += 1
                continue

    # Summary
    if is_standalone:
        print(f"\n{'='*80}")
        print(f"Metro Area Maps Complete")
        print(f"{'='*80}")
        print(f"Total Metros: {total_metros}")
        print(f"Successful: {successful}")
        print(f"Failed: {failed}")
        print(f"Output: {output_dir}/states/*/maps/metros/*.png")
        print(f"{'='*80}\n")

    return 0 if failed == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
