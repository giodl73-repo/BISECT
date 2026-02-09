#!/usr/bin/env python3
"""
Download TIGER/Line 2010 census tract shapefiles for all states.

Downloads from Census Bureau FTP:
https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/
"""

import argparse
import sys
from pathlib import Path
import requests
from tqdm import tqdm
import zipfile
import io

# State FIPS codes
STATE_FIPS = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
    'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
    'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
    'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
    'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
    'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
    'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
    'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
    'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56'
}

BASE_URL = "https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/"


def download_state_tracts(state_code, state_fips, output_dir):
    """Download census tract shapefile for one state."""

    # File naming: tl_2010_01_tract10.zip (state FIPS code)
    filename = f"tl_2010_{state_fips}_tract10.zip"
    url = BASE_URL + filename

    output_path = output_dir / filename

    # Skip if already downloaded
    if output_path.exists():
        print(f"  Already exists, skipping")
        return True

    try:
        # Download with progress bar
        response = requests.get(url, stream=True, timeout=60)
        response.raise_for_status()

        total_size = int(response.headers.get('content-length', 0))

        with open(output_path, 'wb') as f:
            if total_size > 0:
                pbar = tqdm(total=total_size, unit='B', unit_scale=True,
                           desc=f"  Downloading", leave=False)
                for chunk in response.iter_content(chunk_size=8192):
                    f.write(chunk)
                    pbar.update(len(chunk))
                pbar.close()
            else:
                f.write(response.content)

        print(f"  Downloaded {filename} ({total_size / 1024 / 1024:.1f} MB)")
        return True

    except requests.exceptions.RequestException as e:
        print(f"  ERROR downloading {filename}: {e}")
        if output_path.exists():
            output_path.unlink()
        return False


def extract_shapefiles(output_dir):
    """Extract all downloaded ZIP files."""
    print("\nExtracting shapefiles...")

    zip_files = list(output_dir.glob("*.zip"))

    for zip_file in tqdm(zip_files, desc="Extracting"):
        extract_dir = output_dir / zip_file.stem

        # Skip if already extracted
        if extract_dir.exists():
            continue

        extract_dir.mkdir(parents=True, exist_ok=True)

        try:
            with zipfile.ZipFile(zip_file, 'r') as zf:
                zf.extractall(extract_dir)
        except Exception as e:
            print(f"  ERROR extracting {zip_file.name}: {e}")


def main():
    parser = argparse.ArgumentParser(
        description='Download TIGER/Line 2010 census tract shapefiles'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/geography/tiger_2010_tracts'),
        help='Output directory for shapefiles'
    )
    parser.add_argument(
        '--states',
        type=str,
        nargs='+',
        help='Download specific states (e.g., AL CA TX). If not specified, downloads all states.'
    )
    parser.add_argument(
        '--no-extract',
        action='store_true',
        help='Skip extraction of ZIP files'
    )

    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    print(f"Downloading {len(states_to_process)} state tract shapefiles (2010)...")
    print(f"Output directory: {args.output_dir}")
    print()

    # Download each state
    success_count = 0
    failed_states = []

    for state_code, state_fips in tqdm(states_to_process, desc="Downloading states"):
        print(f"\n{state_code}:")
        if download_state_tracts(state_code, state_fips, args.output_dir):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Extract ZIP files
    if not args.no_extract:
        extract_shapefiles(args.output_dir)

    # Summary
    print("\n" + "=" * 60)
    print(f"Downloaded {success_count}/{len(states_to_process)} states successfully")
    if failed_states:
        print(f"Failed states: {', '.join(failed_states)}")
    print("=" * 60)


if __name__ == '__main__':
    main()
