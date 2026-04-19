# Census Tract-Based Redistricting System

Automated redistricting for all 50 US states using recursive bifurcation and the METIS graph partitioning algorithm.

**Last Updated**: January 18, 2026

## Overview

This project implements census tract-based redistricting using:
- **Data Source**: Census tract data from data/2000/, data/2010/, data/2020/ with TIGER/Line shapefiles (tract-level geometries and population)
- **Algorithm**: Recursive bifurcation with METIS gpmetis (niter=100)
- **Adjacency**: Queen contiguity with county-aware water-based adjacency adaptation
- **Visualization**: 4-level progress bars, round-by-round maps, individual district maps, city labels
- **Scale**: Full 50-state processing pipeline with automated orchestration

## Quick Start

### Installation

```bash
pip install -r requirements.txt
```

**Note**: On Windows, `pymetis` may require pre-built wheels. If installation fails, try:
```bash
conda install -c conda-forge metis
pip install pymetis
```

### Process All 50 States

#### Windows Batch Files (Easiest)

```bash
# Multi-year parallel mode (all 3 census years: 2020, 2010, 2000) - NEW DEFAULT ✓
run_redistricting.bat -v v1
run -v v1                                             # Short: doskey alias + short flags

# Single year with custom workers
run_redistricting.bat -y 2020 -w 12 -v v1
run -y 2020 -w 12 -v v1                               # Short: doskey alias

# Skip state processing (fast national post-processing only)
run_redistricting.bat -v v1 --skip-states

# Test/debug runs (outputs to dev/)
run_test.bat -y 2020 -v my_test
runtest -y 2020 -v test                               # Short: doskey alias

# Get help (show all options)
run -h

# Emergency stop (if needed)
CANCEL.bat

# Note: Short flags available: -h=help, -y=year, -v=version, -s=states, -w=workers,
#       -r=reset, -p=print-only, -d=debug, -ey=election-year, -pm=partition-mode, -rt=run-type
#       Long forms also work: --help, --year, --version, --states, --dpi, etc.
```

#### Command Line (All Platforms)

```bash
# Multi-year parallel (DEFAULT) - runs all 3 census years in parallel with hierarchical progress
python scripts/pipeline/run_complete_redistricting.py -v v1
# Runs: 2020, 2010, 2000 concurrently with 12 workers (allocates 4+4+4)

# Single census year
python scripts/pipeline/run_complete_redistricting.py -y 2020 -v v1

# Specific states only
python scripts/pipeline/run_complete_redistricting.py -y 2020 -v v1 -s CA TX NY

# Custom workers (for multi-year: 4 workers → 2+1+1 allocation, 12 workers → 4+4+4)
python scripts/pipeline/run_complete_redistricting.py -w 12 -v v1

# Fresh run (delete existing outputs first)
python scripts/pipeline/run_complete_redistricting.py -v v1 -r

# Debug mode (progress delays for monitoring)
python scripts/pipeline/run_complete_redistricting.py -v v1 -d

# Skip state processing (fast iteration - just rerun national post-processing)
python scripts/pipeline/run_complete_redistricting.py -v v1 --skip-states

# Skip per-state analysis (faster, use old batch post-processing)
python scripts/pipeline/run_complete_redistricting.py -v v1 --skip-analysis

# Test/debug run (outputs to dev/)
python scripts/pipeline/run_complete_redistricting.py -y 2020 -v test -rt test -s VT

# Dry run (print only, no execution)
python scripts/pipeline/run_complete_redistricting.py -p -v preview

# Get help
python scripts/pipeline/run_complete_redistricting.py -h
```

**Performance**:
- **Multi-year parallel** (default with 12 workers): 2-4 hours for all 3 census years (60-70% faster than sequential)
- **Single year**: ~1 hour with 4 workers
- **Subsequent runs** (with `.states_complete` markers): Minutes instead of hours!

### Run Single State

```bash
# Process California through full pipeline
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --output-dir outputs/california

# With custom DPI
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --dpi 150
```

### DPI Options

