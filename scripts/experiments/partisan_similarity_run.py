#!/usr/bin/env python3
"""
Partisan Similarity Districts Experiment (Paper 17)

Creates politically homogeneous districts using edge-weighted graph partitioning
based on partisan vote similarity.

Edge weight formula: w = alpha if |lean_i - lean_j| < tau, else 1
- alpha (scaling factor): 1, 5, 10, 25, 50, 100
- tau (similarity threshold): 10, 15, 20 percentage points

Usage:
    python scripts/experiments/partisan_similarity_run.py --year 2020 --states CA TX --alpha 1 10 50
    python scripts/experiments/partisan_similarity_run.py --year 2020 --alpha all --tau all
"""

import argparse
import sys
from pathlib import Path
import pandas as pd
import geopandas as gpd
import numpy as np
import pickle
import networkx as nx
from typing import Dict, Tuple

# Add src and project root to path
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.partition.metis_wrapper import partition_graph
from apportionment.data.adjacency import load_adjacency_graph
from scripts.config_2020 import STATE_CONFIG_2020

# Extract state-to-districts mapping
STATE_DISTRICTS = {abbr: config['districts'] for abbr, config in STATE_CONFIG_2020.items()}


def load_tract_election_data(year: int) -> pd.DataFrame:
    """Load tract-level presidential election results."""
    print(f"Loading {year} presidential election data...")

    if year == 2020:
        elec_path = Path(f"data/{year}/elections/2020_president/tracts-2020-RLCR.csv")
    else:
        raise ValueError(f"Election data not available for {year}")

    if not elec_path.exists():
        raise FileNotFoundError(f"Election data not found: {elec_path}")

    # Load election data
    elec = pd.read_csv(elec_path, encoding='utf-8-sig')

    # Extract relevant columns
    elec = elec.rename(columns={
        'tract_GEOID': 'GEOID',
        'G20PREDBID': 'dem_votes',
        'G20PRERTRU': 'rep_votes'
    })

    # Keep only GEOID and vote columns
    elec = elec[['GEOID', 'dem_votes', 'rep_votes']].copy()

    # Ensure GEOID is string (for merging with tract data)
    elec['GEOID'] = elec['GEOID'].astype(str).str.zfill(11)  # 11-digit GEOID

    # Compute partisan lean (D% - R%)
    elec['total_votes'] = elec['dem_votes'] + elec['rep_votes']
    elec['dem_share'] = elec['dem_votes'] / elec['total_votes']
    elec['rep_share'] = elec['rep_votes'] / elec['total_votes']
    elec['partisan_lean'] = (elec['dem_share'] - elec['rep_share']) * 100  # Percentage points

    # Remove tracts with no votes
    elec = elec[elec['total_votes'] > 0].copy()

    print(f"Loaded {len(elec):,} tracts with election data")
    print(f"  Mean partisan lean: {elec['partisan_lean'].mean():.1f} pp (D+)")
    print(f"  Std partisan lean: {elec['partisan_lean'].std():.1f} pp")

    return elec


def compute_edge_weights(
    graph: nx.Graph,
    tract_lean: Dict[str, float],
    alpha: float,
    tau: float
) -> Dict[Tuple[str, str], float]:
    """
    Compute edge weights based on partisan similarity.

    Args:
        graph: Adjacency graph
        tract_lean: Dict mapping tract GEOID to partisan lean
        alpha: Weight scaling factor (e.g., 10 means 10x weight)
        tau: Similarity threshold in percentage points

    Returns:
        Dict mapping (tract_i, tract_j) to edge weight
    """
    edge_weights = {}
    missing_lean = []

    for u, v in graph.edges():
        # Get partisan leans
        lean_u = tract_lean.get(u)
        lean_v = tract_lean.get(v)

        # If either tract missing election data, use neutral weight
        if lean_u is None or lean_v is None:
            edge_weights[(u, v)] = 1.0
            if lean_u is None and u not in missing_lean:
                missing_lean.append(u)
            if lean_v is None and v not in missing_lean:
                missing_lean.append(v)
            continue

        # Compute partisan similarity (absolute difference in lean)
        similarity = abs(lean_u - lean_v)

        # Apply edge weight formula
        if similarity < tau:
            # Similar tracts → hard to cut (high weight)
            weight = alpha
        else:
            # Dissimilar tracts → easy to cut (neutral weight)
            weight = 1.0

        edge_weights[(u, v)] = weight

    if missing_lean:
        print(f"  WARNING: {len(missing_lean)} tracts missing election data (using neutral weights)")

    return edge_weights


