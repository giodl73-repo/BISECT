# Paper 17: Partisan Similarity Districts - Experiment Status

**Last Updated**: 2026-02-08
**Status**: Experimental infrastructure complete, initial tests successful

## Objective

Create politically homogeneous districts using edge-weighted graph partitioning based on partisan vote similarity.

## Method

**Edge weight formula**:
```
w = alpha  if |lean_i - lean_j| < tau
w = 1      otherwise
```

**Parameters tested**:
- alpha (scaling factor): 1, 5, 10, 25, 50, 100
- tau (similarity threshold): 10, 15, 20 percentage points

## Scripts

### `scripts/experiments/partisan_similarity_run.py`
Runs redistricting with partisan similarity edge-weighting.

Usage:
```bash
# Single state, single config
python scripts/experiments/partisan_similarity_run.py --year 2020 --states CA --alpha 1 10 50 --tau 15

# Multi-state, full ablation
python scripts/experiments/partisan_similarity_run.py --year 2020 --states CA TX NY PA --alpha all --tau all

# All 50 states (production run)
python scripts/experiments/partisan_similarity_run.py --year 2020 --alpha 1 10 50 --tau 15
```

### `scripts/experiments/partisan_similarity_analyze.py`
Analyzes results and generates figures/tables for paper.

Usage:
```bash
python scripts/experiments/partisan_similarity_analyze.py --year 2020
```

## Initial Test Results

### New Hampshire (2 districts)
- **Baseline (α=1)**: 0% safe seats >10pp
- **Moderate (α=10)**: 50% safe seats >10pp (1 of 2)

### California (52 districts)
- **Baseline (α=1)**: 82.7% safe (>10pp), 65.4% super-safe (>20pp), PP=0.234
- **Strong (α=50)**: 82.7% safe (>10pp), 67.3% super-safe (>20pp), PP=0.179

**Key finding**: Strong weighting increases super-safe seats but reduces compactness.

## Data

- **Election data**: 2020 presidential election at tract level (83K tracts)
- **Geometry**: 2020 Census block groups (239K units)
- **Resolution**: Block groups aggregated to tract level for election data merge

## Next Steps

1. **Full ablation study**: Run all 6 α × 3 τ = 18 configurations
2. **Multi-state analysis**: CA, TX, NY, PA, FL (top 5 by population)
3. **Comparison to enacted maps**: Load enacted 2022 maps and compare metrics
4. **Generate figures**: Trade-off curves, safe seat progression, state comparisons
5. **Write results section**: Draft sections 3-4 with tables and figures

## Output Location

`outputs/partisan_similarity/2020/{config}/`
- `districts.parquet`: District assignments with geometries
- `district_statistics.csv`: Per-district metrics (lean, compactness, population)
