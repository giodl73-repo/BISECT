# Paper 14: County Representation - Status Summary

**Date**: 2026-02-08
**Venue**: State Politics & Policy Quarterly
**Focus**: Direct county representation as alternative to traditional congressional redistricting

---

## Current Status: READY FOR WRITING

All core analyses complete. Figures generated. Framework ready for fragmentation analysis pending external data.

---

## Completed Work

### ✅ Data Collection
- **All 3,142 U.S. counties** loaded from 2020 Census block group data
- **County FIPS extraction** from GEOIDs (1500000USSSCCCTTTTTTB format)
- **State populations** for all 50 states + DC
- **Output**: `outputs/data/2020/counties/all_counties_2020.csv`
- **Total population**: 330,759,736 (99.79% of census total)

### ✅ Huntington-Hill Module
- **Core algorithm** implemented in `src/apportionment/huntington_hill/`
- Priority value calculation: P(n) = population / sqrt(n * (n+1))
- Supports minimum seats constraint (min_seats=1 for constitutional requirement)
- Tested with all county thresholds

### ✅ Threshold Ablation Analysis
- **Script**: `research/14+county-representation/scripts/threshold_ablation.py`
- **Thresholds tested**: 0.5M, 0.75M, 1.0M, 1.25M, 1.5M, 2.0M, 2.5M, 3.0M
- **Key finding**: At 0.5M threshold, 141 counties qualify for 223 seats (51.3% of Congress)
- **Trade-off identified**: Lower threshold = more direct representation, higher threshold = simpler system

### ✅ Representation Equality Analysis
- **Script**: `research/14+county-representation/scripts/analyze_representation_equality.py`
- **Key metrics computed**:
  - Mean population per seat (747K-790K, within 5% of ideal 761K)
  - Coefficient of Variation (CV) - normalized inequality measure
  - Min/max outliers showing impact of min_seats=1 constraint
- **Optimal threshold identified**: 1.5M (CV=0.100, lowest inequality, 25 counties)
- **Constitutional trade-off**: Same as current state system - small entities overrepresented

### ✅ Constraint Analysis
- **Script**: `research/14+county-representation/scripts/analyze_with_constraints.py`
- **Rule tested**: County can only qualify if remaining state >= 761K (1 ideal district)
- **Impact at 0.5M threshold**:
  - Rejects only 3 counties (HI, RI, DE)
  - Improves worst case by 20% (419K → 503K per seat)
  - Reduces inequality range (2.6x → 2.2x)
- **Elegant solution**: Prevents extreme outliers with minimal cost

### ✅ Figure Generation
- **Script**: `research/14+county-representation/scripts/generate_figures.py`
- **Output directory**: `research/14+county-representation/figures/`

**Figure 1: Threshold Sensitivity**
- Shows qualifying counties and % of Congress vs threshold
- Dual y-axis plot with 50% reference line
- Blue (counties) and red (% seats) curves

**Figure 2: Representation Equality**
- CV (coefficient of variation) vs threshold
- Highlights optimal threshold (1.5M) with gold star
- Shows sweet spot for minimizing inequality

**Figure 3: Population per Seat Distribution**
- Box plots for 5 key thresholds (0.5M, 1.0M, 1.5M, 2.0M, 3.0M)
- Green dashed line at ideal (761,169)
- Shows outliers and distribution spread

**Figure 4: Constraint Comparison**
- Side-by-side: Constrained vs unconstrained systems
- Left: Min pop/seat improvement
- Right: CV (overall inequality) impact

---

## Framework Ready (Pending Data)

### ⏳ County Fragmentation Analysis
- **Script**: `research/14+county-representation/scripts/analyze_county_fragmentation.py`
- **Status**: Framework complete, needs enacted district shapefiles
- **Purpose**: Show how current system fragments large counties across multiple districts

**What it will show**:
- LA County (10M people): Currently spans ~13 districts → FRAGMENTED
- Cook County (5.3M): Currently spans ~7 districts → FRAGMENTED
- vs County system: Each = autonomous political unit with multiple representatives

