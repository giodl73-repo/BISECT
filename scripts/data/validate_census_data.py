#!/usr/bin/env python3
"""
Validate that all required census data files exist and have correct format.

This script checks that the census data processing pipeline has generated
all files needed by the redistricting pipeline.

Expected files for each state and year:
1. Tract parquet: outputs/data/{year}/units/{state}_tracts_{year}.parquet
   - Must be GeoDataFrame with geometry column
   - Required columns: GEOID, NAME, population, geometry, AREALAND, AREAWATR
2. Adjacency pickle: outputs/data/{year}/adjacency/{state}_adjacency_{year}.pkl
   - Adjacency graph for tract network
3. Places parquet (optional): outputs/data/{year}/places/{state}_places_{year}.parquet

National files (optional):
- Election data: outputs/data/{year}/elections/{year}_president_tract.parquet
- Demographics: outputs/data/{year}/demographics/{year}_demographics_tract.parquet
"""

import argparse
import sys
from pathlib import Path
from typing import List, Dict, Tuple
import pandas as pd
import geopandas as gpd

# All US states
ALL_STATES = [
    'AL', 'AK', 'AZ', 'AR', 'CA', 'CO', 'CT', 'DE', 'FL', 'GA',
    'HI', 'ID', 'IL', 'IN', 'IA', 'KS', 'KY', 'LA', 'ME', 'MD',
    'MA', 'MI', 'MN', 'MS', 'MO', 'MT', 'NE', 'NV', 'NH', 'NJ',
    'NM', 'NY', 'NC', 'ND', 'OH', 'OK', 'OR', 'PA', 'RI', 'SC',
    'SD', 'TN', 'TX', 'UT', 'VT', 'VA', 'WA', 'WV', 'WI', 'WY'
]


def validate_tract_file(file_path: Path, state: str, year: str) -> Tuple[bool, str]:
    """Validate a tract parquet file."""
    if not file_path.exists():
        return False, "File not found"

    try:
        gdf = gpd.read_parquet(file_path)
    except Exception as e:
        return False, f"Failed to read parquet: {str(e)}"

    # Check it's a GeoDataFrame
    if not isinstance(gdf, gpd.GeoDataFrame):
        return False, "Not a GeoDataFrame (missing geometry)"

    # Check required columns
    required_cols = ['GEOID', 'NAME', 'population', 'geometry']
    missing = [col for col in required_cols if col not in gdf.columns]
    if missing:
        return False, f"Missing required columns: {', '.join(missing)}"

    # Check geometry column
    if gdf.geometry.isna().all():
        return False, "All geometries are null"

    # Check CRS
    if gdf.crs is None:
        return False, "Missing CRS"

    # Check record count
    if len(gdf) == 0:
        return False, "Empty file (0 tracts)"

    return True, f"{len(gdf)} tracts, CRS: {gdf.crs}"


def validate_adjacency_file(file_path: Path, state: str, year: str) -> Tuple[bool, str]:
    """Validate an adjacency pickle file."""
    if not file_path.exists():
        return False, "File not found"

    try:
        import pickle
        with open(file_path, 'rb') as f:
            data = pickle.load(f)
    except Exception as e:
        return False, f"Failed to read pickle: {str(e)}"

    # Check structure
    if not isinstance(data, dict):
        return False, "Not a dictionary"

    required_keys = ['adjacency', 'vertex_weights', 'geoid_to_index', 'index_to_geoid']
    missing = [key for key in required_keys if key not in data]
    if missing:
        return False, f"Missing keys: {', '.join(missing)}"

    n_tracts = len(data['vertex_weights'])
    return True, f"{n_tracts} tracts in graph"


