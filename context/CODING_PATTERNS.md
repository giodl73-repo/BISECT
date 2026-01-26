# Coding Patterns

**Updated**: 2026-01-25

**Related**: [ARCHITECTURE.md](ARCHITECTURE.md), [ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md), [../CLAUDE.md](../CLAUDE.md)

## Progress Bar Protocol (STATUS)

**Env Var**: `TQDM_POSITION` - Coordinates parent-child progress bars
**Unified Module**: `scripts/utils/status_protocol.py` - Centralized STATUS protocol implementation

### Recommended Pattern (Unified StatusReporter)

**Child Process (generating STATUS messages)**:
```python
from scripts.utils.status_protocol import StatusReporter

# Initialize reporter (auto-detects TQDM_POSITION)
reporter = StatusReporter()

# Report basic progress
reporter.report("Processing california")

# Report CENSUS stage progress
reporter.report_census_stage("2020", "Parsing PL 94-171", 5, 50)

# Report CENSUS worker activity
reporter.report_census_worker("2020", 0, 5, "CA", 1, 3, "Parsing PL 94-171")

# Report YEAR completion
reporter.report_year_complete("2020", 24, 50)

# Report WORKER state
reporter.report_worker_state("2020", 1, 12, "california", 3, 7, "political_visualization")

# Child behavior (automatic based on TQDM_POSITION)
if reporter.is_standalone:
    print("Headers OK")  # Standalone: show all output
    pbar = tqdm(...)
else:
    # Parent-called: suppress banners, emit STATUS
    reporter.report(f"Processing X...")
    pbar = tqdm(..., disable=True)  # No child bars
```

**Legacy Pattern (for backwards compatibility)**:
```python
# Still supported via scripts/utils/common.py
from scripts.utils.common import report_progress

report_progress("Processing california")  # Auto-routes to StatusReporter
```

**Parent Monitoring**:
```python
# Option 1: Pass via environment variable (all child processes inherit)
env = os.environ.copy()
env['TQDM_POSITION'] = '999'  # Signal deeply nested child
proc = subprocess.Popen(cmd, env=env, stdout=PIPE, stderr=sys.stderr, text=True, bufsize=1)

# Option 2: Pass via command-line argument (explicit control)
cmd = ['python', 'script.py', '--position', '999', '--year', '2020']
proc = subprocess.Popen(cmd, stdout=PIPE, stderr=sys.stderr, text=True, bufsize=1)

# CRITICAL: Use readline() loop, NOT 'for line in stdout' (blocks until EOF!)
while True:
    line = proc.stdout.readline()
    if not line:
        if proc.poll() is not None:
            break
        continue

    if line.startswith("STATUS:"):
        _, pos, msg = line.split(":", 2)
        progress_bars[int(pos)].set_description_str(msg)

# Ensure process terminated
if proc.returncode is None:
    proc.wait()
```

**Rules**:
- ⚠️ No banners if `send_status`
- ⚠️ Always `flush=True`
- ⚠️ Use `Popen` (not `run` w/ `capture_output`)
- ⚠️ Child bars: `disable=send_status`
- ⚠️ **NEVER** use `for line in proc.stdout:` - blocks until EOF (see Subprocess Pattern below)
- ⚠️ Use `stderr=sys.stderr` not `stderr=PIPE` - pipes block if not read
- ⚠️ Use `--position 999` (or `TQDM_POSITION=999`) for deeply nested children to suppress verbose output
- ⚠️ Position values: `-1` = standalone, `0-50` = specific bar, `999` = deeply nested (suppress all)

## Subprocess Pattern (CRITICAL)

**Problem**: Capturing stdout/stderr with `PIPE` causes blocking if not read properly.

**Anti-Patterns** ❌:
```python
# BLOCKS until process terminates and closes stdout!
proc = subprocess.Popen(cmd, stdout=PIPE)
for line in proc.stdout:  # ❌ Waits for EOF
    process(line)

# Pipes fill and block if not read!
proc = subprocess.Popen(cmd, stderr=PIPE)  # ❌ stderr never read
proc.wait()  # Process hangs when stderr pipe fills (65KB buffer)
```

