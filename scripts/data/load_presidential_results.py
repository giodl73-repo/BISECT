"""
Load 2020 presidential election results by state.

Data source: Dave Leip's Atlas of U.S. Presidential Elections
Aggregated to state-level for proportional seat allocation.
"""

from pathlib import Path
from typing import Dict
import logging

logger = logging.getLogger(__name__)

# 2020 Presidential Election Results (State-level)
# Source: Dave Leip's Atlas, aggregated from county data
PRESIDENTIAL_2020_RESULTS = {
    'alabama': {'Democratic': 0.365, 'Republican': 0.621, 'Other': 0.014},
    'alaska': {'Democratic': 0.428, 'Republican': 0.528, 'Other': 0.044},
    'arizona': {'Democratic': 0.493, 'Republican': 0.491, 'Other': 0.016},
    'arkansas': {'Democratic': 0.347, 'Republican': 0.623, 'Other': 0.030},
    'california': {'Democratic': 0.634, 'Republican': 0.343, 'Other': 0.023},
    'colorado': {'Democratic': 0.553, 'Republican': 0.415, 'Other': 0.032},
    'connecticut': {'Democratic': 0.593, 'Republican': 0.393, 'Other': 0.014},
    'delaware': {'Democratic': 0.587, 'Republican': 0.398, 'Other': 0.015},
    'florida': {'Democratic': 0.477, 'Republican': 0.512, 'Other': 0.011},
    'georgia': {'Democratic': 0.495, 'Republican': 0.493, 'Other': 0.012},
    'hawaii': {'Democratic': 0.637, 'Republican': 0.342, 'Other': 0.021},
    'idaho': {'Democratic': 0.335, 'Republican': 0.638, 'Other': 0.027},
    'illinois': {'Democratic': 0.575, 'Republican': 0.408, 'Other': 0.017},
    'indiana': {'Democratic': 0.409, 'Republican': 0.570, 'Other': 0.021},
    'iowa': {'Democratic': 0.449, 'Republican': 0.532, 'Other': 0.019},
    'kansas': {'Democratic': 0.416, 'Republican': 0.564, 'Other': 0.020},
    'kentucky': {'Democratic': 0.362, 'Republican': 0.620, 'Other': 0.018},
    'louisiana': {'Democratic': 0.398, 'Republican': 0.584, 'Other': 0.018},
    'maine': {'Democratic': 0.532, 'Republican': 0.441, 'Other': 0.027},
    'maryland': {'Democratic': 0.654, 'Republican': 0.323, 'Other': 0.023},
    'massachusetts': {'Democratic': 0.658, 'Republican': 0.323, 'Other': 0.019},
    'michigan': {'Democratic': 0.506, 'Republican': 0.477, 'Other': 0.017},
    'minnesota': {'Democratic': 0.524, 'Republican': 0.453, 'Other': 0.023},
    'mississippi': {'Democratic': 0.415, 'Republican': 0.576, 'Other': 0.009},
    'missouri': {'Democratic': 0.414, 'Republican': 0.565, 'Other': 0.021},
    'montana': {'Democratic': 0.407, 'Republican': 0.567, 'Other': 0.026},
    'nebraska': {'Democratic': 0.394, 'Republican': 0.583, 'Other': 0.023},
    'nevada': {'Democratic': 0.507, 'Republican': 0.478, 'Other': 0.015},
    'new_hampshire': {'Democratic': 0.527, 'Republican': 0.454, 'Other': 0.019},
    'new_jersey': {'Democratic': 0.574, 'Republican': 0.412, 'Other': 0.014},
    'new_mexico': {'Democratic': 0.542, 'Republican': 0.436, 'Other': 0.022},
    'new_york': {'Democratic': 0.605, 'Republican': 0.377, 'Other': 0.018},
    'north_carolina': {'Democratic': 0.487, 'Republican': 0.498, 'Other': 0.015},
    'north_dakota': {'Democratic': 0.316, 'Republican': 0.654, 'Other': 0.030},
    'ohio': {'Democratic': 0.453, 'Republican': 0.532, 'Other': 0.015},
    'oklahoma': {'Democratic': 0.324, 'Republican': 0.654, 'Other': 0.022},
    'oregon': {'Democratic': 0.565, 'Republican': 0.403, 'Other': 0.032},
    'pennsylvania': {'Democratic': 0.501, 'Republican': 0.485, 'Other': 0.014},
    'rhode_island': {'Democratic': 0.595, 'Republican': 0.387, 'Other': 0.018},
    'south_carolina': {'Democratic': 0.432, 'Republican': 0.553, 'Other': 0.015},
    'south_dakota': {'Democratic': 0.358, 'Republican': 0.618, 'Other': 0.024},
    'tennessee': {'Democratic': 0.373, 'Republican': 0.608, 'Other': 0.019},
    'texas': {'Democratic': 0.464, 'Republican': 0.521, 'Other': 0.015},
    'utah': {'Democratic': 0.378, 'Republican': 0.582, 'Other': 0.040},
    'vermont': {'Democratic': 0.661, 'Republican': 0.307, 'Other': 0.032},
    'virginia': {'Democratic': 0.542, 'Republican': 0.441, 'Other': 0.017},
    'washington': {'Democratic': 0.580, 'Republican': 0.387, 'Other': 0.033},
    'west_virginia': {'Democratic': 0.297, 'Republican': 0.687, 'Other': 0.016},
    'wisconsin': {'Democratic': 0.495, 'Republican': 0.489, 'Other': 0.016},
    'wyoming': {'Democratic': 0.264, 'Republican': 0.698, 'Other': 0.038},
}


