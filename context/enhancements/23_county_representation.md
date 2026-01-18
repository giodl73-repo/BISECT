# Enhancement 23: Direct County Representation

**Status**: 📋 PLANNED
**Priority**: Low (Research/Experimental)
**Proposed**: January 15, 2026
**Estimated Complexity**: Very High (15-20 hours)

## Current State

The current redistricting system:
- Allocates congressional seats to states using Huntington-Hill apportionment
- Creates districts within each state independently
- Districts can span multiple counties or split large counties
- Counties have no guaranteed representation

Large urban counties (e.g., Los Angeles County ~10M population) may be split across 10+ congressional districts, potentially diluting county-level political power.

## Goal

**Research Experiment**: Implement a "direct county representation" model where:

1. **First-round apportionment**: Any county with population ≥ average national district size (~769,000 for 2020) automatically gets 1 district
2. **Second-round apportionment**: Run Huntington-Hill on modified entities:
   - States (minus population of counties that got direct representation)
   - Counties that got direct representation (treated as pseudo-states with 1 district fixed)
   - Allocate remaining districts among these mixed entities
3. **Redistricting**: Apply recursive bisection to:
   - Full states for their allocated districts
   - State remainders (state minus large counties) for their allocated districts
   - Large counties get whole-county single district

**Research questions**:
- Does this preserve county identity/power?
- How does compactness change?
- How many counties qualify for direct representation?
- What are partisan implications?

## Implementation Plan

### Phase 1: County Data Integration
- Load county boundary data for all states
- Map census tracts to counties (FIPS codes)
- Calculate county populations for 2020 (and 2010, 2000 if possible)
- Identify counties ≥ average district population
- Files to create:
  - `scripts/experimental/analyze_county_populations.py` - County stats

### Phase 2: Two-Stage Apportionment
- Implement first-round: Allocate 1 district to qualifying counties
- Calculate remaining seats (435 minus counties with direct representation)
- Implement second-round: Huntington-Hill on modified entities:
  - States with adjusted populations (state minus direct-rep counties)
  - Direct-rep counties as fixed 1-district entities
- Output: Seat allocation table showing state vs county breakdown
- Files to create:
  - `scripts/experimental/county_apportionment.py` - Two-stage allocation
- Files to modify:
  - `scripts/apportionment/huntington_hill.py` - Support mixed entity types

### Phase 3: Geographic Partitioning
- For states: Subtract direct-rep county geometries from state geometry
- Create "state remainder" polygons (state minus large counties)
- Handle non-contiguous state remainders (e.g., if large county in middle)
- Build adjacency graphs for:
  - State remainders
  - Full states (for states without direct-rep counties)
- Files to create:
  - `scripts/experimental/partition_state_counties.py` - Geographic split

### Phase 4: Modified Redistricting
- Run redistricting algorithm on:
  - State remainders with their allocated districts
  - Full states with their allocated districts
  - Direct-rep counties: No redistricting (entire county = 1 district)
- Handle edge cases:
  - State becomes non-contiguous after removing county
  - State has multiple large counties
  - State remainder is tiny after removing large counties
- Files to create:
  - `scripts/experimental/county_redistricting.py` - Main algorithm

### Phase 5: Analysis & Comparison
- Generate maps showing:
  - Direct-rep counties (highlighted)
  - District boundaries in state remainders
  - Comparison to standard state-based approach
- Compute metrics:
  - Number of counties with direct representation
  - Compactness statistics
  - Population balance
  - Partisan lean impacts
- Files to create:
  - `scripts/experimental/analyze_county_system.py` - Metrics
  - `scripts/experimental/visualize_county_system.py` - Maps

### Phase 6: Documentation
- Document two-stage apportionment methodology
- Explain county vs state representation trade-offs
- Clarify constitutional challenges
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `scripts/experimental/analyze_county_populations.py` - County stats and thresholds
2. `scripts/experimental/county_apportionment.py` - Two-stage Huntington-Hill
3. `scripts/experimental/partition_state_counties.py` - Geographic splitting
4. `scripts/experimental/county_redistricting.py` - Modified redistricting algorithm
5. `scripts/experimental/analyze_county_system.py` - Comparative metrics
6. `scripts/experimental/visualize_county_system.py` - Map generation
7. `data/experimental/county_boundaries/` - County shapefiles (if not already available)

### Modified Files
1. `scripts/apportionment/huntington_hill.py` - Support mixed entity types (states + counties)
2. `src/apportionment/data/adjacency.py` - Handle state remainder geometries
3. `ARCHITECTURE.md` - Document experimental county system

## Testing Plan

1. **County identification**: Validate which counties qualify (2020)
   ```bash
   python scripts/experimental/analyze_county_populations.py --year 2020 --threshold 769000
   ```