**Correct Pattern** ✅:
```python
# Option 1: Non-blocking readline loop (for real-time monitoring)
proc = subprocess.Popen(cmd, stdout=PIPE, stderr=sys.stderr, text=True, bufsize=1)

while True:
    line = proc.stdout.readline()
    if not line:
        if proc.poll() is not None:  # Process exited
            break
        continue  # Empty line, keep reading

    process(line)

if proc.returncode is None:
    proc.wait()

# Option 2: Let stderr flow through (don't capture)
proc = subprocess.Popen(cmd, stdout=PIPE, stderr=sys.stderr)  # ✅ stderr flows to console

# Option 3: Suppress stderr entirely
proc = subprocess.Popen(cmd, stdout=PIPE, stderr=subprocess.DEVNULL)  # ✅ stderr discarded
```

**Key Rules**:
- ⚠️ **NEVER** use `for line in proc.stdout:` - it blocks until EOF
- ⚠️ **NEVER** use `stderr=PIPE` unless you read from it in a separate thread
- ⚠️ Use `stderr=sys.stderr` (flow through) or `stderr=DEVNULL` (discard)
- ⚠️ Always use `readline()` loop with `poll()` check for real-time output
- ⚠️ Use `bufsize=1` for line buffering

**References**: Commits be6156b, 023268b

## Skip Logic

**Pattern**:
```python
output_file = Path('output.png')

if not args.force and output_file.exists():
    if is_standalone: print(f"[SKIP] Output exists: {output_file}")
    return 0  # Exit silently if called from parent

# Do work...
```

**Benefits**: Resumable, efficient, debuggable

## DPI Threading

**Pattern**:
```python
# CLI
parser.add_argument('--dpi', type=int, default=150)

# Thread through all levels
visualize_xxx(..., dpi=args.dpi)

# Use in matplotlib
fig.savefig(output_file, dpi=dpi, bbox_inches='tight')
```

**Never hardcode DPI** in leaf functions!

## GEOID Handling (CRITICAL)

**Problem**: GEOIDs read as int (losing leading zeros) → merge fails

**Solution**: Force string + zero-pad to 11 chars

```python
# ✅ Loading
tracts = gpd.read_parquet(file)
tracts['GEOID'] = tracts['GEOID'].astype(str).str.zfill(11)

# ✅ Merging
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)
merged = df1.merge(df2, on='GEOID')

# ❌ Wrong
tracts['GEOID']  # int64 → 6001400100 (should be '06001400100')
```

**Always**: `.astype(str).str.zfill(11)` before any GEOID operations

## State Code Handling (CRITICAL)

**Three formats** - must convert before merging:

```python
# 2-letter: Census data
state_codes = ['AL', 'CA', 'TX']

# Full uppercase: Pipeline outputs
state_names = ['ALABAMA', 'CALIFORNIA', 'TEXAS']

# Lowercase underscore: Directories/filenames
file_prefixes = ['alabama', 'california', 'texas']

# Mapping (CRITICAL for merges)
STATE_CODE_TO_NAME = {
    'AL': 'ALABAMA', 'CA': 'CALIFORNIA', 'TX': 'TEXAS', ...
}

# Convert before merge
df1['STATE'] = df1['STATE_CODE'].map(STATE_CODE_TO_NAME)
df2['STATE']  # Already in uppercase format
merged = df1.merge(df2, on='STATE')
```

**Without mapping**: Merge produces 0 rows (silent failure!)

## Map Visualization

### Boundaries (Thin White + Thick Black)

**Pattern**:
```python
# Tracts: Black outline + light fill
ax.add_geometries(tracts.geometry, crs=ccrs.PlateCarree(),
                  facecolor='#f0f0f0', edgecolor='black', linewidth=0.5)

# Districts: Thick white + thick black (clear even with overlaps)
for district_geom in districts.geometry:
    ax.add_geometries([district_geom], crs=ccrs.PlateCarree(),
                      facecolor='none', edgecolor='white', linewidth=4.0)  # White "halo"
    ax.add_geometries([district_geom], crs=ccrs.PlateCarree(),
                      facecolor='none', edgecolor='black', linewidth=2.0)  # Black line
```

### City Labels

```python
# Cities with white halo (readable on any background)
for _, city in cities.iterrows():
    ax.text(city.geometry.x, city.geometry.y, city['NAME'],
            fontsize=8, ha='center', va='bottom',
            path_effects=[
                pe.withStroke(linewidth=3, foreground='white'),  # Halo
                pe.Normal()
            ])
```

### Colormaps