def run_partisan_similarity_redistricting(
    year: int,
    alpha: float,
    tau: float,
    states: list = None,
    output_dir: Path = None
):
    """Run redistricting with partisan similarity edge-weighting."""

    config_name = f"alpha{alpha}_tau{tau}"
    print(f"\n{'='*70}")
    print(f"Partisan Similarity Redistricting: alpha={alpha}, tau={tau}pp")
    print(f"{'='*70}\n")

    # Set output directory
    if output_dir is None:
        output_dir = Path(f"outputs/partisan_similarity/{year}/{config_name}")
    output_dir.mkdir(parents=True, exist_ok=True)

    # Load tract data
    print(f"Loading Census {year} tract data...")
    tract_path = Path(f"outputs/data/{year}/units/tracts_with_geometry.parquet")
    if not tract_path.exists():
        raise FileNotFoundError(f"Tract data not found: {tract_path}")

    tracts = gpd.read_parquet(tract_path)
    print(f"Loaded {len(tracts):,} block groups")

    # Convert block group GEOID (12 digits) to tract GEOID (11 digits) for election data merge
    tracts['tract_geoid'] = tracts['GEOID'].str[:11]

    # Extract state FIPS from GEOID (first 2 digits)
    tracts['state_fips'] = tracts['GEOID'].str[:2]

    # Map state FIPS to abbreviations
    STATE_FIPS_TO_ABBR = {
        '01': 'AL', '02': 'AK', '04': 'AZ', '05': 'AR', '06': 'CA', '08': 'CO', '09': 'CT',
        '10': 'DE', '11': 'DC', '12': 'FL', '13': 'GA', '15': 'HI', '16': 'ID', '17': 'IL',
        '18': 'IN', '19': 'IA', '20': 'KS', '21': 'KY', '22': 'LA', '23': 'ME', '24': 'MD',
        '25': 'MA', '26': 'MI', '27': 'MN', '28': 'MS', '29': 'MO', '30': 'MT', '31': 'NE',
        '32': 'NV', '33': 'NH', '34': 'NJ', '35': 'NM', '36': 'NY', '37': 'NC', '38': 'ND',
        '39': 'OH', '40': 'OK', '41': 'OR', '42': 'PA', '44': 'RI', '45': 'SC', '46': 'SD',
        '47': 'TN', '48': 'TX', '49': 'UT', '50': 'VT', '51': 'VA', '53': 'WA', '54': 'WV',
        '55': 'WI', '56': 'WY'
    }
    tracts['state'] = tracts['state_fips'].map(STATE_FIPS_TO_ABBR)

    # Filter states if specified (do this before loading election data)
    if states:
        tracts = tracts[tracts['state'].isin(states)].copy()
        print(f"\nFiltered to {len(states)} states: {', '.join(states)}")
        print(f"  {len(tracts):,} tracts")

    # Load election data
    elec = load_tract_election_data(year)

    # Merge election data with block groups using tract GEOID
    tracts = tracts.merge(elec[['GEOID', 'partisan_lean']],
                          left_on='tract_geoid', right_on='GEOID',
                          how='left', suffixes=('', '_elec'))
    print(f"Matched {tracts['partisan_lean'].notna().sum():,} block groups with election data")

    # Create tract_lean dict for edge weighting (using block group GEOID as key)
    tract_lean = tracts.set_index('GEOID')['partisan_lean'].to_dict()

    # Load adjacency graph
    print("\nLoading adjacency graph...")
    adj_path = Path(f"outputs/data/{year}/adjacency/tract_adjacency.pkl")
    if not adj_path.exists():
        raise FileNotFoundError(f"Adjacency graph not found: {adj_path}")

    with open(adj_path, 'rb') as f:
        adj_data = pickle.load(f)

    # Extract adjacency structure
    adjacency_list = adj_data['adjacency']
    index_to_geoid = adj_data['index_to_geoid']

    # Convert index-based adjacency to GEOID-based networkx graph
    graph = nx.Graph()
    for idx, neighbors in enumerate(adjacency_list):
        geoid = index_to_geoid[idx]
        graph.add_node(geoid)
        for neighbor_idx in neighbors:
            neighbor_geoid = index_to_geoid[neighbor_idx]
            graph.add_edge(geoid, neighbor_geoid)

    # Filter graph to selected tracts
    selected_geoids = set(tracts['GEOID'].values)
    graph = graph.subgraph(selected_geoids).copy()
    print(f"Adjacency graph: {graph.number_of_nodes():,} nodes, {graph.number_of_edges():,} edges")

    # Compute edge weights
    print(f"\nComputing edge weights (alpha={alpha}, tau={tau}pp)...")
    edge_weights = compute_edge_weights(graph, tract_lean, alpha, tau)

    # Stats on edge weights
    weights = list(edge_weights.values())
    high_weight_pct = (np.array(weights) > 1.0).mean() * 100
    print(f"  {len(edge_weights):,} edges")
    print(f"  {high_weight_pct:.1f}% edges have high weight (alpha={alpha})")
    print(f"  {100-high_weight_pct:.1f}% edges have neutral weight (1.0)")

    # Determine number of districts
    if states:
        num_districts = sum(STATE_DISTRICTS.get(s, 1) for s in states)
    else:
        num_districts = 435

    print(f"\nTarget: {num_districts} districts")

    # Special case: single district (no partitioning needed)
    if num_districts == 1:
        print("\nSingle district - assigning all units to district 0...")
        tracts['district'] = 0

        # Compute statistics
        print("\nComputing district statistics...")
        district_stats = compute_district_statistics(tracts)

        # Save results
        print(f"\nSaving results to {output_dir}/...")
        tracts.to_parquet(output_dir / "districts.parquet")
        district_stats.to_csv(output_dir / "district_statistics.csv", index=False)

        # Print summary
        print_summary(district_stats, alpha, tau)

        return tracts, district_stats

    # Run recursive bisection
    print("\nRunning recursive bisection with partisan similarity edge-weighting...")

    # Convert graph + edge_weights to index-based format
    geoid_to_index = {geoid: idx for idx, geoid in enumerate(sorted(graph.nodes()))}
    index_to_geoid = {idx: geoid for geoid, idx in geoid_to_index.items()}

    # Build adjacency list (index-based)
    adjacency_list = [[] for _ in range(len(graph.nodes()))]
    for u, v in graph.edges():
        idx_u = geoid_to_index[u]
        idx_v = geoid_to_index[v]
        adjacency_list[idx_u].append(idx_v)
        adjacency_list[idx_v].append(idx_u)

    # Build vertex weights (population)
    populations = tracts.set_index('GEOID')['population'].to_dict()
    vertex_weights = np.array([populations[index_to_geoid[i]] for i in range(len(index_to_geoid))])

    # Convert edge weights to index-based format
    edge_weights_indexed = {}
    for (u, v), weight in edge_weights.items():
        if u in geoid_to_index and v in geoid_to_index:
            idx_u = geoid_to_index[u]
            idx_v = geoid_to_index[v]
            # Store with canonical ordering
            key = (min(idx_u, idx_v), max(idx_u, idx_v))
            edge_weights_indexed[key] = weight

    # Run partition
    partition = partition_graph(
        adjacency=adjacency_list,
        vertex_weights=vertex_weights,
        nparts=num_districts,
        edge_weights=edge_weights_indexed,
        ufactor=5  # 0.5% tolerance
    )

    # Convert partition indices to GEOID assignments
    assignments = {index_to_geoid[i]: int(partition[i]) for i in range(len(partition))}

    # Add district assignments to tracts
    tracts['district'] = tracts['GEOID'].map(assignments)

    # Compute district-level statistics
    print("\nComputing district statistics...")
    district_stats = compute_district_statistics(tracts)

    # Save results
    print(f"\nSaving results to {output_dir}/...")
    tracts.to_parquet(output_dir / "districts.parquet")
    district_stats.to_csv(output_dir / "district_statistics.csv", index=False)

    # Print summary
    print_summary(district_stats, alpha, tau)

    return tracts, district_stats


