"""
Analyze compactness of national redistricting vs state-based baseline.

Computes Polsby-Popper scores for all 435 districts and compares to
state-based redistricting from Paper #01.

Usage:
    python scripts/experimental/analyze_national_compactness.py --year 2020

Output:
    outputs/experimental/compactness_comparison_2020.pkl
"""

import argparse
import pickle
from pathlib import Path
from typing import Dict, List

import geopandas as gpd
import numpy as np
import pandas as pd
from tqdm import tqdm

# Add src and scripts to path
import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))
sys.path.insert(0, str(project_root / "scripts"))

from config.download_sources import STATE_FIPS, STATE_NAMES


def load_national_districts(input_file: Path) -> Dict:
    """Load national district assignments."""
    print(f"\nLoading national districts from {input_file}...")

    with open(input_file, 'rb') as f:
        data = pickle.load(f)

    print(f"  Loaded {data['n_districts']} districts")
    print(f"  {data['n_cross_state_districts']} cross state lines")

    return data


def load_tract_geometries(year: int, data_dir: Path, geoid_to_index: Dict[str, int]) -> gpd.GeoDataFrame:
    """Load tract geometries for all states."""
    print(f"\nLoading tract geometries ({year})...")

    all_tracts = []

    for state_abbr, fips in tqdm(STATE_FIPS.items(), desc="Loading geometries"):
        state_name = STATE_NAMES[state_abbr]

        # Try GPKG first, then shapefile
        tract_file_gpkg = data_dir / f"{year}/tiger/tracts/{state_name}/tracts_{state_name}_{year}.gpkg"
        tract_file_shp = data_dir / f"{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp"

        if tract_file_gpkg.exists():
            tract_file = tract_file_gpkg
        elif tract_file_shp.exists():
            tract_file = tract_file_shp
        else:
            continue

        gdf = gpd.read_file(tract_file)

        # Only keep tracts that are in our graph
        gdf = gdf[gdf['GEOID'].isin(geoid_to_index.keys())]

        all_tracts.append(gdf[['GEOID', 'geometry']])

    # Concatenate all states
    national_gdf = gpd.GeoDataFrame(
        pd.concat(all_tracts, ignore_index=True),
        crs=all_tracts[0].crs
    )

    print(f"  Loaded {len(national_gdf):,} tract geometries")

    return national_gdf


def compute_district_geometries(
    tracts_gdf: gpd.GeoDataFrame,
    geoid_to_district: Dict[str, int],
    n_districts: int
) -> gpd.GeoDataFrame:
    """
    Compute district geometries by dissolving tract geometries.

    Returns:
        GeoDataFrame with district geometries and IDs
    """
    print(f"\nComputing district geometries ({n_districts} districts)...")

    # Add district assignments to tracts
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['district'] = tracts_gdf['GEOID'].map(geoid_to_district)

    # Remove tracts without district assignment
    tracts_gdf = tracts_gdf[tracts_gdf['district'].notna()]

    print(f"  Dissolving {len(tracts_gdf):,} tracts into {n_districts} districts...")

    # Dissolve tracts by district
    districts_gdf = tracts_gdf.dissolve(by='district', as_index=False)

    print(f"  Created {len(districts_gdf)} district polygons")

    return districts_gdf


def compute_polsby_popper(geometry) -> float:
    """
    Compute Polsby-Popper compactness score.

    PP = 4π × Area / Perimeter²

    Score ranges from 0 (least compact) to 1 (perfect circle).
    """
    area = geometry.area
    perimeter = geometry.length

    if perimeter == 0:
        return 0.0

    pp = (4 * np.pi * area) / (perimeter ** 2)
    return min(pp, 1.0)  # Cap at 1.0 for numerical stability


def compute_compactness_scores(districts_gdf: gpd.GeoDataFrame) -> Dict:
    """
    Compute Polsby-Popper scores for all districts.

    Returns:
        Dictionary with scores and statistics
    """
    print(f"\nComputing Polsby-Popper compactness scores...")

    # Project to equal-area projection for accurate area/perimeter
    districts_proj = districts_gdf.to_crs('ESRI:102003')  # USA Contiguous Albers Equal Area Conic

    scores = []
    for idx, row in tqdm(districts_proj.iterrows(), total=len(districts_proj), desc="Computing PP scores"):
        pp = compute_polsby_popper(row.geometry)
        scores.append({
            'district': int(row['district']),
            'pp_score': pp,
            'area_sqm': row.geometry.area,
            'perimeter_m': row.geometry.length
        })

    scores_df = pd.DataFrame(scores)

    # Compute statistics
    pp_scores = scores_df['pp_score'].values

    stats = {
        'mean': float(pp_scores.mean()),
        'median': float(np.median(pp_scores)),
        'std': float(pp_scores.std()),
        'min': float(pp_scores.min()),
        'max': float(pp_scores.max()),
        'q25': float(np.percentile(pp_scores, 25)),
        'q75': float(np.percentile(pp_scores, 75)),
        'scores': scores_df.to_dict('records')
    }

    print(f"\n  Compactness Statistics:")
    print(f"    Mean PP: {stats['mean']:.3f}")
    print(f"    Median PP: {stats['median']:.3f}")
    print(f"    Std Dev: {stats['std']:.3f}")
    print(f"    Range: {stats['min']:.3f} - {stats['max']:.3f}")

    return stats


