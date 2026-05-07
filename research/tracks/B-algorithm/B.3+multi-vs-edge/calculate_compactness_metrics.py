"""
Calculate Polsby-Popper and Reock compactness scores for best configurations
"""

import geopandas as gpd
import pandas as pd
import numpy as np
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

def polsby_popper_score(geometry):
    """
    Polsby-Popper score: 4π * Area / Perimeter²

    Perfect circle = 1.0, lower values = less compact
    """
    area = geometry.area
    perimeter = geometry.length
    if perimeter == 0:
        return 0
    return (4 * np.pi * area) / (perimeter ** 2)

def reock_score(geometry):
    """
    Reock score: Area / Area of minimum bounding circle

    Perfect circle = 1.0, lower values = less compact
    """
    area = geometry.area
    # Minimum bounding circle (approximate using bounds)
    minx, miny, maxx, maxy = geometry.bounds
    # Center of bounding box
    center_x = (minx + maxx) / 2
    center_y = (miny + maxy) / 2
    # Maximum distance from center to any point
    radius = max(
        np.sqrt((minx - center_x)**2 + (miny - center_y)**2),
        np.sqrt((maxx - center_x)**2 + (miny - center_y)**2),
        np.sqrt((minx - center_x)**2 + (maxy - center_y)**2),
        np.sqrt((maxx - center_x)**2 + (maxy - center_y)**2)
    )
    circle_area = np.pi * (radius ** 2)
    if circle_area == 0:
        return 0
    return area / circle_area

def load_best_results():
    """Load best results from CSVs"""
    multi_df = pd.read_csv("research/gerry-multi-vs-edge/results/multi_constraint_results.csv")
    edge_df = pd.read_csv("research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv")

    # Best configuration per state
    best_configs = []

    states = ['AL', 'GA', 'LA', 'MS', 'SC']
    state_names = {
        'AL': 'Alabama',
        'GA': 'Georgia',
        'LA': 'Louisiana',
        'MS': 'Mississippi',
        'SC': 'South Carolina'
    }

    for state in states:
        # Multi-constraint best
        multi_state = multi_df[multi_df['state'] == state].sort_values(['mm_count', 'max_minority_pct'], ascending=False)
        multi_best = multi_state.iloc[0]

        best_configs.append({
            'state': state,
            'state_name': state_names[state],
            'method': 'multi',
            'ubvec_minority': multi_best['ubvec_minority'],
            'mm_count': multi_best['mm_count'],
            'max_minority_pct': multi_best['max_minority_pct'],
            'edge_cut': multi_best['edge_cut']
        })

        # Edge-weighted best
        edge_state = edge_df[edge_df['state'] == state].sort_values(['mm_count', 'max_minority_pct'], ascending=False)
        edge_best = edge_state.iloc[0]

        best_configs.append({
            'state': state,
            'state_name': state_names[state],
            'method': 'edge',
            'weight_factor': edge_best['weight_factor'],
            'threshold': edge_best['minority_threshold'],
            'mm_count': edge_best['mm_count'],
            'max_minority_pct': edge_best['max_minority_pct'],
            'edge_cut': edge_best['edge_cut_unweighted']
        })

    return pd.DataFrame(best_configs)

def calculate_district_compactness(state, partition, tracts_gdf):
    """Calculate compactness scores for each district"""
    tracts_gdf = tracts_gdf.copy()
    tracts_gdf['district'] = partition

    # Dissolve tracts by district to get district polygons
    districts = tracts_gdf.dissolve(by='district')

    compactness = []
    for district_id in sorted(districts.index):
        geom = districts.loc[district_id, 'geometry']

        pp_score = polsby_popper_score(geom)
        reock_score_val = reock_score(geom)

        compactness.append({
            'state': state,
            'district': district_id,
            'polsby_popper': pp_score,
            'reock': reock_score_val,
            'area_km2': geom.area / 1e6,  # Convert to km²
            'perimeter_km': geom.length / 1e3  # Convert to km
        })

    return pd.DataFrame(compactness)

