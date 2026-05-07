#!/usr/bin/env python3
"""
Compactness Comparison: MMDs vs Single-Member Districts

Computes Polsby-Popper compactness for MMD configurations and compares
to single-member baseline.

Polsby-Popper: (4 * π * area) / perimeter^2
- Perfect circle = 1.0
- Typical districts = 0.3-0.5
- Highly non-compact = <0.2

Usage:
    python scripts/experiments/mmd_compare_compactness.py --year 2020 --config uniform-5
    python scripts/experiments/mmd_compare_compactness.py --year 2020 --compare-all
"""

import argparse
import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))


# ============================================================================
# Compactness Metrics
# ============================================================================

def compute_polsby_popper(geometry) -> float:
    """
    Compute Polsby-Popper compactness score.

    PP = (4 * π * area) / perimeter^2

    Returns:
        Score between 0 and 1 (1 = perfect circle)
    """

    area = geometry.area
    perimeter = geometry.length  # For polygons, length = perimeter

    if perimeter == 0:
        return 0.0

    pp = (4 * np.pi * area) / (perimeter ** 2)

    # Clamp to [0, 1] (numerical errors can cause values > 1)
    return min(pp, 1.0)


def compute_reock(geometry) -> float:
    """
    Compute Reock compactness score (alternative metric).

    Reock = area / area_of_minimum_bounding_circle

    Returns:
        Score between 0 and 1 (1 = district fills its bounding circle)
    """

    from shapely.geometry import Point

    # Get minimum bounding circle (approximate)
    centroid = geometry.centroid
    max_dist = max(geometry.exterior.distance(Point(centroid.x, centroid.y))
                   for poly in (geometry.geoms if hasattr(geometry, 'geoms') else [geometry]))

    circle_area = np.pi * (max_dist ** 2)

    if circle_area == 0:
        return 0.0

    return geometry.area / circle_area


def interpret_polsby_popper(pp: float) -> str:
    """Provide interpretation of Polsby-Popper score."""

    if pp >= 0.6:
        return "Highly compact"
    elif pp >= 0.5:
        return "Very compact"
    elif pp >= 0.4:
        return "Compact"
    elif pp >= 0.3:
        return "Moderately compact"
    elif pp >= 0.2:
        return "Somewhat compact"
    else:
        return "Not compact"


# ============================================================================
# Load and Compute
# ============================================================================

def load_mmd_geometries(config_dir: Path) -> gpd.GeoDataFrame:
    """Load district geometries from MMD configuration."""

    geom_path = config_dir / "district_geometries.geojson"

    if not geom_path.exists():
        raise FileNotFoundError(
            f"District geometries not found: {geom_path}\n"
            f"Run mmd_generate_districts.py first"
        )

    return gpd.read_file(geom_path)


def load_single_member_baseline(year: int) -> gpd.GeoDataFrame:
    """Load single-member district geometries for comparison."""

    baseline_path = Path(f"outputs/v1/{year}/districts/district_geometries.geojson")

    if not baseline_path.exists():
        raise FileNotFoundError(
            f"Single-member baseline not found: {baseline_path}\n"
            f"Run single-member redistricting first"
        )

    return gpd.read_file(baseline_path)


def compute_compactness_metrics(districts: gpd.GeoDataFrame) -> pd.DataFrame:
    """Compute compactness metrics for all districts."""

    print(f"Computing compactness for {len(districts)} districts...")

    results = []

    for idx, row in districts.iterrows():
        geometry = row['geometry']

        pp = compute_polsby_popper(geometry)
        # reock = compute_reock(geometry)  # Optional, slower

        results.append({
            'district': idx,
            'polsby_popper': pp,
            # 'reock': reock,
            'area': geometry.area,
            'perimeter': geometry.length,
            'population': row.get('population', None),
            'members': row.get('members', None)
        })

    return pd.DataFrame(results)


