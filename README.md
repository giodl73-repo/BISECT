# Census Tract-Based Redistricting System

Automated redistricting for all 50 US states using recursive bifurcation and the METIS graph partitioning algorithm.

**Last Updated**: January 16, 2026

## Overview

This project implements census tract-based redistricting using:
- **Data Source**: Census 2020 & 2010 TIGER/Line shapefiles (tract-level geometries and population)
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
# Parallel mode (4-8 states at once) - FAST ✓
run_parallel.bat

# With custom settings
run_parallel.bat --workers 6 --dpi 200 --version v2

# Sequential mode (one at a time)
run_sequential.bat

# Emergency stop (if needed)
CANCEL.bat
```

#### Command Line (All Platforms)

```bash
# Parallel mode - runs 4 states simultaneously (default)
python scripts/pipeline/run_complete_redistricting.py --mode parallel --year 2020 --version v1

# Sequential mode - one state at a time
python scripts/pipeline/run_complete_redistricting.py --mode sequential --year 2020 --version v1

# Custom workers and quality
python scripts/pipeline/run_complete_redistricting.py --mode parallel --workers 8 --dpi 200

# Fresh run (delete existing outputs first)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --reset

# Skip per-state analysis (faster, use old batch post-processing)
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --skip-analysis
```

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
│   ├── raw/              # Census tracts and places (2020 & 2010)
│   ├── adjacency/        # Adjacency graphs
│   └── processed/        # Other processed data
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

- **Tract Geometries**: TIGER/Line Shapefiles (via pygris)
- **Population**: Census P.L. 94-171 Redistricting File
- **Places (Cities)**: TIGER/Line Places shapefiles
- **Coverage**: All 50 US states, 2020 & 2010 census

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
- `scripts/data/census/download_all_states_tracts.py` - Download census tract data for all states
- `scripts/data/geography/download_places.py` - Download cities/places data
- `scripts/data/geography/build_adjacency.py` - Build adjacency graphs (saves to data/adjacency/)

### Post-Processing
- `scripts/pipeline/add_cities_to_districts.py` - Add city labels to districts
- `scripts/pipeline/create_final_district_summary.py` - Generate statistics and rounds hierarchy
- `scripts/pipeline/visualize_all_rounds.py` - Create round-by-round maps
- `scripts/pipeline/create_individual_district_maps.py` - Generate per-district PNGs

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

For detailed technical documentation, see:

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and architectural decisions
  - Data pipeline architecture
  - Redistricting algorithm details
  - Scope-based analysis pattern
  - Progress bar protocol

- **[CODING_PATTERNS.md](docs/CODING_PATTERNS.md)** - Developer patterns and best practices
  - File organization conventions
  - Progress bar integration
  - Scope-based analysis implementation
  - Testing guidelines

- **[SKILLS.md](docs/SKILLS.md)** - Claude Code skills for automation (25 skills)
  - Enhancement workflow
  - Pipeline execution
  - Visualization & analysis
  - Code organization

- **[CONTRIBUTING.md](docs/CONTRIBUTING.md)** - Development workflow and git practices
  - Feature branch workflow
  - Common development tasks
  - Using Claude Code skills
  - Code review guidelines

- **[ENHANCEMENTS_2026.md](docs/ENHANCEMENTS_2026.md)** - Recent improvements and roadmap
  - Completed enhancements
  - Performance optimizations
  - New features and capabilities

- **[CLAUDE.md](CLAUDE.md)** - Guide for AI-assisted development
  - Project context and architecture
  - Common tasks and workflows
  - Integration patterns

## Future Extensions

- Additional census years (2000)
- Alternative algorithms (K-means, simulated annealing)
- Compactness optimization
- Interactive visualization
