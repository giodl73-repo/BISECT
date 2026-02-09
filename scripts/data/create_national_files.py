#!/usr/bin/env python3
"""
Create national tract and adjacency files from per-state data.

Merges all state-level tract and adjacency data into national files needed for MMD experiments.
"""

import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
import pickle
import networkx as nx

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))
sys.path.insert(0, str(Path(__file__).parent.parent))

from config_2020 import STATE_CONFIG_2020


def merge_national_tracts(year: int, resolution: str = 'block_groups') -> gpd.GeoDataFrame:
    """Merge all state tract/block group files into national file."""

    print(f"Merging {resolution} for all states...")

    units_dir = Path(f"outputs/data/{year}/units")

    all_units = []

    for state_abbr in STATE_CONFIG_2020.keys():
        state_file = units_dir / f"{state_abbr}_{resolution}_{year}.parquet"

        if state_file.exists():
            print(f"  Loading {state_abbr}...")
            state_units = gpd.read_parquet(state_file)
            all_units.append(state_units)
        else:
            print(f"  Warning: {state_abbr} not found")

    if not all_units:
        raise ValueError("No state files found")

    print(f"Concatenating {len(all_units)} states...")
    national = pd.concat(all_units, ignore_index=True)

    # Convert back to GeoDataFrame
    national = gpd.GeoDataFrame(national, geometry='geometry', crs=all_units[0].crs)

    print(f"Total units: {len(national):,}")
    print(f"Total population: {national['population'].sum():,}")

    return national


def merge_national_adjacency(year: int, resolution: str = 'block_group') -> dict:
    """Merge all state adjacency graphs into national graph."""

    print(f"Merging {resolution} adjacency for all states...")

    adj_dir = Path(f"outputs/data/{year}/adjacency")

    # Track global index mapping
    global_index = 0
    global_geoid_to_index = {}
    global_index_to_geoid = {}
    global_adjacency = []
    global_vertex_weights = []

    for state_abbr in STATE_CONFIG_2020.keys():
        adj_file = adj_dir / f"{state_abbr}_{resolution}_adjacency_{year}.pkl"

        if not adj_file.exists():
            print(f"  Warning: {state_abbr} adjacency not found")
            continue

        print(f"  Loading {state_abbr}...")
        with open(adj_file, 'rb') as f:
            state_data = pickle.load(f)

        # Extract components
        state_adj = state_data['adjacency']
        state_weights = state_data['vertex_weights']
        state_idx_to_geoid = state_data['index_to_geoid']
        state_geoid_to_idx = state_data['geoid_to_index']

        # Build local to global index mapping
        local_to_global = {}
        for local_idx, geoid in state_idx_to_geoid.items():
            local_to_global[local_idx] = global_index
            global_geoid_to_index[geoid] = global_index
            global_index_to_geoid[global_index] = geoid
            global_index += 1

        # Remap adjacency lists to global indices
        for local_idx, neighbors in enumerate(state_adj):
            if local_idx < len(state_adj):
                global_neighbors = [local_to_global[local_n] for local_n in neighbors
                                   if local_n in local_to_global]
                global_adjacency.append(global_neighbors)

        # Add vertex weights
        global_vertex_weights.extend(state_weights)

    print(f"Total nodes: {len(global_adjacency):,}")
    print(f"Total edges: {sum(len(neighbors) for neighbors in global_adjacency) // 2:,}")

    # Create combined adjacency dict
    national_adj = {
        'adjacency': global_adjacency,
        'vertex_weights': global_vertex_weights,
        'index_to_geoid': global_index_to_geoid,
        'geoid_to_index': global_geoid_to_index,
        'edge_weights': None  # Can be computed later if needed
    }

    return national_adj


def main():
    year = 2020

    print("="*70)
    print("CREATING NATIONAL FILES FOR MMD EXPERIMENTS")
    print("="*70)

    # Create national tract/block group file
    print("\n[1/2] Creating national units file...")
    national_units = merge_national_tracts(year, resolution='block_groups')

    output_file = Path(f"outputs/data/{year}/units/tracts_with_geometry.parquet")
    output_file.parent.mkdir(parents=True, exist_ok=True)

    national_units.to_parquet(output_file)
    print(f"Saved: {output_file}")
    print(f"Size: {output_file.stat().st_size / 1024 / 1024:.1f} MB")

    # Create national adjacency graph
    print("\n[2/2] Creating national adjacency graph...")
    national_adj = merge_national_adjacency(year, resolution='block_group')

    output_file = Path(f"outputs/data/{year}/adjacency/tract_adjacency.pkl")
    output_file.parent.mkdir(parents=True, exist_ok=True)

    # Save as pickle (adjacency dict)
    with open(output_file, 'wb') as f:
        pickle.dump(national_adj, f)

    print(f"Saved: {output_file}")
    print(f"Size: {output_file.stat().st_size / 1024 / 1024:.1f} MB")

    print("\n" + "="*70)
    print("NATIONAL FILES CREATED SUCCESSFULLY")
    print("="*70)
    print("\nReady to run MMD experiments:")
    print("  python scripts/experiments/mmd_generate_districts.py --members 3 5 --year 2020")


if __name__ == '__main__':
    main()
