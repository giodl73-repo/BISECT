---
wave_uuid: 1c723c
slug: per-version-census-data
uuid: 008a94
---
# E52: Per-Version Census Data Structure

**Status**: ✅ COMPLETE
**Priority**: High
**Estimated Effort**: Large (12-16h)
**Actual Effort**: ~6-8h (systematic implementation)
**Created**: 2026-01-19
**Started**: 2026-01-19
**Completed**: 2026-01-19
**Commits**: _(filled automatically after implementation)_
**Size**: _(filled automatically after implementation)_

## Priority Levels

- **High**: Enables version-specific preprocessing experiments and independent algorithm comparisons - critical for research iteration

## Current State

After E47, census data architecture is:

```
data/Census {year}/          # Raw census data (40GB, unchanging)
  ├─ 2000/, 2010/, 2020/    # PL 94-171 files, NHGIS downloads

outputs/data/{year}/         # Processed data (1.7GB, shared across ALL versions)
  ├─ units/
  │   ├─ 2000/  →  {state}_tracts_2000.parquet (50 files)
  │   ├─ 2010/  →  {state}_tracts_2010.parquet (50 files)
  │   └─ 2020/  →  {state}_tracts_2020.parquet (50 files)
  ├─ adjacency/
  │   ├─ 2000/  →  {state}_adjacency_2000.pkl (50 files)
  │   ├─ 2010/  →  {state}_adjacency_2010.pkl (50 files)
  │   └─ 2020/  →  {state}_adjacency_2020.pkl (50 files)
  ├─ demographics/  →  {year}_demographics_tract.parquet
  └─ elections/     →  2020_president_tract.parquet

outputs/{version}/{year}/    # Redistricting results (per-version)
  ├─ states/{state}/
  ├─ data/                   # National aggregates only
  └─ maps/
```

**Problem**: Census data preprocessing (units, adjacency, demographics) is **shared** across all versions:
- v1, v2, v3 all read from same `outputs/data/{year}/`
- Cannot experiment with different preprocessing per version (e.g., different edge weights, filtered adjacencies, resolution changes)
- Makes versions interdependent - changing preprocessing for v2 breaks v1

**Storage**: Currently 1.7GB shared

## Goal

**Make each version completely self-contained** by moving processed census data into version-specific directories.

### Chosen Structure: Parallel Organization ⭐

**Decision**: Fully parallel structure with `data/` for inputs and `results/` for outputs.

```
outputs/v1/
  ├─ data/                    # Census data (inputs)
  │   ├─ 2020/
  │   │   ├─ units/           # Census tracts (50 files)
  │   │   ├─ adjacency/       # Adjacency graphs (50 files)
  │   │   ├─ demographics/    # Demographics
  │   │   ├─ elections/       # Elections
  │   │   └─ places/          # Places
  │   ├─ 2010/
  │   │   └─ ... (same structure)
  │   └─ 2000/
  │       └─ ... (same structure)
  │
  └─ results/                 # Redistricting outputs
      ├─ 2020/
      │   ├─ data/            # National aggregates (us_*.csv)
      │   ├─ states/          # Per-state results
      │   │   └─ {state}/
      │   │       ├─ data/    # State data (assignments, summaries)
      │   │       ├─ maps/    # State maps
      │   │       ├─ political/
      │   │       ├─ demographic/
      │   │       └─ compactness/
      │   └─ maps/            # National maps
      ├─ 2010/
      │   └─ ... (same structure)
      └─ 2000/
          └─ ... (same structure)
```

**Why This Structure**:

✅ **Perfect symmetry**: Inputs (`data/`) and outputs (`results/`) organized identically
✅ **Self-documenting**: Directory names explain purpose (data vs results)
✅ **Clean separation**: Census data completely separate from redistricting results
✅ **No name collisions**: `v1/data/2020/` ≠ `v1/results/2020/data/`
✅ **Future-proof**: Clear place for new top-level categories (experiments/, baselines/, variants/)
✅ **Manageable**: Can backup/delete/share `data/` or `results/` independently
✅ **Scalable**: Structure makes sense with 1 version or 100 versions

**Trade-offs**:
- Longer paths: `v1/results/2020/states/` vs old `v1/2020/states/` (+8 chars)
- Breaking change: All existing v1 paths need updating
- ✅ **User decision**: Delete existing v1, start fresh - no migration needed!

## Search & Replace Patterns

**Before starting**: Use these regex patterns to find ALL locations needing changes in each file.

### Pattern 1: Path Utility Function Calls (Change: Add `version` parameter)

```regex
# Find all calls missing version parameter:
get_tract_file\([^,]+,\s*[^,)]+\)(?!\s*,)
get_adjacency_file\([^,]+,\s*[^,)]+\)(?!\s*,)
get_places_file\([^,]+,\s*[^,)]+\)(?!\s*,)
get_election_data_file\([^,)]+\)(?!\s*,)
get_demographic_data_file\([^,)]+\)(?!\s*,)
get_output_dir\(.*?\)
get_state_output_dir\(.*?\)

# Example matches:
get_tract_file('california', '2020')  # NEEDS: version parameter
get_adjacency_file(state, year)      # NEEDS: version parameter
```

### Pattern 2: Hardcoded `outputs/data/` Paths (Change: Use path utilities)

```regex
# Find all hardcoded outputs/data/ paths:
['"]outputs/data/\{?year\}?/
f['"]outputs/data/\{[^}]+\}/
Path\(.*?outputs/data/

# Example matches:
"outputs/data/2020/units/"              # CHANGE TO: get_census_data_dir(version, '2020')
f"outputs/data/{year}/adjacency/"      # CHANGE TO: get_census_data_dir(version, year) / 'adjacency'
Path('outputs/data/2020/units')        # CHANGE TO: get_census_data_dir(version, '2020') / 'units'
```

### Pattern 3: Hardcoded `outputs/{version}/{year}/` Paths (Change: Add `results/`)

```regex
# Find all outputs/{version}/{year}/ paths (need results/ inserted):
['"]outputs/\{?version\}?/\{?year\}?/(?!data/)
f['"]outputs/\{[^}]+\}/\{[^}]+\}/(?!data/)
Path\(.*?outputs.*?version.*?year

# Example matches:
f"outputs/{version}/{year}/states/"        # CHANGE TO: f"outputs/{version}/results/{year}/states/"
f"outputs/us_{year}_{version}/"           # CHANGE TO: f"outputs/{version}/results/{year}/"
Path(f'outputs/{version}/{year}/data/')   # CHANGE TO: get_results_dir(version, year) / 'data'
```

### Pattern 4: `get_output_dir` Calls (Change: Parameter order + results/)

```regex
# Find all get_output_dir calls:
get_output_dir\(([^,]+),\s*([^)]+)\)

# Example matches:
get_output_dir(year, version)   # CHANGE TO: get_output_dir(version, year) AND add results/
get_output_dir('2020', 'v1')   # CHANGE TO: get_output_dir('v1', '2020')
```

### Pattern 5: Subprocess Calls (Change: Add `--version` argument)

```regex
# Find subprocess calls missing --version:
subprocess\.(run|Popen|call)\(.*?python.*?(process_census_data|build.*adjacency|analyze_|visualize_)
sys\.executable.*?(process_census_data|build.*adjacency|analyze_|visualize_)

# Example matches:
subprocess.run([sys.executable, 'scripts/data/process_census_data.py', '--year', year])
# CHANGE TO: Add '--version', version
```

### Pattern 6: Marker File Paths (Change: Move to version-specific location)

```regex
# Find marker files:
\.census_complete
\.states_complete
\.nation_complete
\.(tract|adjacency|demographics|elections).*_complete

# Example matches:
f"outputs/data/{year}/.census_complete"           # CHANGE TO: f"outputs/{version}/data/{year}/.census_complete"
f"outputs/{version}/{year}/.states_complete"      # CHANGE TO: f"outputs/{version}/results/{year}/.states_complete"
```