```python
# Political: Blue→Purple→Red
cmap = plt.cm.RdBu_r  # Red (R) to Blue (D)

# Demographic: Sequential
cmap = plt.cm.YlOrRd  # Yellow→Orange→Red (increasing)

# Compactness: Diverging (low=bad, high=good)
cmap = plt.cm.RdYlGn  # Red (bad) → Yellow → Green (good)
```

### Saving

```python
fig.savefig(output_file, dpi=dpi, bbox_inches='tight')
plt.close(fig)  # Free memory
```

## Subprocess Communication

### Calling Children

```python
cmd = f'{sys.executable} scripts/xxx/child.py --state {state} --dpi {dpi}'

if send_status:
    # Parent mode: Pass position, capture output
    env = os.environ.copy()
    env['TQDM_POSITION'] = str(child_pos)
    proc = subprocess.Popen(cmd, shell=True, env=env,
                           stdout=PIPE, stderr=PIPE, text=True, bufsize=1)

    for line in proc.stdout:
        if line.startswith("STATUS:"):
            # Update progress bar

    returncode = proc.wait()
else:
    # Standalone: Direct call (inherit stdout/stderr)
    returncode = subprocess.run(cmd, shell=True).returncode

if returncode != 0:
    print(f"[FAIL] Child failed: {cmd}")
    return returncode
```

**Key**: Use `sys.executable` (not "python"), pass all params

## Scope-Based Analysis Pattern

**Single script handles state + national** (no duplication):

```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')
parser.add_argument('--state', help='State code (required if scope=state)')
parser.add_argument('--state-dir', type=Path, help='State dir (required if scope=state)')
parser.add_argument('--output-dir', type=Path, help='Base dir (required if scope=national)')
parser.add_argument('--version', help='Version (required if scope=national)')
parser.add_argument('--census-year', default='2020', choices=['2020', '2010', '2000'])
parser.add_argument('--dpi', type=int, default=150)
parser.add_argument('--force', action='store_true')

if args.scope == 'state':
    if not args.state or not args.state_dir:
        parser.error("--state and --state-dir required when scope=state")
    return visualize_state(args.state_dir, args.state, args.census_year, args.dpi)

elif args.scope == 'national':
    if not args.output_dir or not args.version:
        parser.error("--output-dir and --version required when scope=national")
    return visualize_national(args.output_dir, args.version, args.census_year, args.dpi)
```

**Benefits**: Per-state runs in parallel (saves 1-2h), national once (aggregates), consistent code

## File Naming Conventions

### States

```
{state}_districts.png                    # All districts overview
{state}_{num_districts}_districts.png    # With count (e.g., california_52_districts.png)
district_{dd}.png                        # Individual districts (zero-padded)
round_{dd}.png                           # Bisection rounds (zero-padded)
{state}_political_lean.png               # Analysis maps
{state}_demographics.png
```

### National

```
us_national_map.png
us_national_map_with_cities.png
us_political_lean.png
us_demographics.png
us_compactness.png
us_round_progression_round_{dd}.png
```

### Data

```
district_summary.csv                     # Per-district metrics
district_cities.csv                      # City assignments
district_political.csv                   # Political analysis
district_demographics.csv                # Demographic analysis
district_compactness.csv                 # Compactness metrics
rounds_hierarchy.csv                     # Bisection tree
```

## Git Commit Messages

**Format**:
```
<type>: <subject>

<body>

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Types**: feat, fix, refactor, docs, test, chore, perf

**Example**:
```
feat: Add parallel multi-year pipeline execution

Implement parallel execution across census years (2020/2010/2000) with:
- Hierarchical progress display via STATUS protocol
- Worker allocation algorithm (4→[2,1,1], 12→[4,4,4])
- .states_complete markers for fast iteration (hours→minutes)
- Parallel national post-processing (60-70% time reduction)

Files modified:
- scripts/pipeline/run_complete_redistricting.py
- scripts/utils/progress_coordinator.py
- scripts/utils/terminal_utils.py

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Error Handling

### Return Codes

```python
def main():
    try:
        # ... work ...
        return 0  # Success
    except FileNotFoundError as e:
        if is_standalone: print(f"[FAIL] File not found: {e}")
        return 1  # Error
    except Exception as e:
        if is_standalone: print(f"[ERROR] {type(e).__name__}: {e}")
        return 2  # Unexpected error
```

### Graceful Degradation

```python
# Optional data - skip if missing (don't crash)
election_data_file = Path(f'data/elections/{year}_president_tract.parquet')
election_available = election_data_file.exists()

if election_available:
    # Run political analysis
else:
    if is_standalone: print(f"[SKIP] Election data not available for {year}")
```

