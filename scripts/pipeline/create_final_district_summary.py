#!/usr/bin/env python3
"""
Create a comprehensive district summary CSV with all statistics and largest cities.
"""

import sys
import os
import time
import json
from pathlib import Path

# Import utility functions
sys.path.insert(0, str(Path(__file__).parent.parent.parent))
from scripts.utils import get_state_config, get_tract_file, get_places_file
import pickle
import pandas as pd
import geopandas as gpd
import numpy as np
import argparse
from tqdm import tqdm
from shapely.ops import unary_union
from shapely.geometry import Point


def polsby_popper_score(geometry):
    """
    Calculate Polsby-Popper compactness score.

    Formula: 4π × area / perimeter²
    Range: 0 to 1, where 1 is a perfect circle
    """
    area = geometry.area
    perimeter = geometry.length

    if perimeter == 0:
        return 0.0

    score = (4 * np.pi * area) / (perimeter ** 2)
    return min(1.0, score)  # Cap at 1.0 due to numerical precision


def minimum_bounding_circle(geometry):
    """
    Compute minimum bounding circle radius using centroid and max distance.
    """
    centroid = geometry.centroid

    # Get boundary points
    if geometry.geom_type == 'Polygon':
        boundary_coords = list(geometry.exterior.coords)
    elif geometry.geom_type == 'MultiPolygon':
        boundary_coords = []
        for poly in geometry.geoms:
            boundary_coords.extend(list(poly.exterior.coords))
    else:
        return 0.0

    # Find maximum distance from centroid to any boundary point
    max_dist = 0.0
    for coord in boundary_coords:
        point = Point(coord)
        dist = centroid.distance(point)
        max_dist = max(max_dist, dist)

    return max_dist


def reock_score(geometry):
    """
    Calculate Reock compactness score.

    Formula: area / area_of_minimum_bounding_circle
    Range: 0 to 1, where 1 is a perfect circle
    """
    radius = minimum_bounding_circle(geometry)

    if radius == 0:
        return 0.0

    circle_area = np.pi * (radius ** 2)
    score = geometry.area / circle_area

    return min(1.0, score)  # Cap at 1.0 due to numerical precision


def create_district_summary(
    tracts_file: str,
    places_file: str,
    assignments_file: str,
    cities_file: str,
    output_file: Path,
    total_pop: int,
    num_districts: int = 52,
    debug: bool = False,
    progress_bar=None
):
    """
    Create comprehensive district summary.
    """
    if debug:
        print("\nCreating district summary CSV...")

    # Load data
    if debug:
        print("  Loading tracts data...")
    tracts = gpd.read_parquet(tracts_file)

    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)

    tracts['district'] = [assignments[i] for i in range(len(tracts))]

    # Load city data (may not exist if places data unavailable)
    from pathlib import Path as PathObj
    cities_path = PathObj(cities_file)
    if cities_path.exists():
        cities_df = pd.read_csv(cities_file)
        has_cities = True
    else:
        # Create empty cities dataframe with required columns
        cities_df = pd.DataFrame({
            'district': range(1, num_districts + 1),
            'largest_city': ['N/A'] * num_districts,
            'city_population': [0] * num_districts
        })
        has_cities = False
        if debug:
            print("  No city data available (places data not found for this census year)")

    # Calculate statistics for each district
    if debug:
        print(f"  Calculating statistics and compactness for {num_districts} districts...")

    ideal_pop = total_pop / num_districts
    rows = []

    for district_id in range(1, num_districts + 1):
        district_tracts = tracts[tracts['district'] == district_id]

        pop = district_tracts['population'].sum()
        n_tracts = len(district_tracts)
        deviation = (pop - ideal_pop) / ideal_pop * 100

        # Get city info
        city_info = cities_df[cities_df['district'] == district_id].iloc[0]

        # Calculate compactness metrics
        # Create district geometry by dissolving all tracts in the district
        district_geometry = unary_union(district_tracts.geometry)

        pp_score = polsby_popper_score(district_geometry)
        r_score = reock_score(district_geometry)

        rows.append({
            'district': district_id,
            'population': int(pop),
            'num_tracts': n_tracts,
            'ideal_population': int(ideal_pop),
            'deviation_from_ideal': pop - ideal_pop,
            'deviation_percent': deviation,
            'largest_city': city_info['largest_city'],
            'city_population': int(city_info['city_population']) if city_info['city_population'] > 0 else 0,
            'polsby_popper': pp_score,
            'reock': r_score
        })

        # Emit STATUS for parent progress bar
        if progress_bar:
            progress_bar.update(1)

        # Additional status reporting for very large states
        if num_districts >= 20 and district_id % 10 == 0 and debug:
            print(f"  Processed {district_id}/{num_districts} districts (compactness: PP={pp_score:.3f}, Reock={r_score:.3f})")

    df = pd.DataFrame(rows)

    # Save
    df.to_csv(output_file, index=False, float_format='%.4f')

    # Print statistics
    if debug:
        print(f"\n  District Summary Statistics:")
        print(f"    Mean Polsby-Popper: {df['polsby_popper'].mean():.3f}")
        print(f"    Mean Reock: {df['reock'].mean():.3f}")
        print(f"    Mean Population Deviation: {abs(df['deviation_percent']).mean():.2f}%")
        print(f"  Saved to: {output_file}")

    return df


