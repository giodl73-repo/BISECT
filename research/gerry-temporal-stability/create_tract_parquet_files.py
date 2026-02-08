"""
Create tract parquet files for 2010 in pipeline format.

Merges demographics CSV with TIGER geometries and saves as GeoParquet.
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

# States for temporal stability analysis
STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']

# State name to FIPS code mapping
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}


def create_tract_parquet(state: str):
    """Create tract parquet file for one state."""
    print(f"\nCreating tract parquet for {state}...")

    project_root = Path(__file__).parent.parent.parent
    fips = STATE_TO_FIPS[state]

    # Load TIGER geometries
    tracts_dir = project_root / f'data/2010/tiger/tracts/tl_2010_{fips}_tract10'
    tracts = gpd.read_file(tracts_dir)
    print(f"  Loaded {len(tracts)} tracts from TIGER")

    # Rename GEOID10 to GEOID for consistency
    if 'GEOID10' in tracts.columns:
        tracts = tracts.rename(columns={'GEOID10': 'GEOID'})

    # Load demographics
    demographics_file = project_root / f'data/2010/demographics/{state}_demographics_2010.csv'
    demographics = pd.read_csv(demographics_file)
    print(f"  Loaded {len(demographics)} tracts from demographics")

    # Ensure GEOID is string in both with proper zero-padding
    tracts['GEOID'] = tracts['GEOID'].astype(str)
    # Demographics GEOIDs are stored as integers, need to be zero-padded to 11 digits
    demographics['GEOID'] = demographics['GEOID'].astype(str).str.zfill(11)

    # Merge geometries with demographics
    merged = tracts.merge(demographics, on='GEOID', how='inner')
    print(f"  Merged: {len(merged)} tracts")

    if len(merged) < len(demographics):
        print(f"  WARNING: Lost {len(demographics) - len(merged)} tracts in merge")

    # Select columns for output (keep GEOID, geometry, and demographic columns)
    # Standard columns expected by pipeline
    output_cols = ['GEOID', 'geometry']

    # Add demographic columns if they exist
    demographic_cols = [
        'total_pop', 'hispanic', 'black_non_hispanic', 'asian_non_hispanic',
        'native_american', 'pacific_islander', 'other', 'two_or_more',
        'white_non_hispanic'
    ]

    for col in demographic_cols:
        if col in merged.columns:
            output_cols.append(col)

    merged_subset = merged[output_cols]

    # Save as parquet
    output_dir = project_root / 'outputs/v1/data/2010/units'
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f'{state}_tracts_2010.parquet'

    merged_subset.to_parquet(output_file, index=False)

    print(f"  Saved: {output_file}")
    print(f"  Size: {output_file.stat().st_size / 1024:.1f} KB")
    print(f"  Columns: {', '.join(output_cols)}")


def main():
    """Create tract parquet files for all states."""
    print("="*70)
    print("CREATE TRACT PARQUET FILES")
    print("="*70)
    print(f"States: {', '.join(STATES)}")
    print(f"Input: data/2010/tiger/tracts/ + data/2010/demographics/")
    print(f"Output: outputs/v1/data/2010/units/*_tracts_2010.parquet")
    print()

    for state in STATES:
        try:
            create_tract_parquet(state)
        except Exception as e:
            print(f"  ERROR: {e}")
            import traceback
            traceback.print_exc()

    print()
    print("="*70)
    print("PARQUET FILES CREATED")
    print("="*70)


if __name__ == '__main__':
    main()
