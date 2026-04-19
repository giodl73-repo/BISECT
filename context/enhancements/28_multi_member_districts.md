# Enhancement 28: Multi-Member Districts

**Status**: 📋 PLANNED
**Priority**: Research
**Proposed**: January 15, 2026
**Estimated Complexity**: High (12-16 hours)
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

The current redistricting system:
- Creates single-member districts (1 representative per district)
- Each district has population ≈ 769,000 (2020)
- Winner-take-all within district (plurality winner gets seat)
- Minority parties within district get no representation

## Goal

**Research Experiment**: Implement "multi-member districts" where:

1. **Multiple representatives per district**: Each district elects 2-3 representatives
2. **Proportional allocation within district**: Seats allocated to parties based on vote share
3. **Larger geographic districts**: Districts have population ≈ 1.5-2.3M (2-3× single-member)
4. **Minority representation**: 60% D + 40% R district → 1 D rep + 1 R rep (in 2-member district)

**Example**: California (52 seats, 2020)
- **Single-member** (current): 52 districts of ~769K population each
- **Two-member**: 26 districts of ~1.54M population each, 2 reps per district
- **Three-member**: 17 districts of ~2.31M population each, 3 reps per district

**District vote allocation**:
- **Two-member district** with 60% D, 40% R vote:
  - 1 Democratic representative (60% → rounds to 1 of 2 seats)
  - 1 Republican representative (40% → rounds to 1 of 2 seats)
- **Three-member district** with 55% D, 38% R, 7% L vote:
  - 2 Democratic representatives (55% → 1.65 rounds to 2 of 3 seats)
  - 1 Republican representative (38% → 1.14 rounds to 1 of 3 seats)
  - 0 Libertarian representatives (7% → 0.21 rounds to 0 seats)

**Research questions**:
- Does multi-member improve minority representation?
- How does proportionality within districts affect overall outcomes?
- What are compactness implications with larger districts?
- What district size (2 vs 3 members) works best?

## Implementation Plan

### Phase 1: District Size Calculation
- Determine new district sizes for multi-member system
- For each state:
  - Two-member: `num_districts_2 = state_seats / 2` (round to ensure even)
  - Three-member: `num_districts_3 = state_seats / 3` (round appropriately)
- Handle states with odd total seats (e.g., 7 seats → 3 two-member + 1 three-member?)
- Calculate target population per district (2× or 3× single-member target)
- Files to create:
  - `scripts/experimental/compute_multi_member_districts.py` - District counts

### Phase 2: Multi-Member Redistricting
- Adapt recursive bisection for larger districts:
  - Input: State, target number of multi-member districts
  - Target population: 1.54M (2-member) or 2.31M (3-member)
  - Output: District assignments for census tracts
- Run redistricting for both scenarios:
  - Two-member districts for all states
  - Three-member districts for all states
- Files to create:
  - `scripts/experimental/multi_member_redistricting.py` - Adapted algorithm
- Files to modify:
  - `src/apportionment/partition/recursive_bisection.py` - Support larger target populations

### Phase 3: Election Results Integration
- Load 2020 election results by census tract (if available)
- Aggregate to multi-member districts
- Calculate party vote shares within each district
- Files to create:
  - `scripts/experimental/aggregate_district_elections.py` - Vote aggregation

### Phase 4: Seat Allocation Within Districts
- For each district, allocate 2 or 3 seats to parties:
  - **Two-member**: Round vote shares to nearest integer (0, 1, or 2)
  - **Three-member**: Allocate 3 seats using largest remainder or d'Hondt method
- Handle edge cases:
  - 100% D district → 2D + 0R (or 3D + 0R + 0O)
  - 50-50 split → 1D + 1R (fair)
  - Three-way split 45-40-15 → 1D + 1R + 1L (in 3-member)
- Files to create:
  - `scripts/experimental/allocate_district_seats.py` - Proportional allocation

### Phase 5: Visualization
- Generate multi-member district maps:
  - Show district boundaries (larger than single-member)
  - Color code by seat allocation (e.g., purple for 1D+1R mixed districts)
  - Display seat counts per district
- Create comparison maps:
  - Single-member vs two-member vs three-member systems
- Files to create:
  - `scripts/experimental/visualize_multi_member_districts.py` - State maps
  - `scripts/experimental/visualize_seat_allocation.py` - Seat composition

