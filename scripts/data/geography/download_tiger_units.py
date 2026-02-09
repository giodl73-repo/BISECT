#!/usr/bin/env python3
"""
Download TIGER/Line geographic unit shapefiles from Census Bureau.

Downloads tract or block shapefiles to data/{year}/tiger/{resolution}/.

Tracts (2020): tl_2020_{fips}_tract
Tracts (2010): tl_2010_{fips}_tract10
Tracts (2000): tl_2010_{fips}_tract00 (uses 2010 TIGER/Line with 2000 boundaries)

Block Groups (2020): tl_2020_{fips}_bg
Block Groups (2010): tl_2010_{fips}_bg10
Block Groups (2000): tl_2010_{fips}_bg00 (uses 2010 TIGER/Line with 2000 boundaries)

Blocks (2020): tl_2020_{fips}_tabblock20
Blocks (2010): tl_2010_{fips}_tabblock10
Blocks (2000): tl_2010_{fips}_tabblock00 (uses 2010 TIGER/Line with 2000 boundaries)

Usage:
    python scripts/data/geography/download_tiger_units.py --year 2020 --resolution tract
    python scripts/data/geography/download_tiger_units.py --year 2020 --resolution block_group --states VT DE
    python scripts/data/geography/download_tiger_units.py --year 2020 --resolution block --states VT DE
    python scripts/data/geography/download_tiger_units.py --year 2020 --resolution tract --force
"""

import argparse
import sys
from pathlib import Path
import urllib.request
import zipfile
import io
from tqdm import tqdm

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


def download_tiger_unit(state_code, state_fips, year, resolution, output_dir, force=False):
    """
    Download TIGER/Line geographic unit shapefile for one state.

    Args:
        state_code: Two-letter state code (e.g., 'CA')
        state_fips: Two-digit FIPS code (e.g., '06')
        year: Census year (2000, 2010, 2020)
        resolution: Geographic resolution ('tract' or 'block')
        output_dir: Base output directory (data/{year}/tiger/{resolution}/)
        force: Redownload even if file exists
    """
    # Determine filename and URL based on year and resolution
    if resolution == 'tract':
        if year == 2020:
            filename = f"tl_2020_{state_fips}_tract"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2020/TRACT"
        elif year == 2010:
            filename = f"tl_2010_{state_fips}_tract10"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010"
        elif year == 2000:
            # 2000 uses 2010 TIGER/Line with 2000 boundaries
            filename = f"tl_2010_{state_fips}_tract00"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2000"
        else:
            print(f"  ERROR: Unsupported year: {year}")
            return False
    elif resolution == 'block_group':
        if year == 2020:
            filename = f"tl_2020_{state_fips}_bg"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2020/BG"
        elif year == 2010:
            filename = f"tl_2010_{state_fips}_bg10"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/BG/2010"
        elif year == 2000:
            # 2000 uses 2010 TIGER/Line with 2000 boundaries
            filename = f"tl_2010_{state_fips}_bg00"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/BG/2000"
        else:
            print(f"  ERROR: Unsupported year: {year}")
            return False
    elif resolution == 'block':
        if year == 2020:
            filename = f"tl_2020_{state_fips}_tabblock20"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2020/TABBLOCK20"
        elif year == 2010:
            filename = f"tl_2010_{state_fips}_tabblock10"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/TABBLOCK/2010"
        elif year == 2000:
            # 2000 uses 2010 TIGER/Line with 2000 boundaries
            filename = f"tl_2010_{state_fips}_tabblock00"
            base_url = "https://www2.census.gov/geo/tiger/TIGER2010/TABBLOCK/2000"
        else:
            print(f"  ERROR: Unsupported year: {year}")
            return False
    else:
        print(f"  ERROR: Unsupported resolution: {resolution}. Supported: tract, block_group, block")
        return False

    output_path = output_dir / filename

    # Check if already exists
    shapefile = output_path / f"{filename}.shp"
    if shapefile.exists() and not force:
        print(f"  [SKIP] Already exists: {filename}")
        return True

    # Download URL
    zip_url = f"{base_url}/{filename}.zip"

    print(f"  Downloading {filename}.zip...")

    try:
        # Download zip file
        with urllib.request.urlopen(zip_url) as response:
            zip_data = response.read()

        # Extract to directory
        output_path.mkdir(parents=True, exist_ok=True)

        with zipfile.ZipFile(io.BytesIO(zip_data)) as zip_ref:
            zip_ref.extractall(output_path)

        # Verify shapefile exists
        if not shapefile.exists():
            print(f"  ERROR: Shapefile not found after extraction: {shapefile}")
            return False

        print(f"  [OK] Downloaded and extracted to {output_path}")
        return True

    except urllib.error.HTTPError as e:
        print(f"  ERROR: HTTP {e.code} - {zip_url}")
        return False
    except Exception as e:
        print(f"  ERROR: {str(e)}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description='Download TIGER/Line geographic unit shapefiles',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Download all states for 2020 tracts
  python scripts/data/geography/download_tiger_tracts.py --year 2020 --resolution tract

  # Download specific states for 2020 blocks
  python scripts/data/geography/download_tiger_tracts.py --year 2020 --resolution block --states VT CA TX

  # Force redownload (overwrite existing)
  python scripts/data/geography/download_tiger_tracts.py --year 2020 --resolution tract --force
        """
    )

    parser.add_argument('--year', type=int, required=True, choices=[2000, 2010, 2020],
                       help='Census year')
    parser.add_argument('--resolution', type=str, required=True, choices=['tract', 'block_group', 'block'],
                       help='Geographic resolution (tract, block_group, or block)')
    parser.add_argument('--states', type=str, nargs='*',
                       help='Specific states to download (default: all 50)')
    parser.add_argument('--force', action='store_true',
                       help='Force redownload even if files exist')
    parser.add_argument('--output-dir', type=Path,
                       help='Override output directory (default: data/{year}/tiger/{resolution}/)')

    args = parser.parse_args()

    # Determine output directory
    if args.output_dir:
        output_dir = args.output_dir
    else:
        output_dir = Path(f'data/{args.year}/tiger/{args.resolution}s')

    output_dir.mkdir(parents=True, exist_ok=True)

    # Determine states to process
    if args.states:
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    print(f"\n{'='*70}")
    print(f"DOWNLOADING TIGER/LINE {args.resolution.upper()} SHAPEFILES FOR {args.year}")
    print(f"{'='*70}\n")
    print(f"Resolution: {args.resolution}")
    print(f"Output directory: {output_dir}")
    print(f"States: {len(states_to_process)}")
    if args.force:
        print(f"Force mode: Redownloading existing files")
    print()

    # Download each state
    success_count = 0
    failed_states = []

    for state_code, state_fips in tqdm(states_to_process, desc="Downloading"):
        print(f"\n{state_code}:")
        if download_tiger_unit(state_code, state_fips, args.year, args.resolution, output_dir, args.force):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    print(f"\n{'='*70}")
    print(f"Downloaded {success_count}/{len(states_to_process)} states successfully")
    if failed_states:
        print(f"Failed states: {', '.join(failed_states)}")
    print(f"{'='*70}\n")

    return 0 if len(failed_states) == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
