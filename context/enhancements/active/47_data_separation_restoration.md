# Enhancement 47: Census Data Processing and Path Reorganization

**Status**: ✅ COMPLETED
**Priority**: Critical
**Completed**: 2026-01-18
**Created**: 2026-01-18
**Commits**: [0972568](https://github.com/giodl_microsoft/redistricting/commit/0972568eeba402ec80c92e76af371342207cf5f6), [da418c7](https://github.com/giodl_microsoft/redistricting/commit/da418c7c0b0329276c6c26eb1ae4c5e3a7d14ad8)
**Size**: M - 792 lines changed (5 files)

## Priority Levels

- **Critical**: Blocker for running V2 and all future pipeline runs

## Current State

### Data Loss Incident
- Complete `data/` directory missing (census tracts, adjacency graphs, processed data)
- V2 pipeline failing with "Adjacency graph not found" errors
- User has restored data to:
  - **`data/NHGIS/`** - Manual NHGIS downloads for 2000:
    - `nhgis0001_shapefile_tl2000_us_tract_2000/` - National tract geometries (252MB .shp)
    - `nhgis0004_shapefile_tl2000_us_place_2000/` - National places geometries
    - `nhgis0006_csv/` - Tract/place population data (FL5001 = Total population)
    - `nhgis0003_shapefile_tl2010_us_cd108th_2000/` - 2000 congressional districts (for baseline)
  - **`data/Census 2000/geos/`** - Geographic header files (`.upl` format, 50 states)
  - **`data/Census 2010/`** - PL 94-171 files (`.pl`, `.plgeo`) for all 50 states
  - **`data/Census 2020/`** - PL 94-171 files (`.pl`) for all 50 states

### Current Architecture Issues
- **Mixed responsibilities**: `data/` contains both raw census data (~40GB, unchanging) and processed data (adjacency graphs, parquet files - regeneratable)
- **No separation**: Makes it unclear what's source material vs. derived artifacts
- **Path confusion**: ~100+ scripts hardcode `data/tracts/{year}/`, `data/adjacency/{year}/` paths
- **Git issues**: `data/` in `.gitignore` but lost recently, unclear what happened

## Goal

**Separate raw census data from processed artifacts** with clear responsibilities:

1. **`data/Census {year}/`** - Raw census data (user-provided, never modified):
   - PL 94-171 redistricting files (`.pl`, population data)
   - NHGIS shapefiles (manually downloaded as needed)
   - NHGIS places files (manually downloaded as needed)

2. **`outputs/data/`** - ALL processed/derived data (regeneratable):
   - Tract parquet files (geometries + population)
   - Places parquet files (city points)
   - Adjacency graphs (NetworkX pkl files)
   - Election data (processed parquet)
   - Demographic data (processed parquet)

3. **Scripts updated** to:
   - Read from `data/Census {year}/`
   - Write to `outputs/data/`
   - Read processed data from `outputs/data/`

4. **Pipeline integration** - Add data building as first stage in `run_complete_redistricting.py`

## 📂 Complete Directory Schema

### Input Data (Raw, User-Provided/Downloaded)

```
data/
  ├─ README.md                           # Overall documentation
  │
  ├─ Census 2000/                        # All 2000 Census data
  │   ├─ README.md                       # 2000-specific docs (source: NHGIS)
  │   ├─ tracts/                         # Official: NHGIS tract shapefiles
  │   │   ├─ US_tract_2000.shp          # National tract geometries (252MB)
  │   │   ├─ US_tract_2000.dbf
  │   │   ├─ US_tract_2000.shx
  │   │   └─ US_tract_2000.prj
  │   ├─ places/                         # Official: NHGIS place shapefiles
  │   │   ├─ US_place_2000.shp          # National places geometries
  │   │   ├─ US_place_2000.dbf
  │   │   ├─ US_place_2000.shx
  │   │   └─ US_place_2000.prj
  │   ├─ population/                     # Official: NHGIS population CSVs
  │   │   ├─ nhgis0006_ds146_2000_tract.csv      # Tract population (FL5001)
  │   │   ├─ nhgis0006_ds146_2000_tract_codebook.txt
  │   │   ├─ nhgis0006_ds146_2000_place.csv      # Place population
  │   │   └─ nhgis0006_ds146_2000_place_codebook.txt
  │   ├─ baseline/                       # Official: 2000 congressional districts
  │   │   ├─ US_cd108th_2000_tl10.shp   # For baseline comparison
  │   │   ├─ US_cd108th_2000_tl10.dbf
  │   │   ├─ US_cd108th_2000_tl10.shx
  │   │   └─ US_cd108th_2000_tl10.prj
  │   ├─ demographics/                   # Downloaded: Census API (age/race/gender)
  │   │   └─ [raw demographic CSV downloads]
  │   ├─ elections/                      # Elections: N/A for 2000
  │   │   └─ [empty - no tract-level election data for 2000]
  │   └─ geos/                           # Legacy: .upl files (not used)
  │       └─ {state}geo.upl
  │
  ├─ Census 2010/                        # All 2010 Census data
  │   ├─ README.md                       # 2010-specific docs (source: PL 94-171)
  │   ├─ {state}2010.pl                  # Official: PL 94-171 population (50 files)
  │   ├─ geos/                           # Official: PL 94-171 geography
  │   │   └─ {state}geo2010.pl          # Geography files (50 files)
  │   ├─ demographics/                   # Downloaded: Census API
  │   │   └─ [raw demographic CSV downloads]
  │   ├─ elections/                      # Elections: N/A for 2010
  │   │   └─ [empty - no tract-level election data for 2010]
  │   └─ tiger/                          # Downloaded: TIGER/Line (temp, ~3GB)
  │       └─ tl_2010_{fips}_tract10/    # Per-state shapefiles
  │           └─ tl_2010_{fips}_tract10.shp
  │
  └─ Census 2020/                        # All 2020 Census data
      ├─ README.md                       # 2020-specific docs (source: PL 94-171)
      ├─ {state}2020.pl                  # Official: PL 94-171 population (50 files)
      ├─ demographics/                   # Downloaded: Census API
      │   └─ [raw demographic CSV downloads]
      ├─ elections/                      # Downloaded: MIT Election Lab
      │   └─ [raw presidential election CSV]
      └─ tiger/                          # Downloaded: TIGER/Line via cenpy (temp)
          └─ {state}_tracts/             # Per-state shapefiles
              └─ *.shp
```

### Output Data (Processed, Regeneratable)

```
outputs/data/                            # ALL processed/derived data
  ├─ README.md                           # Documentation: What's here and how it's generated
  │
  ├─ tracts/                             # Census tract parquet files (geometries + population)
  │   ├─ 2000/
  │   │   ├─ alabama_tracts_2000.parquet         # Per-state tract files
  │   │   ├─ alaska_tracts_2000.parquet          # From NHGIS national file
  │   │   ├─ arizona_tracts_2000.parquet         # ~50 files total
  │   │   └─ ...
  │   ├─ 2010/
  │   │   ├─ alabama_tracts_2010.parquet         # Per-state tract files
  │   │   ├─ alaska_tracts_2010.parquet          # From PL 94-171 + TIGER/Line
  │   │   └─ ...                                 # ~50 files total
  │   └─ 2020/
  │       ├─ alabama_tracts_2020.parquet         # Per-state tract files
  │       ├─ alaska_tracts_2020.parquet          # From PL 94-171 + cenpy
  │       └─ ...                                 # ~50 files total
  │
  ├─ places/                             # City/place point files (optional, can be in tracts/)
  │   ├─ 2000/
  │   │   ├─ alabama_places_2000.parquet         # From NHGIS national file
  │   │   └─ ...
  │   ├─ 2010/
  │   │   ├─ alabama_places_2010.parquet         # Downloaded from Census API
  │   │   └─ ...
  │   └─ 2020/
  │       ├─ alabama_places_2020.parquet         # Downloaded from Census API
  │       └─ ...
  │
  ├─ adjacency/                          # Adjacency graphs (NetworkX pkl files)
  │   ├─ 2000/
  │   │   ├─ alabama_adjacency_2000.pkl          # Built from tract geometries
  │   │   ├─ alaska_adjacency_2000.pkl           # ~2-5MB per state
  │   │   └─ ...                                 # ~50 files total
  │   ├─ 2010/
  │   │   ├─ alabama_adjacency_2010.pkl
  │   │   └─ ...
  │   └─ 2020/
  │       ├─ alabama_adjacency_2020.pkl
  │       └─ ...
  │
  └─ processed/                          # Additional processed data
      ├─ elections/
      │   └─ 2020_president_tract.parquet        # Election results by tract (2020 only)
      ├─ demographics/
      │   ├─ 2000_demographics_tract.parquet     # Demographic data by tract
      │   ├─ 2010_demographics_tract.parquet
      │   └─ 2020_demographics_tract.parquet
      └─ baseline/
          └─ 2000_congressional_districts.parquet # From NHGIS cd108th shapefile
```

### Download Locations Summary

| Data Type | Year | Source | Download Location | Final Output Location |
|-----------|------|--------|-------------------|----------------------|
| Tract geometries | 2000 | NHGIS (manual) | `data/Census 2000/tracts/` | `outputs/data/units/2000/` |
| Tract population | 2000 | NHGIS (manual) | `data/Census 2000/population/` | (merged into tracts/) |
| Places | 2000 | NHGIS (manual) | `data/Census 2000/places/` | `outputs/data/places/2000/` |
| Demographics | 2000 | Census API | `data/Census 2000/demographics/` | `outputs/data/processed/demographics/` |
| Elections | 2000 | N/A | (none) | (none) |
| Tract geometries | 2010 | TIGER/Line FTP | `data/Census 2010/tiger/` (temp) | `outputs/data/units/2010/` |
| Tract population | 2010 | User-provided | `data/Census 2010/*.pl` | (merged into tracts/) |
| Places | 2010 | Census API | `data/Census 2010/tiger/` (temp) | `outputs/data/places/2010/` |
| Demographics | 2010 | Census API | `data/Census 2010/demographics/` | `outputs/data/processed/demographics/` |
| Elections | 2010 | N/A | (none) | (none) |
| Tract geometries | 2020 | cenpy API | `data/Census 2020/tiger/` (temp) | `outputs/data/units/2020/` |
| Tract population | 2020 | User-provided | `data/Census 2020/*.pl` | (merged into tracts/) |
| Places | 2020 | Census API | `data/Census 2020/tiger/` (temp) | `outputs/data/places/2020/` |
| Demographics | 2020 | Census API | `data/Census 2020/demographics/` | `outputs/data/processed/demographics/` |
| Elections | 2020 | MIT Election Lab | `data/Census 2020/elections/` | `outputs/data/processed/elections/` |

### Key Points:

1. **`data/` = Raw inputs** (organized by year, ~40GB total)
   - Official Census products (PL 94-171, NHGIS shapefiles)
   - Downloaded derived data (demographics, elections)
2. **`outputs/data/` = Processed outputs** (regeneratable, ~15-20GB)
3. **Consistent pattern**: Everything organized by census year
   - `data/Census 2000/` = All 2000 data sources
   - `data/Census 2010/` = All 2010 data sources
   - `data/Census 2020/` = All 2020 data sources
4. **Temporary downloads** (TIGER/Line geometries) go in `data/Census {year}/tiger/`, deleted after merge
5. **All pipeline scripts** read from `outputs/data/` (never from `data/`)

## Data Sources & Processing Strategy

### 2000 Census
**What we have**:
- ✅ NHGIS national tract shapefile (`US_tract_2000.shp`, 252MB)
- ✅ NHGIS tract population CSV (`nhgis0006_ds146_2000_tract.csv`, FL5001 = population)
- ✅ NHGIS places shapefile (`US_place_2000.shp`)
- ✅ NHGIS places CSV (`nhgis0006_ds146_2000_place.csv`)
- ⚠️ Census 2000 geos (`.upl` files) - Not needed, NHGIS has everything

**Processing strategy**:
1. Read NHGIS national tract shapefile
2. Split into per-state shapefiles (50 states)
3. Join with NHGIS population CSV (GISJOIN field)
4. Output to `outputs/data/units/2000/{state}_tracts_2000.parquet`
5. Read NHGIS places shapefile, split by state
6. Output to `outputs/data/units/2000/{state}_places_2000.parquet`
7. Build adjacency graphs from tract geometries

**No downloads needed** - Everything is already in `data/NHGIS/`

### 2010 Census
**What we have**:
- ✅ PL 94-171 population files (`.pl`) for all 50 states
- ✅ PL 94-171 geography files (`*geo*.pl`) for all 50 states

**What we'll download**:
- TIGER/Line tract shapefiles (per state, ~3GB total)
- TIGER/Line places shapefiles (per state)

**Processing strategy**:
1. Parse PL 94-171 files → Extract tract population (existing script: `parse_pl94171_tracts_2010.py`)
2. Download TIGER/Line tract shapefiles (existing script: `download_tiger_tracts_2010.py`)
3. Merge population + geometries (existing script: `merge_tracts_with_geometries_2010.py`)
4. Output to `outputs/data/units/2010/{state}_tracts_2010.parquet`
5. Download/process places → `outputs/data/units/2010/{state}_places_2010.parquet`
6. Build adjacency graphs from tract geometries

**Downloads automated** - TIGER/Line from Census Bureau FTP

### 2020 Census
**What we have**:
- ✅ PL 94-171 population files (`.pl`) for all 50 states

**What we'll download**:
- TIGER/Line tract geometries (via cenpy API or FTP)
- TIGER/Line places geometries

**Processing strategy**:
1. Download tract geometries (existing script: `download_all_states_tracts.py` uses cenpy)
2. Merge with population data
3. Output to `outputs/data/units/2020/{state}_tracts_2020.parquet`
4. Download/process places → `outputs/data/units/2020/{state}_places_2020.parquet`
5. Build adjacency graphs from tract geometries

**Downloads automated** - TIGER/Line via cenpy API (faster, more reliable)

## Implementation Plan

### Phase 1: Data Inventory & Documentation (1-2h)

**Goal**: Document current data holdings and create README files

- [x] Audit `data/NHGIS/` - ✅ Complete (2000 tracts, places, CSVs, baseline districts)
- [x] Audit `data/Census 2000/` - ✅ Only has geos/*.upl (not needed, using NHGIS)
- [x] Audit `data/Census 2010/` - ✅ Complete (PL 94-171 .pl + .plgeo files, 50 states)
- [x] Audit `data/Census 2020/` - ✅ Complete (PL 94-171 .pl files, 50 states)
- [x] Identify download needs - ✅ TIGER/Line for 2010/2020 (automated)
- [ ] Create `data/README.md` documenting raw data structure
- [ ] Create `data/NHGIS/README.md` documenting NHGIS downloads
- [ ] Create `outputs/data/README.md` documenting processed data structure

**Deliverables**:
- ✅ Inventory complete - See "Data Sources & Processing Strategy" section above
- README files documenting data provenance and structure

### Phase 2: Path Abstraction Layer (3-4h)

**Goal**: Update `scripts/utils/paths.py` to support new structure

- [ ] Add `get_raw_census_dir(year)` → `data/Census {year}/`
- [ ] Update `get_tract_file(state, year)` → `outputs/data/units/{year}/{state}_tracts_{year}.parquet`
- [ ] Update `get_places_file(state, year)` → `outputs/data/units/{year}/{state}_places_{year}.parquet`
- [ ] Update `get_adjacency_file(state, year)` → `outputs/data/adjacency/{year}/{state}_adjacency_{year}.pkl`
- [ ] Update `get_election_data_file(year)` → `outputs/data/processed/elections/{year}_president_tract.parquet`
- [ ] Update `get_demographic_data_file(year)` → `outputs/data/processed/demographics/{year}_demographics_tract.parquet`
- [ ] Add `ensure_output_data_dirs(year)` helper - Creates all necessary subdirectories

**Deliverables**:
- Updated `scripts/utils/paths.py`
- Unit tests for path functions
- All paths abstracted through single module

### Phase 3: Update Data Processing Scripts (6-8h)

**Goal**: Update all data acquisition/processing scripts to use new paths

#### 3.1: Census Tract Processing (2000 - NHGIS)
- [ ] **Create `process_nhgis_tracts_2000.py`** - NEW SCRIPT
  - Read `data/NHGIS/nhgis0001_shapefile_tl2000_us_tract_2000/US_tract_2000.shp` (national)
  - Read `data/NHGIS/nhgis0006_csv/nhgis0006_ds146_2000_tract.csv` (population)
  - Split national shapefile by state (STATEA field)
  - Join with population CSV on GISJOIN field
  - Output per-state parquet: `outputs/data/units/2000/{state}_tracts_2000.parquet`
  - Fields: GEOID, NAME, AREALAND, AREAWATR, INTPTLAT, INTPTLON, POPULATION, geometry

#### 3.2: Census Tract Processing (2010/2020 - PL 94-171 + TIGER)
- [ ] **`parse_pl94171_tracts_2010.py`** - Read from `data/Census 2010/`, write CSVs to temp
- [ ] **`download_tiger_tracts_2010.py`** - Download TIGER/Line, save to temp
- [ ] **`merge_tracts_with_geometries_2010.py`** - Merge, write to `outputs/data/units/2010/`
- [ ] **`download_all_states_tracts.py`** - Update for 2020, write to `outputs/data/units/2020/`
  - Currently uses cenpy API (good!)
  - Just needs path update to `outputs/data/`

#### 3.3: Places Processing
- [ ] **Create `process_nhgis_places_2000.py`** - NEW SCRIPT
  - Read `data/NHGIS/nhgis0004_shapefile_tl2000_us_place_2000/US_place_2000.shp` (national)
  - Read `data/NHGIS/nhgis0006_csv/nhgis0006_ds146_2000_place.csv` (optional, for additional data)
  - Split national shapefile by state
  - Output per-state parquet: `outputs/data/units/2000/{state}_places_2000.parquet`
- [ ] **`download_places.py`** - Update for 2010/2020, write to `outputs/data/units/{year}/`
- [ ] **`convert_nhgis_places_to_parquet.py`** - Update if used for 2010/2020

#### 3.4: Adjacency Graph Building
- [ ] **`build_tract_adjacency.py`** - Read from `outputs/data/units/{year}/`, write to `outputs/data/adjacency/{year}/`
- [ ] **`build_all_adjacency_graphs.py`** - Update paths
- [ ] **`compute_tract_adjacencies_2000.py`** - Update paths
- [ ] **`compute_tract_adjacencies_2010.py`** - Update paths

#### 3.5: Elections & Demographics
- [ ] **`download_election_data.py`** - Write to `outputs/data/processed/elections/`
- [ ] **`process_election_data.py`** - Write to `outputs/data/processed/elections/`
- [ ] **`download_demographic_data.py`** - Write to `outputs/data/processed/demographics/`
- [ ] **`process_demographic_data.py`** - Write to `outputs/data/processed/demographics/`

**Deliverables**:
- 20+ scripts updated to use new paths
- All scripts read from `data/Census {year}/` or download from web
- All scripts write to `outputs/data/`

### Phase 4: Update Pipeline & Analysis Scripts (4-6h)

**Goal**: Update all pipeline, visualization, and analysis scripts

- [ ] Review all imports of `get_tract_file`, `get_adjacency_file`, etc. (should just work)
- [ ] Test `run_state_redistricting.py` - Should use new paths automatically
- [ ] Test `process_single_state.py` - Should use new paths automatically
- [ ] Test political analysis scripts - Should use new paths automatically
- [ ] Test demographic analysis scripts - Should use new paths automatically
- [ ] Test compactness analysis scripts - Should use new paths automatically
- [ ] Test visualization scripts - Should use new paths automatically
- [ ] Update `.gitignore` - Change `data/` to `data/Census*/` (only ignore census dirs, not data/ itself)

**Deliverables**:
- All pipeline scripts working with new paths
- Verified no hardcoded paths remain

### Phase 5: Master Data Build Script (3-4h)

**Goal**: Create unified script to build all processed data

- [ ] Create `scripts/data/build_all_processed_data.py` with phases:
  - **Phase 1**: Parse PL 94-171 files (2000/2010/2020) → CSVs
  - **Phase 2**: Download TIGER/Line geometries (2000/2010/2020) → Shapefiles
  - **Phase 3**: Merge population + geometries → Parquet files
  - **Phase 4**: Download/convert places → Parquet files
  - **Phase 5**: Build adjacency graphs → Pkl files
  - **Phase 6**: Download/process elections → Parquet files
  - **Phase 7**: Download/process demographics → Parquet files
- [ ] Add `--year` parameter (2000/2010/2020)
- [ ] Add `--skip-existing` flag (skip if outputs already exist)
- [ ] Add `--reset` flag (delete existing outputs and rebuild)
- [ ] Add `--phase` parameter (run specific phase only)
- [ ] Add `--states` parameter (build specific states only, for testing)
- [ ] Parallel processing where possible (use multiprocessing for states)
- [ ] Progress reporting (STATUS protocol)
- [ ] Error logging (use ErrorLogger)

**Deliverables**:
- Master build script with 7 phases
- Can build all data from scratch
- Can build incrementally (skip existing)
- Clear progress and error reporting

### Phase 6: Pipeline Integration (2-3h)

**Goal**: Add data building as first stage in redistricting pipeline

- [ ] Update `run_complete_redistricting.py`:
  - Add `--build-data` flag (default: False)
  - If `--build-data`, run `build_all_processed_data.py` first
  - Check for missing data before starting redistricting
  - Provide helpful error messages if data missing
- [ ] Update `CLAUDE.md` with new data structure
- [ ] Update `ARCHITECTURE.md` with data flow diagram
- [ ] Update `GETTING_STARTED.md` with new setup instructions
- [ ] Update `DATA_FORMATS.md` with new paths

**Deliverables**:
- Pipeline can build data automatically
- Clear error messages for missing data
- Updated documentation

### Phase 7: Testing & Validation (3-4h)

**Goal**: Verify entire data pipeline works end-to-end

- [ ] Test Phase 1: Parse PL 94-171 for all states (2000/2010/2020)
- [ ] Test Phase 2: Download TIGER/Line for sample state (VT)
- [ ] Test Phase 3: Merge tracts for sample state (VT, 2000/2010/2020)
- [ ] Test Phase 4: Download/convert places for sample state (VT)
- [ ] Test Phase 5: Build adjacency graphs for sample state (VT, 2000/2010/2020)
- [ ] Test Phase 6: Download/process elections (2020 only)
- [ ] Test Phase 7: Download/process demographics (all years)
- [ ] Test full build: `build_all_processed_data.py --year 2020 --states VT`
- [ ] Test full pipeline: `run_complete_redistricting.py --year 2020 --version test --states VT`
- [ ] Test multi-year: `run_complete_redistricting.py --version test --states VT` (all 3 years)
- [ ] Validate outputs match previous V1 run (if available)

**Deliverables**:
- All phases tested independently
- Full build tested for sample state
- Full pipeline tested end-to-end
- Validation report

## Files to Modify/Create

### Create (New Files)
- `scripts/data/build_all_processed_data.py` - Master data build script (7 phases, ~500 lines)
- `scripts/data/census/process_nhgis_tracts_2000.py` - Process NHGIS national tract shapefile + CSV (~200 lines)
- `scripts/data/geography/process_nhgis_places_2000.py` - Process NHGIS national places shapefile (~150 lines)
- `data/README.md` - Documentation for raw census data structure
- `data/NHGIS/README.md` - Documentation for NHGIS downloads
- `outputs/data/README.md` - Documentation for processed data structure

### Modify (Existing Files)
- `scripts/utils/paths.py` - Update all path functions for new structure
- `scripts/data/census/parse_pl94171_tracts_2000.py` - Update input/output paths
- `scripts/data/census/parse_pl94171_tracts_2010.py` - Update input/output paths
- `scripts/data/geography/download_tiger_tracts_2000.py` - Update output paths
- `scripts/data/geography/download_tiger_tracts_2010.py` - Update output paths
- `scripts/data/geography/download_places.py` - Update output paths
- `scripts/data/geography/convert_nhgis_places_to_parquet.py` - Update input/output paths
- `scripts/data/merge_tracts_with_geometries_2000.py` - Update input/output paths
- `scripts/data/merge_tracts_with_geometries_2010.py` - Update input/output paths
- `scripts/data/census/download_all_states_tracts.py` - Update output paths (2020)
- `scripts/data/geography/build_tract_adjacency.py` - Update input/output paths
- `scripts/data/geography/build_all_adjacency_graphs.py` - Update paths
- `scripts/data/compute_tract_adjacencies_2000.py` - Update paths
- `scripts/data/compute_tract_adjacencies_2010.py` - Update paths
- `scripts/data/elections/download_election_data.py` - Update output paths
- `scripts/data/elections/process_election_data.py` - Update output paths
- `scripts/data/demographics/download_demographic_data.py` - Update output paths
- `scripts/data/demographics/process_demographic_data.py` - Update output paths
- `scripts/pipeline/run_complete_redistricting.py` - Add data building stage
- `.gitignore` - Update to only ignore `data/Census*/` not entire `data/`
- `CLAUDE.md` - Update data structure documentation
- `context/ARCHITECTURE.md` - Update data flow diagram
- `docs/GETTING_STARTED.md` - Update setup instructions
- `context/DATA_FORMATS.md` - Update with new paths

**Total**: ~25 files modified, 6 files created

## Testing Plan

1. **Phase-by-phase testing** (7 phases × 3 test scenarios):
   - Parse PL 94-171: Test VT for 2000/2010/2020
   - Download TIGER: Test VT for 2000/2010/2020
   - Merge tracts: Test VT for 2000/2010/2020
   - Places: Test VT for 2000/2010/2020
   - Adjacency: Test VT for 2000/2010/2020
   - Elections: Test 2020 only
   - Demographics: Test all years

2. **Full build test**:
   ```bash
   python scripts/data/build_all_processed_data.py --year 2020 --states VT
   ```
   - Verify all outputs created
   - Check file sizes reasonable
   - Validate parquet files load correctly

3. **Pipeline integration test**:
   ```bash
   python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states VT --build-data
   ```
   - Verify data built first
   - Verify redistricting runs
   - Verify outputs generated

4. **Multi-year test**:
   ```bash
   python scripts/pipeline/run_complete_redistricting.py --version test --states VT,DE --build-data
   ```
   - Verify all 3 years built and processed

5. **Validation** (if V1 data available):
   - Compare VT tract counts: V1 vs new
   - Compare VT adjacency graph sizes: V1 vs new
   - Compare VT final districts: V1 vs new

## Success Criteria

- [ ] All raw census data documented in `data/README.md`
- [ ] All PL 94-171 files parsed to CSVs for 2000/2010/2020
- [ ] All TIGER/Line geometries downloaded for sample state (VT)
- [ ] All tracts merged to parquet files in `outputs/data/units/{year}/`
- [ ] All places converted to parquet files in `outputs/data/units/{year}/`
- [ ] All adjacency graphs built in `outputs/data/adjacency/{year}/`
- [ ] All election data processed to `outputs/data/processed/elections/`
- [ ] All demographic data processed to `outputs/data/processed/demographics/`
- [ ] Master build script works end-to-end for sample state
- [ ] Full pipeline runs successfully with `--build-data` flag
- [ ] Multi-year pipeline works (2000/2010/2020)
- [ ] No hardcoded paths to old `data/tracts/` structure remain
- [ ] Documentation updated (CLAUDE.md, ARCHITECTURE.md, GETTING_STARTED.md)
- [ ] `.gitignore` updated to preserve `data/` directory structure
- [ ] V2 run completes successfully

## Benefits

1. **Clear separation of concerns**:
   - Raw census data (immutable, user-provided) vs. derived artifacts (regeneratable)
   - Easy to understand what's source material vs. what's computed

2. **Data loss prevention**:
   - Raw census data clearly identified and documented
   - Processed data can always be regenerated from raw data
   - Clear sourcing instructions for manual NHGIS downloads

3. **Reproducibility**:
   - Master build script can regenerate all processed data from scratch
   - Clear documentation of data provenance
   - Versioned outputs in `outputs/data/`

4. **Pipeline integration**:
   - Data building can be part of normal pipeline
   - `--build-data` flag for fresh runs
   - Skip existing for incremental updates

5. **Git safety**:
   - Only `data/Census*/` ignored (raw census data)
   - `data/README.md` tracked in git
   - Clear structure prevents accidental deletion

## Dependencies

- User has provided PL 94-171 files for 2000/2010/2020 ✅
- User needs to identify which NHGIS files were downloaded manually for 2000
- Existing data processing scripts (20+ scripts in `scripts/data/`)
- Path utilities in `scripts/utils/paths.py`

## Risks & Mitigations

- **Risk 1**: NHGIS manual downloads unclear
  - *Mitigation*: Work with user to document exactly what was downloaded and from where
  - *Mitigation*: Create step-by-step NHGIS download guide

- **Risk 2**: 2000 Census data has different format
  - *Mitigation*: Test 2000 parsing separately, may need format-specific handling
  - *Mitigation*: User has already done some 2000 work, leverage their knowledge

- **Risk 3**: Breaking existing V1 outputs
  - *Mitigation*: Path abstraction layer ensures backward compatibility initially
  - *Mitigation*: Thorough testing with sample states before full rollout

- **Risk 4**: Large data downloads (TIGER/Line ~5GB)
  - *Mitigation*: Add `--skip-existing` flag to avoid re-downloads
  - *Mitigation*: Download only needed years/states for testing

- **Risk 5**: Parallel processing complexity
  - *Mitigation*: Start with sequential processing, add parallelism later
  - *Mitigation*: Use existing multiprocessing patterns from pipeline

## Implementation Notes

### Data Inventory Complete ✅

**2000 Census** - NHGIS downloads complete:
- ✅ National tract shapefile (252MB .shp)
- ✅ National places shapefile
- ✅ Tract/place population CSVs
- ✅ 2000 congressional districts for baseline

**2010/2020 Census** - Will download TIGER/Line automatically:
- ✅ PL 94-171 files already provided by user
- Downloads: TIGER/Line geometries via FTP (2010) or cenpy API (2020)

### TIGER/Line Download Strategy

- 2020: Can use cenpy API (already implemented)
- 2010: TIGER/Line FTP (script exists)
- 2000: TIGER/Line FTP (script exists)

### Data Format Notes

**2000 Census** - NHGIS CSV format:
- GISJOIN: NHGIS join key (e.g., "G0100010020100")
- STATEA, COUNTYA, TRACTA: Geographic codes
- FL5001: Total population
- AREALAND, AREAWATR: Areas in square meters
- INTPTLAT, INTPTLON: Coordinates (need leading + sign)

**2010 Census** - PL 94-171 format:
- 2-file format: population (.pl) + geography (*geo*.pl)
- SUMLEV=140 for tracts
- Need to merge with TIGER/Line geometries

**2020 Census** - PL 94-171 format:
- 2-file format: population (.pl) + geography
- Can use cenpy API to get geometries directly (faster)

### Parallel Processing

- Parse PL 94-171: Can parallelize by state (50 states)
- Download TIGER: Can parallelize by state
- Merge: Can parallelize by state
- Adjacency: Can parallelize by state
- Elections/Demographics: Typically national files, sequential

## Key Decisions

_(To be filled during implementation)_

1. **NHGIS vs TIGER/Line**: Which source for geometries?
2. **Temp files**: Where to store intermediate CSVs/shapefiles?
3. **Error handling**: Stop on first error or continue?
4. **Validation**: How to validate each phase succeeded?

## Related Documentation

- Architecture doc: [ARCHITECTURE.md](../../ARCHITECTURE.md)
- Coding patterns: [CODING_PATTERNS.md](../../CODING_PATTERNS.md)
- Data formats: [DATA_FORMATS.md](../../DATA_FORMATS.md)
- Enhancement 13: [Directory Unification](13_directory_unification.md) - Previous path restructuring
- Enhancement 15: [Multi-Year Support](15_multi_year_support.md) - Multi-year patterns
