"""
Party-Specific Overlapping Districts Redistricting Pipeline.

This script implements the overlapping party-district system:
1. Load statewide vote shares (2020 presidential)
2. Allocate seats to parties using D'Hondt method
3. Run recursive bisection separately for each party
4. Generate party-specific district shapefiles
5. Create overlay visualizations

Usage:
    python run_party_specific_redistricting.py --state pennsylvania --year 2020 --version party_v1
    python run_party_specific_redistricting.py --year 2020 --version party_v1 --all-states
"""

import argparse
import logging
from pathlib import Path
import sys
import json
from typing import Dict, Optional
import subprocess
import numpy as np
import geopandas as gpd
import pandas as pd

# Add project root and src to path
project_root = Path(__file__).parent.parent.parent
src_path = project_root / "src"
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(src_path))

from apportionment.proportional.dhondt import allocate_seats_dhondt, suggest_threshold
from scripts.data.load_presidential_results import get_vote_shares
from scripts.config_2020 import STATE_CONFIG_2020

logger = logging.getLogger(__name__)

# State name to abbreviation mapping
STATE_ABBREV = {
    'alabama': 'AL', 'alaska': 'AK', 'arizona': 'AZ', 'arkansas': 'AR',
    'california': 'CA', 'colorado': 'CO', 'connecticut': 'CT', 'delaware': 'DE',
    'florida': 'FL', 'georgia': 'GA', 'hawaii': 'HI', 'idaho': 'ID',
    'illinois': 'IL', 'indiana': 'IN', 'iowa': 'IA', 'kansas': 'KS',
    'kentucky': 'KY', 'louisiana': 'LA', 'maine': 'ME', 'maryland': 'MD',
    'massachusetts': 'MA', 'michigan': 'MI', 'minnesota': 'MN', 'mississippi': 'MS',
    'missouri': 'MO', 'montana': 'MT', 'nebraska': 'NE', 'nevada': 'NV',
    'new_hampshire': 'NH', 'new_jersey': 'NJ', 'new_mexico': 'NM', 'new_york': 'NY',
    'north_carolina': 'NC', 'north_dakota': 'ND', 'ohio': 'OH', 'oklahoma': 'OK',
    'oregon': 'OR', 'pennsylvania': 'PA', 'rhode_island': 'RI', 'south_carolina': 'SC',
    'south_dakota': 'SD', 'tennessee': 'TN', 'texas': 'TX', 'utah': 'UT',
    'vermont': 'VT', 'virginia': 'VA', 'washington': 'WA', 'west_virginia': 'WV',
    'wisconsin': 'WI', 'wyoming': 'WY'
}


def setup_logging(verbose: bool = False):
    """Configure logging."""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s [%(levelname)s] %(message)s',
        datefmt='%H:%M:%S'
    )


def get_state_seat_count(state: str, year: int) -> int:
    """
    Get number of congressional seats for a state.

    Args:
        state: State name (lowercase with underscores)
        year: Census year (2020, 2010, 2000)

    Returns:
        Number of House seats allocated to the state.
    """
    if year == 2020:
        seat_config = STATE_CONFIG_2020
    else:
        raise ValueError(f"Year {year} not supported yet")

    state_normalized = state.lower().replace(' ', '_').replace('-', '_')

    # Convert to abbreviation if needed
    if state_normalized in STATE_ABBREV:
        state_abbrev = STATE_ABBREV[state_normalized]
    else:
        state_abbrev = state_normalized.upper()

    if state_abbrev not in seat_config:
        raise ValueError(f"State '{state}' not found in configuration")

    return seat_config[state_abbrev]['districts']