## Version, Year, and Output Directory

**Convention**:
```
outputs/us_{year}_{version}/
```

**Threading**:
```python
parser.add_argument('--year', default='2020', choices=['2020', '2010', '2000'])
parser.add_argument('--version', required=True)

output_dir = Path(f'outputs/us_{args.year}_{args.version}')
output_dir.mkdir(parents=True, exist_ok=True)

# Pass to children
cmd = f'... --year {args.year} --version {args.version}'
```

## When to Use What

### Path Objects vs Strings

✅ **Use `Path`**:
```python
from pathlib import Path

tract_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')
if tract_file.exists():
    tracts = gpd.read_parquet(tract_file)

output_dir = Path('outputs') / 'us_2020_v1'
output_dir.mkdir(parents=True, exist_ok=True)
```

❌ **Don't use strings**:
```python
tract_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'  # Hard to check existence
```

### Print vs Logging

✅ **Use print** (scripts are short-lived, not services):
```python
if is_standalone: print(f"[OK] Completed {state}")
if send_status: report_progress(f"Completed {state}")
```

❌ **Don't use logging** (overkill for batch scripts)

### subprocess.run vs Popen

✅ **Use `Popen`** if need real-time output:
```python
proc = subprocess.Popen(cmd, stdout=PIPE, text=True, bufsize=1)
for line in proc.stdout:
    # Process line-by-line
```

✅ **Use `run`** if don't need real-time:
```python
result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
if result.returncode != 0:
    print(result.stderr)
```

## Anti-Patterns (Don't Do This!)

### ❌ Hardcoded DPI

```python
# Wrong - breaks user control
fig.savefig('output.png', dpi=150)

# Right - thread from args
fig.savefig('output.png', dpi=args.dpi)
```

### ❌ Assuming GEOID Type

```python
# Wrong - may be int or str
merged = df1.merge(df2, on='GEOID')  # Fails if one is int, one is str

# Right - force str + zero-pad
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)
merged = df1.merge(df2, on='GEOID')
```

### ❌ Direct State Code Merge

```python
# Wrong - format mismatch (AL vs ALABAMA)
merged = census_df.merge(pipeline_df, on='STATE')  # 0 rows!

# Right - convert first
census_df['STATE'] = census_df['STATE_CODE'].map(STATE_CODE_TO_NAME)
merged = census_df.merge(pipeline_df, on='STATE')
```

### ❌ Missing Skip Logic

```python
# Wrong - recomputes every time
create_expensive_visualization()

# Right - skip if exists
if not args.force and output_file.exists():
    if is_standalone: print(f"[SKIP] {output_file}")
    return 0
create_expensive_visualization()
```

### ❌ Printing Banners in Child

```python
# Wrong - clutters parent output
pos = int(os.environ.get('TQDM_POSITION', '-1'))
print("="*80)  # Always prints!
print("State Redistricting Script")
print("="*80)

# Right - check is_standalone
is_standalone = pos < 0
if is_standalone:
    print("="*80)
    print("State Redistricting Script")
    print("="*80)
```

### ❌ Unicode in Console (Windows)

```python
# Wrong - crashes Windows CP1252
print("✓ Complete")  # UnicodeEncodeError

# Right - ASCII only
print("[OK] Complete")
```

### ❌ STATUS Without flush

```python
# Wrong - parent won't see (buffered)
print(f"STATUS:{pos}:{msg}")

# Right - flush immediately
print(f"STATUS:{pos}:{msg}", flush=True)
```

## Static HTML Generation

**Pattern** (bake data into template):

```python
# Read template
with open('web/dashboard.html', 'r') as f:
    html_template = f.read()

# Load data
districts = pd.read_csv('outputs/us_2020_v1/data/us_district_summary.csv')

# Convert to JS
district_data_js = json.dumps(districts.to_dict('records'), indent=2)

# Inject into template (replace marker)
html_output = html_template.replace(
    '/* DISTRICT_DATA_PLACEHOLDER */',
    f'const districtData = {district_data_js};'
)

# Write
with open('outputs/index.html', 'w') as f:
    f.write(html_output)
```

**Benefits**: Zero dependencies, works offline, fast load, no server needed

## Quick Reference

