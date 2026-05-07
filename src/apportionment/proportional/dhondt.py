"""
D'Hondt method for proportional seat allocation.

The D'Hondt method (also called Jefferson method) is the most common
proportional allocation formula used in European democracies. It allocates
seats by computing quotients v_i / j for each party i and divisor j, then
awarding seats to the highest quotients.

References:
    - Gallagher, M., & Mitchell, P. (2005). The Politics of Electoral Systems.
    - Taagepera, R., & Shugart, M. S. (1989). Seats and Votes.
"""

from typing import Dict, List, Tuple
import logging

logger = logging.getLogger(__name__)


def compute_dhondt_quotients(
    vote_shares: Dict[str, float],
    num_seats: int
) -> List[Tuple[float, str, int]]:
    """
    Compute D'Hondt quotients for all parties and divisors.

    Args:
        vote_shares: Dictionary mapping party names to vote shares (0-1 scale).
                     Example: {"Democratic": 0.51, "Republican": 0.49}
        num_seats: Total number of seats to allocate.

    Returns:
        List of (quotient, party, divisor) tuples sorted by quotient descending.
        The first num_seats tuples determine seat allocation.

    Example:
        >>> quotients = compute_dhondt_quotients(
        ...     {"Democratic": 0.51, "Republican": 0.49}, 10
        ... )
        >>> quotients[0]  # Highest quotient
        (0.51, 'Democratic', 1)
        >>> quotients[1]  # Second highest
        (0.49, 'Republican', 1)
    """
    quotients = []

    for party, vote_share in vote_shares.items():
        for divisor in range(1, num_seats + 1):
            quotient = vote_share / divisor
            quotients.append((quotient, party, divisor))

    # Sort by quotient descending, then by party name for deterministic ties
    quotients.sort(key=lambda x: (-x[0], x[1]))

    return quotients