def run_recursive_bisection_for_party(
    state: str,
    year: int,
    version: str,
    party: str,
    num_districts: int,
    dry_run: bool = False
) -> bool:
    """
    Run recursive bisection to create party-specific districts using party-vote weighting.

    Args:
        state: State name
        year: Census year
        version: Output version identifier
        party: Party name (Democratic, Republican, etc.)
        num_districts: Number of districts to create
        dry_run: If True, print command without executing

    Returns:
        True if successful, False otherwise.
    """
    logger.info(
        f"{state.title()}: Creating {num_districts} {party} districts"
    )

    # Output directory: outputs/{version}_{year}/{state}/{party}/
    output_base = project_root / "outputs" / f"{version}_{year}"
    party_dir = output_base / state / party.lower().replace(' ', '_')

    if dry_run:
        logger.info(f"[DRY RUN] Would create directory: {party_dir}")
        logger.info(
            f"[DRY RUN] Would run redistricting: {num_districts} districts with party-vote weighting"
        )
        return True

    # Create output directory
    party_dir.mkdir(parents=True, exist_ok=True)

    try:
        # Import required modules
        import geopandas as gpd
        import pandas as pd
        from apportionment.partition.recursive_bisection import RecursiveBisection
        from apportionment.data.adjacency import build_adjacency_graph
        from scripts.data.load_tract_votes import disaggregate_votes_to_tracts, get_party_vote_weights, validate_tract_votes
        from scripts.utils import get_tract_file

        # 1. Load tract geometries
        state_abbrev = STATE_ABBREV.get(state.lower())
        tracts_file = get_tract_file(state_abbrev, str(year), 'v1')

        logger.info(f"{state.title()} {party}: Loading tract geometries from {tracts_file}")
        tracts = gpd.read_parquet(tracts_file)
        n_tracts = len(tracts)

        # 2. Build adjacency graph
        logger.info(f"{state.title()} {party}: Building adjacency graph")
        adjacency, tract_populations, idx_to_geoid, geoid_to_idx, _ = build_adjacency_graph(
            tracts,
            compute_boundary_lengths=False
        )

        # 3. Load tract-level votes and extract party-specific weights
        logger.info(f"{state.title()} {party}: Loading tract-level votes")
        tract_votes = disaggregate_votes_to_tracts(state, year)
        validate_tract_votes(tract_votes, state)

        # 4. Extract party-specific vote weights aligned with adjacency graph
        logger.info(f"{state.title()} {party}: Extracting {party} vote weights")

        # Create a mapping from tract_id to votes
        party_col = f"{party.lower()}_votes"
        vote_dict = dict(zip(tract_votes['tract_id'], tract_votes[party_col]))

        # Build vertex_weights array in the same order as adjacency list
        vertex_weights = np.zeros(len(idx_to_geoid), dtype=np.int32)
        for idx in range(len(idx_to_geoid)):
            geoid = idx_to_geoid[idx]
            vertex_weights[idx] = vote_dict.get(geoid, 1)  # Default to 1 if missing

        # Verify alignment
        if len(vertex_weights) != n_tracts:
            raise ValueError(
                f"Vote weight count ({len(vertex_weights)}) doesn't match tract count ({n_tracts})"
            )

        # 5. Run recursive bisection with party-vote weights
        logger.info(
            f"{state.title()} {party}: Running recursive bisection "
            f"({num_districts} districts, {vertex_weights.sum():,.0f} {party} votes)"
        )

        rb = RecursiveBisection(
            adjacency=adjacency,
            vertex_weights=vertex_weights,  # CRITICAL: Party-specific weights!
            num_districts=num_districts,
            state_code=state_abbrev,
            debug=False,
            seed=42  # Reproducible results
        )

        district_assignments = rb.partition()

        # 6. Save results
        logger.info(f"{state.title()} {party}: Saving results")

        # Add district assignments to tracts
        tracts['district'] = district_assignments
        tracts['party'] = party

        # Save shapefile
        output_file = party_dir / f"{state}_{party.lower().replace(' ', '_')}_districts.shp"
        tracts.to_file(output_file)

        # Save CSV with district summaries
        district_summary = tracts.groupby('district').agg({
            'population': 'sum',
            'GEOID': 'count'
        }).rename(columns={'GEOID': 'num_tracts'})

        # Add party vote totals
        tracts_with_votes = tracts.merge(
            tract_votes,
            left_on='GEOID',
            right_on='tract_id',
            how='left'
        )

        party_col = f"{party.lower()}_votes"
        district_summary[party_col] = tracts_with_votes.groupby('district')[party_col].sum()

        csv_file = party_dir / f"{state}_{party.lower().replace(' ', '_')}_summary.csv"
        district_summary.to_csv(csv_file)

        # Save metadata
        metadata = {
            "state": state,
            "party": party,
            "num_districts": num_districts,
            "type": "party_vote_weighted",
            "year": year,
            "version": version,
            "total_tracts": n_tracts,
            "total_party_votes": int(vertex_weights.sum()),
            "avg_votes_per_district": int(vertex_weights.sum() / num_districts)
        }

        metadata_path = party_dir / "metadata.json"
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)

        logger.info(f"{state.title()} {party}: Redistricting complete")
        logger.info(
            f"  -> {num_districts} districts, "
            f"{vertex_weights.sum():,.0f} {party} votes "
            f"({vertex_weights.sum()/num_districts:,.0f} votes/district)"
        )

        return True

    except Exception as e:
        logger.error(
            f"{state.title()} {party}: Redistricting failed\n"
            f"Error: {str(e)}"
        )
        import traceback
        logger.debug(traceback.format_exc())
        return False


def create_whole_state_district(
    state: str,
    year: int,
    version: str,
    party: str,
    dry_run: bool = False
) -> bool:
    """
    Create a single district covering the entire state (for 1-seat parties).

    Args:
        state: State name
        year: Census year
        version: Output version identifier
        party: Party name
        dry_run: If True, print actions without executing

    Returns:
        True if successful, False otherwise.
    """
    logger.info(
        f"{state.title()}: {party} gets 1 seat (whole state district)"
    )

    output_base = project_root / "outputs" / f"{version}_{year}"
    party_dir = output_base / state / party.lower().replace(' ', '_')

    if dry_run:
        logger.info(f"[DRY RUN] Would create whole-state district in {party_dir}")
        return True

    party_dir.mkdir(parents=True, exist_ok=True)

    # Save metadata indicating this is a whole-state district
    metadata = {
        "state": state,
        "party": party,
        "num_districts": 1,
        "type": "whole_state",
        "year": year,
        "version": version
    }

    metadata_path = party_dir / "metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)

    logger.info(f"{state.title()} {party}: Whole-state district created")
    return True


