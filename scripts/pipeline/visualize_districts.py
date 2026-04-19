#!/usr/bin/env python3
"""
Visualize redistricting results as colored district maps.

Example usage:
    python scripts/visualize_districts.py --state CA --num-districts 52
    python scripts/visualize_districts.py --state CA --num-districts 52 --show-population
"""

import argparse
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))

from apportionment.data.io import load_results
from apportionment.visualization.maps import plot_districts, export_district_stats


def main():
    parser = argparse.ArgumentParser(
        description='Visualize redistricting results'
    )
    parser.add_argument(
        '--state',
        type=str,
        required=True,
        help='State code (e.g., CA for California)'
    )
    parser.add_argument(
        '--num-districts',
        type=int,
        required=True,
        help='Number of districts'
    )
    parser.add_argument(
        '--results-dir',
        type=str,
        default='data/results',
        help='Directory containing results (default: ./data/results)'
    )
    parser.add_argument(
        '--output-dir',
        type=str,
        default='outputs/maps',
        help='Output directory for maps (default: ./outputs/maps)'
    )
    parser.add_argument(
        '--show-population',
        action='store_true',
        help='Show population in district labels'
    )
    parser.add_argument(
        '--no-labels',
        action='store_true',
        help='Hide district ID labels'
    )
    parser.add_argument(
        '--dpi',
        type=int,
        default=300,
        help='Image resolution (default: 300)'
    )

    args = parser.parse_args()

    print(f"\n{'=' * 60}")
    print(f"District Map Visualization")
    print(f"{'=' * 60}\n")

    # Load results
    result_dir = Path(args.results_dir) / f"{args.state.lower()}_{args.num_districts}_districts"

    if not result_dir.exists():
        print(f"Error: Results directory not found at {result_dir}")
        print(f"Run redistrict_ca.py first.")
        return 1

    try:
        assignments, districts_gdf, metadata = load_results(str(result_dir))

        # Print summary
        print(f"\nDistrict Summary:")
        print(f"  State: {metadata['state']}")
        print(f"  Number of districts: {metadata['num_districts']}")
        print(f"  Total population: {metadata['total_population']:,}")
        print(f"  Ideal district population: {metadata['ideal_district_population']:,}")
        print(f"  Mean population deviation: ±{metadata['summary']['mean_population_deviation_pct']:.2f}%")
        print(f"  Max population deviation: ±{metadata['summary']['max_population_deviation_pct']:.2f}%")
        print(f"  Mean compactness (Polsby-Popper): {metadata['summary']['mean_polsby_popper']:.4f}")

        # Generate map
        output_dir = Path(args.output_dir)
        output_file = output_dir / f"{args.state.lower()}_{args.num_districts}_districts.png"

        title = f"{args.state.upper()} {args.num_districts} Congressional Districts\nRecursive Bifurcation Algorithm"

        fig, ax = plot_districts(
            districts_gdf=districts_gdf,
            output_path=str(output_file),
            title=title,
            show_labels=not args.no_labels,
            show_population=args.show_population,
            dpi=args.dpi
        )

        # Export statistics
        stats_file = output_dir / f"{args.state.lower()}_{args.num_districts}_districts_stats.csv"
        export_district_stats(districts_gdf, str(stats_file))

        print(f"\n{'=' * 60}")
        print(f"Visualization Complete!")
        print(f"{'=' * 60}\n")
        print(f"Map saved to: {output_file}")
        print(f"Statistics saved to: {stats_file}")

    except Exception as e:
        print(f"\nError during visualization: {e}")
        import traceback
        traceback.print_exc()
        return 1

    return 0


if __name__ == '__main__':
    sys.exit(main())