def compute_district_statistics(tracts: gpd.GeoDataFrame) -> pd.DataFrame:
    """Compute district-level statistics."""

    # Group by district
    district_groups = tracts.groupby('district')

    stats = []
    for district_id, group in district_groups:
        # Population
        pop = group['population'].sum()

        # Partisan lean (weighted by votes)
        if 'partisan_lean' in group.columns:
            # Weight by tract population as proxy for votes
            total_weight = group['population'].sum()
            if total_weight > 0:
                weighted_lean = (group['partisan_lean'] * group['population']).sum() / total_weight
            else:
                weighted_lean = np.nan
        else:
            weighted_lean = np.nan

        # Compactness (Polsby-Popper)
        district_geom = group.unary_union
        area = district_geom.area
        perimeter = district_geom.length
        if perimeter > 0:
            polsby_popper = (4 * np.pi * area) / (perimeter ** 2)
        else:
            polsby_popper = 0.0

        stats.append({
            'district': district_id,
            'population': pop,
            'num_tracts': len(group),
            'partisan_lean': weighted_lean,
            'polsby_popper': polsby_popper
        })

    return pd.DataFrame(stats)


def print_summary(district_stats: pd.DataFrame, alpha: float, tau: float):
    """Print summary statistics."""
    print(f"\n{'='*70}")
    print(f"Results Summary (alpha={alpha}, tau={tau}pp)")
    print(f"{'='*70}\n")

    # Overall stats
    print(f"Districts: {len(district_stats)}")
    print(f"Total population: {district_stats['population'].sum():,}")
    print(f"Mean population: {district_stats['population'].mean():,.0f}")
    print(f"Population std dev: {district_stats['population'].std():,.0f}")

    # Partisan lean
    print(f"\nPartisan Lean:")
    print(f"  Mean |lean|: {district_stats['partisan_lean'].abs().mean():.1f} pp")
    print(f"  Std lean: {district_stats['partisan_lean'].std():.1f} pp")

    # Safe seats
    safe_10 = (district_stats['partisan_lean'].abs() > 10).sum()
    safe_15 = (district_stats['partisan_lean'].abs() > 15).sum()
    safe_20 = (district_stats['partisan_lean'].abs() > 20).sum()

    total = len(district_stats)
    print(f"\nSafe Seats:")
    print(f"  >10pp: {safe_10} ({safe_10/total*100:.1f}%)")
    print(f"  >15pp: {safe_15} ({safe_15/total*100:.1f}%)")
    print(f"  >20pp: {safe_20} ({safe_20/total*100:.1f}%)")

    # Compactness
    print(f"\nCompactness (Polsby-Popper):")
    print(f"  Mean: {district_stats['polsby_popper'].mean():.3f}")
    print(f"  Median: {district_stats['polsby_popper'].median():.3f}")
    print(f"  Min: {district_stats['polsby_popper'].min():.3f}")
    print(f"  Max: {district_stats['polsby_popper'].max():.3f}")