def run_party_specific_redistricting(
    state: str,
    year: int = 2020,
    version: str = "party_v1",
    dry_run: bool = False,
    use_natural_threshold: bool = True
) -> Dict[str, int]:
    """
    Generate overlapping party-specific districts for a state.

    Args:
        state: State name (lowercase with underscores)
        year: Election/census year
        version: Output version identifier
        dry_run: If True, print actions without executing
        use_natural_threshold: If True, use 1/(N+1) threshold

    Returns:
        Dictionary mapping party names to allocated seat counts.
    """
    logger.info(f"\n{'='*60}")
    logger.info(f"Party-Specific Redistricting: {state.title()}")
    logger.info(f"{'='*60}")

    # Step 1: Get vote shares
    vote_shares = get_vote_shares(state, year)
    logger.info(
        f"Vote shares: D={vote_shares['Democratic']:.1%}, "
        f"R={vote_shares['Republican']:.1%}, "
        f"Other={vote_shares['Other']:.1%}"
    )

    # Step 2: Get state seat count
    num_seats = get_state_seat_count(state, year)
    logger.info(f"Total seats: {num_seats}")

    # Step 3: Determine threshold
    if use_natural_threshold:
        threshold = suggest_threshold(num_seats)
        logger.info(f"Natural threshold: {threshold:.1%}")
    else:
        threshold = 0.0
        logger.info("No threshold (all parties eligible)")

    # Step 4: Allocate seats using D'Hondt
    seat_allocation = allocate_seats_dhondt(
        vote_shares=vote_shares,
        num_seats=num_seats,
        threshold=threshold
    )

    # Log allocation
    logger.info("\nSeat Allocation (D'Hondt):")
    for party, seats in sorted(seat_allocation.items(), key=lambda x: -x[1]):
        if seats > 0:
            vote_pct = vote_shares[party] * 100
            seat_pct = (seats / num_seats) * 100
            logger.info(
                f"  {party:12s}: {seats:2d} seats ({seat_pct:4.1f}%) "
                f"from {vote_pct:4.1f}% votes"
            )

    # Step 5: Run redistricting for each party
    logger.info("\nGenerating Party-Specific Districts:")

    success_count = 0
    for party, seats in seat_allocation.items():
        if seats == 0:
            logger.info(f"  {party}: 0 seats (below threshold)")
            continue

        elif seats == 1:
            success = create_whole_state_district(
                state, year, version, party, dry_run
            )

        else:  # seats >= 2
            success = run_recursive_bisection_for_party(
                state, year, version, party, seats, dry_run
            )

        if success:
            success_count += 1

    # Summary
    parties_with_seats = sum(1 for s in seat_allocation.values() if s > 0)
    logger.info(
        f"\nComplete: {success_count}/{parties_with_seats} parties successful"
    )

    return seat_allocation


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate party-specific overlapping districts"
    )
    parser.add_argument(
        '--state', '-s',
        type=str,
        help='State name (lowercase with underscores)'
    )
    parser.add_argument(
        '--year', '-y',
        type=int,
        default=2020,
        help='Election/census year (default: 2020)'
    )
    parser.add_argument(
        '--version', '-v',
        type=str,
        default='party_v1',
        help='Output version identifier (default: party_v1)'
    )
    parser.add_argument(
        '--all-states',
        action='store_true',
        help='Run for all 50 states'
    )
    parser.add_argument(
        '--states',
        nargs='+',
        help='Run for specific list of states'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Print actions without executing'
    )
    parser.add_argument(
        '--no-threshold',
        action='store_true',
        help='Disable natural threshold (all parties eligible)'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Enable debug logging'
    )

    args = parser.parse_args()

    setup_logging(args.verbose)

    # Determine which states to process
    if args.all_states:
        states = list(STATE_CONFIG_2020.keys())
        logger.info(f"Processing all 50 states")
    elif args.states:
        states = [s.lower().replace(' ', '_').replace('-', '_') for s in args.states]
        logger.info(f"Processing {len(states)} states: {', '.join(states)}")
    elif args.state:
        states = [args.state.lower().replace(' ', '_').replace('-', '_')]
    else:
        parser.error("Must specify --state, --states, or --all-states")

    # Process each state
    results = {}
    for state in states:
        try:
            seat_allocation = run_party_specific_redistricting(
                state=state,
                year=args.year,
                version=args.version,
                dry_run=args.dry_run,
                use_natural_threshold=not args.no_threshold
            )
            results[state] = seat_allocation

        except Exception as e:
            logger.error(f"{state.title()}: Failed with error: {e}")
            results[state] = None

    # Final summary
    logger.info(f"\n{'='*60}")
    logger.info("Summary")
    logger.info(f"{'='*60}")

    successful = sum(1 for r in results.values() if r is not None)
    logger.info(f"States processed: {successful}/{len(states)}")

    if successful < len(states):
        failed = [s for s, r in results.items() if r is None]
        logger.warning(f"Failed states: {', '.join(failed)}")


if __name__ == "__main__":
    main()
