"""
Download enacted 2020 Congressional District boundaries from Census Bureau.

Downloads TIGER/Line shapefiles for 118th Congressional Districts
(based on 2020 Census redistricting) for all 50 states.

Data Source:
- U.S. Census Bureau TIGER/Line Shapefiles
- 118th Congressional Districts (2023-2024, based on 2020 Census)
- URL: https://www2.census.gov/geo/tiger/TIGER2020/CD/CD118/

File naming: tl_2020_{FIPS}_cd118.zip
"""

import argparse
import sys
import urllib.request
from pathlib import Path
from typing import Dict

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


# State FIPS codes
STATE_FIPS: Dict[str, str] = {
    'AL': '01', 'AK': '02', 'AZ': '04', 'AR': '05', 'CA': '06',
    'CO': '08', 'CT': '09', 'DE': '10', 'FL': '12', 'GA': '13',
    'HI': '15', 'ID': '16', 'IL': '17', 'IN': '18', 'IA': '19',
    'KS': '20', 'KY': '21', 'LA': '22', 'ME': '23', 'MD': '24',
    'MA': '25', 'MI': '26', 'MN': '27', 'MS': '28', 'MO': '29',
    'MT': '30', 'NE': '31', 'NV': '32', 'NH': '33', 'NJ': '34',
    'NM': '35', 'NY': '36', 'NC': '37', 'ND': '38', 'OH': '39',
    'OK': '40', 'OR': '41', 'PA': '42', 'RI': '44', 'SC': '45',
    'SD': '46', 'TN': '47', 'TX': '48', 'UT': '49', 'VT': '50',
    'VA': '51', 'WA': '53', 'WV': '54', 'WI': '55', 'WY': '56',
}


def download_enacted_districts(output_dir: Path, skip_existing: bool = True):
    """
    Download enacted congressional district shapefiles for all 50 states.

    Parameters
    ----------
    output_dir : Path
        Directory to save downloaded shapefiles
    skip_existing : bool, default True
        Skip states that are already downloaded
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    base_url = "https://www2.census.gov/geo/tiger/TIGER2020/CD/CD118"

    success_count = 0
    skip_count = 0
    error_count = 0

    # Only download states with districts (exclude DC, territories)
    states_with_districts = [
        state for state, config in STATE_CONFIG_2020.items()
        if config['districts'] > 0
    ]

    print(f"Downloading enacted congressional districts for {len(states_with_districts)} states...")
    print(f"Output directory: {output_dir}\n")

    for state in sorted(states_with_districts):
        fips = STATE_FIPS[state]
        filename = f"tl_2020_{fips}_cd118.zip"
        output_path = output_dir / filename

        if skip_existing and output_path.exists():
            print(f"[SKIP] {state}: {filename} already exists")
            skip_count += 1
            continue

        url = f"{base_url}/{filename}"

        try:
            print(f"[DOWNLOAD] {state}: {filename}...", end=" ", flush=True)
            urllib.request.urlretrieve(url, output_path)
            file_size_mb = output_path.stat().st_size / (1024 * 1024)
            print(f"OK ({file_size_mb:.2f} MB)")
            success_count += 1
        except Exception as e:
            print(f"ERROR: {e}")
            error_count += 1
            if output_path.exists():
                output_path.unlink()

    print(f"\n{'=' * 60}")
    print(f"Download Summary:")
    print(f"  Downloaded: {success_count}")
    print(f"  Skipped:    {skip_count}")
    print(f"  Errors:     {error_count}")
    print(f"  Total:      {len(states_with_districts)}")
    print(f"{'=' * 60}")

    if error_count > 0:
        print(f"\n⚠ WARNING: {error_count} downloads failed")
        return False

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Download enacted 2020 congressional district boundaries'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/enacted_districts'),
        help='Output directory for shapefiles (default: data/enacted_districts)'
    )
    parser.add_argument(
        '--force',
        action='store_true',
        help='Re-download even if files exist'
    )

    args = parser.parse_args()

    success = download_enacted_districts(
        output_dir=args.output_dir,
        skip_existing=not args.force
    )

    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()
