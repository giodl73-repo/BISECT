# Print-Only Mode Enhancements - January 10, 2026

## Summary

Updated all print-only modes across the redistricting pipeline to show structured, hierarchical views with tree-style formatting. This makes it much easier to understand what would be executed without actually running the computationally expensive operations.

---

## Enhanced Scripts ✅

### 1. run_complete_redistricting.py ✅
**Master pipeline overview**

**Enhanced to show**:
- Full 4-step pipeline structure upfront
- Tree view of all nested operations
- Clear indication of outputs at each level
- Estimated totals (435 districts, ~600+ maps, 8-12 hours)

**Display**:
```
══════════════════════════════════════════════════════════════════════
US CONGRESSIONAL REDISTRICTING - COMPLETE PIPELINE
══════════════════════════════════════════════════════════════════════

──────────────────────────────────────────────────────────────────────
PIPELINE STRUCTURE - WHAT WOULD BE EXECUTED
──────────────────────────────────────────────────────────────────────

[1/4] PROCESS ALL 50 STATES
    ├─ Script: run_all_states.py
    ├─ For each multi-district state (43 states):
    │   ├─ [1/5] Run redistricting (METIS recursive bisection)
    │   ├─ [2/5] Add cities to districts
    │   ├─ [3/5] Create final district summary
    │   ├─ [4/5] Create individual district maps
    │   └─ [5/5] Visualize intermediate rounds
    └─ Output: outputs/us_2020_v1/states/[state_name]/

[2/4] CREATE ROUNDS HIERARCHY
    ├─ Script: create_rounds_hierarchy.py
    ├─ Aggregate all state bisection hierarchies
    └─ Output: outputs/us_2020_v1/us_rounds_hierarchy.csv

[3/4] CREATE US AGGREGATE FILES
    ├─ Script: create_us_aggregate.py
    ├─ Combine all districts from all states
    ├─ Output: outputs/us_2020_v1/us_all_districts.csv
    ├─ Output: outputs/us_2020_v1/us_district_summary.csv
    └─ Output: outputs/us_2020_v1/US_2020_Redistricting_Report.md

[4/4] CREATE US NATIONAL MAPS
    ├─ Script: create_us_national_map.py
    ├─ Generate national overview maps
    └─ Output: outputs/us_2020_v1/us_national_map.png

──────────────────────────────────────────────────────────────────────
ESTIMATED TOTALS
──────────────────────────────────────────────────────────────────────
  States to process: 43 (multi-district)
  Total districts: 435
  Maps to create: ~600+ (district maps, rounds, national)
  Estimated time: 8-12 hours (full pipeline)
──────────────────────────────────────────────────────────────────────
```

### 2. run_all_states.py ✅
**State-level processing overview**

**Enhanced to show**:
- Tree structure of all 5 steps per state
- Files that would be created at each step
- Timeout for each operation
- Nested subscript execution with visual separators

**Display**:
```
──────────────────────────────────────────────────────────────────────
PRINT-ONLY: California (52 districts)
──────────────────────────────────────────────────────────────────────
Output directory: outputs/us_2020_v1/states/california

Would execute 5 steps:

  ├─ [1/5] Running redistricting
  │      Timeout: 1800s
  │      Would create:
  │        • outputs/.../final_assignments.pkl
  │        • outputs/.../california_52_districts.png
  │        • outputs/.../intermediate/round_*.json
  │
  ├─ [2/5] Adding cities
  │      Timeout: 600s
  │      Would create:
  │        • outputs/.../districts_with_cities.csv
  │
  ├─ [3/5] Creating district summary
  │      Timeout: 300s
  │      Would create:
  │        • outputs/.../district_summary.csv
  │
  ├─ [4/5] Creating 52 district maps
  │      Timeout: 1800s
  │      Would create:
  │        • outputs/.../district_*.png (52 maps)
  │
  └─ [5/5] Visualizing intermediate rounds
         Timeout: 600s
         Would create:
           • outputs/.../maps/rounds/round_*.png

──────────────────────────────────────────────────────────────────────
Now executing subscripts with --print-only to show details...
──────────────────────────────────────────────────────────────────────

▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼ [1/5] ▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼
  [Individual subscript output...]
▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲
```

