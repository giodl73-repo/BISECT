# County Representation Threshold Analysis

**Paper**: 14+county-representation
**Created**: 2026-02-08
**Status**: Exploratory

## Summary

Threshold ablation study shows that direct county representation fundamentally changes congressional apportionment. Even at high thresholds (3M people), significant seats go to just a handful of counties.

## Key Findings

### Threshold Impact (Top 20 Counties, 2020 Census)

| Threshold | Counties | Direct Seats | % of Congress | Affected States |
|-----------|----------|--------------|---------------|-----------------|
| 0.5M-1.5M | 20       | 81           | **18.6%**     | 8               |
| 2.0M      | 15       | 69           | 15.9%         | 7               |
| 2.5M      | 9        | 51           | 11.7%         | 6               |
| 3.0M      | 6        | 40           | 9.2%          | 4               |

### Individual County Representation

**At 0.5M threshold** (top 5 counties):
- **Los Angeles County, CA**: 13 seats (10.0M people)
- **Cook County, IL**: 7 seats (5.3M people)
- **Harris County, TX**: 6 seats (4.7M people)
- **Maricopa County, AZ**: 6 seats (4.4M people)
- **San Diego County, CA**: 4 seats (3.3M people)

### State-Level Impact (0.5M threshold)

**California** (most affected):
- 7 qualifying counties
- 32 direct county seats
- 19 remaining state pool seats
- Total: 51 seats (loses 1 vs current 52)

**Illinois**:
- 1 qualifying county (Cook)
- 7 direct county seats
- 10 remaining state pool seats
- Total: 17 seats (unchanged)

**Texas**:
- 4 qualifying counties (Harris, Dallas, Tarrant, Bexar)
- Total county seats: ~18-20
- Remaining state pool: ~18-20
- Total: 38 seats (unchanged)

## Implications

### Representation Structure

1. **Urban Concentration**: Large urban counties get direct representation
2. **State Fragmentation**: States split into "large county" + "remaining pool"
3. **Weighted Voting**: Counties would need fractional voting in House

### Policy Trade-offs

**Advantages**:
- Eliminates gerrymandering for large counties
- Recognizes urban counties as coherent political units
- Transparent (no redistricting for qualifying counties)

**Disadvantages**:
- Splits state delegations (coordination issues)
- Requires House procedural changes (weighted voting)
- VRA compliance unclear (counties may dilute minority districts)
- Rural areas lose representation power

### Constitutional Feasibility

**Key Question**: Does weighted voting satisfy "one person, one vote"?

**Precedent**:
- Reynolds v. Sims (1964) requires population equality
- No federal precedent for weighted voting
- Likely requires constitutional amendment or Supreme Court case

## Next Steps for Paper

### Data Requirements

1. **Load ALL 3,143 counties** (not just top 20)
   - Source: Census tract-level data already in project
   - Aggregate to county level
   - Include Alaska boroughs, Louisiana parishes

2. **Test full threshold range**
   - 500K to 5M in 250K increments
   - Identify natural breakpoints

3. **Multi-year analysis**
   - Run for 2000, 2010, 2020
   - Track which counties qualify over time

### Analysis Tasks

1. **Compactness comparison**
   - County geometries vs algorithmic districts
   - Are counties more/less compact?

2. **Partisan impact**
   - Aggregate presidential vote by county
   - Compare seat-vote curves: county vs METIS

3. **VRA compliance**
   - Do large counties preserve majority-minority communities?
   - Compare to VRA-compliant METIS redistricting

4. **Representation equality**
   - Deviation from ideal district size
   - With weighted voting: perfect (0% deviation)
   - With integer rounding: analyze inequality

### Integration with METIS Pipeline

**Hybrid redistricting approach**:

```python
# Step 1: Identify qualifying counties
large_counties = counties[counties['population'] > threshold]

# Step 2: Run Huntington-Hill on [states + counties]
entities = []
for state in states:
    remaining_pop = state_pop - sum(large_county_pops_in_state)
    entities.append({'name': f'{state} (remaining)', 'population': remaining_pop})
for county in large_counties:
    entities.append({'name': county['name'], 'population': county['population']})

allocation = huntington_hill_apportion(entities, total_seats=435)

# Step 3: Run METIS only on "remaining" state tracts
for state in states:
    small_county_tracts = tracts[~tracts['county'].isin(large_counties)]
    target_districts = allocation[f'{state} (remaining)']
    metis_redistrict(small_county_tracts, target_districts)

# Step 4: Large counties get direct representation (no redistricting)
# Representatives cast fractional votes proportional to population
```

## Scripts Created

1. **`scripts/threshold_ablation.py`** - Threshold analysis
   - Usage: `python scripts/threshold_ablation.py --year 2020`
   - Tests 8 thresholds (0.5M to 3M)
   - Currently uses top 20 counties (TODO: load all 3,143)

2. **`src/apportionment/huntington_hill/`** - Apportionment module
   - Generic Huntington-Hill implementation
   - Works with any entity list
   - Validates against Census Bureau results

## Research Questions for Paper

1. **RQ1**: How does threshold affect representation distribution?
   - At what threshold do we see qualitative shifts?
   - Is there a natural threshold (e.g., 1 ideal district worth)?

2. **RQ2**: Compactness trade-off?
   - Are counties more/less compact than algorithmic districts?
   - Does this vary by region (West vs East)?

3. **RQ3**: Partisan fairness?
   - Urban counties are Democratic - does this create bias?
   - How do seat-vote curves compare?

4. **RQ4**: VRA implications?
   - Do large urban counties preserve minority representation?
   - Compare to VRA-compliant algorithmic redistricting

5. **RQ5**: Practical feasibility?
   - What House procedural changes are needed?
   - Historical precedent for weighted voting?

## Figures for Paper

1. **Figure 1**: Threshold sensitivity curve
   - X-axis: Threshold (0.5M to 5M)
   - Y-axis: % of seats to direct county representation
   - Shows diminishing returns at high thresholds

2. **Figure 2**: Geographic map of qualifying counties
   - Choropleth: Counties colored by seat allocation
   - At 2M threshold: ~15 counties, concentrated in urban areas

3. **Figure 3**: State delegation splits
   - Stacked bar chart: Direct county seats vs remaining pool
   - States ordered by % county representation
   - Shows California, Illinois, Texas most affected

4. **Figure 4**: Compactness comparison
   - Box plots: County geometries vs METIS districts
   - Separate by threshold level

## Citations

- **Huntington-Hill Method**: Census Bureau apportionment documentation
- **Reynolds v. Sims** (1964): "One person, one vote" precedent
- **Weighted Voting**: International examples (Switzerland, Germany)

## Status

- [x] Huntington-Hill module implemented
- [x] Threshold ablation script working (top 20 counties)
- [ ] Load all 3,143 counties
- [ ] Multi-year analysis (2000/2010/2020)
- [ ] Compactness comparison
- [ ] Partisan impact analysis
- [ ] VRA compliance check
- [ ] Integration with METIS pipeline
- [ ] Generate all figures
- [ ] Draft paper sections

---

**Next immediate task**: Load full county dataset from census tract data, rerun ablation with all 3,143 counties.
