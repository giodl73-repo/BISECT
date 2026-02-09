#!/usr/bin/env python3
"""
MMD Electoral Simulation using D'Hondt Proportional Representation

Simulates elections under multi-member districts with party-list PR.
Uses the D'Hondt method (highest averages) for seat allocation.

Usage:
    python scripts/experiments/mmd_simulate_election.py --year 2020 --method dhondt --config uniform-5
    python scripts/experiments/mmd_simulate_election.py --year 2020 --config adaptive-3-5 --threshold census-standard
"""

import argparse
import sys
from pathlib import Path
from typing import Dict, List, Tuple
import pandas as pd
import numpy as np

sys.path.insert(0, str(Path(__file__).parent.parent.parent / 'src'))


# ============================================================================
# D'Hondt Proportional Representation
# ============================================================================

def dhondt_allocation(votes: Dict[str, int], seats: int) -> Dict[str, int]:
    """
    Allocate seats using D'Hondt method (highest averages).

    Args:
        votes: Dictionary {party: vote_count}
        seats: Number of seats to allocate

    Returns:
        Dictionary {party: seats_won}

    Example:
        votes = {'D': 40000, 'R': 35000, 'L': 15000, 'G': 10000}
        seats = 5
        → {'D': 2, 'R': 2, 'L': 1, 'G': 0}
    """

    # Initialize seat allocation
    allocation = {party: 0 for party in votes.keys()}

    # Allocate seats one by one
    for _ in range(seats):
        # Compute quotients: votes / (seats_won + 1)
        quotients = {
            party: votes[party] / (allocation[party] + 1)
            for party in votes.keys()
        }

        # Award seat to party with highest quotient
        winner = max(quotients, key=quotients.get)
        allocation[winner] += 1

    return allocation


def sainte_lague_allocation(votes: Dict[str, int], seats: int) -> Dict[str, int]:
    """
    Allocate seats using Sainte-Laguë method (alternative to D'Hondt).

    More favorable to smaller parties (divides by odd numbers: 1, 3, 5, ...).
    """

    allocation = {party: 0 for party in votes.keys()}

    for _ in range(seats):
        quotients = {
            party: votes[party] / (2 * allocation[party] + 1)
            for party in votes.keys()
        }

        winner = max(quotients, key=quotients.get)
        allocation[winner] += 1

    return allocation


# ============================================================================
# Electoral Simulation
# ============================================================================

def load_district_assignments(mmd_dir: Path) -> pd.DataFrame:
    """Load district assignments from MMD generation output."""

    assignments_path = mmd_dir / "districts.csv"

    if not assignments_path.exists():
        raise FileNotFoundError(
            f"District assignments not found: {assignments_path}\n"
            f"Run mmd_generate_districts.py first"
        )

    df = pd.read_csv(assignments_path)

    # Standardize column name (handle both 'geoid' and 'GEOID')
    if 'GEOID' in df.columns:
        df = df.rename(columns={'GEOID': 'geoid'})

    return df


def load_election_data(year: int) -> pd.DataFrame:
    """
    Load presidential election data by census tract.

    Expected columns: geoid, dem_votes, rep_votes, lib_votes, grn_votes, oth_votes
    """

    election_path = Path(f"data/{year}/elections/presidential_by_tract.csv")

    if not election_path.exists():
        raise FileNotFoundError(
            f"Election data not found: {election_path}\n"
            f"Presidential vote by tract is required for simulation"
        )

    return pd.read_csv(election_path)


def get_district_members(mmd_dir: Path) -> int:
    """Determine members per district from summary file."""

    summary_path = mmd_dir / "summary.csv"

    if summary_path.exists():
        summary = pd.read_csv(summary_path)
        return int(summary['members_per_district'].iloc[0])

    # Fallback: parse from directory name
    dir_name = mmd_dir.name
    if 'member' in dir_name:
        return int(dir_name.split('-')[0])

    raise ValueError(f"Cannot determine district size from {mmd_dir}")


