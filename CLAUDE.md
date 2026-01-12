# Claude Code Assistant Guide

This document provides context and guidelines for AI assistants working on the Congressional Redistricting codebase.

## Project Overview

This is a congressional redistricting implementation using METIS recursive bisection algorithm to generate 435 districts across all 50 US states based on 2010 and 2020 Census data.

**Core Goal:** Algorithmically generate compact, population-balanced congressional districts using only geographic and demographic constraints (no gerrymandering).

## Key Technologies

- **Algorithm:** METIS graph partitioning (recursive bisection)
- **Language:** Python 3.13+
- **GIS:** GeoPandas, Shapely
- **Visualization:** Matplotlib
- **Data:** Census tract-level population, demographics, election results
- **Web:** Static HTML/JS dashboard

## Critical Files & Directories

### Configuration
- `scripts/config_2020.py` - State district counts for 2020 apportionment
- `scripts/config_2010.py` - State district counts for 2010 apportionment

### Core Algorithm
- `src/apportionment/partition/recursive_bisection.py` - Main redistricting algorithm
- `src/apportionment/partition/metis_wrapper.py` - METIS interface
- `src/apportionment/data/adjacency.py` - Tract adjacency graph generation

### Pipeline Scripts (Executable Entry Points)
- `scripts/pipeline/run_complete_redistricting.py` - **Main orchestrator** for 50-state pipeline
- `scripts/pipeline/run_state_redistricting.py` - Single-state redistricting wrapper
- `scripts/pipeline/process_single_state.py` - Core single-state logic

### Analysis Scripts
- `scripts/political/` - Partisan lean analysis using 2020 election data
- `scripts/demographic/` - Demographic composition analysis
- `scripts/compactness/` - Polsby-Popper and Reock compactness scores

### Batch Files (User Entry Points)
- `run_redistricting.bat` - Main wrapper for running pipeline
- `deploy_web.bat` - Generate and open interactive dashboard
- `CANCEL.bat` - Kill all running Python processes

### Web Dashboard
- `web/dashboard.html` - Single-file interactive dashboard (HTML/CSS/JS)
- `scripts/web/generate_dashboard.py` - Bakes district data into static HTML

## Data Exclusions

**IMPORTANT:** The following are excluded from git (see `.gitignore`):
- `data/` - All census tract shapefiles, demographics, election data (~40GB)
- `outputs/` - All generated maps, CSVs, district assignments (~20GB per run)
- `*.png`, `*.jpg`, `*.pdf` - All images (except docs/)

**Never commit data or output files!**

## Project Structure

```
src/apportionment/         # Python package (library code)
├── partition/             # Core algorithms (import from scripts)
├── data/                  # Data loading utilities
└── visualization/         # Visualization helpers

scripts/                   # Executable scripts (use the library)
├── pipeline/              # Main pipeline orchestration
├── political/             # Political analysis
├── demographic/           # Demographic analysis
├── compactness/           # Compactness analysis
├── web/                   # Dashboard generation
├── config_2020.py         # Configuration
└── config_2010.py         # Configuration

web/                       # Dashboard template
└── dashboard.html         # Single-file static dashboard

docs/                      # Documentation
├── archive/               # Historical session notes
└── ENHANCEMENTS_2026.md   # Recent improvements

paper/                     # Academic paper
├── analysis/              # Statistical analysis scripts
└── sections/              # LaTeX sections
```

## Coding Patterns & Conventions

### Progress Reporting
Scripts that run as child processes use `STATUS:position:message` protocol:
```python
position = int(os.environ.get('TQDM_POSITION', '-1'))
send_status = position >= 0

def report_progress(msg):
    if send_status:
        print(f"STATUS:{position}:{msg}", flush=True)
```

### Import Patterns
- Scripts import from library: `from apportionment.partition.recursive_bisection import ...`
- Scripts import config: `from scripts.config_2020 import STATE_CONFIG_2020`

### File Naming Conventions
- State names: lowercase with underscores (`california`, `new_york`)
- Maps: `{state}_{districts}_districts.png`
- Maps with cities: `{state}_{districts}_districts_with_cities.png`
- National maps: `US_National_Map_435_Districts_2020.png`

### Path Handling
- Use `Path` objects from `pathlib`, not string concatenation
- Always use absolute paths or paths relative to project root
- Dashboard uses `.` for relative paths (deployed inside output directory)

## Common Tasks

### Add a New Analysis Type
1. Create `scripts/{analysis_type}/` directory
2. Create `analyze_{thing}.py` (per-state analysis)
3. Create `run_{analysis}_analysis.py` (all-states wrapper)
4. Create `create_us_national_{analysis}_map.py` (national visualization)
5. Add to pipeline in `run_complete_redistricting.py`
6. Add tab to dashboard in `web/dashboard.html`

### Add a New Command-Line Parameter
1. Add to `argparse` in relevant script
2. Pass through pipeline hierarchy if needed
3. Update batch files if user-facing

### Update Dashboard
1. Edit `web/dashboard.html` (single template file)
2. Run `deploy_web.bat` to regenerate with data
3. Output goes to `outputs/us_{year}_{version}/index.html`

## Testing & Running

### Run Full Pipeline
```bash
run_redistricting.bat --year 2020 --version v1 --dpi 150
```

### Generate Dashboard
```bash
deploy_web.bat --year 2020 --version v1
```

### Print-Only Mode (Dry Run)
```bash
python scripts/pipeline/run_complete_redistricting.py --year 2020 --version v1 --print-only
```

## Important Notes

