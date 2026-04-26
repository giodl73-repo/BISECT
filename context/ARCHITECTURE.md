# System Architecture

**Updated**: 2026-04-25

**Related**: [CODING_PATTERNS.md](CODING_PATTERNS.md), [enhancements/INDEX.md](enhancements/INDEX.md), [SKILLS.md](SKILLS.md), [../CLAUDE.md](../CLAUDE.md)

**Skills**: `/run-redistricting`, `/pipeline-debug`, `/create-{state,national}-map`, `/generate-dashboard`, `/enhancement-{plan,implement}`

## System Overview

**Goal**: Congressional redistricting via METIS recursive bisection → 435 districts, 50 states, 3 census years (2000/2010/2020)

**Flow**: Census tract geometries + population → Recursive bisection (METIS graph partitioning) → District assignments + maps + metrics

**Why this approach?**
- vs Manual: Eliminates gerrymandering, transparent, reproducible
- vs Other algos: Fast (METIS), deterministic, compact districts

## Data Flow

```
Census API → Download tracts/places/demographics/elections
    ↓
Process to parquet, build adjacency graphs (NetworkX)
    ↓
Redistricting (per state): Tracts + Graph + Target Districts → METIS bisection → District assignments
    ↓
Analysis: Political (partisan lean), Demographic (race/gender), Compactness (Polsby-Popper/Reock)
    ↓
Visualization: District maps, round progression, analysis maps
    ↓
Outputs: CSV summaries + PNG maps + Interactive dashboard
```

### File Structure

**Input** (`data/`):
```
tracts/{year}/{state}_tracts_{year}.parquet        # Geometries + pop
tracts/{year}/{state}_places_{year}.parquet        # City points
adjacency/{year}/{state}_adjacency_{year}.pkl      # NetworkX graph
```

**Output** (`outputs/us_{year}_{version}/`):
```
states/{state}/
  ├─ data/                    # final_assignments.json (Rust) / .pkl (Python), district_summary.csv, cities, rounds
  ├─ maps/                    # all_districts.png, districts/, rounds/
  ├─ political/               # district_political.csv, maps/
  ├─ demographic/             # demographics, maps/
  └─ compactness/             # metrics, maps/
data/                         # us_*.csv (national aggregates)
maps/                         # us_*.png (national maps)
```

## Algorithm: Recursive Bisection

**Core**: Partition N tracts → K districts via repeated binary splits

**Process**:
1. Split region into 2 balanced parts (METIS finds optimal graph cut)
2. Recurse on each half until K districts
3. Depth = ⌈log₂K⌉ rounds (e.g., CA: 52 districts → 6 rounds)

**Benefits**: Fast (log₂K rounds), balanced population, compact (min edge cuts), hierarchical, visualizable

**See**: [RECURSIVE_BISECTION.md](../docs/RECURSIVE_BISECTION.md) for algorithm details

## Component Architecture

### Directory Structure

```
src/apportionment/       # Library (pure functions, no I/O)
  ├─ data/               # Loading & validation
  ├─ partition/          # METIS wrapper, bisection algorithm
  ├─ compactness/        # Metric calculations
  └─ visualization/      # Map utilities

scripts/                 # Executables (orchestration, I/O, progress)
  ├─ pipeline/           # Main workflow
  ├─ political/          # Political analysis
  ├─ demographic/        # Demographic analysis
  ├─ compactness/        # Compactness analysis
  └─ data/               # Data acquisition

data/                    # All data (gitignored)
outputs/                 # Results (gitignored)
```

### Separation: Library vs Scripts

**Library** (`src/apportionment/`): Pure functions, no I/O, unit testable, reusable
**Scripts** (`scripts/`): CLI, orchestration, progress reporting, file I/O, subprocess management

**Example**:
```python
# Library: Pure algorithm
def redistribute_recursive(graph, populations, num_districts):
    return assignments  # No I/O, no prints

# Script: Orchestration
def main():
    tracts = gpd.read_parquet(...)  # Load data
    with tqdm() as pbar:             # Progress
        assignments = redistribute_recursive(graph, pops, K)
    save_assignments(...)            # Save
```

## Key Design Decisions

