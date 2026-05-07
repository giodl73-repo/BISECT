# Paper 14: County Representation - Implementation Status

**Created**: 2026-02-08
**Status**: Foundation Complete, Ready for Full Implementation

---

## Summary

Implemented foundation for Paper 14 (Direct County Representation) focusing on Huntington-Hill apportionment with county-level units. The paper analyzes threshold-based apportionment where large counties become independent units in Huntington-Hill, with VRA comparison for remaining state redistricting.

---

## What's Implemented

### 1. Huntington-Hill Apportionment Module ✅

**Location**: `src/apportionment/huntington_hill/`

**Files**:
- `algorithm.py` - Core Huntington-Hill implementation
- `__init__.py` - Module exports

**Features**:
- Generic apportionment for any entity list (states, counties, mixed)
- Priority value calculation: `P(n) = population / sqrt(n * (n+1))`
- Configurable minimum seats per entity
- Validation against Census Bureau results
- Optional detailed output (allocation sequence, priority values)

**Tested**:
```python
entities = [
    {'name': 'California', 'population': 39_538_223},
    {'name': 'Texas', 'population': 29_145_505},
    {'name': 'Vermont', 'population': 643_077}
]
result = apportion(entities, total_seats=100, min_seats=1)
# CA: 57, TX: 42, VT: 1 ✓
```

### 2. Threshold Ablation Analysis ✅

**Location**: `research/14+county-representation/scripts/threshold_ablation.py`

**Features**:
- Tests 8 thresholds (0.5M to 3M people)
- Identifies qualifying counties at each threshold
- Computes hybrid apportionment (states + large counties)
- Reports seats allocated to counties vs states
- Shows state-level impacts

**Sample Output** (0.5-1.5M threshold):
```
Qualifying counties: 20
Total seats to counties: 81 (18.6%)
State pool seats: 354 (81.4%)

Top qualifying counties:
  Los Angeles County, CA    13 seats
  Cook County, IL            7 seats
  Harris County, TX          6 seats
```

### 3. Hybrid Redistricting Integration ✅

**Location**: `research/14+county-representation/scripts/hybrid_redistricting.py`

**Features**:
- Connects Huntington-Hill apportionment to recursive bisection
- Identifies large counties above threshold
- Runs Huntington-Hill on [remaining_states + large_counties]
- Framework for METIS redistricting on state pools (TODO)
- VRA comparison setup (regular vs 5x @ 40%)

**VRA Parameters** (from Paper 03 standard):
- Edge weighting: **5x multiplier** (α = 5.0)
- Minority threshold: **40%** (τ = 0.40) - tract-level edge weighting
- MM district target: **60%** minority in final districts
- State feasibility: **42%** state-wide minority threshold

**Sample Run** (2M threshold, CA/TX/IL):
```bash
python scripts/hybrid_redistricting.py --year 2020 --threshold 2000000 --states CA TX IL

Results:
  Total: 107 seats (52+38+17)
  Direct county seats: 50 (46.7%)
  State pool seats: 57 (53.3%)

  CA: 24 remaining districts
  TX: 23 remaining districts
  IL: 10 remaining districts
```

---

## What's Next (TODOs)

### Phase 1: Full County Data (High Priority)

**Current limitation**: Only top 20 counties hardcoded

**Task**: Load all 3,143 U.S. counties from census tract data

**Implementation**:
```python
# Aggregate tract-level data to county level
tracts = gpd.read_parquet(f'outputs/data/{year}/units/tracts/{state}_tracts_{year}.parquet')
county_pops = tracts.groupby('county_fips').agg({
    'population': 'sum',
    'geometry': lambda x: x.union_all()  # Dissolve to county boundaries
})
```

**Files to create**:
- `research/14+county-representation/scripts/prepare_county_data.py`

**Deliverable**:
- `outputs/data/{year}/counties/all_counties_{year}.parquet` (3,143 rows)

---

### Phase 2: METIS Integration (Medium Priority)

**Task**: Implement actual recursive bisection for "remaining state" entities

**Current**: Framework exists, TODOs marked

