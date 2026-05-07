# Pipeline Scripts

Orchestrates the complete 50-state redistricting workflow.

## Overview

These scripts manage the end-to-end redistricting process:
1. Process individual states through recursive bisection
2. Aggregate results nationally
3. Create visualizations
4. Run analysis (political, demographic)

## Main Entry Points

### run_complete_redistricting.py

**Purpose**: Master orchestrator for full 50-state pipeline

**Usage**:
```bash
# Parallel mode (4-8 states simultaneously)
python scripts/pipeline/run_complete_redistricting.py --workers 4 --year 2020 --version v1

# Sequential mode (one at a time)
python scripts/pipeline/run_complete_redistricting.py --workers 1

# Custom settings
python scripts/pipeline/run_complete_redistricting.py --workers 6 --dpi 150 --version v3

# Skip certain stages
python scripts/pipeline/run_complete_redistricting.py --skip-political --skip-demographic
```

**Parameters**:
- `--workers N`: Parallelism (1=sequential, 2-8=parallel)
- `--year {2020|2010}`: Census year
- `--version ID`: Version identifier (v1, v2, etc.)
- `--dpi N`: Map quality (72-300, default 150)
- `--election-year {2020|2016}`: For political analysis
- `--skip-political`: Skip political analysis
- `--skip-demographic`: Skip demographic analysis
- `--skip-states`: Skip state processing (post-processing only)
- `--reprocess`: Force reprocess all states
- `--print-only`: Dry run (show commands without executing)
- `--debug`: Enable debug mode with delays

**Pipeline Stages**:
1. State redistricting (parallel or sequential)
2. Create US aggregate files
3. Create US rounds hierarchy
4. Create US national maps
5. Run political analysis (all states)
6. Run demographic analysis (all states)
7. Run demographic visualization (all states)

### run_state_redistricting.py

**Purpose**: Process single state through full pipeline

**Usage**:
```bash
# Full state pipeline
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --output-dir outputs/california

# With custom DPI
python scripts/pipeline/run_state_redistricting.py --state CA --dpi 200
```

**What it does**:
1. Load tract data and adjacency graph
2. Run recursive bisection (METIS)
3. Create district summary and cities
4. Generate all maps (districts, rounds, individuals)
5. Calculate compactness metrics

### process_single_state.py

**Purpose**: Low-level state processor (called by run_state_redistricting.py)

**Usage**: Typically called by parent scripts, not directly
```bash
python scripts/pipeline/process_single_state.py --state CA --year 2020 --output-dir outputs/ca --position 1
```

## Aggregation Scripts

### create_us_aggregate.py

**Purpose**: Aggregate district statistics across all 50 states

**Output**: `outputs/us_2020_v1/us_district_summary.csv`

Contains nationwide statistics:
- Total districts: 435
- Population per district
- Compactness metrics (Polsby-Popper, Reock)
- City assignments

### create_us_rounds_hierarchy.py

**Purpose**: Aggregate bisection round data nationally

**Output**: `outputs/us_2020_v1/us_rounds_hierarchy.csv`

Shows hierarchical district structure:
- Round-by-round splits
- District groupings
- Population balance at each level

## Visualization Scripts

### create_us_national_map.py

**Purpose**: Create map of all 435 congressional districts

**Usage**:
```bash
python scripts/pipeline/create_us_national_map.py --year 2020 --output-dir outputs/us_2020_v1 --dpi 150
```

**Outputs**:
- `us_national_map.png` - Basic 435-district map
- `us_national_map_with_cities.png` - With major city labels

**Performance**:
- DPI 150: ~3-5 minutes
- DPI 300: ~10-15 minutes (not recommended)

### visualize_districts.py

**Purpose**: Create district map for a single state

**Called by**: run_state_redistricting.py

**Outputs**:
- State-level district map
- Individual district maps (in districts/ subdirectory)

### visualize_all_rounds.py

**Purpose**: Visualize bisection rounds (how districts were created)

**Usage**:
```bash
python scripts/pipeline/visualize_all_rounds.py --state CA --year 2020 --output-dir outputs/ca/maps --dpi 150
```

**Creates**: `round_0.png`, `round_1.png`, ..., `round_N.png`

### visualize_split.py

**Purpose**: Visualize a single partition split

**Usage**: Debugging tool, typically not called directly

### create_individual_district_maps.py

**Purpose**: Create separate PNG for each district

**Usage**:
```bash
python scripts/pipeline/create_individual_district_maps.py --state CA --tracts data/raw/ca_tracts_2020.parquet --assignments ca/final_assignments.pkl --dpi 150
```

**Output**: `maps/districts/district_01.png`, `district_02.png`, ...

## Utility Scripts

### add_cities_to_districts.py

**Purpose**: Assign major cities to districts

**Usage**:
```bash
python scripts/pipeline/add_cities_to_districts.py --state CA --tracts data/raw/ca_tracts_2020.parquet --places data/raw/ca_places_2020.parquet --assignments ca/final_assignments.pkl
```

**Output**: `district_cities.csv` with major cities per district

### create_final_district_summary.py

**Purpose**: Aggregate district metrics

**Calculates**:
- Population per district
- Number of tracts
- Compactness metrics
- Geographic extents

**Output**: `district_summary.csv`

### create_single_district_states.py

