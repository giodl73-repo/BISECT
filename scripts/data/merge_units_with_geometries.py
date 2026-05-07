#!/usr/bin/env python3
"""
Merge census unit population data with TIGER/Line geometries (all years).

Combines:
- Population CSV from parse_pl94171_{resolution}s_{year}.py
- Shapefile geometries from data/{year}/tiger/{resolution}s/

Produces:
- Final GeoParquet files: outputs/{version}/data/{year}/units/{state}_{resolution}s_{year}.parquet

Supports tract and block resolutions for 2000, 2010, and 2020 census data.
"""

import argparse
import os
import sys
from pathlib import Path
import warnings

# Suppress ALL warnings at every level
warnings.filterwarnings('ignore')
os.environ['PYTHONWARNINGS'] = 'ignore'

# Suppress pandas/matplotlib/geopandas at import time
if not sys.warnoptions:
    warnings.simplefilter('ignore')

import pandas as pd
import geopandas as gpd

# Additional pandas-specific warning suppression
pd.options.mode.chained_assignment = None  # Suppress SettingWithCopyWarning
pd.set_option('mode.copy_on_write', True)

# Redirect stderr to devnull when in child mode to prevent any leakage
if int(os.environ.get('TQDM_POSITION', '-1')) >= 0:
    sys.stderr = open(os.devnull, 'w')

# Add project root to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from tqdm import tqdm
from scripts.utils.status_protocol import StatusReporter
from scripts.utils.paths import get_census_data_dir

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


# Global reporter instance
_reporter = StatusReporter()


def get_shapefile_path(tiger_dir, state_fips, year, resolution='tract'):
    """Get shapefile path based on year and resolution."""
    if resolution == 'tract':
        if year == 2020:
            # tl_2020_{fips}_tract/tl_2020_{fips}_tract.shp
            return tiger_dir / f"tl_2020_{state_fips}_tract" / f"tl_2020_{state_fips}_tract.shp"
        elif year == 2010:
            # tl_2010_{fips}_tract10/tl_2010_{fips}_tract10.shp
            return tiger_dir / f"tl_2010_{state_fips}_tract10" / f"tl_2010_{state_fips}_tract10.shp"
        elif year == 2000:
            # tl_2010_{fips}_tract00/tl_2010_{fips}_tract00.shp (2000 uses 2010 TIGER/Line)
            return tiger_dir / f"tl_2010_{state_fips}_tract00" / f"tl_2010_{state_fips}_tract00.shp"
        else:
            raise ValueError(f"Unsupported year: {year}")
    elif resolution == 'block_group':
        if year == 2020:
            # tl_2020_{fips}_bg/tl_2020_{fips}_bg.shp
            return tiger_dir / f"tl_2020_{state_fips}_bg" / f"tl_2020_{state_fips}_bg.shp"
        elif year == 2010:
            # tl_2010_{fips}_bg10/tl_2010_{fips}_bg10.shp
            return tiger_dir / f"tl_2010_{state_fips}_bg10" / f"tl_2010_{state_fips}_bg10.shp"
        elif year == 2000:
            # tl_2010_{fips}_bg00/tl_2010_{fips}_bg00.shp
            return tiger_dir / f"tl_2010_{state_fips}_bg00" / f"tl_2010_{state_fips}_bg00.shp"
        else:
            raise ValueError(f"Unsupported year: {year}")
    elif resolution == 'block':
        if year == 2020:
            # tl_2020_{fips}_tabblock20/tl_2020_{fips}_tabblock20.shp
            return tiger_dir / f"tl_2020_{state_fips}_tabblock20" / f"tl_2020_{state_fips}_tabblock20.shp"
        elif year == 2010:
            # tl_2010_{fips}_tabblock10/tl_2010_{fips}_tabblock10.shp
            return tiger_dir / f"tl_2010_{state_fips}_tabblock10" / f"tl_2010_{state_fips}_tabblock10.shp"
        elif year == 2000:
            # tl_2010_{fips}_tabblock00/tl_2010_{fips}_tabblock00.shp
            return tiger_dir / f"tl_2010_{state_fips}_tabblock00" / f"tl_2010_{state_fips}_tabblock00.shp"
        else:
            raise ValueError(f"Unsupported year: {year}")
    else:
        raise ValueError(f"Unsupported resolution: {resolution}. Supported: tract, block_group, block")