def load_state_baseline(year: int = 2020) -> Dict:
    """
    Load state-based compactness baseline from Paper #01.

    For now, use the known baseline values from the plan.
    In a full implementation, this would load from Paper #01 results.
    """
    # From plan.md - Paper #01 baseline
    baseline = {
        'mean': 0.461,
        'median': 0.456,
        'std': 0.083,
        'min': 0.18,
        'max': 0.82
    }

    print(f"\nState-based baseline (Paper #01, {year}):")
    print(f"  Mean PP: {baseline['mean']:.3f}")
    print(f"  Median PP: {baseline['median']:.3f}")

    return baseline


def compare_to_baseline(national_stats: Dict, baseline: Dict) -> Dict:
    """Compare national optimization to state-based baseline."""
    print(f"\nComparing to state-based baseline...")

    comparison = {
        'national': national_stats,
        'state_based': baseline,
        'improvement_mean': ((national_stats['mean'] - baseline['mean']) / baseline['mean']) * 100,
        'improvement_median': ((national_stats['median'] - baseline['median']) / baseline['median']) * 100,
    }

    print(f"\n  Improvement over state-based redistricting:")
    print(f"    Mean PP: {comparison['improvement_mean']:+.1f}%")
    print(f"    Median PP: {comparison['improvement_median']:+.1f}%")

    if comparison['improvement_mean'] > 0:
        print(f"    -> National optimization produces MORE compact districts")
    elif comparison['improvement_mean'] < 0:
        print(f"    -> National optimization produces LESS compact districts")
    else:
        print(f"    -> No significant difference")

    return comparison


def save_results(output_file: Path, comparison: Dict, districts_gdf: gpd.GeoDataFrame = None) -> None:
    """Save compactness analysis results."""
    print(f"\nSaving results to {output_file}...")

    data = {
        'comparison': comparison,
        'national_scores': comparison['national']['scores'],
        'state_baseline': comparison['state_based'],
        'improvement_mean_pct': comparison['improvement_mean'],
        'improvement_median_pct': comparison['improvement_median']
    }

    # Optionally save district geometries
    if districts_gdf is not None:
        # Save as separate GeoPackage (smaller than pickle)
        gpkg_file = output_file.parent / output_file.name.replace('.pkl', '.gpkg')
        districts_gdf.to_file(gpkg_file, driver='GPKG')
        print(f"  District geometries saved to {gpkg_file}")

    with open(output_file, 'wb') as f:
        pickle.dump(data, f)

    print(f"  [OK] Saved compactness analysis")


def main():
    parser = argparse.ArgumentParser(description="Analyze national redistricting compactness")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/experimental'), help='Output directory')
    args = parser.parse_args()

    print("="*70)
    print("National Redistricting Compactness Analysis (Paper #13)")
    print("="*70)

    # Load national districts
    districts_file = args.input_dir / f"national_districts_{args.year}.pkl"
    district_data = load_national_districts(districts_file)

    # Load tract geometries
    tracts_gdf = load_tract_geometries(args.year, args.data_dir, district_data['geoid_to_index'])

    # Compute district geometries
    districts_gdf = compute_district_geometries(
        tracts_gdf,
        district_data['geoid_to_district'],
        district_data['n_districts']
    )

    # Compute compactness scores
    national_stats = compute_compactness_scores(districts_gdf)

    # Load state-based baseline
    baseline = load_state_baseline(args.year)

    # Compare to baseline
    comparison = compare_to_baseline(national_stats, baseline)

    # Save results
    output_file = args.output_dir / f"compactness_comparison_{args.year}.pkl"
    save_results(output_file, comparison, districts_gdf)

    print("\n" + "="*70)
    print("Compactness analysis complete!")
    print("="*70)
    print(f"Output: {output_file}")
    print(f"\nKey finding: National optimization produces {comparison['improvement_mean']:+.1f}% change in mean compactness")


if __name__ == '__main__':
    main()