def create_rounds_hierarchy(run_dir: Path, num_districts: int, state_code: str, census_year: str, debug: bool = False):
    """
    Create rounds_hierarchy.csv from intermediate round metadata.

    For single-district states: Creates 6 rounds with the same district.
    For multi-district states: Extends to 6 rounds by repeating final round.
    """
    intermediate_dir = run_dir / 'intermediate'
    hierarchy_data = []

    # Handle single-district states (no intermediate directory)
    if not intermediate_dir.exists() or num_districts == 1:
        if debug:
            print(f"  Single-district state, creating 6-round hierarchy")

        # Load tract data to get population and count
        assignments_file = run_dir / 'data' / 'final_assignments.pkl'
        if not assignments_file.exists():
            if debug:
                print(f"  No assignments file found, skipping hierarchy creation")
            return None

        with open(assignments_file, 'rb') as f:
            assignments = pickle.load(f)

        # Load tract data
        tracts_file = get_tract_file(state_code, census_year)
        if not tracts_file.exists():
            if debug:
                print(f"  Tracts file not found: {tracts_file}")
            return None

        tracts = pd.read_parquet(tracts_file)
        total_pop = int(tracts['population'].sum())
        num_tracts = len(tracts)

        # Create 6 rounds with the same single district
        for round_num in range(1, 7):
            hierarchy_data.append({
                'round': round_num,
                'total_regions': 1,
                'region_id': 0,
                'region_name': f'{state_code}0',
                'population': total_pop,
                'num_tracts': num_tracts,
                'target_districts': 1,
                'population_per_district': total_pop,
                'deviation_from_ideal_pct': 0.0
            })
    else:
        # Multi-district state: load from intermediate files
        round_files = sorted(intermediate_dir.glob('round_*_metadata.json'))

        if not round_files:
            if debug:
                print(f"  No round files found, skipping hierarchy creation")
            return None

        for metadata_file in round_files:
            with open(metadata_file, 'r') as f:
                metadata = json.load(f)

            round_num = metadata['depth']
            num_regions = metadata['num_regions']
            total_pop = metadata.get('total_population', 0)
            ideal_pop = total_pop / num_districts if num_districts > 0 else 0

            # Extract region information
            regions = metadata.get('regions', [])

            for region in regions:
                region_id = region.get('region_id', 0)
                region_name = region.get('name', '')
                population = region.get('population', 0)
                num_blocks = region.get('num_blocks', 0)
                target_districts = region.get('target_districts', 0)

                # Calculate deviation
                if target_districts > 0:
                    per_district_pop = population / target_districts
                    deviation_pct = ((per_district_pop - ideal_pop) / ideal_pop * 100) if ideal_pop > 0 else 0
                else:
                    per_district_pop = population
                    deviation_pct = 0

                hierarchy_data.append({
                    'round': round_num,
                    'total_regions': num_regions,
                    'region_id': region_id,
                    'region_name': region_name,
                    'population': population,
                    'num_tracts': num_blocks,
                    'target_districts': target_districts,
                    'population_per_district': int(per_district_pop),
                    'deviation_from_ideal_pct': round(deviation_pct, 3)
                })

    if not hierarchy_data:
        return None

    df = pd.DataFrame(hierarchy_data)

    # Extend to 6 rounds if needed (for multi-district states that end early)
    max_round = df['round'].max()
    if max_round < 6:
        if debug:
            print(f"  Extending from round {max_round} to round 6")

        # Get the last round's data
        last_round_data = df[df['round'] == max_round].copy()

        # Replicate for rounds max_round+1 to 6
        for round_num in range(max_round + 1, 7):
            extended_round = last_round_data.copy()
            extended_round['round'] = round_num
            df = pd.concat([df, extended_round], ignore_index=True)

    # Sort by round and region_id for consistency
    df = df.sort_values(['round', 'region_id']).reset_index(drop=True)

    data_dir = run_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)
    output_file = data_dir / 'rounds_hierarchy.csv'
    df.to_csv(output_file, index=False)

    return df