### 1. Census Tracts (not blocks/precincts)
✅ Stable boundaries, ~4K people, nationwide availability, consistent data
❌ Blocks too small (7M+), counties too large, precincts vary/unstable

### 2. METIS Graph Partitioning
✅ Fast (10K+ nodes in seconds), quality (compact/balanced), proven, open source
❌ Simulated annealing (slow), genetic algos (complex), greedy (poor quality)

### 3. Recursive Bisection (not K-way)
✅ Fast (log₂K rounds), balanced, hierarchical, visualizable
❌ Slightly less compact than optimal K-way, but much faster/stable

### 4. Parquet (not CSV)
✅ Fast (columnar), compressed (5-10x smaller), typed (no int/string confusion)
CSV only for: Human-readable outputs, Excel compatibility

### 5. STATUS Message Protocol (Progress)
✅ Centralized (parent manages all bars), clean (children emit text), flexible (any depth), non-intrusive (env var detection)
❌ Alt: Each script manages own bar (cluttered, overlapping, no coordination)

**Pattern**:
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
```

### 6. Skip Logic Everywhere
✅ Resumable (restart after failures), efficient (skip recomputation), debuggable (re-run single step)

```python
if not force and output_file.exists(): return 0  # Skip
```

### 7. Scope-Based Analysis Pattern

**Single script handles both state + national scope** (no duplication):

```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')

if scope == 'state':
    # Load ONLY this state's data, create state map
elif scope == 'national':
    # Load ALL states, aggregate, create national map
```

**Architecture**:
```
Per-State (Parallel):
  process_single_state.py → analyze_districts.py → visualize_xxx.py --scope state

Post-Processing (Sequential):
  run_complete_redistricting.py → visualize_xxx.py --scope national
```

**Benefits**: Parallel per-state (saves 1-2h), national once (aggregates results), incremental (re-run national without states), consistent code

**Implemented**: ✅ Compactness, Political, Demographic

## Parallel Multi-Year Execution

**Added**: Enhancement 37 (2026-01-17)

**Architecture**:
```
Main Process (Coordinator)
├─ Year 2020 (2 workers) → States → National (9 parallel tasks)
├─ Year 2010 (1 worker) → States → National (9 parallel tasks)
└─ Year 2000 (1 worker) → States → National (9 parallel tasks)
```

**Worker Allocation**: 4→[2,1,1], 6→[2,2,2], 8→[3,3,2], 12→[4,4,4]

**STATUS Protocol**:
- `STATUS:YEAR:2020:COMPLETE:24/50` - Year progress
- `STATUS:WORKER:2020:1:STATE:12/50:california:STAGE:3/7:district_maps` - Worker status
- `STATUS:YEAR:2020:POSTPROCESS:3/9` - National tasks
- `STATUS:WORKER:2020:1:TASK:3/9:National_district_map` - Task status

**Hierarchical Display**:
```
[2020] ████████░░░░ 24/50 states
  ├─ Worker 1: [12/50] California | Stage 3/7: District Maps
  └─ Worker 2: [13/50] Texas      | Stage 5/7: Political Analysis
```

### `.states_complete` Markers (Smart Iteration)

**Problem**: State processing = 2-4h, but users iterate on national viz (maps/dashboards)

**Solution**: After successful state processing → create `outputs/v1/{year}/.states_complete` marker

**Benefits**: Subsequent runs see marker → skip states → run national only → Hours→Minutes

```bash
# First run: Full (creates marker)
run -v v1                                             # doskey alias (setup_env.bat)

# Second run: Fast! (sees marker, skips states)
run -v v1

# Force rerun: Ignores marker
run -v v1 -r                                          # -r = --reset

