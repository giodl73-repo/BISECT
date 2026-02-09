#!/usr/bin/env python3
"""
Download worker for single-state data downloads.

Handles downloading census data for a single state across multiple data types (aligned with pipeline stages):
- redistricting: Census tract/block geometries and population (PL 94-171 + TIGER/Line)
- demographics: Demographic variables (Census API)
- elections: Election results (Harvard Dataverse)
- places: City/town boundaries (TIGER/Line)
- metros: Metro area boundaries (TIGER/Line CBSAs)
- enacted_districts: Enacted congressional districts (TIGER/Line)

Sends STATUS messages for hierarchical progress tracking.

Usage:
    python download_worker.py --state CA --type redistricting --year 2020 --output-dir outputs/data/2020/
    python download_worker.py --state TX --type demographics --year 2020 --output-dir outputs/data/2020/

Created: 2026-01-18 (Enhancement #48)
Updated: 2026-01-18 (Aligned download types with pipeline stages)
"""

import sys
import os
import argparse
from pathlib import Path

# Add project root to path
script_dir = Path(__file__).parent
project_root = script_dir.parent.parent
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / 'src'))

# Import configuration
from scripts.config.download_sources import (
    get_fips_2digit, validate_state_code, validate_year,
    get_tiger_tract_url, get_tiger_place_url, get_census_config,
    CENSUS_CONFIGS, TIGER_CBSA_URLS, TIGER_CD_URLS
)

# Import download utilities
from scripts.data.download_handler import (
    download_with_retry,
    download_and_extract_zip,
    download_with_fallback,
    check_existing_file
)


def send_status(msg, state_num=None, state_name=None, step=None, step_total=None, step_desc=None):
    """
    Send STATUS message to parent process.

    Args:
        msg: Simple message (used in standalone mode)
        state_num: State number (1-50)
        state_name: State name (lowercase_underscores)
        step: Current step (1-4)
        step_total: Total steps (typically 4)
        step_desc: Step description (e.g., 'downloading_csv')
    """
    # Check if running in multi-year subprocess mode
    is_multi_year = os.environ.get('MULTI_YEAR_SUBPROCESS') == '1'
    worker_id = int(os.environ.get('WORKER_ID', '0'))
    census_year = os.environ.get('CENSUS_YEAR', '2020')

    if is_multi_year and step is not None:
        # Hierarchical format: STATUS:WORKER:2020:1:STATE:12/50:california:STEP:3/4:extracting_zip
        print(f"STATUS:WORKER:{census_year}:{worker_id}:STATE:{state_num}/50:{state_name}:STEP:{step}/{step_total}:{step_desc}", flush=True)
    else:
        # Simple format (standalone or single-year mode)
        pos = int(os.environ.get('TQDM_POSITION', '-1'))
        if pos >= 0:
            print(f"STATUS:{pos}:{msg}", flush=True)


def download_tracts(state_code, year, output_dir, force=False):
    """
    Download census tract geometries (TIGER/Line shapefiles).

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year (2000, 2010, 2020)
        output_dir: Output directory for tracts
        force: Force redownload even if exists

    Returns:
        Tuple of (success: bool, message: str)
    """
    # Build output path
    tracts_dir = Path(output_dir) / f'tracts/{year}'
    tracts_dir.mkdir(parents=True, exist_ok=True)

    state_name = state_code.lower()
    shapefile_name = f'{state_name}_tracts_{year}.shp'
    shapefile_path = tracts_dir / shapefile_name

    # Check if already exists
    if check_existing_file(shapefile_path, force=force):
        return (True, f"[SKIP] Tracts already exist: {shapefile_name}")

    # Get download URL
    try:
        url = get_tiger_tract_url(state_code, year)
    except ValueError as e:
        return (False, f"[ERROR] {e}")

    # Download and extract ZIP
    success = download_and_extract_zip(
        url=url,
        extract_dir=tracts_dir,
        progress_callback=lambda msg: send_status(msg)
    )

    if success:
        return (True, f"[OK] Downloaded tracts: {shapefile_name}")
    else:
        return (False, f"[FAIL] Failed to download tracts")


