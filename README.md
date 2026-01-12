# Census Tract-Based Redistricting System

Automated redistricting for all 50 US states using recursive bifurcation and the METIS graph partitioning algorithm.

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
python scripts/run_complete_redistricting.py --mode parallel --year 2020 --version v1

# Sequential mode - one state at a time
python scripts/run_complete_redistricting.py --mode sequential --year 2020 --version v1

# Custom workers and quality
python scripts/run_complete_redistricting.py --mode parallel --workers 8 --dpi 200
```

### Run Single State

```bash
# Process California through full pipeline
python scripts/run_state_redistricting.py --state CA --year 2020 --output-dir outputs/california

# With custom DPI
python scripts/run_state_redistricting.py --state CA --year 2020 --dpi 150
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
│   └── us_2010_v1/       # Full 50-state 2010 run
├── src/apportionment/
│   ├── data/             # Data acquisition and processing
│   ├── partition/        # Redistricting algorithms (METIS wrapper)
│   └── visualization/    # Map generation
└── scripts/              # Executable scripts (see below)
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
- `run_all_states.py` - Process all 50 states with progress bars
- `run_state_redistricting.py` - Process single state through full pipeline

### Data Preparation
- `download_tracts.py` - Download census tract data
- `download_places.py` - Download cities/places data
- `build_tract_adjacency.py` - Build adjacency graphs (saves to data/adjacency/)

### Post-Processing
- `add_cities_to_districts.py` - Add city labels
- `create_final_district_summary.py` - Generate statistics and rounds hierarchy
- `visualize_all_rounds.py` - Create round-by-round maps
- `create_individual_district_maps.py` - Generate per-district PNGs

## Web Dashboard

An interactive HTML dashboard provides visualization and navigation across all outputs:

### Source and Deployment
- **Source**: `web/dashboard.html` - Master dashboard template
- **Deployment**: Run `python web/deploy_dashboard.py` to copy to `outputs/index.html`
- **Access**: Open `outputs/index.html` in a browser

### Features
- **State Navigation**: Browse all 50 states from sidebar
- **Dimensions**: Overview, Districts, Rounds, Political, Demographics, Compactness, Urban Areas
- **Version Switching**: Navigate between different output directories (`us_2020_v1`, `us_2030_v1`, etc.)
- **Year Switching**: Switch between census years (2020, 2030, 2040)
- **Dynamic Content**: Maps, statistics, and download links for each state and dimension

### Usage
```bash
# Deploy dashboard after generating outputs
python web/deploy_dashboard.py

# Open in browser
open outputs/index.html
```

## Future Extensions

- Additional census years (2000)
- Alternative algorithms (K-means, simulated annealing)
- Compactness optimization
- Interactive visualization
