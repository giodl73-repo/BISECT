# Representation Equality Findings - County-Based Apportionment

**Paper 14**: Direct County Representation
**Analysis Date**: 2026-02-08
**Data**: 3,142 U.S. Counties, 2020 Census

---

## Executive Summary

County-based congressional apportionment faces **the same representation inequality trade-off as the current state-based system**. With minimum 1 seat per entity (constitutional requirement), small entities are overrepresented while maintaining reasonable average representation.

**Key finding**: A **minimum remaining state population constraint** (≥761K) elegantly solves the extreme outlier problem with minimal impact on qualifying counties.

---

## Representation Equality Metrics

### Unconstrained System (Any County Can Fork)

| Threshold | Counties | Mean Pop/Seat | CV | Min Pop/Seat | Max Pop/Seat | Range |
|-----------|----------|---------------|-----|--------------|--------------|-------|
| 0.5M | 141 | 747,334 | 0.192 | **419,229** | 1,092,039 | 2.6x |
| 0.75M | 89 | 790,145 | 0.151 | 438,763 | 1,066,710 | 2.4x |
| **1.0M** | 49 | 766,399 | 0.160 | 438,763 | 1,084,225 | 2.5x |
| 1.25M | 36 | 755,901 | 0.110 | 548,690 | 1,084,225 | 2.0x |
| **1.5M** | 25 | 757,592 | **0.100** | 542,112 | 989,948 | 1.8x |
| 2.0M | 16 | 761,902 | 0.106 | 548,690 | 1,084,225 | 2.0x |
| 2.5M | 9 | 753,733 | 0.101 | 542,112 | 989,948 | 1.8x |
| 3.0M | 6 | 752,775 | 0.100 | 542,112 | 989,948 | 1.8x |

**Ideal**: 761,169 people per seat

### Constrained System (State Must Remain ≥ 761K)

| Threshold | Qualified | Rejected | Mean Pop/Seat | CV | Min Pop/Seat | Max Pop/Seat | Range |
|-----------|-----------|----------|---------------|-----|--------------|--------------|-------|
| 0.5M | 138 | **3** | 754,636 | 0.185 | **503,311** ↑ | 1,097,379 | 2.2x ↓ |
| 0.75M | 88 | **1** | 790,598 | 0.144 | 542,112 ↑ | 1,066,710 | 2.0x ↓ |
| 1.0M | 48 | **1** | 766,795 | 0.150 | 548,690 ↑ | 1,084,225 | 2.0x |
| 1.25M+ | Same | 0 | Same | Same | Same | Same | Same |

**Rejected counties** (constraint violated):
- **Hawaii** - Honolulu County (1.02M) would leave 439K in state
- **Rhode Island** - Providence County (661K) would leave 437K in state
- **Delaware** - New Castle County (571K) would leave 419K in state

---

## Key Findings

### 1. Mean Representation is Surprisingly Good

**All thresholds average within 5% of ideal** (761K people/seat):
- Range: 747K - 790K per seat
- **Best**: 2.0M threshold = 761,902 (0.1% from ideal)
- This suggests Huntington-Hill works well even with diverse entity sizes

### 2. Inequality from Minimum Seats (min_seats=1)

**The problem**: Small entities forced to 1 seat even if they "deserve" < 1

**Worst outliers** (unconstrained, 0.5M threshold):
- **Delaware (remaining)**: 419K people → 1 seat (0.55x ideal representation)
- **Montana (remaining)**: 542K-1.08M people depending on threshold
- **Rhode Island (remaining)**: 437K people → 1 seat

**Same as current system**:
- **Wyoming**: 576K people → 1 seat (0.76x ideal)
- Constitutional compromise: Representation vs equality

### 3. Optimal Threshold: 1.5M - 2.0M

**Sweet spot** for minimizing inequality:
- **1.5M threshold**: CV = 0.100 (lowest), 25 counties, range 1.8x
- **2.0M threshold**: Mean = 761,902 (closest to ideal), 16 counties, CV = 0.106

**Why**:
- Fewer qualifying counties = fewer small entities = less variance
- But: Only 25-16 counties get direct representation (vs 141 at 0.5M)
- Trade-off: Simplicity vs direct representation