def run_best_partitions_and_measure():
    """Run best partitions and measure compactness"""
    from apportionment.partition.metis_executable import partition_graph_with_executable
    from apportionment.data.adjacency import build_adjacency_graph

    results = []

    for state_code in ['AL', 'GA', 'LA', 'MS', 'SC']:
        state_name = {
            'AL': 'alabama',
            'GA': 'georgia',
            'LA': 'louisiana',
            'MS': 'mississippi',
            'SC': 'south_carolina'
        }[state_code]

        print(f"\nProcessing {state_name.title()}...")

        # Load data
        tracts_path = Path(f"outputs/data/2020/units/{state_name}_2020_tracts.geojson")
        if not tracts_path.exists():
            print(f"  Skipping {state_name} - no tract data found")
            continue

        tracts = gpd.read_file(tracts_path)
        demo_path = Path(f"outputs/data/2020/demographics/{state_name}_demographics.csv")
        demo = pd.read_csv(demo_path)
        tracts = tracts.merge(demo, left_on='GEOID', right_on='geoid', how='left')

        # Get district count
        from scripts.config_2020 import STATE_SEATS
        k = STATE_SEATS[state_name.upper()]

        # Multi-constraint best (from CSV)
        best_configs = load_best_results()
        multi_config = best_configs[(best_configs['state'] == state_code) &
                                    (best_configs['method'] == 'multi')].iloc[0]

        print(f"  [1/2] Multi-constraint (ubvec={multi_config['ubvec_minority']})...")
        adj_list = build_adjacency_graph(tracts)

        # 2D weights
        vertex_weights = [[int(row['total_pop']), int(row['minority_vap'])]
                          for _, row in tracts.iterrows()]

        # Target weights for MM districts
        total_pop = tracts['total_pop'].sum()
        total_minority = tracts['minority_vap'].sum()
        target_mm = int(multi_config['mm_count'])  # Use actual target from results

        tpwgts = []
        for i in range(k):
            if i < target_mm:
                mm_frac = 0.60 * (total_minority / k) / total_minority
            else:
                mm_frac = (total_minority - target_mm * 0.60 * (total_minority / k)) / ((k - target_mm) * total_minority)
            tpwgts.extend([1/k, mm_frac])

        partition_multi = partition_graph_with_executable(
            adj_list,
            vertex_weights,
            nparts=k,
            tpwgts=tpwgts,
            ubvec=[1.005, multi_config['ubvec_minority']],
            niter=100,
            seed=42
        )

        # Calculate compactness
        compact_multi = calculate_district_compactness(state_code, partition_multi, tracts)
        compact_multi['method'] = 'Multi-Constraint'
        compact_multi['config'] = f"ubvec={multi_config['ubvec_minority']}"

        # Edge-weighted best
        edge_config = best_configs[(best_configs['state'] == state_code) &
                                   (best_configs['method'] == 'edge')].iloc[0]

        print(f"  [2/2] Edge-weighted ({edge_config['weight_factor']}x @ {edge_config['threshold']*100:.0f}%)...")
        adj_list_edge = build_adjacency_graph(tracts, mode='edge_weighted',
                                             minority_threshold=edge_config['threshold'],
                                             weight_factor=edge_config['weight_factor'])

        vertex_weights_1d = [int(row['total_pop']) for _, row in tracts.iterrows()]

        partition_edge = partition_graph_with_executable(
            adj_list_edge,
            vertex_weights_1d,
            nparts=k,
            ufactor=1.005,
            niter=100,
            seed=42
        )

        # Calculate compactness
        compact_edge = calculate_district_compactness(state_code, partition_edge, tracts)
        compact_edge['method'] = 'Edge-Weighted'
        compact_edge['config'] = f"{edge_config['weight_factor']}x @ {edge_config['threshold']*100:.0f}%"

        results.append(compact_multi)
        results.append(compact_edge)

    return pd.concat(results, ignore_index=True)

def main():
    print("\n" + "="*70)
    print("Calculating Compactness Metrics (Polsby-Popper & Reock)")
    print("="*70)

    # Run partitions and measure compactness
    compactness_df = run_best_partitions_and_measure()

    # Save results
    output_path = Path("research/gerry-multi-vs-edge/results/compactness_metrics.csv")
    compactness_df.to_csv(output_path, index=False)
    print(f"\n[OK] Saved: {output_path}")

    # Calculate summary statistics
    print("\n" + "="*70)
    print("SUMMARY STATISTICS")
    print("="*70)

    summary = compactness_df.groupby(['state', 'method']).agg({
        'polsby_popper': 'mean',
        'reock': 'mean'
    }).reset_index()

    print("\nAverage Compactness Scores by State:")
    print(summary.to_string(index=False))

    # Overall averages
    print("\nOverall Averages:")
    overall = compactness_df.groupby('method').agg({
        'polsby_popper': 'mean',
        'reock': 'mean'
    })
    print(overall.to_string())

    # Percent difference
    pp_multi = overall.loc['Multi-Constraint', 'polsby_popper']
    pp_edge = overall.loc['Edge-Weighted', 'polsby_popper']
    pp_diff = ((pp_edge - pp_multi) / pp_multi) * 100

    reock_multi = overall.loc['Multi-Constraint', 'reock']
    reock_edge = overall.loc['Edge-Weighted', 'reock']
    reock_diff = ((reock_edge - reock_multi) / reock_multi) * 100

    print(f"\nPolsby-Popper: Edge-weighted {pp_diff:+.1f}% vs multi-constraint")
    print(f"Reock: Edge-weighted {reock_diff:+.1f}% vs multi-constraint")

    print("\n" + "="*70)
    print("DONE!")
    print("="*70)

if __name__ == '__main__':
    main()
