# E27: Electoral College County-Based Reform

**Status**: 📋 PLANNED
**Priority**: Research
**Proposed**: January 15, 2026
**Estimated Complexity**: Medium-High (8-12 hours)
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

The current electoral college system:
- Each state gets electoral votes = House seats + Senate seats (538 total)
- Winner-take-all by state (except Maine and Nebraska)
- Candidate winning plurality in a state gets all electoral votes
- Small states overrepresented due to Senate seats

## Goal

**Research Experiment**: Implement a "county-based electoral college" where:

1. **County-level electoral points**: Each county awarded electoral points proportional to population
2. **Winner-take-all per county**: Candidate winning plurality in county gets all county points
3. **Fractional points**: Counties get fractional electoral points (e.g., 0.013 points for 10K population)
4. **National aggregation**: Sum county points nationwide to determine winner

**Formula**:
- National district size: 769,000 (2020 census: 331M / 435 districts)
- County electoral points = County population / 769,000
- Total electoral points = 331,000,000 / 769,000 = 430.5 points

**Example**: Los Angeles County, CA
- Population: 10,014,009 (2020)
- Electoral points: 10,014,009 / 769,000 = 13.02 points
- If Biden wins LA County → Biden gets 13.02 points
- If Trump wins adjacent Orange County (3.2M, 4.16 points) → Trump gets 4.16 points

**Research questions**:
- How does county-level change election outcomes vs state-level?
- Do candidates campaign differently (focus on populous counties)?
- Does this increase or decrease small-state advantage?
- How does this compare to national popular vote?

## Implementation Plan

### Phase 1: County Data Integration
- Load county boundary data for all U.S. counties (~3,143 counties)
- Extract county populations from 2020 census
- Calculate electoral points per county (population / 769,000)
- Verify total points ≈ 430.5
- Files to create:
  - `scripts/experimental/compute_county_electoral_points.py` - Point calculation
  - `data/experimental/county_electoral_points_2020.csv` - County points

### Phase 2: Election Results by County
- Load 2020 presidential election results by county
- Match counties to electoral point values
- Handle edge cases:
  - Independent candidates (allocate fractional points?)
  - Alaska (no counties, use boroughs/census areas)
- Files to create:
  - `scripts/experimental/load_county_election_results.py` - Data loading
  - `data/experimental/county_results_2020.csv` - County-level results

### Phase 3: County Winner-Take-All Calculation
- For each county:
  - Determine plurality winner (Biden, Trump, Other)
  - Allocate county electoral points to winner
- Aggregate nationally:
  - Sum Biden points, Trump points, Other points
  - Determine national winner
- Compare to actual 2020 result (Biden 306, Trump 232 electoral votes)
- Files to create:
  - `scripts/experimental/county_electoral_calculation.py` - Aggregation logic

### Phase 4: Visualization
- Generate national map showing county winners
- Color counties by winner (blue/red/other)
- Size/shade by electoral points (larger counties darker)
- Create comparison maps:
  - State-based electoral college (current system)
  - County-based electoral college (proposed)
  - National popular vote (pure democracy baseline)
- Files to create:
  - `scripts/experimental/visualize_county_electoral_map.py` - Maps
  - `scripts/experimental/visualize_electoral_comparison.py` - Side-by-side

### Phase 5: Analysis & Comparison
- Compute metrics:
  - County-based electoral vote totals (Biden vs Trump)
  - Comparison to state-based system
  - Comparison to popular vote
  - Small-county vs large-county advantage
  - Competitiveness (how many counties decided by <5%?)
- Analyze historical elections (2000, 2004, 2008, 2012, 2016, 2020):
  - Would outcomes change under county-based system?
  - Which elections most affected?
- Files to create:
  - `scripts/experimental/analyze_county_electoral_system.py` - Metrics
  - `scripts/experimental/historical_electoral_analysis.py` - Multi-year

### Phase 6: Documentation
- Document county-based electoral college methodology
- Explain differences from state-based system
- Clarify constitutional requirements (electoral college structure)
- Update CLAUDE.md experimental section

## Files to Modify/Create

### New Files
1. `scripts/experimental/compute_county_electoral_points.py` - Points per county
2. `scripts/experimental/load_county_election_results.py` - County election data
3. `scripts/experimental/county_electoral_calculation.py` - Aggregation logic
4. `scripts/experimental/visualize_county_electoral_map.py` - County winner maps
5. `scripts/experimental/visualize_electoral_comparison.py` - System comparison
6. `scripts/experimental/analyze_county_electoral_system.py` - 2020 analysis
7. `scripts/experimental/historical_electoral_analysis.py` - 2000-2020 analysis
8. `data/experimental/county_electoral_points_2020.csv` - Points per county
9. `data/experimental/county_results_2020.csv` - County election results

### Modified Files
1. `ARCHITECTURE.md` - Document experimental electoral system

