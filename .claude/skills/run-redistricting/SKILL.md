---
name: run-redistricting
description: Execute the full 50-state congressional redistricting pipeline with validation and monitoring. Use when running redistricting, generating districts, creating maps for all states or a subset. Handles prerequisites, progress tracking, and error recovery.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Run Redistricting Pipeline

## Overview

Execute the complete congressional redistricting pipeline for 50 states + DC. Generates district assignments, compactness metrics, political/demographic analysis, visualizations, and interactive dashboard.

## Prerequisites

Before running, validate:
1. **Census tract data** exists for specified year (`data/tracts/{year}/`)
2. **Adjacency graphs** built for all states (`data/adjacency/{year}/`)
3. **Output directory** doesn't conflict (or use `--reset` to clear)
4. **Election data** available if running political analysis (2020 only)

## When to Use This Skill

- User says: "Run redistricting for [year]"
- User says: "Generate congressional districts"
- User says: "Create maps for all states"
- User wants to test algorithm changes
- User wants to compare different modes (edge-weighted vs unweighted)

## Workflow

### Step 1: Validate Prerequisites

Check data availability:
```bash
# List available tract data
ls data/tracts/2020/
# List available adjacency graphs
ls data/adjacency/2020/
```

If data missing:
- Use `/census-download` skill to get tract data
- Use `/adjacency-build` skill to create graphs

### Step 2: Determine Execution Mode

**Print-Only (Dry Run)** - ALWAYS RUN THIS FIRST:
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --print-only
```
- Validates all parameters thread correctly
- Shows what will be executed
- Catches configuration errors early
- Takes ~1 second

**Small State Test** - Quick validation:
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states "VT,DE"
```
- Tests with Vermont and Delaware (fastest states)
- Runtime: ~30 seconds per state
- Validates full workflow end-to-end

**Full 50-State Run** - Production:
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --dpi 150
```
- Processes all 50 states + DC
- Runtime: 2-4 hours (parallel mode)
- Generates complete outputs

### Step 3: Set Parameters

**Required Parameters**:
- `--year`: Census year (2000, 2010, or 2020)
- `--version`: Output version tag (e.g., v1, v2, test)

**Common Optional Parameters**:
| Parameter | Values | Default | Purpose |
|-----------|--------|---------|---------|
| `--mode` | edge-weighted, unweighted | edge-weighted | Algorithm mode |
| `--dpi` | 50-300 | 150 | Map resolution (higher = larger files) |
| `--states` | Comma-separated | All 50 | Subset for testing |
| `--reset` | Flag | False | Delete existing outputs first |
| `--force` | Flag | False | Regenerate all files (skip skip-logic) |
| `--validate` | Flag | False | Run validation after completion |
| `--skip-redistricting` | Flag | False | Analysis only (districts must exist) |
| `--skip-analysis` | Flag | False | Redistricting only (no analysis/maps) |

**Example with multiple parameters**:
```bash
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 \
  --version v1 \
  --mode edge-weighted \
  --dpi 150 \
  --validate
