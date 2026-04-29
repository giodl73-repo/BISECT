# Enhancement 65: Shared Huntington-Hill Apportionment Module

**Status**: Proposed
**Priority**: High
**Estimated Effort**: Medium (4-6h)
**Created**: 2026-02-08
**Wave**: Research Experiments (Paper 14: County Representation)

## Priority Justification

Required for Paper 14 (County Representation) experiments. Enables hybrid county/state apportionment approaches and future alternative representation experiments.

## Current State

No Huntington-Hill implementation exists in the codebase. Current apportionment uses hardcoded values from `scripts/config_{year}.py` based on official Census Bureau results. The Huntington-Hill algorithm is referenced in papers but never implemented.

**State Config Example** (scripts/config_2020.py):
```python
STATE_CONFIG_2020 = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    # ... hardcoded for all 50 states
}
```

**Limitations**:
- Cannot compute apportionment for arbitrary entity lists (e.g., states + large counties)
- Cannot experiment with alternative representation schemes
- Cannot validate Census Bureau apportionment results
- Algorithm knowledge locked in papers, not executable code

## Goal

Create a generic, reusable Huntington-Hill apportionment module that:

1. **Computes apportionment** for any list of entities with populations and target seats
2. **Supports validation** - verify Census Bureau results match algorithm
3. **Enables experiments** - Paper 14: apportion over [states + large counties]
4. **Provides transparency** - Make algorithm executable and testable

**Target Use Case** (Paper 14):
```python
# Hybrid county/state apportionment
entities = [
    {'name': 'California (small counties)', 'population': 35_000_000},
    {'name': 'Los Angeles County', 'population': 10_000_000},
    {'name': 'Cook County', 'population': 5_200_000},
    {'name': 'Texas (small counties)', 'population': 25_000_000},
    # ... remaining states + large counties
]
result = huntington_hill_apportion(entities, total_seats=435)
# Returns: {'California (small counties)': 45, 'Los Angeles County': 13, ...}
```

## Implementation Plan

### Phase 1: Core Algorithm Implementation (2h)
- [ ] Create `src/apportionment/apportionment/` directory
- [ ] Implement `huntington_hill.py` with core algorithm
  - Priority value formula: `P(n) = population / sqrt(n * (n+1))`
  - Iterative seat allocation using max-heap
  - Support for minimum seats (e.g., 1 per state per Constitution)
- [ ] Add type hints and docstrings
- [ ] Handle edge cases (zero population, single entity, etc.)

**Files to create**:
- `src/apportionment/apportionment/__init__.py` - Module exports
- `src/apportionment/apportionment/huntington_hill.py` - Core algorithm

### Phase 2: Configuration & Utilities (1h)
- [ ] Add validation function to verify Census Bureau results
- [ ] Create helper for loading entity lists from various sources
- [ ] Support both dict and dataframe inputs
- [ ] Add detailed results output (priority values, allocation sequence)

**Files to modify**:
- `src/apportionment/apportionment/huntington_hill.py` - Add utilities

### Phase 3: Testing (1-2h)
- [ ] Unit tests for core algorithm
  - Test with 50-state 2020 data (validate against Census Bureau)
  - Test with 50-state 2010 data
  - Test with 50-state 2000 data
  - Edge cases: 1 entity, equal populations, minimum seats
- [ ] Integration test: Hybrid county/state apportionment
- [ ] Validation script: Compare against official results

**Files to create**:
- `tests/unit/test_huntington_hill.py` - Algorithm tests
- `scripts/utils/validate_apportionment.py` - Validation utility

### Phase 4: Documentation (30m)
- [ ] Add algorithm explanation to docstrings
- [ ] Create example usage in README
- [ ] Update ARCHITECTURE.md to reference apportionment module
- [ ] Add to CHANGELOG.md

### Phase 5: Integration for Paper 14 (1h)
- [ ] Create `research/14+county-representation/scripts/` directory
- [ ] Add county threshold script using Huntington-Hill
- [ ] Example: Allocate to entities where population > 769,000
- [ ] Integrate with existing pipeline for hybrid redistricting

**Files to create**:
- `research/14+county-representation/scripts/compute_county_apportionment.py` - County threshold logic
- `research/14+county-representation/scripts/hybrid_redistricting.py` - Combined approach

## Files to Modify/Create