### 4. Constraint Improves Outliers (Minimal Cost)

**Impact of "state must remain ≥ 761K" rule**:

At **0.5M threshold**:
- Rejects: 3 counties (Hawaii, Rhode Island, Delaware)
- **Improvement**: Min pop/seat 419K → 503K (+20%)
- **Improvement**: Range 2.6x → 2.2x (-15%)
- **Improvement**: CV 0.192 → 0.185 (-4%)
- **Cost**: 138 counties qualify instead of 141 (2% fewer)

At **0.75M+ thresholds**:
- Rejects: 0-1 counties
- Minimal impact (constraint rarely binds)

---

## Representation Inequality Comparison

### Current U.S. System (State-Based)

- **Wyoming**: 576,851 people → 1 seat = 576K people/seat
- **Montana**: 1,084,225 people → 2 seats = 542K people/seat
- **California**: ~39.5M people → 52 seats = 760K people/seat
- **Range**: ~1.4x (Wyoming vs ideal)

### County System (Unconstrained, 1.5M threshold)

- **Montana (remaining)**: 542K people → 1 seat = 542K people/seat
- **Delaware (whole state)**: 990K people → 1 seat = 990K people/seat
- **Large counties**: ~761K people/seat (proportional)
- **Range**: 1.8x (worst vs ideal)

### County System (Constrained, 1.5M threshold)

- **Montana (remaining)**: 542K people → 1 seat = 542K people/seat
- **Rhode Island (whole state)**: 1,097K people → 2 seats = 549K people/seat
- **Large counties**: ~761K people/seat (proportional)
- **Range**: 1.8x (same as unconstrained at this threshold)

**Conclusion**: County system has **comparable or slightly worse inequality** than current system (1.8x vs 1.4x range), but same constitutional trade-off.

---

## Policy Implications

### Option 1: Accept Inequality (min_seats=1)

**Pro**:
- Constitutional precedent (current state system)
- Every entity gets minimum representation
- Simpler implementation (integer seats)

**Con**:
- Small entities overrepresented (419K-542K people/seat)
- Range: 1.8x-2.6x inequality depending on threshold
- Same criticism as current system

**With constraint** (state must remain ≥ 761K):
- Reduces worst outliers (+20% improvement)
- Rejects only 1-3 counties
- **Recommended if using this approach**

### Option 2: Weighted Voting (Fractional Seats)

**Pro**:
- **Perfect representation equality** (CV = 0)
- Every person's vote worth exactly the same
- Constitutional purity ("one person, one vote")

**Con**:
- Requires House procedural changes
- No federal precedent for weighted voting
- Complex vote counting (0.5 representative = 0.5 votes)

**Example**:
- Small county (384K people) → 0.50 seats → casts 0.50 votes
- Medium county (761K people) → 1.00 seats → casts 1.00 votes
- Large county (10M people) → 13.14 seats → casts 13.14 votes

### Option 3: Hybrid Approach

**Constrained threshold + Rounding rules**:
- Large counties: Integer seats (round normally)
- Small counties: Pooled in state, get fractional seats via weighted voting
- State pools: Always ≥ 1 seat (may use weighted voting if < 761K)

**Example** (1.5M threshold):
- 25 large counties: Integer seats (13, 7, 6, 4, 4, ...)
- 50 states: Remaining populations get seats
  - Large states (≥2 seats): Integer reps
  - Small states (1 seat): May use weighted if < 761K

---

## Recommendations for Paper 14

### Main Narrative

1. **Frame as constitutional trade-off** (like current system)
   - "County system faces same inequality as state system"
   - "Choice: Minimum representation OR perfect proportionality"

2. **Highlight constraint solution**
   - "Simple rule prevents extreme outliers"
   - "State must remain ≥ 1 ideal district"
   - "Rejects only 1-3 counties, improves min pop/seat by 20%"

3. **Identify optimal threshold**
   - "1.5M-2.0M minimizes inequality (CV = 0.100-0.106)"
   - "But only 16-25 counties benefit"
   - "vs 0.5M: 141 counties, but CV = 0.185-0.192"

