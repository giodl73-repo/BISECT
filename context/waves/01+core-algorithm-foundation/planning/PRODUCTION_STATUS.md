# Production Run Status

**Date**: January 10, 2026
**Status**: ✅ 2020 COMPLETE - 2010 Ready
**Configuration**: niter=100, county-aware water adjacency, city labeling, 4-level progress bars

## 2020 Census Run (COMPLETE)

### Progress: 50/50 states complete (100%)

**Output Directory**: `outputs/us_2020_v1/`

**All 50 States**: Successfully processed
- 44 multi-district states: Full pipeline (redistricting, cities, maps, summary)
- 6 single-district states: At-large designation
- Typical deviation: 0.3-0.5% max
- All visualizations and hierarchies generated

## 2010 Census Run (READY)

### Data Preparation: 100% Complete

**Data Status**:
- ✅ Tracts: 50/50 states downloaded
- ✅ Places: 50/50 states downloaded
- ✅ Adjacency graphs: 50/50 states built (all in data/adjacency/)
- ✅ Configuration: config_2010.py ready

**User will run manually**:
```bash
cd apportionment
python scripts\run_all_states.py --year 2010 --version v1
```

## Recent Improvements

### 1. ✅ 4-Level Progress Bar System

Implemented nested progress bars with file existence indicators:
- **Position 0**: USA-level progress (50 states)
- **Position 1**: State-level progress (5 steps per state)
- **Position 2**: Operation-specific progress (METIS splits, map generation)
- **Position 3**: Color-coded file indicators (green=exists, red=missing)

**Features**:
- Dynamic file display (appears/disappears with operations)
- Clean output (removed all disruptive print statements)
- Accurate path resolution using project root
- Real-time progress tracking with time estimates

### 2. ✅ Integrated Rounds Hierarchy

Refactored rounds hierarchy creation into the main pipeline:
- State-level: Created automatically during Summary stage
- National-level: US aggregate combines all states
- No separate post-processing script needed
- Tracks bisection tree structure round-by-round

**Files Created**:
- `{state_dir}/rounds_hierarchy.csv` - Per-state bisection tree
- `us_rounds_hierarchy.csv` - National aggregate

### 3. ✅ Reordered Pipeline Stages

Optimized pipeline order for logical flow:
1. **Redistricting**: Run METIS recursive bifurcation
2. **Cities**: Add city labels to districts
3. **Round maps**: Visualize bisection rounds
4. **District maps**: Generate individual district PNGs
5. **Summary**: Create statistics and rounds hierarchy

### 4. ✅ Fixed Adjacency Directory

Standardized adjacency graph storage:
- All scripts now use `data/adjacency/` (not `data/processed/`)
- Moved all existing 2010 graphs to correct location
- Future builds automatically go to right place

### 5. ✅ 2010 Census Support

Complete data preparation for 2010 census:
- Downloaded all 50 states tracts
- Downloaded all 50 states places
- Built all 50 states adjacency graphs
- Created config_2010.py with district apportionment
- Ready for full production run

## Output Directory Structure

```
outputs/us_2020_v1/
  states/                          ← All individual states
    california/
      california_52_districts.png
      california_52_districts_with_cities.png
      district_summary.csv
      district_cities.csv
      rounds_hierarchy.csv         ← New: Bisection tree tracking
      intermediate/                ← Round-by-round data
        round_*_metadata.json
        round_*_assignments.json
      maps/
        districts/                 ← Individual district PNGs
          district_01.png
          district_02.png
          ...
        round_1_2_regions.png      ← Round-by-round visualizations
        round_2_4_regions.png
        ...
    texas/
    florida/
    ...
  us_rounds_hierarchy.csv          ← New: National aggregate
```

## Technical Configuration

### METIS Settings
- **niter**: 100 (10x default for maximum refinement)
- **ufactor**: Dynamic by recursion depth
  - Depth 1 (2 regions): 1.001 (0.1% tolerance)
  - Depth 2 (4 regions): 1.002 (0.2% tolerance)
  - Depth 3 (8 regions): 1.003 (0.3% tolerance)
  - Depth 4+ (16+ regions): 1.005 (0.5% tolerance)