**Starting new script**:
- [ ] `--dpi` parameter (if viz)
- [ ] `--force` parameter (skip logic)
- [ ] Check `TQDM_POSITION`
- [ ] `report_progress()` function
- [ ] `.zfill(11)` for GEOIDs
- [ ] `STATE_CODE_TO_NAME` mapping (if state data)
- [ ] File naming conventions
- [ ] Skip logic for outputs
- [ ] Thin white + thick black boundaries (maps)

**Debugging**:
- [ ] GEOID types (`.astype(str).str.zfill(11)`)
- [ ] State codes (AL vs ALABAMA vs alabama)
- [ ] `TQDM_POSITION` passed
- [ ] `flush=True` on STATUS
- [ ] DPI threaded
- [ ] `Path` objects (not strings)

**Committing**:
- [ ] Conventional commit format
- [ ] Detailed description
- [ ] Co-Authored-By line
- [ ] Test before commit

---

## API Patterns (FastAPI Backend) - Wave 9

These patterns apply to the FastAPI backend added in Wave 9.

### Configuration with pydantic-settings

```python
# api/config.py
from pydantic_settings import BaseSettings
from typing import List

class Settings(BaseSettings):
    database_url: str = "postgresql://apportionment:dev@localhost:5434/apportionment"
    cors_origins: List[str] = ["http://localhost:3002"]
    debug: bool = False
    project_root: str = "."
    outputs_dir: str = "outputs"

    # Pipeline defaults
    default_workers: int = 4
    watchdog_timeout: int = 60

    class Config:
        env_file = ".env"

settings = Settings()
```

### Database Session Dependency

```python
# api/database.py
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, Session
from typing import Generator

from .config import settings

engine = create_engine(
    settings.database_url,
    pool_pre_ping=True,
    pool_size=5,
    max_overflow=10
)

SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

def get_db() -> Generator[Session, None, None]:
    """FastAPI dependency for database session."""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
```

### REST Endpoint Pattern

```python
# api/routers/runs.py
from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session
from typing import Optional

from ..database import get_db
from ..schemas.run import RunCreate, RunResponse, RunListResponse
from ..services import run_service

router = APIRouter(prefix="/api/v1/runs", tags=["runs"])

@router.get("", response_model=RunListResponse)
async def list_runs(
    status: Optional[str] = Query(None, description="Filter by status"),
    year: Optional[str] = Query(None, description="Filter by year"),
    limit: int = Query(50, ge=1, le=100),
    offset: int = Query(0, ge=0),
    db: Session = Depends(get_db)
):
    """List all pipeline runs with optional filtering."""
    runs, total = run_service.list_runs(db, status=status, year=year, limit=limit, offset=offset)
    return RunListResponse(runs=runs, total=total, limit=limit, offset=offset)

@router.post("", response_model=RunResponse, status_code=201)
async def create_run(run_create: RunCreate, db: Session = Depends(get_db)):
    """Create a new pipeline run."""
    return run_service.create_run(db, run_create)

@router.get("/{run_id}", response_model=RunDetailResponse)
async def get_run(run_id: int, db: Session = Depends(get_db)):
    """Get detailed run information."""
    run = run_service.get_run(db, run_id)
    if not run:
        raise HTTPException(status_code=404, detail="Run not found")
    return run
```

### Pydantic Schema Pattern

```python
# api/schemas/run.py
from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime
from enum import Enum

class RunStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"

# Separate schemas for create vs response
class RunCreate(BaseModel):
    """Schema for creating a new run."""
    version: str = Field(..., min_length=1, max_length=50)
    years: List[str] = Field(..., min_items=1)
    states: Optional[List[str]] = None  # None = all states
    workers: int = Field(4, ge=1, le=16)
    dpi: int = Field(150, ge=72, le=600)
    partition_mode: str = Field("edge-weighted")

    class Config:
        json_schema_extra = {
            "example": {
                "version": "v1",
                "years": ["2020"],
                "workers": 4
            }
        }

class RunResponse(BaseModel):
    """Schema for run response."""
    id: int
    version: str
    status: RunStatus
    years: List[str]
    created_at: datetime

    class Config:
        from_attributes = True  # Enable ORM mode
```

### Error Handling Pattern

