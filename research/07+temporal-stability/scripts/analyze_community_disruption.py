"""
Analyze communities of interest disruption using county-level changes.

Measures how many counties changed their district-split patterns between
2010 and 2020 for each partitioning method.
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
from collections import defaultdict

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']
METHODS = ['nway', 'recursive']


def load_partition_with_counties(state: str, year: int, method: str):
    """
    Load partition results and join with county information.

    Returns: DataFrame with GEOID, district, county_fips
    """
    # Load partition results
    results_dir = Path('research/gerry-temporal-stability/results')
    partition_file = results_dir / f'{state}_{year}_{method}_partition.csv'

    if not partition_file.exists():
        raise FileNotFoundError(f"Partition file not found: {partition_file}")

    partition_df = pd.read_csv(partition_file)

    # Load tract geometries to get county information
    tracts_dir = Path(f'data/{year}/tiger/tracts/{state}')
    tracts_gdf = gpd.read_file(tracts_dir)

    # Extract county FIPS (first 5 digits of GEOID)
    tracts_gdf['county_fips'] = tracts_gdf['GEOID'].str[:5]

    # Merge
    result = partition_df.merge(
        tracts_gdf[['GEOID', 'county_fips']],
        on='GEOID',
        how='left'
    )

    return result


def compute_county_splits(partition_df: pd.DataFrame):
    """
    Compute how many districts each county spans.

    Returns: Dict mapping county_fips -> set of districts
    """
    county_districts = defaultdict(set)

    for _, row in partition_df.iterrows():
        county = row['county_fips']
        district = row['district']
        county_districts[county].add(district)

    return county_districts


def analyze_county_disruption(state: str, method: str):
    """
    Analyze county disruption for a state and method.

    Returns: Dict with disruption metrics
    """
    print(f"  Analyzing county disruption: {state}, {method}")

    # Load 2010 and 2020 partitions with counties
    partition_2010 = load_partition_with_counties(state, 2010, method)
    partition_2020 = load_partition_with_counties(state, 2020, method)

    # Compute county splits for each year
    county_splits_2010 = compute_county_splits(partition_2010)
    county_splits_2020 = compute_county_splits(partition_2020)

    # Find counties present in both years
    common_counties = set(county_splits_2010.keys()) & set(county_splits_2020.keys())

    if len(common_counties) == 0:
        return None

    # Count counties with changed splits
    counties_changed = 0
    total_counties = len(common_counties)

    change_details = []

    for county in common_counties:
        splits_2010 = len(county_splits_2010[county])
        splits_2020 = len(county_splits_2020[county])

        if splits_2010 != splits_2020:
            counties_changed += 1
            change_details.append({
                'county_fips': county,
                'splits_2010': splits_2010,
                'splits_2020': splits_2020,
                'change': splits_2020 - splits_2010
            })

    disruption_rate = counties_changed / total_counties

    return {
        'state': state,
        'method': method,
        'total_counties': total_counties,
        'counties_changed': counties_changed,
        'disruption_rate': disruption_rate,
        'change_details': change_details
    }


def main():
    """Analyze county disruption for all states and methods."""
    print("="*70)
    print("COMMUNITIES OF INTEREST DISRUPTION ANALYSIS")
    print("="*70)
    print()

    all_results = []

    for state in STATES:
        print(f"{state.upper()}")
        print("-"*70)

        for method in METHODS:
            try:
                result = analyze_county_disruption(state, method)

                if result:
                    all_results.append({
                        'state': result['state'],
                        'method': result['method'],
                        'total_counties': result['total_counties'],
                        'counties_changed': result['counties_changed'],
                        'disruption_rate': result['disruption_rate']
                    })

                    print(f"    {method:12s}: "
                          f"{result['counties_changed']}/{result['total_counties']} counties changed "
                          f"({result['disruption_rate']:.1%})")

            except FileNotFoundError as e:
                print(f"    ERROR ({method}): {e}")
            except Exception as e:
                print(f"    ERROR ({method}): {e}")
                import traceback
                traceback.print_exc()

        print()

    # Save results
    if all_results:
        results_df = pd.DataFrame(all_results)
        output_file = Path('research/gerry-temporal-stability/results/county_disruption.csv')
        output_file.parent.mkdir(parents=True, exist_ok=True)
        results_df.to_csv(output_file, index=False)
        print(f"Saved: {output_file}")

        # Print summary
        print()
        print("="*70)
        print("SUMMARY")
        print("="*70)
        summary = results_df.groupby('method').agg({
            'disruption_rate': 'mean',
            'counties_changed': 'sum',
            'total_counties': 'sum'
        })
        print(summary)

        # Compute improvement
        nway_rate = summary.loc['nway', 'disruption_rate']
        recursive_rate = summary.loc['recursive', 'disruption_rate']
        improvement = nway_rate - recursive_rate

        print()
        print(f"Recursive bisection reduces county disruption by {improvement:.1%}")
        print(f"  N-way:     {nway_rate:.1%} counties disrupted")
        print(f"  Recursive: {recursive_rate:.1%} counties disrupted")


if __name__ == '__main__':
    main()
