#!/usr/bin/env python3
"""
Multi-Member District Generation using Recursive Bisection

Generates MMD configurations with varying representatives per district:
- 3-member: 145 districts × 3 reps = 435 total
- 4-member: 109 districts × 4 reps = 436 total
- 5-member: 87 districts × 5 reps = 435 total

Usage:
    python scripts/experiments/mmd_generate_districts.py --members 3 4 5 --year 2020
"""

import argparse
import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

from apportionment.partition.recursive_bisection import RecursiveBisection
from apportionment.data.adjacency import load_adjacency_graph


def calculate_districts_for_members(total_reps: int, members_per_district: int) -> int:
    """Calculate number of districts needed for given members per district."""
    return (total_reps + members_per_district - 1) // members_per_district  # Ceiling division


def generate_mmd_configuration(year: int, members_per_district: int, output_dir: Path):
    """Generate a single MMD configuration."""
    print(f"\n[MMD Generation] {members_per_district}-member configuration")
    print("=" * 70)

    # Calculate target districts (exclude AK=1, HI=2 seats for contiguous US only)
    total_reps = 432  # 435 - 3 (AK+HI)
    num_districts = calculate_districts_for_members(total_reps, members_per_district)
    actual_total = num_districts * members_per_district

    print(f"Target: {num_districts} districts × {members_per_district} reps = {actual_total} total (continental US)")
    if actual_total != 432:
        print(f"Note: Actual total ({actual_total}) differs from 432 due to rounding")

    # Load census data
    print(f"\nLoading Census {year} data...")
    tract_data_path = Path(f"outputs/data/{year}/units/tracts_with_geometry.parquet")

    if not tract_data_path.exists():
        raise FileNotFoundError(
            f"Tract data not found: {tract_data_path}\n"
            f"Run data preparation pipeline first"
        )

    tracts = gpd.read_parquet(tract_data_path)
    print(f"Loaded {len(tracts):,} census tracts")
    print(f"Total population: {tracts['population'].sum():,}")

    # Filter out non-contiguous states (Alaska, Hawaii) - they create disconnected graph components
    # AK GEOID starts with '02', HI starts with '15'
    original_count = len(tracts)
    tracts = tracts[~tracts['GEOID'].str.startswith(('02', '15'))]
    filtered_count = original_count - len(tracts)
    if filtered_count > 0:
        print(f"Filtered {filtered_count:,} units from AK/HI (continental US only)")
        print(f"Remaining: {len(tracts):,} census tracts, {tracts['population'].sum():,} population")

    # Load adjacency graph
    print("\nLoading adjacency graph...")
    adj_path = Path(f"outputs/data/{year}/adjacency/tract_adjacency.pkl")

    if not adj_path.exists():
        raise FileNotFoundError(
            f"Adjacency graph not found: {adj_path}\n"
            f"Run adjacency generation first"
        )

    # Load adjacency dict and convert to networkx graph
    import pickle
    import networkx as nx

    print("Loading adjacency from pickle...")
    with open(adj_path, 'rb') as f:
        adj_data = pickle.load(f)

    # Convert adjacency list to networkx graph
    print("Converting to networkx graph...")
    graph = nx.Graph()

    # Add nodes with population weights (filter out AK and HI)
    for idx, geoid in adj_data['index_to_geoid'].items():
        if not geoid.startswith(('02', '15')):  # Exclude AK and HI
            graph.add_node(geoid, population=adj_data['vertex_weights'][idx])

    # Add edges from adjacency list (only between continental US nodes)
    for idx, neighbors in enumerate(adj_data['adjacency']):
        geoid = adj_data['index_to_geoid'][idx]
        if not geoid.startswith(('02', '15')):  # Source node in continental US
            for neighbor_idx in neighbors:
                neighbor_geoid = adj_data['index_to_geoid'][neighbor_idx]
                if not neighbor_geoid.startswith(('02', '15')):  # Neighbor also in continental US
                    graph.add_edge(geoid, neighbor_geoid)
    print(f"Loaded graph with {graph.number_of_nodes():,} nodes, {graph.number_of_edges():,} edges (continental US only)")

    # Calculate population target
    total_population = tracts['population'].sum()
    target_pop_per_district = total_population / num_districts
    tolerance = 0.005  # ±0.5%

    print(f"\nPopulation per district: {target_pop_per_district:,.0f}")
    print(f"Tolerance: ±{tolerance*100}% ({target_pop_per_district*tolerance:,.0f})")

    # Run recursive bisection
    print(f"\nRunning recursive bisection for {num_districts} districts...")
    print("(This may take 20-30 minutes)")

    # Create node index mapping
    sorted_nodes = sorted(graph.nodes())
    node_to_idx = {node: idx for idx, node in enumerate(sorted_nodes)}

    # Convert networkx graph to adjacency list format (with integer indices)
    adjacency_list = []
    for node in sorted_nodes:
        # Get neighbors and convert to integer indices
        neighbors = [node_to_idx[neighbor] for neighbor in graph[node]]
        adjacency_list.append(sorted(neighbors))

    # Get vertex weights (populations) in node order
    vertex_weights = np.array([graph.nodes[node]['population'] for node in sorted_nodes])

    # Create and run recursive bisection
    rb = RecursiveBisection(
        adjacency=adjacency_list,
        vertex_weights=vertex_weights,
        num_districts=num_districts,
        state_code="US",
        debug=False,
        ufactor=int(tolerance * 1000)  # Convert 0.005 to 5
    )

    assignments_by_idx = rb.partition()

    # Convert from node indices back to GEOIDs
    idx_to_node = {idx: node for node, idx in node_to_idx.items()}
    assignments = {idx_to_node[idx]: district_id for idx, district_id in assignments_by_idx.items()}

    # Convert to dataframe
    districts_df = pd.DataFrame([
        {'GEOID': geoid, 'district': district_id}
        for geoid, district_id in assignments.items()
    ])

    # Validate
    tract_districts = tracts.merge(districts_df, on='GEOID')
    district_pops = tract_districts.groupby('district')['population'].sum()

    print(f"\n[Validation]")
    print(f"Districts created: {len(district_pops)}")
    print(f"Population balance:")
    print(f"  Min: {district_pops.min():,.0f} ({(district_pops.min()/target_pop_per_district - 1)*100:+.2f}%)")
    print(f"  Max: {district_pops.max():,.0f} ({(district_pops.max()/target_pop_per_district - 1)*100:+.2f}%)")
    print(f"  Mean: {district_pops.mean():,.0f}")
    print(f"  Std: {district_pops.std():,.0f}")

    # Check tolerance
    min_pop_ratio = district_pops.min() / target_pop_per_district
    max_pop_ratio = district_pops.max() / target_pop_per_district

    if min_pop_ratio < (1 - tolerance) or max_pop_ratio > (1 + tolerance):
        print(f"  [WARNING] Population balance exceeds ±{tolerance*100}% tolerance")
    else:
        print(f"  [OK] All districts within ±{tolerance*100}% tolerance")

    # Save outputs
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save district assignments
    assignments_path = output_dir / "districts.csv"
    districts_df.to_csv(assignments_path, index=False)
    print(f"\nSaved: {assignments_path}")

    # Save district geometries (union of tract geometries)
    print("Computing district geometries...")
    district_geoms = tract_districts.dissolve(by='district', aggfunc='sum')
    district_geoms = district_geoms[['geometry', 'population']]
    district_geoms['members'] = members_per_district

    geoms_path = output_dir / "district_geometries.geojson"
    district_geoms.to_file(geoms_path, driver='GeoJSON')
    print(f"Saved: {geoms_path}")

    # Save summary stats
    summary = {
        'year': year,
        'members_per_district': members_per_district,
        'num_districts': len(district_pops),
        'total_representatives': actual_total,
        'total_population': total_population,
        'target_pop_per_district': target_pop_per_district,
        'min_population': district_pops.min(),
        'max_population': district_pops.max(),
        'mean_population': district_pops.mean(),
        'std_population': district_pops.std(),
        'tolerance': tolerance,
        'within_tolerance': min_pop_ratio >= (1 - tolerance) and max_pop_ratio <= (1 + tolerance)
    }

    summary_df = pd.DataFrame([summary])
    summary_path = output_dir / "summary.csv"
    summary_df.to_csv(summary_path, index=False)
    print(f"Saved: {summary_path}")

    print(f"\n[Complete] {members_per_district}-member configuration generated")
    return summary


