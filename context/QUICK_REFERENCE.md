# Quick Reference

**Updated**: 2026-01-18

## Commands

### Census Data Processing
```bash
# Full pipeline (parse → merge → adjacency)
python scripts/data/process_census_data.py --year 2020                # Process all stages
python scripts/data/process_census_data.py --year 2020 --states VT   # Single state
python scripts/data/process_census_data.py --year 2020 --workers 24  # More workers

# Individual stages
python scripts/data/census/parse_pl94171_tracts_2020.py --states VT  # Parse PL 94-171
python scripts/data/merge_units_with_geometries.py --year 2020      # Merge geometries
python scripts/data/geography/build_all_adjacency_graphs.py --year 2020  # Build graphs

# Download TIGER/Line shapefiles
python scripts/data/geography/download_tiger_units.py --year 2020
python scripts/data/geography/download_tiger_units.py --year 2010
python scripts/data/geography/download_tiger_units.py --year 2000

# Validate data completeness
python scripts/data/validate_census_data.py --year 2020
python scripts/data/validate_census_data.py --year 2020 --states VT --verbose

# Reorganize directory structure (one-time)
scripts/data/reorganize_census_data.bat
```

### Pipeline
```bash
# Production runs (outputs/v1/{year}/)
run -h                                                # Show all options and flags
run -v v1                                             # Multi-year parallel - 2-4h (doskey alias)
run -y 2020 -v v1                                     # Single year - ~1h
run -y 2020 -v v1 -st CA TX NY                        # Specific states only
run -w 16 -v v1                                       # Custom workers (5+5+6 allocation)
run -v v1 -r                                          # Fresh run (reset/delete outputs)
run -v v1 -d                                          # Debug mode (progress delays)
run -v v1 -s nation                                   # National only (fast - minutes)
run -p -v test                                        # Dry run (print only)

# Test/debug runs (outputs/dev/{version}_{year}/)
runtest -y 2020 -v my_test                            # Test run (doskey alias)
runtest -y 2020 -v test -st VT                        # Test with single state

# Long forms also work (short flags are optional)
run --year 2020 --version v1 --workers 12 --reset    # All long flags

# Short flag reference:
# -h=help, -y=year, -v=version, -s=states, -w=workers, -r=reset, -p=print-only,
# -d=debug, -ey=election-year, -pm=partition-mode, -rt=run-type
```

### Dashboard
```bash
python scripts/web/generate_master_dashboard.py     # Generate both dashboards
start outputs/index.html                             # Master
start outputs/us_2020_v1/index.html                 # Specific run
```

### Tests
```bash
pytest tests/ -v                 # All 215 tests (~25s)
pytest tests/unit/ -v            # Unit 135 tests (~9s)
pytest tests/integration/ -v     # Integration 24 tests (~3s)
pytest tests/e2e/ -v             # E2E 56 tests (~10s)
pytest tests/ --cov=src/apportionment --cov-report=html  # With coverage
pytest tests/unit/test_file.py::test_func -v       # Specific test
```

### Emergency
```bash
CANCEL.bat  # Kill all Python processes (Windows)
```

## File Paths

**Raw Data**:
```
data/{year}/
├── redistricting/                    # PL 94-171 redistricting files
│   ├── {state}{year}.pl/             # State PL files (2010, 2020)
│   └── {state}geo.upl                # State geographic files (2000)
└── tiger/                            # TIGER/Line tract shapefiles
    └── tl_{year}_{fips}_tract/
```

**Processed Data**:
```
outputs/data/{year}/
├── tracts/{state}_tracts_{year}.parquet              # Census tract GeoParquet
├── adjacency/{state}_adjacency_{year}.pkl            # Adjacency graphs
├── places/{state}_places_{year}.parquet              # Place labels
├── elections/{year}_president_tract.parquet          # Election data
└── demographics/{year}_demographics_tract.parquet    # Demographics
```

**Pipeline Output**:
```
outputs/us_{year}_{version}/
├── states/{state_name}/
│   ├── data/           # final_assignments.pkl, district_summary.csv, district_cities.csv, rounds_hierarchy.csv
│   ├── maps/           # all_districts.png, rounds/, districts/
│   ├── political/      # district_political.csv, maps/
│   ├── demographic/    # district_demographics.csv, maps/
│   └── compactness/    # district_compactness.csv, maps/
├── data/               # us_*.csv files
└── maps/               # us_*.png, rounds/
```

## DPI Options
- 100: Fast, dev
- 150: Default ✓
- 200: High quality
- 300: Print (slow, large)

## Troubleshooting

### Unicode Windows
**Problem**: `UnicodeEncodeError: 'charmap' codec can't encode character`
**Fix**: Replace Unicode w/ ASCII: ✓→`[OK]`, ✗→`[FAIL]`, →→`->`, •→`-`

### GEOID Type
**Problem**: `TypeError: merge on int64 and object columns`
**Fix**: `df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)`

### Missing Data
**Problem**: `FileNotFoundError`
**Fix**:
```python
if not data_file.exists():
    print(f"[SKIP] Data not available: {data_file}")
    return 0
```

