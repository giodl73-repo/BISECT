#!/usr/bin/env python3
"""
MMD Ablation Study: District Size and Density Threshold Analysis

Tests all combinations of:
- District sizes: Uniform (3/5/7) and Adaptive (3-5, 5-7, 3-5-7, 3-7)
- Density thresholds: Very conservative to very liberal

Usage:
    python scripts/experiments/mmd_ablation_study.py --year 2020 --output ablation_results
"""

import argparse
import sys
from pathlib import Path
from typing import List, Dict, Tuple, Optional
import pandas as pd
import geopandas as gpd
import numpy as np
from itertools import product

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

from apportionment.partition.recursive_bisection import partition_graph
from apportionment.data.adjacency import load_adjacency_graph


# ============================================================================
# CONFIGURATION: Ablation parameters
# ============================================================================

UNIFORM_CONFIGS = [
    {'name': 'uniform-3', 'sizes': [3], 'description': 'All 3-member districts'},
    {'name': 'uniform-5', 'sizes': [5], 'description': 'All 5-member districts (BASELINE)'},
    {'name': 'uniform-7', 'sizes': [7], 'description': 'All 7-member districts'},
]

ADAPTIVE_CONFIGS = [
    {'name': 'adaptive-3-5', 'sizes': [3, 5], 'description': 'Two-tier: 3 rural, 5 urban'},
    {'name': 'adaptive-5-7', 'sizes': [5, 7], 'description': 'Two-tier: 5 suburban, 7 urban'},
    {'name': 'adaptive-3-5-7', 'sizes': [3, 5, 7], 'description': 'Three-tier: full adaptive'},
    {'name': 'adaptive-3-7', 'sizes': [3, 7], 'description': 'Two-tier: 3 rural, 7 urban (skip middle)'},
]

DENSITY_THRESHOLDS = [
    {
        'name': 'very-conservative',
        'urban': 5000,
        'suburban': 2000,
        'description': '5k+/2k-5k/<2k people per sq mi'
    },
    {
        'name': 'census-standard',
        'urban': 3000,
        'suburban': 1500,
        'description': '3k+/1.5k-3k/<1.5k (Census Bureau standard)'
    },
    {
        'name': 'liberal',
        'urban': 2000,
        'suburban': 1000,
        'description': '2k+/1k-2k/<1k people per sq mi'
    },
    {
        'name': 'very-liberal',
        'urban': 1500,
        'suburban': 800,
        'description': '1.5k+/0.8k-1.5k/<0.8k people per sq mi'
    },
]


# ============================================================================
# Helper Functions
# ============================================================================

def compute_tract_density(tracts: gpd.GeoDataFrame) -> pd.Series:
    """Compute population density (people per sq mi) for each tract."""
    # Convert area to square miles (assuming area is in sq meters from CRS)
    sq_meters_to_sq_miles = 2.59e6
    tracts['area_sq_mi'] = tracts.geometry.area / sq_meters_to_sq_miles
    tracts['density'] = tracts['population'] / tracts['area_sq_mi']
    return tracts['density']


def classify_tracts_by_density(
    tracts: gpd.GeoDataFrame,
    threshold_config: Dict,
    available_sizes: List[int]
) -> pd.Series:
    """Classify each tract as urban/suburban/rural based on density thresholds."""

    density = tracts['density']

    # Default classification
    classification = pd.Series('rural', index=tracts.index)

    if len(available_sizes) == 1:
        # Uniform system - all same size
        classification[:] = f"{available_sizes[0]}-member"

    elif len(available_sizes) == 2:
        # Two-tier system
        if 7 in available_sizes and 5 in available_sizes:
            # 5-7 system
            classification[density >= threshold_config['urban']] = '7-member'
            classification[density < threshold_config['urban']] = '5-member'

        elif 5 in available_sizes and 3 in available_sizes:
            # 3-5 system
            classification[density >= threshold_config['urban']] = '5-member'
            classification[density < threshold_config['urban']] = '3-member'

        elif 7 in available_sizes and 3 in available_sizes:
            # 3-7 system (skip middle)
            classification[density >= threshold_config['urban']] = '7-member'
            classification[density < threshold_config['urban']] = '3-member'

    elif len(available_sizes) == 3:
        # Three-tier system (3-5-7)
        classification[density >= threshold_config['urban']] = '7-member'
        classification[(density >= threshold_config['suburban']) &
                      (density < threshold_config['urban'])] = '5-member'
        classification[density < threshold_config['suburban']] = '3-member'

    return classification