def main():
    parser = argparse.ArgumentParser(description="Partisan Similarity Districts Experiment")
    parser.add_argument('--year', type=int, default=2020, help="Census year")
    parser.add_argument('--states', nargs='+', help="State abbreviations (e.g., CA TX NY)")
    parser.add_argument('--alpha', nargs='+', help="Alpha values (e.g., 1 10 50) or 'all'")
    parser.add_argument('--tau', nargs='+', help="Tau values (e.g., 10 15 20) or 'all'")
    parser.add_argument('--output', type=str, help="Output directory")

    args = parser.parse_args()

    # Parse alpha values
    if args.alpha and 'all' in args.alpha:
        alphas = [1, 5, 10, 25, 50, 100]
    elif args.alpha:
        alphas = [float(a) for a in args.alpha]
    else:
        alphas = [1, 10, 50]  # Default: baseline, moderate, strong

    # Parse tau values
    if args.tau and 'all' in args.tau:
        taus = [10, 15, 20]
    elif args.tau:
        taus = [float(t) for t in args.tau]
    else:
        taus = [15]  # Default: moderate threshold

    # Run experiments
    print(f"Running {len(alphas)} × {len(taus)} = {len(alphas)*len(taus)} configurations\n")

    for alpha in alphas:
        for tau in taus:
            try:
                tracts, stats = run_partisan_similarity_redistricting(
                    year=args.year,
                    alpha=alpha,
                    tau=tau,
                    states=args.states,
                    output_dir=Path(args.output) if args.output else None
                )
            except Exception as e:
                print(f"\nERROR in alpha={alpha}, tau={tau}: {e}")
                continue


if __name__ == '__main__':
    main()