```python
# api/utils/exceptions.py
from fastapi import HTTPException, status

class APIError(HTTPException):
    """Base API exception."""
    def __init__(self, detail: str, status_code: int = 500):
        super().__init__(status_code=status_code, detail=detail)

class NotFoundError(APIError):
    def __init__(self, resource: str, id: any):
        super().__init__(f"{resource} with id {id} not found", status.HTTP_404_NOT_FOUND)

class ConflictError(APIError):
    def __init__(self, detail: str):
        super().__init__(detail, status.HTTP_409_CONFLICT)

# Usage
@router.post("/{run_id}/actions/cancel")
async def cancel_run(run_id: int, db: Session = Depends(get_db)):
    run = run_service.get_run(db, run_id)
    if not run:
        raise NotFoundError("Run", run_id)
    if run.status != RunStatus.RUNNING:
        raise ConflictError(f"Cannot cancel run in {run.status} state")
    return run_service.cancel_run(db, run_id)
```

### Async Subprocess Pattern

```python
# api/workers/executor.py
import asyncio
from pathlib import Path
from typing import Callable, Optional

class PipelineExecutor:
    """Async wrapper for pipeline subprocess."""

    def __init__(self, run_id: int, command: list[str], on_progress: Callable, on_complete: Callable):
        self.run_id = run_id
        self.command = command
        self.on_progress = on_progress
        self.on_complete = on_complete
        self.process: Optional[asyncio.subprocess.Process] = None
        self._cancelled = False

    async def start(self):
        """Start subprocess and monitor output."""
        env = {
            **os.environ,
            "TQDM_POSITION": "999",
            "PYTHONUNBUFFERED": "1",
        }

        self.process = await asyncio.create_subprocess_exec(
            *self.command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            env=env,
        )

        # Monitor stdout for STATUS messages
        asyncio.create_task(self._monitor_stdout())

        # Wait for completion
        returncode = await self.process.wait()
        await self.on_complete(returncode)

    async def _monitor_stdout(self):
        """Parse STATUS messages from stdout."""
        async for line in self.process.stdout:
            decoded = line.decode().strip()
            if decoded.startswith("STATUS:"):
                progress = self._parse_status(decoded)
                if progress:
                    await self.on_progress(progress)

    async def cancel(self, timeout: float = 5.0):
        """Cancel with graceful shutdown."""
        self._cancelled = True
        if self.process:
            self.process.terminate()
            try:
                await asyncio.wait_for(self.process.wait(), timeout=timeout)
            except asyncio.TimeoutError:
                self.process.kill()
```

### STATUS Protocol Bridge Pattern

```python
# api/utils/status_bridge.py
from scripts.utils.status_protocol import parse_status_message

class StatusBridge:
    """Bridge CLI STATUS protocol to API progress structure."""

    def __init__(self):
        self.progress = {}

    def process_line(self, line: str) -> Optional[dict]:
        """Parse STATUS line and update progress."""
        msg_type, data = parse_status_message(line)
        if msg_type is None:
            return None

        year = data.get("year")

        if msg_type == "YEAR":
            self.progress.setdefault("years", {})[year] = {
                "states_completed": data["completed"],
                "states_total": data["total"],
                "status": "running"
            }

        elif msg_type == "WORKER":
            workers = self.progress.setdefault("years", {}).setdefault(year, {}).setdefault("workers", {})
            workers[str(data["worker_id"])] = {
                "state": data["state_name"],
                "stage": f"{data['stage']}/{data['stage_total']}",
            }

        return self.progress
```

### File-Based Progress Fallback Pattern

```python
# api/utils/file_progress.py
import json
import aiofiles
from pathlib import Path

class FileProgressManager:
    """Atomic file-based progress for reliability."""

    def __init__(self, progress_file: Path):
        self.progress_file = progress_file

    async def write(self, progress: dict):
        """Atomic write: temp file + rename."""
        temp = self.progress_file.with_suffix(".tmp")
        async with aiofiles.open(temp, "w") as f:
            await f.write(json.dumps(progress, indent=2))
        temp.rename(self.progress_file)

    async def read(self) -> Optional[dict]:
        if not self.progress_file.exists():
            return None
        async with aiofiles.open(self.progress_file, "r") as f:
            return json.loads(await f.read())
```

---

## React Patterns (Frontend) - Wave 9

These patterns apply to the React dashboard added in Wave 9.

### Data Fetching with React Query