if __name__ == '__main__':
    import sys

    parser = argparse.ArgumentParser(description='Create final district summary CSV')
    parser.add_argument('run_dir', type=str, help='Run directory (e.g., outputs/us_2020_v1/states/california)')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, NY)')
    parser.add_argument('--year', type=str, default='2020', choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')
    parser.add_argument('--print-only', action='store_true',
                       help='Print what would be done without executing')
    parser.add_argument('--debug', action='store_true',
                       help='Enable debug mode with progress delays and file display')
    parser.add_argument('--dpi', type=int, default=150,
                       help='DPI for output maps (not used by this script, but accepted for consistency)')
    parser.add_argument('--position', type=int, default=2, help='Progress bar position (for parallel mode)')
    args = parser.parse_args()

    run_dir = Path(args.run_dir)
    if args.debug: print(f"[DEBUG] run_dir = {run_dir}", file=sys.stderr, flush=True)

    # Get state info from arguments
    state_code = args.state.upper()
    STATE_CONFIG = get_state_config(args.year)
    config = STATE_CONFIG.get(state_code)
    if not config:
        print(f"ERROR: Unknown state code {state_code}")
        sys.exit(1)
    state_name = config['name']

    if args.debug: print(f"[DEBUG] State: {state_name} ({state_code})", file=sys.stderr, flush=True)

    # Load tract and places files (unified directory structure)
    tracts_file = str(get_tract_file(state_code, args.year))
    places_file = str(get_places_file(state_code, args.year))

    # References to data/ subdirectory
    data_dir = run_dir / 'data'
    assignments_file = data_dir / 'final_assignments.pkl'
    cities_file = data_dir / 'district_cities.csv'
    output_file = data_dir / 'district_summary.csv'

    # Get number of districts (needed for progress bar and skip logic)
    try:
        STATE_CONFIG = get_state_config(args.year)
        config = STATE_CONFIG.get(state_code.upper(), {})
        num_districts = config.get('districts', 1)
    except:
        num_districts = 1

    if args.debug: print(f"[DEBUG] num_districts = {num_districts}", file=sys.stderr, flush=True)

    # Show progress bars for integration with parent script (no file display - uses multiple files)
    operation_pos = args.position
    if args.debug: print(f"[DEBUG] operation_pos = {operation_pos}", file=sys.stderr, flush=True)

    stage_pbar = None
    if args.debug: print(f"[DEBUG] Checking if should create progress bar: print_only={args.print_only}, position={args.position}", file=sys.stderr, flush=True)
    if args.print_only or args.position != 2 and args.position != 999:
        if args.debug: print(f"[DEBUG] Creating progress bar at position {operation_pos}", file=sys.stderr, flush=True)
        # Create progress bar at operation_pos (number of districts to summarize)
        stage_pbar = tqdm(total=num_districts,
                         desc=f"{state_name} [{num_districts}D] Creating summary",
                         unit="district",
                         position=operation_pos,
                         leave=True,
                         ncols=120)
        if args.debug: print(f"[DEBUG] Progress bar created", file=sys.stderr, flush=True)
    else:
        if args.debug: print(f"[DEBUG] No progress bar created", file=sys.stderr, flush=True)

    # Check if output already exists
    if args.debug: print(f"[DEBUG] Checking if output exists: {output_file}", file=sys.stderr, flush=True)
    if not args.print_only and output_file.exists():
        skip_exists = True
        if args.debug: print(f"[DEBUG] Output file exists, skipping: {output_file}", file=sys.stderr, flush=True)
    else:
        skip_exists = False
        if args.debug: print(f"[DEBUG] Output file doesn't exist, will create: {output_file}", file=sys.stderr, flush=True)

    # Print-only mode or skip if output exists
    if args.print_only or skip_exists:
        if args.debug: print(f"[DEBUG] Entering skip path (print_only={args.print_only}, skip_exists={skip_exists})", file=sys.stderr, flush=True)
        # Update progress bar description to show skipping status
        if skip_exists and not args.print_only and stage_pbar:
            stage_pbar.set_description("    Creating summary (skipped - output exists)")

        if args.debug:
            if args.debug: print(f"[DEBUG] Debug mode: simulating progress for {num_districts} districts", file=sys.stderr, flush=True)
            # Debug mode: show progress with delays
            if stage_pbar:
                for i in range(num_districts):
                    time.sleep(0.02)  # Simulate statistics calculation (faster than maps)
                    stage_pbar.update(1)
            else:
                time.sleep(0.5)  # Just wait without progress
            if args.debug: print(f"[DEBUG] Simulation complete", file=sys.stderr, flush=True)
        else:
            # Non-debug mode: instant completion
            if stage_pbar:
                stage_pbar.update(num_districts)

        # Close progress bar
        if stage_pbar:
            stage_pbar.close()
            del stage_pbar

        # Create rounds hierarchy even when skipping main processing
        # (it has its own existence check and might need to be generated)
        if args.debug:
            print("\nCreating rounds hierarchy (skip path)...")
        create_rounds_hierarchy(run_dir, num_districts, state_code, args.year, debug=args.debug)

        if args.debug: print(f"[DEBUG] Exiting (skip path)", file=sys.stderr, flush=True)
        sys.stdout.flush()
        sys.stderr.flush()
        import os
        os._exit(0)  # Force exit without cleanup

    if args.debug: print(f"[DEBUG] Processing mode - loading tracts from {tracts_file}", file=sys.stderr, flush=True)
    # Load tracts to get total population
    import geopandas as gpd
    tracts = gpd.read_parquet(tracts_file)
    total_pop = int(tracts['population'].sum())

    # Auto-detect number of districts from assignments
    with open(assignments_file, 'rb') as f:
        assignments = pickle.load(f)
    num_districts = max(assignments.values())


    create_district_summary(
        tracts_file,
        places_file,
        str(assignments_file),
        str(cities_file),
        output_file,
        total_pop=total_pop,
        num_districts=num_districts,
        debug=args.debug,
        progress_bar=stage_pbar
    )

    # Create rounds hierarchy (all states, including single-district)
    if args.debug:
        print("\nCreating rounds hierarchy...")
    create_rounds_hierarchy(run_dir, num_districts, debug=args.debug)

    # Close progress bar if it was created
    if stage_pbar:
        stage_pbar.close()
        del stage_pbar

    if args.debug:
        print("\n" + "=" * 70)
        print("=" * 70)

    # Force clean exit
    sys.stdout.flush()
    sys.stderr.flush()
    import os
    os._exit(0)
