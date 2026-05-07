#!/usr/bin/env python3
"""
Download or process TIGER/Line 2000 census tract shapefiles for all states.

IMPORTANT: Census 2000 tract shapefiles are not directly available from Census Bureau
in the same format as 2010/2020. The recommended source is NHGIS (National Historical
Geographic Information System) which provides modernized, standardized shapefiles.

NHGIS: https://www.nhgis.org/

Manual download process:
1. Create free NHGIS account at https://www.nhgis.org/
2. Select "GIS Files" -> "Census Tract" -> "2000"
3. Select all states or specific states
4. Download and extract to data/geography/nhgis_2000_tracts/
5. Run this script with --process-nhgis flag to organize files

This script provides:
- Instructions for manual download
- Processing of NHGIS downloads into standardized structure
- Conversion to format matching 2010/2020 shapefiles
"""

import argparse
import sys
from pathlib import Path
import geopandas as gpd
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


def print_instructions():
    """Print instructions for manual NHGIS download."""
    print("=" * 70)
    print("CENSUS 2000 TRACT SHAPEFILE DOWNLOAD INSTRUCTIONS")
    print("=" * 70)
    print()
    print("Census 2000 tract boundaries must be obtained from NHGIS:")
    print()
    print("1. Go to https://www.nhgis.org/")
    print("2. Create a free account (if you don't have one)")
    print("3. Click 'Get Data' -> 'Select Data'")
    print("4. Under 'GIS Files', select:")
    print("   - Geographic Level: Census Tract")
    print("   - Year: 2000")
    print("   - Extent: All states (or specific states)")
    print("5. Add to cart and submit extract request")
    print("6. Download the extract when ready (usually <1 hour)")
    print("7. Extract ZIP file to: data/geography/nhgis_2000_tracts/")
    print()
    print("8. Run this script with --process-nhgis to organize files:")
    print("   python scripts/data/geography/download_tiger_tracts_2000.py --process-nhgis")
    print()
    print("=" * 70)


def process_nhgis_download(nhgis_dir, output_dir):
    """Process NHGIS downloaded files into standardized structure."""

    print("Processing NHGIS 2000 tract shapefiles...")
    print(f"NHGIS directory: {nhgis_dir}")
    print(f"Output directory: {output_dir}")
    print()

    if not nhgis_dir.exists():
        print(f"ERROR: NHGIS directory not found: {nhgis_dir}")
        print("Please download NHGIS data first (see instructions above)")
        return False

    # Find all shapefiles in NHGIS directory
    shapefiles = list(nhgis_dir.rglob("*.shp"))

    if not shapefiles:
        print(f"ERROR: No shapefiles found in {nhgis_dir}")
        print("Please ensure NHGIS extract is properly downloaded and extracted")
        return False

    print(f"Found {len(shapefiles)} shapefile(s)")
    print()

    # Process each shapefile
    output_dir.mkdir(parents=True, exist_ok=True)
    success_count = 0

    for shp_file in tqdm(shapefiles, desc="Processing"):
        try:
            # Read shapefile
            gdf = gpd.read_file(shp_file)

            # Determine state from geometry or filename
            # NHGIS files may be national or per-state
            if 'STATEFP' in gdf.columns or 'STATEFP00' in gdf.columns:
                state_col = 'STATEFP' if 'STATEFP' in gdf.columns else 'STATEFP00'
                states_in_file = gdf[state_col].unique()

                # Split by state and save
                for state_fips in states_in_file:
                    state_gdf = gdf[gdf[state_col] == state_fips].copy()

                    # Find state code from FIPS
                    state_code = None
                    for code, fips in STATE_FIPS.items():
                        if fips == state_fips:
                            state_code = code.lower()
                            break

                    if state_code:
                        output_file = output_dir / f"{state_code}_tracts_2000.shp"
                        state_gdf.to_file(output_file)
                        success_count += 1

            else:
                # Single-state file, save as-is
                output_file = output_dir / shp_file.name
                gdf.to_file(output_file)
                success_count += 1

        except Exception as e:
            print(f"  ERROR processing {shp_file.name}: {e}")

    print()
    print("=" * 60)
    print(f"Processed {success_count} state(s) successfully")
    print("=" * 60)

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Download/process 2000 census tract shapefiles from NHGIS',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Show download instructions
  python download_tiger_tracts_2000.py

  # Process NHGIS download
  python download_tiger_tracts_2000.py --process-nhgis

  # Process with custom directories
  python download_tiger_tracts_2000.py --process-nhgis \\
    --nhgis-dir data/nhgis_extract \\
    --output-dir data/geography/tiger_2000_tracts
        """
    )
    parser.add_argument(
        '--process-nhgis',
        action='store_true',
        help='Process NHGIS downloaded files into standardized structure'
    )
    parser.add_argument(
        '--nhgis-dir',
        type=Path,
        default=Path('data/geography/nhgis_2000_tracts'),
        help='Directory containing NHGIS downloaded files'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('data/geography/tiger_2000_tracts'),
        help='Output directory for processed shapefiles'
    )

    args = parser.parse_args()

    if args.process_nhgis:
        process_nhgis_download(args.nhgis_dir, args.output_dir)
    else:
        print_instructions()


if __name__ == '__main__':
    main()
