# Claude AI Guide - Congressional Redistricting

**Updated**: 2026-01-18

## Project Context
Congressional redistricting via METIS recursive bisection → 435 districts, 50 states, 3 census years (2000/2010/2020). Purely algorithmic (no gerrymandering). Goal: compact + population-balanced districts.

**Stack**: Python 3.13+, METIS, GeoPandas, Matplotlib | **Data**: Census tracts (~40GB) | **Output**: Maps/CSVs (~20GB/run)

## Critical Files

**Core Algorithm**:
- `src/apportionment/partition/recursive_bisection.py` - Main algorithm
- `src/apportionment/partition/metis_wrapper.py` - METIS interface
- `src/apportionment/data/adjacency.py` - Graph generation

**Pipeline** (executable):
- `scripts/pipeline/run_complete_redistricting.py` - Main orchestrator (parallel multi-year)
- `scripts/pipeline/process_nation.py` - National post-processing (9 parallel tasks)
- `scripts/pipeline/run_state_redistricting.py` - Single state wrapper
- `scripts/pipeline/process_single_state.py` - Core state logic (STATUS protocol)

**Config**: `scripts/config_{2000,2010,2020}.py` - State district counts

**Analysis**: `scripts/{political,demographic,compactness}/` - Metrics

**Progress**: `scripts/utils/{progress_coordinator,terminal_utils}.py` - Hierarchical display

**Web**: `web/dashboard.html`, `scripts/web/generate_dashboard.py` - Static dashboard

**Entry**: `run_redistricting.bat`, `deploy_web.bat`, `CANCEL.bat`

## Structure
```
src/apportionment/    # Library (partition/, data/, visualization/)
scripts/              # Executables (pipeline/, data/, political/, demographic/, compactness/, web/, config_*.py)
data/{year}/          # Raw census data (redistricting/, tiger/tracts/, tiger/blocks/, demographics/, elections/)
outputs/data/{year}/  # Processed data (units/, adjacency/, places/, elections/, demographics/)
web/                  # dashboard.html, master_dashboard.html
artifacts/            # papers/, presentations/, guides/ (LaTeX)
context/              # AI context (enhancements/, archive/, patterns, architecture)
docs/                 # Human docs (RECURSIVE_BISECTION.md, DEPENDENCIES.md, CENSUS_DATA_PROCESSING.md, etc.)
tests/                # unit/ (135), integration/ (24), e2e/ (56) - 215 total, ~24s
```

## Git Rules
⚠️ **NEVER commit**: `data/`, `outputs/`, `*.{png,jpg,pdf}` (except docs/) - See `.gitignore`

## Skills (31 total)
**Common**: `/enhancement-{plan,implement}`, `/run-{redistricting,tests}`, `/debug-{tests,pipeline}`
**See**: [context/SKILLS.md](context/SKILLS.md)

## Coding Patterns

### STATUS Protocol (Progress Reporting)
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
```

### Conventions
- **State names**: lowercase_underscores (`california`, `new_york`)
- **Paths**: Use `Path` objects, not strings
- **Imports**: `from apportionment.partition...`, `from scripts.config_2020...`
- **Windows**: ⚠️ ASCII ONLY (`[OK]`/`[FAIL]`/`->`) - NO Unicode (✓✗→•) → crashes CP1252

**See**: [context/CODING_PATTERNS.md](context/CODING_PATTERNS.md) for full patterns

## Common Commands

```bash
# Pipeline (production - outputs/v1/{year}/)
run -v v1                                             # Multi-year parallel (2-4h) - doskey alias
run -y 2020 -v v1                                     # Single year (~1h)
run -y 2020 -v v1 -st CA TX NY                        # Specific states only
run -v v1 -s nation                                   # National only (fast)

# Test/debug runs (outputs/dev/{version}_{year}/)
runtest -y 2020 -v test                               # Test run - doskey alias
runtest -y 2020 -v test -st VT                        # Test single state
run -p -v test                                        # Dry run (print-only)
run -v v1 -d                                          # Debug mode (progress delays)

# Downloads (separate from pipeline - manual control)
python scripts/data/download_orchestrator.py --stages redistricting demographics --year 2020 --check-only  # Check cache
python scripts/data/download_orchestrator.py --stages redistricting --year 2020 --workers 4  # Download missing data
python scripts/data/download_orchestrator.py --type demographics --year 2020 --states VT DE  # Test with small states

# Short flags: -h=help, -y=year, -v=version, -s=stages, -st=states, -w=workers, -r=reset,
#              -p=print-only, -d=debug, -ey=election-year, -pm=partition-mode, -rt=run-type
# Long forms also work: --help, --year, --version, --stages, --states, --workers, --dpi, etc.

# Dashboard
python scripts/web/generate_master_dashboard.py
deploy_web.bat --year 2020 --version v1