### Git Workflow
- Fresh repo created Jan 2026 with clean history
- Old repo with data files archived in `.git.old/`
- Always verify no data/outputs files staged before commit

### Performance
- 50-state full run: ~2-4 hours (parallel mode)
- Single state: 30 seconds - 5 minutes depending on size
- Dashboard generation: ~5 seconds

### Windows-Specific
- METIS binary: `bin/gpmetis.exe` (Windows build)
- Batch files use Windows-style paths with backslashes
- Line endings: CRLF (Windows) - Git auto-converts

### Algorithm Constraints
- Equal population: Districts within ±0.5% of target population
- Contiguity: All districts must be geographically contiguous
- Compactness: Optimized via METIS edge-cut minimization
- No political/racial considerations (purely algorithmic)

## Documentation Files - When to Read What

### Start Here
- **`CLAUDE.md`** (this file) - AI assistant guide, project overview, quick reference

### Understand the System
- **`README.md`** - User-facing project description, setup instructions, usage examples
  - Read when: Understanding what the project does, how users interact with it

- **`ARCHITECTURE.md`** - System design, data flow, component relationships, technical decisions
  - Read when: Understanding how components interact, modifying core architecture, adding major features

### Development Guidelines
- **`CODING_PATTERNS.md`** - Detailed coding conventions, naming patterns, progress reporting protocol
  - Read when: Writing new code, ensuring consistency with existing patterns

- **`CONTRIBUTING.md`** - Development workflow, git practices, code review guidelines
  - Read when: Making contributions, understanding development process

### Data & Setup
- **`DATA_FORMATS.md`** - File formats, CSV schemas, data structures, column definitions
  - Read when: Working with data files, understanding input/output formats

- **`DEPENDENCIES.md`** - Required packages, installation instructions, environment setup
  - Read when: Setting up development environment, debugging dependency issues

### History & Changes
- **`CHANGELOG.md`** - Version history, feature additions, bug fixes
  - Read when: Understanding what changed between versions, tracking feature history

- **`docs/archive/`** - Historical session notes from previous Claude conversations
  - Read when: Understanding why specific decisions were made, detailed implementation history

### Quick Decision Tree

**User asks about setup/installation?** → Read `README.md` + `DEPENDENCIES.md`

**Need to understand how something works?** → Read `ARCHITECTURE.md`

**Writing new code?** → Read `CODING_PATTERNS.md`

**Working with data files?** → Read `DATA_FORMATS.md`

**Understanding recent changes?** → Read `CHANGELOG.md`

**Making modifications/contributions?** → Read `CONTRIBUTING.md`

**General orientation/quick reference?** → Read `CLAUDE.md` (this file)

## Common Pitfalls

1. **Don't commit data files** - Always excluded via .gitignore
2. **Config imports** - Use `from scripts.config_2020 import ...` (not from root)
3. **Dashboard paths** - Use `getBasePath()` which returns `.` for relative paths
4. **Progress bars** - Child processes must use STATUS protocol, not direct tqdm
5. **Line endings** - Git auto-converts LF to CRLF on Windows
6. **State names** - Always lowercase with underscores in code

## Recent Major Changes (Jan 2026)

- Added compactness visualization pipeline (Polsby-Popper, Reock)
- Created static dashboard generator with district data baking
- Added Cities tab to dashboard (separate from Overview)
- Integrated political and demographic national maps
- Moved config files from root to scripts/
- Fresh git repo (removed 240MB of data from history)

## Future Enhancements

### National Round Progression Maps
**Goal:** Create national-level visualization of recursive bisection progression (rounds 1-6+)

**Description:**
Generate a series of national maps showing how the US is progressively divided:
- Round 1: 2 regions (first bisection)
- Round 2: 4 regions (second bisection)
- Round 3: 8 regions
- Round 4: 16 regions
- Round 5: 32 regions
- Round 6: 64 regions
- Continue through later rounds as states complete their divisions

**Implementation:**
1. **Data Collection Phase:**
   - Aggregate round data from all 50 states' `rounds/round_N_assignments.pkl` files
   - Track which states have completed which rounds
   - Handle states with different final round counts (1-district states vs 52-district California)

2. **Visualization Script:**
   - Create `scripts/pipeline/create_us_national_rounds_progression.py`
   - Generate maps: `us_national_round_1_2020.png`, `us_national_round_2_2020.png`, etc.
   - Use consistent color scheme across rounds
   - Show state boundaries with districts/regions overlaid
   - As states complete their final districts, show them fully divided in subsequent rounds

3. **Pipeline Integration:**
   - Add as post-processing step after `create_us_rounds_hierarchy.py`
   - Run before demographic/political visualization (reuse tract loading logic)

4. **Dashboard Integration:**
   - Add to USA row, Rounds tab
   - Show progressive sequence of national bisection
   - Allow users to see national-level recursive division pattern

**Benefits:**
- Visualize national-level recursive bisection strategy
- Understand how equal-population constraint affects regional divisions
- Compare bisection patterns across geographic regions
- Educational tool for understanding METIS recursive algorithm at scale

**Files to Create:**
- `scripts/pipeline/create_us_national_rounds_progression.py` - Main generation script
- Update `web/dashboard.html` - Add USA Rounds view with progression maps
- Update `run_complete_redistricting.py` - Add pipeline step

**Estimated Complexity:** Medium (2-3 hours)
- Similar to existing national map generation
- Complexity in aggregating round data across states with different completion points

## Questions or Issues?

Check `docs/archive/` for historical session notes and implementation details from previous development sessions.
