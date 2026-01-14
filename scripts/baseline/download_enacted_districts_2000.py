"""
Download enacted 2000 Congressional District boundaries from Census Bureau.

Downloads TIGER/Line shapefiles for 107th Congressional Districts
(based on 2000 Census redistricting) for all 50 states.

Data Source:
- U.S. Census Bureau TIGER/Line Shapefiles
- 107th Congressional Districts (2001-2003, based on 2000 Census)
- URL: https://www2.census.gov/geo/tiger/TIGER2012/CD/tl_2012_us_cd107.zip
       (or alternative vintages if 2012 doesn't have CD107)

Note: 107th Congress was the first congress after 2000 Census redistricting

File naming: tl_2012_us_cd107.zip (nationwide file)
"""

import argparse
import sys
import urllib.request
from pathlib import Path
from typing import Dict, List, Tuple

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

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


def try_download(url: str, output_path: Path) -> Tuple[bool, str]:
    """
    Attempt to download from URL.

    Returns (success, message)
    """
    try:
        urllib.request.urlretrieve(url, output_path)
        size_mb = output_path.stat().st_size / (1024 * 1024)
        return True, f"Downloaded successfully ({size_mb:.2f} MB)"
    except urllib.error.HTTPError as e:
        return False, f"HTTP {e.code}: {e.reason}"
    except Exception as e:
        return False, str(e)


def download_enacted_districts_2000(output_dir: Path, skip_existing: bool = True):
    """
    Download enacted congressional district shapefiles for 2000 Census.

    Parameters
    ----------
    output_dir : Path
        Directory to save downloaded shapefiles
    skip_existing : bool, default True
        Skip if file already exists
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    print("="*70)
    print("  Downloading 107th Congressional Districts (2000 Census)")
    print("="*70)
    print(f"\nOutput directory: {output_dir}\n")

    # Try multiple potential sources for CD107
    # TIGER/Line data availability varies by vintage
    potential_sources = [
        ("TIGER2012", "tl_2012_us_cd107.zip",
         "https://www2.census.gov/geo/tiger/TIGER2012/CD/tl_2012_us_cd107.zip"),
        ("TIGER2010", "tl_2010_us_cd107.zip",
         "https://www2.census.gov/geo/tiger/TIGER2010/CD/107/tl_2010_us_cd107.zip"),
        ("TIGER2004", "tl_2004_us_cd107.zip",
         "https://www2.census.gov/geo/tiger/TIGER2004/CD/tl_2004_us_cd107.zip"),
    ]

    filename = "tl_2012_us_cd107.zip"  # Preferred filename
    output_path = output_dir / filename

    # Check if already exists
    if skip_existing and output_path.exists():
        print(f"[SKIP] {filename} already exists")
        print(f"       ({output_path.stat().st_size / (1024 * 1024):.2f} MB)")
        print("\n" + "="*70)
        return True

    # Try each potential source
    print("Attempting to download from Census Bureau...")
    print("(Trying multiple TIGER vintages - CD107 availability varies)\n")

    for i, (vintage, fname, url) in enumerate(potential_sources, 1):
        print(f"[{i}/{len(potential_sources)}] Trying {vintage}: {fname}...")
        print(f"      URL: {url}")

        success, message = try_download(url, output_path)

        if success:
            print(f"      [OK] {message}")
            print(f"\nSuccessfully downloaded 107th Congressional Districts!")
            print(f"Saved as: {output_path}")
            print("\n" + "="*70)
            return True
        else:
            print(f"      [FAIL] {message}")
            # Clean up failed download
            if output_path.exists():
                output_path.unlink()

    # All sources failed
    print("\n" + "="*70)
    print("ERROR: Could not download CD107 from any source")
    print("="*70)
    print("\nAlternative options:")
    print("1. Manually download from: https://www.census.gov/geographies/mapping-files/")
    print("   Search for '107th Congressional Districts' or 'CD107'")
    print(f"2. Save the file as: {output_path}")
    print("3. Re-run this script with --skip-existing")
    print("\n" + "="*70)
    return False


def main():
    parser = argparse.ArgumentParser(
        description='Download 2000 Census enacted congressional districts (CD107)'
    )
    parser.add_argument('--output-dir', type=str,
                       default='data/enacted_districts/2000',
                       help='Output directory for shapefiles')
    parser.add_argument('--no-skip-existing', action='store_true',
                       help='Re-download even if file exists')

    args = parser.parse_args()

    output_dir = Path(args.output_dir)
    skip_existing = not args.no_skip_existing

    success = download_enacted_districts_2000(output_dir, skip_existing=skip_existing)

    return 0 if success else 1


if __name__ == '__main__':
    sys.exit(main())
