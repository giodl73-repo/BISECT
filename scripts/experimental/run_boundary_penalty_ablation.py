"""
State boundary penalty ablation study for Paper #13.

Sweeps through different penalty values β for crossing state boundaries:
  β = 0:     No penalty (pure national optimization)
  β = small: Weak penalty (prefer within-state but allow crossing)
  β = large: Strong penalty (rarely cross)
  β = ∞:     Never cross (equivalent to state-based)

For each β, applies penalty to cross-state edges and runs redistricting.

Edge weighting: edge_weight(i,j) *= (1 + β) if edge crosses state boundary

Usage:
    python scripts/experimental/run_boundary_penalty_ablation.py --year 2020

Output:
    outputs/experimental/ablation_beta_{beta}_2020.pkl (one per β value)
    outputs/experimental/ablation_summary_2020.pkl (aggregate results)
"""

import argparse
import pickle
import time
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np
import pandas as pd
from tqdm import tqdm

# Add src and scripts to path
import sys
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))
sys.path.insert(0, str(project_root / "scripts"))

from apportionment.partition.recursive_bisection import RecursiveBisection


def load_national_data(input_dir: Path, year: int) -> Tuple[Dict, Dict, Dict]:
    """Load adjacency graph, population, and cross-state edge data."""
    print(f"\nLoading national data...")

    # Load adjacency graph
    adj_file = input_dir / f"national_adjacency_{year}.pkl"
    with open(adj_file, 'rb') as f:
        adj_data = pickle.load(f)

    # Load baseline redistricting (β=0)
    baseline_file = input_dir / f"national_districts_{year}.pkl"
    with open(baseline_file, 'rb') as f:
        baseline_data = pickle.load(f)

    print(f"  Adjacency: {len(adj_data['adjacency']):,} nodes, {len(adj_data['cross_state_edges']):,} cross-state edges")
    print(f"  Population: {baseline_data['total_population']:,} total")

    return adj_data, baseline_data, {}


def apply_boundary_penalty(
    base_edge_weights: Dict[Tuple[int, int], float],
    cross_state_edges: List[Tuple[int, int]],
    beta: float
) -> Dict[Tuple[int, int], float]:
    """
    Apply penalty β to cross-state edges.

    New weight = base_weight × (1 + β) for cross-state edges
    """
    # Convert cross-state edges to set for fast lookup
    cross_state_set = set()
    for (i, j) in cross_state_edges:
        cross_state_set.add((min(i, j), max(i, j)))

    # Apply penalty
    penalized_weights = {}
    for (i, j), weight in base_edge_weights.items():
        key = (min(i, j), max(i, j))
        if key in cross_state_set:
            penalized_weights[(i, j)] = weight * (1 + beta)
        else:
            penalized_weights[(i, j)] = weight

    return penalized_weights


def run_redistricting_with_penalty(
    adjacency: List[List[int]],
    populations: np.ndarray,
    base_edge_weights: Dict[Tuple[int, int], float],
    cross_state_edges: List[Tuple[int, int]],
    beta: float,
    n_districts: int = 435,
    output_dir: Path = None,
    seed: int = None
) -> np.ndarray:
    """
    Run national redistricting with boundary crossing penalty β.
    """
    print(f"\n{'='*70}")
    print(f"Running redistricting with beta = {beta}")
    print(f"{'='*70}")

    # Apply penalty to edge weights
    print(f"  Applying boundary penalty: beta = {beta}")
    edge_weights = apply_boundary_penalty(base_edge_weights, cross_state_edges, beta)

    n_cross_state = len([e for e in edge_weights.items() if e[0] in {(min(i,j), max(i,j)) for i,j in cross_state_edges}])
    print(f"  {n_cross_state:,} cross-state edges penalized by factor {1+beta:.2f}")

    start_time = time.time()

    # Create partitioner
    intermediate_dir = output_dir / f'intermediate_beta_{beta}' if output_dir else None
    if intermediate_dir:
        intermediate_dir.mkdir(parents=True, exist_ok=True)

    partitioner = RecursiveBisection(
        adjacency=adjacency,
        vertex_weights=populations,
        num_districts=n_districts,
        save_intermediate=False,  # Skip for speed
        intermediate_dir=None,
        state_code=f'NATIONAL_BETA_{beta}',
        tqdm_position=0,
        debug=False,
        edge_weights=edge_weights,
        ufactor=5,  # ±0.5%
        niter=100,
        objtype='cut',
        seed=seed,
        vra_mode=False
    )

    # Run the algorithm
    districts = partitioner.partition()

    elapsed = time.time() - start_time
    print(f"  Redistricting complete in {elapsed:.1f}s")

    return districts