```

### Step 4: Monitor Progress

Pipeline uses STATUS protocol for real-time progress reporting:

**Progress bars show**:
- **Top-level**: Overall pipeline progress (redistricting → analysis → post-processing)
- **State bars**: Per-state redistricting (parallel, 50 simultaneous)
- **Analysis bars**: Per-state analysis stages (compactness, political, demographic)
- **Post-processing**: National aggregation (sequential)

**Expected milestones**:
1. Redistricting: ~30-60 minutes (parallel)
2. Analysis: ~30-60 minutes (parallel)
3. Visualization: ~30-60 minutes (parallel)
4. Post-processing: ~10-15 minutes (sequential)
5. Dashboard: ~5 seconds

### Step 5: Handle Errors

**Common issues and solutions**:

**Missing data**:
```
Error: Census tract data not found for california
Solution: Run /census-download --year 2020 --state california
```

**Graph connectivity**:
```
Error: Graph has multiple connected components for hawaii
Solution: Rebuild adjacency with water connections
```

**Unicode errors (Windows)**:
```
Error: UnicodeEncodeError: 'charmap' codec can't encode character
Solution: Scripts should use ASCII ([OK] not ✓) - report as bug
```

**METIS errors**:
```
Error: Edge weight overflow
Solution: Check for extremely long boundaries (>100km edge weights)
```

**Out of memory**:
```
Error: MemoryError during California processing
Solution: Close other applications, or process states individually
```

## Pipeline Stages

### Stage 1: Redistricting (Per-State, Parallel)

For each state:
1. Load census tracts (`data/tracts/{year}/{state}_tracts_{year}.parquet`)
2. Load adjacency graph (`data/adjacency/{year}/{state}_adjacency_{year}.pkl`)
3. Run METIS recursive bisection to create districts
4. Validate equal population (within ±0.5%)
5. Save district assignments (`outputs/us_{year}_{version}/states/{state}/data/districts.csv`)

**Outputs per state**:
- `districts.csv` - Tract-to-district assignments
- `district_summary.csv` - District populations, areas
- `rounds_hierarchy.csv` - Recursive bisection tree

### Stage 2: Analysis (Per-State, Parallel)

For each state:

**Compactness** (always runs):
- Polsby-Popper scores: (4π × area) / perimeter²
- Reock scores: area / bounding circle area
- Outputs: `compactness.csv`, `maps/compactness_polsby_popper.png`

**Political** (2020 only, if election data available):
- Partisan lean from 2020 presidential results
- D/R vote shares per district
- Outputs: `political_lean.csv`, `maps/political_lean.png`

**Demographic** (always runs):
- Racial/ethnic composition per district
- White, Black, Hispanic, Asian, Other percentages
- Outputs: `demographic_composition.csv`, `maps/demographic_*.png`

### Stage 3: State Maps (Per-State, Parallel)

Generate visualizations:
- District assignment map
- Compactness score map
- Political lean map (if 2020)
- Demographic composition maps (3 types)

**Map style**:
- Thin white tract boundaries
- Thick black district boundaries
- Color-coded by metric
- District numbers labeled

### Stage 4: Post-Processing (Sequential)

**National maps** (all 50 states combined):
- All 435 districts map
- National political lean map
- National demographic maps
- Includes Alaska/Hawaii insets

**Metro area maps** (if CBSA data available):
- Top 20 metropolitan areas
- Focused district views
- Organized by state

**Round progression** (recursive bisection visualization):
- Maps showing districts at each round
- National-level bisection tree
- Round 1: 2 regions → Round 9: 435 districts

### Stage 5: Dashboard Generation

Create static HTML dashboard:
- Bakes all data into single HTML file
- Interactive state/district navigation
- Links to all maps and CSVs
- Opens automatically in browser

**Output**: `outputs/us_{year}_{version}/index.html`

## Expected Runtime

| Configuration | Time |
|---------------|------|
| Print-only | ~1 second |
| Single small state (VT/DE) | ~30 seconds |
| Single medium state (AL) | ~2 minutes |
| Single large state (CA/TX) | ~5 minutes |
| 50 states (parallel) | 2-4 hours |
| 50 states (--force, regenerate all) | 3-5 hours |

## After Completion

1. **Dashboard opens automatically** in browser
2. **Validate outputs** (if `--validate` flag used):
   ```bash
   python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v1
   ```
3. **Review key metrics**:
   - Population balance (should be within ±0.5%)
   - Compactness scores (higher is better)
   - Political lean distribution
4. **Archive results** if production run:
   ```bash
   # Copy to safe location for comparison
   cp -r outputs/us_2020_v1 archived_runs/
   ```

## Troubleshooting

See [troubleshooting.md](troubleshooting.md) for detailed error resolution.

**Quick fixes**:
- **Pipeline hangs**: Check for zombie Python processes, use CANCEL.bat
- **Maps look wrong**: Check DPI setting, verify data loaded correctly
- **Missing analysis**: Check if data available (election data for political)
- **Validation fails**: Review error messages, check for missing files

## Output Directory Structure

```
outputs/us_{year}_{version}/
├── index.html                      # Dashboard
├── maps/                           # National maps
│   ├── us_all_districts.png
│   ├── us_political_lean.png
│   └── rounds/                     # Round progression
├── metro/                          # Metro area maps
│   ├── los_angeles/
│   └── new_york/
└── states/                         # Per-state outputs
    ├── california/
    │   ├── data/
    │   │   ├── districts.csv
    │   │   ├── district_summary.csv
    │   │   ├── compactness.csv
    │   │   └── political_lean.csv
    │   └── maps/
    │       ├── districts.png
    │       ├── compactness_polsby_popper.png
    │       └── political_lean.png
    └── [49 more states...]
```

## What You'll Get

After successful completion:
- **435 congressional districts** across all 50 states
- **Equal population** within ±0.5% per district
- **Compactness metrics** (Polsby-Popper, Reock)
- **Political analysis** (if 2020 with election data)
- **Demographic analysis** for all districts
- **State maps** (~6 per state, 300+ total)
- **National maps** (5+)
- **Interactive dashboard** with all data

## Next Steps

- Compare multiple runs using different parameters
- Use `/run-experiment` to test algorithm variants
- Use `/validate-compactness` to compare against baselines
- Use `/run-statistical-analysis` for quantitative comparisons