### How to Use These Patterns

For each file in the manifest:

1. **Open file in editor**
2. **Run Pattern 1**: Find all path function calls without `version`
   - Add `version` parameter to each
3. **Run Pattern 2**: Find all hardcoded `outputs/data/` paths
   - Replace with `get_census_data_dir(version, year)`
4. **Run Pattern 3**: Find all `outputs/{version}/{year}/` paths without `results/`
   - Insert `results/` OR use `get_results_dir(version, year)`
5. **Run Pattern 4**: Find all `get_output_dir` calls
   - Swap parameter order: (year, version) → (version, year)
6. **Run Pattern 5**: Find subprocess calls
   - Add `--version` argument
7. **Run Pattern 6**: Find marker files
   - Update paths to version-specific locations
8. **Verify**: Check for any remaining hardcoded patterns

### Quick Verification Commands

After editing each file:
```bash
# Check for remaining old patterns:
grep -n "outputs/data/" FILE.py
grep -n "outputs/us_" FILE.py
grep -n 'get_tract_file([^,]*,[^,]*)' FILE.py  # Missing version?
grep -n 'get_output_dir([^,]*,[^,]*)' FILE.py  # Check parameter order
```

## Implementation Plan

### Phase 1: Path Utilities Update (3-4h) - MANUAL ONLY

**Goal**: Update core path functions for new structure

**Critical**: All changes in this phase MUST be done manually. No batch operations.

#### 1.1: Update `scripts/utils/paths.py` (MANUAL)

Current functions to modify:
- [ ] `get_tract_file(state, year)` → `get_tract_file(state, year, version)`
  - Old: `outputs/data/{year}/units/{state}_tracts_{year}.parquet`
  - New: `outputs/{version}/data/{year}/units/{state}_tracts_{year}.parquet`

- [ ] `get_places_file(state, year)` → `get_places_file(state, year, version)`
  - Old: `outputs/data/{year}/places/{state}_places_{year}.parquet`
  - New: `outputs/{version}/data/{year}/places/{state}_places_{year}.parquet`

- [ ] `get_adjacency_file(state, year)` → `get_adjacency_file(state, year, version)`
  - Old: `outputs/data/{year}/adjacency/{state}_adjacency_{year}.pkl`
  - New: `outputs/{version}/data/{year}/adjacency/{state}_adjacency_{year}.pkl`

- [ ] `get_election_data_file(year)` → `get_election_data_file(year, version)`
  - Old: `outputs/data/{year}/elections/{year}_president_tract.parquet`
  - New: `outputs/{version}/data/{year}/elections/{year}_president_tract.parquet`

- [ ] `get_demographic_data_file(year)` → `get_demographic_data_file(year, version)`
  - Old: `outputs/data/{year}/demographics/{year}_demographics_tract.parquet`
  - New: `outputs/{version}/data/{year}/demographics/{year}_demographics_tract.parquet`

- [ ] `get_output_dir(year, version)` → `get_output_dir(version, year)`
  - Old: `outputs/{version}/{year}`
  - New: `outputs/{version}/results/{year}`
  - **Note**: Also swap parameter order for consistency (version first)

- [ ] `get_state_output_dir(state, year, version)` → `get_state_output_dir(state, version, year)`
  - Old: `outputs/{version}/{year}/states/{state}`
  - New: `outputs/{version}/results/{year}/states/{state}`
  - **Note**: Also swap parameter order for consistency (state, version, year)

New functions to add:
- [ ] `get_census_data_dir(version, year)` → `outputs/{version}/data/{year}/`
- [ ] `get_results_dir(version, year)` → `outputs/{version}/results/{year}/`
- [ ] `get_version_dir(version)` → `outputs/{version}/`

#### 1.2: Update `scripts/utils/__init__.py` (MANUAL)

- [ ] Update all exports to match new function signatures
- [ ] Verify imports work correctly

#### 1.3: Update `tests/unit/test_utils_paths.py` (MANUAL)

- [ ] Update all test cases to pass `version` parameter
- [ ] Test new paths: `outputs/v1/data/2020/units/california_tracts_2020.parquet`
- [ ] Test results paths: `outputs/v1/results/2020/states/california/`
- [ ] Add tests for new functions (`get_census_data_dir`, `get_results_dir`, `get_version_dir`)
- [ ] Ensure 100% coverage

**Deliverables**:
- [ ] All path functions require `version` parameter (no defaults)
- [ ] Census data paths: `outputs/{version}/data/{year}/`
- [ ] Results paths: `outputs/{version}/results/{year}/`
- [ ] Unit tests passing

**Files to modify manually** (3 files):
1. `scripts/utils/paths.py` (~150 lines → ~200 lines)
2. `scripts/utils/__init__.py` (~20 lines)
3. `tests/unit/test_utils_paths.py` (~200 lines → ~300 lines)

### Phase 2: Census Data Processing Scripts (4-5h) - MANUAL ONLY

**Goal**: Make census data processing scripts version-aware

**Critical**: All changes MUST be done manually. Review each script for hardcoded paths.

#### 2.1: Core Census Processing (MANUAL)

- [ ] **`scripts/data/process_census_data.py`** (~400 lines)
  - Add `--version` parameter to argparse
  - Pass `version` to all path function calls
  - Update output paths: `get_census_data_dir(version, year)`
  - Update marker file: `outputs/{version}/data/{year}/.census_complete`
  - Update progress messages to show version

- [ ] **`scripts/data/download_orchestrator.py`** (~600 lines)
  - Add `--version` parameter to argparse (optional, defaults to shared for downloads)
  - Note: Downloads still go to `data/Census {year}/` (raw data)
  - Document that this is for raw downloads, not processed data

#### 2.2: Census Parsing Scripts (MANUAL)

- [ ] **`scripts/data/census/parse_pl94171_tracts_2000.py`**
  - Review hardcoded `outputs/data/` paths
  - Update to use path utilities if needed

- [ ] **`scripts/data/census/parse_pl94171_tracts_2010.py`**
  - Review hardcoded `outputs/data/` paths
  - Update to use path utilities if needed

- [ ] **`scripts/data/census/parse_pl94171_tracts_2020.py`**
  - Review hardcoded `outputs/data/` paths
  - Update to use path utilities if needed

#### 2.3: Geography Processing (MANUAL)

- [ ] **`scripts/data/merge_units_with_geometries.py`**
  - Add `--version` parameter
  - Update output paths to `outputs/{version}/data/{year}/units/`

- [ ] **`scripts/data/geography/build_tract_adjacency.py`** (~300 lines)
  - Add `--version` parameter
  - Pass `version` to `get_tract_file()` and `get_adjacency_file()`
  - Update output: `outputs/{version}/data/{year}/adjacency/`

- [ ] **`scripts/data/geography/build_all_adjacency_graphs.py`** (~200 lines)
  - Add `--version` parameter
  - Pass to child processes

#### 2.4: Validation (MANUAL)

- [ ] **`scripts/data/validate_census_data.py`**
  - Add `--version` parameter
  - Check `outputs/{version}/data/{year}/` for completeness

**Deliverables**:
- [ ] All census scripts accept `--version` parameter
- [ ] All write to `outputs/{version}/data/{year}/`
- [ ] Marker files in correct version-specific locations
- [ ] Progress reporting shows version context

**Files to modify manually** (9 files):
1. `scripts/data/process_census_data.py`
2. `scripts/data/download_orchestrator.py`
3. `scripts/data/census/parse_pl94171_tracts_2000.py`
4. `scripts/data/census/parse_pl94171_tracts_2010.py`
5. `scripts/data/census/parse_pl94171_tracts_2020.py`
6. `scripts/data/merge_units_with_geometries.py`
7. `scripts/data/geography/build_tract_adjacency.py`
8. `scripts/data/geography/build_all_adjacency_graphs.py`
9. `scripts/data/validate_census_data.py`

