# 2010 Census Data Pipeline Status

**Date**: January 10, 2026
**Status**: In Progress - Data Download Phase

---

## Overview

Processing the 2010 census data for congressional redistricting across all 50 states. This will create a complete comparison dataset with the 2020 census results to analyze district boundary changes and population shifts.

---

## Completed Tasks ✅

### 1. Download Script Updates ✅
Updated all download scripts to support multiple census years:

**download_tracts.py**:
- Added year parameter support (2020, 2010, 2000)
- Fixed column name detection for 2010 (GEOID10, NAME10, ALAND10, AWATER10, etc.)
- Updated Census API connection for 2010 data:
  - 2020: `DECENNIALPL2020`, variable `P1_001N`
  - 2010: `DECENNIALSF12010`, variable `P001001`
- Added debug output showing available columns

**download_all_states_tracts.py**:
- Added `--year` parameter via argparse
- Added `--skip` parameter for flexible state selection
- Updated to use year-specific file paths
- Uses absolute paths with `sys.executable` and `Path(__file__).parent`

**download_places.py**:
- Added year parameter support
- Updated Census API endpoints:
  - 2020: `/data/2020/dec/pl` with variable `P1_001N`
  - 2010: `/data/2010/dec/sf1` with variable `P001001`

**download_all_places.py**:
- Added `--year` parameter via argparse
- Added `--skip` parameter for flexible state selection
- Updated to pass year to subprocess calls
- Uses absolute paths for cross-directory execution

### 2. Test Download ✅
Successfully tested 2010 tract download for Delaware:
- Downloaded 218 tracts
- Fetched population: 897,934 total
- Verified 2010 column names: GEOID10, NAME10, ALAND10, AWATER10, INTPTLAT10, INTPTLON10
- Saved to: `data/raw/de_tracts_2010.parquet` (1.6 MB)

---

## In Progress 🔄

### 3. Download 2010 Tracts for All 50 States
**Status**: Currently Running (Background Task ID: b7ffec1)

**Command**:
```bash
python scripts/download_all_states_tracts.py --year 2010 --skip DE DC
```

**Progress**:
- Currently processing: Alabama (state 1/49)
- Alabama: ~76% complete (51/67 counties processed)
- Estimated time: ~2-4 hours for all states
- Output: `data/raw/{state}_tracts_2010.parquet` for each state

**File Format**:
- GeoParquet with snappy compression
- Columns: GEOID, NAME, ALAND, AWATER, INTPTLAT, INTPTLON, population, geometry
- Population data from Census API SF1 2010

**States Being Downloaded**: 49 states (skipping DE already done, DC not needed for redistricting)

---

## Pending Tasks 📋

### 4. Build 2010 Adjacency Graphs
Once tract downloads complete, build adjacency graphs:

**Command**:
```bash
python scripts/build_all_adjacency_graphs.py --year 2010
```

**What it does**:
- Computes Queen contiguity for all tract pairs
- Adds county-aware water-based adjacency for islands
- Connects each island to nearest neighbor (prefers same county)
- Saves to: `data/adjacency/{state}_adjacency_2010.pkl`

**Estimated time**: ~30 minutes for all 50 states

### 5. Download 2010 Places Data
Download city/town boundaries and populations:

**Command**:
```bash
python scripts/download_all_places.py --year 2010
```

**What it does**:
- Downloads place boundaries from Census TIGER 2010
- Fetches population from Census API SF1
- Filters to places with population > 0
- Saves to: `data/raw/{state}_places_2010.parquet`

**Estimated time**: ~1-2 hours for all 50 states

### 6. Run 2010 Redistricting for All States
Execute full redistricting pipeline:

**Command**:
```bash
python scripts/run_complete_redistricting.py --year 2010 --version v1
```

**Or for specific states**:
```bash
python scripts/run_all_states.py --year 2010 --version v1 CA TX NY
```

**What it does**:
- Runs recursive bisection with METIS (niter=100)
- Creates district boundaries for all multi-district states
- Adds city labels via spatial joins
- Generates individual district maps
- Visualizes intermediate rounds
- Organizes output in: `outputs/us_2010_v1/states/{state_name}/`

**Output per state**:
- `{state}_{districts}_districts.png` - Final map
- `districts.csv` - Tract-to-district assignments
- `district_summary.csv` - Population statistics
- `rounds_hierarchy.csv` - Bisection tree
- `intermediate/` - Round-by-round maps
- `district_{N}.png` - Individual district maps

**Estimated time**: ~6-12 hours for all 43 multi-district states (parallel execution possible)

---

## 2010 vs 2020 Key Differences