### Phase 6: Analysis & Comparison
- Compute metrics:
  - **Proportionality**: Seats vs votes at state and national level
  - **Minority representation**: Number of districts with mixed party delegations
  - **Compactness**: Compare to single-member districts
  - **Wasted votes**: Reduction vs single-member system
- Compare systems:
  - Single-member (current)
  - Two-member (proportional within district)
  - Three-member (proportional within district)
- Files to create:
  - `scripts/experimental/analyze_multi_member_system.py` - Metrics

### Phase 7: Documentation
- Document multi-member district methodology
- Explain proportional allocation within districts
- Clarify constitutional challenges
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `scripts/experimental/compute_multi_member_districts.py` - District size calculation
2. `scripts/experimental/multi_member_redistricting.py` - Adapted redistricting algorithm
3. `scripts/experimental/aggregate_district_elections.py` - Vote aggregation
4. `scripts/experimental/allocate_district_seats.py` - Proportional seat allocation
5. `scripts/experimental/visualize_multi_member_districts.py` - District maps
6. `scripts/experimental/visualize_seat_allocation.py` - Seat composition charts
7. `scripts/experimental/analyze_multi_member_system.py` - Comparative metrics

### Modified Files
1. `src/apportionment/partition/recursive_bisection.py` - Support variable target populations
2. `scripts/config_2020.py` - Add multi-member district counts per state
3. `ARCHITECTURE.md` - Document experimental multi-member system

## Testing Plan

1. **District count validation**: Verify all states have valid multi-member counts
   ```bash
   python scripts/experimental/compute_multi_member_districts.py --year 2020 --validate
   ```

2. **Single-state redistricting**: Test on medium state (e.g., Alabama with 7 seats)
   ```bash
   # Two-member: 3 districts (6 seats) + 1 single-member (1 seat)? Or 4 districts (uneven)?
   python scripts/experimental/multi_member_redistricting.py --state AL --members 2 --year 2020
   ```

3. **Large state test**: Test on California (52 seats)
   ```bash
   # Two-member: 26 districts
   python scripts/experimental/multi_member_redistricting.py --state CA --members 2 --year 2020
   # Three-member: 17 districts (51 seats) + 1 single-member (1 seat)?
   python scripts/experimental/multi_member_redistricting.py --state CA --members 3 --year 2020
   ```

4. **Seat allocation test**: Verify proportional allocation within districts
   ```bash
   python scripts/experimental/allocate_district_seats.py --state CA --members 2 --year 2020 --validate
   ```

5. **Full national run**: All states, both 2-member and 3-member scenarios
   ```bash
   python scripts/experimental/multi_member_redistricting.py --year 2020 --members 2 --version multi2_v1
   python scripts/experimental/multi_member_redistricting.py --year 2020 --members 3 --version multi3_v1
   ```

6. **Quantitative validation**:
   - Verify proportionality improvement vs single-member
   - Check compactness scores (larger districts may be more compact)
   - Count mixed-party districts

## Benefits

- **Minority representation**: Minority parties get seats within districts
- **Proportionality**: Closer seat-vote proportionality than single-member
- **Reduced gerrymandering**: Harder to manipulate proportional allocation
- **Larger districts**: May improve compactness (fewer, larger districts)
- **Research contribution**: Quantify multi-member benefits
- **Academic publication**: Comparative analysis of district systems
- **Policy discussion**: Inform debate on representation reform

## Success Criteria

- [ ] Multi-member district counts computed for all states
- [ ] Redistricting completes for 2-member and 3-member scenarios
- [ ] All districts within ±0.5% of target population (2× or 3× single-member)
- [ ] Seat allocation within districts uses proportional method
- [ ] Visualization maps show district boundaries and seat compositions
- [ ] Proportionality metrics computed and compared to single-member
- [ ] Compactness metrics computed for larger districts
- [ ] Documentation explains constitutional issues

## Estimated Complexity

**Effort**: 12-16 hours
- Phase 1 (District Size): 2-3 hours (calculation, edge cases)
- Phase 2 (Redistricting): 3-4 hours (adapted algorithm, testing)
- Phase 3 (Election Results): 2-3 hours (vote aggregation)
- Phase 4 (Seat Allocation): 2-3 hours (proportional methods)
- Phase 5 (Visualization): 2-3 hours (maps with seat composition)
- Phase 6 (Analysis): 2-3 hours (proportionality metrics)
- Phase 7 (Documentation): 1 hour