**Data needed**:
1. **Enacted congressional district shapefiles**:
   - Census TIGER/Line: `https://www2.census.gov/geo/tiger/TIGER{year}/CD/`
   - Files: `tl_{year}_us_cd116.zip` (2020), cd113 (2010), cd110 (2000)
   - Target location: `data/enacted/{year}/tl_{year}_us_cd.shp`

2. **County boundary shapefiles**:
   - Census TIGER/Line: `https://www2.census.gov/geo/tiger/TIGER{year}/COUNTY/`
   - File: `tl_{year}_us_county.zip`
   - Target location: `data/tiger/counties/tl_{year}_us_county.shp`

**Spatial analysis to compute**:
- Number of districts each county intersects
- Population distribution across district pieces
- Fragmentation index (Herfindahl-Hirschman Index)
- Compare 2000 vs 2010 vs 2020 to show trend

---

## Key Findings (Ready for Paper)

### 1. County System Faces Same Inequality Trade-off as State System
- **Mean representation**: Excellent (747K-790K vs ideal 761K)
- **Outliers from min_seats=1**: Small entities overrepresented (419K-542K per seat)
- **Range**: 1.8x-2.6x inequality depending on threshold
- **Current state system**: ~1.4x range (Wyoming vs ideal)
- **Conclusion**: County system has comparable inequality, same constitutional compromise

### 2. Optimal Threshold: 1.5M - 2.0M
- **1.5M threshold**: CV=0.100 (lowest inequality), 25 counties, 1.8x range
- **2.0M threshold**: Mean=761,902 (closest to ideal), 16 counties, CV=0.106
- **Trade-off**: Fewer counties benefit but lower inequality
- **Policy choice**: Simplicity vs broad direct representation

### 3. Constraint Elegantly Solves Outlier Problem
- **Rule**: State must remain ≥ 761K after county forks
- **At 0.5M**: Rejects 3 counties, improves min by 20%, cost only 2% fewer counties
- **At 0.75M+**: Rejects 0-1 counties, minimal impact
- **Recommendation**: Use constraint at all thresholds

### 4. Representation Equality Alternative: Weighted Voting
- **With fractional seats**: Perfect equality (CV=0)
- **Example**: Small county (384K) → 0.50 seats → casts 0.50 votes in House
- **Requires**: House procedural changes, no federal precedent
- **International**: Exists in some parliamentary systems

---

## Paper Structure (Ready to Write)

### Abstract
- Problem: Traditional redistricting fragments large counties
- Proposal: Counties above threshold become autonomous entities
- Method: Huntington-Hill apportionment with 8 thresholds tested
- Finding: Mean equality excellent, outliers from min_seats=1
- Solution: Minimum remaining state constraint + weighted voting option

### Introduction
- Gerrymandering crisis and redistricting litigation
- Large counties lose political identity when carved up
- County-based system preserves natural communities
- Research question: At what threshold? What are representation equality implications?

### Literature Review
- Congressional apportionment history
- Huntington-Hill method and Baker v. Carr
- County governance and home rule
- Representation equality principles

### Method
- Data: 3,142 counties, 2020 Census
- Huntington-Hill with min_seats=1
- 8 thresholds from 0.5M to 3.0M
- Metrics: CV, pop/seat, min/max outliers
- Constraint: State must remain ≥ 761K

### Results
- **Threshold sensitivity** (Figure 1): 141 counties at 0.5M → 6 counties at 3.0M
- **Representation equality** (Figure 2): Optimal at 1.5M (CV=0.100)
- **Distribution analysis** (Figure 3): Mean close to ideal, outliers from min_seats
- **Constraint impact** (Figure 4): Improves worst case by 20%

### Discussion
- Constitutional trade-off: Minimum representation vs perfect proportionality
- Same issue as current state system (Wyoming problem)
- Policy choices:
  1. Accept inequality (min_seats=1) - constitutional precedent
  2. Weighted voting (fractional seats) - perfect equality
  3. Hybrid approach - large counties integer, small pools fractional
- Optimal threshold depends on goals (representation vs simplicity)

### Conclusion
- County system preserves local identity
- Faces same equality challenges as state system
- Constraint elegantly handles outliers
- Weighted voting offers path to perfect equality
- Future work: Fragmentation analysis with enacted plans

