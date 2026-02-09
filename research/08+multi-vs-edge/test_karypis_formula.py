"""
Test different tpwgts formulas to see which works best.

Comparing:
1. "Paper's wrong formula" (what Karypis criticized): t_i = 0.60 / k
2. "My correct formula" (what I implemented): t_i = 0.60 / (k * m_state)
3. Even higher target to account for METIS undershooting: t_i = 0.80 / (k * m_state)
"""

import sys
from pathlib import Path
import numpy as np
import pandas as pd
import geopandas as gpd

project_root = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(project_root / 'src'))
sys.path.insert(0, str(project_root))

from apportionment.data.adjacency import build_adjacency_graph
from apportionment.partition.metis_executable import partition_graph_with_executable


def create_vertex_weights(state_gdf, state_code):
    """Create 2D vertex weights: [population, minority_vap]"""
    vertex_weights_2d = []
    for _, row in state_gdf.iterrows():
        pop = row['P1_001N']  # Total population
        minority_vap = row.get('minority_vap', 0)
        vertex_weights_2d.append([pop, minority_vap])
    return vertex_weights_2d


def test_formula(state_gdf, state_code, k, target_mm, formula_name, tpwgts, ubvec):
    """Run METIS with specific tpwgts and return results"""

    print(f"\n{'='*70}")
    print(f"Testing: {formula_name}")
    print(f"  tpwgts for MM districts: {tpwgts[0][1]:.4f}")
    print(f"  tpwgts for other districts: {tpwgts[target_mm][1]:.4f}")
    print(f"{'='*70}")

    # Build adjacency
    adjacency_list = build_adjacency_graph(
        state_gdf,
        state_code,
        mode='normal',
        water_adjacency=True
    )

    # Create vertex weights
    vertex_weights_2d = create_vertex_weights(state_gdf, state_code)

    # Run METIS
    tpwgts_list = tpwgts.tolist()

    print(f"\nRunning METIS with ubvec={ubvec}...")
    partition = partition_graph_with_executable(
        adjacency_list,
        vertex_weights_2d,
        nparts=k,
        target_weights=tpwgts_list,
        ubvec=ubvec,
        niter=100,
        debug=True
    )

    # Analyze results
    state_gdf['district'] = partition

    district_stats = []
    for dist in range(k):
        dist_gdf = state_gdf[state_gdf['district'] == dist]
        total_pop = dist_gdf['P1_001N'].sum()
        total_minority = dist_gdf['minority_vap'].sum()
        minority_pct = total_minority / total_pop if total_pop > 0 else 0
        district_stats.append({
            'district': dist,
            'population': total_pop,
            'minority_vap': total_minority,
            'minority_pct': minority_pct,
            'is_mm': minority_pct >= 0.5
        })

    df_stats = pd.DataFrame(district_stats)
    df_stats = df_stats.sort_values('minority_pct', ascending=False)

    mm_count = (df_stats['minority_pct'] >= 0.5).sum()
    max_minority = df_stats['minority_pct'].max()

    print(f"\nResults:")
    print(f"  MM districts achieved: {mm_count}/{target_mm}")
    print(f"  Max minority %: {max_minority:.1%}")
    print(f"  Success: {mm_count >= target_mm}")
    print(f"\n  District minority percentages (sorted):")
    for _, row in df_stats.head(k).iterrows():
        mm_marker = " [MM]" if row['is_mm'] else ""
        print(f"    District {row['district']}: {row['minority_pct']:.1%}{mm_marker}")

    return {
        'formula': formula_name,
        'mm_count': mm_count,
        'max_minority_pct': max_minority,
        'success': mm_count >= target_mm,
        'district_pcts': df_stats['minority_pct'].tolist()
    }