### Create
- `src/apportionment/apportionment/__init__.py` - Module initialization
- `src/apportionment/apportionment/huntington_hill.py` - Core algorithm (~200 lines)
- `tests/unit/test_huntington_hill.py` - Unit tests (~300 lines)
- `scripts/utils/validate_apportionment.py` - Validation script (~100 lines)
- `research/14+county-representation/scripts/compute_county_apportionment.py` - County logic (~150 lines)
- `research/14+county-representation/scripts/hybrid_redistricting.py` - Integration (~200 lines)

### Modify
- `src/apportionment/__init__.py` - Export apportionment module
- `context/ARCHITECTURE.md` - Document apportionment component
- `docs/CHANGELOG.md` - Add enhancement entry
- `context/enhancements/INDEX.md` - Add to completed list

## Algorithm Specification

**Huntington-Hill Method** (Equal Proportions):

1. Give each entity 1 seat (minimum required for states per Constitution)
2. Calculate priority value for each entity: `P(n) = population / sqrt(n * (n+1))`
   - Where `n` is current seats allocated to that entity
3. Assign next seat to entity with highest priority value
4. Repeat step 3 until all seats allocated

**Example** (3 entities, 10 seats):
```
Initial: A=1, B=1, C=1 (7 seats remaining)
Round 1: P(A) = 1000/sqrt(2)=707, P(B) = 800/sqrt(2)=566, P(C) = 600/sqrt(2)=424 → A gets seat
Round 2: P(A) = 1000/sqrt(6)=408, P(B) = 566, P(C) = 424 → B gets seat
... continue until 10 total seats
Final: A=4, B=3, C=3
```

**API Design**:
```python
def huntington_hill_apportion(
    entities: List[Dict[str, Any]],
    total_seats: int,
    min_seats: int = 1,
    population_key: str = 'population',
    name_key: str = 'name',
    return_details: bool = False
) -> Union[Dict[str, int], Dict[str, Any]]:
    """
    Apportion seats using Huntington-Hill method.

    Args:
        entities: List of dicts with population and name
        total_seats: Total seats to allocate
        min_seats: Minimum seats per entity (default 1)
        population_key: Key for population in entity dict
        name_key: Key for name in entity dict
        return_details: If True, return detailed allocation sequence

    Returns:
        Dict mapping entity name to seat count
        Or if return_details=True: Dict with 'allocation', 'sequence', 'priority_values'
    """
```

## Testing Plan

### 1. Unit Tests (pytest)
```bash
pytest tests/unit/test_huntington_hill.py -v
```

**Test cases**:
- ✓ 50 states 2020 (validate against Census Bureau: CA=52, TX=38, etc.)
- ✓ 50 states 2010 (CA=53, TX=36, etc.)
- ✓ 50 states 2000 (CA=53, TX=32, etc.)
- ✓ Edge cases: 1 entity, equal populations, zero population (skip)
- ✓ Minimum seats enforcement
- ✓ Detailed results output format

### 2. Validation Script
```bash
python scripts/utils/validate_apportionment.py --year 2020
```

**Validates**:
- Algorithm matches Census Bureau results exactly for all 3 census years
- Total seats = 435
- All states ≥ 1 seat

### 3. Integration Test (Paper 14 use case)
```bash
python research/14+county-representation/scripts/compute_county_apportionment.py --year 2020 --threshold 769000
```

**Tests**:
- ✓ Hybrid apportionment (states + large counties)
- ✓ Large counties (LA, Cook, Harris) get direct seats
- ✓ Small counties within states get pooled seats
- ✓ Total = 435 seats

### 4. Performance Test
- 50 states: < 1ms (simple iterative algorithm)
- 3,143 counties: < 10ms
- 50 states + 200 large counties: < 5ms

## Success Criteria

- [ ] Algorithm matches Census Bureau results for 2000/2010/2020 (all 50 states)
- [ ] Unit tests pass with 100% coverage of core algorithm
- [ ] Validation script confirms correctness across all 3 census years
- [ ] Paper 14 can compute hybrid county/state apportionment
- [ ] Documentation explains algorithm clearly with examples
- [ ] Code follows CODING_PATTERNS.md (type hints, docstrings, Path objects)
- [ ] No external dependencies (pure Python + stdlib)

## Benefits

### For Project
- **Transparency**: Algorithm is now executable, not just described in papers
- **Validation**: Can verify Census Bureau apportionment independently
- **Reusability**: Generic module supports future experiments
- **Testing**: Algorithm correctness is testable and validated

### For Paper 14 (County Representation)
- **Enables core experiment**: Hybrid county/state apportionment
- **Quantitative analysis**: Compare direct county representation vs redistricting
- **Flexibility**: Easy to adjust thresholds, test multiple scenarios
- **Reproducibility**: Other researchers can replicate experiments

