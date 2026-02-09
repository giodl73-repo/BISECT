#!/usr/bin/env python3
"""
Add mock population data for testing purposes.

For real redistricting, you should use Census API or manually add population data.
"""

import sys
from pathlib import Path
import numpy as np

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks
import geopandas as gpd

def add_mock_population(input_file, output_file=None):
    """Add mock population based on land area."""
    blocks_gdf = load_blocks(input_file)

    # Assign population proportional to land area (rough approximation)
    # Use Alpine County's actual population (~1,200) distributed across blocks
    total_land_area = blocks_gdf['ALAND'].sum()

    if total_land_area > 0:
        # Distribute 1200 people across blocks proportional to land area
        blocks_gdf['population'] = (blocks_gdf['ALAND'] / total_land_area * 1200).astype(int)

        # Ensure at least some blocks have population
        # Give at least 1 person to largest blocks
        zero_pop = blocks_gdf['population'] == 0
        if zero_pop.sum() > 0:
            # Add 1 to top 50% of blocks by area
            median_area = blocks_gdf['ALAND'].median()
            large_blocks = (blocks_gdf['ALAND'] >= median_area) & zero_pop
            blocks_gdf.loc[large_blocks, 'population'] = 1

    print(f"\nMock Population Summary:")
    print(f"  Total population: {blocks_gdf['population'].sum():,}")
    print(f"  Blocks with population > 0: {(blocks_gdf['population'] > 0).sum()}")

    # Save
    if output_file is None:
        output_file = input_file

    blocks_gdf.to_parquet(output_file, compression='snappy')
    print(f"\nSaved to: {output_file}")

if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description='Add mock population data')
    parser.add_argument('input_file', help='Input parquet file')
    parser.add_argument('--output', help='Output file (default: overwrite input)')

    args = parser.parse_args()

    add_mock_population(args.input_file, args.output)