- `--dpi 100`: Fast, lower quality
- `--dpi 150`: Default - good balance ✓
- `--dpi 200`: High quality
- `--dpi 300`: Print quality (slow)

## Project Structure

```
apportionment/
├── data/
│   ├── 2000/             # 2000 census raw data
│   │   ├── redistricting/# PL 94-171 redistricting files
│   │   └── tiger/        # TIGER/Line tract shapefiles
│   ├── 2010/             # 2010 census raw data
│   │   ├── redistricting/# PL 94-171 redistricting files
│   │   └── tiger/        # TIGER/Line tract shapefiles
│   └── 2020/             # 2020 census raw data
│       ├── redistricting/# PL 94-171 redistricting files
│       └── tiger/        # TIGER/Line tract shapefiles
├── outputs/
│   ├── data/             # Processed census data
│   │   ├── 2000/
│   │   │   ├── tracts/   # Census tract GeoParquet files
│   │   │   ├── adjacency/# Adjacency graphs
│   │   │   ├── places/   # Place labels
│   │   │   ├── elections/# Election data (2000)
│   │   │   └── demographics/# Demographics (2000)
│   │   ├── 2010/
│   │   │   ├── tracts/   # Census tract GeoParquet files
│   │   │   ├── adjacency/# Adjacency graphs
│   │   │   ├── places/   # Place labels
│   │   │   ├── elections/# Election data (2010)
│   │   │   └── demographics/# Demographics (2010)
│   │   └── 2020/
│   │       ├── tracts/   # Census tract GeoParquet files
│   │       ├── adjacency/# Adjacency graphs
│   │       ├── places/   # Place labels
│   │       ├── elections/# Election data (2020)
│   │       └── demographics/# Demographics (2020)
├── outputs/
│   ├── us_2020_v1/       # Full 50-state 2020 run
│   │   ├── states/       # Individual state directories
│   │   │   ├── california/
│   │   │   │   ├── intermediate/       # Round-by-round data
│   │   │   │   ├── maps/
│   │   │   │   │   ├── districts/      # Individual district PNGs
│   │   │   │   │   └── round_*.png     # Bisection round PNGs
│   │   │   │   ├── district_summary.csv
│   │   │   │   ├── district_cities.csv
│   │   │   │   ├── rounds_hierarchy.csv
│   │   │   │   └── *.png               # Final maps
│   │   │   └── ...
│   │   └── us_rounds_hierarchy.csv     # National aggregate
│   ├── us_2010_v1/       # Full 50-state 2010 run
│   ├── artifacts/        # Compiled academic outputs (PDFs)
│   │   ├── papers/       # Generated papers
│   │   ├── presentations/# Generated presentations
│   │   └── guides/       # Generated guides
│   ├── figures/          # Shared figures for papers/presentations
│   └── index.html        # Master dashboard
├── artifacts/            # Academic output sources (LaTeX)
│   ├── papers/           # Paper LaTeX sources
│   ├── presentations/    # Presentation LaTeX sources
│   ├── guides/           # Guide LaTeX sources
│   └── compile.bat       # Master compilation script
├── src/apportionment/
│   ├── data/             # Data acquisition and processing
│   ├── partition/        # Redistricting algorithms (METIS wrapper)
│   └── visualization/    # Map generation
├── scripts/              # Executable scripts (see below)
└── web/                  # Dashboard templates
    ├── dashboard.html    # Individual run dashboard
    └── master_dashboard.html  # Cross-run master dashboard
```

## Algorithm Details

### Recursive Bifurcation

For California (52 congressional districts):
1. Start with all blocks in the state
2. Split 52 → 26/26 using METIS
3. Recursively split each half: 26 → 13/13
4. Handle odd splits: 13 → 7/6
5. Continue until each region contains 1 district

### Water-Based Adjacency

Census blocks separated by water bodies (e.g., San Francisco Bay) can be considered adjacent:
- Standard Queen contiguity for land-based adjacency
- Distance-band method (default 1km) for blocks across water
- Enables districts to naturally span water bodies

## Data Sources

