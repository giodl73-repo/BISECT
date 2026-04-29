# Claude AI Guide - Congressional Redistricting

**Updated**: 2026-04-29

## Project Context
Congressional redistricting via METIS recursive bisection → 435 districts, 50 states, 3 census years (2000/2010/2020). Purely algorithmic (no gerrymandering). Goal: compact + population-balanced districts.

**Stack**: Rust (`redist` CLI — primary, ~213× faster) + Python 3.13+ (dashboard, research, data download only) | METIS, GeoPandas, Matplotlib | **Data**: Census tracts (~40GB) | **Output**: Maps/CSVs (~20GB/run)

## Critical Files

**Core Algorithm** (Rust — production):
- `redist/crates/redist-core/` - bisection, edge-weighting, METIS file I/O, FIPS, population balance
- `redist/crates/redist-data/` - TIGER reading, adjacency build, .adj.bin serialization

**Pipeline** (Rust binary — production, post-cutover 2026-04-29):
- `redist run` - full pipeline orchestrator (replaces `run_complete_redistricting.py`)
- `redist state --state <CODE>` - single-state pipeline (replaces `run_state_redistricting.py` and `process_single_state.py`)
- `redist states --year <YEAR>` - parallel multi-state runner
- `redist run -s nation` - national post-processing only (replaces `process_nation.py`)

**Python pipeline (archived under `archive/python-pipeline-final/` per Plan 02 — sealed reference, not maintained):**
- The Python orchestrator scripts that previously lived in `scripts/pipeline/` are forensic reference only.
- See `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md`.

**Config**: `scripts/config_{2000,2010,2020}.py` - State district counts

**Analysis**: `scripts/{political,demographic,compactness}/` - Metrics

**Progress**: `scripts/utils/{progress_coordinator,terminal_utils}.py` - Hierarchical display

**Web**: `web/dashboard.html`, `scripts/web/generate_dashboard.py` - Static dashboard

**Rust CLI** (`~213× faster than Python`):
- `redist/crates/redist-cli/` - Binary source (`redist state`, `redist states`, `redist run`, `redist fetch`, `redist analyze`, `redist map`)
- `redist/crates/redist-map/` - Native map rendering crate (SVG→PNG via resvg, Liberation Sans embedded)
- `redist/crates/redist-analysis/` - Analytics crate (demographic, political, urban, compactness, summary)
- `docs/REDIST_CLI.md` - Full CLI reference (commands, flags, env vars)
- `scripts/data/generate_adj_bin.py` - Convert pkl adjacency files to fast `.adj.bin` format

**Entry**: `redist` binary (primary, invoked by `run` and `runtest` doskey aliases since 2026-04-29 cutover), `setup_env.bat` (sets aliases + checks `redist` is on PATH), `scripts/web/deploy_docs.py` (dashboard)

## Structure
```
redist/               # Rust workspace — production code
  crates/             # 8 crates: redist-{core,data,analysis,cli,map,report,tui,web}
  python/redist_py/   # PyO3 bridge (used by research scripts, not production)
src/apportionment/    # Python library — visualization helpers + huntington_hill (partition/, data/, visualization/maps.py archived)
scripts/              # Python executables — dashboard generation, data download, research, figures
                      # (pipeline orchestrators archived under archive/python-pipeline-final/)
archive/              # Forensic-only: archive/python-pipeline-final/ holds the last working Python pipeline
data/{year}/          # Raw census data (redistricting/, tiger/tracts/, tiger/blocks/, demographics/, elections/)
outputs/data/{year}/  # Processed data (units/, adjacency/, places/, elections/, demographics/)
web/                  # dashboard.html, master_dashboard.html
artifacts/            # papers/, presentations/, guides/ (LaTeX)
context/              # AI context (enhancements/, archive/, patterns, architecture)
docs/                 # Human docs — REDIST_CLI.md is canonical CLI reference
tests/                # unit/, integration/, e2e/, acceptance/ — pytest tests/ -v
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

# Short flags (Rust `redist run`):
#   -h=help, -y=year, -v=version, -s=stages, -w=workers, -r=reset,
#   -p=print-only, -d=debug, -e=election-year, -m=partition-mode
# Long forms (use these for clarity): --help --year --version --stages --states --workers --dpi
#                                     --run-type --partition-mode --election-year
# Note: `--states` (plural) takes a list and has NO short flag in Rust.

# Dashboard
python scripts/web/generate_master_dashboard.py
python scripts/web/deploy_docs.py --version V3 --year 2020 --out dashboard_2020.html

# Tests
pytest tests/unit/ -v      # Unit tests (~1000)
pytest tests/integration/  # Integration tests (require pipeline outputs)
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
- **2026-04-29**: **Python pipeline archived** — pipeline orchestrators (`run_complete_redistricting.py`, `run_states_parallel.py`, `process_nation.py`, `process_single_state.py`, `run_state_redistricting.py`) and the Python algorithm library (`src/apportionment/partition/`, `src/apportionment/data/`, `src/apportionment/visualization/maps.py`) moved to `archive/python-pipeline-final/`. Validation harness (`compare_rust_vs_python.py`, `validate_rust_vs_python.py`) and one-time bridge (`generate_adj_bin.py`) deleted. `redist-web` stub documented as reserved.
- **2026-04-29**: **Entry-point cutover** — `run` and `runtest` doskey aliases now invoke `redist` Rust binary directly. See `docs/superpowers/specs/2026-04-29-rust-python-final-architecture.md`.
- **2026-04-29**: **Pitfalls PP-15, PP-16, PP-17 added** — entry-point PATH preflight, rollback dependency tracking, structural sensitive-asset blocking. See `design/pitfalls/pitfalls-pipeline.md`.
- **2026-04-29**: **Louisiana v. Callais** ruling integrated — partisan edge-weighting plan (Plan 03) drafted, gated on cutover + cleanup completion.
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