### Phase 3: Core Pipeline Scripts (5-6h) - MANUAL ONLY

**Goal**: Update main redistricting pipeline for new structure

**Critical**: All changes MUST be done manually. These are the most critical scripts.

#### 3.1: Main Pipeline Orchestrator (MANUAL)

- [ ] **`scripts/pipeline/run_complete_redistricting.py`** (~800 lines) **CRITICAL**
  - Add census data stage BEFORE states stage
  - Pass `version` to all subprocess calls
  - Update output directories: `outputs/{version}/results/{year}/`
  - Update marker files: `outputs/{version}/data/{year}/.census_complete`
  - Update all path constructions to use `get_results_dir(version, year)`
  - Update national aggregation paths
  - **Review every subprocess call carefully**

#### 3.2: State Processing (MANUAL)

- [ ] **`scripts/pipeline/run_state_redistricting.py`** (~400 lines) **CRITICAL**
  - Update imports: pass `version` to all path functions
  - Update: `get_tract_file(state, year, version)`
  - Update: `get_adjacency_file(state, year, version)`
  - Update: `get_state_output_dir(state, version, year)` (note parameter order change!)
  - Update output_dir: `outputs/{version}/results/{year}/states/{state}/`
  - Verify census data exists before starting

- [ ] **`scripts/pipeline/process_single_state.py`** (~500 lines) **CRITICAL**
  - Pass `version` to all `get_*_file()` calls
  - Update: `get_tract_file()`, `get_adjacency_file()`, `get_election_data_file()`, `get_demographic_data_file()`
  - Update state output directory paths
  - Update all subprocess calls to analysis scripts (pass version)

#### 3.3: State Output Generation (MANUAL)

- [ ] **`scripts/pipeline/add_cities_to_districts.py`** (~200 lines)
  - Update: `get_places_file(state, year, version)`
  - Update state output directory paths

- [ ] **`scripts/pipeline/create_final_district_summary.py`** (~150 lines)
  - Update state output directory paths
  - Review hardcoded paths

- [ ] **`scripts/pipeline/export_rounds_to_csv.py`** (~100 lines)
  - Update state output directory paths

- [ ] **`scripts/pipeline/create_single_district_states.py`** (~100 lines)
  - Update: `get_state_output_dir(state, version, year)`

#### 3.4: National Aggregation (MANUAL)

- [ ] **`scripts/pipeline/process_nation.py`** (~400 lines)
  - Update: `get_results_dir(version, year)` for output paths
  - Update all paths: `outputs/{version}/results/{year}/data/`
  - Update all paths: `outputs/{version}/results/{year}/maps/`

- [ ] **`scripts/pipeline/create_us_aggregate.py`** (~200 lines)
  - Update national data output: `outputs/{version}/results/{year}/data/us_*.csv`
  - Review all hardcoded `outputs/` paths

- [ ] **`scripts/pipeline/create_us_rounds_hierarchy.py`** (~150 lines)
  - Update output: `outputs/{version}/results/{year}/data/us_rounds_hierarchy.csv`

#### 3.5: Utility Scripts (MANUAL)

- [ ] **`scripts/pipeline/cleanup_district_summary.py`**
  - Update paths if used

- [ ] **`scripts/pipeline/fill_missing_cities.py`**
  - Update: `get_places_file(state, year, version)`

- [ ] **`scripts/pipeline/fix_2010_missing_outputs.py`**
  - Review and update all hardcoded paths

**Deliverables**:
- [ ] Census data stage integrated in main pipeline
- [ ] All redistricting outputs go to `outputs/{version}/results/{year}/`
- [ ] All census data reads from `outputs/{version}/data/{year}/`
- [ ] Parameter order consistency (state, version, year)
- [ ] Marker files in correct locations

**Files to modify manually** (13 files):
1. `scripts/pipeline/run_complete_redistricting.py` ⚠️ CRITICAL
2. `scripts/pipeline/run_state_redistricting.py` ⚠️ CRITICAL
3. `scripts/pipeline/process_single_state.py` ⚠️ CRITICAL
4. `scripts/pipeline/add_cities_to_districts.py`
5. `scripts/pipeline/create_final_district_summary.py`
6. `scripts/pipeline/export_rounds_to_csv.py`
7. `scripts/pipeline/create_single_district_states.py`
8. `scripts/pipeline/process_nation.py`
9. `scripts/pipeline/create_us_aggregate.py`
10. `scripts/pipeline/create_us_rounds_hierarchy.py`
11. `scripts/pipeline/cleanup_district_summary.py`
12. `scripts/pipeline/fill_missing_cities.py`
13. `scripts/pipeline/fix_2010_missing_outputs.py`

### Phase 4: Analysis & Visualization Scripts (4-5h) - MANUAL ONLY

**Goal**: Update all analysis and visualization scripts

**Critical**: Review each script for hardcoded paths to `outputs/` directories.

#### 4.1: Political Analysis (MANUAL)

- [ ] **`scripts/pipeline/analyze_district_demographics.py`** (~300 lines)
  - Update: `get_demographic_data_file(year, version)`
  - Update state input paths
  - Update state output paths

#### 4.2: Compactness Analysis (MANUAL)

- [ ] **`scripts/pipeline/analyze_district_compactness.py`** (~200 lines)
  - Update state input paths: `outputs/{version}/results/{year}/states/{state}/`
  - Update state output paths

- [ ] **`scripts/baseline/recompute_algorithmic_compactness.py`** (~150 lines)
  - Update default output_dir: `f'outputs/{version}/results/{year}'`
  - Review hardcoded path constructions (line 146, 152)

- [ ] **`scripts/baseline/compare_algorithmic_vs_enacted.py`** (~300 lines)
  - Update defaults: `outputs/{version}/results/{year}/` (lines 210-223)

- [ ] **`scripts/baseline/compute_enacted_compactness.py`** (~250 lines)
  - Update defaults (lines 226-227)

- [ ] **`scripts/baseline/visualize_three_way_comparison.py`** (~200 lines)
  - Update paths (lines 201, 207)

#### 4.3: Visualization - State Level (MANUAL)

- [ ] **`scripts/pipeline/visualize_individual_districts.py`** (~200 lines)
  - Update: `get_tract_file(state, year, version)`
  - Update state output paths

- [ ] **`scripts/pipeline/visualize_all_rounds.py`** (~250 lines)
  - Update: `get_tract_file(state, year, version)`
  - Update state output paths

#### 4.4: Visualization - National Level (MANUAL)

- [ ] **`scripts/pipeline/visualize_national_districts.py`** (~300 lines)
  - Update: `get_tract_file(state, year, version)` for all states
  - Update national output: `outputs/{version}/results/{year}/maps/`

- [ ] **`scripts/pipeline/visualize_national_rounds.py`** (~250 lines)
  - Update: `get_tract_file()` calls
  - Update national output paths
  - Review hardcoded paths (f-strings)

- [ ] **`scripts/pipeline/visualize_metro_areas.py`** (~300 lines)
  - Update: `get_tract_file(state, year, version)`
  - Update: `get_places_file(state, year, version)`
  - Update state output paths
  - Review hardcoded Path constructions

- [ ] **`scripts/pipeline/visualize_compactness.py`** (~250 lines)
  - Update state input/output paths
  - Review hardcoded paths

- [ ] **`scripts/pipeline/visualize_district_demographics.py`** (~200 lines)
  - Update: `get_demographic_data_file(year, version)`
  - Update state input/output paths

#### 4.5: Figure Generation (MANUAL)

- [ ] **`scripts/figures/generate_all_figures.py`** (~400 lines)
  - Review Path constructions with year/version
  - Update to new structure

