# Enhancement 22: National Redistricting (No State Boundaries)

**Status**: 📋 PLANNED
**Priority**: Research
**Proposed**: January 15, 2026
**Estimated Complexity**: High (10-15 hours)

## Current State

The current redistricting algorithm respects state boundaries. Each state is processed independently to create its allocated number of congressional districts. This follows the constitutional requirement that representatives are apportioned to states.

The algorithm:
- Runs independently for each state (50 separate processes)
- Creates districts that cannot cross state lines
- Produces 435 districts total across all states
- State populations determine district allocation via Huntington-Hill method

## Goal

**Research Experiment**: Implement a "national redistricting" mode that ignores state boundaries entirely and creates 435 equal-population districts across the entire nation using recursive bisection.

This is a **theoretical experiment** to explore:
- What optimal district shapes look like without political boundaries
- How compactness metrics compare to state-based approach
- Whether natural geographic regions emerge from pure algorithm
- Academic baseline for comparing constrained vs unconstrained optimization

**Note**: This violates constitutional requirements (representatives must be apportioned to states) and is purely for research/comparative analysis.

## Implementation Plan

### Phase 1: National Graph Construction
- Load census tract data for all 50 states
- Build single unified adjacency graph for entire nation
- Handle cross-state boundaries as normal adjacencies
- Edge weights: actual geographic boundary lengths between tracts
- Files to create:
  - `scripts/experimental/build_national_graph.py` - Construct unified graph

### Phase 2: National Recursive Bisection
- Adapt recursive bisection algorithm for national scope
- Target: 435 districts of equal population (~769,000 each for 2020)
- No state boundary constraints
- Output: District assignments for all census tracts nationally
- Files to create:
  - `scripts/experimental/national_redistricting.py` - Main algorithm
- Files to modify:
  - `src/apportionment/partition/recursive_bisection.py` - Add optional national mode

### Phase 3: Visualization
- Generate national map showing all 435 districts
- Color districts randomly (like state maps)
- Create comparison map: state-based vs national boundaries
- Highlight where national districts cross state lines
- Files to create:
  - `scripts/experimental/visualize_national_districts.py` - Mapping

### Phase 4: Analysis & Comparison
- Compute compactness metrics (Polsby-Popper, Reock) for national districts
- Compare to state-based approach:
  - Mean/median compactness
  - Perimeter statistics
  - Number of state boundaries crossed
- Generate comparison report
- Files to create:
  - `scripts/experimental/analyze_national_districts.py` - Metrics

### Phase 5: Documentation
- Document as experimental/research feature
- Clarify constitutional limitations
- Explain research value for academic baseline
- Update CLAUDE.md with experimental features section

## Files to Modify/Create

### New Files
1. `scripts/experimental/build_national_graph.py` - Build unified 50-state graph
2. `scripts/experimental/national_redistricting.py` - Run algorithm nationally
3. `scripts/experimental/visualize_national_districts.py` - Map generation
4. `scripts/experimental/analyze_national_districts.py` - Compactness analysis
5. `outputs/experimental/national/` - Output directory for results

### Modified Files
1. `src/apportionment/partition/recursive_bisection.py` - Optional national mode flag
2. `ARCHITECTURE.md` - Document experimental features
3. `CLAUDE.md` - Add experimental enhancements section

## Testing Plan

1. **Data validation**: Verify all 50 states' tract data loads correctly
   ```bash
   python scripts/experimental/build_national_graph.py --year 2020 --validate-only
   ```

2. **Graph connectivity**: Ensure single connected component
   ```bash
   python scripts/experimental/build_national_graph.py --year 2020 --check-connectivity
   ```

3. **Small-scale test**: Run on subset (e.g., New England states)
   ```bash
   python scripts/experimental/national_redistricting.py --states "CT,RI,MA,VT,NH,ME" --year 2020
   ```

4. **Full national run**: All 435 districts (expect 2-4 hours)
   ```bash
   python scripts/experimental/national_redistricting.py --year 2020 --version national_v1
   ```

5. **Quantitative validation**:
   - Verify 435 districts created
   - Check population balance (±0.5% target)
   - Compute compactness statistics
   - Count state boundary crossings

## Benefits

- **Academic baseline**: Pure geometric optimization without political constraints
- **Research insights**: Understand impact of state boundaries on compactness
- **Comparative analysis**: Quantify cost of state boundary constraints
- **Visualization**: Interesting maps showing "natural" regions
- **Paper material**: Novel analysis for academic publication

## Success Criteria

- [ ] Single unified national graph constructed for all 50 states
- [ ] Recursive bisection produces exactly 435 districts
- [ ] All districts within ±0.5% of target population (~769,000 for 2020)
- [ ] Compactness metrics computed for all districts
- [ ] Comparison statistics vs state-based approach generated
- [ ] Visualization maps created showing national districts
- [ ] Documentation clearly labels this as experimental/theoretical
- [ ] Report documents number of state boundaries crossed

## Estimated Complexity

**Effort**: 10-15 hours
- Phase 1 (Graph): 3-4 hours (large data volume, connectivity validation)
- Phase 2 (Algorithm): 3-4 hours (adaptation + testing)
- Phase 3 (Visualization): 2-3 hours (national scale mapping)
- Phase 4 (Analysis): 2-3 hours (comparative metrics)
- Phase 5 (Documentation): 1 hour

**Risk**: Medium
- Large graph size may stress memory/performance
- Cross-state adjacencies need careful handling
- Visualization may be cluttered with 435 districts

**Dependencies**: None

## Implementation Notes

### Performance Considerations
- National graph: ~74,000 census tracts (2020)
- Adjacency edges: ~200,000+ edges
- METIS should handle this scale (tested up to millions of nodes)
- May need 16GB+ RAM for full graph in memory

### Visualization Challenges
- 435 districts on single national map = high density
- Consider insets for Alaska/Hawaii (like current national maps)
- Use random colors (not enough distinct colors for 435)
- Provide zoomed regional views

### Key Differences from State-Based
1. **Single graph** instead of 50 independent graphs
2. **One recursive bisection run** instead of 50 parallel runs
3. **No state boundaries** in adjacency constraints
4. **Direct METIS call** for 435-way partition (depth log₂(435) ≈ 9 rounds)

### Academic Value
- Establishes upper bound for compactness (no political constraints)
- Quantifies "cost" of federalism on district geometry
- Novel contribution to redistricting literature
- Useful for comparative political science research

## Related Enhancements

- Enhancement 7: Edge-Weighted Recursive Bisection (algorithm foundation)
- Enhancement 3: National Maps (visualization patterns)
- Enhancement 12: Edge-Weighted Algorithm Analysis (comparison methodology)

## Constitutional Note

**IMPORTANT**: This is a theoretical experiment only. The U.S. Constitution requires:
- Representatives are apportioned to states (Article I, Section 2)
- Each state must have at least one representative
- Districts cannot cross state lines

This enhancement is for **research purposes** to establish a geometric baseline for comparison to constitutionally-compliant approaches.
