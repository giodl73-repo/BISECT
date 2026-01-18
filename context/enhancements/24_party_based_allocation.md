# Enhancement 24: Party-Based District Allocation

**Status**: 📋 PLANNED
**Priority**: Low (Research/Experimental)
**Proposed**: January 15, 2026
**Estimated Complexity**: Very High (20-25 hours)

## Current State

The current redistricting system:
- Allocates congressional seats to states using Huntington-Hill apportionment
- Creates single unified district map per state
- Each geographic location belongs to exactly one district
- Voter representation determined by geographic district only

## Goal

**Research Experiment**: Implement a "party-based district allocation" model where:

1. **National party vote totals**: Aggregate statewide party votes across all states
2. **Party-based apportionment**: Allocate 435 seats to parties using Huntington-Hill based on national vote share
3. **Per-party redistricting**: For each party in each state:
   - Calculate party's seat allocation within that state
   - Run recursive bisection to create party-specific district map
   - Result: Multiple overlapping district maps (one per party per state)
4. **Dual representation**: Each voter has representation from their party's district map
   - Democratic voters represented by Democratic district map
   - Republican voters represented by Republican district map
   - Third-party voters represented by their party's map

**Example**: California with 53 seats (2020):
- Statewide 2020 results: ~63% D, ~34% R, ~3% other
- Party allocation: 45 Democratic seats, 7 Republican seats, 1 other
- Output: Three overlapping maps covering entire California
  - Democratic map: 45 districts (covers whole state)
  - Republican map: 7 districts (covers whole state)
  - Other map: 1 district (entire state)

**Research questions**:
- Does this achieve proportional representation?
- How does compactness compare to geographic districts?
- What are implications for governance?
- How does dual representation work in practice?

## Implementation Plan

### Phase 1: Party Vote Data Integration
- Load 2020 presidential election results by state
- Aggregate statewide party vote totals (D, R, Libertarian, Green, Other)
- Calculate national party vote shares
- Validate totals sum to 435 seats
- Files to create:
  - `scripts/experimental/aggregate_party_votes.py` - Statewide totals
  - `data/experimental/party_votes_2020.csv` - National party votes

### Phase 2: Party-Based Apportionment
- Implement Huntington-Hill for party-based allocation
- Allocate 435 seats to parties based on national vote share
- Output: Seat count per party (e.g., 234D, 186R, 10L, 3G, 2O)
- Files to create:
  - `scripts/experimental/party_apportionment.py` - National allocation
- Files to modify:
  - `scripts/apportionment/huntington_hill.py` - Support party entities

### Phase 3: State-Level Party Allocation
- For each state, determine party seat breakdown
- Use state-level party vote shares to allocate state's seats among parties
- Handle rounding (ensure state totals match current allocation)
- Output: Per-state party seat table
- Files to create:
  - `scripts/experimental/state_party_allocation.py` - State-level breakdown

### Phase 4: Per-Party Redistricting
- For each (state, party) pair:
  - Load census tract data for state
  - Run recursive bisection for party's allocated seats
  - Output: District assignments for this party's map
- Handle edge cases:
  - Parties with 0 seats in a state (no map)
  - Parties with 1 seat in a state (entire state = district)
- Files to create:
  - `scripts/experimental/party_redistricting.py` - Per-party algorithm
- Files to modify:
  - `src/apportionment/partition/recursive_bisection.py` - Support party mode flag

### Phase 5: Overlapping Map Visualization
- Generate per-party district maps for each state
- Show multiple maps side-by-side (D map | R map | Other map)
- Color districts within each map
- Create national aggregation showing party seat distributions
- Files to create:
  - `scripts/experimental/visualize_party_maps.py` - State maps
  - `scripts/experimental/visualize_national_party_allocation.py` - National view

### Phase 6: Analysis & Comparison
- Compute metrics:
  - Proportionality (seats vs votes)
  - Compactness per party's districts
  - Overlap patterns (do party maps look similar?)
- Compare to standard geographic approach:
  - Seat-vote proportionality
  - Wasted vote calculations
  - Compactness differences
- Files to create:
  - `scripts/experimental/analyze_party_system.py` - Metrics

### Phase 7: Documentation
- Document party-based apportionment methodology
- Explain dual representation concept
- Clarify constitutional challenges
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `scripts/experimental/aggregate_party_votes.py` - National party vote totals
2. `scripts/experimental/party_apportionment.py` - National Huntington-Hill by party
3. `scripts/experimental/state_party_allocation.py` - Per-state party seat breakdown
4. `scripts/experimental/party_redistricting.py` - Per-party recursive bisection
5. `scripts/experimental/visualize_party_maps.py` - Overlapping state maps
6. `scripts/experimental/visualize_national_party_allocation.py` - National aggregation
7. `scripts/experimental/analyze_party_system.py` - Proportionality metrics
8. `data/experimental/party_votes_2020.csv` - Party vote totals

### Modified Files
1. `scripts/apportionment/huntington_hill.py` - Support party entities
2. `src/apportionment/partition/recursive_bisection.py` - Party mode flag
3. `docs/ARCHITECTURE.md` - Document experimental party system

## Testing Plan

1. **Party vote aggregation**: Validate statewide totals match known results
   ```bash
   python scripts/experimental/aggregate_party_votes.py --year 2020 --validate
   ```

2. **National apportionment**: Verify 435 total seats, proportional allocation
   ```bash
   python scripts/experimental/party_apportionment.py --year 2020 --validate
   ```

3. **Single-state test**: Test on state with clear party split (e.g., Pennsylvania)
   ```bash
   python scripts/experimental/party_redistricting.py --state PA --year 2020 --version test
   ```

