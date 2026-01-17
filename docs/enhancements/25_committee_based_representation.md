# Enhancement 25: Committee-Based Representation

**Status**: 📋 PLANNED
**Priority**: Low (Research/Experimental)
**Proposed**: January 15, 2026
**Estimated Complexity**: High (12-15 hours)

## Current State

The current congressional system:
- 435 representatives elected from geographic districts
- Each representative serves on multiple committees
- Committee assignments determined after election
- Voters elect representatives, not committee members directly

## Goal

**Research Experiment**: Implement a "committee-based representation" model where:

1. **Committee seat allocation**: Allocate 435 seats across congressional committees using policy area weightings
2. **Statewide committee elections**: Within each committee, all states vote for all committee members
3. **Proportional state representation**: Use apportionment (Huntington-Hill) to allocate committee seats to states based on population
4. **Specialized representatives**: Each representative serves on exactly one committee, elected by statewide vote within that state

**Example**: Foreign Affairs Committee (50 seats allocated)
- California: 12 Foreign Affairs seats (based on population)
- Texas: 7 Foreign Affairs seats
- Vermont: 0-1 Foreign Affairs seats
- Voters in each state vote for their state's Foreign Affairs representatives
- All 50 Foreign Affairs representatives elected via statewide contests

**Research questions**:
- Does committee specialization improve governance?
- How does voter knowledge change when voting for specialized roles?
- What are implications for constituent services?
- How to allocate seats across committees fairly?

## Implementation Plan

### Phase 1: Committee Structure Definition
- Define congressional committee structure (current: ~20 standing committees)
- Allocate 435 seats across committees based on:
  - Current committee sizes
  - Policy area importance
  - Workload distribution
- Output: Committee seat allocation table
- Files to create:
  - `data/experimental/committee_allocation.csv` - Seat counts per committee
  - `scripts/experimental/allocate_committee_seats.py` - Allocation logic

### Phase 2: Per-Committee Apportionment
- For each committee, allocate seats to states using Huntington-Hill
- Input: Committee size (e.g., 50 seats), state populations
- Output: Per-state seat count for each committee
- Handle edge cases:
  - Small committees (e.g., 10 seats) → many states get 0 seats
  - Large committees (e.g., 100 seats) → all states get seats
- Files to create:
  - `scripts/experimental/committee_apportionment.py` - Per-committee H-H

### Phase 3: Statewide Election Simulation
- Simulate statewide elections for committee members
- Use 2020 election data to estimate party vote shares per state
- Assign committee seats to parties based on statewide proportional vote
- Output: Per-state party breakdown for each committee
- Files to create:
  - `scripts/experimental/simulate_committee_elections.py` - Election results

### Phase 4: Visualization
- Generate visualizations:
  - National committee seat allocation (pie chart or bar chart)
  - Per-state committee representation matrix (states × committees)
  - Geographic maps showing committee representation density
- Files to create:
  - `scripts/experimental/visualize_committee_allocation.py` - Charts
  - `scripts/experimental/visualize_state_committees.py` - State maps

### Phase 5: Analysis & Comparison
- Compute metrics:
  - Committee size distribution
  - State representation balance (does every state have seats?)
  - Party proportionality within committees
  - Geographic coverage (which regions well-represented?)
- Compare to current system:
  - Representation equity
  - Specialization vs generalization trade-offs
- Files to create:
  - `scripts/experimental/analyze_committee_system.py` - Metrics

### Phase 6: Documentation
- Document committee-based representation methodology
- Explain statewide election mechanics
- Clarify constitutional challenges
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `data/experimental/committee_allocation.csv` - Seat counts per committee
2. `scripts/experimental/allocate_committee_seats.py` - Committee seat allocation
3. `scripts/experimental/committee_apportionment.py` - Per-committee Huntington-Hill
4. `scripts/experimental/simulate_committee_elections.py` - Statewide elections
5. `scripts/experimental/visualize_committee_allocation.py` - National charts
6. `scripts/experimental/visualize_state_committees.py` - State representation maps
7. `scripts/experimental/analyze_committee_system.py` - Comparative metrics