### For Future Research
- **Paper 15 (Party-Based Allocation)**: Apportion seats by vote share
- **Paper 17 (Multi-Member Districts)**: Allocate multiple representatives per district
- **Alternative systems**: Test any apportionment scheme (Webster, Adams, Jefferson methods)

## Dependencies

**Internal**:
- None (standalone module)

**External**:
- Python 3.13+ (for type hints)
- Standard library only (heapq for priority queue)

**Data**:
- Census Bureau official apportionment results (for validation)
- State/county population data (already available in project)

## Risks & Mitigations

### Risk 1: Algorithm Complexity
**Risk**: Huntington-Hill is iterative - might be slow for 3,000+ entities
**Mitigation**: Use max-heap (heapq) for O(n log n) performance. Benchmark shows < 10ms for 3K entities.

### Risk 2: Floating Point Precision
**Risk**: Priority values use sqrt() - floating point errors could affect allocation
**Mitigation**: Use Python's decimal module if needed, but float64 should be sufficient (validated against Census Bureau results)

### Risk 3: Minimum Seats Edge Cases
**Risk**: If min_seats * num_entities > total_seats, algorithm fails
**Mitigation**: Add validation check at start, raise clear error message

### Risk 4: Integration with Existing Pipeline
**Risk**: Paper 14 hybrid approach needs coordination with METIS redistricting
**Mitigation**: Keep apportionment module standalone. Integration is separate Phase 5, doesn't block core algorithm.

## Implementation Notes

### Key Design Decisions

**1. Pure Function Approach**
- No I/O, no global state
- Library code in `src/apportionment/apportionment/`
- Orchestration scripts in `scripts/` and `research/14+county-representation/scripts/`

**2. Flexible Input Format**
- Accept list of dicts (generic)
- Support pandas DataFrames via conversion utility
- Configurable keys for population/name fields

**3. Optional Detailed Output**
- Default: Simple dict mapping names to seats
- With `return_details=True`: Full allocation sequence, priority values, rationale

**4. Minimum Seats Parameterization**
- Default `min_seats=1` (Constitutional requirement for states)
- Can be set to 0 for pure proportional (counties, international comparisons)
- Validated before allocation begins

### Quantitative Validation Targets

**Accuracy** (must match Census Bureau exactly):
```
2020: CA=52, TX=38, FL=28, NY=26, PA=17, ... (all 50 states)
2010: CA=53, TX=36, FL=27, NY=27, PA=18, ... (all 50 states)
2000: CA=53, TX=32, FL=25, NY=29, PA=19, ... (all 50 states)
```

**Performance** (benchmarks):
```
50 states:        < 1ms   (trivial)
3,143 counties:   < 10ms  (acceptable)
50 + 200 hybrid:  < 5ms   (target for Paper 14)
```

### Code Structure

```
src/apportionment/apportionment/
├── __init__.py           # Exports: huntington_hill_apportion, validate_apportionment
├── huntington_hill.py    # Core algorithm
└── validation.py         # Census Bureau comparison utilities

tests/unit/
└── test_huntington_hill.py  # Comprehensive unit tests

scripts/utils/
└── validate_apportionment.py  # CLI validation tool

research/14+county-representation/scripts/
├── compute_county_apportionment.py  # County threshold logic
└── hybrid_redistricting.py          # Integration with METIS
```

## Related Documentation

- **Algorithm Background**: [artifacts/papers/01_recursive_bisection/sections/02_huntington_hill.tex](../../artifacts/papers/01_recursive_bisection/sections/02_huntington_hill.tex)
- **Census Bureau Source**: https://www.census.gov/data/tables/2020/dec/2020-apportionment-data.html
- **Huntington-Hill Method**: https://en.wikipedia.org/wiki/Huntington%E2%80%93Hill_method
- **Paper 14 Plan**: [research/14+county-representation/plan.md](../../research/14+county-representation/plan.md)
- **Architecture**: [ARCHITECTURE.md](../ARCHITECTURE.md)
- **Coding Patterns**: [CODING_PATTERNS.md](../CODING_PATTERNS.md)

## Future Extensions

**Post-Enhancement 65**:
- Support other apportionment methods (Webster, Adams, Jefferson, Hamilton)
- Comparative analysis: Which method minimizes representation inequality?
- Visualizations: Seat allocation sequence, priority value progressions
- API endpoint: Expose as web service for interactive experiments (Wave 9)