**Purpose**: Handle at-large states (Alaska, Delaware, Montana, etc.)

**Usage**: Automatically called for single-district states

**Creates**:
- Simplified directory structure
- Basic at-large district map
- Minimal CSV files

## Key Patterns

### Script Communication

**Parent-Child Protocol**:
```python
# Parent sets environment variable
env = os.environ.copy()
env['TQDM_POSITION'] = str(position)

# Child detects
position = int(os.environ.get('TQDM_POSITION', '-1'))
send_status = position >= 0

# Child reports progress
if send_status:
    print(f"STATUS:{position}:Processing state X...", flush=True)

# Parent monitors
for line in proc.stdout:
    if line.startswith("STATUS:"):
        # Update progress bar
        ...
```

### Skip Logic

All scripts check for existing outputs:
```python
if not args.force and output_file.exists():
    print(f"Output exists - skipping: {output_file}")
    return 0
```

**Why?**
- Resumable after failures
- Skip expensive re-computation
- Enable incremental development

### DPI Threading

All visualization scripts accept `--dpi`:
```python
parser.add_argument('--dpi', type=int, default=150,
                   choices=[72, 100, 150, 200, 300])
```

**Guidelines**:
- Development: 100
- Production: 150 (default)
- High quality: 200
- Print: 300 (slow!)

## Script Dependency Graph

```
run_complete_redistricting.py
├── [Parallel/Sequential] Per-state processing
│   ├── run_state_redistricting.py
│   │   ├── process_single_state.py
│   │   │   └── src/redistricting/bisection.py (recursive bisection)
│   │   ├── create_final_district_summary.py
│   │   ├── add_cities_to_districts.py
│   │   ├── visualize_districts.py
│   │   ├── visualize_all_rounds.py
│   │   └── create_individual_district_maps.py
│   └── create_single_district_states.py (for at-large states)
│
├── create_us_aggregate.py
├── create_us_rounds_hierarchy.py
├── create_us_national_map.py
├── scripts/political/run_political_analysis.py
│   ├── analyze_districts.py
│   └── visualize_partisan_lean.py
├── scripts/political/run_demographic_analysis.py
│   └── analyze_district_demographics.py
└── scripts/political/run_demographic_visualization.py
    └── visualize_district_demographics.py
```

## Common Issues

### Issue: Duplicate Progress Bars

**Symptom**: Multiple progress bars for same task

**Cause**: Parent not setting `TQDM_POSITION` environment variable

**Fix**:
```python
env = os.environ.copy()
env['TQDM_POSITION'] = str(position)
subprocess.Popen(cmd, env=env, ...)
```

### Issue: Slow Map Generation

**Symptom**: Maps taking 10+ minutes each

**Cause**: DPI set to 300 (either hardcoded or passed)

**Fix**:
- Use `--dpi 150` (default)
- Check for hardcoded `dpi=300` in savefig calls
- See `../../context/CODING_PATTERNS.md` for DPI threading pattern

### Issue: Missing GEOID Data

**Symptom**: "Could not join demographic/election data"

**Cause**: GEOID type mismatch (int vs string)

**Fix**: Always use `.astype(str).str.zfill(11)`
```python
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)
```

### Issue: Failed States Not Resuming

**Symptom**: Failed states re-run from scratch

**Cause**: Missing skip logic or incomplete output detection

**Fix**: Ensure all required files checked:
```python
required_files = [
    'final_assignments.pkl',
    'district_summary.csv',
    'district_cities.csv',
    'maps/'
]
if all(f.exists() for f in required_files):
    # Skip
```

## Performance Tips

1. **Use Parallel Mode**: 4-8 workers reduces total time from 8h to 2h
2. **Use DPI 150**: Balance between quality and speed
3. **Skip Logic**: Enable resumable pipelines
4. **Parquet Format**: Much faster than CSV for large data

## Output Structure

```
outputs/us_2020_v1/
├── states/
│   ├── alabama/
│   │   ├── final_assignments.pkl
│   │   ├── district_summary.csv
│   │   ├── district_cities.csv
│   │   ├── rounds_hierarchy.csv
│   │   ├── intermediate/
│   │   │   └── round_*.pkl
│   │   ├── maps/
│   │   │   ├── alabama_7_districts.png
│   │   │   ├── alabama_7_districts_with_cities.png
│   │   │   ├── districts/
│   │   │   │   ├── district_01.png
│   │   │   │   └── ...
│   │   │   └── round_*.png
│   │   ├── political_analysis/
│   │   │   ├── district_political_2020.csv
│   │   │   └── maps/
│   │   │       ├── partisan_lean.png
│   │   │       └── partisan_lean_with_margins.png
│   │   └── demographic_analysis/
│   │       ├── district_demographics.csv
│   │       └── maps/
│   │           ├── gender_balance.png
│   │           ├── majority_race.png
│   │           └── diversity_index.png
│   └── ...
├── us_district_summary.csv
├── us_rounds_hierarchy.csv
├── us_national_map.png
└── us_national_map_with_cities.png
```

## See Also

- `../../context/CODING_PATTERNS.md` - Detailed coding conventions
- `../../context/ARCHITECTURE.md` - System design and algorithms
- `scripts/political/README.md` - Political analysis scripts
- `scripts/data/README.md` - Data acquisition scripts