### Modified Files
1. `scripts/apportionment/huntington_hill.py` - Support committee entities
2. `docs/ARCHITECTURE.md` - Document experimental committee system

## Testing Plan

1. **Committee allocation validation**: Verify 435 total seats
   ```bash
   python scripts/experimental/allocate_committee_seats.py --validate
   ```

2. **Per-committee apportionment**: Test on single committee (e.g., Foreign Affairs)
   ```bash
   python scripts/experimental/committee_apportionment.py --committee "Foreign Affairs" --seats 50
   ```

3. **All committees apportionment**: Verify all states represented somewhere
   ```bash
   python scripts/experimental/committee_apportionment.py --all-committees --validate
   ```

4. **Election simulation**: Test statewide contests for one committee
   ```bash
   python scripts/experimental/simulate_committee_elections.py --committee "Foreign Affairs" --year 2020
   ```

5. **Visualization test**: Generate committee allocation charts
   ```bash
   python scripts/experimental/visualize_committee_allocation.py --year 2020 --version test
   ```

6. **Quantitative validation**:
   - Verify representation equity (all states have some seats)
   - Check committee size balance
   - Validate party proportionality within committees

## Benefits

- **Expertise focus**: Representatives specialize in single policy area
- **Voter knowledge**: Voters choose experts for specific domains
- **Statewide accountability**: Representatives accountable to entire state, not district
- **Research contribution**: Novel representation model
- **Academic publication**: Unique approach to legislative structure
- **Policy discussion**: Inform debate on congressional reform

## Success Criteria

- [ ] Committee seat allocation sums to 435 seats
- [ ] All committees assigned seat counts based on policy importance
- [ ] Per-committee apportionment completes for all committees
- [ ] All states represented in at least one committee
- [ ] Statewide election results simulated for all committees
- [ ] Visualization charts generated (national and state-level)
- [ ] Analysis report compares to current geographic system
- [ ] Documentation explains constitutional limitations

## Estimated Complexity

**Effort**: 12-15 hours
- Phase 1 (Committee Structure): 2-3 hours (research, allocation logic)
- Phase 2 (Per-Committee Apportionment): 3-4 hours (iterative H-H)
- Phase 3 (Election Simulation): 2-3 hours (vote share mapping)
- Phase 4 (Visualization): 2-3 hours (charts, maps)
- Phase 5 (Analysis): 2-3 hours (comparative metrics)
- Phase 6 (Documentation): 1 hour

**Risk**: Medium-High
- **Committee allocation subjectivity**: How to weight policy areas fairly?
- **Small committee problem**: States may get 0 seats in most committees
- **Data availability**: Need committee importance rankings
- **Visualization complexity**: States × committees matrix is large (50 × 20)

**Dependencies**:
- Current congressional committee structure
- Policy area importance rankings (may need expert input)

## Implementation Notes

### Committee Seat Allocation (Example)

Based on current House standing committees (~20 committees):

| Committee | Current Size | Proposed Allocation | Rationale |
|-----------|-------------|---------------------|-----------|
| Appropriations | 60 | 70 | Budget control, high importance |
| Ways and Means | 42 | 50 | Tax policy, high importance |
| Foreign Affairs | 50 | 50 | International relations |
| Armed Services | 62 | 50 | Defense policy |
| Energy and Commerce | 55 | 40 | Broad policy scope |
| Judiciary | 40 | 35 | Legal oversight |
| Agriculture | 50 | 25 | Limited geographic scope |
| Transportation | 60 | 30 | Infrastructure |
| Education and Labor | 50 | 30 | Domestic policy |
| Others (11 committees) | ~250 | 55 | Smaller policy areas |
| **Total** | ~679 | **435** | Scaled to match House size |

**Allocation method**:
1. Rank committees by workload/importance
2. Assign initial allocation based on current size
3. Scale to 435 total using proportional reduction
4. Round to integers ensuring 435 total

### Per-Committee Apportionment Examples

