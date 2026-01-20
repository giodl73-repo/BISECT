#!/usr/bin/env python3
"""Test script to check census data completion."""

from pathlib import Path
import sys

# Add scripts/data to path
sys.path.insert(0, str(Path('scripts/data')))

from process_census_data import check_all_stages_complete

# Check each year
for year in ['2000', '2010', '2020']:
    output_dir = Path(f'outputs/V20/{year}')
    required_stages = ['tracts', 'adjacency']
    resolution = 'tract'

    complete = check_all_stages_complete(output_dir, required_stages, resolution, False)
    print(f"{year}: census_complete={complete}")

    # Check individual markers
    for stage in required_stages:
        marker = output_dir / f'.{resolution}_{stage}_complete'
        exists = marker.exists()
        print(f"  - {marker.name}: {exists}")