- **Tract Geometries**: TIGER/Line Shapefiles (from Census Bureau)
- **Population**: Census P.L. 94-171 Redistricting Files
- **Places (Cities)**: TIGER/Line Places shapefiles
- **Coverage**: All 50 US states, 2000, 2010, and 2020 census

## Key Features

### 4-Level Progress Bar System
- **Position 0**: USA-level progress (50 states)
- **Position 1**: State-level progress (5 steps per state)
- **Position 2**: Operation-specific progress (METIS splits, map generation)
- **Position 3**: Color-coded file existence indicators (green=exists, red=missing)

### Pipeline Stages (Per State)
1. **Redistricting**: Recursive bifurcation with METIS (niter=100)
2. **Cities**: Spatial join to add city labels to districts
3. **Round maps**: Visualize each bisection round
4. **District maps**: Generate individual PNG for each district
5. **Summary**: Create statistics CSV and rounds hierarchy

### Integrated Rounds Hierarchy
- Automatic creation during Summary stage
- Tracks bisection tree structure (round-by-round)
- National aggregate (`us_rounds_hierarchy.csv`) combines all states

### County-Aware Water Adjacency
- Island tracts prefer same-county connections
- Prevents cross-county island assignments
- Uses GEOID substring matching

## Scripts

### Main Orchestration
- `scripts/pipeline/run_complete_redistricting.py` - Process all 50 states with orchestration and progress bars
- `scripts/pipeline/run_state_redistricting.py` - Process single state through full pipeline

### Data Preparation
- `scripts/data/process_census_data.py` - Process census data (parse → merge → adjacency)
- `scripts/data/census/parse_pl94171_tracts_{year}.py` - Parse PL 94-171 files
- `scripts/data/merge_tracts_with_geometries.py` - Merge population + TIGER/Line geometries
- `scripts/data/geography/download_tiger_tracts.py` - Download TIGER/Line tract shapefiles
- `scripts/data/geography/build_all_adjacency_graphs.py` - Build adjacency graphs
- `scripts/data/geography/download_places.py` - Download cities/places data
- `scripts/data/validate_census_data.py` - Validate census data completeness

### Post-Processing
- `scripts/pipeline/add_cities_to_districts.py` - Add city labels to districts
- `scripts/pipeline/create_final_district_summary.py` - Generate statistics and rounds hierarchy
- `scripts/pipeline/visualize_all_rounds.py` - Create round-by-round maps
- `scripts/pipeline/create_individual_district_maps.py` - Generate per-district PNGs

## Testing

Comprehensive automated test suite with 90%+ code coverage across all pipeline components.

### Quick Test

```bash
# Run all tests (< 20 seconds)
pytest tests/ -v

# Run only unit tests (7 seconds)
pytest tests/unit/ -v

# Run E2E dashboard tests (8 seconds)
pytest tests/e2e/ -v
```

### Test Coverage

**Total: 215 tests in ~25 seconds**

| Category | Tests | Coverage |
|----------|-------|----------|
| Unit Tests | 135 | 95%+ |
| Integration Tests | 24 | 85%+ |
| E2E Tests | 56 | 90%+ |

**What's Tested:**
- ✅ Redistricting algorithm and METIS integration
- ✅ Political, demographic, compactness analysis
- ✅ Visualization and aggregation scripts
- ✅ Complete pipeline flows (multi-stage)
- ✅ Dashboard functionality (all tabs, state switching)
- ✅ Artifact validation (catches pipeline failures)

**Key Features:**
- Fast execution with mock data generators
- No external dependencies (all data mocked)
- Pipeline guardian tests catch breaking changes
- CI/CD ready with automated fixtures

See [tests/README.md](tests/README.md) for complete testing guide.

## Web Dashboard

Two interactive HTML dashboards provide visualization and navigation:

### Individual Run Dashboard
Per-run dashboard showing detailed results for a single redistricting run:

- **Source**: `web/dashboard.html` - Template for individual runs
- **Generated**: `outputs/us_{year}_{version}/index.html`
- **Features**:
  - **State Navigation**: Browse all 50 states from sidebar
  - **Tabs**: Overview, Districts, Rounds, USA (national), Political, Demographics, Compactness, Urban Areas
  - **Dynamic Content**: Maps, statistics, and download links per state