4. **Visualization test**: Verify overlapping maps render correctly
   ```bash
   python scripts/experimental/visualize_party_maps.py --state PA --year 2020 --version test
   ```

5. **Full national run**: All states, all parties
   ```bash
   python scripts/experimental/party_redistricting.py --year 2020 --version party_v1
   ```

6. **Quantitative validation**:
   - Verify seat-vote proportionality improved vs geographic districts
   - Check compactness metrics per party
   - Compare to international proportional representation systems

## Benefits

- **Proportional representation**: Seats match vote shares (eliminates gerrymandering)
- **Research contribution**: Novel approach to proportional representation
- **Compactness analysis**: Do party-specific districts differ geometrically?
- **Academic publication**: Unique hybrid of geographic and proportional systems
- **Policy discussion**: Inform debate on representation reform

## Success Criteria

- [ ] National party vote totals computed and validated
- [ ] National Huntington-Hill allocates 435 seats proportionally
- [ ] Per-state party seat breakdowns sum to state totals
- [ ] Redistricting completes for all (state, party) pairs
- [ ] All districts within ±0.5% of target population per party
- [ ] Overlapping maps visualized correctly
- [ ] Proportionality metrics computed (seats vs votes)
- [ ] Comparison report generated vs geographic approach
- [ ] Documentation explains constitutional limitations

## Estimated Complexity

**Effort**: 20-25 hours
- Phase 1 (Vote Data): 2-3 hours (data aggregation, validation)
- Phase 2 (National Apportionment): 2-3 hours (party-based H-H)
- Phase 3 (State Allocation): 3-4 hours (rounding, validation)
- Phase 4 (Per-Party Redistricting): 6-8 hours (complex iteration, edge cases)
- Phase 5 (Visualization): 4-5 hours (overlapping maps, layout)
- Phase 6 (Analysis): 3-4 hours (proportionality metrics)
- Phase 7 (Documentation): 1 hour

**Risk**: Very High
- **Data complexity**: Need reliable party vote data by state
- **Computational intensity**: N_states × N_parties redistricting runs (~100-150 total)
- **Edge cases**: States with single party, parties with 0-1 seats
- **Visualization challenge**: Overlapping maps difficult to display clearly
- **Conceptual complexity**: Dual representation is novel, hard to explain

**Dependencies**:
- Reliable 2020 presidential election results by state
- May need additional party data (not just D/R)

## Implementation Notes

### Expected Party Allocation (2020 data)

Based on 2020 presidential popular vote (~51% Biden, ~47% Trump, ~2% other):
- Democratic Party: ~221 seats (51% of 435)
- Republican Party: ~204 seats (47% of 435)
- Libertarian: ~6 seats (1.2% of 435)
- Green: ~2 seats (0.3% of 435)
- Other: ~2 seats (remaining)

**Contrast with actual 2020 results**: House was 222D-213R (due to geographic districting, not proportional representation).

### State-Level Examples

**California** (53 seats, ~63% D, ~34% R):
- Democratic map: 33-35 districts covering entire state
- Republican map: 18-20 districts covering entire state
- Each voter represented by one district from their party's map

**Texas** (38 seats, ~46% D, ~52% R):
- Democratic map: 17-18 districts covering entire state
- Republican map: 20-21 districts covering entire state

**Wyoming** (1 seat, ~70% R, ~27% D):
- Republican map: 1 district (entire state)
- Democratic map: 0 districts (no representation)
- Problem: Democratic voters in Wyoming have no district

### Dual Representation Mechanics

**How it works**:
1. Voter registers party affiliation
2. Voter assigned to party's district map for their state
3. Vote counts toward representative from party's district
4. Each party elects its proportional share of representatives

**Governance implications**:
- Representatives elected by party supporters only
- No "swing" districts (everyone in safe seat for their party)
- Eliminates geographic representation
- May increase partisan polarization

### Computational Considerations

- **Redistricting runs**: ~50 states × ~2-3 major parties = 100-150 runs
- **Performance**: If 1 state = 2 minutes, 100 runs = 3-4 hours sequential
- **Parallelization**: Can run (state, party) pairs in parallel
- **Memory**: Need to keep multiple district maps in memory for visualization

### Constitutional Issues

- Violates geographic representation principle (representatives from states, not parties)
- Requires party registration (not universally required in U.S.)
- Eliminates state-level apportionment (Article I, Section 2)
- Would require constitutional amendment
- Purely theoretical/academic model

### International Comparisons

Similar systems exist internationally:
- **Germany**: Mixed-member proportional (geographic + party list)
- **New Zealand**: MMP system (similar to Germany)
- **Israel**: Pure proportional representation (party lists only)

This model is closest to Israel's system but retains geographic districting within each party.

## Related Enhancements

- Enhancement 22: National Redistricting (algorithmic baseline)
- Enhancement 23: Direct County Representation (alternative apportionment)
- Enhancement 28: Multi-Member Districts (another proportional approach)

## Research Value

This model explores:
- **Proportional representation**: Pure seat-vote proportionality
- **Geographic vs partisan**: Eliminates geographic gerrymandering entirely
- **Dual representation**: Each voter has representative from their party
- **Compactness comparison**: Do party maps differ from geographic maps?
- **Governance implications**: What happens when representatives elected only by supporters?

## Visualization Challenges

**Overlapping maps are hard to display**:

**Option 1**: Side-by-side maps
```
[D Map: 45 districts] [R Map: 7 districts] [O Map: 1 district]
```

**Option 2**: Layered transparency (not recommended, cluttered)

**Option 3**: Sequential frames (animation)

**Option 4**: Interactive web dashboard with party selection

Recommend Option 1 (side-by-side) for static outputs, Option 4 for web dashboard.