def analyze_results(
    districts: np.ndarray,
    populations: np.ndarray,
    geoid_to_index: Dict[str, int],
    index_to_geoid: Dict[int, str],
    cross_state_edges: List[Tuple[int, int]],
    state_abbrs: List[str],
    beta: float
) -> Dict:
    """
    Analyze redistricting results for given β.

    Returns:
        Dictionary with metrics (compactness, cross-state districts, etc.)
    """
    print(f"  Analyzing results for beta = {beta}...")

    # Count cross-state districts
    cross_state_districts = set()
    for (i, j) in cross_state_edges:
        if districts[i] == districts[j]:
            cross_state_districts.add(int(districts[i]))

    n_cross_state = len(cross_state_districts)
    pct_cross_state = n_cross_state / 435 * 100

    # Compute district populations
    district_pops = []
    for d in range(435):
        mask = (districts == d)
        pop = populations[mask].sum()
        district_pops.append(pop)

    district_pops = np.array(district_pops)
    target_pop = populations.sum() / 435

    # Population balance
    deviations = np.abs(district_pops - target_pop) / target_pop * 100
    max_deviation = deviations.max()
    mean_deviation = deviations.mean()

    results = {
        'beta': beta,
        'n_cross_state_districts': n_cross_state,
        'pct_cross_state_districts': pct_cross_state,
        'max_pop_deviation_pct': max_deviation,
        'mean_pop_deviation_pct': mean_deviation,
        'district_pops': district_pops.tolist()
    }

    print(f"    Cross-state districts: {n_cross_state} ({pct_cross_state:.1f}%)")
    print(f"    Population balance: ±{max_deviation:.2f}% max")

    return results


def main():
    parser = argparse.ArgumentParser(description="State boundary penalty ablation study")
    parser.add_argument('--year', type=int, default=2020, help='Census year')
    parser.add_argument('--data-dir', type=Path, default=Path('data'), help='Data directory')
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/experimental'), help='Input directory')
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/experimental'), help='Output directory')
    parser.add_argument('--betas', type=str, default='0,0.1,0.5,1,2,5,10,20,50,100', help='Comma-separated β values')
    parser.add_argument('--seed', type=int, default=42, help='Random seed')
    args = parser.parse_args()

    # Parse beta values
    beta_values = [float(b) for b in args.betas.split(',')]

    print("="*70)
    print("State Boundary Penalty Ablation Study (Paper #13)")
    print("="*70)
    print(f"Year: {args.year}")
    print(f"Beta values: {beta_values}")
    print(f"Seed: {args.seed}")
    print("="*70)

    # Load national data
    adj_data, baseline_data, _ = load_national_data(args.input_dir, args.year)

    adjacency = adj_data['adjacency']
    cross_state_edges = adj_data['cross_state_edges']
    geoid_to_index = adj_data['geoid_to_index']
    index_to_geoid = adj_data['index_to_geoid']
    populations = baseline_data['populations']
    state_abbrs = adj_data['state_abbrs']

    # Base edge weights (uniform)
    base_edge_weights = {}
    for i in range(len(adjacency)):
        for j in adjacency[i]:
            if j > i:
                base_edge_weights[(i, j)] = 1.0

    # Run ablation study
    all_results = []

    for i, beta in enumerate(beta_values):
        print(f"\n{'='*70}")
        print(f"Beta sweep: {i+1}/{len(beta_values)} (beta={beta})")
        print(f"{'='*70}")

        try:
            # Run redistricting
            districts = run_redistricting_with_penalty(
                adjacency=adjacency,
                populations=populations,
                base_edge_weights=base_edge_weights,
                cross_state_edges=cross_state_edges,
                beta=beta,
                n_districts=435,
                output_dir=args.output_dir,
                seed=args.seed
            )

            # Analyze results
            results = analyze_results(
                districts=districts,
                populations=populations,
                geoid_to_index=geoid_to_index,
                index_to_geoid=index_to_geoid,
                cross_state_edges=cross_state_edges,
                state_abbrs=state_abbrs,
                beta=beta
            )

            # Save individual result
            beta_file = args.output_dir / f"ablation_beta_{beta}_{args.year}.pkl"
            with open(beta_file, 'wb') as f:
                pickle.dump({
                    'beta': beta,
                    'districts': districts,
                    'results': results,
                    'geoid_to_district': {index_to_geoid[i]: int(districts[i]) for i in range(len(districts))}
                }, f)

            print(f"  [OK] Saved results for beta={beta}")
            all_results.append(results)

        except Exception as e:
            print(f"  [ERROR] Failed for beta={beta}: {e}")
            import traceback
            traceback.print_exc()
            print(f"  Continuing with next beta value...")
            continue

    # Save summary
    summary_file = args.output_dir / f"ablation_summary_{args.year}.pkl"
    summary_df = pd.DataFrame(all_results)

    with open(summary_file, 'wb') as f:
        pickle.dump({
            'summary': summary_df.to_dict('records'),
            'beta_values': beta_values,
            'year': args.year
        }, f)

    # Display summary
    print("\n" + "="*70)
    print("Ablation Study Complete")
    print("="*70)
    print("\nSummary:")
    print(summary_df[['beta', 'n_cross_state_districts', 'pct_cross_state_districts', 'max_pop_deviation_pct']])

    print(f"\nOutputs:")
    print(f"  Individual results: ablation_beta_{{beta}}_{args.year}.pkl")
    print(f"  Summary: {summary_file}")

    print(f"\nNext: Run compactness analysis for each β value")


if __name__ == '__main__':
    main()