**Foreign Affairs (50 seats)**:
- California: 12 seats (39M / 331M × 50 ≈ 5.9, rounds up)
- Texas: 9 seats (29M / 331M × 50 ≈ 4.4)
- Florida: 6 seats (21M / 331M × 50 ≈ 3.2)
- New York: 6 seats (19M / 331M × 50 ≈ 2.9)
- ...
- Wyoming: 0 seats (0.6M / 331M × 50 ≈ 0.09, rounds down)

**Small states problem**: Wyoming, Vermont, Alaska get 0 seats in most committees.

**Large committee (Appropriations, 70 seats)**:
- Wyoming: 0-1 seats (may qualify with larger committee)

**Solution**: Guarantee each state at least 1 seat across all committees combined.

### Statewide Election Mechanics

**California Foreign Affairs (12 seats)**:
- All California voters vote for 12 Foreign Affairs representatives
- Proportional allocation based on vote shares:
  - If 60% Democratic, 35% Republican: 7D + 4R + 1O
- Top 7 Democratic candidates elected
- Top 4 Republican candidates elected
- Top 1 Other candidate elected

**Ballot design**:
- Voters select party preference
- Vote allocated to party's candidate list
- Candidates ranked within party

### Representation Equity

**Problem**: Small states may have no representation on many committees.

**Analysis**:
- Wyoming population: 0.58M (0.175% of US)
- 0.175% of 435 seats = 0.76 seats total
- Distributed across 20 committees: 0.038 seats per committee

**Solutions**:
1. **Guarantee minimum**: Each state gets at least 1 seat total (distributed across committees)
2. **Committee consolidation**: Fewer, larger committees (e.g., 10 committees of 43-44 seats each)
3. **Tiered system**: Small states share representatives across committees

### Constitutional Issues

- Violates geographic representation (Article I, Section 2: "Representatives shall be apportioned among the several States")
- Eliminates district-level representation
- Changes fundamental structure of House (committees currently internal organization)
- Would require constitutional amendment
- Purely theoretical/academic model

### Governance Implications

**Advantages**:
- Expertise: Representatives specialize in single policy area
- Accountability: Statewide elections hold representatives accountable to broader constituency
- Eliminates gerrymandering: No districts to manipulate

**Disadvantages**:
- No constituent services: No local representative for individual constituent issues
- Small state exclusion: Wyoming voters may have no voice on most issues
- Coordination: No generalist representatives to coordinate across issues
- Ballot complexity: Voters must understand 20 different committee elections

### International Comparisons

No direct international parallel exists. Closest analogs:
- **European Parliament**: Proportional representation by party, members assigned to committees
- **Swiss Federal Council**: Executive positions allocated to parties proportionally

This model is unprecedented in combining:
- Statewide elections
- Committee specialization
- Proportional representation within committees

## Related Enhancements

- Enhancement 24: Party-Based District Allocation (similar proportional approach)
- Enhancement 22: National Redistricting (eliminates boundaries differently)

## Research Value

This model explores:
- **Specialization vs generalization**: Trade-offs in representative roles
- **Voter competence**: Can voters evaluate specialized candidates effectively?
- **Constituent services**: What happens without local representatives?
- **Small state representation**: Equity issues in specialized systems
- **Legislative efficiency**: Do specialized representatives legislate better?

## Visualization Outputs

**National Committee Allocation** (bar chart):
```
Appropriations        ████████████████████ 70 seats
Ways and Means        ████████████████ 50 seats
Foreign Affairs       ████████████████ 50 seats
Armed Services        ████████████████ 50 seats
Energy and Commerce   ████████████ 40 seats
...
```

**State Representation Matrix** (heatmap):
```
              Approps  W&M  Foreign  Armed  Energy  ...
California       15     12     12      11      9
Texas            11      9      9       8      7
Florida           8      6      6       6      5
...
Wyoming           0      0      0       0      0
```

**Geographic Representation Map**:
- Color states by total committee seats
- Darker = more representation
- Highlight states with zero seats on specific committees
