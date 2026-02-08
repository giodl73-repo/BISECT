"""
Compute Partisan Bias for Algorithmic and Enacted Maps

This script calculates partisan bias (seat share - vote share) for both
algorithmic and enacted congressional district maps across all 50 states.

Partisan Bias = (Democratic seat%) - (Statewide Democratic vote%)

Positive bias = Democrats advantaged
Negative bias = Democrats disadvantaged

Author: Claude
Date: 2026-02-07
"""

import json
from pathlib import Path
import pandas as pd
import numpy as np
from typing import Dict, Tuple
import sys

# Add project root to path
project_root = Path(__file__).parents[3]
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(project_root / "src"))

from scripts.config_2020 import STATE_CONFIG_2020
from scripts.config.download_sources import STATE_NAMES

# Create STATES_2020 dict in expected format (lowercase_underscore_name -> districts)
STATES_2020 = {
    STATE_NAMES[abbrev]: config['districts']
    for abbrev, config in STATE_CONFIG_2020.items()
}


def load_algorithmic_results(state_name: str, year: int = 2020) -> Dict:
    """Load algorithmic redistricting results for a state."""
    results_dir = Path(f"outputs/v1/{year}/states/{state_name}/data")

    # Try loading district summary
    summary_file = results_dir / "district_summary.csv"
    if not summary_file.exists():
        return None

    df = pd.read_csv(summary_file)
    return df


def load_enacted_results(state_name: str, year: int = 2020) -> pd.DataFrame:
    """Load enacted district results for a state."""
    # This would load from pre-computed enacted district analysis
    # For now, we'll use placeholder since this may not be computed yet
    enacted_file = Path(f"data/{year}/enacted_districts/{state_name}_enacted.csv")

    if enacted_file.exists():
        return pd.read_csv(enacted_file)
    return None


def compute_district_winners_simple(merged_df: pd.DataFrame) -> pd.DataFrame:
    """
    Aggregate tract-level votes by district and determine winners.

    merged_df should have: district, biden_votes, trump_votes

    Returns DataFrame with: district, dem_votes, rep_votes, winner
    """
    # Aggregate by district
    district_votes = merged_df.groupby('district').agg({
        'biden_votes': 'sum',
        'trump_votes': 'sum'
    }).reset_index()

    # Rename for consistency
    district_votes = district_votes.rename(columns={
        'biden_votes': 'dem_votes',
        'trump_votes': 'rep_votes'
    })

    # Determine winner
    district_votes['total_votes'] = (
        district_votes['dem_votes'] + district_votes['rep_votes']
    )
    district_votes['dem_vote_share'] = (
        district_votes['dem_votes'] / district_votes['total_votes']
    )
    district_votes['winner'] = district_votes['dem_vote_share'].apply(
        lambda x: 'D' if x > 0.5 else 'R'
    )

    return district_votes


def compute_state_partisan_bias(
    district_results: pd.DataFrame,
    statewide_dem_share: float
) -> Tuple[float, float, int, int]:
    """
    Compute partisan bias for a state.

    Returns:
        (partisan_bias, dem_seat_share, dem_seats, total_districts)
    """
    total_districts = len(district_results)
    dem_seats = (district_results['winner'] == 'D').sum()
    dem_seat_share = dem_seats / total_districts

    partisan_bias = dem_seat_share - statewide_dem_share

    return partisan_bias, dem_seat_share, dem_seats, total_districts


def load_prepared_tract_data(state_name: str) -> pd.DataFrame:
    """Load prepared tract data for a state."""
    data_dir = Path("research/gerry-recursive-bisection/data/geographic_sorting/tracts")
    filepath = data_dir / f"{state_name}_tracts_prepared.gpkg"

    if not filepath.exists():
        return None

    import geopandas as gpd
    gdf = gpd.read_file(filepath)

    # Convert to DataFrame (drop geometry for election analysis)
    df = pd.DataFrame(gdf.drop(columns='geometry'))
    return df


