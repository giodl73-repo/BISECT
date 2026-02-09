#!/usr/bin/env python3
"""
Download Census block data from TIGER/Line shapefiles.

Example usage:
    python scripts/download_data.py --state CA --year 2020
    python scripts/download_data.py --state CA --year 2020 --county 075  # San Francisco only
"""

import argparse
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import download_blocks


def main():
    parser = argparse.ArgumentParser(
        description='Download Census block data from TIGER/Line shapefiles'
    )
    parser.add_argument(
        '--state',
        type=str,
        required=True,
        help='State code (e.g., CA for California)'
    )
    parser.add_argument(
        '--year',
        type=int,
        default=2020,
        help='Census year (default: 2020)'
    )
    parser.add_argument(
        '--county',
        type=str,
        default=None,
        help='County FIPS code for single county download (optional)'
    )
    parser.add_argument(
        '--cache-dir',
        type=str,
        default='cache',
        help='Directory for pygris cache (default: ./cache)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='data/raw',
        help='Output directory for processed data (default: ./data/raw)'
    )

    args = parser.parse_args()

    print(f"\n{'=' * 60}")
    print(f"Census Block Data Download")
    print(f"{'=' * 60}\n")

    try:
        blocks_gdf = download_blocks(
            state=args.state,
            year=args.year,
            county=args.county,
            cache_dir=args.cache_dir,
            output_dir=args.output_dir
        )

        print(f"\n{'=' * 60}")
        print(f"Download Complete!")
        print(f"{'=' * 60}\n")
        print(f"Downloaded {len(blocks_gdf):,} blocks")
        print(f"Total population: {blocks_gdf['population'].sum():,}")
        print(f"\nData saved to: {args.output_dir}/")

    except Exception as e:
        print(f"\nError: {e}")
        return 1

    return 0


if __name__ == '__main__':
    sys.exit(main())