def compare_compactness(
    mmd_compactness: pd.DataFrame,
    baseline_compactness: pd.DataFrame,
    config_name: str
) -> pd.DataFrame:
    """Compare MMD compactness to single-member baseline."""

    # Summary statistics
    mmd_stats = {
        'config_name': config_name,
        'system': 'MMD',
        'num_districts': len(mmd_compactness),
        'mean_pp': mmd_compactness['polsby_popper'].mean(),
        'median_pp': mmd_compactness['polsby_popper'].median(),
        'std_pp': mmd_compactness['polsby_popper'].std(),
        'min_pp': mmd_compactness['polsby_popper'].min(),
        'max_pp': mmd_compactness['polsby_popper'].max(),
        'pct_pp_above_0.4': (mmd_compactness['polsby_popper'] >= 0.4).mean() * 100,
        'pct_pp_above_0.3': (mmd_compactness['polsby_popper'] >= 0.3).mean() * 100,
    }

    baseline_stats = {
        'config_name': 'single-member',
        'system': 'Baseline',
        'num_districts': len(baseline_compactness),
        'mean_pp': baseline_compactness['polsby_popper'].mean(),
        'median_pp': baseline_compactness['polsby_popper'].median(),
        'std_pp': baseline_compactness['polsby_popper'].std(),
        'min_pp': baseline_compactness['polsby_popper'].min(),
        'max_pp': baseline_compactness['polsby_popper'].max(),
        'pct_pp_above_0.4': (baseline_compactness['polsby_popper'] >= 0.4).mean() * 100,
        'pct_pp_above_0.3': (baseline_compactness['polsby_popper'] >= 0.3).mean() * 100,
    }

    comparison = pd.DataFrame([baseline_stats, mmd_stats])

    # Compute change
    comparison['mean_pp_change'] = comparison['mean_pp'] - baseline_stats['mean_pp']
    comparison['mean_pp_change_pct'] = (comparison['mean_pp_change'] / baseline_stats['mean_pp']) * 100

    return comparison


# ============================================================================
# Multi-Configuration Comparison
# ============================================================================

