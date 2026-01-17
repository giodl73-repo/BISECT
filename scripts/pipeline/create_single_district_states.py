#!/usr/bin/env python3
"""
Create placeholder structures for single-district (at-large) states.
These states have only 1 congressional district covering the entire state.
"""

import geopandas as gpd
import pandas as pd
from pathlib import Path
import matplotlib.pyplot as plt
import matplotlib.patheffects as path_effects

# Single-district states
SINGLE_DISTRICT_STATES = {
    'DE': 'Delaware',
    'SD': 'South Dakota',
    'ND': 'North Dakota',
    'AK': 'Alaska',
    'VT': 'Vermont',
    'WY': 'Wyoming'
}


def create_single_district_state(state_code, state_name, dpi=150):
    """Create all files for a single-district state."""

    print(f"{'='*70}")
    print(f"Creating single-district structure for {state_name}")
    print(f"{'='*70}")

    state_dir = Path(f'outputs/us_2020_redistricting/{state_name.lower().replace(" ", "_")}')
    state_dir.mkdir(parents=True, exist_ok=True)

    # Load tract data
    tracts_file = f'data/raw/{state_code.lower()}_tracts_2020.parquet'
    places_file = f'data/raw/{state_code.lower()}_places_2020.parquet'

    if not Path(tracts_file).exists():
        print(f"ERROR: Missing tract data for {state_name}")
        return False

    tracts = gpd.read_parquet(tracts_file)
    total_pop = int(tracts['population'].sum())

    print(f"  Population: {total_pop:,}")
    print(f"  Tracts: {len(tracts):,}")

    # Create district_cities.csv
    cities_data = {
        'district': [1],
        'largest_city': ['At-Large District'],
        'city_population': [0],
        'city_lon': [None],
        'city_lat': [None]
    }

    # Try to find actual largest city if places file exists
    if Path(places_file).exists():
        places = gpd.read_parquet(places_file)
        if len(places) > 0:
            largest = places.nlargest(1, 'population').iloc[0]
            cities_data['largest_city'] = [largest['NAME']]
            cities_data['city_population'] = [int(largest['population'])]
            centroid = largest.geometry.representative_point()
            cities_data['city_lon'] = [centroid.x]
            cities_data['city_lat'] = [centroid.y]

    cities_df = pd.DataFrame(cities_data)
    data_dir = state_dir / 'data'
    data_dir.mkdir(parents=True, exist_ok=True)
    cities_file = data_dir / 'district_cities.csv'
    cities_df.to_csv(cities_file, index=False)
    print(f"  [OK] Created: data/district_cities.csv")
    if cities_data['largest_city'][0] != 'At-Large District':
        print(f"    Largest city: {cities_data['largest_city'][0]} ({cities_data['city_population'][0]:,})")

    # Create district_summary.csv
    summary_data = {
        'district': [1],
        'population': [total_pop],
        'ideal_population': [total_pop],
        'deviation': [0],
        'deviation_pct': [0.0],
        'largest_city': [cities_data['largest_city'][0]]
    }

    summary_df = pd.DataFrame(summary_data)
    summary_file = data_dir / 'district_summary.csv'  # data_dir already created above
    summary_df.to_csv(summary_file, index=False)
    print(f"  [OK] Created: data/district_summary.csv")

    # Create simple map
    print(f"  Creating map...")
    tracts['district'] = 1

    fig, ax = plt.subplots(1, 1, figsize=(12, 10))

    tracts.plot(
        ax=ax,
        color='#1f77b4',
        edgecolor='white',
        linewidth=0.2,
        alpha=0.8
    )

    # Add "1" in center
    try:
        centroid = tracts.geometry.union_all().representative_point()
        text = ax.text(centroid.x, centroid.y, '1',
                fontsize=40, fontweight='bold', ha='center', va='center',
                color='white', zorder=10)
        text.set_path_effects([path_effects.Stroke(linewidth=3, foreground='black'),
                              path_effects.Normal()])
    except:
        pass

    ax.set_axis_off()
    ax.set_title(f'{state_name} Congressional District\n'
                 'At-Large (Single District)',
                 fontsize=16, fontweight='bold', pad=20)

    # Stats text box removed for cleaner visualization


    # Information available in CSV data files

    plt.tight_layout()

    map_file = state_dir / f'{state_name.lower().replace(" ", "_")}_1_district.png'
    plt.savefig(map_file, dpi=dpi, bbox_inches='tight')
    plt.close()
    print(f"  [OK] Created: {map_file.name}")

    # Create map with city label if we have a city
    if cities_data['largest_city'][0] != 'At-Large District' and cities_data['city_lon'][0]:
        fig, ax = plt.subplots(1, 1, figsize=(12, 10))

        tracts.plot(
            ax=ax,
            color='#1f77b4',
            edgecolor='white',
            linewidth=0.2,
            alpha=0.8
        )

        # Add city marker
        city_lon = cities_data['city_lon'][0]
        city_lat = cities_data['city_lat'][0]
        city_name = cities_data['largest_city'][0]

        ax.plot(city_lon, city_lat, 'o', color='red', markersize=12, zorder=5,
                markeredgecolor='white', markeredgewidth=2)

        # Add city label
        text = ax.text(city_lon, city_lat, f'  {city_name}',
                fontsize=12, fontweight='bold', ha='left', va='center',
                color='black', zorder=10,
                bbox=dict(boxstyle='round,pad=0.3', facecolor='white', edgecolor='black', alpha=0.8))

        ax.set_axis_off()
        ax.set_title(f'{state_name} Congressional District - At-Large\n'
                     f'Largest city: {city_name}',
                     fontsize=16, fontweight='bold', pad=20)

        plt.tight_layout()

        map_with_city_file = state_dir / f'{state_name.lower().replace(" ", "_")}_1_district_with_cities.png'
        plt.savefig(map_with_city_file, dpi=dpi, bbox_inches='tight')
        plt.close()
        print(f"  [OK] Created: {map_with_city_file.name}")

    print(f"[OK] {state_name} complete!")
    return True


def main():
    """Create structures for all single-district states."""
    import argparse

    parser = argparse.ArgumentParser(description='Create single-district state structures')
    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')
    args = parser.parse_args()

    print("\n" + "="*70)
    print("Creating Single-District State Structures")
    print("="*70)

    successful = []
    failed = []

    for state_code, state_name in sorted(SINGLE_DISTRICT_STATES.items()):
        if create_single_district_state(state_code, state_name, args.dpi):
            successful.append(state_name)
        else:
            failed.append(state_name)

    print("\n" + "="*70)
    print("SUMMARY")
    print("="*70)
    print(f"Successful: {len(successful)}/{len(SINGLE_DISTRICT_STATES)}")
    for s in successful:
        print(f"  [OK] {s}")

    if failed:
        print(f"\nFailed: {len(failed)}")
        for f in failed:
            print(f"  [FAIL] {f}")

    print("="*70)

    return 0 if not failed else 1


if __name__ == '__main__':
    import sys
    sys.exit(main())
