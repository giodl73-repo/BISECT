# Enhancement 36: Experimental Variants Configuration System

**Status**: ✅ COMPLETED
**Priority**: High
**Estimated Complexity**: Medium-High (6-10 hours)
**Actual Time**: ~7 hours
**Created**: January 17, 2026
**Started**: January 17, 2026
**Completed**: January 17, 2026
**Commits**: [eaf579b](https://github.com/giodl_microsoft/redistricting/commit/eaf579be099723453406b4a6d6cc578ec1afd6bb), [461edd2](https://github.com/giodl_microsoft/redistricting/commit/461edd2cb4700c2542084388733ac0640410134a), [e30792c](https://github.com/giodl_microsoft/redistricting/commit/e30792c69ab0f4032479687dd9d062dd8f0fba97), [b6f90b8](https://github.com/giodl_microsoft/redistricting/commit/b6f90b82ac3b87d07c3b1112ee8e2451b43fa83e), [18c5fe5](https://github.com/giodl_microsoft/redistricting/commit/18c5fe516e72b94d68681dfa888b54f9a8ebb59d), [1de416c](https://github.com/giodl_microsoft/redistricting/commit/1de416ca4bea97fd6081c0fc8505f8223f1dbc07)
**Size**: XL - 13,470 lines changed (84 files)

## Current State

The current output directory structure has several limitations:

**Directory Structure Issues:**
- Mixed experiments and production runs in flat `outputs/` directory
- Structure: `outputs/us_{year}_{version}/` with no clear organization
- Temporary test runs (e.g., `alabama_edge_test`, `iowa_edge_test`) clutter the outputs directory
- Artifacts and figures mixed with redistricting outputs
- No clear separation between experiments and production runs

**Configuration Tracking Issues:**
- No metadata file tracking algorithmic choices in each run
- Cannot determine from output alone whether run used:
  - Tract vs block level data
  - Edge-weighted vs unweighted mode
  - Other parameter choices
- Dashboard doesn't display experiment configuration
- Difficult to compare variants systematically

**Example Current Structure (OLD - to be replaced):**
```
outputs/
  us_2020_v1/                      # Edge-weighted production run
  us_2020_v1_noedge/              # Unweighted variant (_noedge suffix - BAD)
  alabama_edge_test/               # Experiment (mixed with production)
  iowa_edge_test/                  # Experiment
  baseline_comparison/             # Analysis outputs
  artifacts/                       # Compiled PDFs only
  figures/                         # Should be under artifacts/
  comparison.json                  # Loose files
  edge_weighted_comparison_50states.csv
  index.html                       # Master dashboard
```

**Problems with Current Approach:**
1. `_noedge` suffix encodes algorithm choice in directory name (not scalable)
2. No way to track other parameters (tract vs block, edge weight scaling, etc.)
3. Mixed production, experiments, and test runs in flat structure

## Goal

Create a comprehensive experimental variants configuration system that:

1. **Tracks all algorithmic choices** via `config.json` in each output directory
2. **Reorganizes output directory structure** to separate production, experiments, and dev runs
3. **Hides dev/debug runs** from main dashboard and directory listings
4. **Displays experiment settings** prominently on dashboard
5. **Enables systematic variant comparison** for future experiments (tract vs block, parameter sweeps, etc.)
6. **Cleans up cluttered outputs directory** with clear organizational hierarchy

## Proposed Output Structure

### Option A: Version-First Hierarchy (User Preference)
```
outputs/
  v1/                              # Production version directory
    2020/                          # Year subdirectory
      config.json                  # Run configuration metadata
      states/
      maps/
      data/
      index.html
    2010/
      config.json
      ...
    2000/
      config.json
      ...
  experiments/                     # Experimental runs (research variants)
    tract_vs_block_2020/
      tract_v1/
        config.json
        ...
      block_v1/
        config.json
        ...
    edge_weight_comparison_2020/
      weighted_v1/
        config.json
        ...
      unweighted_v1/
        config.json
        ...
  dev/                             # Development/validation runs (hidden from UI)
    test_2020_vermont/
      config.json
      ...
    debug_alabama_edge/
      config.json
      ...
  artifacts/                       # All academic outputs
    papers/                        # Compiled PDFs
    presentations/                 # Compiled PDFs
    guides/                        # Compiled PDFs
    figures/                       # Generated figures (used by LaTeX)
  master_index.html                # Master dashboard
```

### Option B: Category-First (Alternative)
```
outputs/
  production/
    us_2020_v1/
      config.json
      ...
    us_2010_v1/
    us_2000_v1/
  experiments/
    [same as Option A]
  dev/                             # Development/validation runs (hidden from UI)
    test_2020_vermont/
    debug_alabama_edge/
  artifacts/                       # All academic outputs
    papers/
    presentations/
    guides/
    figures/
  master_index.html
```

**Recommendation:** Option A (version-first) as user indicated preference for "version on top and census years one directory below"

## Configuration File Schema

### config.json Format
```json
{
  "metadata": {
    "created": "2026-01-17T14:30:00",
    "version": "v1",
    "census_year": 2020,
    "election_year": 2020,
    "run_type": "production",  // "production", "experiment", or "test"
    "scope": "us",             // "us" for 50-state run, "state" for single state
    "states": ["all"],         // ["all"] or specific states like ["california", "texas"]
    "experiment_name": null,   // e.g., "tract_vs_block_2020" for experiments
    "description": "Production run with edge-weighted algorithm"
  },
  "algorithm": {
    "partition_mode": "edge_weighted",
    "data_level": "tract",
    "compactness_metric": "polsby_popper",
    "population_tolerance": 0.005
  },
  "pipeline": {
    "skip_political": false,
    "skip_demographic": false,
    "skip_compactness": false,
    "skip_metro": false,
    "dpi": 150
  },
  "data_sources": {
    "tract_shapefile": "data/tracts/2020/*.parquet",
    "election_data": "data/election_data/2020/*.csv",
    "demographic_data": "data/census/2020/*.parquet"
  },
  "system": {
    "python_version": "3.13.1",
    "metis_version": "5.1.0",
    "execution_time_seconds": 7320,
    "hostname": "DESKTOP-XXXXX"
  }
}
```

### Future Extensibility

The schema supports future experimental variants:
```json
{
  "algorithm": {
    "partition_mode": "edge_weighted",
    "data_level": "block",              // NEW: tract vs block
    "edge_weight_scaling": 100,         // NEW: parameter sweep
    "minimum_tract_population": 1000,   // NEW: data filtering
    "custom_edge_weights": false        // NEW: future experiments
  }
}
```

## Implementation Plan

### Phase 1: Define Configuration Schema and Writer
- [ ] Create `src/apportionment/config/run_config.py` module
- [ ] Define `RunConfig` dataclass with all fields
- [ ] Implement `write_config()` function to generate config.json
- [ ] Implement `read_config()` function to load config.json
- [ ] Add validation logic for required fields
- [ ] Write unit tests for config serialization/deserialization

**Files:**
- `src/apportionment/config/__init__.py` - New module
- `src/apportionment/config/run_config.py` - Config dataclass and I/O
- `tests/unit/test_run_config.py` - Unit tests

### Phase 2: Implement New Output Directory Structure
- [ ] Wipe `outputs/` directory (user has backup)
- [ ] Create new directory structure: `outputs/v1/`, `outputs/experiments/`, `outputs/dev/`, `outputs/artifacts/`
- [ ] Move `artifacts/` contents from repo root to `outputs/artifacts/`
- [ ] Update `.gitignore` to reflect new structure (exclude `outputs/dev/` from git)
- [ ] Create `outputs/README.md` documenting new structure
- [ ] Update figure generation scripts to use `outputs/artifacts/figures/`

**Files:**
- `outputs/README.md` - Structure documentation
- `.gitignore` - Update output patterns
- `scripts/figures/generate_all_figures.py` - Update to outputs/artifacts/figures/
- `scripts/figures/create_figure_variants.py` - Update to outputs/artifacts/figures/

### Phase 3: Update Pipeline Scripts to Write Config
- [ ] Modify `run_complete_redistricting.py` to write config.json
- [ ] Update output directory path logic (support new structure)
- [ ] **Remove `_noedge` suffix logic** (partition mode now in config.json, not directory name)
- [ ] New path logic: `outputs/{run_type}/{version}/{year}/` or `outputs/experiments/{experiment_name}/`
- [ ] Add `--run-type` parameter (production/experiment/test)
- [ ] Add `--experiment-name` parameter for experiments
- [ ] Auto-detect test runs (states like VT/DE with version "test")
- [ ] Write config.json at start of pipeline (with initial metadata)
- [ ] Update config.json at end with timing/system info
- [ ] **Handle single-state runs**: If running standalone state (not part of US run), create `outputs/dev/{state}_{year}_{timestamp}/config.json`
- [ ] Modify `process_single_state.py` to write config.json when run standalone
- [ ] Update all analysis scripts to read config if needed

**Files:**
- `scripts/pipeline/run_complete_redistricting.py` - Config generation, remove `_noedge` logic (line 363)
- `scripts/pipeline/process_single_state.py` - Write config.json for standalone state runs
- All analysis scripts that reference output directories

### Phase 4: Update Dashboard to Display Configuration
- [ ] Modify `scripts/web/generate_dashboard.py` to read config.json
- [ ] Add "Run Configuration" section to dashboard
- [ ] Display algorithm settings (partition mode, data level)
- [ ] Display pipeline settings (skip flags, DPI)
- [ ] Add metadata section (created date, version, year)
- [ ] Update master dashboard to show production and experiment runs (exclude tests/)
- [ ] Add config comparison view (side-by-side)
- [ ] Add run_type badge/label (production/experiment/test)

**Files:**
- `scripts/web/generate_dashboard.py` - Read and embed config
- `web/dashboard.html` - Display config section
- `scripts/web/generate_master_dashboard.py` - Multi-run config view
- `web/master_dashboard.html` - Config comparison table

### Phase 5: Update Batch Files and CLI
- [ ] Update `run_redistricting.bat` with new parameters
- [ ] Update `deploy_web.bat` to handle new directory structure
- [ ] Add `--output-base` parameter to specify output root
- [ ] Update help text with new structure examples
- [ ] Create `create_experiment.bat` helper script

**Files:**
- `run_redistricting.bat` - New parameters
- `deploy_web.bat` - New path logic
- `create_experiment.bat` - Helper for experiments

### Phase 6: Update Existing Tests for New Structure
- [ ] Update pipeline test fixtures to expect new directory structure
- [ ] Modify `tests/e2e/test_pipeline.py` to use new paths (v1/2020/ instead of us_2020_v1/)
- [ ] Update mock data generation to create config.json files
- [ ] Add test for config.json validation (schema, required fields)
- [ ] Add test for run type detection (production/experiment/test)
- [ ] Add test for directory path logic (ensure correct routing)
- [ ] Update dashboard tests to expect config display section
- [ ] Update any other tests that hardcode output paths

**Files:**
- `tests/e2e/test_pipeline.py` - Update for new structure
- `tests/fixtures/generate_mock_run.py` - Generate config.json
- `tests/unit/test_run_config.py` - New tests for config module
- Any other test files referencing output paths

### Phase 7: Integration Testing and Validation
- [ ] Test print-only mode with new structure
- [ ] Run small state test (VT) with config generation
- [ ] Verify config.json is written correctly
- [ ] Test dashboard displays config correctly
- [ ] Run experiment variant (edge-weighted vs unweighted comparison)
- [ ] Test master dashboard with multiple configs
- [ ] Verify LaTeX compilation still works from new artifacts location

### Phase 8: Documentation
- [ ] Update `ARCHITECTURE.md` with new output structure
- [ ] Update `CODING_PATTERNS.md` with config usage
- [ ] Create `docs/EXPERIMENTS.md` guide for running variants
- [ ] Update `README.md` with new directory structure
- [ ] Update `CLAUDE.md` with config patterns
- [ ] Update `artifacts/compile.bat` to work from new location

## Files to Modify/Create

### New Files:
1. `src/apportionment/config/__init__.py` - Config module initialization
2. `src/apportionment/config/run_config.py` - RunConfig dataclass and I/O functions
3. `outputs/README.md` - Documentation of new directory structure
4. `docs/EXPERIMENTS.md` - Guide for running experimental variants
5. `tests/unit/test_run_config.py` - Unit tests for config module
6. `create_experiment.bat` - Helper script for experiment setup

### Modified Files:
1. `scripts/pipeline/run_complete_redistricting.py` - Generate config.json, new directory logic
2. `scripts/web/generate_dashboard.py` - Read and display config, update figures path
3. `scripts/web/generate_master_dashboard.py` - Multi-config comparison, update figures path
4. `scripts/figures/generate_all_figures.py` - Update output path to artifacts/figures/
5. `scripts/figures/create_figure_variants.py` - Update output path to artifacts/figures/
6. `web/dashboard.html` - Add config display section
7. `web/master_dashboard.html` - Add config comparison table
8. `run_redistricting.bat` - New parameters (--run-type, --experiment-name)
9. `deploy_web.bat` - Handle new directory structure
10. `.gitignore` - Update output directory patterns
11. `ARCHITECTURE.md` - Document new structure
12. `CODING_PATTERNS.md` - Config usage patterns
13. `README.md` - Update directory structure documentation
14. `CLAUDE.md` - Update with config patterns

## Testing Plan

### 1. Config Generation Test
```bash
# Test config.json creation
python -c "from apportionment.config.run_config import RunConfig, write_config; ..."
```

### 2. Print-Only Mode Test
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --print-only --run-type experiment --experiment-name test_config
```

### 3. Small State Test with New Structure
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states VT --run-type experiment --experiment-name test_variant
```

Verify:
- [ ] Config.json exists at `outputs/experiments/test_variant/test_2020/config.json`
- [ ] Config contains correct metadata
- [ ] All fields populated correctly

### 4. Dashboard Config Display Test
```bash
python scripts/web/generate_dashboard.py --year 2020 --version test --output-dir outputs/experiments/test_variant/test_2020
```

Verify:
- [ ] Dashboard shows "Run Configuration" section
- [ ] Algorithm settings displayed correctly
- [ ] Metadata displayed correctly

### 5. Multi-Year Test
```bash
# Test with 2020, 2010, 2000
python scripts/pipeline/run_complete_redistricting.py --year 2010 --version test --states DE --run-type production
```

### 6. Experiment Variant Test
```bash
# Test tract vs block comparison
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states VT --run-type experiment --experiment-name tract_vs_block --data-level tract

python scripts/pipeline/run_complete_redistricting.py --year 2020 --version test --states VT --run-type experiment --experiment-name tract_vs_block --data-level block
```

Compare configs side-by-side in master dashboard.

## Success Criteria

- [ ] Every pipeline run generates a valid config.json file
- [ ] Config.json contains all required fields (metadata, algorithm, pipeline, system)
- [ ] New output directory structure is clean and organized (v1/, experiments/, tests/, artifacts/)
- [ ] Production, experiments, and test runs properly separated
- [ ] Test runs excluded from master dashboard and main listings
- [ ] Dashboard displays configuration prominently
- [ ] Master dashboard allows config comparison
- [ ] Documentation updated (ARCHITECTURE.md, EXPERIMENTS.md, README.md)
- [ ] Config schema is extensible for future variants
- [ ] LaTeX compilation works from new artifacts/ location

## Benefits

1. **Reproducibility**: Every run fully documented with algorithmic choices
2. **Experiment Organization**: Clear separation of production, experiments, and tests
3. **Clean Development**: Test/debug runs hidden from main UI and directory listings
4. **Systematic Comparison**: Easy to compare variants side-by-side
5. **Dashboard Transparency**: Users can see exactly what settings were used
6. **Future-Proof**: Schema supports future experimental parameters (tract vs block, parameter sweeps)
7. **Clean Structure**: Organized outputs directory with clear hierarchy
8. **Version Flexibility**: Can maintain multiple production versions simultaneously
9. **Audit Trail**: Complete record of what was run when

## Dependencies

- None (self-contained enhancement)

## Risks & Mitigations

### Risk 1: Config schema needs frequent updates as experiments evolve
- **Mitigation**:
  - Design extensible schema with "custom_parameters" dict
  - Version the config schema itself (schema_version: 1.0)
  - Make all fields except core metadata optional
  - Document schema evolution in EXPERIMENTS.md

### Risk 2: Large refactor touches many files
- **Mitigation**:
  - Implement incrementally (Phase 1-7)
  - Test after each phase
  - Use feature flags to toggle new behavior
  - Can rollback individual phases if needed

### Risk 3: User preference for directory structure may change
- **Mitigation**:
  - Make structure configurable via environment variable
  - Support multiple structure patterns
  - Document structure rationale
  - Easy to adjust since starting fresh (no migration)

## Implementation Notes

### Single-State Runs

When running a single state independently (not part of full US pipeline):
```bash
python scripts/pipeline/process_single_state.py --state california --year 2020 --version test
```

Output structure:
```
outputs/dev/california_2020_20260117_143052/
  config.json              # State-level config
  states/
    california/
      data/
      maps/
      ...
  index.html              # State dashboard
```

Config includes `"scope": "state"` field to differentiate from US-level runs.

### Directory Structure Decision

**User Feedback:**
> "perhaps i would prefer the version be on top and the census years be one directory below. but i can live with it as need be."

This suggests **Option A** (version-first hierarchy):
```
outputs/v1/2020/
outputs/v1/2010/
outputs/v1/2000/
```

Rather than year-first:
```
outputs/us_2020_v1/
outputs/us_2010_v1/
outputs/us_2000_v1/
```

**Rationale:**
- Easier to compare same version across years
- Cleaner organization as versions evolve
- Separates version iteration from census year
- Matches user preference

### Critical User Quote

> "if we are going to change the outputdir structure lets do it now. it is super cluttered and we also generate artifacts and other things."

This enhancement should be done **before** running tract vs block experiments to avoid migrating experiments later.

### Config Generation Timing

Config should be written in **two phases**:
1. **Start of pipeline**: Metadata, algorithm settings, pipeline flags (known upfront)
2. **End of pipeline**: System info, execution time, completion status

This allows:
- Debugging failed runs (config exists even if pipeline crashes)
- Progress tracking (can see what's running)
- Complete audit trail (final timing in config)

### Dashboard Integration

Config display should be **prominent and scannable**:
- Show at top of dashboard (above district summary)
- Use color coding for experiment vs production
- Highlight differences from defaults
- Link to config.json for raw file access

## Related Documentation

- Enhancement #13: [Directory Unification](13_directory_unification.md) - Previous structure refactoring
- Enhancement #29: [Artifacts Dashboard Tab](29_artifacts_dashboard_tab.md) - Dashboard enhancements
- Architecture doc: [ARCHITECTURE.md](../ARCHITECTURE.md)
- Coding patterns: [CODING_PATTERNS.md](../CODING_PATTERNS.md)