```typescript
// features/runs/hooks.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { runApi } from '@/api/runs';

export function useRuns(filters?: RunFilters) {
  return useQuery({
    queryKey: ['runs', filters],
    queryFn: () => runApi.list(filters),
    staleTime: 30000,  // 30 seconds
  });
}

export function useRun(runId: number) {
  return useQuery({
    queryKey: ['runs', runId],
    queryFn: () => runApi.get(runId),
    refetchInterval: (data) => {
      // Poll every 2s while running, stop when complete
      return data?.status === 'running' ? 2000 : false;
    },
  });
}

export function useCreateRun() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: RunCreate) => runApi.create(data),
    onSuccess: () => {
      // Invalidate runs list to refresh
      queryClient.invalidateQueries({ queryKey: ['runs'] });
    },
  });
}
```

### Container/Presentational Pattern

```typescript
// Container: Handles data fetching
function RunList() {
  const { data, isLoading, error, refetch } = useRuns();
  const [filters, setFilters] = useState<RunFilters>({});

  if (isLoading) return <Spinner size="lg" />;
  if (error) return <ErrorBanner error={error} onRetry={refetch} />;

  return (
    <div className="space-y-4">
      <RunFilters filters={filters} onChange={setFilters} />
      <RunTable runs={data.runs} onRowClick={handleRowClick} />
    </div>
  );
}

// Presentational: Receives props, pure rendering
interface RunTableProps {
  runs: Run[];
  onRowClick: (run: Run) => void;
}

function RunTable({ runs, onRowClick }: RunTableProps) {
  return (
    <table className="min-w-full">
      <thead>...</thead>
      <tbody>
        {runs.map((run) => (
          <tr key={run.id} onClick={() => onRowClick(run)}>
            <td>{run.version}</td>
            <td><StatusBadge status={run.status} /></td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

### Error Boundary Pattern

```typescript
// components/ui/ErrorBoundary.tsx
import { Component, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback || (
        <div className="p-4 bg-red-50 border border-red-200 rounded">
          <h2 className="text-red-800 font-semibold">Something went wrong</h2>
          <p className="text-red-600">{this.state.error?.message}</p>
        </div>
      );
    }
    return this.props.children;
  }
}
```

### Progress Polling Pattern

```typescript
// features/runs/RunProgress.tsx
function RunProgress({ runId }: { runId: number }) {
  const { data: progress, isLoading } = useQuery({
    queryKey: ['runs', runId, 'progress'],
    queryFn: () => runApi.getProgress(runId),
    refetchInterval: 2000,  // Poll every 2 seconds
  });

  if (isLoading) return <Spinner />;

  return (
    <div className="space-y-4">
      <ProgressBar
        value={progress.overall_progress * 100}
        label={`${Math.round(progress.overall_progress * 100)}%`}
      />
      <p className="text-sm text-gray-500">
        ETA: {formatDuration(progress.eta_seconds)}
      </p>
      {Object.entries(progress.years).map(([year, yearProgress]) => (
        <YearProgress key={year} year={year} progress={yearProgress} />
      ))}
    </div>
  );
}
```

### Map Visualization Pattern (Leaflet)

```typescript
// features/districts/DistrictMap.tsx
import { MapContainer, TileLayer, GeoJSON } from 'react-leaflet';
import L from 'leaflet';

interface DistrictMapProps {
  districts: District[];
  colorBy: 'default' | 'compactness' | 'partisan';
  onDistrictClick?: (district: District) => void;
}

function DistrictMap({ districts, colorBy, onDistrictClick }: DistrictMapProps) {
  // Use canvas renderer for better performance with many features
  const renderer = L.canvas({ padding: 0.5 });

  // Memoize GeoJSON conversion
  const geojsonData = useMemo(() => ({
    type: 'FeatureCollection',
    features: districts.map(d => ({
      type: 'Feature',
      id: d.id,
      geometry: d.geometry,
      properties: d,
    })),
  }), [districts]);

  const styleFeature = (feature: any) => ({
    fillColor: getDistrictColor(feature.properties, colorBy),
    weight: 1,
    color: '#666',
    fillOpacity: 0.7,
  });

  return (
    <MapContainer center={[39.8, -98.6]} zoom={4} className="h-[600px]">
      <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" />
      <GeoJSON
        data={geojsonData}
        style={styleFeature}
        renderer={renderer}
        onEachFeature={(feature, layer) => {
          layer.on('click', () => onDistrictClick?.(feature.properties));
        }}
      />
    </MapContainer>
  );
}
```

### API Client Pattern

```typescript
// api/client.ts
import axios from 'axios';

export const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:8002',
  headers: { 'Content-Type': 'application/json' },
});