def analyze_state(state_name: str, year: int = 2020) -> Dict:
    """Analyze partisan bias for a single state."""
    # Load prepared tract data (has election results)
    tract_df = load_prepared_tract_data(state_name)
    if tract_df is None:
        return None

    # Compute statewide Democratic vote share
    statewide_dem_votes = tract_df['biden_votes'].sum()
    statewide_rep_votes = tract_df['trump_votes'].sum()
    statewide_total = statewide_dem_votes + statewide_rep_votes

    if statewide_total == 0:
        return None

    statewide_dem_share = statewide_dem_votes / statewide_total

    # Load algorithmic district assignments
    algo_file = Path(f"outputs/v1/{year}/states/{state_name}/data/final_assignments.pkl")
    if not algo_file.exists():
        return None

    import pickle
    with open(algo_file, 'rb') as f:
        algo_assignments = pickle.load(f)

    # Convert to DataFrame if needed
    if isinstance(algo_assignments, dict):
        algo_df = pd.DataFrame(list(algo_assignments.items()),
                              columns=['tract_geoid', 'district'])
    else:
        algo_df = algo_assignments
        if 'tract_id' in algo_df.columns:
            algo_df = algo_df.rename(columns={'tract_id': 'tract_geoid'})

    # Merge tract data with district assignments
    merged = tract_df.merge(algo_df, on='tract_geoid', how='inner')

    # Compute algorithmic district winners
    algo_district_results = compute_district_winners_simple(merged)

    # Compute algorithmic partisan bias
    algo_bias, algo_dem_seat_share, algo_dem_seats, n_districts = (
        compute_state_partisan_bias(algo_district_results, statewide_dem_share)
    )

    # Try to load and compute enacted bias
    enacted_bias = None
    enacted_dem_seat_share = None
    enacted_dem_seats = None

    # For now, we'll leave enacted as None since we may not have pre-computed this
    # This can be filled in later with actual enacted district data

    results = {
        'state': state_name,
        'n_districts': n_districts,
        'statewide_dem_vote_pct': statewide_dem_share * 100,
        'algo_dem_seat_pct': algo_dem_seat_share * 100,
        'algo_dem_seats': algo_dem_seats,
        'algo_partisan_bias': algo_bias * 100,  # Convert to percentage points
        'enacted_dem_seat_pct': enacted_dem_seat_share * 100 if enacted_dem_seat_share else None,
        'enacted_dem_seats': enacted_dem_seats,
        'enacted_partisan_bias': enacted_bias * 100 if enacted_bias else None,
        'bias_difference': None  # (enacted_bias - algo_bias) if both available
    }

    return results


def analyze_all_states(year: int = 2020) -> pd.DataFrame:
    """Analyze partisan bias for all 50 states."""
    results = []

    for state_name, n_districts in STATES_2020.items():
        print(f"Processing {state_name}...")

        # Skip single-district states
        if n_districts == 1:
            print(f"  Skipping {state_name} (single district)")
            continue

        try:
            state_results = analyze_state(state_name, year)

            if state_results is None:
                print(f"  Skipping {state_name} (missing data)")
                continue

            results.append(state_results)

            print(f"  [OK] {state_name}: Statewide D = {state_results['statewide_dem_vote_pct']:.1f}%, "
                  f"Algo D seats = {state_results['algo_dem_seat_pct']:.1f}%, "
                  f"Bias = {state_results['algo_partisan_bias']:+.1f} pp")

        except Exception as e:
            print(f"  Error processing {state_name}: {e}")
            import traceback
            traceback.print_exc()
            continue

    # Create DataFrame
    df = pd.DataFrame(results)

    # Sort by algorithmic partisan bias (descending)
    df = df.sort_values('algo_partisan_bias', ascending=False)

    return df


def main():
    """Main execution."""
    print("="*60)
    print("Computing Partisan Bias for All 50 States")
    print("="*60)
    print()

    # Analyze all states
    results_df = analyze_all_states(year=2020)

    # Save results
    output_dir = Path("research/gerry-recursive-bisection/data/geographic_sorting")
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / "partisan_bias_by_state.csv"
    results_df.to_csv(output_file, index=False)

    print()
    print("="*60)
    print(f"Results saved to: {output_file}")
    print("="*60)
    print()

    # Print summary statistics
    print("Summary Statistics:")
    print(f"  States analyzed: {len(results_df)}")
    print(f"  Mean partisan bias: {results_df['algo_partisan_bias'].mean():.2f} pp")
    print(f"  Std dev: {results_df['algo_partisan_bias'].std():.2f} pp")
    print(f"  Range: [{results_df['algo_partisan_bias'].min():.2f}, "
          f"{results_df['algo_partisan_bias'].max():.2f}] pp")
    print()

    # Count Democrat vs. Republican advantages
    dem_advantage = (results_df['algo_partisan_bias'] > 0).sum()
    rep_advantage = (results_df['algo_partisan_bias'] < 0).sum()
    neutral = (results_df['algo_partisan_bias'].abs() < 1).sum()

    print(f"  Democratic advantage (>0): {dem_advantage} states")
    print(f"  Republican advantage (<0): {rep_advantage} states")
    print(f"  Neutral (|bias| < 1 pp): {neutral} states")
    print()

    # Print top 10 Democratic advantages
    print("Top 10 States with Democratic Advantage:")
    print(results_df[['state', 'statewide_dem_vote_pct', 'algo_dem_seat_pct',
                     'algo_partisan_bias', 'n_districts']]
          .head(10).to_string(index=False))
    print()

    # Print top 10 Republican advantages
    print("Top 10 States with Republican Advantage:")
    print(results_df[['state', 'statewide_dem_vote_pct', 'algo_dem_seat_pct',
                     'algo_partisan_bias', 'n_districts']]
          .tail(10).to_string(index=False))


if __name__ == "__main__":
    main()
