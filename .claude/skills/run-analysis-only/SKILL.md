---
name: run-analysis-only
description: Run analysis stages without redistricting when district assignments already exist. Use when you want to regenerate analysis, update maps, or add new analysis types without rerunning METIS partitioning.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Run Analysis Only

## Overview

Execute analysis stages (compactness, political, demographic) and regenerate visualizations without rerunning the redistricting algorithm. Useful when district assignments exist and you want to update analysis or change map styling.

## Prerequisites

- District assignments must exist: `outputs/us_{year}_{version}/states/*/data/districts.csv`
- Census tract data available
- For political analysis: 2020 election data required

## When to Use This Skill

- User says: "Regenerate the analysis"
- User says: "Update maps without redistricting"
- District assignments exist, want new analysis type
- Want to change map DPI or styling
- Testing new analysis code

## Workflow

### Step 1: Verify District Assignments Exist

Check that redistricting has been completed:
```bash
# Check for district files
ls outputs/us_2020_v1/states/*/data/districts.csv

# Or use validation
python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v1
```

If districts don't exist, use `/run-redistricting` first.

### Step 2: Run Pipeline with Skip Flag

```bash
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting
```

This executes:
1. ✓ Per-state analysis (parallel)
2. ✓ State map generation (parallel)
3. ✓ Post-processing (national maps, metro maps)
4. ✓ Dashboard regeneration
5. ✗ Redistricting (skipped)

### Step 3: Monitor Progress

Progress bars show:
- Analysis stages per state (compactness, political, demographic)
- Map generation per state
- National aggregation
- Dashboard generation

**Expected runtime**: ~1-2 hours (much faster than full redistricting)

## Common Use Cases

### Use Case 1: Change Map Resolution

```bash
# Original run at DPI 150
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --dpi 150

# Regenerate maps at higher resolution
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --dpi 300 --skip-redistricting --force
```

Note: Use `--force` to override skip logic and regenerate existing maps.

### Use Case 2: Add Political Analysis

```bash
# Original run without election data (e.g., 2010)
# Later, download 2020 election data

# Rerun analysis to add political lean
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --skip-redistricting
```

### Use Case 3: Fix Analysis Bug

```bash
# Fix bug in compactness calculation code
# Regenerate analysis without redistricting

python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --skip-redistricting --force
```

### Use Case 4: Test New Analysis Type

```bash
# Add new analysis script to pipeline
# Test without waiting for full redistricting

python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --skip-redistricting --states "VT,DE"
```

## What Gets Regenerated

**Per-State (Parallel)**:
- `compactness.csv` and compactness maps
- `political_lean.csv` and political maps (if 2020)
- `demographic_composition.csv` and demographic maps
- All state maps in `maps/` directory

**National (Post-Processing)**:
- `maps/us_all_districts.png`
- `maps/us_political_lean.png` (if 2020)
- `maps/us_demographic_*.png`
- `maps/rounds/round_*.png` (round progression)

**Metro Areas** (if CBSA data available):
- Top 20 metro area focused maps

**Dashboard**:
- `index.html` with updated data

## What Does NOT Change

- District assignments (`districts.csv`) - unchanged
- District summaries (`district_summary.csv`) - unchanged
- Rounds hierarchy (`rounds_hierarchy.csv`) - unchanged
- METIS partitioning - not rerun

## Performance Comparison

| Task | With Redistricting | Analysis Only |
|------|-------------------|---------------|
| Small state (VT) | 30 seconds | 10 seconds |
| Medium state (AL) | 2 minutes | 30 seconds |
| Large state (CA) | 5 minutes | 2 minutes |
| All 50 states | 2-4 hours | 1-2 hours |

Analysis-only is **~50-60% faster** because METIS partitioning is skipped.

## Combining with Other Flags

```bash
# Analysis only + specific states + force regeneration
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --states "CA,TX,FL" \
  --force

# Analysis only + validation
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --validate

# Analysis only + print to see what will run
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --skip-redistricting \
  --print-only
```

## Error Handling

**Missing district files**:
```
Error: District assignments not found for california
Solution: Run full redistricting first (remove --skip-redistricting flag)
```

**Mismatched versions**:
```
Error: Census year in config doesn't match district files
Solution: Ensure --year parameter matches original redistricting run
```

## What You'll Get

- Updated analysis CSVs with latest algorithms
- Regenerated maps with current styling/DPI
- Updated dashboard with refreshed data
- All without rerunning expensive METIS partitioning

## Next Steps

- Review updated dashboard
- Compare analysis results before/after changes
- Use `/validate-compactness` to check metrics
- Use `/run-statistical-analysis` for quantitative comparison
