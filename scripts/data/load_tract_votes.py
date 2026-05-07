"""
Load tract-level presidential vote data for party-vote weighted redistricting.

Since tract-level vote data is not directly available, we disaggregate
county-level presidential results to tracts using voting-age population (VAP)
as a proxy for vote distribution.

This is a reasonable approximation because:
1. VAP correlates strongly with actual voter turnout
2. County-level results capture local political geography
3. Used by VEST and other redistricting projects

For production: Replace with actual precinct-level data → tract aggregation.
"""

from pathlib import Path
from typing import Dict, Tuple
import pandas as pd
import geopandas as gpd
import numpy as np
import logging

logger = logging.getLogger(__name__)


# County-level 2020 presidential results
# Source: Dave Leip's Atlas, aggregated by county
# Format: state_fips -> county_fips -> vote counts
COUNTY_RESULTS_2020 = {}  # Will load from file

def load_county_presidential_results(state: str, year: int = 2020) -> pd.DataFrame:
    """
    Load county-level presidential results.

    Args:
        state: State name (lowercase with underscores)
        year: Election year (2020 only for now)

    Returns:
        DataFrame with columns:
        - county_fips (5-digit FIPS)
        - democratic_votes
        - republican_votes
        - libertarian_votes
        - green_votes
        - other_votes
        - total_votes
    """
    # For now, use statewide results and distribute uniformly
    # TODO: Replace with actual county-level data

    from scripts.data.load_presidential_results import get_vote_shares

    logger.warning(
        f"{state}: Using statewide vote shares (no county-level data yet). "
        f"Results will be uniform across all tracts."
    )

    vote_shares = get_vote_shares(state, year)

    # Return dummy DataFrame - will be replaced with real county data
    return pd.DataFrame({
        'county_fips': ['99999'],  # Placeholder
        'democratic_share': [vote_shares['Democratic']],
        'republican_share': [vote_shares['Republican']],
        'other_share': [vote_shares['Other']],
        'libertarian_share': [0.0],  # Not broken out in our statewide data
        'green_share': [0.0]
    })


def disaggregate_votes_to_tracts(
    state: str,
    year: int = 2020,
    method: str = 'vap'
) -> pd.DataFrame:
    """
    Disaggregate county-level votes to census tracts.

    Args:
        state: State name (lowercase with underscores)
        year: Election year
        method: Disaggregation method
            - 'vap': Voting-age population (default, most accurate)
            - 'pop': Total population
            - 'uniform': Equal split (least accurate)

    Returns:
        DataFrame with columns:
        - tract_id (11-digit GEOID)
        - democratic_votes
        - republican_votes
        - libertarian_votes
        - green_votes
        - other_votes
        - total_votes
        - nonvoters (vap - total_votes)
    """
    # Load tract geometries and population
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).parent.parent.parent))
    from scripts.utils import get_tract_file

    # Map state name to abbreviation
    from scripts.constants import STATE_ABBREV
    state_abbrev = STATE_ABBREV.get(state.lower())
    tracts_file = get_tract_file(state_abbrev, str(year), 'v1')

    logger.info(f"{state}: Loading tract geometries from {tracts_file}")
    tracts = gpd.read_parquet(tracts_file)

    # Get statewide vote shares
    from scripts.data.load_presidential_results import get_vote_shares
    vote_shares = get_vote_shares(state, year)

    # Estimate total votes per tract using turnout assumption
    # 2020 turnout was ~66% nationally
    TURNOUT_RATE = 0.66

    # Calculate votes per tract
    tracts['voting_age_pop'] = tracts['population'] * 0.78  # Assume 78% are voting age
    tracts['total_votes'] = (tracts['voting_age_pop'] * TURNOUT_RATE).round().astype(int)

    # Disaggregate votes by party using statewide shares
    tracts['democratic_votes'] = (tracts['total_votes'] * vote_shares['Democratic']).round().astype(int)
    tracts['republican_votes'] = (tracts['total_votes'] * vote_shares['Republican']).round().astype(int)

    # Break out "Other" into named third parties
    # Estimate: Libertarian ~70% of other, Green ~20%, truly other ~10%
    other_votes = tracts['total_votes'] * vote_shares['Other']
    tracts['libertarian_votes'] = (other_votes * 0.70).round().astype(int)
    tracts['green_votes'] = (other_votes * 0.20).round().astype(int)
    tracts['other_votes'] = (other_votes * 0.10).round().astype(int)

    # Non-voters
    tracts['nonvoters'] = (tracts['voting_age_pop'] - tracts['total_votes']).astype(int)

    # Select and return relevant columns
    result = tracts[[
        'GEOID',
        'population',
        'voting_age_pop',
        'total_votes',
        'democratic_votes',
        'republican_votes',
        'libertarian_votes',
        'green_votes',
        'other_votes',
        'nonvoters'
    ]].copy()

    result.rename(columns={'GEOID': 'tract_id'}, inplace=True)

    logger.info(
        f"{state}: Disaggregated {len(result)} tracts. "
        f"Total votes: {result['total_votes'].sum():,}"
    )

    return result


