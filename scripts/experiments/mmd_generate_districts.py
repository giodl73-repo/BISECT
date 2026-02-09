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

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

from apportionment.partition.recursive_bisection import partition_graph
from apportionment.data.adjacency import load_adjacency_graph


def calculate_districts_for_members(total_reps: int, members_per_district: int) -> int:
    """Calculate number of districts needed for given members per district."""
    return (total_reps + members_per_district - 1) // members_per_district  # Ceiling division


def generate_mmd_configuration(year: int, members_per_district: int, output_dir: Path):
    """Generate a single MMD configuration."""
    print(f"\n[MMD Generation] {members_per_district}-member configuration")
    print("=" * 70)

    # Calculate target districts
    total_reps = 435
    num_districts = calculate_districts_for_members(total_reps, members_per_district)
    actual_total = num_districts * members_per_district

    print(f"Target: {num_districts} districts × {members_per_district} reps = {actual_total} total")
    if actual_total != 435:
        print(f"WARNING: Actual total ({actual_total}) differs from 435")

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

    # Load adjacency graph
    print("\nLoading adjacency graph...")
    adj_path = Path(f"outputs/data/{year}/adjacency/tract_adjacency.parquet")

    if not adj_path.exists():
        raise FileNotFoundError(
            f"Adjacency graph not found: {adj_path}\n"
            f"Run adjacency generation first"
        )

    graph = load_adjacency_graph(adj_path)
    print(f"Loaded graph with {graph.number_of_nodes():,} nodes, {graph.number_of_edges():,} edges")

    # Calculate population target
    total_population = tracts['population'].sum()
    target_pop_per_district = total_population / num_districts
    tolerance = 0.005  # ±0.5%

    print(f"\nPopulation per district: {target_pop_per_district:,.0f}")
    print(f"Tolerance: ±{tolerance*100}% ({target_pop_per_district*tolerance:,.0f})")

    # Run recursive bisection
    print(f"\nRunning recursive bisection for {num_districts} districts...")
    print("(This may take 20-30 minutes)")

    assignments = partition_graph(
        graph=graph,
        populations=tracts.set_index('geoid')['population'].to_dict(),
        num_districts=num_districts,
        population_tolerance=tolerance,
        compactness_weight=1.0
    )

    # Convert to dataframe
    districts_df = pd.DataFrame([
        {'geoid': geoid, 'district': district_id}
        for geoid, district_id in assignments.items()
    ])

    # Validate
    tract_districts = tracts.merge(districts_df, on='geoid')
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
                        choices=[3, 4, 5],
                        help='Members per district to generate (e.g., --members 3 4 5)')
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