# Explicit skip: Skip states without marker
run -v v1 --skip-states
```

### Parallel National Post-Processing

**Before**: All 3 years finish states → then sequential national
**After**: Each year launches national immediately after states finish → 3 national phases run **in parallel**

```
2020: States (1-2h) → National (15min) → Done
2010: States (1-2h) → National (15min) → Done  } All 3 national phases
2000: States (1-2h) → National (15min) → Done  } run in parallel
```

**Performance**:
- First run: 2-4h (all 3 years)
- Subsequent (w/ markers): Minutes
- Time reduction: 60-70% vs sequential

**Files**: `progress_coordinator.py`, `terminal_utils.py`, `process_nation.py`

### Pipeline Orchestrator Pattern

**Added**: 2026-01-18 (Refactor)

**Purpose**: Unified subprocess management for multi-stage, multi-year pipeline execution with automatic phase transitions and completion tracking.

**Architecture**:
```
PipelineOrchestrator
├─ ProcessMonitor (low-level)
│  ├─ Spawn subprocess with line-buffered stdout
│  ├─ Monitor via daemon thread + readline loop
│  ├─ Parse STATUS messages
│  └─ Route to handlers (coordinator updates)
│
└─ PipelineOrchestrator (high-level)
   ├─ Stage registration (census → states → nation)
   ├─ Sequential stage execution (all years complete stage before next)
   ├─ Marker file tracking (.census_complete, .states_complete, .nation_complete)
   ├─ Automatic phase transitions
   └─ Unified error handling
```

**Key Components**:

1. **ProcessMonitor** (`scripts/utils/pipeline_orchestrator.py`):
   - Low-level subprocess wrapper
   - Non-blocking readline loop (NOT `for line in stdout` - blocks until EOF!)
   - Thread-safe message routing
   - Proper process cleanup

2. **PipelineOrchestrator** (`scripts/utils/pipeline_orchestrator.py`):
   - High-level multi-stage, multi-year coordinator
   - Sequential stages: All years finish census → all years finish states → all years finish nation
   - Marker files: `.{stage}_complete` for skip logic
   - Completion callbacks for phase transitions

**Usage Pattern**:
```python
# Create orchestrator
orchestrator = PipelineOrchestrator(
    coordinator=coordinator,
    display_lock=display_lock,
    years=['2020', '2010', '2000'],
    output_dirs={'2020': Path(...), ...}
)

# Register stages
orchestrator.add_stage('census', census_command_builder, census_handlers)
orchestrator.add_stage('states', states_command_builder, state_handlers)
orchestrator.add_stage('nation', nation_command_builder, nation_handlers)

# Run pipeline
results = orchestrator.run_pipeline(
    stages=['census', 'states', 'nation'],
    skip_stages_if_complete=True,
    reset=False
)
```

**Benefits**:
- ✅ Centralized subprocess management (no scattered Popen calls)
- ✅ Automatic marker file handling (skip completed stages)
- ✅ Thread-safe coordinator updates (no race conditions)
- ✅ Clean error handling (per-stage failure tracking)
- ✅ Testable components (31 unit + integration tests)

**Critical Pattern - Non-Blocking Readline**:
```python
# ❌ WRONG - Blocks until EOF (process exit)
for line in proc.stdout:
    process(line)

# ✅ CORRECT - Non-blocking with poll() check
while True:
    line = proc.stdout.readline()
    if not line:
        if proc.poll() is not None:  # Process exited
            break
        continue  # Empty line, keep reading
    process(line)
```

**Files**: `scripts/utils/pipeline_orchestrator.py`, `scripts/pipeline/run_complete_redistricting.py`
**Tests**: `tests/unit/test_pipeline_orchestrator.py` (19 tests), `tests/integration/test_pipeline_orchestrator_integration.py` (12 tests)

## Scalability

**Current**:
- 50 states, 3 census years (parallel)
- ~84K tracts nationwide
- 435 total districts
- Multi-year parallel: 2-4h (first run), minutes (subsequent w/ markers)
- Single year: ~1h

**Bottlenecks**:
1. METIS: O(N log N) - not a bottleneck (5-30min/state)
2. Map rendering: O(N²) complex geometries - bottleneck for CA/TX (solution: configurable DPI, skip logic)
3. I/O: Minor (solution: Parquet compression)

**Parallelization**: State-level (natural unit, independent, load balancing)
❌ Not tract-level: Recursive bisection serial, METIS already parallelizes internally

**Future**:
- 100 states: Current architecture handles (add workers)
- 10x tracts (blocks): METIS handles (tested 100K+ nodes), map rendering slower (lower DPI/simplified geometries)
- More analysis types: Easily extended (add scripts, integrate into pipeline)

## Web Dashboard

**Architecture**: Single-page app (HTML/CSS/JS) → Zero dependencies, works offline, deployable anywhere

**Design**:
- Single HTML file: `web/dashboard.html` (source) → `outputs/index.html` (deployed)
- Hash-based nav: `#us_2020_v1` switches output dir (bookmarkable, shareable)
- CSS Grid: Responsive (rounds max 2/row, districts auto-fit)

