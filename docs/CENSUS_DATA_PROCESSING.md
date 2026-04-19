# Census Data Processing

## Overview

The census data processing pipeline transforms raw census data into pipeline-ready GeoParquet files.

**Raw data location**: `data/{year}/`
**Processed data location**: `outputs/data/`

## Data Organization

```
data/{year}/
  redistricting/         <- PL 94-171 redistricting files
    {state}{year}.pl/    <- State-specific PL files (2010, 2020)
    {state}geo.upl       <- State geographic files (2000)
  tiger/                 <- TIGER/Line tract shapefiles
    tl_{year}_{fips}_tract/
  demographics/          <- Demographic data
  elections/             <- Election data
  tiger_cousub/          <- County subdivision shapefiles

outputs/data/{year}/
  tracts/                <- Processed tract GeoParquet files
    {state}_tracts_{year}.parquet
  adjacency/             <- Adjacency graphs
    {state}_adjacency_{year}.pkl
  places/                <- Place labels for maps
    {state}_places_{year}.parquet
  elections/             <- Processed election data for this census year
    {year}_president_tract.parquet
  demographics/          <- Processed demographic data for this census year
    {year}_demographics_tract.parquet
```

## Processing Pipeline

### Step 1: Parse PL 94-171 Files
Extracts population and geographic data from Census Bureau redistricting files.

**Scripts**:
- `scripts/data/census/parse_pl94171_tracts_2020.py` - Parse 2020 pipe-delimited files
- `scripts/data/census/parse_pl94171_tracts_2010.py` - Parse 2010 fixed-width files
- `scripts/data/census/parse_pl94171_tracts_2000.py` - Parse 2000 .upl files

**Input**: `data/{year}/redistricting/`
**Output**: CSV files with population data (temp, not used by pipeline)

**Status**: ✅ Complete

### Step 2: Download TIGER/Line Shapefiles
Downloads tract boundary shapefiles from Census Bureau.

**Script**: `scripts/data/geography/download_tiger_units.py`

**Output**: `data/{year}/tiger/`

**Status**:
- ✅ 2020: Already downloaded
- ❌ 2010: Need to download
- ❌ 2000: Need to download

**Command**:
```bash
python scripts/data/geography/download_tiger_units.py --year 2020
python scripts/data/geography/download_tiger_units.py --year 2010
python scripts/data/geography/download_tiger_units.py --year 2000
```

### Step 3: Merge Population + Geometry
Combines population data with tract geometries to create GeoParquet files.

**Script**: `scripts/data/merge_units_with_geometries.py` (unified for all years)

**Input**:
- CSV from Step 1
- TIGER/Line shapefiles from Step 2

**Output**: `outputs/data/{year}/units/{state}_tracts_{year}.parquet`

**Status**: ✅ Complete (integrated in pipeline with parallel processing)

### Step 4: Build Adjacency Graphs
Creates tract adjacency networks for redistricting algorithm.

**Script**: `scripts/data/geography/build_all_adjacency_graphs.py`

**Input**: `outputs/data/{year}/units/*.parquet`
**Output**: `outputs/data/{year}/adjacency/*.pkl`

**Status**: ✅ Exists (already integrated in pipeline)

## Current Status

### Completed (2026-01-18)
- ✅ Created PL 94-171 parsing scripts for all 3 census years
- ✅ Added parallel state processing (12 workers default)
- ✅ Integrated STATUS progress protocol
- ✅ Reorganized data structure (`redistricting/` subdirectory)
- ✅ Created TIGER/Line download script
- ✅ Created validation script
- ✅ Created unified merge script (all years)
- ✅ Integrated merge into pipeline with parallel processing

### Remaining Work
1. **Download TIGER/Line shapefiles for 2010 and 2000**
   - Run download script for each year
   - ~5GB per year
   - Command: `python scripts/data/geography/download_tiger_units.py --year {year}`

2. **Test full pipeline**
   - Parse → Merge → Adjacency → Validate
   - Test Vermont: `run -y 2020 -v test -s data -st VT`
   - Verify all 50 states for all 3 years

## Validation

Check that all required files exist and have correct format:

```bash
python scripts/data/validate_census_data.py --year 2020
python scripts/data/validate_census_data.py --year 2020 --states VT --verbose
```

Expected output:
```
[OK] Tracts      : 50/50 states valid
[OK] Adjacency   : 50/50 states valid
```

## Commands

### Full Pipeline (when complete)
```bash
# Process all data for a census year
python scripts/data/process_census_data.py --year 2020

# Process specific states only
python scripts/data/process_census_data.py --year 2020 --states VT DE RI

# Use more workers for faster processing
python scripts/data/process_census_data.py --year 2020 --workers 24

# Process specific stages
python scripts/data/process_census_data.py --year 2020 --stages tracts merge adjacency
```

### Individual Steps
```bash
# Parse PL 94-171 files only
python scripts/data/census/parse_pl94171_tracts_2020.py --states VT

# Download TIGER/Line shapefiles
python scripts/data/geography/download_tiger_units.py --year 2020 --states VT

# Merge into GeoParquet
python scripts/data/merge_units_with_geometries.py --year 2020 --states VT

# Build adjacency graphs
python scripts/data/geography/build_all_adjacency_graphs.py --year 2020 --states VT
```

## Performance

**Parallel Processing** (12 workers):
- Parse PL 94-171: ~28 seconds (50 states)
- Download TIGER/Line: ~5-10 minutes (50 states, network dependent)
- Merge: TBD
- Build adjacency: ~10-15 minutes (50 states)

**Total**: ~20-30 minutes per census year (when fully implemented)

## File Formats

### Input Formats
- **PL 94-171 (2020)**: Pipe-delimited text files
- **PL 94-171 (2010)**: Fixed-width text files
- **PL 94-171 (2000)**: .upl format text files
- **TIGER/Line**: Shapefiles (.shp, .shx, .dbf, .prj)

### Output Formats
- **Tracts**: GeoParquet with geometry column (EPSG:4269)
- **Adjacency**: Pickle files with graph structure

### Required Columns in Tract Files
- `GEOID` (str): 11-digit tract identifier
- `NAME` (str): Tract name
- `population` (int): Total population
- `AREALAND` (int): Land area (sq meters)
- `AREAWATR` (int): Water area (sq meters)
- `INTPTLAT` (float): Latitude
- `INTPTLON` (float): Longitude
- `geometry` (geometry): Tract polygon boundaries
- `total_area` (int): Total area
- `water_fraction` (float): Water percentage

## Notes

- **2020 TIGER/Line**: Already downloaded to `data/2020/tiger/`
- **2010 TIGER/Line**: Uses `tl_2010_{fips}_tract10` naming
- **2000 TIGER/Line**: Uses 2010 TIGER/Line with 2000 boundaries (`tl_2010_{fips}_tract00`)
- **CRS**: All tract files use EPSG:4269 (NAD83)
- **Parallel processing**: Each state processed independently for speed
- **Incremental processing**: Per-stage markers prevent reprocessing completed states