def normalize_geoid(gdf, year):
    """Normalize GEOID column name based on year."""
    if 'GEOID' in gdf.columns:
        return gdf
    elif year == 2020 and 'GEOID20' in gdf.columns:
        return gdf.rename(columns={'GEOID20': 'GEOID'})
    elif year == 2010 and 'GEOID10' in gdf.columns:
        return gdf.rename(columns={'GEOID10': 'GEOID'})
    elif year == 2000 and 'CTIDFP00' in gdf.columns:
        # 2000 might use different field name
        return gdf.rename(columns={'CTIDFP00': 'GEOID'})
    else:
        raise ValueError(f"Could not find GEOID column. Available: {list(gdf.columns)}")


def merge_state_data(state_code, state_fips, year, csv_dir, tiger_dir, output_dir, resolution='tract', use_status=False):
    """Merge census population data with shapefile geometries for one state."""

    state_lower = state_code.lower()

    def log(msg):
        if use_status:
            _reporter.report(f"{state_code}: {msg}")
        else:
            print(f"  {msg}")

    # Input files
    pop_csv = csv_dir / f"{state_lower}_{resolution}s_{year}_population.csv"
    shapefile = get_shapefile_path(tiger_dir, state_fips, year, resolution)

    if not pop_csv.exists():
        log(f"ERROR: Population CSV not found: {pop_csv}")
        return False

    if not shapefile.exists():
        log(f"ERROR: Shapefile not found: {shapefile}")
        return False

    # Read population data
    log("Reading population data...")
    pop_df = pd.read_csv(pop_csv, dtype={'GEOID': str})

    # Read shapefile
    log("Reading shapefile...")
    gdf = gpd.read_file(shapefile)

    # Normalize GEOID column
    try:
        gdf = normalize_geoid(gdf, year)
    except ValueError as e:
        log(f"ERROR: {e}")
        return False

    # Convert GEOID to string for consistent merging
    gdf['GEOID'] = gdf['GEOID'].astype(str)
    pop_df['GEOID'] = pop_df['GEOID'].astype(str)

    # Strip Census prefixes from CSV GEOID if present (2020 PL 94-171 format)
    # 1400000US = tract (11 digits), 1500000US = block group (12 digits), 7500000US = block (15 digits)
    if pop_df['GEOID'].str.startswith('1400000US').any():
        pop_df['GEOID'] = pop_df['GEOID'].str.replace('1400000US', '', regex=False)
    elif pop_df['GEOID'].str.startswith('1500000US').any():
        pop_df['GEOID'] = pop_df['GEOID'].str.replace('1500000US', '', regex=False)
    elif pop_df['GEOID'].str.startswith('7500000US').any():
        pop_df['GEOID'] = pop_df['GEOID'].str.replace('7500000US', '', regex=False)

    # Merge on GEOID
    log("Merging data...")
    merged = gdf[['GEOID', 'geometry']].merge(pop_df, on='GEOID', how='inner')

    if len(merged) == 0:
        log(f"ERROR: No matching records after merge")
        log(f"Shapefile records: {len(gdf)}, Sample: {gdf['GEOID'].iloc[0] if len(gdf) > 0 else 'N/A'}")
        log(f"Population records: {len(pop_df)}, Sample: {pop_df['GEOID'].iloc[0] if len(pop_df) > 0 else 'N/A'}")
        return False

    if len(merged) < len(pop_df):
        missing = len(pop_df) - len(merged)
        log(f"WARNING: {missing} tracts have no geometry ({len(pop_df)} tracts, {len(merged)} with geometry)")

    # Ensure CRS is set (should be EPSG:4269 for all TIGER/Line)
    if merged.crs is None:
        merged = merged.set_crs('EPSG:4269')

    # Save to parquet
    output_file = output_dir / f"{state_lower}_{resolution}s_{year}.parquet"
    merged.to_parquet(output_file, index=False)

    log(f"Saved {len(merged)} tracts to {output_file.name} (CRS: {merged.crs})")

    return True