**Implementation**:
```python
# In hybrid_redistricting.py
from apportionment.partition.recursive_bisection import RecursiveBisection
from apportionment.data.loader import load_redistricting_data

# Load tracts excluding large counties
tracts, adjacency = load_redistricting_data(year, state)
small_county_tracts = tracts[~tracts['county_fips'].isin(large_county_fips)]

# Regular recursive bisection
rb_regular = RecursiveBisection(small_county_tracts, adjacency, target_districts)
assignments_regular = rb_regular.partition()

# VRA recursive bisection (5x @ 40%)
edge_weights = compute_edge_weights_vra(small_county_tracts, adjacency)
rb_vra = RecursiveBisection(small_county_tracts, adjacency, target_districts, edge_weights=edge_weights)
assignments_vra = rb_vra.partition()
```

**Deliverable**:
- Complete `hybrid_redistricting.py` implementation
- Output district assignments for both regular and VRA versions

---

### Phase 3: Analysis & Comparison (Medium Priority)

**Task**: Compare county-based vs algorithmic redistricting

**Metrics to compute**:

1. **Compactness**:
   - County geometries (Polsby-Popper/Reock)
   - Compare to Paper 01 algorithmic districts
   - Do counties perform better/worse?

2. **Partisan Outcomes**:
   - Aggregate presidential votes by county
   - Compute D/R seat allocations
   - Seat-vote curves: county vs METIS
   - Does county system favor urban (D) or rural (R)?

3. **VRA Compliance**:
   - For states needing MM districts: regular vs VRA
   - Does VRA-constrained redistricting help/hurt?
   - Compare minority representation across approaches

4. **Representation Equality**:
   - With weighted voting: 0% deviation (perfect)
   - With integer rounding: analyze inequality
   - Compare to current system

**Files to create**:
- `research/14+county-representation/scripts/analyze_compactness.py`
- `research/14+county-representation/scripts/analyze_partisan_outcomes.py`
- `research/14+county-representation/scripts/analyze_vra_compliance.py`

---

### Phase 4: Figures for Paper (Low Priority)

**Figures to generate**:

1. **Figure 1**: Threshold sensitivity curve
   - X: Threshold (0.5M to 5M)
   - Y: % seats to direct county representation
   - Shows optimal threshold range

2. **Figure 2**: Geographic distribution map
   - Choropleth: Counties colored by seat allocation
   - Different thresholds: 1M, 2M, 3M
   - Highlight urban concentration

3. **Figure 3**: State delegation splits
   - Stacked bar: County seats vs remaining pool
   - States ordered by % county representation
   - Shows which states most affected

4. **Figure 4**: Compactness comparison
   - Box plots: Counties vs METIS districts
   - By threshold level
   - Statistical significance tests

5. **Figure 5**: VRA comparison
   - Minority representation: Regular vs VRA
   - For high-minority states (≥42%)
   - Show trade-offs

**Files to create**:
- `research/14+county-representation/scripts/generate_figure_{1-5}.py`

---

### Phase 5: Paper Writing (High Priority)

**Current status**: Structure in `plan.md`, LaTeX scaffold in `main.tex`

**Sections to write** (based on plan.md):

1. **Introduction** (600 words)
   - Problem: Redistricting enables gerrymandering
   - Alternative: Use existing county boundaries
   - Contribution: Feasibility analysis with H-H apportionment

2. **Historical Context** (700 words)
   - Reynolds v. Sims (1964) - struck down county-based state legislatures
   - Weighted voting as constitutional solution
   - International examples (Switzerland, Germany)

3. **Methodology** (800 words)
   - County data (3,143 counties)
   - Huntington-Hill apportionment formula
   - Threshold ablation approach
   - VRA comparison (5x @ 40%)

4. **Results** (1,200 words)
   - Threshold analysis (TABLE 1: ablation results)
   - Geographic distribution (FIGURE 2: map)
   - Compactness comparison (FIGURE 4: box plots)
   - VRA compliance (FIGURE 5: comparison)

5. **Discussion** (1,000 words)
   - Constitutional feasibility (weighted voting)
   - Practical implementation (House procedures)
   - Advantages vs algorithmic redistricting
   - Disadvantages (VRA, novelty)

6. **Conclusion** (400 words)
   - Summary of findings
   - Policy recommendations
   - Future work

**Target venue**: State Politics & Policy Quarterly (7,000-9,000 words)

---

## Key Design Decisions

### 1. Focus on Huntington-Hill (Not Recursive Bisection)

**Decision**: Paper emphasizes the *apportionment* innovation (county-level H-H), not the *redistricting* implementation (METIS).