def download_demographics(state_code, year, output_dir, force=False):
    """
    Download demographic data (Census API).

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year (2000, 2010, 2020)
        output_dir: Output directory for demographics
        force: Force redownload even if exists

    Returns:
        Tuple of (success: bool, message: str)
    """
    import pandas as pd
    import requests

    # Build output path
    demo_dir = Path(output_dir) / f'demographics/{year}'
    demo_dir.mkdir(parents=True, exist_ok=True)

    state_name = state_code.lower()
    csv_name = f'{state_name}_demographics_{year}.csv'
    csv_path = demo_dir / csv_name

    # Check if already exists
    if check_existing_file(csv_path, force=force):
        return (True, f"[SKIP] Demographics already exist: {csv_name}")

    # Get Census API configuration
    try:
        config = get_census_config(year)
        state_fips = get_fips_2digit(state_code)
    except ValueError as e:
        return (False, f"[ERROR] {e}")

    api_base = config['api_base']
    variables = config['variables']

    # Build API request
    variables_list = list(set(variables.values()))  # Deduplicate
    variables_str = ','.join(variables_list)
    url = f"{api_base}?get={variables_str}&for=tract:*&in=state:{state_fips}"

    # Download with retry
    try:
        send_status(f"Downloading demographics for {state_code}")

        response = requests.get(url, timeout=30)
        response.raise_for_status()

        data = response.json()
        headers = data[0]
        rows = data[1:]

        df = pd.DataFrame(rows, columns=headers)

        # Create GEOID
        df['GEOID'] = df['state'] + df['county'] + df['tract']

        # Convert numeric columns
        unique_vars = set(variables.values())
        for api_var in unique_vars:
            if api_var in df.columns:
                df[api_var] = pd.to_numeric(df[api_var], errors='coerce')

        # Create standard name columns
        for standard_name, api_var in variables.items():
            if api_var in df.columns:
                df[standard_name] = df[api_var]

        # Calculate "Other" category
        if all(col in df.columns for col in ['total_pop_race', 'white_non_hispanic', 'black_non_hispanic', 'asian_non_hispanic', 'hispanic']):
            df['other'] = (
                df['total_pop_race'] -
                df['white_non_hispanic'] -
                df['black_non_hispanic'] -
                df['asian_non_hispanic'] -
                df['hispanic']
            )

        # Select final columns
        final_cols = [
            'GEOID', 'state', 'county', 'tract',
            'total_pop', 'male', 'female',
            'white_non_hispanic', 'black_non_hispanic',
            'asian_non_hispanic', 'hispanic'
        ]
        if 'other' in df.columns:
            final_cols.append('other')

        df = df[[col for col in final_cols if col in df.columns]]

        # Save to CSV
        df.to_csv(csv_path, index=False)

        return (True, f"[OK] Downloaded demographics: {csv_name} ({len(df)} tracts)")

    except requests.exceptions.RequestException as e:
        return (False, f"[FAIL] Census API error: {e}")
    except Exception as e:
        return (False, f"[FAIL] Processing error: {e}")


def download_places(state_code, year, output_dir, force=False):
    """
    Download place (city/town) boundaries (TIGER/Line shapefiles).

    Args:
        state_code: 2-letter state code (e.g., 'CA')
        year: Census year (2000, 2010, 2020)
        output_dir: Output directory for places
        force: Force redownload even if exists

    Returns:
        Tuple of (success: bool, message: str)
    """
    # Build output path
    places_dir = Path(output_dir) / f'places/{year}'
    places_dir.mkdir(parents=True, exist_ok=True)

    state_name = state_code.lower()
    shapefile_name = f'{state_name}_places_{year}.shp'
    shapefile_path = places_dir / shapefile_name

    # Check if already exists
    if check_existing_file(shapefile_path, force=force):
        return (True, f"[SKIP] Places already exist: {shapefile_name}")

    # Get download URL
    try:
        url = get_tiger_place_url(state_code, year)
    except ValueError as e:
        return (False, f"[ERROR] {e}")

    # Download and extract ZIP
    success = download_and_extract_zip(
        url=url,
        extract_dir=places_dir,
        progress_callback=lambda msg: send_status(msg)
    )

    if success:
        return (True, f"[OK] Downloaded places: {shapefile_name}")
    else:
        return (False, f"[FAIL] Failed to download places")


