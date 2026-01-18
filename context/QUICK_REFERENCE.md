# Quick Reference

**Updated**: 2026-01-17

## Commands

### Pipeline
```bash
run_redistricting.bat --version v1                    # Multi-year parallel (all 3 years) - 2-4h
run_redistricting.bat --year 2020 --version v1        # Single year - ~1h
run_redistricting.bat --workers 12 --version v1       # Custom workers (4+4+4 allocation)
run_redistricting.bat --version v1 --reset            # Fresh run (delete outputs)
run_redistricting.bat --version v1 --skip-states      # National only (fast - minutes)
python scripts/pipeline/run_complete_redistricting.py --print-only  # Dry run

# Single state
python scripts/pipeline/run_state_redistricting.py --state CA --year 2020 --output-dir outputs/california
```

### Dashboard
```bash
python scripts/web/generate_master_dashboard.py     # Generate both dashboards
start outputs/index.html                             # Master
start outputs/us_2020_v1/index.html                 # Specific run
```

### Tests
```bash
pytest tests/ -v                 # All (18s)
pytest tests/unit/ -v            # Unit (7s)
pytest tests/e2e/ -v             # E2E (8s)
pytest tests/ --cov=src/apportionment --cov-report=html  # With coverage
pytest tests/unit/test_file.py::test_func -v       # Specific test
```

### Data Prep
```bash
python scripts/data/census/download_all_states_tracts.py --year 2020
python scripts/data/geography/build_adjacency.py --state CA --year 2020
python scripts/data/geography/check_graph_connectivity.py --year 2020
```

### Emergency
```bash
CANCEL.bat  # Kill all Python processes (Windows)
```

## File Paths

**Input**:
```
data/tracts/{year}/{state}_tracts_{year}.parquet
data/tracts/{year}/{state}_places_{year}.parquet
data/adjacency/{year}/{state}_adjacency_{year}.pkl
```

**Output**:
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

**Pipeline**:
- `--version v1`: Output version
- `--year {2020|2010|2000|all}`: Census year(s) (default: all)
- `--workers N`: Parallel workers (default: 12)
- `--dpi N`: Image DPI (default: 150)
- `--reset`: Delete existing outputs
- `--skip-states`: National post-processing only
- `--skip-analysis`: Skip per-state analysis (legacy)
- `--print-only`: Dry run
- `--force`: Override skip logic

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

tract_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')
tracts = gpd.read_parquet(tract_file)

graph_file = Path(f'data/adjacency/{year}/{state}_adjacency_{year}.pkl')
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

**Tests**: All ~18s, unit ~7s, integration ~5s, e2e ~8s

## Skills (Claude Code)

**Common**: `/run-{redistricting,tests}`, `/debug-{tests,pipeline}`, `/enhancement-{plan,implement}`
**See**: [SKILLS.md](SKILLS.md) - 31 total skills

## Tools

**Enhancement Manager**: `cd tools/enhancement_manager && run.bat` → http://localhost:5001

## Docs

**Start**: [README.md](../README.md), [CLAUDE.md](../CLAUDE.md)
**System**: [ARCHITECTURE.md](ARCHITECTURE.md), [RECURSIVE_BISECTION.md](RECURSIVE_BISECTION.md)
**Dev**: [CODING_PATTERNS.md](CODING_PATTERNS.md), [ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md), [CONTRIBUTING.md](CONTRIBUTING.md)
**Ref**: [TESTING.md](TESTING.md), [DATA_FORMATS.md](DATA_FORMATS.md), [DEPENDENCIES.md](DEPENDENCIES.md)
**History**: [CHANGELOG.md](CHANGELOG.md), [enhancements/INDEX.md](enhancements/INDEX.md), [archive/](archive/)