4. **Present weighted voting alternative**
   - "With fractional votes: Perfect equality (CV = 0)"
   - "Requires House procedural innovation"
   - "International precedent exists"

### Key Figures

**Figure 1**: Threshold sensitivity (from earlier analysis)
- X: Threshold, Y: % seats to counties
- Shows 0.5M = 51%, 1.5M = 21%, 3M = 9%

**Figure 2**: Representation equality vs threshold
- X: Threshold, Y: CV (coefficient of variation)
- Shows sweet spot at 1.5M
- Compare constrained vs unconstrained

**Figure 3**: Population per seat distribution
- Box plots at different thresholds
- Show range, outliers, median
- Highlight worst cases (Delaware, Montana)

**Figure 4**: Comparison to current system
- Side-by-side: State system vs County system
- Same inequality metric (pop/seat)
- "Similar constitutional compromise"

### Tables

**Table 1**: Threshold ablation (seats allocated)
- Already have this from earlier analysis
- 8 thresholds, counties qualified, seats, % of Congress

**Table 2**: Representation equality metrics
- This document's main table
- Mean, CV, min, max pop/seat
- Constrained vs unconstrained

**Table 3**: Rejected counties under constraint
- Hawaii, Rhode Island, Delaware
- Show why (remaining state too small)

---

## Statistical Significance

### Coefficient of Variation (CV)

**CV = Standard Deviation / Mean**

Normalized measure of inequality:
- **0.0** = Perfect equality (all entities same pop/seat)
- **0.1** = Low inequality (current system target)
- **0.2** = Moderate inequality
- **0.3+** = High inequality

**Results**:
- **Best threshold**: 1.5M (CV = 0.100)
- **Worst threshold**: 0.5M (CV = 0.192)
- **Current state system**: ~0.05-0.10 (estimate)
- **County system**: Comparable at optimal threshold

### Range (Max / Min)

**Ratio of most to least represented**:
- **Current system**: ~1.4x (Wyoming vs ideal)
- **County 1.5M**: 1.8x (worst vs ideal)
- **County 0.5M**: 2.6x (worst vs ideal)

**Interpretation**: County system has slightly more inequality than current system, but within acceptable bounds at optimal threshold.

---

## Implementation Notes

### For Threshold Ablation Script

```python
# Already implemented:
python scripts/threshold_ablation.py --year 2020

# Outputs: Seat allocation across thresholds
```

### For Representation Equality Script

```python
# Already implemented:
python scripts/analyze_representation_equality.py --year 2020

# Outputs: Pop/seat statistics across thresholds
```

### For Constrained Analysis Script

```python
# Already implemented:
python scripts/analyze_with_constraints.py --year 2020 --min-state 761169

# Outputs: Constrained vs unconstrained comparison
# Test other minimums:
python scripts/analyze_with_constraints.py --year 2020 --min-state 1000000  # 1M min
python scripts/analyze_with_constraints.py --year 2020 --min-state 1500000  # 1.5M min
```

---

## Next Steps for Paper

1. ✅ **Data collection**: All 3,142 counties loaded
2. ✅ **Threshold analysis**: 8 thresholds tested
3. ✅ **Representation equality**: Pop/seat statistics computed
4. ✅ **Constraint analysis**: Minimum state rule tested
5. ✅ **Figures**: 4 figures generated (threshold sensitivity, equality, distribution, constraint)
6. ⏳ **Fragmentation analysis**: REQUIRES enacted district shapefiles (2000, 2010, 2020)
7. ⏳ **County boundaries**: REQUIRES county geometry shapefiles for spatial overlay
8. ⏳ **Writing**: Draft results and discussion sections
9. ⏳ **VRA analysis**: Test with/without VRA for affected states (optional)

---

**Status**: Representation equality analysis complete. Figures generated. Core argument established.

**Data Needed for Fragmentation Analysis**:
- Enacted congressional district shapefiles (2000, 2010, 2020) - from Census Bureau or DRA
- County boundary shapefiles with geometries - from Census TIGER/Line

**Current Focus**: Paper can be written with existing analyses. Fragmentation analysis would strengthen the autonomy argument but requires external spatial data.