### Master Dashboard
Cross-run dashboard comparing multiple redistricting runs:

- **Source**: `web/master_dashboard.html` - Cross-run comparison template
- **Generated**: `outputs/index.html`
- **Features**:
  - **Overview Tab**: Clickable run cards for quick navigation
  - **Compactness Tab**: Side-by-side compactness analysis across runs
  - **Artifacts Tab**: View compiled PDFs (papers, presentations, guides)
  - **Run Comparison**: Compare 2000, 2010, 2020 census years side-by-side

### Usage
```bash
# Generate both dashboards after pipeline completes
python scripts/web/generate_master_dashboard.py

# Open master dashboard
open outputs/index.html

# Open specific run dashboard
open outputs/us_2020_v1/index.html
```

## Documentation

### User Guides (Start Here!)

**New to the project?** Start with these comprehensive, user-friendly guides:

- **[GETTING_STARTED.md](docs/GETTING_STARTED.md)** - Complete setup and first-run tutorial
  - Installation walkthrough
  - Prerequisites and dependencies
  - Running your first redistricting
  - Troubleshooting common issues

- **[DATA_DICTIONARY.md](docs/DATA_DICTIONARY.md)** - Field-by-field explanation of all outputs
  - CSV column definitions
  - Compactness metrics explained
  - Political and demographic data
  - How to interpret results

- **[VISUALIZATION_GUIDE.md](docs/VISUALIZATION_GUIDE.md)** - Reading maps and dashboards
  - Map types and what they show
  - Dashboard navigation
  - Interpreting compactness scores
  - Political and demographic visualization

- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Solutions to common problems
  - Installation errors
  - Runtime issues
  - Data problems
  - Performance optimization

### Algorithm Documentation

- **[RECURSIVE_BISECTION.md](docs/RECURSIVE_BISECTION.md)** - How the redistricting algorithm works
  - Step-by-step explanation
  - METIS graph partitioning
  - Population balancing
  - Compactness optimization

- **[CENSUS_DATA_PROCESSING.md](docs/CENSUS_DATA_PROCESSING.md)** - Census data processing pipeline
  - Data organization and structure
  - Processing steps (parse → merge → adjacency)
  - Commands and validation
  - File formats and requirements

- **[DEPENDENCIES.md](docs/DEPENDENCIES.md)** - Software requirements and setup
  - Python packages
  - METIS installation
  - Platform-specific instructions

### Developer Resources (AI-Optimized)

**For developers and AI assistants** - these docs are optimized for conciseness and token efficiency:

- **[ARCHITECTURE.md](context/ARCHITECTURE.md)** - System design (compact notation)
- **[CODING_PATTERNS.md](context/CODING_PATTERNS.md)** - Code conventions (pattern-first)
- **[TESTING.md](context/TESTING.md)** - Test system documentation
- **[SKILLS.md](context/SKILLS.md)** - Claude Code skills catalog (31 skills)
- **[ENHANCEMENT_WORKFLOW.md](context/ENHANCEMENT_WORKFLOW.md)** - Development process

### Contributing

- **[CONTRIBUTING.md](docs/CONTRIBUTING.md)** - Development workflow and git practices
  - Feature branch workflow
  - Common development tasks
  - Using Claude Code skills
  - Code review guidelines

- **[CHANGELOG.md](docs/CHANGELOG.md)** - Version history and recent changes

- **[Enhancement Index](context/enhancements/INDEX.md)** - Recent improvements and roadmap

### AI-Assisted Development

- **[CLAUDE.md](CLAUDE.md)** - Guide for Claude Code (AI assistant)
  - Project context and patterns
  - Common tasks and workflows
  - Skill usage

## Future Extensions

- Alternative algorithms (K-means, simulated annealing)
- Compactness optimization beyond edge-weighted METIS
- Interactive visualization
- Additional census years (1990, earlier decades)