**Risk**: Medium-High
- **Odd seat counts**: States with seats not divisible by 2 or 3 (e.g., 7 seats)
- **Election data**: Need tract-level or district-level election results
- **Proportional methods**: Multiple approaches (largest remainder, d'Hondt, Sainte-Laguë)
- **Compactness**: Larger districts may be easier or harder to make compact
- **Visualization**: Showing seat composition on maps is complex

**Dependencies**:
- 2020 election results by census tract or aggregated to multi-member districts

## Implementation Notes

### District Count Examples (2020)

| State | Current Seats | Two-Member Districts | Three-Member Districts |
|-------|--------------|---------------------|----------------------|
| California | 52 | 26 districts (52 seats) | 17 districts (51 seats) + 1 single (1 seat) |
| Texas | 38 | 19 districts (38 seats) | 12 districts (36 seats) + 2 singles (2 seats)? |
| Florida | 28 | 14 districts (28 seats) | 9 districts (27 seats) + 1 single (1 seat) |
| New York | 26 | 13 districts (26 seats) | 8 districts (24 seats) + 2 singles (2 seats)? |
| Pennsylvania | 17 | 8 districts (16 seats) + 1 single (1 seat)? | 5 districts (15 seats) + 2 singles (2 seats)? |
| Alabama | 7 | 3 districts (6 seats) + 1 single (1 seat)? | 2 districts (6 seats) + 1 single (1 seat) |
| Vermont | 1 | 1 single-member district (unchanged) | 1 single-member district (unchanged) |

**Handling odd seats**:
- **Option 1**: Mix multi-member and single-member districts
  - E.g., Alabama (7 seats): 3 two-member districts (6 seats) + 1 single-member (1 seat)
- **Option 2**: Use 3-member for states not divisible by 2
  - E.g., Alabama (7 seats): 2 three-member districts (6 seats) + 1 single-member (1 seat)
- **Option 3**: Allow uneven district sizes
  - E.g., Alabama (7 seats): 3 districts of 2, 2, 3 members

**Recommend Option 1** for simplicity.

### Target Population Per District

**2020 Census** (331,449,281 population, 435 districts):
- Single-member target: 761,838 per district
- Two-member target: 1,523,676 per district (2×)
- Three-member target: 2,285,514 per district (3×)

### Proportional Allocation Methods

**Two-member districts**:
- Round vote shares to nearest integer (0, 1, or 2 seats)
- Examples:
  - 60% D, 40% R → 1.2D rounds to 1, 0.8R rounds to 1 → **1D + 1R**
  - 75% D, 25% R → 1.5D rounds to 2, 0.5R rounds to 0 → **2D + 0R**
  - 50% D, 50% R → 1.0D, 1.0R → **1D + 1R**

**Three-member districts**:
- Use largest remainder or d'Hondt method
- **Largest remainder**:
  1. Multiply vote shares by 3: D=1.65, R=1.14, L=0.21
  2. Allocate integer parts: D=1, R=1, L=0 (total=2 seats)
  3. Allocate 1 remaining seat to largest remainder: D (0.65 > R's 0.14 > L's 0.21)
  4. Final: **2D + 1R + 0L**
- **d'Hondt**:
  1. Divide vote shares by 1, 2, 3
  2. Allocate 3 seats to highest quotients
  3. Often produces same result as largest remainder

**Recommend largest remainder method** for transparency.

### Expected Proportionality Improvement

**Single-member system** (current):
- California 2020 results: ~63% D, ~34% R
- Actual seats (2022 congressional map): 42D + 10R = 80.8% D seats
- **Seat-vote gap**: 80.8% - 63% = 17.8% overrepresentation for Democrats

**Two-member system** (proportional):
- Same California: 26 districts of 1.54M population
- Expected seat allocation:
  - If all districts mirror statewide 63-34: Each district gets 1D + 1R
  - Total: 26D + 26R = 50% D seats (underrepresentation!)
  - **Issue**: Rounding at district level doesn't aggregate to state proportionality

**Better approach**: Some districts 2D+0R, some 1D+1R
- To get 63% D seats (33 of 52 seats):
  - 7 districts with 2D+0R = 14D seats
  - 19 districts with 1D+1R = 19D + 19R seats
  - Total: 33D + 19R = 63% D (matches vote share!)

**Key insight**: Multi-member districts improve proportionality but still require careful geographic distribution.

### Compactness Implications

**Hypothesis**: Larger districts may be more compact.

**Reasoning**:
- Fewer districts → fewer boundaries to optimize
- Larger districts → more flexibility in shape
- Example: California with 26 districts instead of 52 may produce rounder districts

**Test**: Compute Polsby-Popper scores for single-member vs multi-member.

### Mixed-Party Representation

**Expected pattern**:
- **Competitive states** (PA, MI, WI, AZ, GA): Many 1D+1R districts
- **Blue states** (CA, NY, MA): Mix of 2D+0R and 1D+1R districts
- **Red states** (AL, WY, OK): Mix of 2R+0D and 1R+1D districts

**Metric**: Percentage of districts with mixed-party delegations.

**Hypothesis**: 30-50% of districts will have mixed delegations (vs 0% in single-member).

### Constitutional Issues

**Historical context**:
- Multi-member districts were common in early U.S. history
- Banned by 1967 law: "Representatives shall be elected...by districts...of contiguous and compact territory containing as nearly as practicable an equal number of inhabitants"
- Law explicitly requires single-member districts

**To implement**:
- Repeal 1967 law (requires act of Congress)
- Or constitutional interpretation that law oversteps Article I authority

**Less problematic than some other enhancements** (still geographic, still state-based), but requires legislative change.

### Governance Implications

**Advantages**:
- **Minority representation**: 40% minority gets seat in 2-member district
- **Proportionality**: Closer to vote share than single-member
- **Reduced gerrymandering**: Harder to pack/crack with proportional allocation
- **Bipartisan representation**: Many districts have both D and R representatives

**Disadvantages**:
- **Constituent confusion**: Which representative to contact? (both represent you)
- **Accountability**: Harder to hold representatives accountable if two serve same area
- **Campaign complexity**: Voters must evaluate 2-3 candidates per party
- **Ballot length**: Longer ballots with more candidates

### International Comparisons

**Multi-member districts used internationally**:
- **Ireland**: 3-5 member districts, single transferable vote (STV)
- **Malta**: 5-member districts, STV
- **Australia (Senate)**: Multi-member states, proportional representation
- **Spain**: Multi-member provinces (50-36 seats for Madrid province)

**U.S. historical use**:
- **19th century**: Many states used multi-member districts
- **At-large elections**: Some states elected all representatives at-large (entire state = 1 district)
- **Banned 1967**: Single-member districts required

### Allocation Method Comparison

| Method | Two-Member (60-40) | Three-Member (55-38-7) | Notes |
|--------|-------------------|----------------------|-------|
| Simple rounding | 1D + 1R | 2D + 1R + 0L | Rounds to nearest integer |
| Largest remainder | 1D + 1R | 2D + 1R + 0L | Allocates remaining seats by fractional part |
| d'Hondt | 1D + 1R | 2D + 1R + 0L | Favors larger parties slightly |
| Sainte-Laguë | 1D + 1R | 2D + 1R + 0L | More proportional for small parties |

**All methods produce similar results** for 2-3 member districts. Use **largest remainder** for simplicity.

## Related Enhancements

- Enhancement 24: Party-Based District Allocation (another proportional approach)
- Enhancement 22: National Redistricting (different approach to fairness)

## Research Value

This model explores:
- **Multi-member benefits**: Does proportionality within districts improve representation?
- **Compactness**: Are larger districts more compact?
- **Minority representation**: Do minorities get better representation?
- **Gerrymandering resistance**: Harder to manipulate with proportional allocation?
- **Feasibility**: Practical reform that could be implemented with legislative change (no constitutional amendment needed)

## Visualization Outputs

**Multi-Member District Map** (California, 2-member):
- 26 districts (vs 52 single-member)
- Color code by seat allocation:
  - Blue: 2D + 0R districts
  - Purple: 1D + 1R mixed districts
  - Red: 0D + 2R districts
- Label districts with seat counts

**Seat Composition Chart**:
```
California (52 seats):
  Single-member:  [42D]      [10R]           (80.8% D)
  Two-member:     [33D]         [19R]        (63.5% D)
  Vote share:     [63% D]       [34% R]      (63.0% D)
```

Shows two-member system matches vote share better than single-member.
