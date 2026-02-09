"""
Huntington-Hill Apportionment Algorithm (Equal Proportions Method)

Used by the U.S. Census Bureau to allocate congressional seats to states.
Can be applied to any list of entities with populations.

References:
- https://www.census.gov/data/tables/2020/dec/2020-apportionment-data.html
- https://en.wikipedia.org/wiki/Huntington%E2%80%93Hill_method
"""

import heapq
from math import sqrt
from typing import Dict, List, Optional, Union, Any


def priority_value(population: int, current_seats: int) -> float:
    """
    Calculate Huntington-Hill priority value for next seat allocation.

    Formula: P(n) = population / sqrt(n * (n + 1))

    Where n is the current number of seats allocated to the entity.
    Higher priority value = more deserving of next seat.

    Args:
        population: Entity's population
        current_seats: Current number of seats allocated

    Returns:
        Priority value (higher = more deserving)
    """
    if population == 0:
        return 0.0

    n = current_seats
    return population / sqrt(n * (n + 1))


def apportion(
    entities: List[Dict[str, Any]],
    total_seats: int,
    min_seats: int = 1,
    population_key: str = 'population',
    name_key: str = 'name',
    return_details: bool = False
) -> Union[Dict[str, int], Dict[str, Any]]:
    """
    Apportion seats using Huntington-Hill method (Equal Proportions).

    Algorithm:
    1. Give each entity min_seats
    2. Calculate priority value for each: P(n) = pop / sqrt(n * (n+1))
    3. Assign next seat to entity with highest priority
    4. Repeat until all seats allocated

    Args:
        entities: List of dicts with population and name
        total_seats: Total seats to allocate
        min_seats: Minimum seats per entity (default 1 for U.S. states)
        population_key: Key for population in entity dict
        name_key: Key for name in entity dict
        return_details: If True, return allocation sequence and priority values

    Returns:
        Dict mapping entity name to seat count
        Or if return_details=True: Dict with 'allocation', 'sequence', 'final_priorities'

    Raises:
        ValueError: If min_seats * len(entities) > total_seats
        ValueError: If any population is negative

    Example:
        >>> entities = [
        ...     {'name': 'California', 'population': 39_538_223},
        ...     {'name': 'Texas', 'population': 29_145_505},
        ...     {'name': 'Vermont', 'population': 643_077}
        ... ]
        >>> result = apportion(entities, total_seats=100, min_seats=1)
        >>> result['California']  # Will get most seats (proportional to population)
    """
    # Validation
    if not entities:
        return {} if not return_details else {'allocation': {}, 'sequence': [], 'final_priorities': {}}

    num_entities = len(entities)
    min_seats_total = min_seats * num_entities

    if min_seats_total > total_seats:
        raise ValueError(
            f"Cannot allocate {min_seats} minimum seats to {num_entities} entities "
            f"({min_seats_total} total) with only {total_seats} seats available"
        )

    for entity in entities:
        pop = entity.get(population_key, 0)
        if pop < 0:
            raise ValueError(f"Entity '{entity.get(name_key)}' has negative population: {pop}")

    # Initialize: Give each entity min_seats
    allocation = {entity[name_key]: min_seats for entity in entities}
    seats_remaining = total_seats - min_seats_total

    # Track allocation sequence if details requested
    sequence = []

    # Build max-heap of priority values (use negative for max-heap)
    # Heap entry: (-priority, entity_name, population, current_seats)
    heap = []
    for entity in entities:
        name = entity[name_key]
        pop = entity[population_key]
        current = allocation[name]
        priority = priority_value(pop, current)
        heapq.heappush(heap, (-priority, name, pop, current))

    # Allocate remaining seats
    while seats_remaining > 0:
        # Get entity with highest priority
        neg_priority, name, pop, current = heapq.heappop(heap)

        # Allocate seat
        allocation[name] += 1
        seats_remaining -= 1

        if return_details:
            sequence.append({
                'seat_number': total_seats - seats_remaining,
                'entity': name,
                'priority': -neg_priority,
                'seats_before': current,
                'seats_after': current + 1
            })

        # Recalculate priority with new seat count and push back
        new_current = current + 1
        new_priority = priority_value(pop, new_current)
        heapq.heappush(heap, (-new_priority, name, pop, new_current))

    if return_details:
        # Calculate final priority values (what would be used for next seat)
        final_priorities = {}
        for entity in entities:
            name = entity[name_key]
            pop = entity[population_key]
            current = allocation[name]
            final_priorities[name] = priority_value(pop, current)

        return {
            'allocation': allocation,
            'sequence': sequence,
            'final_priorities': final_priorities
        }

    return allocation


def validate_census_apportionment(
    year: int,
    state_populations: Dict[str, int],
    expected_allocation: Dict[str, int]
) -> Dict[str, Any]:
    """
    Validate that Huntington-Hill algorithm matches Census Bureau apportionment.

    Args:
        year: Census year (2000, 2010, or 2020)
        state_populations: Dict mapping state abbreviation to population
        expected_allocation: Dict mapping state abbreviation to expected seats

    Returns:
        Dict with validation results:
        - 'matches': bool - True if algorithm matches expected
        - 'differences': List of states with mismatches
        - 'computed_allocation': Dict of computed seat allocations

    Example:
        >>> state_pops = {'CA': 39_538_223, 'TX': 29_145_505, ...}
        >>> expected = {'CA': 52, 'TX': 38, ...}
        >>> result = validate_census_apportionment(2020, state_pops, expected)
        >>> result['matches']
        True
    """
    # Convert to entity list format
    entities = [
        {'name': state, 'population': pop}
        for state, pop in state_populations.items()
    ]

    # Run algorithm
    computed = apportion(entities, total_seats=435, min_seats=1)

    # Compare results
    differences = []
    for state in state_populations:
        expected_seats = expected_allocation.get(state, 0)
        computed_seats = computed.get(state, 0)
        if expected_seats != computed_seats:
            differences.append({
                'state': state,
                'expected': expected_seats,
                'computed': computed_seats,
                'difference': computed_seats - expected_seats
            })

    return {
        'year': year,
        'matches': len(differences) == 0,
        'differences': differences,
        'computed_allocation': computed,
        'total_seats_computed': sum(computed.values()),
        'total_seats_expected': sum(expected_allocation.values())
    }
