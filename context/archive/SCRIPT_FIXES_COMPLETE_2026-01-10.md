# Script Fixes Complete - January 10, 2026

## All Fixes Completed ✅

### 1. Configuration Files ✅
- **Created** `config_2020.py` with 2020 census apportionment (CA=52, TX=38, NY=26, etc.)
- **Existing** `config_2010.py` with 2010 census apportionment (CA=53, TX=36, NY=27, etc.)
- Scripts dynamically load configuration based on `--year` parameter
- Total districts verified: 435 for both years

### 2. Year Parameter Threading ✅
**All scripts now support `--year 2020|2010|2000`**:
- `run_complete_redistricting.py` ✅
- `run_all_states.py` ✅
- `run_state_redistricting.py` ✅
- `add_cities_to_districts.py` ✅
- `create_final_district_summary.py` ✅
- `create_individual_district_maps.py` ✅
- `visualize_all_rounds.py` ✅
- `build_tract_adjacency.py` ✅
- `build_all_adjacency_graphs.py` ✅

### 3. Print-Only Debug Mode ✅
**Full hierarchy support for dry-run testing**:
- Master scripts execute subscripts WITH `--print-only` flag
- Each script shows what it would do without executing
- Commands displayed with clear separators for easy copy-paste
- METIS and heavy operations skipped in print-only mode
- Prerequisites checks skipped in print-only mode

**Command Display Format**:
```
  [1/5] Running redistricting
      Command: C:\...\python.exe C:\...\run_state_redistricting.py --state CA --year 2010 ...
      ------------------------------------------------------------
      [PRINT-ONLY] California Redistricting - 2010 Census
      Would load files:
        - Adjacency graph: data/adjacency/ca_adjacency_2010.pkl
        - Tracts: data/raw/ca_tracts_2010.parquet
      Would run:
        - Recursive bisection: 53 districts
        - METIS graph partitioning (niter=100)
      Would create:
        - Output directory: outputs/california_full_TIMESTAMP
        - districts.csv, district_summary.csv, rounds_hierarchy.csv
        - california_districts_map.png
        - intermediate/ (round-by-round maps)
      ------------------------------------------------------------
```

### 4. Absolute Path Resolution ✅
**Fixed path resolution issues**:
- Uses `Path(__file__).parent` for script directory resolution
- Uses `sys.executable` for Python executable path
- No more "scripts/scripts/..." path doubling errors
- Works correctly from any directory

### 5. District Count Configuration ✅
**Dynamic loading based on census year**:
- 2020: CA=52, TX=38, FL=28, NY=26, PA=17, IL=17
- 2010: CA=53, TX=36, FL=27, NY=27, PA=18, IL=18
- Differences properly reflected in state configurations

### 6. Adjacency Graph Year-Specific Naming ✅
**Fixed adjacency graph file paths to support multiple census years**:

**Before**:
- Path: `data/processed/{state}_tracts_graph.pkl`
- Problem: Single filename can't support multiple census tract boundaries

**After**:
- Path: `data/adjacency/{state}_adjacency_{year}.pkl`
- Examples: `ca_adjacency_2020.pkl`, `ca_adjacency_2010.pkl`

**Changes made**:
1. `build_tract_adjacency.py`:
   - Added `year` parameter to function signature
   - Changed default output_dir from `'data/processed'` to `'data/adjacency'`
   - Updated output filename to include year: `f"{state}_adjacency_{year}.pkl"`
   - Pass year parameter through from argparse to function

2. `build_all_adjacency_graphs.py`:
   - Added `--year` parameter support via argparse
   - Updated `check_existing()` to use new path with year
   - Updated `check_tracts_exist()` to use year parameter
   - Updated `build_adjacency_graph()` to pass `--year` to subprocess
   - Used absolute paths with `sys.executable` and `Path(__file__).parent`

3. `run_state_redistricting.py`:
   - Changed graph_file path to: `f'data/adjacency/{state_code.lower()}_adjacency_{year}.pkl'`
   - Changed tracts_file path to: `f'data/raw/{state_code.lower()}_tracts_{year}.parquet'`

4. `run_all_states.py`:
   - Updated `check_prerequisites()` to use year-specific adjacency path:
     `graph_file = Path(f'data/adjacency/{state_code_lower}_adjacency_{year}.pkl')`

### 7. Migration of Existing 2020 Graphs ✅
**Preserved existing adjacency graphs**:
- Migrated all 50 state adjacency graphs from `data/processed/` to `data/adjacency/`
- Renamed from `{state}_tracts_graph.pkl` to `{state}_adjacency_2020.pkl`
- Saved hours of rebuild time for 2020 graphs

**Migration command used**:
```python
python -c "from pathlib import Path; src = Path('data/processed'); dst = Path('data/adjacency'); dst.mkdir(exist_ok=True); [Path(dst / f'{f.stem.split('_tracts_graph')[0]}_adjacency_2020.pkl').write_bytes(f.read_bytes()) for f in src.glob('*_tracts_graph.pkl')]; print(len(list(dst.glob('*.pkl'))))"
```

Result: 50 files successfully migrated

### 8. Hardcoded References Fixed ✅
**Removed all hardcoded 2020 references**:
- `check_prerequisites()` now uses year parameter
- Titles changed from "2020 Census" to dynamic `{args.year} Census`
- File paths use year variable instead of hardcoded "2020"
- Print statements reference `args.year` instead of constant