**Navigation** (3-level):
1. Year/Version: Top selector (us_2020_v1, us_2030_v1)
2. State: Sidebar (Alabama → Wyoming)
3. Dimension: Tabs (Overview, Districts, Rounds, Political, Demographics, Compactness, Urban)

**Deployment**:
```bash
python scripts/pipeline/run_complete_redistricting.py --workers 4
python web/deploy_dashboard.py
open outputs/index.html
```

**Path Resolution**: Dashboard reads year/version from URL hash → constructs paths: `{year}_{version}/states/{state}/...` (no config needed)

## Summary

**Principles**:
1. Separation: Library (algorithm) vs Scripts (orchestration)
2. Data Flow: Raw → Processed → Analyzed → Visualized
3. Scalability: State-level parallelism, efficient algorithms
4. Reproducibility: Deterministic, version-controlled
5. Extensibility: Easy to add analysis/visualization
6. Maintainability: Clear patterns, documented conventions

---

## Rust CLI (Complete — 2026-04-25)

The `redist` binary is the recommended entry point for all redistricting runs.
Python pipeline remains for post-processing analysis (maps, political/demographic CSVs).

**Performance**: 50-state run: Python ~55 min → Rust ~15.5 s (**213× faster**)

### 5-Crate Workspace (`redist/`)

```
redist/
├── crates/
│   ├── redist-core/        Graph, Partition, BisectionTree, VRA edge weights
│   ├── redist-data/        AdjacencyGraph, .adj.bin serialization, adjacency loader
│   ├── redist-cli/         Binary: state, states, run, fetch subcommands
│   └── redist-analysis/    Compactness (PP, Reock, CHR), VRA analysis
└── python/
    └── redist_py/          PyO3 bindings (Python calls into redist-core/data)
```

| Crate | Python equivalent | Key types |
|-------|-------------------|-----------|
| `redist-core` | `src/apportionment/partition/` | `Graph`, `Partition`, `BisectionTree`, `build_vra_edge_weights` |
| `redist-data` | `src/apportionment/data/` | `AdjacencyGraph`, `serialize_adjacency`, `deserialize_adjacency` |
| `redist-cli` | `scripts/pipeline/run_*.py` | `StateConfig`, `run_states_parallel`, `load_adjacency` |
| `redist-analysis` | `scripts/compactness/`, `scripts/political/` | `CompactnessMetrics`, `VraAnalysis` |

### Adjacency Data Flow

```
redist fetch --release           → downloads {state}_adjacency_{year}.pkl
generate_adj_bin.py              → converts pkl → .adj.bin (v2) + _geoids.json
adjacency_loader.rs              → loads .adj.bin natively (zero Python subprocess)
                                   falls back to pkl shim if .adj.bin absent
```

**.adj.bin format v2** (`RADJ` magic, little-endian): header (n_vertices, n_edges) →
vertex_weights (i64×n) → adjacency lists → edge_weights (f64). See `serialize.rs`.

### Key Design Decisions (Rust)

**REDIST_PYTHON** env var: Rust binary uses caller's Python for pkl shim — avoids
environment mismatch (PP-04).

**REDIST_GH** env var: Override `gh` binary for `redist fetch --release` — enables
testing without network (fake gh script).

**Rayon for parallelism**: All 50 states run concurrently on a thread pool sized to
`--workers`. Each state is independent (separate METIS invocations).

**ufactor_for_depth()**: Returns decimal (1.001–1.005), not integer. METIS expects
`ufactor` as an imbalance tolerance multiplier (1 + ε), not a percentage (PP-07).

**See**: [`design/rust-port/`](../design/rust-port/) for migration log and benchmarks.
[`docs/REDIST_CLI.md`](../docs/REDIST_CLI.md) for command reference.

---

**Algorithm**: Recursive bisection w/ METIS → Fast (O(N log K)), quality (compact/balanced), scalable (all 50 states)

**Philosophy**: Simple > complex, Fast > perfect, Reproducible > novel, Practical > academic

