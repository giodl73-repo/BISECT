"""
Build national adjacency graph for all 85,331 census tracts across 50 states.

This script creates a unified national graph treating cross-state boundaries
as normal adjacencies. Used for Paper #13 national redistricting experiment.

Usage:
    python scripts/experimental/build_national_adjacency_graph.py --year 2020

Output:
    outputs/experimental/national_adjacency_2020.pkl
    Contains: adjacency list, geoid_to_index mapping, cross-state edge list
"""

import argparse
import pickle
from pathlib import Path
from typing import Dict, List, Set, Tuple

import geopandas as gpd
import numpy as np
from libpysal import weights
from tqdm import tqdm

# Add src and scripts to path
import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))
sys.path.insert(0, str(project_root / "scripts"))

from apportionment.data.adjacency import _compute_boundary_lengths
from config.download_sources import STATE_FIPS, STATE_NAMES


def load_all_state_tracts(year: int, data_dir: Path) -> gpd.GeoDataFrame:
    """
    Load census tract geometries for all 50 states + DC.

    Returns:
        GeoDataFrame with columns: GEOID, geometry, STATE_FIPS
    """
    print(f"\nLoading tract geometries for all states ({year})...")

    all_tracts = []

    for state_abbr, fips in tqdm(STATE_FIPS.items(), desc="Loading states"):
        state_name = STATE_NAMES[state_abbr]

        # Try multiple file patterns (gpkg first, then shapefile)
        tract_file_gpkg = data_dir / f"{year}/tiger/tracts/{state_name}/tracts_{state_name}_{year}.gpkg"
        tract_file_shp = data_dir / f"{year}/tiger/tracts/tl_{year}_{fips}_tract/tl_{year}_{fips}_tract.shp"

        if tract_file_gpkg.exists():
            tract_file = tract_file_gpkg
        elif tract_file_shp.exists():
            tract_file = tract_file_shp
        else:
            print(f"  Warning: Missing tract file for {state_abbr} ({state_name})")
            continue

        gdf = gpd.read_file(tract_file)

        # Add state identifier
        gdf['STATE_FIPS'] = fips
        gdf['STATE_ABBR'] = state_abbr
        gdf['STATE_NAME'] = state_name

        all_tracts.append(gdf[['GEOID', 'geometry', 'STATE_FIPS', 'STATE_ABBR', 'STATE_NAME']])

    # Concatenate all states
    national_gdf = gpd.GeoDataFrame(
        pd.concat(all_tracts, ignore_index=True),
        crs=all_tracts[0].crs
    )

    print(f"  Loaded {len(national_gdf):,} tracts from {len(all_tracts)} states")

    return national_gdf


def build_national_adjacency(gdf: gpd.GeoDataFrame) -> Tuple[List[List[int]], Dict[str, int], Dict[int, str], List[Tuple[int, int]]]:
    """
    Build adjacency graph for all tracts, including cross-state edges.

    Returns:
        adjacency: List of neighbor indices for each tract
        geoid_to_index: Mapping from GEOID to index
        index_to_geoid: Mapping from index to GEOID
        cross_state_edges: List of (i, j) pairs where tracts are in different states
    """
    print("\nBuilding national adjacency graph...")

    # Create index mappings
    geoid_to_index = {geoid: i for i, geoid in enumerate(gdf['GEOID'])}
    index_to_geoid = {i: geoid for geoid, i in geoid_to_index.items()}

    # Compute Queen contiguity (shares edge or vertex)
    print("  Computing spatial adjacency (Queen contiguity)...")
    w = weights.Queen.from_dataframe(gdf, use_index=False, silence_warnings=True)

    # Convert to adjacency list
    adjacency = [[] for _ in range(len(gdf))]
    cross_state_edges = []

    print("  Building adjacency list and identifying cross-state edges...")
    for i in tqdm(range(len(gdf)), desc="  Processing tracts"):
        neighbors = w.neighbors.get(i, [])
        adjacency[i] = neighbors

        # Check for cross-state edges
        state_i = gdf.iloc[i]['STATE_FIPS']
        for j in neighbors:
            if j > i:  # Only count each edge once
                state_j = gdf.iloc[j]['STATE_FIPS']
                if state_i != state_j:
                    cross_state_edges.append((i, j))

    total_edges = sum(len(neighbors) for neighbors in adjacency) // 2
    print(f"  Graph statistics:")
    print(f"    Nodes: {len(adjacency):,}")
    print(f"    Edges: {total_edges:,}")
    print(f"    Cross-state edges: {len(cross_state_edges):,}")

    return adjacency, geoid_to_index, index_to_geoid, cross_state_edges