def main():
    parser = argparse.ArgumentParser(
        description='Validate census data completeness',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument('--year', type=int, required=True, choices=[2000, 2010, 2020],
                       help='Census year to validate')
    parser.add_argument('--states', type=str, nargs='*',
                       help='Specific states to check (default: all 50)')
    parser.add_argument('--verbose', action='store_true',
                       help='Show details for each state')

    args = parser.parse_args()

    states = [s.upper() for s in args.states] if args.states else ALL_STATES
    year = str(args.year)

    print(f"\n{'='*70}")
    print(f"VALIDATING CENSUS DATA FOR {year}")
    print(f"{'='*70}\n")

    results = {
        'tracts': {'valid': [], 'invalid': []},
        'adjacency': {'valid': [], 'invalid': []},
        'places': {'valid': [], 'invalid': []}
    }

    for state in states:
        state_lower = state.lower()

        if args.verbose:
            print(f"\n{state}:")

        # Check tract file
        tract_file = Path(f'outputs/data/{year}/units/{state_lower}_tracts_{year}.parquet')
        is_valid, message = validate_tract_file(tract_file, state, year)

        if is_valid:
            results['tracts']['valid'].append(state)
            if args.verbose:
                print(f"  [OK] Tracts: {message}")
        else:
            results['tracts']['invalid'].append((state, message))
            if args.verbose:
                print(f"  [FAIL] Tracts: {message}")

        # Check adjacency file
        adj_file = Path(f'outputs/data/{year}/adjacency/{state_lower}_adjacency_{year}.pkl')
        is_valid, message = validate_adjacency_file(adj_file, state, year)

        if is_valid:
            results['adjacency']['valid'].append(state)
            if args.verbose:
                print(f"  [OK] Adjacency: {message}")
        else:
            results['adjacency']['invalid'].append((state, message))
            if args.verbose:
                print(f"  [FAIL] Adjacency: {message}")

        # Check places file (optional)
        places_file = Path(f'outputs/data/{year}/places/{state_lower}_places_{year}.parquet')
        if places_file.exists():
            results['places']['valid'].append(state)
            if args.verbose:
                print(f"  [OK] Places: exists")

    # Summary
    print(f"\n{'='*70}")
    print(f"VALIDATION SUMMARY")
    print(f"{'='*70}\n")

    for data_type in ['tracts', 'adjacency', 'places']:
        valid_count = len(results[data_type]['valid'])
        invalid_count = len(results[data_type]['invalid'])
        total = valid_count + invalid_count

        status = "[OK]" if invalid_count == 0 else "[FAIL]"
        print(f"{status} {data_type.capitalize():12s}: {valid_count}/{len(states)} states valid")

        if invalid_count > 0 and data_type != 'places':
            print(f"     Failed states:")
            for state, reason in results[data_type]['invalid']:
                print(f"       {state}: {reason}")

    # Check national files
    print(f"\nNational files (optional):")

    election_file = Path(f'outputs/data/{year}/elections/{year}_president_tract.parquet')
    if election_file.exists():
        print(f"  [OK] Election data: {election_file.name}")
    else:
        print(f"  [SKIP] Election data not found")

    demo_file = Path(f'outputs/data/{year}/demographics/{year}_demographics_tract.parquet')
    if demo_file.exists():
        print(f"  [OK] Demographics: {demo_file.name}")
    else:
        print(f"  [SKIP] Demographics not found")

    print(f"\n{'='*70}\n")

    # Exit code
    tract_failures = len(results['tracts']['invalid'])
    adj_failures = len(results['adjacency']['invalid'])

    if tract_failures + adj_failures == 0:
        print("[SUCCESS] All required files validated successfully!")
        return 0
    else:
        print(f"[ERROR] {tract_failures + adj_failures} validation failures")
        print("\nNext steps:")
        if tract_failures > 0:
            print("  1. Run: python scripts/data/process_census_data.py --year {year} --stages tracts")
        if adj_failures > 0:
            print("  2. Run: python scripts/data/process_census_data.py --year {year} --stages adjacency")
        return 1


if __name__ == '__main__':
    sys.exit(main())
