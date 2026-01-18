# Claude AI Guide - Congressional Redistricting

**Updated**: 2026-01-17

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
scripts/              # Executables (pipeline/, political/, demographic/, compactness/, web/, config_*.py)
web/                  # dashboard.html, master_dashboard.html
artifacts/            # papers/, presentations/, guides/ (LaTeX)
context/              # AI context (enhancements/, archive/, patterns, architecture)
docs/                 # Human docs (RECURSIVE_BISECTION.md, DEPENDENCIES.md, etc.)
tests/                # unit/ (110), integration/ (21), e2e/ (20) - 18s total
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
# Pipeline
run_redistricting.bat --version v1                    # Multi-year parallel (2-4h)
run_redistricting.bat --year 2020 --version v1        # Single year (~1h)
run_redistricting.bat --version v1 --skip-states      # National only (fast)
python scripts/pipeline/run_complete_redistricting.py --print-only  # Dry run

# Dashboard
python scripts/web/generate_master_dashboard.py
deploy_web.bat --year 2020 --version v1

# Tests
pytest tests/ -v           # All (18s)
pytest tests/unit/ -v      # Unit only
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
- **2026-01-17**: Parallel multi-year pipeline (Enhancement 37) - 60-70% faster
- **2026-01-16**: Test suite complete (151 tests, 18s) + test skills (Enhancement 34)
- **2026-01-15**: Artifacts reorg (Enhancement 29), RBA formalization

**See**: [docs/CHANGELOG.md](docs/CHANGELOG.md)

## Common Pitfalls
1. ⚠️ Don't commit data/outputs
2. Config imports: `from scripts.config_2020 import...`
3. Progress bars: Use STATUS protocol in child processes
4. State names: lowercase_underscores
5. Unicode: NEVER in console (Windows CP1252)
6. **Tests: Add systematically for EVERY enhancement** (not ad-hoc)

## Documentation
**Start**: [CLAUDE.md](CLAUDE.md) (this), [README.md](README.md)
**System**: [ARCHITECTURE.md](context/ARCHITECTURE.md), [RECURSIVE_BISECTION.md](docs/RECURSIVE_BISECTION.md)
**Dev**: [CODING_PATTERNS.md](context/CODING_PATTERNS.md), [ENHANCEMENT_WORKFLOW.md](context/ENHANCEMENT_WORKFLOW.md), [CONTRIBUTING.md](docs/CONTRIBUTING.md)
**Ref**: [QUICK_REFERENCE.md](context/QUICK_REFERENCE.md), [SKILLS.md](context/SKILLS.md), [DATA_FORMATS.md](context/DATA_FORMATS.md), [TESTING.md](context/TESTING.md)
**History**: [CHANGELOG.md](docs/CHANGELOG.md), [enhancements/INDEX.md](context/enhancements/INDEX.md), [archive/](context/archive/)

## Tools
**Enhancement Manager**: `cd tools/enhancement_manager && run.bat` → http://localhost:5001
