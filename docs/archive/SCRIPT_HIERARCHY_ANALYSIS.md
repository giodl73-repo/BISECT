# Script Hierarchy Analysis

## Current Call Chain

### Parallel Mode
```
run_parallel.bat
  → run_complete_redistricting.py --workers 4
    → process_single_state.py (one per state, 4-8 workers in parallel)
      → run_state_redistricting.py
      → add_cities_to_districts.py
      → create_final_district_summary.py
      → visualize_all_rounds.py
      → create_individual_district_maps.py
```

### Sequential Mode
```
run_sequential.bat
  → run_complete_redistricting.py --workers 1
    → run_state_redistricting.py (one state at a time)
    → add_cities_to_districts.py
    → create_final_district_summary.py
    → visualize_all_rounds.py
    → create_individual_district_maps.py
```

## Script Status

### ✅ ACTIVE - Used in Production

**Entry Points:**
- `run_parallel.bat` - Launch parallel mode
- `run_sequential.bat` - Launch sequential mode
- `run_redistricting.bat` - Launch with custom args
- `CANCEL.bat` - Emergency stop

**Top-Level Orchestrator:**
- `run_complete_redistricting.py` - Main orchestrator (handles both parallel and sequential modes)

**Worker Scripts:**
- `process_single_state.py` - Parallel mode worker (wraps pipeline for one state)
- `run_state_redistricting.py` - Single state full pipeline

**Child Scripts (Core Work):**
- `add_cities_to_districts.py` - Add city labels and create map
- `create_final_district_summary.py` - Generate statistics CSV
- `visualize_all_rounds.py` - Create round-by-round PNGs
- `create_individual_district_maps.py` - Generate per-district PNGs

**Data Preparation:**
- `download_tracts.py` - Download census tract shapefiles
- `download_places.py` - Download city/place shapefiles
- `build_tract_adjacency.py` - Build adjacency graphs

### ✅ CLEANUP COMPLETED

**Removed Scripts:**
- `run_all_states.py` - DELETED (was redundant, replaced by run_all_states_simplified.py)
- `run_all_states_parallel.py` - DELETED (merged into run_complete_redistricting.py)
- `run_all_states_simplified.py` - DELETED (merged into run_complete_redistricting.py)

**Result:** All state orchestration logic is now consolidated into `run_complete_redistricting.py`, eliminating duplication and simplifying maintenance.

## Architecture Improvements

### Consolidation Results

**Before:** 3 orchestrator scripts with ~60% code duplication
- `run_complete_redistricting.py` (213 lines) - dispatcher
- `run_all_states_parallel.py` (313 lines) - parallel mode
- `run_all_states_simplified.py` (293 lines) - sequential mode
- Total: ~819 lines

**After:** 1 unified orchestrator script
- `run_complete_redistricting.py` (593 lines) - handles everything
- Total: 593 lines

**Benefits:**
- Eliminated ~60% code duplication
- Single source of truth for orchestration logic
- Simpler to understand and maintain
- Mode selection via `--workers` flag (1=sequential, 2-8=parallel)
- No subprocess overhead for orchestrator dispatch

### Why We Have process_single_state.py

- **Purpose:** Wrapper for parallel workers
- **Reason:** Each parallel worker needs to run the full pipeline for one state in its own subprocess
- **Keep:** YES - essential for parallel mode (multiprocessing isolation)
- **Note:** Sequential mode calls pipeline scripts directly without this wrapper

## Summary

**Total Scripts:** 15 (excluding data prep and validation)
**Active Scripts:** 15
**Deleted Scripts:** 3 (run_all_states.py, run_all_states_parallel.py, run_all_states_simplified.py)

**Status:** Codebase successfully consolidated - all orchestration logic merged into single entry point.
