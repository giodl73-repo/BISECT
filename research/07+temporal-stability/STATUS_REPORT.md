# Temporal Stability Project - Status Report
**Date:** 2026-02-08
**Status:** 70% Ready - Data Complete, Processing Needed

## ✅ Completed

### 1. Project Structure ✓
```
research/gerry-temporal-stability/
├── PLAN.md                    ✓ Research design complete
├── README.md                  ✓ Documentation complete
├── main.tex                   ✓ LaTeX framework ready
├── references.bib             ✓ Bibliography ready
├── sections/ (8 files)        ✓ All sections drafted
├── scripts/ (5 files)         ✓ All experiment scripts created
│   ├── run_2010_redistricting.py        [UPDATED - uses FIPS paths]
│   ├── compute_stability_metrics.py     [UPDATED - uses FIPS paths]
│   ├── analyze_community_disruption.py
│   └── visualize_stability.py
├── run_all_experiments.py     ✓ Master orchestrator ready
├── results/                   ✓ Created (empty)
├── figures/                   ✓ Created (empty)
└── tables/                    ✓ Created (empty)
```

### 2. Census Data ✓
All raw 2010 census data exists:

**TIGER Tracts:** 5/5 states ✓
- `data/2010/tiger/tracts/tl_2010_01_tract10/` (Alabama)
- `data/2010/tiger/tracts/tl_2010_13_tract10/` (Georgia)
- `data/2010/tiger/tracts/tl_2010_22_tract10/` (Louisiana)
- `data/2010/tiger/tracts/tl_2010_28_tract10/` (Mississippi)
- `data/2010/tiger/tracts/tl_2010_45_tract10/` (South Carolina)

**Demographics:** 5/5 states ✓
- `data/2010/demographics/alabama_demographics_2010.csv` (70 KB)
- `data/2010/demographics/georgia_demographics_2010.csv` (121 KB)
- `data/2010/demographics/louisiana_demographics_2010.csv` (69 KB)
- `data/2010/demographics/mississippi_demographics_2010.csv` (40 KB)
- `data/2010/demographics/south_carolina_demographics_2010.csv` (67 KB)

### 3. Tract Relationship Files ✓
Downloaded from Census Bureau and processed:
- `data/tract_relationships/alabama_2010_2020.csv` (1,967 relationships)
- `data/tract_relationships/georgia_2010_2020.csv` (3,719 relationships)
- `data/tract_relationships/louisiana_2010_2020.csv` (2,069 relationships)
- `data/tract_relationships/mississippi_2010_2020.csv` (1,276 relationships)
- `data/tract_relationships/south_carolina_2010_2020.csv` (1,889 relationships)

### 4. Script Updates ✓
- `run_2010_redistricting.py` - Fixed to use FIPS-based tract paths
- `compute_stability_metrics.py` - Added STATE_TO_FIPS mapping

## ❌ Remaining Work

### 1. Build Adjacency Matrices (CRITICAL)
**Required for:** Graph partitioning

**Command:**
```bash
python scripts/data/build_adjacency.py --year 2010 --states AL GA LA MS SC
```

**Output:** 
- `outputs/data/2010/adjacency/alabama_adjacency.npz`
- `outputs/data/2010/adjacency/georgia_adjacency.npz`
- `outputs/data/2010/adjacency/louisiana_adjacency.npz`
- `outputs/data/2010/adjacency/mississippi_adjacency.npz`
- `outputs/data/2010/adjacency/south_carolina_adjacency.npz`

**Time:** ~5-10 minutes

### 2. Check config_2010.py Exists
Need to verify `scripts/config_2010.py` has STATE_DISTRICTS mapping:
```python
STATE_DISTRICTS = {
    'alabama': 7,
    'georgia': 14,
    'louisiana': 6,
    'mississippi': 4,
    'south_carolina': 7
}
```

### 3. Run Experiments
After adjacency matrices are built:
```bash
cd research/gerry-temporal-stability
python run_all_experiments.py
```

This will:
1. Check prerequisites
2. Run 2010 redistricting (n-way + recursive)
3. Compute stability metrics
4. Analyze community disruption
5. Generate visualizations

**Time:** ~1-2 hours total

## 🎯 Quick Start

Run these commands in order:

```bash
# 1. Build adjacency matrices (MUST DO FIRST)
python scripts/data/build_adjacency.py --year 2010 --states AL GA LA MS SC

# 2. Run all experiments
cd research/gerry-temporal-stability
python run_all_experiments.py

# 3. Check results
ls -lh results/
ls -lh figures/

# 4. Compile paper (after filling in results)
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

## 📊 Expected Results

Once experiments complete, you should have:

### Results Files
- `results/alabama_2010_nway_partition.csv`
- `results/alabama_2010_recursive_partition.csv`
- `results/georgia_2010_nway_partition.csv`
- `results/georgia_2010_recursive_partition.csv`
- ... (20 partition files total: 5 states × 2 methods × 2 years)
- `results/stability_metrics.csv`
- `results/county_disruption.csv`

### Figures
- `figures/stability_comparison.png`
- `figures/state_breakdown.png`
- `figures/demographic_correlation.png`

### Summary Statistics (Expected)
| Metric | N-Way | Recursive | Improvement |
|--------|-------|-----------|-------------|
| Tract Stability | ~70% | ~80% | +10 pts |
| Pop Disruption | ~30% | ~20% | -10 pts |
| Boundary Stability | ~65% | ~78% | +13 pts |
| County Disruption | ~36% | ~22% | -14 pts |

## 🔍 Troubleshooting

If adjacency building fails:
```bash
# Check tract data is readable
python -c "import geopandas as gpd; tracts = gpd.read_file('data/2010/tiger/tracts/tl_2010_01_tract10'); print(f'Alabama: {len(tracts)} tracts')"

# Check demographics is readable
python -c "import pandas as pd; df = pd.read_csv('data/2010/demographics/alabama_demographics_2010.csv'); print(df.shape)"
```

If redistricting fails:
```bash
# Test with just Alabama first
cd research/gerry-temporal-stability
python -c "from scripts.run_2010_redistricting import load_2010_census_data; data = load_2010_census_data('alabama'); print(f'Loaded {len(data)} tracts')"
```

## 📝 Notes

- 2020 data already exists from Papers 1-2, no action needed
- Scripts have been updated to use FIPS-based paths (tl_2010_01_tract10, etc.)
- All experiments are read-only, won't modify existing data
- Results will be saved in `research/gerry-temporal-stability/results/`
- Paper sections have placeholders (TBD) that need real results

## Next Step

**ACTION REQUIRED:** Build adjacency matrices
```bash
python scripts/data/build_adjacency.py --year 2010 --states AL GA LA MS SC
```

After this completes, everything else will run smoothly!