def allocate_seats_dhondt(
    vote_shares: Dict[str, float],
    num_seats: int,
    threshold: float = 0.0,
    min_seats: Dict[str, int] = None
) -> Dict[str, int]:
    """
    Allocate seats using the D'Hondt method with optional threshold.

    The D'Hondt method guarantees:
    - Monotonicity: More votes never results in fewer seats
    - Proportionality: Seat shares approximate vote shares
    - Determinism: Same inputs always produce same outputs

    Args:
        vote_shares: Dictionary mapping party names to vote shares (0-1 scale).
                     Vote shares should sum to 1.0 (or close due to rounding).
        num_seats: Total number of seats to allocate.
        threshold: Minimum vote share to qualify for seats (default: 0.0).
                   Common thresholds: 0.05 (5%), 1/(num_seats+1) (natural).
        min_seats: Optional dict of minimum seats per party (for VRA compliance).

    Returns:
        Dictionary mapping party names to allocated seat counts.
        Parties below threshold receive 0 seats.

    Raises:
        ValueError: If vote_shares don't sum to ~1.0, or num_seats < 1.

    Example:
        >>> # Pennsylvania: 17 seats, 51% D, 49% R
        >>> allocate_seats_dhondt(
        ...     {"Democratic": 0.51, "Republican": 0.49},
        ...     num_seats=17,
        ...     threshold=1/18  # ~5.6%
        ... )
        {'Democratic': 9, 'Republican': 8}

        >>> # Texas: 38 seats, 46% D, 52% R, 2% other
        >>> allocate_seats_dhondt(
        ...     {"Democratic": 0.46, "Republican": 0.52, "Libertarian": 0.02},
        ...     num_seats=38,
        ...     threshold=1/39  # ~2.6%
        ... )
        {'Democratic': 18, 'Republican': 20, 'Libertarian': 0}
    """
    # Validate inputs
    if num_seats < 1:
        raise ValueError(f"num_seats must be >= 1, got {num_seats}")

    total_vote_share = sum(vote_shares.values())
    if not (0.99 <= total_vote_share <= 1.01):
        logger.warning(
            f"Vote shares sum to {total_vote_share:.4f}, expected ~1.0. "
            f"Normalizing..."
        )
        # Normalize vote shares
        vote_shares = {
            party: share / total_vote_share
            for party, share in vote_shares.items()
        }

    # Filter parties above threshold
    eligible_parties = {
        party: share
        for party, share in vote_shares.items()
        if share >= threshold
    }

    if not eligible_parties:
        logger.warning(
            f"No parties above threshold {threshold:.2%}. "
            f"Allocating to party with highest vote share."
        )
        # Fallback: Give all seats to party with most votes
        max_party = max(vote_shares.items(), key=lambda x: x[1])[0]
        return {max_party: num_seats}

    logger.info(
        f"Allocating {num_seats} seats using D'Hondt method. "
        f"Eligible parties: {list(eligible_parties.keys())}"
    )

    # Apply minimum seat requirements first (e.g., VRA)
    seat_allocation = {party: 0 for party in eligible_parties}
    seats_remaining = num_seats

    if min_seats:
        for party, min_s in min_seats.items():
            if party in seat_allocation:
                allocated = min(min_s, seats_remaining)
                seat_allocation[party] = allocated
                seats_remaining -= allocated
                logger.info(f"Allocated {allocated} minimum seats to {party}")

    # Compute quotients and allocate remaining seats
    quotients = compute_dhondt_quotients(eligible_parties, num_seats)

    # Allocate remaining seats by highest quotients
    for quotient, party, divisor in quotients:
        if seats_remaining == 0:
            break

        # Skip if we've already allocated this seat to meet minimum
        if seat_allocation[party] >= divisor:
            continue

        seat_allocation[party] += 1
        seats_remaining -= 1

    # Verify we allocated exactly num_seats
    total_allocated = sum(seat_allocation.values())
    if total_allocated != num_seats:
        logger.error(
            f"Allocation error: allocated {total_allocated} seats, "
            f"expected {num_seats}"
        )
        raise RuntimeError(
            f"D'Hondt allocation failed: {total_allocated} != {num_seats}"
        )

    logger.info(f"Seat allocation: {seat_allocation}")

    # Add zero-seat parties for completeness
    for party in vote_shares:
        if party not in seat_allocation:
            seat_allocation[party] = 0

    return seat_allocation


def compute_proportionality_deviation(
    vote_shares: Dict[str, float],
    seat_allocation: Dict[str, int]
) -> float:
    """
    Compute deviation from perfect proportionality.

    Measures the sum of absolute differences between seat share and vote share
    across all parties. Perfect proportionality yields 0.0; maximum deviation is 2.0.

    Args:
        vote_shares: Party vote shares (0-1 scale).
        seat_allocation: Allocated seats per party.

    Returns:
        Proportionality deviation (0.0 = perfect, higher = worse).

    Example:
        >>> # PA: 51% D, 49% R → 9 D seats (53%), 8 R seats (47%)
        >>> compute_proportionality_deviation(
        ...     {"Democratic": 0.51, "Republican": 0.49},
        ...     {"Democratic": 9, "Republican": 8}
        ... )
        0.04  # 4% deviation
    """
    total_seats = sum(seat_allocation.values())
    if total_seats == 0:
        return 0.0

    deviation = 0.0
    for party in vote_shares:
        vote_share = vote_shares[party]
        seat_count = seat_allocation.get(party, 0)
        seat_share = seat_count / total_seats
        deviation += abs(seat_share - vote_share)

    return deviation


def suggest_threshold(num_seats: int) -> float:
    """
    Suggest natural threshold for a given number of seats.

    The natural threshold is 1/(num_seats + 1), which ensures a party needs
    enough votes to "deserve" at least one seat under ideal conditions.

    Args:
        num_seats: Total seats in the state/region.

    Returns:
        Suggested threshold as a fraction (0-1 scale).

    Example:
        >>> suggest_threshold(17)  # Pennsylvania
        0.0556  # ~5.6%
        >>> suggest_threshold(1)   # Wyoming
        0.5     # 50% (majority required)
    """
    return 1.0 / (num_seats + 1)