def determine_district_targets(
    tracts: gpd.GeoDataFrame,
    available_sizes: List[int],
    threshold_config: Optional[Dict] = None
) -> Dict[int, int]:
    """
    Determine how many districts of each size to create.

    Returns: {members_per_district: num_districts}
    Example: {3: 80, 5: 30, 7: 12} means 80 3-member, 30 5-member, 12 7-member districts
    """

    total_reps = 435

    if len(available_sizes) == 1:
        # Uniform system
        members = available_sizes[0]
        num_districts = (total_reps + members - 1) // members
        return {members: num_districts}

    # Adaptive system - classify tracts by density
    classification = classify_tracts_by_density(tracts, threshold_config, available_sizes)

    # Count population in each class
    pop_by_class = tracts.groupby(classification)['population'].sum()
    total_pop = tracts['population'].sum()

    # Allocate representatives proportionally
    targets = {}
    allocated_reps = 0

    for size_str in sorted(pop_by_class.index):
        members = int(size_str.split('-')[0])
        pop_share = pop_by_class[size_str] / total_pop
        reps_for_class = round(pop_share * total_reps)

        # Round to nearest multiple of members
        reps_for_class = (reps_for_class // members) * members

        num_districts = reps_for_class // members

        if num_districts > 0:
            targets[members] = num_districts
            allocated_reps += reps_for_class

    # Adjust for rounding errors (ensure exactly 435 reps)
    diff = total_reps - allocated_reps

    if diff != 0:
        # Add/remove districts from largest category
        largest_member = max(targets.keys())
        adjustment_districts = diff // largest_member
        targets[largest_member] += adjustment_districts

    return targets


def run_single_configuration(
    tracts: gpd.GeoDataFrame,
    graph,
    config_name: str,
    available_sizes: List[int],
    threshold_config: Optional[Dict] = None,
    year: int = 2020
) -> Dict:
    """Run redistricting for a single ablation configuration."""

    print(f"\n{'='*70}")
    print(f"Configuration: {config_name}")
    print(f"{'='*70}")

    # Determine targets
    targets = determine_district_targets(tracts, available_sizes, threshold_config)

    print(f"District targets:")
    for members, count in sorted(targets.items()):
        print(f"  {count} districts × {members} members = {count * members} reps")

    total_reps = sum(count * members for members, count in targets.items())
    print(f"Total: {total_reps} representatives")

    # TODO: Implement adaptive recursive bisection
    # For now, placeholder showing what needs to be implemented

    print(f"[TODO] Run recursive bisection with adaptive district sizes")
    print(f"[TODO] This requires extending partition_graph() to accept size targets")

    # Return placeholder results
    result = {
        'config_name': config_name,
        'year': year,
        'available_sizes': str(available_sizes),
        'threshold_name': threshold_config['name'] if threshold_config else 'uniform',
        'target_districts': sum(targets.values()),
        'target_reps': total_reps,
        **{f'num_{k}_member': v for k, v in targets.items()},
        # Metrics (to be computed)
        'gallagher_index': None,
        'mean_polsby_popper': None,
        'median_polsby_popper': None,
        'minor_party_seats': None,
        'libertarian_seats': None,
        'green_seats': None,
        'num_tiers': len(available_sizes),
    }

    return result


# ============================================================================
# Main Ablation Study
# ============================================================================

def run_ablation_study(year: int, output_dir: Path):
    """Run complete ablation study."""

    print("="*70)
    print("MMD ABLATION STUDY")
    print("="*70)
    print(f"Census year: {year}")
    print(f"Output directory: {output_dir}")

    # Load data
    print(f"\nLoading Census {year} data...")
    tract_data_path = Path(f"outputs/data/{year}/units/tracts_with_geometry.parquet")

    if not tract_data_path.exists():
        raise FileNotFoundError(f"Tract data not found: {tract_data_path}")

    tracts = gpd.read_parquet(tract_data_path)
    print(f"Loaded {len(tracts):,} census tracts")

    # Compute density
    print("Computing tract densities...")
    compute_tract_density(tracts)
    print(f"Density range: {tracts['density'].min():.0f} to {tracts['density'].max():.0f} people/sq mi")
    print(f"Median density: {tracts['density'].median():.0f} people/sq mi")

    # Load adjacency graph
    print("\nLoading adjacency graph...")
    adj_path = Path(f"outputs/data/{year}/adjacency/tract_adjacency.parquet")
    graph = load_adjacency_graph(adj_path)
    print(f"Loaded graph: {graph.number_of_nodes():,} nodes, {graph.number_of_edges():,} edges")

    # Run all configurations
    results = []

    # 1. Uniform systems (no density thresholds needed)
    print("\n" + "="*70)
    print("PHASE 1: UNIFORM SYSTEMS")
    print("="*70)

    for config in UNIFORM_CONFIGS:
        result = run_single_configuration(
            tracts=tracts,
            graph=graph,
            config_name=config['name'],
            available_sizes=config['sizes'],
            threshold_config=None,
            year=year
        )
        results.append(result)

    # 2. Adaptive systems (test all density thresholds)
    print("\n" + "="*70)
    print("PHASE 2: ADAPTIVE SYSTEMS")
    print("="*70)

    for config, threshold in product(ADAPTIVE_CONFIGS, DENSITY_THRESHOLDS):
        config_name = f"{config['name']}__{threshold['name']}"

        result = run_single_configuration(
            tracts=tracts,
            graph=graph,
            config_name=config_name,
            available_sizes=config['sizes'],
            threshold_config=threshold,
            year=year
        )
        results.append(result)

    # Save results
    output_dir.mkdir(parents=True, exist_ok=True)

    results_df = pd.DataFrame(results)
    results_path = output_dir / f"ablation_results_{year}.csv"
    results_df.to_csv(results_path, index=False)

    print("\n" + "="*70)
    print("ABLATION STUDY COMPLETE")
    print("="*70)
    print(f"Tested {len(results)} configurations:")
    print(f"  - {len(UNIFORM_CONFIGS)} uniform systems")
    print(f"  - {len(ADAPTIVE_CONFIGS) * len(DENSITY_THRESHOLDS)} adaptive systems")
    print(f"\nResults saved: {results_path}")

    # Summary table
    print("\nConfiguration Summary:")
    summary_cols = ['config_name', 'target_districts', 'target_reps', 'num_tiers']
    if 'num_3_member' in results_df.columns:
        summary_cols.extend([c for c in results_df.columns if c.startswith('num_') and c.endswith('_member')])

    print(results_df[summary_cols].to_string(index=False))

    return results_df


def main():
    parser = argparse.ArgumentParser(description="Run MMD ablation study")
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020])
    parser.add_argument('--output', type=Path, default=Path('outputs/mmd/ablation'))

    args = parser.parse_args()

    results = run_ablation_study(args.year, args.output)

    print(f"\n[SUCCESS] Ablation study complete")
    print(f"Next step: Analyze results to determine optimal configuration")
    print(f"          Run: python scripts/experiments/mmd_analyze_ablation.py")


if __name__ == '__main__':
    main()