# Tests
pytest tests/ -v           # All 187 tests (~23s)
pytest tests/unit/ -v      # Unit only (110 tests)
```

**See**: [context/QUICK_REFERENCE.md](context/QUICK_REFERENCE.md) for troubleshooting

## Enhancement Workflow (6 phases)
1. **Research**: Review docs/archives/dependencies
2. **Planning**: Create spec (context/enhancements/active/), identify tests needed
3. **Implementation**: TodoWrite, follow patterns, **write tests as you code** (TDD)
4. **Testing**: Automated tests (unit/integration/e2e/dashboard) → print-only → small state → multi-year
5. **Documentation**: Update ALL affected docs (mandatory)
6. **Completion**: Git commit, archive notes

**CRITICAL**: Add tests for EVERY enhancement (unit/integration/e2e/dashboard) - use decision tree in workflow doc.

**See**: [context/ENHANCEMENT_WORKFLOW.md](context/ENHANCEMENT_WORKFLOW.md)

## Test Requirements (MANDATORY)
```
Add function/class?     → unit tests (tests/unit/)
Multi-component logic?  → integration tests (tests/integration/)
Pipeline workflow?      → e2e tests (tests/e2e/)
Visualization/dashboard?→ dashboard tests (tests/e2e/)
```
**Quality**: >80% unit coverage, reliable (no flaky tests), multi-year support

## Performance
- Multi-year parallel (12 workers): 2-4h (3 census years)
- Single year (4 workers): ~1h
- Subsequent runs w/ `.states_complete`: Minutes (national only)
- Single state (VT/DE): 30s-2min
- Dashboard: ~5s

## Algorithm Constraints
- Population: ±0.5% of target
- Contiguity: All districts connected
- Compactness: METIS edge-cut minimization
- No political/racial data

## Recent Changes
- **2026-02-08**: MAUP Sensitivity Analysis (Paper 11) - Phase 2 complete: Built adjacency graphs for all 50 states at block group (239K units) and block (8.1M units) resolutions, validated multi-resolution infrastructure with 10-state subset (30 successful runs), confirmed algorithm scalability across 130× unit count range
- **2026-02-08**: Multi-resolution redistricting infrastructure - Added resolution parameter to pipeline scripts, created `run_multi_resolution_validation.py`, updated path utilities with backward compatibility for tract naming
- **2026-04-24**: Removed Wave 9 FastAPI/React app (api/, backend/, frontend/) — replaced by static dashboard + planned Rust CLI port
- **2026-01-18**: Resolution-independent restructuring - `units/` directory, `tiger/tracts/` + `tiger/blocks/` structure
- **2026-01-18**: Script renames - `download_tiger_units.py`, `merge_units_with_geometries.py` (resolution-aware)
- **2026-01-18**: Download orchestrator (Enhancement 48) - Parallel downloads, cache checking, 4-8x faster
- **2026-01-18**: Centralized download config - STATE_FIPS, CENSUS_CONFIGS single source (75 unit tests)
- **2026-01-18**: Stage-aware downloads - Check cache, skip existing data, download only what's missing
- **2026-01-18**: Census data processing (Enhancement 47) - Parse/merge/adjacency pipeline integrated
- **2026-01-18**: Path reorganization - `data/{year}/` and `outputs/data/{year}/` structure

**See**: [docs/CHANGELOG.md](docs/CHANGELOG.md)

## Common Pitfalls
1. ⚠️ Don't commit data/outputs
2. Config imports: `from scripts.config_2020 import...`
3. Progress bars: Use STATUS protocol in child processes
4. State names: lowercase_underscores
5. Unicode: NEVER in console (Windows CP1252)
6. **Tests: Add systematically for EVERY enhancement** (not ad-hoc)
7. **Error logs**: Check `outputs/{version}/{year}/error.log` for failures

## Documentation
**Start**: [CLAUDE.md](CLAUDE.md) (this), [README.md](README.md)
**System**: [ARCHITECTURE.md](context/ARCHITECTURE.md), [RECURSIVE_BISECTION.md](docs/RECURSIVE_BISECTION.md)
**Dev**: [CODING_PATTERNS.md](context/CODING_PATTERNS.md), [ENHANCEMENT_WORKFLOW.md](context/ENHANCEMENT_WORKFLOW.md), [CONTRIBUTING.md](docs/CONTRIBUTING.md)
**Ref**: [QUICK_REFERENCE.md](context/QUICK_REFERENCE.md), [SKILLS.md](context/SKILLS.md), [DATA_FORMATS.md](context/DATA_FORMATS.md), [TESTING.md](context/TESTING.md)
**History**: [CHANGELOG.md](docs/CHANGELOG.md), [enhancements/INDEX.md](context/enhancements/INDEX.md), [archive/](context/archive/)