2. **Apportionment validation**: Verify totals add to 435
   ```bash
   python scripts/experimental/county_apportionment.py --year 2020 --validate
   ```

3. **Single-state test**: Test on state with large county (e.g., California with Los Angeles)
   ```bash
   python scripts/experimental/county_redistricting.py --state CA --year 2020 --version test
   ```

4. **Non-contiguity handling**: Test on state where county removal creates islands
   ```bash
   python scripts/experimental/county_redistricting.py --state NY --year 2020 --version test
   ```

5. **Full national run**: All states with county system
   ```bash
   python scripts/experimental/county_redistricting.py --year 2020 --version county_v1
   ```

6. **Quantitative validation**:
   - Count direct-rep counties
   - Verify 435 total districts
   - Check population balance
   - Compare compactness to baseline

## Benefits

- **Research contribution**: Novel apportionment model preserving county identity
- **Federalism analysis**: Quantify trade-offs between state and county representation
- **Urban county power**: Explores alternative to county splitting
- **Academic publication**: Unique approach to multi-level representation
- **Policy discussion**: Inform debate on representation models

## Success Criteria

- [ ] County populations computed for all U.S. counties
- [ ] Counties ≥769K identified and validated
- [ ] Two-stage apportionment produces 435 total districts
- [ ] State remainder geometries created correctly
- [ ] Non-contiguous state remainders handled gracefully
- [ ] Redistricting completes for all states/remainders
- [ ] All districts within ±0.5% of target population
- [ ] Comparison report generated vs standard state-based model
- [ ] Documentation explains constitutional limitations

## Estimated Complexity

**Effort**: 15-20 hours
- Phase 1 (County Data): 2-3 hours (data integration)
- Phase 2 (Apportionment): 3-4 hours (complex two-stage logic)
- Phase 3 (Geographic Partition): 4-5 hours (geometry operations, edge cases)
- Phase 4 (Redistricting): 3-4 hours (modified algorithm)
- Phase 5 (Analysis): 2-3 hours (metrics and visualization)
- Phase 6 (Documentation): 1 hour

**Risk**: High
- **Non-contiguity**: Removing county may fragment state (e.g., NYC from NY)
- **Edge cases**: States with multiple large counties, tiny remainders
- **Data complexity**: County boundaries, tract-to-county mappings
- **Apportionment math**: Two-stage Huntington-Hill is complex

**Dependencies**:
- County boundary shapefiles (may need to download from Census TIGER/Line)
- Tract-to-county FIPS mapping

## Implementation Notes

### Expected Direct-Rep Counties (2020 data)

Based on ~769K threshold, counties likely to qualify:
- Los Angeles County, CA (~10M): 13 districts in standard model → 1 in county model
- Cook County, IL (~5M): 7 districts → 1
- Harris County, TX (~4.7M): 6 districts → 1
- Maricopa County, AZ (~4.4M): 6 districts → 1
- San Diego County, CA (~3.3M): 4 districts → 1
- Orange County, CA (~3.2M): 4 districts → 1
- Miami-Dade County, FL (~2.7M): 3 districts → 1
- Kings County (Brooklyn), NY (~2.7M): 3 districts → 1
- Queens County, NY (~2.4M): 3 districts → 1
- ... (~15-25 total counties estimated)

This would allocate ~15-25 districts to counties, leaving ~410-420 districts for states.

### Non-Contiguity Challenges

**New York example**:
- Remove NYC counties (Kings, Queens, Bronx, New York, Richmond): ~8.3M population
- Leaves Long Island, Westchester, and Upstate NY as separate non-contiguous pieces
- **Solution**: Treat as multiple independent regions for redistricting

**Illinois example**:
- Remove Cook County (Chicago): ~5M population
- Leaves downstate Illinois relatively intact

### Huntington-Hill Modification

Standard H-H:
```
Priority = P / sqrt(n * (n+1))
```

Modified two-stage:
1. **Round 1**: Identify counties with P ≥ 769K, allocate n=1 each
2. **Round 2**: For remaining seats:
   - States: Use adjusted population P_state - P_direct_counties
   - Direct counties: Already have n=1, do not participate in round 2

### Constitutional Issues

- Violates state sovereignty (counties treated as equals to states)
- May violate "one person, one vote" if county thresholds create imbalances
- Would require constitutional amendment
- Purely theoretical/academic model

## Related Enhancements

- Enhancement 22: National Redistricting (alternative constraint model)
- Enhancement 4: Urban Metro Area Maps (county-level visualization patterns)

## Research Value

This model explores:
- **Multi-level federalism**: Balancing state and county representation
- **Urban vs rural power**: Direct county representation favors large urban counties
- **Representation trade-offs**: County identity vs district compactness
- **Apportionment theory**: Two-stage allocation mechanisms
