"""
Data I/O utilities for saving and loading redistricting results.
"""

import json
from pathlib import Path
from typing import Dict

import geopandas as gpd
import pandas as pd


def save_results(
    assignments: Dict[str, int],
    blocks_gdf: gpd.GeoDataFrame,
    output_dir: str,
    state: str,
    num_districts: int
):
    """
    Save redistricting results to disk.

    Creates:
    - assignments.csv: Block-to-district mapping
    - districts.geojson: Dissolved district boundaries
    - metadata.json: Population statistics

    Parameters
    ----------
    assignments : Dict[str, int]
        Mapping from GEOID to district_id
    blocks_gdf : gpd.GeoDataFrame
        Block geometries with population
    output_dir : str
        Output directory
    state : str
        State code
    num_districts : int
        Number of districts
    """
    output_dir = Path(output_dir)
    result_dir = output_dir / f"{state.lower()}_{num_districts}_districts"
    result_dir.mkdir(parents=True, exist_ok=True)

    print(f"\nSaving results to {result_dir}...")

    # 1. Save assignments CSV
    assignments_df = pd.DataFrame([
        {'GEOID': geoid, 'district_id': district_id}
        for geoid, district_id in assignments.items()
    ])
    assignments_path = result_dir / 'assignments.csv'
    assignments_df.to_csv(assignments_path, index=False)
    print(f"  Saved assignments: {assignments_path}")

    # 2. Create district boundaries (dissolve blocks by district)
    print("  Creating district boundaries...")
    blocks_with_districts = blocks_gdf.merge(
        assignments_df,
        on='GEOID',
        how='inner'
    )

    districts_gdf = blocks_with_districts.dissolve(
        by='district_id',
        aggfunc={
            'population': 'sum',
            'ALAND': 'sum',
            'AWATER': 'sum'
        }
    ).reset_index()

    # Add district statistics
    districts_gdf['total_area'] = districts_gdf['ALAND'] + districts_gdf['AWATER']
    districts_gdf['land_area_km2'] = districts_gdf['ALAND'] / 1e6
    districts_gdf['density_per_km2'] = districts_gdf['population'] / districts_gdf['land_area_km2']

    # Compute compactness (Polsby-Popper score)
    # PP = 4π * area / perimeter²
    # Score of 1 = perfect circle, closer to 0 = less compact
    districts_projected = districts_gdf.to_crs('EPSG:3310')  # CA Albers for area calc
    districts_gdf['polsby_popper'] = (
        4 * 3.14159 * districts_projected.geometry.area /
        (districts_projected.geometry.length ** 2)
    )

    # Save as GeoJSON
    districts_path = result_dir / 'districts.geojson'
    districts_gdf.to_file(districts_path, driver='GeoJSON')
    print(f"  Saved districts: {districts_path}")

    # 3. Save metadata
    ideal_population = blocks_with_districts['population'].sum() / num_districts

    district_stats = []
    for _, row in districts_gdf.iterrows():
        deviation = (row['population'] - ideal_population) / ideal_population
        district_stats.append({
            'district_id': int(row['district_id']),
            'population': int(row['population']),
            'population_deviation_pct': round(deviation * 100, 2),
            'land_area_km2': round(row['land_area_km2'], 2),
            'density_per_km2': round(row['density_per_km2'], 1),
            'polsby_popper_score': round(row['polsby_popper'], 4)
        })

    metadata = {
        'state': state,
        'num_districts': num_districts,
        'total_population': int(blocks_with_districts['population'].sum()),
        'ideal_district_population': int(ideal_population),
        'districts': district_stats,
        'summary': {
            'mean_population_deviation_pct': round(
                abs(pd.Series([d['population_deviation_pct'] for d in district_stats])).mean(), 2
            ),
            'max_population_deviation_pct': round(
                abs(pd.Series([d['population_deviation_pct'] for d in district_stats])).max(), 2
            ),
            'mean_polsby_popper': round(districts_gdf['polsby_popper'].mean(), 4),
        }
    }

    metadata_path = result_dir / 'metadata.json'
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    print(f"  Saved metadata: {metadata_path}")

    print(f"\nResults saved to: {result_dir}")
    print(f"\nSummary Statistics:")
    print(f"  Mean population deviation: ±{metadata['summary']['mean_population_deviation_pct']:.2f}%")
    print(f"  Max population deviation: ±{metadata['summary']['max_population_deviation_pct']:.2f}%")
    print(f"  Mean compactness (Polsby-Popper): {metadata['summary']['mean_polsby_popper']:.4f}")


def load_results(result_dir: str) -> tuple:
    """
    Load redistricting results from disk.

    Parameters
    ----------
    result_dir : str
        Directory containing results

    Returns
    -------
    assignments : pd.DataFrame
        Block-to-district assignments
    districts : gpd.GeoDataFrame
        District boundaries
    metadata : dict
        Metadata and statistics
    """
    result_dir = Path(result_dir)

    print(f"Loading results from {result_dir}...")

    assignments = pd.read_csv(result_dir / 'assignments.csv')
    districts = gpd.read_file(result_dir / 'districts.geojson')

    with open(result_dir / 'metadata.json', 'r') as f:
        metadata = json.load(f)

    print(f"Loaded {len(assignments):,} block assignments")
    print(f"Loaded {len(districts)} districts")

    return assignments, districts, metadata