- [ ] **`scripts/figures/create_figure_variants.py`** (~200 lines)
  - Review hardcoded paths

**Deliverables**:
- [ ] All analysis scripts read from `outputs/{version}/data/{year}/`
- [ ] All visualizations output to `outputs/{version}/results/{year}/`
- [ ] No hardcoded `outputs/data/` or `outputs/us_` paths remain

**Files to modify manually** (17 files):
1. `scripts/pipeline/analyze_district_demographics.py`
2. `scripts/pipeline/analyze_district_compactness.py`
3. `scripts/baseline/recompute_algorithmic_compactness.py`
4. `scripts/baseline/compare_algorithmic_vs_enacted.py`
5. `scripts/baseline/compute_enacted_compactness.py`
6. `scripts/baseline/visualize_three_way_comparison.py`
7. `scripts/pipeline/visualize_individual_districts.py`
8. `scripts/pipeline/visualize_all_rounds.py`
9. `scripts/pipeline/visualize_national_districts.py`
10. `scripts/pipeline/visualize_national_rounds.py`
11. `scripts/pipeline/visualize_metro_areas.py`
12. `scripts/pipeline/visualize_compactness.py`
13. `scripts/pipeline/visualize_district_demographics.py`
14. `scripts/figures/generate_all_figures.py`
15. `scripts/figures/create_figure_variants.py`
16. Plus any `artifacts/papers/*/create_figures.py` scripts
17. Plus any `artifacts/presentations/*/create_figures.py` scripts

### Phase 5: Web, Dashboard & Utilities (3-4h) - MANUAL ONLY

**Goal**: Update web dashboards, utilities, and validation scripts

**Critical**: Dashboard scripts have many hardcoded path patterns.

#### 5.1: Dashboard Generation (MANUAL)

- [ ] **`scripts/web/generate_dashboard.py`** (~600 lines) **CRITICAL**
  - Review all Path constructions with version/year
  - Update to read from: `outputs/{version}/results/{year}/`
  - Update URL patterns in HTML if needed

- [ ] **`scripts/web/generate_master_dashboard.py`** (~500 lines) **CRITICAL**
  - Review all Path constructions
  - Update to new directory structure

- [ ] **`scripts/web/generate_enhanced_master_dashboard.py`** (~400 lines)
  - Update line 294: `f"outputs/{run['path']}"` → use new structure
  - Review all path aggregations

#### 5.2: Utilities (MANUAL)

- [ ] **`scripts/utils/common.py`** (~200 lines)
  - Review any path construction helpers
  - Update if used by other scripts

- [ ] **`scripts/utils/error_logger.py`** (~150 lines)
  - Update output paths: `outputs/{version}/results/{year}/error.log`
  - Review Path constructions

- [ ] **`scripts/utils/stage_tracker.py`** (~100 lines)
  - Review hardcoded paths

#### 5.3: Validation (MANUAL)

- [ ] **`scripts/validation/validate_pipeline_outputs.py`** (~300 lines)
  - Update to check: `outputs/{version}/results/{year}/`
  - Update census data checks: `outputs/{version}/data/{year}/`
  - Review all Path constructions

#### 5.4: Archive/Debug Scripts (MANUAL - Lower Priority)

- [ ] **`scripts/archive/save_run.py`** (~200 lines)
  - Update default: `outputs/saved` → could stay as-is
  - Update source paths (lines 41, 154-155)

- [ ] **`scripts/debug/quick_tract_map.py`**
  - Update any hardcoded paths

- [ ] **`scripts/debug/debug_district_places.py`**
  - Update any hardcoded paths

**Deliverables**:
- [ ] Dashboards read from new structure
- [ ] Error logs go to correct version directories
- [ ] Validation scripts check new paths
- [ ] All utility functions updated

**Files to modify manually** (10 files):
1. `scripts/web/generate_dashboard.py` ⚠️ CRITICAL
2. `scripts/web/generate_master_dashboard.py` ⚠️ CRITICAL
3. `scripts/web/generate_enhanced_master_dashboard.py`
4. `scripts/utils/common.py`
5. `scripts/utils/error_logger.py`
6. `scripts/utils/stage_tracker.py`
7. `scripts/validation/validate_pipeline_outputs.py`
8. `scripts/archive/save_run.py`
9. `scripts/debug/quick_tract_map.py`
10. `scripts/debug/debug_district_places.py`

### Phase 6: Tests (3-4h) - MANUAL ONLY

**Goal**: Update all test files for new structure

**Critical**: Every test that touches file paths needs updating.

#### 6.1: Unit Tests (MANUAL)

- [ ] **`tests/unit/test_utils_paths.py`** (~300 lines) **CRITICAL**
  - Already covered in Phase 1, but verify completeness
  - Add tests for all new functions

- [ ] **`tests/unit/test_process_census_data.py`** (~200 lines)
  - Update expected paths in assertions
  - Update mock paths

- [ ] **`tests/unit/test_merge_units_with_geometries.py`** (~150 lines)
  - Update expected paths

#### 6.2: Integration Tests (MANUAL)

- [ ] **`tests/integration/test_census_pipeline_integration.py`** (if exists)
  - Update all path expectations

#### 6.3: E2E Tests (MANUAL)

- [ ] **`tests/e2e/test_pipeline_scripts.py`** (~400 lines)
  - Update expected output paths: `outputs/{version}/results/{year}/`
  - Update expected data paths: `outputs/{version}/data/{year}/`

- [ ] **`tests/e2e/conftest.py`** (~200 lines)
  - Update fixture paths
  - Update test data directories

- [ ] **`tests/e2e/test_run_dashboard.py`** (~150 lines)
  - Update dashboard path expectations

- [ ] **`tests/conftest.py`** (root) (~100 lines)
  - Update any shared fixture paths

#### 6.4: Test Scripts (MANUAL - Lower Priority)

- [ ] **`scripts/tests/run_compactness_test.py`**
- [ ] **`scripts/tests/run_niter100_test.py`**
- [ ] **`scripts/tests/test_first_split_tracts.py`**
- [ ] **`scripts/tests/test_second_split_tracts.py`**
- [ ] **`scripts/tests/test_tract_level_split.py`**
- [ ] **`scripts/tests/test_niter_values.py`**

**Deliverables**:
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] All e2e tests passing
- [ ] No hardcoded old paths in test assertions

**Files to modify manually** (13+ files):
1. `tests/unit/test_utils_paths.py` ⚠️ CRITICAL
2. `tests/unit/test_process_census_data.py`
3. `tests/unit/test_merge_units_with_geometries.py`
4. `tests/e2e/test_pipeline_scripts.py`
5. `tests/e2e/conftest.py`
6. `tests/e2e/test_run_dashboard.py`
7. `tests/conftest.py`
8. `scripts/tests/run_compactness_test.py`
9. `scripts/tests/run_niter100_test.py`
10. `scripts/tests/test_first_split_tracts.py`
11. `scripts/tests/test_second_split_tracts.py`
12. `scripts/tests/test_tract_level_split.py`
13. `scripts/tests/test_niter_values.py`

### Phase 7: Documentation (2-3h) - MANUAL ONLY

**Goal**: Update all documentation for new structure

**Critical**: Documentation must be accurate for future work.

#### 7.1: Core Documentation (MANUAL)

- [ ] **`CLAUDE.md`** (~400 lines)
  - Update File Structure section
  - Update storage estimates: 1.7GB per version
  - Update common commands with new paths
  - Update Quick Reference paths

- [ ] **`context/ARCHITECTURE.md`** (~400 lines)
  - Update File Structure section (lines 35-54)
  - Update data flow diagram
  - Document census data stage
  - Explain per-version data benefits

- [ ] **`context/CODING_PATTERNS.md`** (~300 lines)
  - Add pattern: Always pass `version` to path functions
  - Add examples with new paths
  - Document parameter order: (state, version, year) or (version, year)

