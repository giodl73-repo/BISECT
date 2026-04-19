"""
Download enacted 2010 Congressional District boundaries from Census Bureau.

Downloads TIGER/Line shapefiles for 112th Congressional Districts
(based on 2010 Census redistricting) for all 50 states.

Data Source:
- U.S. Census Bureau TIGER/Line Shapefiles
- 112th Congressional Districts (2011-2013, based on 2010 Census)
- URL: https://www2.census.gov/geo/tiger/TIGER2012/CD/tl_2012_us_cd112.zip

Note: 112th Congress was the first congress after 2010 Census redistricting

File naming: tl_2012_us_cd112.zip (nationwide file only)
"""

import argparse
import sys
import urllib.request
from pathlib import Path
from typing import Dict

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2010 import STATE_CONFIG_2010


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


def download_enacted_districts_2010(output_dir: Path, use_nationwide: bool = True, skip_existing: bool = True):
    """
    Download enacted congressional district shapefiles for 2010 Census.

    Parameters
    ----------
    output_dir : Path
        Directory to save downloaded shapefiles
    use_nationwide : bool, default True
        Download single nationwide file vs per-state files
    skip_existing : bool, default True
        Skip if file already exists
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    # Only download states with districts (exclude DC, territories)
    states_with_districts = [
        state for state, config in STATE_CONFIG_2010.items()
        if config['districts'] > 0
    ]

    if use_nationwide:
        # Download single nationwide CD112 shapefile
        print(f"Downloading nationwide 112th Congressional Districts (2010 Census)...")
        print(f"Output directory: {output_dir}\n")

        filename = "tl_2012_us_cd112.zip"
        output_path = output_dir / filename
        url = f"https://www2.census.gov/geo/tiger/TIGER2012/CD/{filename}"

        if skip_existing and output_path.exists():
            print(f"[SKIP] {filename} already exists")
            print(f"       ({output_path.stat().st_size / (1024 * 1024):.2f} MB)")
            return True

        try:
            print(f"[DOWNLOAD] {filename}...", flush=True)
            print(f"           This may take several minutes (nationwide file)...")
            urllib.request.urlretrieve(url, output_path)
            file_size_mb = output_path.stat().st_size / (1024 * 1024)
            print(f"[OK] Downloaded {file_size_mb:.2f} MB")
            return True
        except Exception as e:
            print(f"[ERROR] {e}")
            if output_path.exists():
                output_path.unlink()
            return False

    else:
        # Download per-state files (if available)
        print(f"Downloading enacted congressional districts for {len(states_with_districts)} states...")
        print(f"Output directory: {output_dir}\n")

        success_count = 0
        skip_count = 0
        error_count = 0

        # Try TIGER2013 CD directory for per-state files
        base_url = "https://www2.census.gov/geo/tiger/TIGER2013/CD"

        for state in sorted(states_with_districts):
            fips = STATE_FIPS[state]
            filename = f"tl_2013_{fips}_cd113.zip"
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

        return error_count == 0


def main():
    parser = argparse.ArgumentParser(
        description='Download enacted 2010 congressional district boundaries (CD113)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/enacted_districts/2010'),
        help='Output directory for shapefiles (default: data/enacted_districts/2010)'
    )
    parser.add_argument(
        '--per-state',
        action='store_true',
        help='Download per-state files instead of nationwide file'
    )
    parser.add_argument(
        '--force',
        action='store_true',
        help='Re-download even if files exist'
    )

    args = parser.parse_args()

    success = download_enacted_districts_2010(
        output_dir=args.output_dir,
        use_nationwide=not args.per_state,
        skip_existing=not args.force
    )

    if success:
        print("\n[OK] Download complete!")
        print(f"\nNext steps:")
        print(f"1. Compute compactness: python scripts/baseline/compute_enacted_compactness_2010.py")
        print(f"2. Compare results: python scripts/baseline/compare_algorithmic_vs_enacted.py --year 2010")
    else:
        print("\n[ERROR] Download failed")

    sys.exit(0 if success else 1)


if __name__ == '__main__':
    main()