## Test Commands

### Print-Only Mode (Full Hierarchy)
```bash
# Full pipeline for 2010
python scripts/run_complete_redistricting.py --year 2010 --version v1 --print-only

# Single state for 2010
python scripts/run_all_states.py --year 2010 --version v1 --print-only CA

# Individual script for 2010
python scripts/run_state_redistricting.py --state CA --year 2010 --output-dir test --print-only

# Build adjacency graphs for 2010
python scripts/build_all_adjacency_graphs.py --year 2010
```

### Verify District Counts
```bash
python scripts/run_all_states.py --year 2010 --version v1 --print-only CA TX NY
# Should show: CA=53, TX=36, NY=27

python scripts/run_all_states.py --year 2020 --version v1 --print-only CA TX NY
# Should show: CA=52, TX=38, NY=26
```

### Verify Adjacency Graph Paths
```bash
# Check existing 2020 graphs
ls data/adjacency/*_adjacency_2020.pkl | wc -l
# Should show: 50

# Test building a 2010 graph (when data is ready)
python scripts/build_tract_adjacency.py --state CA --year 2010
```

## Files Modified

### Master Scripts
- `run_complete_redistricting.py` - Orchestrator with print-only and year threading
- `run_all_states.py` - State processor with dynamic config loading

### Core Scripts
- `run_state_redistricting.py` - METIS redistricting with year-specific paths
- `add_cities_to_districts.py` - City labeling with year support
- `create_final_district_summary.py` - Statistics with year parameter
- `create_individual_district_maps.py` - District maps with year parameter
- `visualize_all_rounds.py` - Round visualizations with year parameter

### Data Processing Scripts
- `build_tract_adjacency.py` - Adjacency graph builder with year-specific naming
- `build_all_adjacency_graphs.py` - Batch builder with year support

### Configuration
- `config_2020.py` - 2020 apportionment (52 entries × districts, total 435)
- `config_2010.py` - 2010 apportionment (52 entries × districts, total 435)

## Directory Structure

### Data Organization
```
data/
├── raw/
│   ├── {state}_tracts_2020.parquet    # 2020 Census tract geometries
│   ├── {state}_tracts_2010.parquet    # 2010 Census tract geometries
│   ├── {state}_places_2020.parquet    # 2020 Census places (cities)
│   └── {state}_places_2010.parquet    # 2010 Census places (cities)
├── adjacency/
│   ├── {state}_adjacency_2020.pkl     # 2020 tract adjacency graphs
│   └── {state}_adjacency_2010.pkl     # 2010 tract adjacency graphs
└── processed/                          # Legacy directory (can be cleaned up)
    └── {state}_tracts_graph.pkl       # Old format (deprecated)

outputs/
├── us_2020_v1/                        # 2020 census results
│   └── states/
│       ├── california/
│       ├── texas/
│       └── ...
└── us_2010_v1/                        # 2010 census results (to be created)
    └── states/
        ├── california/
        ├── texas/
        └── ...
```

## Next Steps

### Ready for 2010 Data Pipeline:

1. **Download 2010 tract shapefiles** from Census TIGER/Line
   - URL: https://www2.census.gov/geo/tiger/TIGER2010/TRACT/2010/
   - Format: Shapefile with GEOID10 field
   - Convert to parquet: `{state}_tracts_2010.parquet`

2. **Create 2010 adjacency graphs**
   ```bash
   python scripts/build_all_adjacency_graphs.py --year 2010
   ```
   - Uses county-aware water connections
   - Saves to `data/adjacency/{state}_adjacency_2010.pkl`

3. **Download 2010 places (cities) data**
   - URL: https://www2.census.gov/geo/tiger/TIGER2010/PLACE/2010/
   - Convert to parquet: `{state}_places_2010.parquet`

4. **Run full 2010 pipeline**
   ```bash
   # Test with print-only first
   python scripts/run_complete_redistricting.py --year 2010 --version v1 --print-only

   # Execute full run
   python scripts/run_complete_redistricting.py --year 2010 --version v1
   ```

5. **Compare 2010 vs 2020 results**
   - Analyze district boundary changes
   - Compare population distributions
   - Identify gained/lost districts by state

## Summary Statistics

### Script Updates
- **9 scripts** updated with year parameter support
- **9 scripts** updated with print-only mode
- **2 configuration files** created/verified
- **50 adjacency graphs** migrated to new structure
- **100%** of hardcoded 2020 references removed

### Testing Coverage
- Print-only mode tested across full pipeline ✅
- Year parameter threading verified ✅
- Path resolution tested from multiple directories ✅
- District count accuracy verified for 2010 and 2020 ✅
- Adjacency graph naming verified ✅

### Code Quality
- Absolute path resolution prevents directory issues ✅
- Dynamic configuration loading supports multiple years ✅
- Print-only mode enables safe testing without execution ✅
- Year-specific file paths support multiple census datasets ✅
- Backward compatibility maintained for existing 2020 data ✅

---

**Date**: January 10, 2026
**Status**: All script fixes complete ✅
**Ready for**: 2010 census data pipeline
**Blockers**: None - all prerequisites satisfied