def simulate_mmd_election(
    districts: pd.DataFrame,
    elections: pd.DataFrame,
    members_per_district: int,
    method: str = 'dhondt'
) -> Tuple[pd.DataFrame, pd.DataFrame]:
    """
    Simulate election under MMD proportional representation.

    Returns:
        (district_results, national_summary)
    """

    print(f"\n[Electoral Simulation]")
    print(f"Method: {method.upper()}")
    print(f"Districts: {districts['district'].nunique()}")
    print(f"Members per district: {members_per_district}")

    # Convert block group GEOIDs to tract GEOIDs (first 11 digits)
    # Block group: SSCCCTTTTTTG (12 digits)
    # Tract: SSCCCTTTTTT (11 digits)
    districts['geoid_str'] = districts['geoid'].astype(str).str.zfill(12)
    districts['tract_geoid'] = districts['geoid_str'].str[:11]

    # Ensure election GEOIDs are also strings with proper padding
    elections['geoid'] = elections['geoid'].astype(str).str.zfill(11)

    # Merge district assignments with election data (using tract GEOID)
    merged = districts.merge(elections, left_on='tract_geoid', right_on='geoid', how='left', suffixes=('', '_elec'))

    # Check for missing election data
    missing = merged[merged['dem_votes'].isna()]
    if len(missing) > 0:
        print(f"Warning: {len(missing)} tracts missing election data (will be excluded)")
        merged = merged.dropna(subset=['dem_votes'])

    # Aggregate votes by district
    vote_cols = ['dem_votes', 'rep_votes', 'lib_votes', 'grn_votes', 'oth_votes']
    district_votes = merged.groupby('district')[vote_cols].sum()

    print(f"\nSimulating seat allocation for {len(district_votes)} districts...")

    # Allocate seats in each district
    allocation_func = dhondt_allocation if method == 'dhondt' else sainte_lague_allocation

    district_results = []

    for district_id, votes in district_votes.iterrows():
        # Convert to dictionary
        votes_dict = {
            'Democratic': votes['dem_votes'],
            'Republican': votes['rep_votes'],
            'Libertarian': votes['lib_votes'],
            'Green': votes['grn_votes'],
            'Other': votes['oth_votes']
        }

        # Allocate seats
        seats_won = allocation_func(votes_dict, members_per_district)

        # Calculate vote shares
        total_votes = sum(votes_dict.values())
        vote_shares = {party: v / total_votes for party, v in votes_dict.items()}

        # Record results
        district_results.append({
            'district': district_id,
            'total_votes': total_votes,
            'dem_votes': votes['dem_votes'],
            'rep_votes': votes['rep_votes'],
            'lib_votes': votes['lib_votes'],
            'grn_votes': votes['grn_votes'],
            'oth_votes': votes['oth_votes'],
            'dem_share': vote_shares['Democratic'],
            'rep_share': vote_shares['Republican'],
            'lib_share': vote_shares['Libertarian'],
            'grn_share': vote_shares['Green'],
            'oth_share': vote_shares['Other'],
            'dem_seats': seats_won['Democratic'],
            'rep_seats': seats_won['Republican'],
            'lib_seats': seats_won['Libertarian'],
            'grn_seats': seats_won['Green'],
            'oth_seats': seats_won['Other'],
            'total_seats': members_per_district
        })

    district_results_df = pd.DataFrame(district_results)

    # Compute national summary
    national_votes = district_results_df[vote_cols].sum()
    national_seats = district_results_df[['dem_seats', 'rep_seats', 'lib_seats', 'grn_seats', 'oth_seats']].sum()

    total_votes = national_votes.sum()
    total_seats = national_seats.sum()

    national_summary = pd.DataFrame({
        'party': ['Democratic', 'Republican', 'Libertarian', 'Green', 'Other'],
        'votes': [national_votes['dem_votes'], national_votes['rep_votes'],
                  national_votes['lib_votes'], national_votes['grn_votes'],
                  national_votes['oth_votes']],
        'seats': [national_seats['dem_seats'], national_seats['rep_seats'],
                  national_seats['lib_seats'], national_seats['grn_seats'],
                  national_seats['oth_seats']]
    })

    national_summary['vote_share'] = national_summary['votes'] / total_votes
    national_summary['seat_share'] = national_summary['seats'] / total_seats
    national_summary['deviation'] = national_summary['seat_share'] - national_summary['vote_share']

    # Display summary
    print(f"\n[National Results]")
    print(f"Total votes: {total_votes:,}")
    print(f"Total seats: {int(total_seats)}")
    print("\n" + national_summary.to_string(index=False))

    # Highlight minor parties
    minor_party_seats = national_summary[~national_summary['party'].isin(['Democratic', 'Republican'])]['seats'].sum()
    print(f"\nMinor party seats: {int(minor_party_seats)} ({minor_party_seats/total_seats*100:.1f}%)")

    return district_results_df, national_summary