def main():
    parser = argparse.ArgumentParser(
        description='Merge census unit data with TIGER/Line geometries',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Merge all states for 2020 tracts
  python scripts/data/merge_tracts_with_geometries.py --year 2020 --resolution tract

  # Merge specific states for 2020 blocks
  python scripts/data/merge_tracts_with_geometries.py --year 2020 --resolution block --states VT CA TX

  # Custom directories
  python scripts/data/merge_tracts_with_geometries.py --year 2020 --resolution tract \\
    --csv-dir outputs/data/units/2020 \\
    --tiger-dir data/2020/tiger/tracts \\
    --output-dir outputs/data/units/2020
        """
    )
    parser.add_argument(
        '--year',
        type=int,
        required=True,
        choices=[2000, 2010, 2020],
        help='Census year'
    )
    parser.add_argument(
        '--version',
        type=str,
        help='Version identifier (e.g., v1, test). Optional for compatibility with census processing pipeline.'
    )
    parser.add_argument(
        '--resolution',
        type=str,
        default='tract',
        choices=['tract', 'block_group', 'block'],
        help='Geographic resolution (tract, block_group, or block; default: tract)'
    )
    parser.add_argument(
        '--csv-dir',
        type=Path,
        help='Directory containing population CSV files (default: outputs/data/{year}/units)'
    )
    parser.add_argument(
        '--tiger-dir',
        type=Path,
        help='Directory containing TIGER/Line shapefiles (default: data/{year}/tiger/{resolution}s)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        help='Output directory for merged parquet files (default: outputs/data/{year}/units)'
    )
    parser.add_argument(
        '--states',
        type=str,
        nargs='*',
        help='Process specific states (e.g., AL CA TX). If not specified, processes all states.'
    )

    args = parser.parse_args()

    # Set default directories based on year, version, and resolution
    if args.csv_dir is None:
        if args.version:
            # Version-specific path
            args.csv_dir = get_census_data_dir(args.version, str(args.year)) / 'units'
        else:
            # Legacy shared path for backward compatibility
            args.csv_dir = Path(f'outputs/data/{args.year}/units')
    if args.tiger_dir is None:
        args.tiger_dir = Path(f'data/{args.year}/tiger/{args.resolution}s')
    if args.output_dir is None:
        if args.version:
            # Version-specific path
            args.output_dir = get_census_data_dir(args.version, str(args.year)) / 'units'
        else:
            # Legacy shared path for backward compatibility
            args.output_dir = Path(f'outputs/data/{args.year}/units')

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Determine which states to process
    if args.states:
        states_to_process = [(s.upper(), STATE_FIPS[s.upper()]) for s in args.states]
    else:
        states_to_process = [(code, fips) for code, fips in sorted(STATE_FIPS.items())]

    # Check if running as child process (uses STATUS protocol)
    tqdm_pos = int(os.environ.get('TQDM_POSITION', '-1'))
    use_status = tqdm_pos >= 0

    if use_status:
        _reporter.report(f"Merging {len(states_to_process)} states ({args.year})")
    else:
        print(f"\nMerging {len(states_to_process)} states ({args.year})...")
        print(f"CSV directory: {args.csv_dir}")
        print(f"TIGER directory: {args.tiger_dir}")
        print(f"Output directory: {args.output_dir}")
        print()

    # Process each state
    success_count = 0
    failed_states = []

    # Use tqdm only when running standalone
    iterator = states_to_process if use_status else tqdm(states_to_process, desc="Merging states")

    for state_code, state_fips in iterator:
        if not use_status:
            print(f"\n{state_code}:")
        if merge_state_data(state_code, state_fips, args.year, args.csv_dir, args.tiger_dir, args.output_dir, args.resolution, use_status):
            success_count += 1
        else:
            failed_states.append(state_code)

    # Summary
    if use_status:
        _reporter.report(f"Completed {success_count}/{len(states_to_process)} states")
        if failed_states:
            _reporter.report(f"Failed: {', '.join(failed_states)}")
    else:
        print("\n" + "=" * 60)
        print(f"Merged {success_count}/{len(states_to_process)} states successfully")
        if failed_states:
            print(f"Failed states: {', '.join(failed_states)}")
        print("=" * 60)

    return 0 if len(failed_states) == 0 else 1


if __name__ == '__main__':
    sys.exit(main())