**Rationale**:
- Main contribution: "What if counties are H-H units?"
- Recursive bisection is just the tool to implement remaining state allocations
- Can compare regular vs VRA as secondary analysis

### 2. Threshold as Policy Choice (Not Mathematical)

**Decision**: No single "optimal" threshold - present trade-offs at different levels

**Rationale**:
- Same constitutional compromise as small states (representation vs equality)
- Wyoming gets 1 seat despite deserving 0.75 - large counties face same trade-off
- Policy decision: More counties = more direct representation, more House procedural complexity

### 3. VRA Standard: 5x @ 40%

**Decision**: Use established parameters from Paper 03

**Rationale**:
- 5x edge weighting (α = 5.0) - proven effective for VRA compliance
- 40% tract threshold (τ = 0.40) - tracts >40% minority get edges weighted
- 60% MM target - standard for majority-minority districts
- 42% state threshold - states ≥42% state-wide minority can achieve VRA compliance

### 4. Weighted Voting Assumption

**Decision**: Assume counties can cast fractional votes proportional to population

**Rationale**:
- Achieves perfect representation equality (0% deviation)
- Constitutional question open (no federal precedent)
- International precedent exists (Switzerland, Germany)
- Discuss feasibility in paper, don't assume constitutional approval

---

## Testing Status

### Unit Tests
- [ ] Huntington-Hill algorithm (verify against Census Bureau 2000/2010/2020)
- [ ] Priority value calculation
- [ ] Edge cases (1 entity, equal populations, min seats)

### Integration Tests
- [x] Threshold ablation with top 20 counties ✓
- [x] Hybrid redistricting framework (CA/TX/IL) ✓
- [ ] Full county dataset (3,143 counties)
- [ ] METIS integration (regular + VRA)

### Validation
- [ ] Algorithm matches Census Bureau apportionment exactly
- [ ] Threshold sensitivity analysis complete
- [ ] Compactness metrics computed
- [ ] Partisan outcomes analyzed
- [ ] VRA compliance verified

---

## File Manifest

### Source Code
```
src/apportionment/huntington_hill/
├── __init__.py                 # Module exports
└── algorithm.py                # Core H-H implementation (200 lines)

research/14+county-representation/scripts/
├── threshold_ablation.py       # Threshold analysis (340 lines)
├── hybrid_redistricting.py     # Integration framework (310 lines)
└── [TODO] prepare_county_data.py
└── [TODO] analyze_*.py         # Analysis scripts
└── [TODO] generate_figure_*.py # Figure generation
```

### Documentation
```
research/14+county-representation/
├── plan.md                     # Original paper plan ✓
├── THRESHOLD_ANALYSIS.md       # Threshold findings ✓
├── IMPLEMENTATION_STATUS.md    # This file ✓
├── main.tex                    # LaTeX scaffold ✓
├── sections/*.tex              # Paper sections (stubs)
└── _panel.yaml                 # Panel metadata ✓
```

### Data (TODO)
```
outputs/data/2020/counties/
└── all_counties_2020.parquet   # 3,143 counties with pops/geometries
```

---

## Timeline Estimate

Based on implementation plan:

| Phase | Task | Effort | Priority |
|-------|------|--------|----------|
| 1 | Full county data | 1 day | High |
| 2 | METIS integration | 2 days | Medium |
| 3 | Analysis scripts | 3 days | Medium |
| 4 | Figure generation | 2 days | Low |
| 5 | Paper writing | 1 week | High |

**Total**: ~2-3 weeks for complete implementation + paper draft

---

## Next Immediate Steps

1. **Load full county dataset** (3,143 counties from tract data)
   - Aggregate population by county FIPS
   - Dissolve tract geometries to county boundaries
   - Save to `outputs/data/2020/counties/all_counties_2020.parquet`

2. **Run threshold ablation on full dataset**
   - Test thresholds: 500K, 761K, 1M, 1.5M, 2M, 2.5M, 3M, 5M
   - Generate results table for paper

3. **Implement METIS integration**
   - Complete `hybrid_redistricting.py` TODOs
   - Run on 1-2 test states first (VT, CA)
   - Verify regular vs VRA comparison works

4. **Start paper writing**
   - Draft Introduction and Methodology first
   - Use placeholder results from top-20 analysis
   - Refine as full data becomes available

---

**Status**: Foundation complete, ready for full implementation and analysis.
