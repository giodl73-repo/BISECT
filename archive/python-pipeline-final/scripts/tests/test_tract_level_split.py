#!/usr/bin/env python3
"""
Test redistricting at tract level (faster than block level).

Aggregates blocks to census tracts for initial splits, then disaggregates
back to blocks.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / 'src'))

from apportionment.data.census import load_blocks
import numpy as np
import networkx as nx


def aggregate_to_tracts(blocks_gdf):
    """Aggregate blocks to tract level."""
    import geopandas as gpd
    from shapely.ops import unary_union

    # Extract tract from GEOID (first 11 digits: state(2) + county(3) + tract(6))
    blocks_gdf['TRACT_GEOID'] = blocks_gdf['GEOID'].str[:11]

    # Aggregate statistics
    tract_stats = blocks_gdf.groupby('TRACT_GEOID').agg({
        'population': 'sum',
        'ALAND': 'sum',
        'AWATER': 'sum'
    }).reset_index()

    # Dissolve geometries by tract (union all blocks in each tract)
    print("Dissolving block geometries to tracts...")
    tract_geoms = blocks_gdf.dissolve(by='TRACT_GEOID', aggfunc='sum')
    tract_geoms = tract_geoms.reset_index()[['TRACT_GEOID', 'geometry']]

    # Merge statistics with geometries
    tract_gdf = tract_stats.merge(tract_geoms, on='TRACT_GEOID', how='left')
    tract_gdf = gpd.GeoDataFrame(tract_gdf, geometry='geometry', crs=blocks_gdf.crs)

    print(f"Aggregated {len(blocks_gdf):,} blocks to {len(tract_gdf):,} tracts")
    print(f"Average blocks per tract: {len(blocks_gdf) / len(tract_gdf):.1f}")

    return tract_gdf


def build_tract_adjacency(tract_gdf):
    """Build adjacency graph at tract level."""
    from libpysal import weights

    print("Building tract-level adjacency...")
    queen_weights = weights.contiguity.Queen.from_dataframe(
        tract_gdf,
        use_index=False,
        silence_warnings=True
    )

    # Convert to adjacency list
    adjacency = [[] for _ in range(len(tract_gdf))]
    for i in range(len(tract_gdf)):
        if i in queen_weights.neighbors:
            adjacency[i] = list(queen_weights.neighbors[i])

    print(f"Tract graph: {len(adjacency)} nodes, {queen_weights.s0 / 2:.0f} edges")
    print(f"Average neighbors: {queen_weights.mean_neighbors:.1f}")

    return adjacency


def test_first_split_on_tracts():
    """Test first split using tract-level aggregation."""
    from apportionment.partition.metis_wrapper import partition_graph

    # Load blocks
    blocks_file = Path('data/raw/ca_blocks_2020.parquet')
    print("Loading California blocks...")
    blocks_gdf = load_blocks(str(blocks_file))

    # Aggregate to tracts
    tract_gdf = aggregate_to_tracts(blocks_gdf)

    # Build tract adjacency
    adjacency = build_tract_adjacency(tract_gdf)

    # Get vertex weights
    vertex_weights = tract_gdf['population'].values.astype(np.int32)
    total_population = int(vertex_weights.sum())

    num_districts = 52
    ideal_district_pop = total_population / num_districts

    print(f"\n{'=' * 60}")
    print(f"California Redistricting - Tract-Level First Split")
    print(f"{'=' * 60}")
    print(f"Total tracts: {len(tract_gdf):,}")
    print(f"Total population: {total_population:,}")
    print(f"Target districts: {num_districts}")
    print(f"Ideal district population: {ideal_district_pop:,.0f}")

    # First split: 52 -> 26/26
    k = 52
    k_left = k // 2
    k_right = k - k_left

    target_weights = [k_left / k, k_right / k]

    print(f"\nFirst Split Configuration:")
    print(f"  Left partition:  {k_left} districts")
    print(f"  Right partition: {k_right} districts")
    print(f"  Target weights: {target_weights[0]:.4f} / {target_weights[1]:.4f}")

    # Expected populations
    expected_left_pop = total_population * target_weights[0]
    expected_right_pop = total_population * target_weights[1]

    print(f"\nExpected populations:")
    print(f"  Left:  {expected_left_pop:,.0f} -> {k_left} districts × {expected_left_pop/k_left:,.0f} per district")
    print(f"  Right: {expected_right_pop:,.0f} -> {k_right} districts × {expected_right_pop/k_right:,.0f} per district")

    # Perform the split
    print(f"\nCalling METIS to split {len(tract_gdf):,} tracts...")

    try:
        parts = partition_graph(
            adjacency=adjacency,
            vertex_weights=vertex_weights,
            nparts=2,
            target_weights=target_weights,
            recursive=True
        )

        # Analyze results
        left_tracts = [i for i, p in enumerate(parts) if p == 0]
        right_tracts = [i for i, p in enumerate(parts) if p == 1]

        left_pop = sum(vertex_weights[i] for i in left_tracts)
        right_pop = sum(vertex_weights[i] for i in right_tracts)

        print(f"\n{'=' * 60}")
        print(f"METIS Split Results")
        print(f"{'=' * 60}")

        print(f"\nLeft partition:")
        print(f"  Tracts: {len(left_tracts):,}")
        print(f"  Population: {left_pop:,}")
        print(f"  Difference from target: {left_pop - expected_left_pop:+,.0f} ({(left_pop - expected_left_pop) / expected_left_pop * 100:+.2f}%)")
        print(f"  Per district: {left_pop / k_left:,.0f}")

        print(f"\nRight partition:")
        print(f"  Tracts: {len(right_tracts):,}")
        print(f"  Population: {right_pop:,}")
        print(f"  Difference from target: {right_pop - expected_right_pop:+,.0f} ({(right_pop - expected_right_pop) / expected_right_pop * 100:+.2f}%)")
        print(f"  Per district: {right_pop / k_right:,.0f}")

        # Check balance
        left_per_district = left_pop / k_left
        right_per_district = right_pop / k_right
        left_deviation = (left_per_district - ideal_district_pop) / ideal_district_pop * 100
        right_deviation = (right_per_district - ideal_district_pop) / ideal_district_pop * 100

        max_deviation = max(abs(left_deviation), abs(right_deviation))

        print(f"\nBalance Analysis:")
        print(f"  Left deviation:  {left_deviation:+.2f}%")
        print(f"  Right deviation: {right_deviation:+.2f}%")
        print(f"  Max deviation:   {max_deviation:.2f}%")

        if max_deviation < 1:
            print(f"\nOK: Excellent balance!")
        elif max_deviation < 2:
            print(f"\nOK: Good balance")
        else:
            print(f"\nWARNING: Significant imbalance")

        # Save assignments
        print(f"\nSaving assignments...")
        import pickle
        output_dir = Path('outputs/splits')
        output_dir.mkdir(parents=True, exist_ok=True)

        assignments_file = output_dir / 'ca_first_split_assignments.pkl'
        with open(assignments_file, 'wb') as f:
            pickle.dump({i: parts[i] for i in range(len(parts))}, f)
        print(f"  Saved to: {assignments_file}")

        # Generate map
        print(f"\nGenerating map...")
        import matplotlib.pyplot as plt
        import matplotlib.patches as mpatches

        fig, ax = plt.subplots(1, 1, figsize=(14, 12))

        # Plot each partition
        colors = ['#e74c3c', '#3498db']
        tract_gdf['partition'] = parts

        for i, part_id in enumerate([0, 1]):
            partition_data = tract_gdf[tract_gdf['partition'] == part_id].copy()

            partition_data.plot(
                ax=ax,
                color=colors[i],
                edgecolor='white',
                linewidth=0.1,
                alpha=0.7,
                aspect='equal'
            )

        ax.set_axis_off()
        ax.set_title('California First Split: 2 Regions (26 districts each)',
                     fontsize=16, fontweight='bold', pad=20)

        # Legend
        legend_elements = [
            mpatches.Patch(facecolor=colors[0], edgecolor='white',
                          label=f"Region 0: {left_pop:,} people ({len(left_tracts):,} tracts)"),
            mpatches.Patch(facecolor=colors[1], edgecolor='white',
                          label=f"Region 1: {right_pop:,} people ({len(right_tracts):,} tracts)")
        ]

        ax.legend(handles=legend_elements, loc='lower right',
                 frameon=True, fancybox=True, shadow=True, fontsize=11)

        plt.tight_layout()

        map_file = output_dir / 'ca_first_split_map.png'
        print(f"  Saving map to: {map_file}")
        plt.savefig(map_file, dpi=300, bbox_inches='tight')
        print(f"  Map saved!")
        plt.close()

        print(f"\n{'=' * 60}")
        print(f"SUCCESS!")
        print(f"{'=' * 60}")
        print(f"\nFiles saved:")
        print(f"  Assignments: {assignments_file}")
        print(f"  Map: {map_file}")

        return 0

    except Exception as e:
        print(f"\nERROR: Error: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(test_first_split_on_tracts())