def compare_all_configurations(base_dir: Path, year: int) -> pd.DataFrame:
    """Compare compactness across all MMD configurations."""

    print("\n[Comparing All Configurations]")

    # Load single-member baseline
    print("Loading single-member baseline...")
    try:
        baseline_districts = load_single_member_baseline(year)
        baseline_compactness = compute_compactness_metrics(baseline_districts)
        baseline_mean_pp = baseline_compactness['polsby_popper'].mean()
    except FileNotFoundError as e:
        print(f"Warning: Could not load baseline: {e}")
        baseline_mean_pp = None

    # Find all configuration directories
    config_dirs = [d for d in base_dir.iterdir() if d.is_dir()]

    if len(config_dirs) == 0:
        raise ValueError(f"No configuration directories found in {base_dir}")

    print(f"Found {len(config_dirs)} configurations")

    results = []

    for config_dir in config_dirs:
        config_name = config_dir.name

        try:
            # Load MMD geometries
            mmd_districts = load_mmd_geometries(config_dir)
            mmd_compactness = compute_compactness_metrics(mmd_districts)

            # Compute stats
            stats = {
                'config_name': config_name,
                'year': year,
                'num_districts': len(mmd_compactness),
                'mean_pp': mmd_compactness['polsby_popper'].mean(),
                'median_pp': mmd_compactness['polsby_popper'].median(),
                'std_pp': mmd_compactness['polsby_popper'].std(),
                'min_pp': mmd_compactness['polsby_popper'].min(),
                'max_pp': mmd_compactness['polsby_popper'].max(),
                'interpretation': interpret_polsby_popper(mmd_compactness['polsby_popper'].mean())
            }

            # Add comparison to baseline
            if baseline_mean_pp:
                stats['baseline_mean_pp'] = baseline_mean_pp
                stats['pp_change'] = stats['mean_pp'] - baseline_mean_pp
                stats['pp_change_pct'] = (stats['pp_change'] / baseline_mean_pp) * 100

            # Parse configuration
            if 'uniform' in config_name:
                members = int(config_name.split('-')[1])
                stats['system_type'] = 'uniform'
                stats['members'] = members
            else:
                stats['system_type'] = config_name.split('__')[0]
                stats['members'] = 'adaptive'

            results.append(stats)

        except FileNotFoundError as e:
            print(f"  Skipping {config_name}: {e}")
            continue

    if len(results) == 0:
        raise ValueError("No valid configurations found")

    results_df = pd.DataFrame(results)

    # Sort by mean PP (descending - higher is better)
    results_df = results_df.sort_values('mean_pp', ascending=False)

    return results_df


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description="Compare compactness across MMD configurations")
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020])
    parser.add_argument('--config', type=str,
                        help='Single configuration to analyze (default: analyze all)')
    parser.add_argument('--compare-all', action='store_true',
                        help='Compare all configurations')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/mmd'))
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/mmd'))

    args = parser.parse_args()

    print("="*70)
    print("COMPACTNESS COMPARISON")
    print("="*70)
    print(f"Year: {args.year}")

    if args.compare_all or not args.config:
        # Compare all configurations
        comparison_df = compare_all_configurations(args.input_dir, args.year)

        print("\n[Compactness Comparison]")

        display_cols = ['config_name', 'num_districts', 'mean_pp', 'median_pp',
                        'pp_change_pct', 'interpretation']
        display_cols = [c for c in display_cols if c in comparison_df.columns]

        print(comparison_df[display_cols].to_string(index=False))

        # Identify most compact configuration
        best_config = comparison_df.iloc[0]
        print(f"\n[Most Compact Configuration]")
        print(f"  Config: {best_config['config_name']}")
        print(f"  Mean PP: {best_config['mean_pp']:.4f}")
        if 'pp_change_pct' in best_config:
            print(f"  Change from baseline: {best_config['pp_change_pct']:+.2f}%")

        # Save comparison
        output_path = args.output_dir / f"compactness_comparison_{args.year}.csv"
        comparison_df.to_csv(output_path, index=False)
        print(f"\nSaved: {output_path}")

    else:
        # Single configuration analysis
        config_dir = args.input_dir / args.config

        print(f"Configuration: {args.config}")

        # Load and compute
        mmd_districts = load_mmd_geometries(config_dir)
        mmd_compactness = compute_compactness_metrics(mmd_districts)

        print(f"\n[Compactness Statistics]")
        print(f"Districts: {len(mmd_compactness)}")
        print(f"Mean PP: {mmd_compactness['polsby_popper'].mean():.4f}")
        print(f"Median PP: {mmd_compactness['polsby_popper'].median():.4f}")
        print(f"Std PP: {mmd_compactness['polsby_popper'].std():.4f}")
        print(f"Range: [{mmd_compactness['polsby_popper'].min():.4f}, {mmd_compactness['polsby_popper'].max():.4f}]")

        # Distribution
        print(f"\nDistribution:")
        print(f"  PP >= 0.5: {(mmd_compactness['polsby_popper'] >= 0.5).sum()} ({(mmd_compactness['polsby_popper'] >= 0.5).mean()*100:.1f}%)")
        print(f"  PP >= 0.4: {(mmd_compactness['polsby_popper'] >= 0.4).sum()} ({(mmd_compactness['polsby_popper'] >= 0.4).mean()*100:.1f}%)")
        print(f"  PP >= 0.3: {(mmd_compactness['polsby_popper'] >= 0.3).sum()} ({(mmd_compactness['polsby_popper'] >= 0.3).mean()*100:.1f}%)")

        # Compare to baseline
        try:
            baseline_districts = load_single_member_baseline(args.year)
            baseline_compactness = compute_compactness_metrics(baseline_districts)

            comparison = compare_compactness(mmd_compactness, baseline_compactness, args.config)

            print(f"\n[Comparison to Single-Member Baseline]")
            display_cols = ['system', 'num_districts', 'mean_pp', 'median_pp', 'mean_pp_change_pct']
            print(comparison[display_cols].to_string(index=False))

            # Save comparison
            output_path = config_dir / "compactness_comparison.csv"
            comparison.to_csv(output_path, index=False)
            print(f"\nSaved: {output_path}")

        except FileNotFoundError:
            print(f"\nWarning: Could not load single-member baseline for comparison")

        # Save district-level compactness
        district_output_path = config_dir / "district_compactness.csv"
        mmd_compactness.to_csv(district_output_path, index=False)
        print(f"District-level compactness: {district_output_path}")

    print(f"\n[SUCCESS] Compactness comparison complete")


if __name__ == '__main__':
    main()