### 3. run_state_redistricting.py ✅
**Redistricting operation details**

**Enhanced to show**:
- 6-step process with tree structure
- Data loading details
- METIS algorithm parameters
- Intermediate and final outputs
- Estimated depth and file counts

**Display**:
```
══════════════════════════════════════════════════════════════════════
[PRINT-ONLY] California Redistricting - 2020 Census
══════════════════════════════════════════════════════════════════════
Districts: 52
Output: outputs/california_full_TIMESTAMP

WOULD EXECUTE:

  ┌─ STEP 1: Load data
  │   ├─ Load adjacency graph: data/adjacency/ca_adjacency_2020.pkl
  │   │     • Nodes: ~7800 tracts (estimated)
  │   │     • Edges: Queen contiguity + water adjacency
  │   └─ Load tracts: data/raw/ca_tracts_2020.parquet
  │         • Geometry + population data
  │
  ├─ STEP 2: Run recursive bisection
  │   ├─ Algorithm: METIS graph partitioning
  │   ├─ Parameters: niter=100, balanced partitions
  │   ├─ Depth: 6 rounds
  │   └─ Target: 52 equal-population districts
  │
  ├─ STEP 3: Save intermediate results
  │   └─ For each bisection round:
  │       • outputs/.../intermediate/round_N_metadata.json
  │       • outputs/.../intermediate/round_N_assignments.json
  │
  ├─ STEP 4: Analyze final districts
  │   └─ Calculate for each of 52 districts:
  │       • Population count
  │       • Deviation from ideal
  │       • Number of tracts
  │
  ├─ STEP 5: Save final results
  │   └─ Create:
  │       • outputs/.../final_assignments.pkl
  │       • District-to-tract mapping
  │
  └─ STEP 6: Generate final map
      └─ Create:
          • outputs/.../california_52_districts.png
          • Full-state map with all districts colored
          • District numbers labeled at centroids

──────────────────────────────────────────────────────────────────────
ESTIMATED OUTPUT:
──────────────────────────────────────────────────────────────────────
  Files to create: ~15
  Intermediate rounds: 6
  Estimated time: 15-30 minutes
  Max deviation target: <1%
══════════════════════════════════════════════════════════════════════
```

### 4. add_cities_to_districts.py ✅
**City labeling operation**

**Enhanced to show**:
- 5-step spatial join process
- Data loading details
- City selection algorithm
- Output format description

**Display**:
```
══════════════════════════════════════════════════════════════════════
[PRINT-ONLY] Add Cities - California (2020 Census)
══════════════════════════════════════════════════════════════════════

WOULD EXECUTE:

  ┌─ STEP 1: Load data
  │   ├─ Load tracts: data/raw/ca_tracts_2020.parquet
  │   │     • Tract geometries with population
  │   ├─ Load places: data/raw/ca_places_2020.parquet
  │   │     • City boundaries and populations
  │   └─ Load assignments: outputs/.../final_assignments.pkl
  │         • Tract-to-district mappings
  │
  ├─ STEP 2: Perform spatial joins
  │   ├─ Join places to tracts (spatial intersection)
  │   └─ Map tracts to districts
  │       • Results: Places → Tracts → Districts
  │
  ├─ STEP 3: Identify largest city per district
  │   └─ For each district:
  │       • Find all cities intersecting the district
  │       • Select largest by population
  │       • Handle multi-district cities (e.g., NYC)
  │
  ├─ STEP 4: Save results
  │   └─ Create:
  │       • outputs/.../districts_with_cities.csv
  │       • district_id, city_name, city_pop columns
  │
  └─ STEP 5: Generate labeled map
      └─ Create:
          • outputs/.../california_districts_with_cities.png
          • Full-state map with city names labeled

──────────────────────────────────────────────────────────────────────
ESTIMATED OUTPUT:
──────────────────────────────────────────────────────────────────────
  Files to create: 2
  Estimated time: 2-5 minutes
══════════════════════════════════════════════════════════════════════
```