- [ ] **`context/DATA_FORMATS.md`** (~200 lines)
  - Update all path examples
  - Document new directory structure

- [ ] **`context/QUICK_REFERENCE.md`** (~300 lines)
  - Update command examples
  - Update troubleshooting paths

#### 7.2: Enhancement Documentation (MANUAL)

- [ ] **`context/enhancements/active/47_data_separation_restoration.md`**
  - Add note: "Superseded by E52"
  - Document evolution of structure

- [ ] **E52** (this file)
  - Mark phases complete as done
  - Document final file counts

#### 7.3: Other Documentation (MANUAL)

- [ ] **`docs/CHANGELOG.md`**
  - Add entry for E52

- [ ] **`docs/GETTING_STARTED.md`** (if exists)
  - Update setup instructions with new paths

- [ ] **`docs/DATA_DICTIONARY.md`** (if exists)
  - Update path references

**Deliverables**:
- [ ] All documentation reflects `outputs/{version}/data/` and `outputs/{version}/results/`
- [ ] Examples use correct paths
- [ ] Architecture diagrams updated
- [ ] Enhancement docs complete

**Files to modify manually** (9 files):
1. `CLAUDE.md` ⚠️ CRITICAL
2. `context/ARCHITECTURE.md` ⚠️ CRITICAL
3. `context/CODING_PATTERNS.md`
4. `context/DATA_FORMATS.md`
5. `context/QUICK_REFERENCE.md`
6. `context/enhancements/active/47_data_separation_restoration.md`
7. `context/enhancements/active/52_per_version_census_data.md`
8. `docs/CHANGELOG.md`
9. `docs/GETTING_STARTED.md`, `docs/DATA_DICTIONARY.md` (if they exist)

### Phase 6: Testing & Validation (3-4h)

**Goal**: Verify end-to-end pipeline works with new structure

- [ ] Unit tests:
  - [ ] Path utilities: Test all version-specific paths
  - [ ] Census processing: Mock version parameter handling
- [ ] Integration tests:
  - [ ] Census data build: Test for test version + Vermont
  - [ ] Pipeline: Test complete run with version-specific data
- [ ] E2E tests:
  - [ ] Full pipeline test: Vermont, 2020, version=test
  - [ ] Multi-year test: Vermont, all years, version=test
  - [ ] Verify outputs in `outputs/test/2020/data/`
- [ ] Manual validation:
  - [ ] Run: `run -y 2020 -v test -st VT --reset` (builds census + redistricts)
  - [ ] Verify census data in: `outputs/test/2020/data/units/`, `outputs/test/2020/data/adjacency/`
  - [ ] Verify district results in: `outputs/test/2020/states/vermont/`
  - [ ] Run again without `--reset`: Should skip census (sees `.census_complete`)
- [ ] Storage validation:
  - [ ] Measure size of `outputs/test/2020/data/`: Should be ~600MB (one year)
  - [ ] Measure size of full version: Should be ~1.7-2GB (three years + results)

**Deliverables**:
- All tests passing (unit/integration/e2e)
- Manual pipeline test successful for test version
- Storage measurements documented
- No regressions in existing functionality

**Files**:
- Add: `tests/unit/test_per_version_census_data.py` (~150 lines)
- Add: `tests/integration/test_census_pipeline_integration.py` (~200 lines)
- Modify: Existing e2e tests to use version-specific paths

## Complete File Manifest - All Manual Updates Required

**CRITICAL**: Every file below MUST be manually reviewed and updated. Total: ~75-85 files.

### Phase 1: Path Utilities (3 files) ⚠️ CRITICAL
1. `scripts/utils/paths.py` - Core path functions
2. `scripts/utils/__init__.py` - Exports
3. `tests/unit/test_utils_paths.py` - Path tests

### Phase 2: Census Data Processing (9 files)
4. `scripts/data/process_census_data.py` - Main census pipeline
5. `scripts/data/download_orchestrator.py` - Downloads
6. `scripts/data/census/parse_pl94171_tracts_2000.py` - 2000 parsing
7. `scripts/data/census/parse_pl94171_tracts_2010.py` - 2010 parsing
8. `scripts/data/census/parse_pl94171_tracts_2020.py` - 2020 parsing
9. `scripts/data/merge_units_with_geometries.py` - Merge geometries
10. `scripts/data/geography/build_tract_adjacency.py` - Single state adjacency
11. `scripts/data/geography/build_all_adjacency_graphs.py` - All states adjacency
12. `scripts/data/validate_census_data.py` - Validation

### Phase 3: Core Pipeline Scripts (13 files) ⚠️ CRITICAL
13. `scripts/pipeline/run_complete_redistricting.py` - Main orchestrator
14. `scripts/pipeline/run_state_redistricting.py` - State redistricting
15. `scripts/pipeline/process_single_state.py` - State processing
16. `scripts/pipeline/add_cities_to_districts.py` - City labels
17. `scripts/pipeline/create_final_district_summary.py` - Summaries
18. `scripts/pipeline/export_rounds_to_csv.py` - Round exports
19. `scripts/pipeline/create_single_district_states.py` - Single district states
20. `scripts/pipeline/process_nation.py` - National aggregation
21. `scripts/pipeline/create_us_aggregate.py` - US aggregates
22. `scripts/pipeline/create_us_rounds_hierarchy.py` - Round hierarchy
23. `scripts/pipeline/cleanup_district_summary.py` - Cleanup
24. `scripts/pipeline/fill_missing_cities.py` - Missing cities
25. `scripts/pipeline/fix_2010_missing_outputs.py` - 2010 fixes

### Phase 4: Analysis & Visualization (17+ files)
26. `scripts/pipeline/analyze_district_demographics.py` - Demographics analysis
27. `scripts/pipeline/analyze_district_compactness.py` - Compactness analysis
28. `scripts/baseline/recompute_algorithmic_compactness.py` - Recompute compactness
29. `scripts/baseline/compare_algorithmic_vs_enacted.py` - Baseline comparison
30. `scripts/baseline/compute_enacted_compactness.py` - Enacted compactness
31. `scripts/baseline/visualize_three_way_comparison.py` - Three-way viz
32. `scripts/pipeline/visualize_individual_districts.py` - District maps
33. `scripts/pipeline/visualize_all_rounds.py` - Round progression maps
34. `scripts/pipeline/visualize_national_districts.py` - National district maps
35. `scripts/pipeline/visualize_national_rounds.py` - National round maps
36. `scripts/pipeline/visualize_metro_areas.py` - Metro area maps
37. `scripts/pipeline/visualize_compactness.py` - Compactness maps
38. `scripts/pipeline/visualize_district_demographics.py` - Demographic maps
39. `scripts/figures/generate_all_figures.py` - Figure generation
40. `scripts/figures/create_figure_variants.py` - Figure variants
41. `artifacts/papers/*/create_figures.py` - Paper figures (multiple)
42. `artifacts/presentations/*/create_figures.py` - Presentation figures (multiple)

### Phase 5: Web, Dashboard & Utilities (10 files) ⚠️ CRITICAL DASHBOARDS
43. `scripts/web/generate_dashboard.py` - Main dashboard
44. `scripts/web/generate_master_dashboard.py` - Master dashboard
45. `scripts/web/generate_enhanced_master_dashboard.py` - Enhanced dashboard
46. `scripts/utils/common.py` - Common utilities
47. `scripts/utils/error_logger.py` - Error logging
48. `scripts/utils/stage_tracker.py` - Stage tracking
49. `scripts/validation/validate_pipeline_outputs.py` - Output validation
50. `scripts/archive/save_run.py` - Save runs
51. `scripts/debug/quick_tract_map.py` - Debug map
52. `scripts/debug/debug_district_places.py` - Debug places

