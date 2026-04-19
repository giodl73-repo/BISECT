# Coding Patterns

**Updated**: 2026-01-19

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