// Response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Handle auth error
    }
    return Promise.reject(error);
  }
);

// api/runs.ts
export const runApi = {
  list: (filters?: RunFilters) =>
    apiClient.get<RunListResponse>('/api/v1/runs', { params: filters }).then(r => r.data),

  get: (id: number) =>
    apiClient.get<RunDetailResponse>(`/api/v1/runs/${id}`).then(r => r.data),

  create: (data: RunCreate) =>
    apiClient.post<RunResponse>('/api/v1/runs', data).then(r => r.data),

  start: (id: number) =>
    apiClient.post<RunResponse>(`/api/v1/runs/${id}/actions/start`).then(r => r.data),

  cancel: (id: number) =>
    apiClient.post<RunResponse>(`/api/v1/runs/${id}/actions/cancel`).then(r => r.data),

  getProgress: (id: number) =>
    apiClient.get<RunProgressResponse>(`/api/v1/runs/${id}/progress`).then(r => r.data),
};
```

---

## API/Frontend Anti-Patterns

### API Anti-Patterns

```python
# ❌ Sync blocking in async endpoint
@router.get("/slow")
async def slow_endpoint():
    subprocess.run(["long_command"])  # Blocks event loop!

# ✅ Use asyncio subprocess
@router.get("/slow")
async def slow_endpoint():
    proc = await asyncio.create_subprocess_exec(...)
    await proc.wait()

# ❌ N+1 queries
@router.get("/runs")
async def list_runs(db: Session = Depends(get_db)):
    runs = db.query(Run).all()
    for run in runs:
        run.year_details  # N additional queries!

# ✅ Eager loading
@router.get("/runs")
async def list_runs(db: Session = Depends(get_db)):
    runs = db.query(Run).options(selectinload(Run.year_details)).all()

# ❌ Unbounded queries
@router.get("/runs")
async def list_runs(db: Session = Depends(get_db)):
    return db.query(Run).all()  # Could return millions!

# ✅ Always paginate
@router.get("/runs")
async def list_runs(limit: int = Query(50, le=100), offset: int = Query(0)):
    return db.query(Run).limit(limit).offset(offset).all()
```

### Frontend Anti-Patterns

```typescript
// ❌ Direct API calls in component
function RunList() {
  const [runs, setRuns] = useState([]);
  useEffect(() => {
    fetch('/api/runs').then(r => r.json()).then(setRuns);
  }, []);  // No loading/error handling!
}

// ✅ Use React Query hooks
function RunList() {
  const { data, isLoading, error } = useRuns();
  if (isLoading) return <Spinner />;
  if (error) return <ErrorBanner error={error} />;
  return <RunTable runs={data.runs} />;
}

// ❌ Prop drilling
function App() {
  const [selectedRun, setSelectedRun] = useState(null);
  return (
    <Layout selectedRun={selectedRun}>
      <Page1 selectedRun={selectedRun} setSelectedRun={setSelectedRun}>
        <Component selectedRun={selectedRun} />
      </Page1>
    </Layout>
  );
}

// ✅ Use context or URL state
function App() {
  return (
    <Routes>
      <Route path="/runs/:runId" element={<RunDetail />} />
    </Routes>
  );
}

// ❌ Missing memoization for expensive computations
function DistrictMap({ districts, colorBy }) {
  // Recomputes on every render!
  const geojson = {
    features: districts.map(d => ({ geometry: d.geometry }))
  };
}

// ✅ Memoize expensive computations
function DistrictMap({ districts, colorBy }) {
  const geojson = useMemo(() => ({
    features: districts.map(d => ({ geometry: d.geometry }))
  }), [districts]);
}
```

---

## Quick Reference: API/Frontend

**Starting new API endpoint**:
- [ ] Pydantic schema (request + response)
- [ ] Service function (business logic)
- [ ] Router function (HTTP handling)
- [ ] Register in main.py
- [ ] Add tests

**Starting new React feature**:
- [ ] Types (interfaces)
- [ ] API client functions
- [ ] Custom hooks (useQuery/useMutation)
- [ ] Container component
- [ ] Presentational components
- [ ] Add route
- [ ] Add tests

**API Testing**:
- [ ] Unit tests for services
- [ ] Integration tests for endpoints
- [ ] E2E test with VT

**Frontend Testing**:
- [ ] Unit tests for hooks/utils
- [ ] Component tests with Testing Library
- [ ] E2E tests with Playwright
