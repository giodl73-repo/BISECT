#!/usr/bin/env python3
"""
Validate 2020 Census tract data against official Census Bureau totals.

Checks that downloaded tract data has correct population totals.
"""

import pandas as pd
import geopandas as gpd
from pathlib import Path
from typing import Dict

# Official 2020 Census state populations from Census Bureau
# Source: https://state.1keydata.com/state-population.php
# (Based on official Census data)
OFFICIAL_2020_POPULATIONS = {
    'AL': 5_024_279,
    'AK': 733_391,
    'AZ': 7_151_502,
    'AR': 3_011_524,
    'CA': 39_538_223,
    'CO': 5_773_714,
    'CT': 3_605_944,
    'DE': 989_948,
    'FL': 21_538_187,
    'GA': 10_711_908,
    'HI': 1_455_271,
    'ID': 1_839_106,
    'IL': 12_812_508,
    'IN': 6_785_528,
    'IA': 3_190_369,
    'KS': 2_937_880,
    'KY': 4_505_836,
    'LA': 4_657_757,
    'ME': 1_362_359,
    'MD': 6_177_224,
    'MA': 7_029_917,
    'MI': 10_077_331,
    'MN': 5_706_494,
    'MS': 2_961_279,
    'MO': 6_154_913,
    'MT': 1_084_225,
    'NE': 1_961_504,
    'NV': 3_104_614,
    'NH': 1_377_529,
    'NJ': 9_288_994,
    'NM': 2_117_522,
    'NY': 20_201_249,
    'NC': 10_439_388,
    'ND': 779_094,
    'OH': 11_799_448,
    'OK': 3_959_353,
    'OR': 4_237_256,
    'PA': 13_002_700,
    'RI': 1_097_379,
    'SC': 5_118_425,
    'SD': 886_667,
    'TN': 6_910_840,
    'TX': 29_145_505,
    'UT': 3_271_616,
    'VT': 643_077,
    'VA': 8_631_393,
    'WA': 7_705_281,
    'WV': 1_793_716,
    'WI': 5_893_718,
    'WY': 576_851,
}


def validate_2020_data(data_dir: Path = None) -> pd.DataFrame:
    """
    Validate 2020 census tract data against official totals.

    Parameters
    ----------
    data_dir : Path, optional
        Directory containing tract parquet files (default: data/raw)

    Returns
    -------
    pd.DataFrame
        Validation results with columns:
        - state_code
        - official_pop
        - downloaded_pop
        - num_tracts
        - difference
        - percent_complete
        - status
    """
    if data_dir is None:
        data_dir = Path('data/raw')

    results = []

    for state_code in sorted(OFFICIAL_2020_POPULATIONS.keys()):
        tract_file = data_dir / f'{state_code.lower()}_tracts_2020.parquet'
        official_pop = OFFICIAL_2020_POPULATIONS[state_code]

        if not tract_file.exists():
            results.append({
                'state_code': state_code,
                'official_pop': official_pop,
                'downloaded_pop': 0,
                'num_tracts': 0,
                'difference': -official_pop,
                'percent_complete': 0.0,
                'status': 'MISSING'
            })
            continue

        # Read tract data
        tracts = gpd.read_parquet(tract_file)
        downloaded_pop = int(tracts['population'].sum())
        num_tracts = len(tracts)

        difference = downloaded_pop - official_pop
        percent_complete = (downloaded_pop / official_pop * 100) if official_pop > 0 else 0

        # Determine status
        if abs(difference) <= 100:  # Allow for rounding
            status = 'OK'
        elif percent_complete >= 99.0:
            status = 'MINOR'
        elif percent_complete >= 95.0:
            status = 'INCOMPLETE'
        else:
            status = 'MAJOR ISSUE'

        results.append({
            'state_code': state_code,
            'official_pop': official_pop,
            'downloaded_pop': downloaded_pop,
            'num_tracts': num_tracts,
            'difference': difference,
            'percent_complete': percent_complete,
            'status': status
        })

    df = pd.DataFrame(results)
    return df


def print_validation_report(df: pd.DataFrame):
    """Print formatted validation report."""
    print("\n" + "=" * 90)
    print("2020 Census Data Validation Report")
    print("=" * 90)
    print()

    # Summary statistics
    total_official = df['official_pop'].sum()
    total_downloaded = df['downloaded_pop'].sum()
    total_diff = total_downloaded - total_official
    overall_percent = (total_downloaded / total_official * 100) if total_official > 0 else 0

    print(f"Overall Statistics:")
    print(f"  Official 2020 US Population:   {total_official:>15,}")
    print(f"  Downloaded Population:         {total_downloaded:>15,}")
    print(f"  Difference:                    {total_diff:>15,} ({overall_percent:.2f}%)")
    print()

    # Count states by status
    status_counts = df['status'].value_counts()
    print(f"States by Status:")
    for status in ['OK', 'MINOR', 'INCOMPLETE', 'MAJOR ISSUE', 'MISSING']:
        count = status_counts.get(status, 0)
        if count > 0:
            print(f"  {status:15s}: {count:>2} states")
    print()

    # Show problematic states
    problem_states = df[df['status'].isin(['INCOMPLETE', 'MAJOR ISSUE', 'MISSING'])]
    if not problem_states.empty:
        print("=" * 90)
        print("PROBLEMATIC STATES (Need Attention)")
        print("=" * 90)
        print()
        print(f"{'State':<6} {'Official Pop':>15} {'Downloaded':>15} {'Tracts':>8} {'%Complete':>10} {'Status':<15}")
        print("-" * 90)

        for _, row in problem_states.iterrows():
            print(f"{row['state_code']:<6} "
                  f"{row['official_pop']:>15,} "
                  f"{row['downloaded_pop']:>15,} "
                  f"{row['num_tracts']:>8,} "
                  f"{row['percent_complete']:>9.2f}% "
                  f"{row['status']:<15}")
        print()

    # Show minor issues
    minor_states = df[df['status'] == 'MINOR']
    if not minor_states.empty:
        print("=" * 90)
        print("MINOR DISCREPANCIES (99%+ complete, likely rounding)")
        print("=" * 90)
        print()
        print(f"{'State':<6} {'Official Pop':>15} {'Downloaded':>15} {'Difference':>12}")
        print("-" * 90)

        for _, row in minor_states.iterrows():
            print(f"{row['state_code']:<6} "
                  f"{row['official_pop']:>15,} "
                  f"{row['downloaded_pop']:>15,} "
                  f"{row['difference']:>12,}")
        print()

    # Count OK states
    ok_count = len(df[df['status'] == 'OK'])
    print("=" * 90)
    print(f"VALIDATED: {ok_count} of 50 states have correct population data")
    print("=" * 90)


def main():
    """Run validation and print report."""
    df = validate_2020_data()
    print_validation_report(df)

    # Save detailed results
    output_file = Path('data/processed/2020_validation_results.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\nDetailed results saved to: {output_file}")


if __name__ == '__main__':
    main()
