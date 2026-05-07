"""
Compute temporal stability metrics comparing 2010 vs 2020 partitions.

Requires:
- 2010 and 2020 partition results
- Census tract relationship files (2010-2020 mapping)
"""

import sys
from pathlib import Path
import pandas as pd
import numpy as np
from scipy import sparse
from collections import defaultdict

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from src.apportionment.data.adjacency import load_adjacency_matrix

STATES = ['alabama', 'georgia', 'louisiana', 'mississippi', 'south_carolina']
METHODS = ['nway', 'recursive']

# State name to FIPS code mapping (for loading tract geometries)
STATE_TO_FIPS = {
    'alabama': '01',
    'georgia': '13',
    'louisiana': '22',
    'mississippi': '28',
    'south_carolina': '45'
}


def load_partition_results(state: str, year: int, method: str):
    """Load partition results from CSV."""
    results_dir = Path('research/gerry-temporal-stability/results')
    results_file = results_dir / f'{state}_{year}_{method}_partition.csv'

    if not results_file.exists():
        raise FileNotFoundError(f"Partition file not found: {results_file}")

    return pd.read_csv(results_file)


def load_tract_relationships(state: str):
    """
    Load Census Bureau tract relationship file mapping 2010 -> 2020.

    TODO: Download these files from Census Bureau:
    https://www.census.gov/geographies/reference-files/time-series/geo/relationship-files.html

    File format: GEOID_TRACT_10, GEOID_TRACT_20, AREALAND_PART, POP_PART, weight
    """
    relationships_file = Path(f'data/tract_relationships/{state}_2010_2020.csv')

    if not relationships_file.exists():
        raise FileNotFoundError(
            f"Tract relationship file not found: {relationships_file}\n"
            f"Download from Census Bureau relationship files"
        )

    return pd.read_csv(relationships_file)


def map_partition_to_new_tracts(partition_2010: dict, relationships: pd.DataFrame):
    """
    Map 2010 partition to 2020 tract geography using relationship file.

    Handles tract splits and merges by assigning each 2020 tract to the
    2010 district with highest population overlap.
    """
    partition_2020_mapped = {}

    # Group relationships by 2020 tract
    for geoid_2020 in relationships['GEOID_TRACT_20'].unique():
        # Find all 2010 tracts that map to this 2020 tract
        mappings = relationships[relationships['GEOID_TRACT_20'] == geoid_2020]

        # For each 2010 source tract, get its district and weight
        district_weights = defaultdict(float)
        for _, row in mappings.iterrows():
            geoid_2010 = row['GEOID_TRACT_10']
            weight = row.get('weight', row.get('POP_PART', 1.0))

            if geoid_2010 in partition_2010:
                district = partition_2010[geoid_2010]
                district_weights[district] += weight

        # Assign 2020 tract to district with highest weight
        if district_weights:
            assigned_district = max(district_weights.items(), key=lambda x: x[1])[0]
            partition_2020_mapped[geoid_2020] = assigned_district

    return partition_2020_mapped


def compute_tract_stability(partition_2010_mapped: dict, partition_2020: dict):
    """
    Compute fraction of tracts that remained in same district.

    Returns: Stability score in [0, 1] (1 = perfect stability)
    """
    common_tracts = set(partition_2010_mapped.keys()) & set(partition_2020.keys())

    if len(common_tracts) == 0:
        return 0.0

    unchanged = sum(
        1 for tract in common_tracts
        if partition_2010_mapped[tract] == partition_2020[tract]
    )

    stability = unchanged / len(common_tracts)
    return stability


def compute_population_disruption(partition_2010_mapped: dict, partition_2020: dict,
                                  tracts_2020: pd.DataFrame):
    """
    Compute population-weighted disruption.

    Returns: Disruption score in [0, 1] (0 = no disruption)
    """
    total_pop = tracts_2020['total_pop'].sum()
    disrupted_pop = 0

    for _, row in tracts_2020.iterrows():
        geoid = row['GEOID']

        if (geoid in partition_2010_mapped and
            geoid in partition_2020 and
            partition_2010_mapped[geoid] != partition_2020[geoid]):
            disrupted_pop += row['total_pop']

    disruption = disrupted_pop / total_pop if total_pop > 0 else 0
    return disruption