def get_vote_shares(state: str, year: int = 2020) -> Dict[str, float]:
    """
    Get presidential vote shares for a state.

    Args:
        state: State name (lowercase with underscores, e.g., 'pennsylvania')
        year: Election year (currently only 2020 supported)

    Returns:
        Dictionary mapping party names to vote shares (0-1 scale).
        Keys: 'Democratic', 'Republican', 'Other'

    Raises:
        ValueError: If state not found or year not supported.

    Example:
        >>> get_vote_shares('pennsylvania', 2020)
        {'Democratic': 0.501, 'Republican': 0.485, 'Other': 0.014}
    """
    if year != 2020:
        raise ValueError(f"Year {year} not supported. Only 2020 data available.")

    state_normalized = state.lower().replace(' ', '_').replace('-', '_')

    if state_normalized not in PRESIDENTIAL_2020_RESULTS:
        available = ', '.join(sorted(PRESIDENTIAL_2020_RESULTS.keys()))
        raise ValueError(
            f"State '{state}' not found. Available states: {available}"
        )

    vote_shares = PRESIDENTIAL_2020_RESULTS[state_normalized]

    # Validate vote shares sum to ~1.0
    total = sum(vote_shares.values())
    if not (0.99 <= total <= 1.01):
        logger.warning(
            f"{state}: vote shares sum to {total:.4f}, expected ~1.0"
        )

    logger.info(
        f"{state.title()} 2020: D={vote_shares['Democratic']:.1%}, "
        f"R={vote_shares['Republican']:.1%}, Other={vote_shares['Other']:.1%}"
    )

    return vote_shares


def get_all_states_vote_shares(year: int = 2020) -> Dict[str, Dict[str, float]]:
    """
    Get vote shares for all 50 states.

    Args:
        year: Election year (currently only 2020 supported)

    Returns:
        Dictionary mapping state names to vote share dictionaries.

    Example:
        >>> results = get_all_states_vote_shares(2020)
        >>> results['pennsylvania']
        {'Democratic': 0.501, 'Republican': 0.485, 'Other': 0.014}
    """
    if year != 2020:
        raise ValueError(f"Year {year} not supported. Only 2020 data available.")

    return PRESIDENTIAL_2020_RESULTS.copy()


def get_competitive_states(
    threshold: float = 0.05,
    year: int = 2020
) -> Dict[str, Dict[str, float]]:
    """
    Get states where margin between top two parties < threshold.

    Args:
        threshold: Maximum margin to be considered competitive (default: 5%)
        year: Election year (currently only 2020 supported)

    Returns:
        Dictionary of competitive states and their vote shares.

    Example:
        >>> competitive = get_competitive_states(threshold=0.05)
        >>> 'pennsylvania' in competitive  # 50.1% vs 48.5% = 1.6% margin
        True
        >>> 'wyoming' in competitive  # 69.8% vs 26.4% = 43.4% margin
        False
    """
    all_results = get_all_states_vote_shares(year)
    competitive = {}

    for state, vote_shares in all_results.items():
        dem = vote_shares['Democratic']
        rep = vote_shares['Republican']
        margin = abs(dem - rep)

        if margin <= threshold:
            competitive[state] = vote_shares
            logger.info(
                f"Competitive: {state.title()} (margin: {margin:.1%})"
            )

    return competitive


def validate_vote_data():
    """
    Validate all vote share data.

    Checks:
    - All states present (50 total)
    - Vote shares sum to ~1.0
    - No negative values
    - Reasonable ranges (no party > 80% in 2020)

    Raises:
        AssertionError: If validation fails.
    """
    results = get_all_states_vote_shares(2020)

    # Check count
    assert len(results) == 50, f"Expected 50 states, got {len(results)}"

    # Check each state
    for state, vote_shares in results.items():
        # Sum check
        total = sum(vote_shares.values())
        assert 0.99 <= total <= 1.01, \
            f"{state}: vote shares sum to {total:.4f}"

        # Non-negative check
        for party, share in vote_shares.items():
            assert share >= 0, f"{state} {party}: negative vote share {share}"

        # Reasonable range check (no party > 80% in 2020)
        for party, share in vote_shares.items():
            if party != 'Other':
                assert share <= 0.80, \
                    f"{state} {party}: unusually high vote share {share:.1%}"

    logger.info("Vote data validation passed: 50 states, all valid")


if __name__ == "__main__":
    # Validate data
    validate_vote_data()

    # Show competitive states
    print("\nCompetitive States (margin < 5%):")
    competitive = get_competitive_states(threshold=0.05)
    for state in sorted(competitive.keys()):
        shares = competitive[state]
        margin = abs(shares['Democratic'] - shares['Republican'])
        print(f"  {state.replace('_', ' ').title():20s} "
              f"D: {shares['Democratic']:.1%}  "
              f"R: {shares['Republican']:.1%}  "
              f"Margin: {margin:.1%}")