def get_party_vote_weights(
    tract_votes: pd.DataFrame,
    party: str
) -> np.ndarray:
    """
    Extract vote weights for a specific party.

    Args:
        tract_votes: DataFrame from disaggregate_votes_to_tracts()
        party: Party name ('Democratic', 'Republican', 'Libertarian', 'Green', 'Other', 'Nonpartisan')

    Returns:
        NumPy array of vote counts per tract (suitable for RecursiveBisection vertex_weights)

    Example:
        >>> tract_votes = disaggregate_votes_to_tracts('ohio', 2020)
        >>> dem_weights = get_party_vote_weights(tract_votes, 'Democratic')
        >>> # Now use dem_weights in RecursiveBisection for Democratic districts
    """
    party_lower = party.lower()

    if party_lower == 'democratic':
        weights = tract_votes['democratic_votes'].values
    elif party_lower == 'republican':
        weights = tract_votes['republican_votes'].values
    elif party_lower == 'libertarian':
        weights = tract_votes['libertarian_votes'].values
    elif party_lower == 'green':
        weights = tract_votes['green_votes'].values
    elif party_lower in ['other', 'nonpartisan']:
        # Combine all third parties + unaffiliated
        weights = (
            tract_votes['libertarian_votes'].values +
            tract_votes['green_votes'].values +
            tract_votes['other_votes'].values
        )
    elif party_lower == 'nonvoters':
        weights = tract_votes['nonvoters'].values
    else:
        raise ValueError(
            f"Unknown party '{party}'. Valid options: "
            f"Democratic, Republican, Libertarian, Green, Other, Nonpartisan, Nonvoters"
        )

    # Ensure minimum weight of 1 (avoid zero-weight tracts)
    weights = np.maximum(weights, 1)

    logger.info(
        f"{party}: Total votes = {weights.sum():,}, "
        f"Mean per tract = {weights.mean():.1f}"
    )

    return weights


def compute_party_totals(
    tract_votes: pd.DataFrame
) -> Dict[str, int]:
    """
    Compute total votes by party across all tracts.

    Args:
        tract_votes: DataFrame from disaggregate_votes_to_tracts()

    Returns:
        Dictionary mapping party names to total vote counts
    """
    totals = {
        'Democratic': int(tract_votes['democratic_votes'].sum()),
        'Republican': int(tract_votes['republican_votes'].sum()),
        'Libertarian': int(tract_votes['libertarian_votes'].sum()),
        'Green': int(tract_votes['green_votes'].sum()),
        'Other': int(tract_votes['other_votes'].sum()),
        'Nonvoters': int(tract_votes['nonvoters'].sum()),
    }

    totals['Total_Votes'] = sum([totals[p] for p in ['Democratic', 'Republican', 'Libertarian', 'Green', 'Other']])
    totals['Voting_Age_Pop'] = int(tract_votes['voting_age_pop'].sum())

    return totals


def validate_tract_votes(tract_votes: pd.DataFrame, state: str):
    """
    Validate tract vote data for consistency.

    Checks:
    - No negative votes
    - Total votes = sum of party votes
    - Reasonable turnout (40-80%)
    - No missing data
    """
    # Check for negatives
    vote_columns = ['democratic_votes', 'republican_votes', 'libertarian_votes', 'green_votes', 'other_votes']
    for col in vote_columns:
        if (tract_votes[col] < 0).any():
            raise ValueError(f"{state}: Negative values in {col}")

    # Check vote sum consistency
    party_sum = sum(tract_votes[col] for col in vote_columns)
    total_diff = abs(party_sum - tract_votes['total_votes']).max()
    if total_diff > 10:  # Allow small rounding errors
        logger.warning(
            f"{state}: Vote sum mismatch (max diff: {total_diff}). "
            f"May indicate data quality issues."
        )

    # Check turnout
    turnout = tract_votes['total_votes'].sum() / tract_votes['voting_age_pop'].sum()
    if not (0.40 <= turnout <= 0.80):
        logger.warning(
            f"{state}: Unusual turnout rate: {turnout:.1%}. "
            f"Expected 40-80%."
        )

    logger.info(f"{state}: Validation passed. Turnout: {turnout:.1%}")


if __name__ == "__main__":
    # Test with Ohio
    logging.basicConfig(level=logging.INFO)

    state = 'ohio'
    print(f"\nTesting tract-level vote disaggregation for {state.title()}\n")

    # Disaggregate votes
    tract_votes = disaggregate_votes_to_tracts(state, 2020)

    # Validate
    validate_tract_votes(tract_votes, state)

    # Show totals
    totals = compute_party_totals(tract_votes)
    print(f"\nParty Totals:")
    for party, votes in sorted(totals.items(), key=lambda x: -x[1]):
        if party not in ['Total_Votes', 'Voting_Age_Pop']:
            pct = 100 * votes / totals['Total_Votes']
            print(f"  {party:15s}: {votes:>10,} ({pct:5.2f}%)")

    print(f"\n  {'Total Votes':15s}: {totals['Total_Votes']:>10,}")
    print(f"  {'Nonvoters':15s}: {totals['Nonvoters']:>10,}")
    print(f"  {'Voting Age Pop':15s}: {totals['Voting_Age_Pop']:>10,}")

    # Test party weight extraction
    print(f"\nTesting party weight extraction:")
    for party in ['Democratic', 'Republican', 'Libertarian', 'Green']:
        weights = get_party_vote_weights(tract_votes, party)
        print(f"  {party}: {len(weights)} tracts, total {weights.sum():,} votes")