### State Codes
Use standard mapping:
- 2-letter: `AL`, `CA`, `TX` (Census data)
- Full uppercase: `ALABAMA`, `CALIFORNIA`, `TEXAS` (pipeline outputs)
- Lowercase underscore: `alabama`, `california`, `texas` (directories)

### Pipeline Fails
1. Check error message
2. Use `/pipeline-debug` skill
3. Just re-run (per-stage skip logic resumes from failure)

### Test Fails
1. Use `/debug-tests` skill
2. Check 6 patterns: imports, mocks, assertions, Playwright, fixtures, teardown
3. Run specific: `pytest tests/unit/test_file.py::test_func -v`

## Flags

**Pipeline Short Flags** (all have long equivalents):

| Short | Long | Values | Default | Description |
|-------|------|--------|---------|-------------|
| `-h` | `--help` | - | - | Show all options |
| `-y` | `--year` | 2020/2010/2000/all | all | Census year(s) |
| `-v` | `--version` | string | v1 | Output version identifier |
| `-s` | `--states` | state codes | all | Specific states (e.g., CA TX NY) |
| `-w` | `--workers` | 1-24 | 12 | Parallel workers |
| `-r` | `--reset` | flag | - | Delete outputs (fresh run) |
| `-p` | `--print-only` | flag | - | Dry run (no execution) |
| `-d` | `--debug` | flag | - | Debug mode with delays |
| `-ey` | `--election-year` | 2020/2016 | 2020 | Election data year |
| `-pm` | `--partition-mode` | edge-weighted/unweighted | edge-weighted | METIS partition mode |
| `-rt` | `--run-type` | production/test/experiment | production | Output directory type |

**Pipeline Long Flags** (no short equivalent):
- `--dpi N`: Map DPI (72/100/150/200/300, default: 150)
- `--skip-states`: National post-processing only
- `--skip-analysis`: Skip per-state analysis (legacy)
- `--force`: Override skip logic
- `--skip-political`: Skip political analysis
- `--skip-demographic`: Skip demographic analysis
- `--states-only`: Process states, skip post-processing
- `--reprocess`: Reprocess all states (ignore skip logic)
- `--experiment-name`: Name for experiment runs (required with `-rt experiment`)

**Tests**:
- `-v`: Verbose
- `-s`: Show prints
- `-x`: Stop on first fail
- `--cov`: Coverage
- `-k PATTERN`: Filter tests
- `--markers`: List markers

## Environment

**TQDM_POSITION**: Progress bar hierarchy position (set by parent)
```python
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0: print(f"STATUS:{position}:{msg}", flush=True)
```

## Critical Files

**Config**: `scripts/config_{2000,2010,2020}.py` - State district counts
**Algorithm**: `src/apportionment/partition/{recursive_bisection,metis_wrapper}.py`
**Pipeline**: `scripts/pipeline/{run_complete_redistricting,process_nation,run_state_redistricting,process_single_state}.py`
**Progress**: `scripts/utils/{progress_coordinator,terminal_utils}.py`

## Patterns

**Load Data**:
```python
from pathlib import Path
import geopandas as gpd
import pickle

# Using path utilities (recommended)
from scripts.utils.paths import get_tract_file, get_adjacency_file

tract_file = get_tract_file(state, year)  # outputs/data/{year}/units/{state}_tracts_{year}.parquet
tracts = gpd.read_parquet(tract_file)

graph_file = get_adjacency_file(state, year)  # outputs/data/{year}/adjacency/{state}_adjacency_{year}.pkl
with open(graph_file, 'rb') as f:
    adjacency = pickle.load(f)
```

**Skip Logic**:
```python
output_file = Path('output.png')
if not args.force and output_file.exists():
    if is_standalone: print(f"[SKIP] Output exists: {output_file}")
    return 0
```

**Progress**:
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
```

## Performance

**Pipeline**:
- Multi-year parallel (12 workers): 2-4h (3 years)
- Single year (4 workers): ~1h
- Subsequent runs (w/ `.states_complete`): Minutes
- Single state VT/DE: 30s-2min
- Single state CA/TX: 3-5min

**Tests**: 215 total (~25s): unit 135 (~9s), integration 24 (~3s), e2e 56 (~10s)

## Skills (Claude Code)

**Common**: `/run-{redistricting,tests}`, `/debug-{tests,pipeline}`, `/enhancement-{plan,implement}`
**See**: [SKILLS.md](SKILLS.md) - 31 total skills

## Tools

**Enhancement Manager**: `cd tools/enhancement_manager && run.bat` → http://localhost:5001

## Docs

**Start**: [README.md](../README.md), [CLAUDE.md](../CLAUDE.md)
**System**: [ARCHITECTURE.md](ARCHITECTURE.md), [RECURSIVE_BISECTION.md](../docs/RECURSIVE_BISECTION.md)
**Dev**: [CODING_PATTERNS.md](CODING_PATTERNS.md), [ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md), [CONTRIBUTING.md](../docs/CONTRIBUTING.md)
**Ref**: [TESTING.md](TESTING.md), [DATA_FORMATS.md](DATA_FORMATS.md), [DEPENDENCIES.md](../docs/DEPENDENCIES.md)
**History**: [CHANGELOG.md](../docs/CHANGELOG.md), [enhancements/INDEX.md](enhancements/INDEX.md), [archive/](archive/)