def validate_national_graph(
    gdf: gpd.GeoDataFrame,
    adjacency: List[List[int]],
    cross_state_edges: List[Tuple[int, int]]
) -> None:
    """
    Validate national graph properties.

    Checks:
    - Graph is connected (single component)
    - All tracts have at least one neighbor
    - Cross-state edges are reasonable
    """
    print("\nValidating national graph...")

    # Check connectivity via DFS
    visited = set()
    stack = [0]

    while stack:
        node = stack.pop()
        if node in visited:
            continue
        visited.add(node)
        stack.extend(adjacency[node])

    if len(visited) == len(adjacency):
        print("  [OK] Graph is fully connected (single component)")
    else:
        print(f"  [WARNING] Graph has {len(adjacency) - len(visited)} unreachable nodes")

    # Check isolated nodes
    isolated = [i for i in range(len(adjacency)) if len(adjacency[i]) == 0]
    if isolated:
        print(f"  [WARNING] {len(isolated)} isolated nodes (islands?)")
        for i in isolated[:5]:  # Show first 5
            geoid = gdf.iloc[i]['GEOID']
            state = gdf.iloc[i]['STATE_ABBR']
            print(f"    {geoid} ({state})")
    else:
        print("  [OK] No isolated nodes")

    # Analyze cross-state edges by state pair
    print("\n  Cross-state adjacencies by border:")

    state_pairs = {}
    for i, j in cross_state_edges:
        state_i = gdf.iloc[i]['STATE_ABBR']
        state_j = gdf.iloc[j]['STATE_ABBR']
        pair = tuple(sorted([state_i, state_j]))
        state_pairs[pair] = state_pairs.get(pair, 0) + 1

    # Sort by edge count
    sorted_pairs = sorted(state_pairs.items(), key=lambda x: x[1], reverse=True)

    for (state_a, state_b), count in sorted_pairs[:10]:  # Top 10
        print(f"    {state_a} <-> {state_b}: {count:,} edges")

    print(f"\n  Total state-pair borders: {len(state_pairs)}")


def main():
    parser = argparse.ArgumentParser(description="Build national adjacency graph")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/experimental'), help='Output directory')
    args = parser.parse_args()

    # Create output directory
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Load all state tracts
    national_gdf = load_all_state_tracts(args.year, args.data_dir)

    # Build adjacency graph
    adjacency, geoid_to_index, index_to_geoid, cross_state_edges = build_national_adjacency(national_gdf)

    # Validate graph
    validate_national_graph(national_gdf, adjacency, cross_state_edges)

    # Save to file
    output_file = args.output_dir / f"national_adjacency_{args.year}.pkl"

    print(f"\nSaving national adjacency graph to {output_file}...")

    with open(output_file, 'wb') as f:
        pickle.dump({
            'adjacency': adjacency,
            'geoid_to_index': geoid_to_index,
            'index_to_geoid': index_to_geoid,
            'cross_state_edges': cross_state_edges,
            'year': args.year,
            'n_tracts': len(adjacency),
            'n_edges': sum(len(neighbors) for neighbors in adjacency) // 2,
            'n_cross_state_edges': len(cross_state_edges),
            'tract_geoids': list(geoid_to_index.keys()),
            'state_fips': national_gdf['STATE_FIPS'].tolist(),
            'state_abbrs': national_gdf['STATE_ABBR'].tolist(),
            'state_names': national_gdf['STATE_NAME'].tolist()
        }, f)

    print(f"  [OK] Saved {len(adjacency):,} nodes, {sum(len(n) for n in adjacency) // 2:,} edges")
    print(f"\nNational adjacency graph ready for redistricting!")


if __name__ == '__main__':
    import pandas as pd
    main()