def main():
    parser = argparse.ArgumentParser(description="Generate MMD configurations")
    parser.add_argument('--members', type=int, nargs='+', required=True,
                        choices=[3, 4, 5, 7],
                        help='Members per district to generate (e.g., --members 3 5 7)')
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020],
                        help='Census year (default: 2020)')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/mmd'),
                        help='Output directory (default: outputs/mmd)')

    args = parser.parse_args()

    print("=" * 70)
    print("MMD DISTRICT GENERATION")
    print("=" * 70)
    print(f"Census year: {args.year}")
    print(f"Configurations: {args.members}-member districts")
    print(f"Output directory: {args.output_dir}")

    summaries = []

    for members in sorted(args.members):
        config_dir = args.output_dir / f"{members}-member"
        summary = generate_mmd_configuration(args.year, members, config_dir)
        summaries.append(summary)

    # Save combined summary
    print("\n" + "=" * 70)
    print("ALL CONFIGURATIONS COMPLETE")
    print("=" * 70)

    all_summaries = pd.DataFrame(summaries)
    all_summary_path = args.output_dir / "all_configurations_summary.csv"
    all_summaries.to_csv(all_summary_path, index=False)
    print(f"\nCombined summary: {all_summary_path}")

    print("\nSummary table:")
    print(all_summaries[['members_per_district', 'num_districts', 'total_representatives',
                          'mean_population', 'within_tolerance']].to_string(index=False))

    print(f"\n[SUCCESS] Generated {len(args.members)} MMD configurations")
    print(f"Next step: Run electoral simulation (scripts/experiments/mmd_simulate_election.py)")


if __name__ == '__main__':
    main()
