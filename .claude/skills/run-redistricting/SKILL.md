---
name: run-redistricting
description: Execute full 50-state redistricting pipeline with validation/monitoring. Generates districts, maps for all/subset of states. Handles prerequisites, progress tracking, error recovery.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Run Redistricting Pipeline

Executes complete congressional redistricting for 50 states + DC → district assignments, compactness metrics, political/demographic analysis, visualizations, dashboard.

## Prerequisites
**Validate**:
1. Census tract data `data/tracts/{year}/` exists
2. Adjacency graphs `data/adjacency/{year}/` built
3. Output directory doesn't conflict (or use `--reset`)
4. Election data available for political analysis (2020 only)

**If missing**: `/census-download` for tracts, `/adjacency-build` for graphs

## When to Use
User says: "run redistricting [year]", "generate congressional districts", "create maps for all states"

## Workflow

### Step 1: Validate Data
```bash
ls data/tracts/2020/      # Check tract data
ls data/adjacency/2020/   # Check graphs
```

### Step 2: Execution Mode

**Print-Only** (ALWAYS FIRST, ~1s):
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --print-only
```
Validates params, shows what will execute, catches config errors

**Small State Test** (VT/DE, ~30s each):
```bash
--year 2020 --version test --states "VT,DE"
```
Quick end-to-end validation

**Full 50-State** (2-4h, parallel):
```bash
--year 2020 --version v1 --dpi 150
```
Production run

### Step 3: Parameters

**Required**: `--year` (2000/2010/2020), `--version` (v1, v2, test)

**Common Options**:
| Param | Values | Default | Purpose |
|-------|--------|---------|---------|
| `--mode` | edge-weighted, unweighted | edge-weighted | Algorithm mode |
| `--dpi` | 50-300 | 150 | Map resolution |
| `--states` | Comma-separated | All 50 | Subset for testing |
| `--reset` | Flag | False | Delete existing outputs |
| `--force` | Flag | False | Regenerate all (ignore skip) |
| `--validate` | Flag | False | Run validation after |
| `--skip-redistricting` | Flag | False | Analysis only |
| `--skip-analysis` | Flag | False | Redistricting only |

**Example**:
```bash
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version v1 --mode edge-weighted --dpi 150 --validate
```

### Step 4: Monitor Progress

**STATUS protocol shows**:
• **Top-level**: Overall progress (redistricting → analysis → post-processing)
• **State bars** (parallel, 50 simultaneous): Per-state redistricting
• **Analysis bars** (parallel): Compactness, political, demographic per state
• **Post-processing** (sequential): National aggregation

**Milestones**: Redistricting (30-60m) → Analysis (30-60m) → Visualization (30-60m) → Post-processing (10-15m) → Dashboard (5s)

### Step 5: Handle Errors

**Common Issues**:
```
Missing data: "Census tract data not found" → /census-download --year 2020 --state X
Graph connectivity: "Multiple connected components" → Rebuild adjacency with water connections
Unicode (Windows): "UnicodeEncodeError" → Report as bug (should use ASCII not ✓)
METIS: "Edge weight overflow" → Check extremely long boundaries (>100km)
Memory: "MemoryError" → Close apps or process states individually
```

## Pipeline Stages

### Stage 1: Redistricting (Per-State, Parallel)
**Per state**: Load tracts `data/tracts/{year}/{state}_tracts_{year}.parquet` → Load graph `data/adjacency/{year}/{state}_adjacency_{year}.pkl` → METIS recursive bisection → Validate ±0.5% population → Save `outputs/us_{year}_{version}/states/{state}/data/districts.csv`
**Outputs**: `districts.csv`, `district_summary.csv`, `rounds_hierarchy.csv`

### Stage 2: Analysis (Per-State, Parallel)
• **Compactness** (always): Polsby-Popper (4π × area / perimeter²), Reock (area / bounding circle) → `compactness.csv`, maps
• **Political** (2020 only, if election data): D/R vote shares from 2020 presidential → `political_lean.csv`, maps
• **Demographic** (always): Race/ethnicity (White, Black, Hispanic, Asian, Other) → `demographic_composition.csv`, maps

### Stage 3: State Maps (Per-State, Parallel)
**Generate**: District assignment, compactness score, political lean (if 2020), demographic composition (3 types)
**Style**: Thin white tract boundaries, thick black district boundaries, color-coded by metric, district numbers labeled

### Stage 4: Post-Processing (Sequential)
• **National maps**: All 435 districts, political lean, demographics, AK/HI insets
• **Metro maps** (if CBSA data): Top 20 metro areas, focused views
• **Round progression**: Recursive bisection visualization (Round 1: 2 regions → Round 9: 435 districts)

### Stage 5: Dashboard
**Create**: Static HTML with all data baked in → Interactive state/district navigation → Links to maps/CSVs → Auto-opens in browser
**Output**: `outputs/us_{year}_{version}/index.html`

## Runtime

| Config | Time |
|--------|------|
| Print-only | ~1s |
| Small state (VT/DE) | ~30s |
| Medium state (AL) | ~2m |
| Large state (CA/TX) | ~5m |
| 50 states (parallel) | 2-4h |
| 50 states (--force) | 3-5h |

## After Completion

1. **Dashboard opens** automatically
2. **Validate** (if `--validate`): `python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v1`
3. **Review metrics**: Population balance (±0.5%), compactness scores, political lean
4. **Archive** (if production): `cp -r outputs/us_2020_v1 archived_runs/`

## Troubleshooting
• **Pipeline hangs**: Check zombie Python processes, use CANCEL.bat
• **Maps wrong**: Check DPI, verify data loaded
• **Missing analysis**: Check data availability (election for political)
• **Validation fails**: Review errors, check missing files

## Output Structure
```
outputs/us_{year}_{version}/
├── index.html             # Dashboard
├── maps/                  # National maps (us_all_districts.png, us_political_lean.png, rounds/)
├── metro/                 # Metro area maps (los_angeles/, new_york/)
└── states/                # Per-state outputs
    ├── california/
    │   ├── data/          # districts.csv, district_summary.csv, compactness.csv, political_lean.csv
    │   └── maps/          # districts.png, compactness_polsby_popper.png, political_lean.png
    └── [49 more...]
```

## Output
✅ 435 districts across 50 states
✅ Equal population ±0.5%
✅ Compactness metrics (Polsby-Popper, Reock)
✅ Political analysis (if 2020 + election data)
✅ Demographic analysis
✅ State maps (~6/state, 300+ total)
✅ National maps (5+)
✅ Interactive dashboard

## Next Steps
**Compare runs**: `/run-experiment` for algorithm variants, `/validate-compactness` vs baselines, `/run-statistical-analysis` for quantitative comparisons