def compare_to_single_member_baseline(
    national_summary: pd.DataFrame,
    year: int
) -> pd.DataFrame:
    """Compare MMD results to single-member baseline."""

    # Load single-member results (from actual elections or algorithmic baseline)
    baseline_path = Path(f"outputs/v1/{year}/national/seats_by_party.csv")

    if not baseline_path.exists():
        print(f"\nWarning: Single-member baseline not found at {baseline_path}")
        return national_summary

    baseline = pd.read_csv(baseline_path)

    # Merge with MMD results
    comparison = national_summary.merge(
        baseline[['party', 'seats']],
        on='party',
        how='left',
        suffixes=('_mmd', '_single')
    )

    comparison['seat_change'] = comparison['seats_mmd'] - comparison['seats_single']

    print(f"\n[Comparison to Single-Member Baseline]")
    print(comparison[['party', 'seats_single', 'seats_mmd', 'seat_change']].to_string(index=False))

    return comparison


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(description="Simulate MMD election with proportional representation")
    parser.add_argument('--year', type=int, default=2020, choices=[2000, 2010, 2020])
    parser.add_argument('--config', type=str, required=True,
                        help='Configuration name (e.g., uniform-5, adaptive-3-5)')
    parser.add_argument('--threshold', type=str,
                        help='Density threshold variant (for adaptive configs)')
    parser.add_argument('--method', type=str, default='dhondt', choices=['dhondt', 'sainte-lague'])
    parser.add_argument('--input-dir', type=Path, default=Path('outputs/mmd'))
    parser.add_argument('--output-dir', type=Path, default=Path('outputs/mmd'))

    args = parser.parse_args()

    print("="*70)
    print("MMD ELECTORAL SIMULATION")
    print("="*70)
    print(f"Year: {args.year}")
    print(f"Configuration: {args.config}")
    if args.threshold:
        print(f"Threshold: {args.threshold}")
    print(f"Method: {args.method}")

    # Determine input directory
    if args.threshold:
        config_name = f"{args.config}__{args.threshold}"
        mmd_dir = args.input_dir / config_name
    else:
        mmd_dir = args.input_dir / args.config

    if not mmd_dir.exists():
        raise FileNotFoundError(
            f"MMD configuration not found: {mmd_dir}\n"
            f"Run mmd_generate_districts.py first"
        )

    # Load district assignments
    print(f"\nLoading district assignments from {mmd_dir}...")
    districts = load_district_assignments(mmd_dir)
    members = get_district_members(mmd_dir)
    print(f"Loaded {len(districts):,} tract assignments")
    print(f"Districts: {districts['district'].nunique()}")
    print(f"Members per district: {members}")

    # Load election data
    print(f"\nLoading {args.year} presidential election data...")
    elections = load_election_data(args.year)
    print(f"Loaded {len(elections):,} tract vote totals")

    # Run simulation
    district_results, national_summary = simulate_mmd_election(
        districts=districts,
        elections=elections,
        members_per_district=members,
        method=args.method
    )

    # Compare to baseline
    comparison = compare_to_single_member_baseline(national_summary, args.year)

    # Save results
    output_dir = args.output_dir / config_name if args.threshold else args.output_dir / args.config
    output_dir.mkdir(parents=True, exist_ok=True)

    district_path = output_dir / "district_election_results.csv"
    national_path = output_dir / "national_summary.csv"

    district_results.to_csv(district_path, index=False)
    national_summary.to_csv(national_path, index=False)

    print(f"\n[Output]")
    print(f"District results: {district_path}")
    print(f"National summary: {national_path}")

    print(f"\n[SUCCESS] Electoral simulation complete")
    print(f"Next step: Compute proportionality metrics (mmd_compute_gallagher.py)")


if __name__ == '__main__':
    main()