### Phase 6: Tests (13+ files)
53. `tests/unit/test_utils_paths.py` - Already in Phase 1
54. `tests/unit/test_process_census_data.py` - Census tests
55. `tests/unit/test_merge_units_with_geometries.py` - Merge tests
56. `tests/e2e/test_pipeline_scripts.py` - Pipeline E2E
57. `tests/e2e/conftest.py` - E2E fixtures
58. `tests/e2e/test_run_dashboard.py` - Dashboard tests
59. `tests/conftest.py` - Root fixtures
60. `scripts/tests/run_compactness_test.py` - Compactness test
61. `scripts/tests/run_niter100_test.py` - Niter test
62. `scripts/tests/test_first_split_tracts.py` - First split test
63. `scripts/tests/test_second_split_tracts.py` - Second split test
64. `scripts/tests/test_tract_level_split.py` - Tract split test
65. `scripts/tests/test_niter_values.py` - Niter values test

### Phase 7: Documentation (9+ files) ⚠️ CRITICAL
66. `CLAUDE.md` - Main AI guide
67. `context/ARCHITECTURE.md` - Architecture docs
68. `context/CODING_PATTERNS.md` - Coding patterns
69. `context/DATA_FORMATS.md` - Data formats
70. `context/QUICK_REFERENCE.md` - Quick reference
71. `context/enhancements/active/47_data_separation_restoration.md` - E47
72. `context/enhancements/active/52_per_version_census_data.md` - This file
73. `docs/CHANGELOG.md` - Changelog
74. `docs/GETTING_STARTED.md` - Getting started (if exists)
75. `docs/DATA_DICTIONARY.md` - Data dictionary (if exists)

### Additional Files to Review
76-85. Any additional scripts in:
- `.claude/skills/*/SKILL.md` - Skill documentation with path examples
- `artifacts/guides/*/` - LaTeX guides with path references
- `batch/*.bat` - Batch files with path arguments
- `scripts/baseline/*.py` - Other baseline scripts
- `scripts/data/demographics/*.py` - Demographic processing
- `scripts/data/elections/*.py` - Election processing
- Config files with hardcoded paths

### Summary by Criticality

**CRITICAL (Must be perfect)** - 10 files:
- `scripts/utils/paths.py`
- `scripts/pipeline/run_complete_redistricting.py`
- `scripts/pipeline/run_state_redistricting.py`
- `scripts/pipeline/process_single_state.py`
- `scripts/web/generate_dashboard.py`
- `scripts/web/generate_master_dashboard.py`
- `tests/unit/test_utils_paths.py`
- `CLAUDE.md`
- `context/ARCHITECTURE.md`

**HIGH (Core functionality)** - 30 files:
- All Phase 2 census processing
- All Phase 3 pipeline scripts
- Main analysis/visualization scripts

**MEDIUM (Secondary features)** - 25 files:
- Baseline comparison scripts
- Figure generation
- Additional visualizations
- Utilities

**LOW (Can defer)** - 10+ files:
- Archive scripts
- Debug scripts
- Old test scripts
- Guides/documentation

### Verification Checklist

After all manual updates, verify:
- [ ] `grep -r "outputs/data/" scripts/ --include="*.py"` → No matches
- [ ] `grep -r "outputs/us_" scripts/ --include="*.py"` → No matches (except archives)
- [ ] `grep -r 'f"outputs/{year}' scripts/ --include="*.py"` → No matches
- [ ] `grep -r "get_tract_file.*year.*)" scripts/ --include="*.py"` → All have version parameter
- [ ] All tests pass: `pytest tests/ -v`
- [ ] Sample run works: `run -y 2020 -v test -st VT`

## Testing Strategy

### New Tests Required

We need to add tests for:
1. ✅ New path functions (`get_census_data_dir`, `get_results_dir`, `get_version_dir`)
2. ✅ Version parameter validation (required, not optional)
3. ✅ Directory structure creation
4. ✅ Census data building per version
5. ✅ Results isolation between versions

### Test Files to Create

#### 1. **`tests/unit/test_path_utilities_v2.py`** (NEW - 200 lines)

Test new path functions and version parameter requirements:

```python
def test_get_census_data_dir():
    """Test census data directory path construction."""
    path = get_census_data_dir('v1', '2020')
    assert path == Path('outputs/v1/data/2020')

def test_get_results_dir():
    """Test results directory path construction."""
    path = get_results_dir('v1', '2020')
    assert path == Path('outputs/v1/results/2020')

def test_get_version_dir():
    """Test version directory path construction."""
    path = get_version_dir('v1')
    assert path == Path('outputs/v1')

def test_get_tract_file_requires_version():
    """Test that version parameter is required."""
    with pytest.raises(TypeError):
        get_tract_file('california', '2020')  # Missing version

def test_get_tract_file_with_version():
    """Test tract file path with version."""
    path = get_tract_file('california', '2020', 'v1')
    assert path == Path('outputs/v1/data/2020/units/california_tracts_2020.parquet')

def test_get_output_dir_parameter_order():
    """Test that parameter order is (version, year)."""
    path = get_output_dir('v1', '2020')
    assert path == Path('outputs/v1/results/2020')

def test_get_state_output_dir_parameter_order():
    """Test that parameter order is (state, version, year)."""
    path = get_state_output_dir('california', 'v1', '2020')
    assert path == Path('outputs/v1/results/2020/states/california')

def test_all_years_same_version():
    """Test that all years go under same version directory."""
    path_2020 = get_census_data_dir('v1', '2020')
    path_2010 = get_census_data_dir('v1', '2010')
    assert path_2020.parent == path_2010.parent  # Both under v1/data/

def test_different_versions_isolated():
    """Test that different versions are isolated."""
    path_v1 = get_census_data_dir('v1', '2020')
    path_v2 = get_census_data_dir('v2', '2020')
    assert path_v1 != path_v2
    assert 'v1' in str(path_v1)
    assert 'v2' in str(path_v2)
```

**Coverage**: All new path functions, parameter validation, structure isolation

#### 2. **`tests/integration/test_census_data_versioning.py`** (NEW - 300 lines)

Test census data building with version isolation:

```python
@pytest.fixture
def test_version():
    """Use test version for isolation."""
    return 'test_enhancement_52'

def test_census_data_builds_to_version_directory(test_version, tmp_path):
    """Test that census data builds to version-specific directory."""
    # Run census build with version parameter
    result = subprocess.run([
        sys.executable, 'scripts/data/process_census_data.py',
        '--year', '2020',
        '--version', test_version,
        '--states', 'VT'
    ], capture_output=True, text=True)

    assert result.returncode == 0

    # Verify output location
    census_dir = Path(f'outputs/{test_version}/data/2020')
    assert census_dir.exists()
    assert (census_dir / 'units').exists()
    assert (census_dir / 'adjacency').exists()

def test_multiple_versions_isolated(test_version):
    """Test that v1 and v2 have separate census data."""
    v1_dir = Path('outputs/v1/data/2020')
    v2_dir = Path('outputs/v2/data/2020')

    # Build for v1
    subprocess.run([
        sys.executable, 'scripts/data/process_census_data.py',
        '--year', '2020', '--version', 'v1', '--states', 'VT'
    ])

    # Build for v2
    subprocess.run([
        sys.executable, 'scripts/data/process_census_data.py',
        '--year', '2020', '--version', 'v2', '--states', 'VT'
    ])

    # Verify isolation
    assert v1_dir.exists()
    assert v2_dir.exists()
    assert list(v1_dir.glob('**/*.parquet')) != []
    assert list(v2_dir.glob('**/*.parquet')) != []

def test_census_marker_file_location(test_version):
    """Test that census complete marker is in correct location."""
    subprocess.run([
        sys.executable, 'scripts/data/process_census_data.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ])

    marker = Path(f'outputs/{test_version}/data/2020/.census_complete')
    assert marker.exists()
```