def download_metros(state_code, year, output_dir, force=False):
    """
    Download metro/CBSA boundaries (national file).

    Note: This is a national file, not per-state. Worker should be called once only.

    Args:
        state_code: Ignored (metros are national)
        year: Census year (2000, 2010, 2020)
        output_dir: Output directory for metros
        force: Force redownload even if exists

    Returns:
        Tuple of (success: bool, message: str)
    """
    import geopandas as gpd

    # Build output path
    metros_dir = Path(output_dir) / f'metros/{year}'
    metros_dir.mkdir(parents=True, exist_ok=True)

    output_file = metros_dir / f'us_cbsa_{year}.parquet'

    # Check if already exists
    if check_existing_file(output_file, force=force):
        return (True, f"[SKIP] Metros already exist: us_cbsa_{year}.parquet")

    # Get download URL
    if year not in TIGER_CBSA_URLS:
        return (True, f"[SKIP] Metro boundaries not available for {year} via TIGER (requires NHGIS)")

    # Special case: 2000 uses MSA/PMSA system (not CBSA), requires NHGIS download
    if year == 2000:
        return (True, f"[SKIP] 2000 metros require NHGIS download (MSA/PMSA system, not CBSA)")

    url = TIGER_CBSA_URLS[year]

    try:
        # Download directly with geopandas
        send_status(f"Downloading metros from {url}")
        cbsa_gdf = gpd.read_file(url)

        # Standardize column names for different years
        if year == 2010:
            rename_map = {
                'GEOID10': 'GEOID',
                'NAME10': 'NAME',
                'NAMELSAD10': 'NAMELSAD',
                'LSAD10': 'LSAD'
            }
            cbsa_gdf = cbsa_gdf.rename(columns={k: v for k, v in rename_map.items() if k in cbsa_gdf.columns})
        elif year == 2000:
            rename_map = {
                'GEOID00': 'GEOID',
                'NAME00': 'NAME',
                'NAMELSAD00': 'NAMELSAD',
                'LSAD00': 'LSAD'
            }
            cbsa_gdf = cbsa_gdf.rename(columns={k: v for k, v in rename_map.items() if k in cbsa_gdf.columns})

        # Ensure EPSG:4326
        if cbsa_gdf.crs != 'EPSG:4326':
            cbsa_gdf = cbsa_gdf.to_crs('EPSG:4326')

        # Save to parquet
        cbsa_gdf.to_parquet(output_file)

        return (True, f"[OK] Downloaded metros: {len(cbsa_gdf)} CBSAs")

    except Exception as e:
        return (False, f"[FAIL] Failed to download metros: {e}")


def download_enacted_districts(state_code, year, output_dir, force=False):
    """
    Download enacted congressional districts (national file).

    Note: This is a national file, not per-state. Worker should be called once only.

    Args:
        state_code: Ignored (districts are national)
        year: Census year (2000, 2010, 2020)
        output_dir: Output directory for baseline
        force: Force redownload even if exists

    Returns:
        Tuple of (success: bool, message: str)
    """
    import geopandas as gpd

    # Build output path
    baseline_dir = Path(output_dir) / f'baseline/{year}'
    baseline_dir.mkdir(parents=True, exist_ok=True)

    # Determine which Congress based on year
    congress_map = {
        2000: '107',  # 107th Congress (2001-2003)
        2010: '113',  # 113th Congress (2013-2015)
        2020: '118'   # 118th Congress (2023-2025)
    }

    congress = congress_map.get(year)
    if not congress:
        return (False, f"[ERROR] No congressional districts defined for {year}")

    output_file = baseline_dir / f'us_cd{congress}_{year}.parquet'

    # Check if already exists
    if check_existing_file(output_file, force=force):
        return (True, f"[SKIP] Enacted districts already exist: us_cd{congress}_{year}.parquet")

    # Get download URL
    if year not in TIGER_CD_URLS:
        return (False, f"[ERROR] Congressional districts not available for {year}")

    cd_config = TIGER_CD_URLS[year]

    # Get list of URLs to try
    urls_to_try = []

    # Try primary URL first
    if f'cd{congress}' in cd_config:
        urls_to_try.append(cd_config[f'cd{congress}'])

    # Add fallback URLs
    if 'fallback' in cd_config:
        for url in cd_config['fallback']:
            if url not in urls_to_try:
                urls_to_try.append(url)

    if not urls_to_try:
        return (False, f"[ERROR] No URL found for {congress}th Congress")

    # Try each URL until one works
    last_error = None
    for url in urls_to_try:
        try:
            send_status(f"Downloading enacted districts from {url}")
            cd_gdf = gpd.read_file(url)

            # Ensure EPSG:4326
            if cd_gdf.crs != 'EPSG:4326':
                cd_gdf = cd_gdf.to_crs('EPSG:4326')

            # Save to parquet
            cd_gdf.to_parquet(output_file)

            return (True, f"[OK] Downloaded enacted districts: {len(cd_gdf)} districts")

        except Exception as e:
            last_error = e
            continue  # Try next URL

    return (False, f"[FAIL] Failed to download enacted districts: {last_error}")