def main():
    print("="*70)
    print("TESTING DIFFERENT TPWGTS FORMULAS")
    print("="*70)

    # Load Alabama data
    state_code = 'AL'
    k = 7
    target_mm = 2

    state_gdf = gpd.read_file(f'outputs/data/2020/units/alabama_2020_tracts.geojson')

    # Add minority_vap if not present
    if 'minority_vap' not in state_gdf.columns:
        # Calculate from census columns
        white_alone_vap = state_gdf.get('P3_003N', 0)
        total_vap = state_gdf.get('P3_001N', 0)
        state_gdf['minority_vap'] = total_vap - white_alone_vap

    total_pop = state_gdf['P1_001N'].sum()
    total_minority = state_gdf['minority_vap'].sum()
    m_state = total_minority / total_pop

    print(f"\nAlabama Stats:")
    print(f"  Districts (k): {k}")
    print(f"  Target MM: {target_mm}")
    print(f"  Total population: {total_pop:,}")
    print(f"  Total minority VAP: {total_minority:,}")
    print(f"  State minority %: {m_state:.1%}")
    print()

    ubvec = [1.005, 1.5]  # Use moderate tolerance

    results = []

    # Formula 1: "Paper's wrong formula" - what Karypis criticized
    print("\n" + "="*70)
    print("FORMULA 1: Paper's Wrong Formula")
    print("  t_i^min = 0.60 / k (concentration * population_weight)")
    print("="*70)

    tpwgts_wrong = np.zeros((k, 2))
    tpwgts_wrong[:, 0] = 1.0 / k  # Population: equal

    # MM districts get 0.60/k each
    for i in range(target_mm):
        tpwgts_wrong[i, 1] = 0.60 / k

    # Other districts share remainder
    remaining = 1.0 - (target_mm * 0.60 / k)
    for i in range(target_mm, k):
        tpwgts_wrong[i, 1] = remaining / (k - target_mm)

    print(f"\n  Within-district minority target:")
    within_target_wrong = (0.60 / k) * total_minority / (total_pop / k)
    print(f"    MM districts: {within_target_wrong:.1%}")

    result1 = test_formula(state_gdf.copy(), state_code, k, target_mm,
                          "Paper's Wrong Formula", tpwgts_wrong, ubvec)
    results.append(result1)

    # Formula 2: "My correct formula"
    print("\n" + "="*70)
    print("FORMULA 2: My Correct Formula")
    print("  t_i^min = 0.60 / (k * m_state)")
    print("="*70)

    tpwgts_correct = np.zeros((k, 2))
    tpwgts_correct[:, 0] = 1.0 / k

    target_mm_fraction = 0.60 / (k * m_state)

    for i in range(target_mm):
        tpwgts_correct[i, 1] = target_mm_fraction

    remaining = 1.0 - (target_mm * target_mm_fraction)
    for i in range(target_mm, k):
        tpwgts_correct[i, 1] = remaining / (k - target_mm)

    print(f"\n  Within-district minority target:")
    within_target_correct = target_mm_fraction * total_minority / (total_pop / k)
    print(f"    MM districts: {within_target_correct:.1%}")

    result2 = test_formula(state_gdf.copy(), state_code, k, target_mm,
                          "My Correct Formula (0.60 target)", tpwgts_correct, ubvec)
    results.append(result2)

    # Formula 3: Higher target (0.80) to account for METIS undershooting
    print("\n" + "="*70)
    print("FORMULA 3: Higher Target to Compensate")
    print("  t_i^min = 0.80 / (k * m_state)")
    print("="*70)

    tpwgts_high = np.zeros((k, 2))
    tpwgts_high[:, 0] = 1.0 / k

    target_mm_fraction_high = 0.80 / (k * m_state)

    for i in range(target_mm):
        tpwgts_high[i, 1] = target_mm_fraction_high

    remaining = 1.0 - (target_mm * target_mm_fraction_high)
    for i in range(target_mm, k):
        tpwgts_high[i, 1] = remaining / (k - target_mm)

    print(f"\n  Within-district minority target:")
    within_target_high = target_mm_fraction_high * total_minority / (total_pop / k)
    print(f"    MM districts: {within_target_high:.1%}")

    result3 = test_formula(state_gdf.copy(), state_code, k, target_mm,
                          "Higher Target (0.80)", tpwgts_high, ubvec)
    results.append(result3)

    # Summary
    print("\n" + "="*70)
    print("SUMMARY COMPARISON")
    print("="*70)
    print()

    df_results = pd.DataFrame(results)
    print(df_results[['formula', 'mm_count', 'max_minority_pct', 'success']].to_string(index=False))

    print("\n" + "="*70)
    print("CONCLUSION:")
    print("="*70)

    best = df_results.loc[df_results['mm_count'].idxmax()]
    print(f"\nBest performing formula: {best['formula']}")
    print(f"  MM districts: {best['mm_count']}/{target_mm}")
    print(f"  Max minority: {best['max_minority_pct']:.1%}")


if __name__ == '__main__':
    main()