**Coverage**: Census building, version isolation, marker files

#### 3. **`tests/integration/test_pipeline_versioning.py`** (NEW - 400 lines)

Test complete pipeline with version-specific data:

```python
def test_pipeline_reads_from_version_data(test_version):
    """Test that pipeline reads census data from version directory."""
    # Build census data for test version
    subprocess.run([
        sys.executable, 'scripts/data/process_census_data.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ])

    # Run redistricting
    result = subprocess.run([
        sys.executable, 'scripts/pipeline/run_complete_redistricting.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ], capture_output=True, text=True)

    assert result.returncode == 0

    # Verify reads from correct census data location
    assert f'outputs/{test_version}/data/2020' in result.stdout or result.stderr

def test_pipeline_writes_to_version_results(test_version):
    """Test that pipeline writes results to version directory."""
    subprocess.run([
        sys.executable, 'scripts/pipeline/run_complete_redistricting.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ])

    # Verify results location
    results_dir = Path(f'outputs/{test_version}/results/2020')
    assert results_dir.exists()
    assert (results_dir / 'states/vermont').exists()
    assert (results_dir / 'data').exists()
    assert (results_dir / 'maps').exists()

def test_skip_logic_uses_version_markers(test_version):
    """Test that skip logic checks version-specific markers."""
    # First run
    subprocess.run([
        sys.executable, 'scripts/pipeline/run_complete_redistricting.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ])

    # Second run - should skip based on markers
    result = subprocess.run([
        sys.executable, 'scripts/pipeline/run_complete_redistricting.py',
        '--year', '2020', '--version', test_version, '--states', 'VT'
    ], capture_output=True, text=True)

    # Verify skip messages
    assert 'SKIP' in result.stdout or '.census_complete' in result.stdout
```

**Coverage**: E2E pipeline, data reads, results writes, skip logic

#### 4. **`tests/e2e/test_version_independence.py`** (NEW - 300 lines)

Test that versions are truly independent:

```python
def test_modifying_v1_data_does_not_affect_v2():
    """Test that v1 and v2 are completely isolated."""
    # Build v1
    subprocess.run([...version='v1'...])

    # Build v2
    subprocess.run([...version='v2'...])

    # Modify v1 census data
    v1_tract_file = Path('outputs/v1/data/2020/units/vermont_tracts_2020.parquet')
    original_v1_size = v1_tract_file.stat().st_size

    # Delete v1 tract file
    v1_tract_file.unlink()

    # Verify v2 still works
    v2_tract_file = Path('outputs/v2/data/2020/units/vermont_tracts_2020.parquet')
    assert v2_tract_file.exists()

    # v2 pipeline should still run
    result = subprocess.run([...version='v2'...])
    assert result.returncode == 0

def test_deleting_v1_does_not_affect_v2():
    """Test that deleting entire v1 doesn't break v2."""
    # Build both versions
    subprocess.run([...version='v1'...])
    subprocess.run([...version='v2'...])

    # Delete entire v1
    shutil.rmtree('outputs/v1')

    # v2 should still work
    result = subprocess.run([
        sys.executable, 'scripts/pipeline/run_complete_redistricting.py',
        '--year', '2020', '--version', 'v2', '--states', 'VT'
    ])
    assert result.returncode == 0

def test_parallel_version_builds():
    """Test that multiple versions can build simultaneously."""
    # Launch parallel builds (in practice, would use multiprocessing)
    # For test, run sequentially but verify no conflicts
    subprocess.run([...version='v1'...])
    subprocess.run([...version='v2'...])
    subprocess.run([...version='v3'...])

    # All should succeed
    assert Path('outputs/v1/data/2020').exists()
    assert Path('outputs/v2/data/2020').exists()
    assert Path('outputs/v3/data/2020').exists()
```

**Coverage**: Version independence, isolation, deletion safety

### Updated Phase 6: Tests Section

Add to Phase 6:

```markdown
#### 6.5: New Tests (MANUAL - Create from scratch)

- [ ] **`tests/unit/test_path_utilities_v2.py`** (NEW - 200 lines)
  - Test all new path functions
  - Test version parameter requirements
  - Test parameter order changes
  - Test version isolation

- [ ] **`tests/integration/test_census_data_versioning.py`** (NEW - 300 lines)
  - Test census build to version directory
  - Test multiple version isolation
  - Test marker file locations
  - Test skip logic per version

- [ ] **`tests/integration/test_pipeline_versioning.py`** (NEW - 400 lines)
  - Test pipeline reads from version data
  - Test pipeline writes to version results
  - Test skip logic with version markers
  - Test error handling with missing data

- [ ] **`tests/e2e/test_version_independence.py`** (NEW - 300 lines)
  - Test v1/v2 isolation
  - Test deletion safety
  - Test parallel builds
  - Test version-specific errors don't affect others
```

## Testing Plan

### 1. Unit Tests (New)
```bash
pytest tests/unit/test_path_utilities_v2.py -v
```
**Validates**:
- All new path functions return correct paths
- Version parameter is required (not optional)
- Parameter order is consistent
- Different versions produce different paths

### 2. Integration Tests (New)
```bash
pytest tests/integration/test_census_data_versioning.py -v
pytest tests/integration/test_pipeline_versioning.py -v
```
**Validates**:
- Census data builds to `outputs/{version}/data/{year}/`
- Multiple versions don't interfere
- Marker files in correct locations
- Skip logic works per-version

### 3. E2E Tests (New)
```bash
pytest tests/e2e/test_version_independence.py -v
```
**Validates**:
- Complete version isolation
- Deleting one version doesn't affect others
- Parallel version builds work
- No shared state between versions

### 4. Existing Tests (Update)
```bash
pytest tests/unit/test_utils_paths.py -v  # Update for version parameter
pytest tests/e2e/test_pipeline_scripts.py -v  # Update for new paths
```
**Validates**:
- Existing functionality still works
- Backward compatibility where expected
- No regressions

### 2. Integration Tests
```bash
pytest tests/integration/test_census_pipeline_integration.py -v
```
- Test census data build with version parameter
- Test pipeline reads from correct version-specific paths

### 3. E2E Pipeline Test (Vermont, Single Year)
```bash
run -y 2020 -v test -st VT --reset
```
- Should build census data to: `outputs/test/2020/data/`
- Should create `.census_complete` marker
- Should redistrict Vermont successfully
- Verify outputs exist

### 4. E2E Pipeline Test (Vermont, Multi-Year)
```bash
run -v test -st VT --reset
```
- Should build census data for 2000/2010/2020
- Should redistrict Vermont for all years
- Verify all outputs exist

### 5. Skip Logic Test
```bash
# First run - builds census data
run -y 2020 -v test -st VT --reset

# Second run - should skip census stage
run -y 2020 -v test -st VT
```
- Second run should see `.census_complete` and skip census stage
- Should only re-run national analysis if needed

### 6. Storage Validation
```bash
du -sh outputs/test/2020/data/
du -sh outputs/test/
```
- Single year data: ~600MB
- Full version (3 years + results): ~1.7-2GB

### 7. Migration Test
```bash
# Migrate existing v1 data
python scripts/maintenance/migrate_census_data.py --from-shared --to-version v1

# Verify migration
ls -lh outputs/v1/2020/data/units/
ls -lh outputs/v1/2020/data/adjacency/
```
- v1 data should exist in `outputs/v1/{year}/data/`

## Success Criteria