### County-Aware Water Adjacency
Already implemented in `build_tract_adjacency.py`:
- Island tracts prefer same-county connections
- Uses GEOID substring matching (chars 2-4)
- Tested on New York: 93 same-county, 0 cross-county connections

### Key Files Modified

**`scripts/run_all_states.py`**:
- Implemented 4-level progress bar system
- Added colorize_files() for existence indicators
- Dynamic file display creation/destruction
- Integrated US rounds hierarchy aggregation
- Reordered pipeline stages

**`scripts/create_final_district_summary.py`**:
- Added create_rounds_hierarchy() function
- Automatically called during Summary stage
- Processes intermediate JSON metadata

**`src/apportionment/partition/recursive_bisection.py`**:
- Removed verbose print statements
- Clean output for progress bar display

**`scripts/visualize_all_rounds.py`**:
- Removed round info print statements
- Clean progress bar integration

**`scripts/build_tract_adjacency.py`**:
- Fixed default output directory to data/adjacency/

## Processing Pipeline

Each state goes through 5 steps:

1. **[1/5] Redistricting** (~60-70% of time)
   - Load tract adjacency graph
   - Run recursive bisection with METIS (niter=100)
   - Save intermediate results at each round
   - Create final district assignments
   - Generate final map

2. **[2/5] Cities** (~10% of time)
   - Load places data
   - Spatial join to identify cities in each district
   - Create district_cities.csv
   - Generate map with city labels

3. **[3/5] Round Maps** (~10% of time)
   - Create maps showing each round of bisection
   - Visualize progressive splitting (2, 4, 8, 16... regions)
   - Saves to maps/round_*.png

4. **[4/5] District Maps** (~15% of time)
   - Generate individual PNG for each district
   - High-resolution output with labels
   - Saves to maps/districts/

5. **[5/5] Summary** (~5% of time)
   - Calculate population statistics
   - Compute deviations from ideal
   - Create district_summary.csv
   - Create rounds_hierarchy.csv (bisection tree tracking)

## Performance Metrics

**Per-State Times** (with niter=100):
- California (52 districts): ~1-2 minutes
- Texas (38 districts): ~1-2 minutes
- Florida (28 districts): ~1 minute
- Medium states (8-15 districts): ~30-60 seconds
- Small states (2-8 districts): ~20-40 seconds

**Total Estimated Time**: 2-3 hours for all 44 multi-district states

**Quality Achieved**:
- All states: < 0.5% max deviation
- Best: Florida 0.22%, Ohio 0.20%
- Typical: 0.3-0.4% max deviation

## Next Steps

### After Current Run Completes

1. **Create US Aggregate Files**
   ```bash
   python scripts/create_us_aggregate.py
   ```
   - Combines all 44 states + 6 single-district states
   - Creates `us_all_districts.csv` with all 435 districts
   - Creates `us_district_summary.csv` with full statistics
   - Generates comprehensive markdown report

2. **Create National Map**
   ```bash
   python scripts/create_us_national_map.py
   ```
   - US map showing all 435 districts
   - Version with major city labels
   - High-resolution output

3. **Calculate Compactness Metrics** (Optional)
   ```bash
   # Run for all states
   for state_dir in outputs/us_2020_v2/states/*/; do
       python scripts/calculate_compactness_metrics.py "$state_dir"
   done
   ```
   - Adds Polsby-Popper scores to district_summary.csv
   - Also calculates Reock and Convex Hull ratios

4. **Create Rounds Hierarchy**
   ```bash
   python scripts/create_rounds_hierarchy.py
   ```
   - Extracts bisection tree structure
   - Shows how districts were created round-by-round

## Scripts Ready for Use

### Core Redistricting
- ✅ `run_all_states.py` - Main orchestrator with progress bar
- ✅ `run_state_redistricting.py` - Individual state processing
- ✅ `build_tract_adjacency.py` - County-aware adjacency