### District Count Changes
States that gained/lost districts between 2010 and 2020:

**Lost 1 District** (2010→2020):
- California: 53 → 52
- Illinois: 18 → 17
- Michigan: 14 → 13
- New York: 27 → 26
- Ohio: 16 → 15
- Pennsylvania: 18 → 17
- West Virginia: 3 → 2

**Gained 1 District**:
- Colorado: 7 → 8
- Florida: 27 → 28
- Montana: 1 → 2
- North Carolina: 13 → 14
- Oregon: 5 → 6

**Gained 2 Districts**:
- Texas: 36 → 38

### Data Format Differences
- 2010 uses "10" suffix: GEOID10, ALAND10, AWATER10, INTPTLAT10, INTPTLON10
- 2020 uses standard names: GEOID, ALAND, AWATER, INTPTLAT, INTPTLON
- Download scripts automatically detect and normalize column names

### Census API Differences
- 2010: `DECENNIALSF12010` with variable `P001001`
- 2020: `DECENNIALPL2020` with variable `P1_001N`

---

## Data Directory Structure

```
data/
├── raw/
│   ├── {state}_tracts_2010.parquet    ✅ In Progress (1/50)
│   ├── {state}_tracts_2020.parquet    ✅ Complete (50/50)
│   ├── {state}_places_2010.parquet    ⏳ Pending
│   └── {state}_places_2020.parquet    ✅ Complete (50/50)
├── adjacency/
│   ├── {state}_adjacency_2010.pkl     ⏳ Pending
│   └── {state}_adjacency_2020.pkl     ✅ Complete (50/50)

outputs/
├── us_2020_v1/                        ✅ Complete
│   └── states/
│       ├── california/ (52 districts)
│       ├── texas/ (38 districts)
│       └── ... (all 43 multi-district states)
└── us_2010_v1/                        ⏳ Pending
    └── states/
        ├── california/ (53 districts)
        ├── texas/ (36 districts)
        └── ... (all 43 multi-district states)
```

---

## Timeline Estimate

| Phase | Status | Est. Time | ETA |
|-------|--------|-----------|-----|
| Download 2010 tracts | In Progress | 2-4 hours | 2-4 hours from now |
| Build adjacency graphs | Pending | 30 min | +30 min after tracts |
| Download 2010 places | Pending | 1-2 hours | +2 hours after adjacency |
| Run redistricting | Pending | 6-12 hours | +12 hours after places |
| **Total** | **In Progress** | **10-18 hours** | **~24 hours from start** |

*Note: Redistricting can be parallelized by running multiple states simultaneously*

---

## Next Steps

1. **Monitor tract downloads** (currently running in background)
   - Check progress: `tail -f C:\Users\giodl\AppData\Local\Temp\claude\C--src-apportionment\tasks\b7ffec1.output`
   - When complete, verify: `ls data/raw/*_tracts_2010.parquet | wc -l` should show 50

2. **Build adjacency graphs** once tracts are downloaded
   ```bash
   python scripts/build_all_adjacency_graphs.py --year 2010
   ```

3. **Download places data** in parallel while adjacency graphs build
   ```bash
   python scripts/download_all_places.py --year 2010
   ```

4. **Run redistricting** for all states once data is ready
   ```bash
   python scripts/run_complete_redistricting.py --year 2010 --version v1
   ```

5. **Compare results** between 2010 and 2020
   - Analyze boundary changes
   - Compare population distributions
   - Identify demographic shifts

---

## Commands Quick Reference

### Check download progress
```bash
# Count completed tract files
ls data/raw/*_tracts_2010.parquet | wc -l

# View download progress
tail -50 C:\Users\giodl\AppData\Local\Temp\claude\C--src-apportionment\tasks\b7ffec1.output
```

### Test individual state
```bash
# Download tracts for one state
python scripts/download_tracts.py --state CA --year 2010

# Build adjacency for one state
python scripts/build_tract_adjacency.py --state CA --year 2010

# Download places for one state
python scripts/download_places.py --state CA --year 2010

# Run redistricting for one state
python scripts/run_state_redistricting.py --state CA --year 2010 --output-dir test_ca_2010
```

### Run full pipeline (when data is ready)
```bash
# Full automated pipeline
python scripts/run_complete_redistricting.py --year 2010 --version v1

# Or step-by-step with print-only first
python scripts/run_complete_redistricting.py --year 2010 --version v1 --print-only
python scripts/run_complete_redistricting.py --year 2010 --version v1
```

---

**Last Updated**: January 10, 2026
**Current Task**: Downloading 2010 Census tracts (1/49 states in progress)
**Next Milestone**: Complete tract downloads, begin adjacency graph building