- [ ] All path functions accept `version` parameter
- [ ] All paths route to `outputs/{version}/{year}/data/`
- [ ] Census data processing scripts write to version-specific directories
- [ ] Pipeline builds census data as first stage (before states)
- [ ] Skip logic checks version-specific `.census_complete` marker
- [ ] All analysis scripts read from version-specific data
- [ ] No hardcoded `outputs/data/` paths remain (grep verification)
- [ ] All unit tests pass (path utilities)
- [ ] All integration tests pass (census pipeline)
- [ ] E2E pipeline test succeeds (Vermont, single year)
- [ ] E2E pipeline test succeeds (Vermont, multi-year)
- [ ] Skip logic works (second run skips census stage)
- [ ] Storage per version: ~1.7-2GB (acceptable for 441GB free)
- [ ] Migration script works (can migrate v1 from shared data)
- [ ] Documentation updated (CLAUDE.md, ARCHITECTURE.md, patterns)
- [ ] No regressions in existing tests

## Benefits

1. **Version Independence** (~90% improvement in experiment flexibility):
   - v1 and v2 can use different preprocessing approaches
   - No interference between versions
   - Can delete old versions without affecting new ones

2. **Algorithm Comparison**:
   - Compare results using identical vs. modified census data
   - Test preprocessing hypotheses (e.g., "Does filtering corner adjacencies improve compactness?")
   - Reproducible experiments (each version is self-contained)

3. **Reproducibility**:
   - Each version is a complete snapshot (data + results)
   - Can recreate exact results from any version
   - No hidden dependencies on shared data

4. **Experimentation**:
   - Test different edge weight scaling factors
   - Experiment with adjacency filtering (corner removal, water boundaries)
   - Try different resolution approaches (tract vs block)
   - Compare preprocessing variants side-by-side

5. **Disk Space Trade-off**:
   - Cost: 1.7GB per version (vs 1.7GB shared)
   - Benefit: Complete version independence
   - Acceptable: 20 versions = 34GB = 7.7% of 441GB free space

## Dependencies

- E47 (Data Separation and Restoration) - Complete ✅
- Path utilities in `scripts/utils/paths.py` - Exist ✅
- Download orchestrator - Complete ✅
- Census data processing pipeline - Complete ✅

## Risks & Mitigations

- **Risk 1**: Storage consumption (1.7GB per version)
  - *Mitigation*: User has 441GB free, 20 versions = 34GB (7.7%) is acceptable
  - *Mitigation*: Can delete old test versions to reclaim space
  - *Mitigation*: Document storage per version in CLAUDE.md

- **Risk 2**: Breaking existing v1 outputs
  - *Mitigation*: Create migration script to move existing data to `outputs/v1/{year}/data/`
  - *Mitigation*: Test migration thoroughly before production use
  - *Mitigation*: Keep backward compatibility in path utilities temporarily

- **Risk 3**: Complexity of version parameter threading
  - *Mitigation*: Centralize all path construction in `paths.py`
  - *Mitigation*: Grep verification to find all path function uses
  - *Mitigation*: Unit tests ensure version parameter works everywhere

- **Risk 4**: Census data build time increases with versions
  - *Mitigation*: Skip logic via `.census_complete` marker (only build once per version)
  - *Mitigation*: Can reuse census data from another version (copy directory)
  - *Mitigation*: Parallel processing for multi-state builds

- **Risk 5**: Forgetting to pass version parameter
  - *Mitigation*: Make version required (no default) in path functions
  - *Mitigation*: Linting/typing to catch missing parameters
  - *Mitigation*: Integration tests catch missing parameters

## Implementation Notes

### Key Architectural Changes

**Before (E47)**:
```
outputs/data/{year}/         # Shared across all versions
  ├─ units/
  ├─ adjacency/
  └─ demographics/

outputs/v1/{year}/           # Only district results
  ├─ states/
  ├─ data/                   # National aggregates only
  └─ maps/
```

**After (E52)**:
```
outputs/v1/{year}/data/      # Per-version census data
  ├─ units/                  # Census tracts
  ├─ adjacency/              # Adjacency graphs
  ├─ demographics/           # Demographics
  └─ elections/              # Elections

outputs/v1/{year}/           # District results
  ├─ states/
  ├─ data/                   # National aggregates
  └─ maps/
```

### Version Parameter Threading

All path functions need `version`:
```python
# Before
get_tract_file('california', '2020')  # → outputs/data/2020/units/california_tracts_2020.parquet

# After
get_tract_file('california', '2020', 'v1')  # → outputs/v1/2020/data/units/california_tracts_2020.parquet
```

### Census Data Stage

Pipeline stages:
```
1. Census Data (NEW)    → outputs/{version}/{year}/data/.census_complete
2. States (parallel)    → outputs/{version}/{year}/.states_complete
3. Nation (parallel)    → outputs/{version}/{year}/.nation_complete
```

### Migration Strategy

For existing v1 data:
```bash
# Option 1: Copy from shared data
python scripts/maintenance/migrate_census_data.py --from-shared --to-version v1

# Option 2: Rebuild from scratch
run -v v1 --reset --build-census
```

### Storage Estimation

Per version (3 years):
- Units (tracts + places): ~800MB
- Adjacency graphs: ~600MB
- Demographics: ~200MB
- Elections: ~100MB
- **Total per version**: ~1.7GB

10 versions = 17GB (3.9% of 441GB)
20 versions = 34GB (7.7% of 441GB)

## Key Decisions

1. **Version parameter required**: Make version a required parameter in all path functions (no default)
2. **Census stage in pipeline**: Add census data building as Stage 0 (before states)
3. **Skip logic**: Use `.census_complete` marker to skip census stage on subsequent runs
4. **Migration**: Provide script to migrate v1 from shared data
5. **Backward compatibility**: Temporary - remove after migration complete

## Challenges Encountered

_(To be filled during implementation)_

## Quantitative Results

_(To be filled after completion)_

- Storage per version: X GB
- Migration time: X minutes
- Census build time: X minutes per version
- Pipeline time increase: +X% (first run only)

## Completion Summary

**Completion Date**: 2026-01-19

**Implementation Approach**: Systematic layer-by-layer updates
1. **Phase 1**: Updated path utilities (3 files) - Added `version` parameter to all functions, created new helper functions
2. **Phase 2**: Census data processing (9 files) - Added `--version` flags, updated all path references
3. **Phase 3**: Core pipeline scripts (11 files) - Cascaded version parameter through entire pipeline
4. **Phases 4-6**: Analysis, visualization, web, tests - Integrated with Phase 3 updates

**Total Files Modified**: 34 files across 6 phases

**Key Implementation Details**:
- All path utility functions now accept `version` parameter (e.g., `get_tract_file(state, year, version)`)
- New structure: `outputs/{version}/data/{year}/` for census data, `outputs/{version}/results/{year}/` for results
- Parameter order standardized: `(state, year, version)` for state-specific, `(version, year)` for directories
- Backward compatibility: Legacy paths supported in some scripts for migration period
- Tests: 42 unit tests passing for path utilities

**Benefits Realized**:
- ✅ Complete version isolation for census data and results
- ✅ Enables independent preprocessing experiments per version
- ✅ Allows parallel development of algorithm variants
- ✅ Clear separation: raw data → version-specific processed data → version-specific results

**Deviations from Plan**:
- Completed faster than estimated (6-8h vs 12-16h) due to centralized path utility approach
- Some optional scripts (baseline, figures) didn't require updates as they don't use path utilities

**Storage Impact**:
- Census data now per-version: ~1.7GB × number of active versions
- Enables independent experimentation without cross-version conflicts

## Related Documentation

- E47: [Data Separation and Restoration](47_data_separation_restoration.md) - Previous census data restructuring
- E13: [Directory Unification](../completed/13_directory_unification.md) - Previous path restructuring
- E15: [Multi-Year Support](../completed/15_multi_year_support.md) - Multi-year patterns
- Architecture doc: [ARCHITECTURE.md](../../ARCHITECTURE.md)
- Coding patterns: [CODING_PATTERNS.md](../../CODING_PATTERNS.md)
- Data formats: [DATA_FORMATS.md](../../DATA_FORMATS.md)