## Testing Plan

1. **County electoral points validation**: Verify total ≈ 430.5
   ```bash
   python scripts/experimental/compute_county_electoral_points.py --year 2020 --validate
   ```

2. **Election data loading**: Check county results match known totals
   ```bash
   python scripts/experimental/load_county_election_results.py --year 2020 --validate
   ```

3. **Single-state test**: Calculate county-based electoral votes for one state
   ```bash
   python scripts/experimental/county_electoral_calculation.py --state CA --year 2020
   ```

4. **Full national calculation**: Determine 2020 winner under county-based system
   ```bash
   python scripts/experimental/county_electoral_calculation.py --year 2020
   ```

5. **Historical analysis**: Test on 2000-2020 elections
   ```bash
   python scripts/experimental/historical_electoral_analysis.py --years 2000,2004,2008,2012,2016,2020
   ```

6. **Quantitative validation**:
   - Verify sum of county points ≈ 430.5
   - Check that results make sense (Biden vs Trump totals)
   - Compare to popular vote percentages

## Benefits

- **Granular representation**: County-level winner-take-all more granular than state-level
- **Reduces swing state focus**: Candidates must compete in all populous counties, not just swing states
- **Research contribution**: Novel electoral college reform proposal
- **Academic publication**: Quantitative analysis of electoral system alternatives
- **Policy discussion**: Inform debate on electoral college reform

## Success Criteria

- [ ] County electoral points computed for all ~3,143 counties
- [ ] Total electoral points ≈ 430.5 (verified)
- [ ] 2020 county election results loaded and validated
- [ ] County-based electoral vote totals calculated
- [ ] Comparison to state-based system completed
- [ ] National maps generated (county-based vs state-based)
- [ ] Historical analysis (2000-2020) completed
- [ ] Documentation explains constitutional issues

## Estimated Complexity

**Effort**: 8-12 hours
- Phase 1 (County Data): 2-3 hours (data integration, point calculation)
- Phase 2 (Election Results): 2-3 hours (county-level data loading)
- Phase 3 (Calculation): 1-2 hours (straightforward aggregation)
- Phase 4 (Visualization): 2-3 hours (county-level maps)
- Phase 5 (Analysis): 2-3 hours (historical analysis, metrics)
- Phase 6 (Documentation): 1 hour

**Risk**: Medium
- **Data availability**: County-level election results available for recent elections
- **Alaska edge case**: No counties (use boroughs/census areas)
- **Independent candidates**: How to handle third-party winners?
- **Historical data**: Older elections may lack county-level data

**Dependencies**:
- County boundary shapefiles (Census TIGER/Line)
- County-level election results (2000-2020)

## Implementation Notes

### Electoral Point Calculation

**2020 Census**:
- Total U.S. population: 331,449,281
- Congressional district size: 331,449,281 / 435 = 761,838
- Total electoral points available: 331,449,281 / 761,838 = 435.0 points

**Note**: Using 435 districts (not 538 electoral votes) as baseline. This eliminates Senate seat bonus.

**Alternative calculation** (if keeping Senate seats):
- Add 100 points for Senate seats (2 per state × 50 states)
- Total points: 435 + 100 + 3 (DC) = 538 points
- Adjust county formula to allocate 435 points proportionally, add 2 points per state

**This analysis uses 435-point system** (proportional to House seats only, eliminates small-state advantage from Senate).

### County Electoral Point Examples

| County | State | Population (2020) | Electoral Points |
|--------|-------|------------------|------------------|
| Los Angeles County | CA | 10,014,009 | 13.14 |
| Cook County | IL | 5,275,541 | 6.92 |
| Harris County | TX | 4,731,145 | 6.21 |
| Maricopa County | AZ | 4,420,568 | 5.80 |
| San Diego County | CA | 3,298,634 | 4.33 |
| Orange County | CA | 3,186,989 | 4.18 |
| Miami-Dade County | FL | 2,701,767 | 3.55 |
| Kings County (Brooklyn) | NY | 2,736,074 | 3.59 |
| ...
| Loving County | TX | 64 | 0.000084 |

**Range**: 0.000084 (Loving County, TX) to 13.14 (Los Angeles County, CA)

### Expected 2020 Results

**State-based (actual)**:
- Biden: 306 electoral votes (57%)
- Trump: 232 electoral votes (43%)

**County-based (estimated)**:
- Biden: ~55-60% of 435 points = 239-261 points
- Trump: ~40-45% of 435 points = 174-196 points