### 5. create_final_district_summary.py ✅
**Summary statistics generation**

**Enhanced to show**:
- 4-step calculation process
- Statistics computed per district
- Spatial analysis details
- Output CSV format

**Display**:
```
══════════════════════════════════════════════════════════════════════
[PRINT-ONLY] Create Summary - California (2020 Census)
══════════════════════════════════════════════════════════════════════

WOULD EXECUTE:

  ┌─ STEP 1: Load data
  │   ├─ Load tracts: data/raw/ca_tracts_2020.parquet
  │   │     • Calculate total population
  │   ├─ Load places: data/raw/ca_places_2020.parquet
  │   │     • Get city geometries
  │   ├─ Load assignments: outputs/.../final_assignments.pkl
  │   │     • District assignments per tract
  │   └─ Load cities: outputs/.../district_cities.csv
  │         • City names per district
  │
  ├─ STEP 2: Calculate district statistics
  │   └─ For each district:
  │       • Total population (sum of tract populations)
  │       • Number of tracts
  │       • Deviation from ideal population (%)
  │       • Largest city name and population
  │
  ├─ STEP 3: Analyze spatial properties
  │   └─ For each district:
  │       • Calculate district area (sq km)
  │       • Calculate centroid coordinates
  │       • Compute compactness metrics
  │
  └─ STEP 4: Save summary
      └─ Create:
          • outputs/.../district_summary.csv
          • Columns: district, population, tracts, deviation%,
            city, city_pop, area_sq_km, centroid_x, centroid_y

──────────────────────────────────────────────────────────────────────
ESTIMATED OUTPUT:
──────────────────────────────────────────────────────────────────────
  Files to create: 1
  Estimated time: 1-2 minutes
══════════════════════════════════════════════════════════════════════
```

### 6. create_individual_district_maps.py ✅
**Individual district map generation**

**Enhanced to show**:
- 3-step rendering process
- Map composition details
- Visual elements included
- Output specifications

**Display**:
```
══════════════════════════════════════════════════════════════════════
[PRINT-ONLY] Create District Maps - California (2020 Census)
══════════════════════════════════════════════════════════════════════

WOULD EXECUTE:

  ┌─ STEP 1: Load data
  │   ├─ Load tracts: data/raw/ca_tracts_2020.parquet
  │   │     • Tract geometries
  │   ├─ Load places: data/raw/ca_places_2020.parquet
  │   │     • City boundaries for labeling
  │   ├─ Load assignments: outputs/.../final_assignments.pkl
  │   │     • District assignments
  │   ├─ Load cities: outputs/.../district_cities.csv
  │   │     • City names for titles
  │   └─ Load summary: outputs/.../district_summary.csv
  │         • Population statistics
  │
  ├─ STEP 2: Generate individual district maps
  │   └─ For each district:
  │       • Focus view on district with 20% margin
  │       • Plot district tracts (bright color)
  │       • Plot neighboring districts (faded)
  │       • Add city labels within district
  │       • Add statistics box (population, deviation)
  │       • Save as district_{N}.png
  │
  └─ STEP 3: Save all maps
      └─ Create:
          • outputs/.../district_1.png
          • outputs/.../district_2.png
          • ... (one per district)
          • Title format: '{State} District {N} - {city_name}'

──────────────────────────────────────────────────────────────────────
ESTIMATED OUTPUT:
──────────────────────────────────────────────────────────────────────
  Files to create: ~52 (varies by state)
  Resolution: 12x10 inches @ 300 DPI
  Estimated time: 5-15 minutes
══════════════════════════════════════════════════────────────════════
```

### 7. visualize_all_rounds.py ✅
**Already had good print-only mode**