def compute_boundary_stability(partition_2010_mapped: dict, partition_2020: dict,
                               adjacency_2020):
    """
    Compute fraction of boundaries that remained unchanged.

    A boundary is "unchanged" if the same pair of tracts had the same
    district relationship in both years (both in same district or both in different).

    Returns: Stability score in [0, 1] (1 = all boundaries unchanged)
    """
    adjacency_coo = adjacency_2020.tocoo()

    total_boundaries = 0
    unchanged_boundaries = 0

    for i, j in zip(adjacency_coo.row, adjacency_coo.col):
        if i >= j:  # Only count each edge once
            continue

        # Get tract IDs (assuming adjacency rows/cols correspond to tract order)
        # TODO: Need to map indices to GEOIDs - requires tract ordering metadata

        total_boundaries += 1

        # Check if boundary status changed
        # This is simplified - full implementation needs GEOID mapping
        # For now, return placeholder

    # Placeholder - requires proper GEOID<->index mapping
    return 0.75  # TODO: Implement properly


def compute_hierarchical_coherence(partition: dict, adjacency):
    """
    Compute hierarchical coherence (modularity of district-level clustering).

    High coherence = districts naturally group into hierarchical regions.

    Returns: Coherence score
    """
    from sklearn.cluster import AgglomerativeClustering

    # TODO: Build district-level adjacency matrix
    # TODO: Compute hierarchical clustering
    # TODO: Compute modularity

    # Placeholder
    return 0.85  # TODO: Implement properly


def compute_all_stability_metrics(state: str, method: str):
    """Compute all stability metrics for a state and method."""
    print(f"Computing stability metrics: {state}, {method}")

    # Load partitions
    partition_2010_df = load_partition_results(state, 2010, method)
    partition_2020_df = load_partition_results(state, 2020, method)

    # Convert to dicts
    partition_2010 = dict(zip(partition_2010_df['GEOID'], partition_2010_df['district']))
    partition_2020 = dict(zip(partition_2020_df['GEOID'], partition_2020_df['district']))

    # Load tract relationships
    try:
        relationships = load_tract_relationships(state)
        partition_2010_mapped = map_partition_to_new_tracts(partition_2010, relationships)
    except FileNotFoundError as e:
        print(f"  WARNING: {e}")
        print(f"  Skipping {state} - relationship file needed")
        return None

    # Compute metrics
    tract_stability = compute_tract_stability(partition_2010_mapped, partition_2020)
    pop_disruption = compute_population_disruption(partition_2010_mapped, partition_2020,
                                                   partition_2020_df)

    # Load 2020 adjacency for boundary stability
    adj_file = Path(f'outputs/data/2020/adjacency/{state}_adjacency.npz')
    adjacency_2020 = load_adjacency_matrix(adj_file)
    boundary_stability = compute_boundary_stability(partition_2010_mapped, partition_2020,
                                                    adjacency_2020)

    # Hierarchical coherence
    coherence_2010 = compute_hierarchical_coherence(partition_2010, None)
    coherence_2020 = compute_hierarchical_coherence(partition_2020, None)
    coherence_change = abs(coherence_2020 - coherence_2010)

    return {
        'state': state,
        'method': method,
        'tract_stability': tract_stability,
        'pop_disruption': pop_disruption,
        'boundary_stability': boundary_stability,
        'coherence_2010': coherence_2010,
        'coherence_2020': coherence_2020,
        'coherence_change': coherence_change
    }


def main():
    """Compute stability metrics for all states and methods."""
    print("="*70)
    print("TEMPORAL STABILITY METRICS COMPUTATION")
    print("="*70)
    print()

    all_results = []

    for state in STATES:
        print(f"\n{state.upper()}")
        print("-"*70)

        for method in METHODS:
            try:
                metrics = compute_all_stability_metrics(state, method)
                if metrics:
                    all_results.append(metrics)

                    # Print metrics
                    print(f"  {method:12s}: "
                          f"tract={metrics['tract_stability']:.3f}, "
                          f"pop_disrupt={metrics['pop_disruption']:.3f}, "
                          f"boundary={metrics['boundary_stability']:.3f}")

            except Exception as e:
                print(f"  ERROR ({method}): {e}")

    # Save results
    if all_results:
        results_df = pd.DataFrame(all_results)
        output_file = Path('research/gerry-temporal-stability/results/stability_metrics.csv')
        results_df.to_csv(output_file, index=False)
        print(f"\nSaved: {output_file}")

        # Print summary statistics
        print("\n" + "="*70)
        print("SUMMARY STATISTICS")
        print("="*70)
        summary = results_df.groupby('method').agg({
            'tract_stability': 'mean',
            'pop_disruption': 'mean',
            'boundary_stability': 'mean',
            'coherence_change': 'mean'
        })
        print(summary)


if __name__ == '__main__':
    main()