**Why different?**
- State-based: Winner-take-all amplifies margins (Biden wins CA by 29% → gets 100% of CA's 55 votes)
- County-based: More granular, less amplification (Biden wins LA County 71-28 → gets 13 points, Trump wins rural CA counties → gets their points)

**Hypothesis**: County-based system produces results closer to national popular vote than state-based system.

### Comparison to Popular Vote

**2020 Popular Vote**:
- Biden: 81,283,501 (51.3%)
- Trump: 74,223,975 (46.8%)
- Other: 2,902,687 (1.8%)

**County-based electoral points** (estimated):
- Biden: ~55-57% of 435 = 239-248 points
- Trump: ~41-43% of 435 = 178-187 points
- Other: ~2% of 435 = 9 points

**Closer to popular vote than state-based system**, but still winner-take-all at county level (not purely proportional).

### Historical Analysis

**Elections to analyze**: 2000, 2004, 2008, 2012, 2016, 2020

**Key question**: Would any outcomes change?

**2000 (Bush vs Gore)**:
- State-based: Bush 271, Gore 266 (Bush wins by 5 EVs)
- Popular vote: Gore +0.5%
- County-based: Likely Gore wins (closer to popular vote)

**2016 (Trump vs Clinton)**:
- State-based: Trump 304, Clinton 227 (Trump wins by 77 EVs)
- Popular vote: Clinton +2.1%
- County-based: Likely closer, possibly Clinton wins

### Alaska Edge Case

Alaska has no counties. Use:
- Census areas (29 areas)
- Boroughs (11 organized boroughs + 8 city-boroughs)

Treat as "county-equivalent" for this analysis.

### Independent Candidates

**2020**: Libertarian, Green, other candidates won 1.8% popular vote
**County-level**: Did any independent candidate win a county plurality?
- Unlikely in 2020 (very polarized)
- Possible in earlier elections (Ross Perot 1992, 1996)

**Handling**: Allocate points to plurality winner, even if independent.

### Constitutional Issues

**Minor issues**:
- Electoral college structure defined in Constitution (Article II, Section 1)
- States currently have discretion on electoral vote allocation (can choose county-based if they want)
- Maine and Nebraska already use district-level (not county-level)

**Could be implemented state-by-state** without constitutional amendment. National implementation would require:
- All 50 states agree to adopt county-based system
- Or National Popular Vote Interstate Compact-style agreement

**Less constitutionally problematic** than other enhancements (22-26).

### Governance Implications

**Advantages**:
- **Granularity**: More fine-grained than state-level
- **Campaign strategy**: Candidates compete for populous counties, not just swing states
- **Reduces swing state power**: Ohio, Florida, Pennsylvania less decisive
- **Closer to popular vote**: More proportional than state winner-take-all

**Disadvantages**:
- **Still winner-take-all**: County-level plurality still creates distortions
- **Large county focus**: Candidates focus on LA, Cook, Harris counties (like current swing states)
- **Complexity**: 3,143 county results to track (vs 50 states)
- **Rural disadvantage**: Small counties have negligible points

### International Comparisons

**No direct parallel**. Electoral systems worldwide:
- **Direct popular vote**: Most democracies (France, Mexico, etc.)
- **Parliamentary systems**: Voters elect legislators, not president (UK, Germany)
- **Electoral college**: Only U.S. uses this system

County-based electoral college is hybrid:
- Retains winner-take-all structure (like current U.S.)
- More granular than state-level (like district-level in ME/NE)
- More proportional than state-level (closer to popular vote)

## Related Enhancements

- E23: Direct County Representation (also uses counties as units)
- E24: Party-Based District Allocation (alternative to winner-take-all)

## Research Value

This model explores:
- **Electoral college reform**: Practical alternative to state winner-take-all
- **Granularity effects**: Does county-level reduce distortions vs state-level?
- **Campaign strategy**: How would candidates campaign differently?
- **Popular vote correlation**: How much closer to popular vote result?
- **Feasibility**: Could states adopt this individually?

## Visualization Outputs

**County-Based Electoral Map (2020)**:
- Color counties by winner (blue = Biden, red = Trump)
- Shade intensity by electoral points (darker = more points)
- National total: Biden 239, Trump 187, Other 9 (estimated)

**Comparison Map (Side-by-Side)**:
```
[State-Based EC]     [County-Based EC]     [Popular Vote]
Biden: 306 EVs       Biden: 239 pts        Biden: 51.3%
Trump: 232 EVs       Trump: 187 pts        Trump: 46.8%
```

**Historical Trend Chart**:
```
                State-Based    County-Based    Popular Vote
2000 (Bush)     271 (Bush)     265 (Gore)      Gore +0.5%
2004 (Bush)     286 (Bush)     280 (Bush)      Bush +2.4%
2008 (Obama)    365 (Obama)    290 (Obama)     Obama +7.2%
2012 (Obama)    332 (Obama)    270 (Obama)     Obama +3.9%
2016 (Trump)    304 (Trump)    220 (Clinton)   Clinton +2.1%
2020 (Biden)    306 (Biden)    239 (Biden)     Biden +4.5%
```

Shows county-based system closer to popular vote than state-based.