---

## Files Summary

### Scripts (All Working)
```
research/14+county-representation/scripts/
├── prepare_county_data.py              # Data preparation (3,142 counties)
├── threshold_ablation.py               # Seat allocation across thresholds
├── analyze_representation_equality.py  # Pop/seat statistics and CV
├── analyze_with_constraints.py         # Minimum state constraint testing
├── analyze_county_fragmentation.py     # Framework (needs shapefiles)
└── generate_figures.py                 # Publication-quality figures
```

### Outputs
```
research/14+county-representation/
├── figures/
│   ├── figure1_threshold_sensitivity.png
│   ├── figure2_representation_equality.png
│   ├── figure3_pop_per_seat_distribution.png
│   └── figure4_constraint_comparison.png
├── REPRESENTATION_EQUALITY_FINDINGS.md  # Comprehensive findings doc
├── PAPER_STATUS.md                      # This file
└── _panel.yaml                          # Panel tracking metadata
```

### Source Code
```
src/apportionment/huntington_hill/
├── __init__.py
└── algorithm.py                         # Core H-H implementation
```

### Data
```
outputs/data/2020/counties/
└── all_counties_2020.csv               # 3,142 counties with populations
```

---

## Next Steps

### Immediate (Can Do Now)
1. **Write paper sections** using existing analyses and figures
2. **LaTeX compilation** with figures embedded
3. **Literature review** on county governance and apportionment
4. **Revise argument** to emphasize county autonomy (per user direction)

### Short-term (Needs External Data)
1. **Download enacted district shapefiles** from Census Bureau
2. **Download county boundary shapefiles** from TIGER/Line
3. **Run fragmentation analysis** showing how counties are carved up
4. **Generate fragmentation figures** for 2000/2010/2020 comparison

### Optional Enhancements
1. **VRA analysis**: Run with/without 5x @ 40% edge weighting
2. **Compactness comparison**: County boundaries vs METIS districts
3. **Multi-year analysis**: Compare 2000, 2010, 2020 thresholds
4. **Historical trend**: Show if fragmentation increasing over time

---

## Commands to Run

### Generate figures (already done)
```bash
python research/14+county-representation/scripts/generate_figures.py --year 2020
```

### Run individual analyses
```bash
python research/14+county-representation/scripts/threshold_ablation.py --year 2020
python research/14+county-representation/scripts/analyze_representation_equality.py --year 2020
python research/14+county-representation/scripts/analyze_with_constraints.py --year 2020 --min-state 761169
```

### Fragmentation (when data available)
```bash
python research/14+county-representation/scripts/analyze_county_fragmentation.py --year 2020 --states CA TX IL
python research/14+county-representation/scripts/analyze_county_fragmentation.py --year 2020  # All states
```

### Compile LaTeX
```bash
cd research/14+county-representation
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

---

## Paper Writing Focus

### Main Argument (Updated Per User Direction)
**"Counties lose autonomy when they get carved up across multiple districts"**

1. **Current system**: LA County (10M) fragmented across ~13 districts
2. **Problem**: Loses political coherence, no unified representation
3. **County system**: LA County = autonomous entity with 13 representatives
4. **Preservation**: Natural community boundaries maintained

### Supporting Evidence
1. **Threshold analysis**: Shows feasibility (141 counties at 0.5M can work)
2. **Equality metrics**: Shows trade-offs (mean good, outliers from min_seats)
3. **Constraint solution**: Shows how to handle edge cases elegantly
4. **Fragmentation** (pending): Shows extent of current problem

### Key Tables for Paper
- **Table 1**: Threshold ablation (8 rows, 5 columns)
- **Table 2**: Representation equality metrics (8 rows, 7 columns)
- **Table 3**: Rejected counties under constraint (3 rows: HI, RI, DE)

---

## Status: READY FOR PAPER WRITING

All analyses complete. Figures generated. Core argument established.

**Can write now**: Introduction, Literature Review, Method, Results, Discussion, Conclusion

**Can add later**: Fragmentation analysis once shapefiles downloaded (strengthens autonomy argument)