**Display**:
```
══════════════════════════════════════════════════════════════════════
[PRINT-ONLY] Visualize Rounds - California - 2020 Census
══════════════════════════════════════════════════════════════════════
Would load files:
  - Tracts: data/raw/ca_tracts_2020.parquet
  - Intermediate rounds: outputs/.../intermediate/round_*.json

Would create:
  - Output directory: outputs/.../maps/rounds
  - PNG map for each round of bisection
  - Shows progressive refinement from 1 -> N districts
══════════════════════════════════════════════════════════════════════
```

---

## Key Enhancements

### 1. Tree-Style Formatting ✅
Uses Unicode box-drawing characters for clear hierarchy:
```
  ┌─ STEP 1: Load data
  │   ├─ Load tracts: ...
  │   │     • Details
  │   └─ Load places: ...
  │
  ├─ STEP 2: Process
  │   └─ Sub-task
  │
  └─ STEP 3: Save
      └─ Output files
```

### 2. Visual Separators ✅
Different separator styles for different levels:
- `═` (double line) - Major section headers
- `─` (single line) - Subsection dividers
- `▼` and `▲` - Nested subscript boundaries

### 3. Detailed File Paths ✅
Shows exact files that would be created:
```
Would create:
  • outputs/us_2020_v1/states/california/final_assignments.pkl
  • outputs/us_2020_v1/states/california/district_*.png (52 maps)
```

### 4. Estimated Metrics ✅
Provides planning information:
```
ESTIMATED OUTPUT:
  Files to create: ~15
  Estimated time: 15-30 minutes
  Max deviation target: <1%
```

### 5. Algorithm Details ✅
Shows technical parameters:
```
├─ Algorithm: METIS graph partitioning
├─ Parameters: niter=100, balanced partitions
├─ Depth: 6 rounds
└─ Target: 52 equal-population districts
```

---

## Usage

### Test Individual Script Print-Only
```bash
python scripts/run_state_redistricting.py --state CA --year 2020 --output-dir test --print-only
```

### Test State Pipeline Print-Only
```bash
python scripts/run_all_states.py --year 2020 --version v1 --print-only CA
```

### Test Full Pipeline Print-Only
```bash
python scripts/run_complete_redistricting.py --year 2020 --version v1 --print-only
```

---

## Benefits

### 1. Understanding ✅
- See the entire pipeline structure before execution
- Understand data flow between steps
- Identify all input and output files

### 2. Debugging ✅
- Verify file paths are correct
- Check that all required data exists
- Understand dependencies between steps

### 3. Planning ✅
- Estimate total run time
- Understand computational requirements
- Plan storage needs

### 4. Documentation ✅
- Self-documenting pipeline structure
- Clear indication of what each step does
- Easy to copy-paste commands for manual execution

### 5. Testing ✅
- Dry-run before committing compute resources
- Verify year parameter threading
- Check output directory structure

---

## Example Full Pipeline Print-Only Output

Running:
```bash
python scripts/run_complete_redistricting.py --year 2010 --version v1 --print-only
```

Produces ~500 lines of structured output showing:
1. **Pipeline structure** - All 4 major steps
2. **State breakdown** - What happens for each state
3. **Nested operations** - All 5 steps per state
4. **Individual subscripts** - Detailed view of each operation
5. **File outputs** - Every file that would be created
6. **Estimates** - Total files, time, resources needed

This allows you to review the ENTIRE pipeline execution plan before running a 8-12 hour job.

---

## Technical Details

### Tree Drawing Characters
- `┌` `├` `└` - Branch connectors
- `│` - Vertical line
- `─` - Horizontal line
- `═` - Double horizontal line
- `•` - Bullet points for details
- `▼` `▲` - Section markers

### Color Support
Currently using monochrome output. Could be enhanced with terminal colors using `colorama`:
```python
from colorama import Fore, Style
print(f"{Fore.GREEN}✓ Complete{Style.RESET_ALL}")
print(f"{Fore.RED}✗ Failed{Style.RESET_ALL}")
```

---

**Date**: January 10, 2026
**Status**: All print-only modes enhanced ✅
**Scripts Updated**: 6 (visualize_all_rounds already good)
**Coverage**: 100% of pipeline