def main():
    parser = argparse.ArgumentParser(description='Download census data for a single state')
    parser.add_argument('--state', type=str, required=True, help='State code (e.g., CA, TX)')
    parser.add_argument('--type', type=str, required=True,
                       choices=['redistricting', 'demographics', 'places', 'elections', 'metros', 'enacted_districts', 'all'],
                       help='Data type to download (aligned with pipeline stages)')
    parser.add_argument('--year', type=str, default='2020', help='Census year (2000, 2010, 2020)')
    parser.add_argument('--output-dir', type=str, required=True, help='Output directory')
    parser.add_argument('--force', action='store_true', help='Force redownload even if exists')
    args = parser.parse_args()

    # Validate inputs
    try:
        validate_state_code(args.state)
        year_int = validate_year(args.year)
    except ValueError as e:
        print(f"[ERROR] {e}", file=sys.stderr)
        return 1

    state_code = args.state.upper()
    state_name = state_code.lower()

    # Get state number for progress tracking (approximate)
    from scripts.config.download_sources import ALL_STATES
    try:
        state_num = ALL_STATES.index(state_code) + 1
    except ValueError:
        state_num = 0

    # Define download steps based on type
    if args.type == 'redistricting':
        steps = [
            ('Checking existing', None),
            ('Downloading ZIP', download_tracts),
            ('Extracting files', None),
            ('Validating', None)
        ]
    elif args.type == 'demographics':
        steps = [
            ('Checking existing', None),
            ('Downloading CSV', download_demographics),
            ('Processing data', None),
            ('Validating', None)
        ]
    elif args.type == 'places':
        steps = [
            ('Checking existing', None),
            ('Downloading ZIP', download_places),
            ('Extracting files', None),
            ('Validating', None)
        ]
    elif args.type == 'metros':
        steps = [
            ('Checking existing', None),
            ('Downloading CBSA', download_metros),
            ('Processing data', None),
            ('Saving', None)
        ]
    elif args.type == 'enacted_districts':
        steps = [
            ('Checking existing', None),
            ('Downloading CD', download_enacted_districts),
            ('Processing data', None),
            ('Saving', None)
        ]
    elif args.type == 'all':
        print("[ERROR] 'all' type not implemented in worker (use orchestrator)", file=sys.stderr)
        return 1
    else:
        print(f"[ERROR] Type '{args.type}' not yet implemented", file=sys.stderr)
        return 1

    # Execute download
    for i, (step_label, download_func) in enumerate(steps, 1):
        step_desc = step_label.lower().replace(' ', '_')

        # Send status update
        send_status(
            f"{state_name} {step_label}",
            state_num=state_num,
            state_name=state_name,
            step=i,
            step_total=len(steps),
            step_desc=step_desc
        )

        # Only execute if function provided (some steps are placeholders)
        if download_func:
            success, message = download_func(state_code, year_int, args.output_dir, args.force)

            print(message)

            if not success:
                print(f"[FAIL] {state_name} failed at {step_label}", file=sys.stderr)
                return 1

    # Success
    send_status(
        f"{state_name} COMPLETE",
        state_num=state_num,
        state_name=state_name,
        step=len(steps),
        step_total=len(steps),
        step_desc='complete'
    )

    return 0


if __name__ == '__main__':
    sys.exit(main())