### Post-Processing
- ✅ `add_cities_to_districts.py` - City labeling
- ✅ `create_final_district_summary.py` - Statistics
- ✅ `create_individual_district_maps.py` - District visualizations
- ✅ `visualize_all_rounds.py` - Round-by-round viz

### Aggregation
- ✅ `create_us_aggregate.py` - Combine all states
- ✅ `create_us_national_map.py` - National visualization
- ✅ `create_single_district_states.py` - Handle 1-district states
- ✅ `create_rounds_hierarchy.py` - Extract bisection tree

### Analysis
- ✅ `calculate_compactness_metrics.py` - Polsby-Popper scores

## Algorithm Details

**Recursive Bisection**:
- Splits regions repeatedly until target number of districts reached
- Uses METIS graph partitioning at each step
- Enforces contiguity with `-contig` flag
- Balances population with dynamic ufactor

**METIS Command Example**:
```bash
gpmetis.exe -contig -minconn -ufactor=1.003 -niter=100 \
    -tpwgts=tpwgts.txt graph.txt 2
```

**Key Parameters**:
- `-contig`: Enforce contiguous partitions
- `-minconn`: Minimize edge cuts
- `-ufactor`: Load imbalance tolerance
- `-niter`: Number of refinement iterations
- `-tpwgts`: Target weights for each partition

## Known Issues & Solutions

### ✅ FIXED: niter Parameter Not Working
**Problem**: First tests weren't actually using custom niter values
**Solution**: Modified `metis_executable.py` and `metis_wrapper.py` to properly pass `-niter=` flag to gpmetis.exe

### ✅ FIXED: Unicode Encoding Errors
**Problem**: Checkmark characters (✓) causing Windows encoding errors
**Solution**: Changed to ASCII `[OK]` messages

### ✅ FIXED: Progress Bar Not Visible
**Problem**: Running in background hides live progress bar
**Solution**: User runs script in foreground terminal to see live updates

### ⚠️ Directory Structure Migration
**Note**: Old timestamped directories exist but are obsolete
- `outputs/us_2020_20260109_*` - Old format
- `outputs/us_2020_v2/` - Current format (use this)

## File Locations

**Project Root**: `apportionment\`

**Scripts**: `apportionment\scripts\`

**Data**:
- Raw: `apportionment\data\raw\` (tracts, places)
- Processed: `apportionment\data\processed\` (adjacency graphs)

**Outputs**: `apportionment\outputs\us_2020_v2\`

**Documentation**:
- `PRODUCTION_STATUS.md` - This file
- `PRODUCTION_RUN_READY.md` - Original planning doc
- `COMPACTNESS_IMPROVEMENTS.md` - Improvement strategies
- `IMPROVEMENTS_TODO.md` - Future enhancements
- `README.md` - Project overview
- `claude_session_notes.md` - Development history

## Commands Reference

### Run Full Production
```bash
cd apportionment
python scripts\run_all_states.py --version v2
```

### Run Specific States
```bash
python scripts\run_all_states.py --version v2 CA TX FL
```

### Resume Existing Run
```bash
python scripts\run_all_states.py --output-dir outputs/us_2020_v2
```

### Create Aggregates
```bash
python scripts\create_us_aggregate.py
python scripts\create_us_national_map.py
```

### Check State Completion
```bash
ls outputs/us_2020_v2/states/
```

### Count Completed States
```bash
ls -d outputs/us_2020_v2/states/* | wc -l
```

## Contact & Session Info

**Session Date**: January 9, 2026
**Model**: Claude Sonnet 4.5
**Configuration**: niter=100, dynamic ufactor, county-aware water adjacency

**Key Decisions**:
1. Use niter=100 (not 20 or 50) for maximum quality
2. Version-based naming (v1, v2, ...) instead of timestamps
3. Progress bar with full subprocess output visibility
4. States subdirectory for organization
5. Skip existing states to enable resume capability

---

*Last Updated: 2026-01-09 21:30 - User running v2 production in terminal*
