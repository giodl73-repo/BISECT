# Changelog

All notable changes to the Congressional Redistricting project.

**Last Updated**: January 16, 2026

## Related Documentation

- **[Enhancement Index](enhancements/INDEX.md)** - Master index of all enhancements (detailed specifications and implementation plans)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and architectural decisions
- **[CODING_PATTERNS.md](CODING_PATTERNS.md)** - Implementation patterns

**Note**: This changelog tracks historical changes chronologically. For detailed enhancement specifications and future roadmap, see `enhancements/INDEX.md`.

## [Unreleased]

### Added
- Demographic analysis and visualization (3 map types per state)
- Political analysis with partisan lean maps
- Comprehensive directory READMEs for AI assistant context
- DPI configuration threading through all visualization scripts
- Progress message standardization across analysis scripts

### Changed
- **2026-01-15**: Algorithm formalization and figure quality improvements
  - Formalized recursive bisection algorithm with RBA (Recursive Bisection Algorithm) notation in laymen's guide
  - Added mathematical set notation for output: {(R₁, P₁, 1), ..., (Rₙ, Pₙ, n)} where P₁...Pₙ ≈ P/n
  - Updated recursive step to use union operator: RBA(R₁, P₁, x) ∪ RBA(R₂, P₂, y)
  - Changed Minnesota/Alabama examples from inline text to numbered lists (Round 1, 2, 3)
  - Increased all figure font sizes by 1 point for better readability (titles: 14→15, labels: 9→10, region labels: 11→12)
  - Fixed boundary_labels='none' logic to hide edge labels but still display compactness metrics
  - Removed redundant "Region 1/2" labels from graph panel when both map and graph panels shown
  - Moved water crossings explanation to footnote for improved text flow
  - Updated `generate_all_figures.py` to use 2010 census data (that's what we have downloaded)
  - Fixed `presentation.tex` to reference correct figure filename
- **2026-01-15**: Reorganized enhancement tracking into individual files
  - Split `ENHANCEMENTS_2026.md` (2,931 lines) into 18 individual files
  - Created `docs/enhancements/` directory structure with `completed/`, `active/`, and `templates/` subdirectories
  - Created master index at `docs/enhancements/INDEX.md`
  - Original `ENHANCEMENTS_2026.md` now serves as redirect pointer
  - Improved maintainability and navigation of enhancement documentation
- Improved progress bar UX with state-specific status messages
- Map boundaries: thin white tract lines + thick black district overlays
- Default DPI changed from 300 to 150 for better performance

### Fixed
- **2026-01-15**: Fixed dashboard district data loading paths
  - Updated `scripts/web/generate_dashboard.py` to look for `district_cities.csv` in correct `data/` subdirectory
  - Fixed all CSV download links in `web/dashboard.html` template to use correct paths
  - Districts tab now properly displays individual district maps for all states

## 2026-01-16 - Enhancement 29: Artifacts Dashboard Tab

### Changed
- Reorganized artifacts into top-level `artifacts/` directory
  - Moved `papers/`, `presentations/`, and `guides/` into `artifacts/`
  - Created master `artifacts/compile.bat` with flag threading
  - All compilation outputs to `outputs/artifacts/`
- Fixed all artifact script paths for 3-level directory hierarchy (../../ → ../../../)
  - Updated `presentation.tex` image paths
  - Updated `create_appendix_examples.py` and `create_figures.py` paths
  - Updated all `compile.bat` OUTPUT_DIR paths
- Implemented compilation flag threading
  - Added `--reset` flag threading through all compile.bat files
  - Added `--skip-figures` flag to suppress duplicate figure generation
  - Master compile.bat generates shared figures first (Phase 1)
  - Child scripts skip figure generation after Phase 1

### Fixed
- Fixed \\n appearing as literal text in map titles (changed \\\\n to \\n)
  - Applied fix across 15+ visualization scripts
  - Includes state maps, national maps, round progression, compactness, political, demographic
- Removed yellow stats boxes from all maps for cleaner visualization
- Reduced district label font sizes for better visual balance
  - 2-4 regions: 40 → 24 fontsize
  - 5-8 regions: 28 → 20 fontsize
- Fixed national rounds path in dashboard (`maps/rounds/round_XX.png`)

### Added
- Master dashboard restructured with 3-tab layout
  - **Overview Tab**: Clickable run cards with hover effects (no dropdown needed)
  - **Compactness Tab**: Moved all compactness analysis here (new dedicated tab)
  - **Artifacts Tab**: PDF viewer for papers, presentations, and guides
- Run card features:
  - Color-coded mode badges (green=edge-weighted, red=unweighted)
  - Year badges with distinct colors
  - Hover effects with lift and shadow
  - Direct navigation to run dashboards on click

### Files Modified
- 40+ files (artifacts/, scripts/, web/)
- Created `docs/enhancements/active/29_artifacts_dashboard_tab.md`

## 2026-01-15 - Enhancement 18: Figure Quality Improvement

### Changed
- Real census tract examples (appendix) now use strict validation criteria
- Ratio accuracy: Within 0.5% of target (was unvalidated)
- Compactness requirement: Both regions >= 0.25 Polsby-Popper score
- Retry logic: Up to 26 attempts to find optimal examples (25 retries)
- Region labels now positioned outside combined boundary (no overlap with tracts)
- Label positioning adapts to horizontal vs vertical splits

### Removed
- Prime number references from laymen_guide.tex (unnecessary complexity)
- Removed "(sum=X)" from subsection titles
- Removed "Mathematical structure" bullet point from summary

### Improved
- Tract count optimized: 12 tracts provides best balance of clarity and validation
- All 6 examples now meet strict validation criteria:
  - Phoenix: 0.00% error (perfect ratio match)
  - Minneapolis: 0.10% error
  - Los Angeles: 0.11% error
  - Atlanta: 0.15% error
  - Miami: 0.37% error
  - Houston: 0.47% error
- Compactness scores range from 0.25 to 0.42

### Files Modified
- `presentations/edge_weighted_bisection/create_appendix_examples.py` - Validation and retry logic
- `presentations/edge_weighted_bisection/laymen_guide.tex` - Documentation clarity

## 2026-01-14 - Enhancement 17: Artifact Naming Standardization

### Changed
- Standardized all artifact naming conventions across state and national outputs
- Removed year suffixes from filenames (year is in directory path)
- Renamed directories: `political_analysis/` → `political/`, `demographic_analysis/` → `demographic/`
- Organized CSVs in `data/` subdirectory, maps in `maps/` subdirectory
- Changed national maps: `US_National_Map_435_Districts_2020.png` → `maps/us_all_districts.png`
- Changed round maps: `round_1_2_regions.png` → `maps/rounds/round_01.png` (zero-padded)
- Changed district maps: `district_01_los_angeles.png` → `district_01.png` (no city slug)
- All filenames now use snake_case consistently

### Fixed
- Path migration bugs in compactness, political, and demographic analysis scripts
- Updated 7 analysis scripts to use new `data/` subdirectory structure
- Fixed output directory creation in political analysis

### Files Modified
- 16 generator scripts updated for new naming conventions
- 7 analysis scripts updated for new directory structure
- Validation script updated to reflect new paths
- Web dashboard updated to reference new filenames

## 2026-01-14 - Enhancement 15: Multi-Year Pipeline Support

### Added
- Full 2000 census data support (tract data, adjacency graphs)
- Enhanced skip logic for missing data (election data, metro maps)
- Year-specific handling throughout pipeline

### Changed
- Data paths support all three census years (2000, 2010, 2020)
- Validation framework tests all three years
- Pipeline gracefully skips unavailable data per year

## 2026-01-14 - Enhancement 14: Pipeline Output Validation Framework

### Added
- Comprehensive validation script: `scripts/validation/validate_pipeline_outputs.py`
- Validates all pipeline outputs for completeness
- Checks state-level and national-level artifacts
- Reports missing files, completion percentages
- Integrated into main pipeline with `--validate` flag

### Features
- Per-state validation tracking
- Per-stage completion reporting
- Detailed missing file identification
- Summary statistics per census year

## 2026-01-14 - Enhancement 13: Directory Unification

### Changed
- Unified year-specific paths into single directory structure
- Moved from `data/raw/{year}/` → `data/tracts/{year}/`
- Moved from `data/adjacency/{year}/` → `data/adjacency/{year}/` (standardized)
- Removed ~80 lines of conditional path logic
- Preserved intentional conditionals (config imports)

### Fixed
- Path inconsistencies across census years
- Manual editing used for safety on critical changes

## 2026-01-11 - Documentation Refactoring

### Added
- `CODING_PATTERNS.md` - Comprehensive coding patterns for AI assistants
- `ARCHITECTURE.md` - System design and algorithm documentation
- `scripts/data/README.md` - Data acquisition documentation
- `scripts/pipeline/README.md` - Pipeline orchestration guide
- `scripts/political/README.md` - Political and demographic analysis
- `src/apportionment/README.md` - Core library documentation

### Changed
- Progress messages now use "Analysis (x/50) - State Name" format consistently
- Capitalized "Analysis" in all progress reporting

## 2026-01-11 - Demographic Visualization

### Added
- `visualize_district_demographics.py` - Creates 3 demographic maps per state:
  - Gender balance map (male-leaning, female-leaning, balanced)
  - Majority race map (shows plurality demographic group)
  - Diversity index map (Shannon entropy-based measure)
- `run_demographic_visualization.py` - Batch processing for all 50 states
- Integrated demographic visualization into main pipeline

### Fixed
- GEOID type mismatch in demographic analysis (int64 vs object)
- Added missing `os` import in analyze_districts.py

## 2026-01-11 - DPI Performance Optimization

### Changed
- Added `--dpi` parameter to all visualization scripts:
  - `create_us_national_map.py`
  - `visualize_split.py`
  - `create_single_district_states.py`
- Updated `run_complete_redistricting.py` to pass `--dpi` to all children
- Changed default from hardcoded 300 to configurable 150

### Performance
- ~4x speedup for national map generation (15min → 3-5min at DPI 150)

## 2026-01-11 - Demographic Analysis Integration

### Added
- `analyze_district_demographics.py` - Calculate demographics per district
- `run_demographic_analysis.py` - Batch processing for all 50 states
- Integrated into main pipeline

### Fixed
- GEOID handling: Always use `.astype(str).str.zfill(11)` before merge
- Proper zero-padding for state codes (06 for CA, 01 for AL, etc.)

## 2026-01-10 - Pipeline Progress Improvements

### Changed
- Enhanced progress bar system with real-time STATUS message protocol
- Added state-by-state progress tracking (1/50, 2/50, etc.)
- Improved parent-child communication via TQDM_POSITION env variable
- Post-processing steps now show detailed progress:
  - "Waiting..." → "Starting..." → "Progress..." → "COMPLETE"

### Added
- `PROGRESS_BAR_GUIDE.md` - Documentation for progress bar protocol
- Subprocess monitoring with Popen for real-time updates

## 2026-01-10 - Script Organization

### Changed
- Reorganized scripts into logical subdirectories:
  - `scripts/data/` - Data acquisition and processing
  - `scripts/pipeline/` - Main redistricting pipeline
  - `scripts/political/` - Political and demographic analysis
- Updated all import paths and script references

### Added
- Skip logic for all visualization scripts (resume capability)
- Force flag (`--force`) to override skip logic

## 2026-01-10 - Map Boundary Improvements

### Changed
- Standardized map visualization pattern across all scripts:
  - Thin white tract boundaries (0.1pt)
  - Thick black district boundaries (1.5pt) as dissolve overlay
- Fixed title formatting (removed `\n` causing rendering issues)
- Consistent legend placement and styling

### Technical
- Used `dissolve(by='district')` to create district polygons
- Used `boundary.plot()` with `zorder=10` for overlay

## 2026-01-10 - Print-Only Mode

### Added
- `--print-only` flag for dry-run mode
- Shows all commands that would be executed
- No file operations or state changes
- Useful for debugging and verification

## 2026-01-10 - 2010 Census Pipeline

### Added
- Support for 2010 Census data processing
- `fix_2010_population.py` - Backfill 2010 population from PL 94-171 files
- Instructions for re-downloading 2010 data

### Changed
- Census year now configurable: `--year {2020|2010}`
- Version identifier support: `--version v1`, `--version v2`, etc.

## 2026-01-09 - Political Analysis

### Added
- `analyze_districts.py` - Calculate partisan lean per district
  - Biden/Trump two-party vote percentages
  - Democratic margin calculation
  - Lean classification (Strong D/Lean D/Toss-up/Lean R/Strong R)
- `visualize_partisan_lean.py` - Create partisan lean maps
  - Basic partisan lean map with district colors
  - Enhanced map with margins table
- `run_political_analysis.py` - Batch processing for all states

### Data
- MIT Election Lab 2020 presidential results
- Geocoded from precincts to census tracts

## 2026-01-09 - Compactness Metrics

### Added
- Polsby-Popper score calculation per district
- Reock score (circle ratio) calculation
- Metrics included in district summary CSV

### Technical
- Polsby-Popper: `4π × Area / Perimeter²` (1.0 = perfect circle)
- Reock: `Area / Area of minimum bounding circle`

## 2026-01-09 - Initial Pipeline

### Added
- Complete 50-state redistricting pipeline
- Recursive bisection algorithm with METIS integration
- Adjacency graph construction from tract geometries
- District visualization with city labels
- National aggregate maps

### Features
- Parallel processing (4-8 states simultaneously)
- Resume capability with skip logic
- Hierarchical output structure
- Intermediate round visualization

### Technical
- Census tract-level redistricting (~84K tracts nationwide)
- Queen contiguity for adjacency
- ±1% population deviation tolerance
- Binary splits for hierarchical district structure

## Data Sources

### Census Data
- **TIGER/Line Shapefiles** (2020, 2010)
  - Tract geometries and boundaries
  - Place (city) points for labeling
- **Demographic and Housing Characteristics (DHC)** (2020)
  - Sex (male, female)
  - Race/ethnicity (White NH, Black NH, Asian NH, Hispanic, Other)
- **PL 94-171 Redistricting Files** (2020, 2010)
  - Population counts per tract

### Election Data
- **MIT Election Data + Science Lab**
  - 2020 Presidential election results
  - 2016 Presidential election results
  - Geocoded from precincts to census tracts
  - Coverage: 48 states (missing AK, HI tract-level data)

## Performance Benchmarks

### Full 50-State Pipeline
- **Sequential Mode** (1 worker): ~8-10 hours
- **Parallel Mode** (4 workers): ~2-3 hours
- **Parallel Mode** (8 workers): ~1.5-2 hours

### Per-State Processing
- Small states (<10 districts): 1-5 minutes
- Medium states (10-30 districts): 5-15 minutes
- Large states (>30 districts): 15-60 minutes
- California (52 districts): ~45-60 minutes

### Visualization
- State district map (DPI 150): 10-30 seconds
- National map (DPI 150): 3-5 minutes
- National map (DPI 300): 10-15 minutes (not recommended)

### Analysis
- Political analysis (per state): 5-60 seconds
- Demographic analysis (per state): 1-2 seconds
- Demographic visualization (per state): 5-30 seconds

## System Requirements

### Minimum
- Python 3.8+
- 8 GB RAM
- 20 GB disk space (includes all data)
- METIS library or executable

### Recommended
- Python 3.10+
- 16 GB RAM (for parallel processing)
- 30 GB disk space
- 4-8 CPU cores (for parallel mode)
- METIS library (faster than executable)

## Known Issues

### Alaska and Hawaii
- No tract-level election data available
- Political analysis will fail (expected)
- Demographic analysis works fine

### GEOID Type Mismatches
- Census tract GEOIDs lose leading zeros when stored as integers
- Always use `.astype(str).str.zfill(11)` before merging DataFrames
- See `CODING_PATTERNS.md` for details

### DPI and Performance
- DPI 300 causes 4x slowdown vs DPI 150
- DPI 150 provides excellent quality for most uses
- Only use DPI 300 for print-quality outputs

### Census API Rate Limiting
- Census API may rate limit during data downloads
- Scripts automatically retry with exponential backoff
- If persistent, wait a few minutes and re-run

## Future Work

### High Priority
- Unit tests for core algorithms
- Integration tests for full pipeline
- Error handling improvements
- Validation against actual districts

### Medium Priority
- Multi-member district support
- Alternative compactness measures
- Interactive visualization
- Web interface

### Low Priority
- Historical census data (2000, 1990)
- Block-level redistricting option
- Custom adjacency rules
- Export to other formats (KML, TopoJSON)

## Contributing

See `CONTRIBUTING.md` for development guidelines.

## Documentation

- `README.md` - Project overview and quickstart
- `ARCHITECTURE.md` - System design and algorithms
- `CODING_PATTERNS.md` - Coding conventions and patterns
- `DEPENDENCIES.md` - Installation and requirements
- `scripts/*/README.md` - Directory-specific documentation

## License

This project is for educational and research purposes.
