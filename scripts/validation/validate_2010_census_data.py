#!/usr/bin/env python3
"""
Validate 2010 Census tract data against official Census Bureau totals.

Checks that downloaded tract data has correct population totals.
"""

import pandas as pd
import geopandas as gpd
from pathlib import Path
from typing import Dict

# Official 2010 Census state populations from Census Bureau
# Source: https://www.netstate.com/states/tables/state_population_2010.htm
OFFICIAL_2010_POPULATIONS = {
    'AL': 4_779_736,
    'AK': 710_231,
    'AZ': 6_392_017,
    'AR': 2_915_918,
    'CA': 37_253_956,
    'CO': 5_029_196,
    'CT': 3_574_097,
    'DE': 897_934,
    'FL': 18_801_310,
    'GA': 9_687_653,
    'HI': 1_360_301,
    'ID': 1_567_582,
    'IL': 12_830_632,
    'IN': 6_483_802,
    'IA': 3_046_355,
    'KS': 2_853_118,
    'KY': 4_339_367,
    'LA': 4_533_372,
    'ME': 1_328_361,
    'MD': 5_773_552,
    'MA': 6_547_629,
    'MI': 9_883_640,
    'MN': 5_303_925,
    'MS': 2_967_297,
    'MO': 5_988_927,
    'MT': 989_415,
    'NE': 1_826_341,
    'NV': 2_700_551,
    'NH': 1_316_470,
    'NJ': 8_791_894,
    'NM': 2_059_179,
    'NY': 19_378_102,
    'NC': 9_535_483,
    'ND': 672_591,
    'OH': 11_536_504,
    'OK': 3_751_351,
    'OR': 3_831_074,
    'PA': 12_702_379,
    'RI': 1_052_567,
    'SC': 4_625_364,
    'SD': 814_180,
    'TN': 6_346_105,
    'TX': 25_145_561,
    'UT': 2_763_885,
    'VT': 625_741,
    'VA': 8_001_024,
    'WA': 6_724_540,
    'WV': 1_852_994,
    'WI': 5_686_986,
    'WY': 563_626,
}


def validate_2010_data(data_dir: Path = None) -> pd.DataFrame:
    """
    Validate 2010 census tract data against official totals.

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

    for state_code in sorted(OFFICIAL_2010_POPULATIONS.keys()):
        tract_file = data_dir / f'{state_code.lower()}_tracts_2010.parquet'
        official_pop = OFFICIAL_2010_POPULATIONS[state_code]

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
    print("2010 Census Data Validation Report")
    print("=" * 90)
    print()

    # Summary statistics
    total_official = df['official_pop'].sum()
    total_downloaded = df['downloaded_pop'].sum()
    total_diff = total_downloaded - total_official
    overall_percent = (total_downloaded / total_official * 100) if total_official > 0 else 0

    print(f"Overall Statistics:")
    print(f"  Official 2010 US Population:   {total_official:>15,}")
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
    df = validate_2010_data()
    print_validation_report(df)

    # Save detailed results
    output_file = Path('data/processed/2010_validation_results.csv')
    output_file.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(output_file, index=False)
    print(f"\nDetailed results saved to: {output_file}")


if __name__ == '__main__':
    main()
