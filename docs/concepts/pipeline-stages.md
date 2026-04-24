# Pipeline Stages

## Short version

Five stages run in sequence: data prep → redistricting → analysis → visualization → dashboard. Each state runs all stages in parallel with other states. The orchestrator tracks completion with hidden marker files so interrupted runs can resume without reprocessing.

---

## The five stages

```
[1] Data Prep     Download + process Census files, build adjacency graphs
     ↓
[2] Redistricting Run METIS recursive bisection, assign tracts to districts
     ↓
[3] Analysis      Compute compactness, political, demographic metrics
     ↓
[4] Visualization Generate maps (PNG) for districts, rounds, metro areas
     ↓
[5] Dashboard     Generate static HTML dashboard baking in all data
```

## Stage 1: Data prep

**Script**: `scripts/data/process_census_data.py` (via `download_orchestrator.py`)

Processes all 50 states in parallel:
1. Download TIGER tract shapefiles from Census Bureau
2. Download PL 94-171 population files
3. Merge population onto tract geometries
4. Compute tract adjacency graphs with shared boundary lengths
5. Write `outputs/data/{year}/adjacency/{state}_adjacency_{year}.pkl`

Completion markers: `.tract_tracts_complete`, `.tract_merge_complete`, `.tract_adjacency_complete`

This stage is skipped if markers exist (resumable). Force re-run with `--reset`.

## Stage 2: Redistricting

**Script**: `scripts/pipeline/run_state_redistricting.py`

For each state:
1. Load adjacency graph from Stage 1
2. Run recursive bisection (METIS `gpmetis`) until N districts
3. Write `final_assignments.pkl` — tract → district mapping
4. Write `intermediate/depth_NN/` — round-by-round assignments

This is the core computation. Takes 2–30 minutes per state depending on size. California (~8,000 tracts, 52 districts) is the longest; Wyoming (~130 tracts, 1 district) is trivial.

## Stage 3: Analysis

**Scripts**: `scripts/pipeline/create_final_district_summary.py`, `scripts/political/analyze_districts.py`, `scripts/demographic/analyze_district_demographics.py`, `scripts/compactness/analyze_district_compactness.py`

Reads `final_assignments.pkl` and computes:
- Population balance (deviation from ideal)
- Polsby-Popper and Reock compactness scores
- Election results per district (2020 only, requires election data)
- Race/ethnicity breakdown per district

Writes CSVs to `states/{state}/data/` and `states/{state}/political/`, `states/{state}/demographic/`, `states/{state}/compactness/`.

## Stage 4: Visualization

**Scripts**: `scripts/pipeline/visualize_*.py`

Generates PNG maps using GeoPandas + Matplotlib:
- `maps/all_districts.png` — final district map with colors
- `maps/rounds/round_NN.png` — bisection round progression
- `maps/districts/district_NN.png` — individual district maps
- `compactness/maps/polsby_popper.png` — compactness heatmap
- `political/maps/partisan_lean.png` — partisan lean by district
- `demographic/maps/majority_race.png` — majority race by district

DPI defaults to 150. Use `--dpi 300` for publication-quality output.

## Stage 5: Dashboard

**Scripts**: `scripts/web/generate_master_dashboard.py`, `scripts/web/generate_dashboard.py`

Generates static HTML files with all data and image references baked in:
- `outputs/{version}/{year}/index.html` — per-year state dashboard
- `outputs/{version}/index.html` — cross-year version dashboard
- `outputs/index.html` — master landing page

The HTML is self-contained: district data, VRA analysis, and artifact paths are injected as JavaScript constants. No server needed — open directly in a browser.

## Running a single stage

```bash
# Data only (skip redistricting)
run -y 2020 -v v1 -s data

# States + nation only (skip data, assume adjacency exists)
run -y 2020 -v v1 -s states nation

# National post-processing only (states already done)
run -y 2020 -v v1 -s nation
```

## Resuming interrupted runs

If a run is interrupted, re-run the same command. The orchestrator reads marker files and skips completed stages/states. States listed in `.states_complete` are skipped. Use `--reprocess` to force everything to re-run.

## Parallel execution

The orchestrator runs 12 states in parallel (default). Each state worker is an independent subprocess reading from the shared adjacency graphs and writing to its own `states/{state}/` directory. No shared state between workers — safe to kill and restart.

## Further reading

- `docs/PIPELINE_OUTPUTS.md` — complete list of every file the pipeline writes
- `docs/RECURSIVE_BISECTION.md` — algorithm detail
- `CLAUDE.md` — quick reference for all run commands
