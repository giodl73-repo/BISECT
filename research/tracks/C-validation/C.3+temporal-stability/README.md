# Paper 7: Cross-Census Temporal Stability

**Research Question:** Do recursive bisection's hierarchical structures provide greater temporal stability across census cycles (2010→2020) compared to n-way partitioning?

## Quick Start

```bash
# 1. Run 2010 redistricting (both methods, 5 states)
cd research/gerry-temporal-stability
python scripts/run_2010_redistricting.py

# 2. Compute stability metrics (requires relationship files)
python scripts/compute_stability_metrics.py

# 3. Generate visualizations
python scripts/visualize_stability.py

# 4. Compile paper
cd research/gerry-temporal-stability
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

## Data Requirements

### 2010 Census Data
- Census tracts: `data/2010/tiger/tracts/{state}/`
- Demographics: `data/2010/demographics/{state}/tract_demographics.csv`
- Adjacency: `outputs/data/2010/adjacency/{state}_adjacency.npz`

Download if missing:
```bash
python scripts/data/download_orchestrator.py --year 2010 --states alabama georgia louisiana mississippi south_carolina
python scripts/data/build_adjacency.py --year 2010 --states alabama georgia louisiana mississippi south_carolina
```

### 2020 Census Data
Should already exist from Papers 1-2. If not:
```bash
python scripts/data/download_orchestrator.py --year 2020 --states alabama georgia louisiana mississippi south_carolina
```

### Tract Relationship Files
**Critical:** Need Census Bureau relationship files mapping 2010→2020 tracts.

Download from: https://www.census.gov/geographies/reference-files/time-series/geo/relationship-files.html

Place in: `data/tract_relationships/{state}_2010_2020.csv`

Format:
```
GEOID_TRACT_10,GEOID_TRACT_20,AREALAND_PART,POP_PART,weight
01001020100,01001020100,123456789,5432,1.0
01001020200,01001020201,987654321,2718,0.6
01001020200,01001020202,456789123,1814,0.4
```

## Project Structure

```
research/gerry-temporal-stability/
├── PLAN.md                    # Research plan and design
├── README.md                  # This file
├── main.tex                   # Main LaTeX document
├── references.bib             # Bibliography
├── sections/                  # Paper sections
│   ├── 01_introduction.tex
│   ├── 02_background.tex
│   ├── 03_methodology.tex
│   ├── 04_results.tex
│   ├── 05_discussion.tex
│   ├── 06_limitations.tex
│   ├── 07_future_work.tex
│   └── 08_conclusion.tex
├── scripts/                   # Experiment code
│   ├── run_2010_redistricting.py       # Run 2010 partitioning
│   ├── compute_stability_metrics.py    # Compute temporal metrics
│   ├── analyze_hierarchical_preservation.py  # Tree structure analysis
│   ├── analyze_community_disruption.py       # County-level analysis
│   └── visualize_stability.py          # Generate figures
├── results/                   # Experimental results
│   ├── {state}_2010_nway_partition.csv
│   ├── {state}_2010_recursive_partition.csv
│   ├── {state}_2020_nway_partition.csv
│   ├── {state}_2020_recursive_partition.csv
│   └── stability_metrics.csv
├── figures/                   # Generated figures
└── tables/                    # Generated tables
```

## Experiment Pipeline

### Phase 1: 2010 Redistricting
Run both n-way and recursive bisection on 2010 census data for 5 states.

**Status:** NOT STARTED
- [ ] Download 2010 census data
- [ ] Build 2010 adjacency matrices
- [ ] Run n-way partitioning (2010)
- [ ] Run recursive bisection (2010)

### Phase 2: Temporal Stability Metrics
Compare 2010 and 2020 partitions using tract relationship files.

**Status:** NOT STARTED
- [ ] Download tract relationship files
- [ ] Implement tract mapping algorithm
- [ ] Compute tract reassignment rates
- [ ] Compute population disruption
- [ ] Compute boundary stability
- [ ] Compute hierarchical coherence

### Phase 3: Analysis
**Status:** NOT STARTED
- [ ] Hierarchical structure preservation (dendrograms)
- [ ] Communities of interest disruption (counties)
- [ ] Demographic shift correlation
- [ ] Statistical tests (paired t-tests)

### Phase 4: Visualization
**Status:** NOT STARTED
- [ ] Stability comparison bar charts
- [ ] Hierarchical preservation dendrograms
- [ ] County disruption heatmaps
- [ ] Demographic correlation scatter plots

### Phase 5: Writing
**Status:** STRUCTURE COMPLETE
- [x] Create paper structure
- [x] Draft all sections (placeholder text)
- [ ] Fill results tables with actual data
- [ ] Generate figures
- [ ] Revise and polish

## Key Metrics

### Tract Stability
Fraction of census tracts remaining in same district (2010→2020).
- Higher = more stable
- Expected: Recursive ~80%, N-way ~70%

### Population Disruption
Population-weighted fraction affected by district changes.
- Lower = less disruption
- Expected: Recursive ~20%, N-way ~30%

### Boundary Stability
Fraction of tract boundaries with unchanged district relationship.
- Higher = more stable
- Expected: Recursive ~78%, N-way ~65%

### Hierarchical Coherence Change
Change in modularity of district-level clustering.
- Lower = more preserved structure
- Expected: Recursive ~0.09, N-way ~0.18

## Expected Findings

**Hypothesis:** Recursive bisection provides greater temporal stability due to hierarchical structure preservation.

**Predictions:**
1. Recursive achieves +10-14 percentage points better tract stability
2. Top-level splits preserve >90% tract overlap for recursive
3. Counties see -14 percentage points less disruption with recursive
4. Performance-stability tradeoff: N-way +4.3 pts performance, Recursive +14 pts stability

## Integration with Other Papers

### Paper 1 (Recursive Bisection)
- Uses recursive bisection results for 2020
- Demonstrates edge-weighting effectiveness

### Paper 2 (N-way vs Recursive)
- Establishes performance comparison (n-way +4.3 pts)
- Provides 2020 baseline for both methods

### Paper 7 (This Paper)
- Extends to temporal dimension
- Adds 2010 data for both methods
- Quantifies stability tradeoff

## Timeline

1. **Week 1-2:** Data acquisition (2010 census, relationship files)
2. **Week 3:** Run 2010 redistricting experiments
3. **Week 4:** Compute stability metrics
4. **Week 5:** Hierarchical and community analysis
5. **Week 6:** Statistical analysis and visualization
6. **Week 7-8:** Results interpretation and writing
7. **Week 9:** Revision and polish
8. **Week 10:** Final compilation and submission prep

## References

See `references.bib` for full citations.

Key references:
- Karypis & Kumar (1998): METIS algorithm
- Census Bureau (2021): Tract relationship files
- Bozkaya & Ozsoyoglu (1999): Temporal stability theory
- Morrill (1987): Communities of interest
