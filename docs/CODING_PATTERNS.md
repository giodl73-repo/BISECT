# Coding Patterns and Conventions

This document captures critical patterns and conventions used throughout the codebase. Following these patterns ensures consistency, proper progress reporting, and correct behavior.

**Last Updated**: January 12, 2026

## Related Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and architectural decisions
- **[ENHANCEMENTS_2026.md](ENHANCEMENTS_2026.md)** - Recent improvements and roadmap
- **[../README.md](../README.md)** - User-facing project overview
- **[../CLAUDE.md](../CLAUDE.md)** - AI assistant guide and quick reference

## Table of Contents
1. [Progress Bar Protocol](#progress-bar-protocol)
2. [Skip Logic Pattern](#skip-logic-pattern)
3. [DPI Threading](#dpi-threading)
4. [GEOID Handling](#geoid-handling-critical)
5. [State Code Handling](#state-code-handling-critical)
6. [Map Visualization](#map-visualization)
7. [Subprocess Communication](#subprocess-communication)
8. [Scope-Based Analysis Pattern](#scope-based-analysis-pattern)
9. [File Naming Conventions](#file-naming-conventions)
10. [Git Commit Messages](#git-commit-messages)
11. [Error Handling](#error-handling)
12. [When to Use What](#when-to-use-what)
13. [Static HTML Generation](#static-html-generation)

---

## Progress Bar Protocol

### Environment Variable: TQDM_POSITION

**Purpose**: Coordinates progress bars across parent-child process hierarchies.

**Pattern**:
```python
import os
from tqdm import tqdm

# Child script detects if called from parent
position = int(os.environ.get('TQDM_POSITION', '-1'))
send_status = position >= 0
is_standalone = not send_status

# Report progress to parent
def report_progress(msg):
    if send_status:
        print(f"STATUS:{position}:{msg}", flush=True)

# Use different output modes
if is_standalone:
    # Standalone: show detailed output and progress bars
    print("Detailed header information")
    pbar = tqdm(...)
else:
    # Called from parent: suppress banners, emit STATUS messages
    report_progress(f"Processing state X...")
```

### STATUS Message Protocol

**Format**: `STATUS:{position}:{message}`

**Parent Monitoring**:
```python
import subprocess

env = os.environ.copy()
env['TQDM_POSITION'] = str(position)

proc = subprocess.Popen(cmd, shell=True, env=env,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                       text=True, bufsize=1)

# Monitor stdout for STATUS messages
for line in proc.stdout:
    line = line.strip()
    if line.startswith("STATUS:"):
        parts = line.split(":", 2)
        if len(parts) >= 3:
            pos = int(parts[1])
            msg = parts[2]
            progress_bars[pos].set_description_str(f"[{pos}] {msg}")
            progress_bars[pos].refresh()
```

**Critical Rules**:
- Never print banners/headers if `send_status` is True
- Always flush STATUS messages: `print(..., flush=True)`
- Parent must read stdout in real-time (use `Popen`, not `run` with `capture_output`)
- Child progress bars should be `disable=send_status`

---

## Skip Logic Pattern

### Critical Rules

1. **ALWAYS implement per-stage skip logic** - Each script checks its own outputs
2. **NEVER skip entire pipelines** - Let each stage decide if it needs to run
3. **ALWAYS add --force flag** - Allow users to override skip logic
4. **NEVER assume upstream outputs** - Always check if they exist first

### Why Per-Stage Skip Logic?

**Enables incremental updates**: When you add a new analysis stage (e.g., metro area maps), users can run the pipeline again and ONLY the new stage runs. Without per-stage skip logic, they'd have to reprocess everything or use complex flags.

**Bad Example** (what we had before):
```python
# DON'T DO THIS - skips entire state if basic outputs exist
if state_dir.exists():
    required_files = [
        state_dir / 'final_assignments.pkl',
        state_dir / 'district_summary.csv',
        state_dir / 'maps'
    ]
    if all(f.exists() for f in required_files):
        print(f"{state_name} SKIPPED")
        return True  # Skip entire state pipeline!
```

**Problem**: If you add metro maps later, states that already ran won't get metros generated.

**Good Example** (per-stage skip):
```python
# DO THIS - each stage checks its own outputs
def run_state_pipeline(state_code, state_dir):
    # Stage 1: Redistricting
    run_redistricting(state_code, state_dir)  # Checks for final_assignments.pkl

    # Stage 2: Cities
    add_cities(state_dir)  # Checks for district_cities.csv

    # Stage 3: Summary
    create_summary(state_dir)  # Checks for district_summary.csv

    # Stage 4: Round maps
    create_round_maps(state_dir)  # Checks for round_*.png files

    # Stage 5: District maps
    create_district_maps(state_dir)  # Checks for districts/*.png files

    # Stage 6-10: Analysis (if enabled)
    if run_analysis:
        analyze_political(state_dir)  # Checks for political CSVs
        visualize_political(state_dir)  # Checks for political maps
        # ... etc for demographic, compactness, metros
```

**Benefit**: Adding stage 11 (metros) later? Re-run the pipeline and stages 1-10 skip instantly, only stage 11 runs.

### Standard Per-Stage Pattern

**Every pipeline script MUST implement this:**

```python
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    args = parser.parse_args()

    output_file = Path('output.png')

    # Check if output exists
    if not args.force and output_file.exists():
        if is_standalone:
            print(f"Output already exists - skipping: {output_file}")
            print("Use --force to regenerate")
        return 0  # Success - skipped

    # ... do work ...
    return 0  # Success - completed
```

### Multiple Output Files

```python
required_files = [
    output_dir / 'file1.csv',
    output_dir / 'file2.png',
    output_dir / 'file3.json'
]

if not args.force and all(f.exists() for f in required_files):
    if is_standalone:
        print("All outputs exist - skipping")
        for f in required_files:
            print(f"  {f.name}")
        print("\nUse --force to regenerate")
    return 0
```

### Return Codes

**CRITICAL**: Always return 0 for success, even when skipping:
```python
# CORRECT
if output_file.exists() and not args.force:
    print("Skipping - already exists")
    return 0  # Success!

# WRONG
if output_file.exists() and not args.force:
    print("Skipping - already exists")
    return None  # Parent thinks this failed!
```

**Why**: Prevents expensive re-computation, enables pipeline resumption after failures, **enables incremental updates when adding new stages**.

---

## DPI Threading

### All Visualization Scripts Must Accept --dpi

**Pattern**:
```python
def create_map(data, output_file, dpi=150):
    """Create map with configurable DPI."""
    fig, ax = plt.subplots(figsize=(12, 10))
    # ... plotting ...
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--dpi', type=int, default=150,
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps (default: 150)')
    args = parser.parse_args()

    create_map(data, output_file, dpi=args.dpi)
```

### Pipeline Integration

**Parent script must pass --dpi to all children**:
```python
pipeline_steps.append({
    'name': 'Create maps',
    'command': f'{sys.executable} scripts/create_map.py --dpi {args.dpi}',
    'critical': False
})
```

**DPI Guidelines**:
- 72-100: Development/testing
- 150: Default production (good balance)
- 200: High quality
- 300: Print quality (slow, large files)

**Critical**: Never hardcode `dpi=300` in savefig calls!

---

## GEOID Handling (CRITICAL!)

### The Problem

Census tract GEOIDs are 11-digit identifiers but are often stored as integers, losing leading zeros.

**Format**: `SSCCCTTTTTT`
- SS = 2-digit state FIPS
- CCC = 3-digit county FIPS
- TTTTTT = 6-digit tract code

**Example**: California tract: `06001400100`

### The Pattern

**ALWAYS use this pattern when working with GEOIDs**:
```python
# Converting to string
geoid = str(raw_geoid).zfill(11)

# Loading from CSV
df['GEOID'] = df['GEOID'].astype(str).str.zfill(11)

# Before merging DataFrames
df1['GEOID'] = df1['GEOID'].astype(str).str.zfill(11)
df2['GEOID'] = df2['GEOID'].astype(str).str.zfill(11)
result = df1.merge(df2, on='GEOID', how='inner')
```

### Common Mistake

```python
# BAD - Type mismatch error
demo_df['GEOID'] = int  # or stays as object
assignments_df['GEOID'] = str
result = demo_df.merge(assignments_df, on='GEOID')  # ERROR!

# GOOD
demo_df['GEOID'] = demo_df['GEOID'].astype(str).str.zfill(11)
assignments_df['GEOID'] = assignments_df['GEOID'].astype(str).str.zfill(11)
result = demo_df.merge(assignments_df, on='GEOID')  # Works!
```

**Why Critical**: Type mismatches cause merge failures that are hard to debug.

---

## State Code Handling (CRITICAL!)

### The Problem

State identifiers appear in **three different formats** across the codebase, causing frequent mismatches when merging data:

1. **2-letter state codes**: `AL`, `CA`, `TX` (used in Census files, enacted baseline data)
2. **Full uppercase names**: `ALABAMA`, `CALIFORNIA`, `TEXAS` (used in pipeline outputs, algorithmic data)
3. **Lowercase with underscores**: `alabama`, `california`, `texas` (used in directory names)

**Common error**: State data from enacted baselines (AL, CA) fails to match with algorithmic data (ALABAMA, CALIFORNIA).

### Standard State Code Mapping

**ALWAYS use this mapping** when converting between formats:

```python
# Standard mapping (add to top of script)
STATE_CODE_TO_NAME = {
    'AL': 'ALABAMA', 'AK': 'ALASKA', 'AZ': 'ARIZONA', 'AR': 'ARKANSAS',
    'CA': 'CALIFORNIA', 'CO': 'COLORADO', 'CT': 'CONNECTICUT', 'DE': 'DELAWARE',
    'FL': 'FLORIDA', 'GA': 'GEORGIA', 'HI': 'HAWAII', 'ID': 'IDAHO',
    'IL': 'ILLINOIS', 'IN': 'INDIANA', 'IA': 'IOWA', 'KS': 'KANSAS',
    'KY': 'KENTUCKY', 'LA': 'LOUISIANA', 'ME': 'MAINE', 'MD': 'MARYLAND',
    'MA': 'MASSACHUSETTS', 'MI': 'MICHIGAN', 'MN': 'MINNESOTA', 'MS': 'MISSISSIPPI',
    'MO': 'MISSOURI', 'MT': 'MONTANA', 'NE': 'NEBRASKA', 'NV': 'NEVADA',
    'NH': 'NEW_HAMPSHIRE', 'NJ': 'NEW_JERSEY', 'NM': 'NEW_MEXICO', 'NY': 'NEW_YORK',
    'NC': 'NORTH_CAROLINA', 'ND': 'NORTH_DAKOTA', 'OH': 'OHIO', 'OK': 'OKLAHOMA',
    'OR': 'OREGON', 'PA': 'PENNSYLVANIA', 'RI': 'RHODE_ISLAND', 'SC': 'SOUTH_CAROLINA',
    'SD': 'SOUTH_DAKOTA', 'TN': 'TENNESSEE', 'TX': 'TEXAS', 'UT': 'UTAH',
    'VT': 'VERMONT', 'VA': 'VIRGINIA', 'WA': 'WASHINGTON', 'WV': 'WEST_VIRGINIA',
    'WI': 'WISCONSIN', 'WY': 'WYOMING'
}

# Reverse mapping (full name to code)
STATE_NAME_TO_CODE = {v: k for k, v in STATE_CODE_TO_NAME.items()}
```

### Common Use Cases

#### 1. Matching Enacted Baseline with Algorithmic Data

```python
# Enacted baseline CSV has 'state' column with 2-letter codes
enacted_df = pd.read_csv('enacted_compactness_2020.csv')
# state column: AL, CA, TX, ...

# Algorithmic data has 'state' column with full names
algo_df = pd.read_csv('district_summary.csv')
# state column: ALABAMA, CALIFORNIA, TEXAS, ...

# Convert enacted baseline codes to names BEFORE merging
enacted_df['state'] = enacted_df['state'].map(STATE_CODE_TO_NAME)

# Now they match
merged = algo_df.merge(enacted_df, on='state', suffixes=('_algo', '_enacted'))
```

#### 2. Loading Enacted Baseline Data Per-State

```python
def load_baseline_data(year):
    df = pd.read_csv(f'enacted_compactness_{year}.csv')

    # Detect column name (varies: 'state_code', 'state', 'STATE')
    state_col = None
    for col in ['state_code', 'state', 'STATE', 'state_name']:
        if col in df.columns:
            state_col = col
            break

    # Group by state and convert codes to names
    state_stats = {}
    for state_code in df[state_col].unique():
        state_df = df[df[state_col] == state_code]

        # Convert to full name for matching
        state_key = STATE_CODE_TO_NAME.get(state_code.upper(), state_code.upper())

        state_stats[state_key] = {
            'mean_pp': state_df['polsby_popper'].mean(),
            'num_districts': len(state_df)
        }

    return state_stats
```

#### 3. Directory Name to Full Name Conversion

```python
# Directory names use lowercase with underscores
state_dir = Path('outputs/us_2020_v1/states/north_carolina')

# Convert to full name for matching
state_name = state_dir.name.upper()  # -> NORTH_CAROLINA

# Or convert to 2-letter code
state_code = STATE_NAME_TO_CODE.get(state_name)  # -> NC
```

### Format Conventions

**When to use each format:**

| Format | Use Case | Example |
|--------|----------|---------|
| **2-letter code** | Census data, FIPS codes, enacted baselines | `AL`, `CA`, `TX` |
| **Full uppercase** | Pipeline outputs, internal data structures, matching | `ALABAMA`, `CALIFORNIA`, `TEXAS` |
| **Lowercase underscore** | Directory names, file prefixes | `alabama`, `california`, `texas` |

### BAD vs GOOD Examples

```python
# BAD - Direct comparison fails
if enacted_state == algo_state:  # 'AL' != 'ALABAMA', always False
    print("Match found")

# GOOD - Convert before comparing
enacted_state_name = STATE_CODE_TO_NAME.get(enacted_state.upper())
if enacted_state_name == algo_state:
    print("Match found")
```

```python
# BAD - Hardcoding state codes
states_to_process = ['AL', 'CA', 'TX']  # Fragile, error-prone

# GOOD - Use the mapping
states_to_process = STATE_CODE_TO_NAME.keys()  # All 50 states
# or
states_to_process = STATE_CODE_TO_NAME.values()  # Full names
```

### Real-World Issue Example

**Problem encountered**: State-by-state dashboard showed "-" for all enacted data because:
- Enacted baseline CSV: `state='AL'`
- Algorithmic data: `state='ALABAMA'`
- No matches found during merge

**Solution**: Added `STATE_CODE_TO_NAME` mapping and converted enacted baseline codes to full names before merging.

### Critical Rules

1. **ALWAYS convert to a common format before merging**
2. **NEVER hardcode state codes** - use the mapping dictionary
3. **ALWAYS uppercase state codes** before lookup: `state_code.upper()`
4. **Document which format** your function expects in docstrings
5. **Add the mapping dictionary** to any script that merges state data

---

## Map Visualization

### Standard Map Pattern

**All district maps should use this pattern**:

```python
import matplotlib.pyplot as plt
import geopandas as gpd

def create_district_map(tracts_gdf, output_file, dpi=150):
    """Standard district map with proper boundaries."""

    fig, ax = plt.subplots(1, 1, figsize=(14, 10))

    # Plot tracts with THIN WHITE boundaries
    tracts_gdf.plot(
        ax=ax,
        column='some_attribute',  # Color by attribute
        edgecolor='white',        # Thin white tract boundaries
        linewidth=0.1,            # Very thin
        alpha=0.8,
        legend=True
    )

    # Add THICK BLACK district boundaries as overlay
    districts_dissolved = tracts_gdf.dissolve(by='district', as_index=False)
    districts_dissolved.boundary.plot(
        ax=ax,
        edgecolor='black',        # Thick black district boundaries
        linewidth=1.5,            # Thick
        zorder=10                 # On top
    )

    ax.set_axis_off()
    ax.set_title('Map Title', fontsize=16, fontweight='bold', pad=20)

    plt.tight_layout()
    plt.savefig(output_file, dpi=dpi, bbox_inches='tight')
    plt.close()
```

### Why This Pattern?

- **Thin white tract boundaries**: Shows internal structure without cluttering
- **Thick black district overlay**: Makes district boundaries prominent
- **dissolve() then boundary.plot()**: Merges adjacent tracts into district polygons
- **zorder=10**: Ensures district boundaries render on top

### Title Formatting

**Avoid `\n` in titles** - causes rendering issues:
```python
# BAD
ax.set_title('State Name\nCongressional Districts')

# GOOD
ax.set_title('State Name - Congressional Districts')
# or
ax.set_title('State Name Congressional Districts', fontsize=16)
```

---

## Subprocess Communication

### When to Use subprocess.Popen vs subprocess.run

**Use `Popen` when:**
- Need to monitor real-time output (STATUS messages)
- Long-running processes
- Parent manages child's progress bar

```python
proc = subprocess.Popen(
    cmd, shell=True, env=env,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1  # Line buffering
)

# Read output in real-time
for line in proc.stdout:
    if line.startswith("STATUS:"):
        # Update progress bar
        ...

proc.wait()
```

**Use `run` when:**
- Simple execution, don't need output
- Child handles its own progress
- Fire and forget

```python
result = subprocess.run(
    cmd,
    capture_output=True,  # or False
    text=True
)

if result.returncode != 0:
    print(f"Failed: {result.stderr}")
```

### Environment Variable Passing

```python
env = os.environ.copy()
env['TQDM_POSITION'] = str(position)
env['DPI'] = str(dpi)

# Pass to child
subprocess.Popen(cmd, env=env, ...)
```

---

## Scope-Based Analysis Pattern

### Overview

**All analysis and visualization scripts must support both state and national scope using a single unified script.**

This pattern enables:
- ✅ **Parallel execution**: Per-state analysis runs during state processing
- ✅ **Code reuse**: Same script for both scopes (no duplication)
- ✅ **Maintainability**: One place to update logic
- ✅ **Performance**: Saves 1-2 hours by parallelizing analysis

### Standard Pattern

Every analysis/visualization script follows this structure:

```python
#!/usr/bin/env python3
"""
Analyze/visualize XXX characteristics of districts.
"""

import pandas as pd
import geopandas as gpd
from pathlib import Path
import argparse
import sys


def analyze_state_xxx(state_dir, census_year='2020'):
    """Analyze XXX for a single state."""
    state_dir = Path(state_dir)

    # Load ONLY this state's data
    # NEVER load all 50 states in state scope!

    # Output to: state_dir / 'xxx_analysis' / 'district_xxx.csv'
    return 0


def visualize_state_xxx(state_dir, state_code, census_year, dpi=150):
    """Visualize XXX for a single state."""
    state_dir = Path(state_dir)

    # Check if analysis exists
    analysis_file = state_dir / 'xxx_analysis' / 'district_xxx.csv'
    if not analysis_file.exists():
        print(f"ERROR: Analysis not found: {analysis_file}")
        print(f"Run analyze_xxx.py first")
        return 1

    # Load state-specific data ONLY
    tracts_file = Path(f'data/tracts/{census_year}/{state_code.lower()}_tracts_{census_year}.parquet')
    tracts_gdf = gpd.read_parquet(tracts_file)

    # Load analysis results
    analysis_df = pd.read_csv(analysis_file)

    # Create visualizations
    output_dir = state_dir / 'xxx_analysis' / 'maps'
    output_dir.mkdir(parents=True, exist_ok=True)

    create_xxx_map(tracts_gdf, analysis_df, output_dir / 'xxx_map.png', dpi)

    return 0


def visualize_national_xxx(output_dir, version, census_year, dpi=150, position=-1, force=False):
    """Aggregate all states into national visualization."""
    base_dir = Path(output_dir)

    # Progress bar protocol
    send_status = position >= 0
    is_standalone = not send_status

    def report_progress(msg):
        if send_status:
            print(f"STATUS:{position}:{msg}", flush=True)

    # Output file
    output_file = base_dir / f'us_national_xxx_{census_year}.png'

    # Check if output exists
    if not force and output_file.exists():
        if is_standalone:
            print(f"Output already exists: {output_file}")
            print("Use --force to regenerate")
        return 0

    report_progress("Creating national XXX map - Loading data")

    # Load ALL 50 states
    all_state_data = []
    for state_name in STATE_NAMES:
        state_dir = base_dir / 'states' / state_name
        if not state_dir.exists():
            continue

        # Load state's analysis results
        analysis_file = state_dir / 'xxx_analysis' / 'district_xxx.csv'
        if analysis_file.exists():
            df = pd.read_csv(analysis_file)
            df['state'] = state_name
            all_state_data.append(df)

    if not all_state_data:
        print("ERROR: No state data found")
        return 1

    us_data = pd.concat(all_state_data, ignore_index=True)

    # Create national visualization
    create_national_xxx_map(us_data, output_file, dpi)

    report_progress("Creating national XXX map - Complete")
    return 0


def main():
    parser = argparse.ArgumentParser(
        description='Analyze/visualize XXX at state or national scope'
    )

    # Scope selection
    parser.add_argument('--scope', choices=['state', 'national'], default='national',
                       help='Scope: state (single state) or national (all states, default)')
    parser.add_argument('--census-year', type=str, default='2020',
                       choices=['2020', '2010', '2000'],
                       help='Census year (default: 2020)')

    # State scope arguments
    parser.add_argument('--state', type=str,
                       help='State code (2-letter, required if scope=state)')
    parser.add_argument('--state-dir', type=str,
                       help='State directory (required if scope=state)')

    # National scope arguments
    parser.add_argument('--output-dir', type=str,
                       help='Base output directory (required if scope=national)')
    parser.add_argument('--version', type=str,
                       help='Version (required if scope=national)')

    # Common arguments
    parser.add_argument('--dpi', type=int, default=150,
                       choices=[72, 100, 150, 200, 300],
                       help='DPI for output maps (default: 150)')
    parser.add_argument('--force', action='store_true',
                       help='Force regeneration even if outputs exist')
    parser.add_argument('--position', type=int, default=-1,
                       help='Progress bar position (for parent coordination)')

    args = parser.parse_args()

    # Validate and dispatch
    if args.scope == 'state':
        if not args.state or not args.state_dir:
            parser.error("--state and --state-dir required when scope=state")
        return visualize_state_xxx(args.state_dir, args.state,
                                   args.census_year, args.dpi)

    elif args.scope == 'national':
        if not args.output_dir or not args.version:
            parser.error("--output-dir and --version required when scope=national")
        return visualize_national_xxx(args.output_dir, args.version,
                                      args.census_year, args.dpi,
                                      args.position, args.force)


if __name__ == '__main__':
    sys.exit(main())
```

### Integration Points

#### 1. Per-State Pipeline (Parallel)

Add to `process_single_state.py`:

```python
if args.run_analysis:
    # Analysis script
    xxx_analyze = Path(__file__).parent.parent / 'xxx' / 'analyze_xxx.py'
    steps.append((
        "XXX analysis",
        f'{sys.executable} {xxx_analyze} {state_dir} --census-year {args.year}'.strip()
    ))

    # Visualization script
    xxx_visualize = Path(__file__).parent.parent / 'xxx' / 'visualize_xxx.py'
    steps.append((
        "XXX visualization",
        f'{sys.executable} {xxx_visualize} --scope state --state {state_code} '
        f'--state-dir {state_dir} --census-year {args.year} --dpi {args.dpi} '
        f'--position {child_position}'.strip()
    ))
```

**Key points:**
- Only runs if `--run-analysis` enabled (default: True)
- Uses `--scope state` to invoke state-specific function
- Passes `child_position=999` to suppress child progress bars
- Analysis runs BEFORE visualization

#### 2. Post-Processing (Sequential)

Add to `run_complete_redistricting.py`:

```python
# Create national XXX map (after per-state analysis completes)
if not args.skip_xxx and (output_dir.exists() or args.print_only):
    xxx_script = scripts_dir.parent / 'xxx' / 'visualize_xxx.py'
    pipeline_steps.append({
        'name': 'Create national XXX map',
        'command': f'{sys.executable} {xxx_script} --scope national '
                   f'--output-dir {output_dir} --version {args.version} '
                   f'--census-year {args.year} --dpi {args.dpi}'.strip(),
        'critical': False
    })
```

**Key points:**
- Uses `--scope national` to invoke national aggregation function
- Runs AFTER all per-state processing completes
- Aggregates per-state analysis results into one national map

### Critical Rules

1. **NEVER load all 50 states in state scope**
   ```python
   # BAD - loads 84,000 tracts nationwide
   if args.scope == 'state':
       all_tracts = gpd.read_parquet('data/tracts/us_all_tracts.parquet')

   # GOOD - loads only this state's tracts
   if args.scope == 'state':
       tracts = gpd.read_parquet(f'data/tracts/{census_year}/{state_code.lower()}_tracts_{census_year}.parquet')
   ```

2. **Analysis before visualization**
   - Per-state: analyze_xxx.py → visualize_xxx.py --scope state
   - National: All states analyzed → visualize_xxx.py --scope national

3. **Default scope should be national**
   - Maintains backward compatibility with old scripts
   - State scope is explicit: `--scope state`

4. **Follow progress bar protocol**
   - State scope: Respects `--position` parameter
   - National scope: Uses STATUS messages when position >= 0

5. **Implement skip logic**
   - Both state and national functions check for existing outputs
   - Use `--force` to regenerate

### Examples

**Existing implementations:**
- ✅ `scripts/compactness/visualize_compactness.py`
- ✅ `scripts/political/visualize_partisan_lean.py`
- ✅ `scripts/demographic/visualize_district_demographics.py`

**Usage:**

```bash
# State scope (called by process_single_state.py)
python scripts/xxx/visualize_xxx.py \
  --scope state \
  --state CA \
  --state-dir outputs/us_2020_v1/states/california \
  --census-year 2020 \
  --dpi 150

# National scope (called by run_complete_redistricting.py)
python scripts/xxx/visualize_xxx.py \
  --scope national \
  --output-dir outputs/us_2020_v1 \
  --version v1 \
  --census-year 2020 \
  --dpi 150
```

### Benefits

- **Performance**: Per-state analysis runs in parallel (50 states × 4 workers = massive speedup)
- **Maintainability**: Single script to update, not three (wrapper, state logic, national logic)
- **Consistency**: Same code path ensures state and national outputs match
- **Flexibility**: Can re-run national without re-analyzing all states

---

## File Naming Conventions

### Census/Geographic Data
```
{state_code}_tracts_{year}.parquet           # State tracts
{state_code}_places_{year}.parquet           # State places
{state_code}_adjacency_{year}.pkl            # Adjacency graph
```

### Processed Data
```
{year}_president_tract.parquet               # Election data
{year}_demographics_tract.parquet            # Demographic data
```

### Output Files
```
# State output directory
outputs/us_{year}_{version}/states/{state_name}/

# District files
district_summary.csv                         # District metrics
district_cities.csv                          # Major cities per district
final_assignments.pkl                        # tract_idx -> district
rounds_hierarchy.csv                         # Bisection tree

# Maps directory
maps/
  {state_name}_52_districts.png             # Final map
  {state_name}_52_districts_with_cities.png # With city labels
  districts/                                 # Individual district maps
    district_01.png
    district_02.png
  round_0.png                               # Bisection round maps
  round_1.png

# Analysis subdirectories
political_analysis/
  district_political_{year}.csv             # Partisan lean
  maps/
    partisan_lean.png
    partisan_lean_with_margins.png

demographic_analysis/
  district_demographics.csv                 # Demographics
  maps/
    gender_balance.png
    majority_race.png
    diversity_index.png
```

### Script Naming
```
# Action verbs
download_*.py          # Downloads data
process_*.py           # Processes/transforms data
create_*.py            # Creates outputs
analyze_*.py           # Analyzes data
visualize_*.py         # Creates visualizations

# Batch runners
run_*.py               # Orchestrates multiple operations
```

---

## Git Commit Messages

### Format

```
<type>: <short summary> (50 chars or less)

<detailed description>
- Bullet points for changes
- What was the problem
- How was it fixed
- What's the impact

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

### Types
- `fix`: Bug fixes
- `feat`: New features
- `refactor`: Code restructuring without behavior change
- `perf`: Performance improvements
- `docs`: Documentation changes
- `style`: Formatting, whitespace
- `test`: Adding/updating tests
- `chore`: Build process, dependencies

### Example
```
fix: GEOID type mismatch in demographic analysis

Problem:
- All 50 states failed demographic analysis
- ValueError: "You are trying to merge on int64 and object columns"

Solution:
- Ensure both DataFrames have GEOID as string type before merge
- Added str.zfill(11) to both demographic and assignments data

Impact:
- All 50 states now successfully analyzed
- Creates district_demographics.csv for each state

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

---

## Error Handling

### Standard Pattern

```python
def main():
    try:
        # Load required data
        if not required_file.exists():
            raise FileNotFoundError(f"Required file not found: {required_file}")

        # Do work
        result = process_data()

        # Save outputs
        result.to_csv(output_file)

        return 0

    except FileNotFoundError as e:
        print(f"ERROR: {e}")
        print(f"Run prerequisite script first")
        return 1

    except Exception as e:
        print(f"ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1
```

### Informative Error Messages

```python
# BAD
raise ValueError("Invalid input")

# GOOD
raise ValueError(
    f"Invalid state code: {state_code}\n"
    f"Expected 2-letter code (e.g., 'CA', 'NY')\n"
    f"Got: {state_code}"
)

# EVEN BETTER - suggest solution
if not demographic_file.exists():
    raise FileNotFoundError(
        f"Demographic data not found: {demographic_file}\n"
        f"\n"
        f"Run these commands to prepare data:\n"
        f"  python scripts/data/demographics/download_demographic_data_robust.py\n"
        f"  python scripts/data/demographics/process_demographic_data.py"
    )
```

---

## Version, Year, and Output Directory Convention

### Standard Pattern for Pipeline Scripts

**All major pipeline scripts should accept these three parameters**:

```python
parser = argparse.ArgumentParser()
parser.add_argument('--year', type=str, default='2020',
                   choices=['2020', '2010', '2000'],
                   help='Census year (default: 2020)')
parser.add_argument('--version', type=str, default='v1',
                   help='Version identifier (default: v1)')
parser.add_argument('--output-dir', type=str, default=None,
                   help='Override output directory (default: outputs/us_{year}_{version})')
args = parser.parse_args()

# Determine output directory
if args.output_dir:
    output_dir = Path(args.output_dir)
else:
    output_dir = Path(f'outputs/us_{args.year}_{args.version}')
```

### Why This Pattern?

**Year**: Supports multiple census datasets
- 2020: Latest census
- 2010: Historical comparison
- 2000: Long-term trends

**Version**: Multiple runs with different parameters
- v1: Initial run
- v2: Adjusted parameters
- v3: Different algorithm variant

**Output-dir override**: Flexibility
- Development: `--output-dir outputs/dev_test`
- Production: Uses standard `outputs/us_2020_v1`
- Comparison: `outputs/us_2020_metis_vs_greedy`

### Passing to Child Scripts

**Parent should pass these parameters to all children**:
```python
# In run_complete_redistricting.py
pipeline_steps.append({
    'name': 'Political analysis',
    'command': (
        f'{sys.executable} scripts/political/run_political_analysis.py '
        f'--census-year {args.year} '
        f'--version {args.version} '
        f'--election-year {args.election_year}'
    ).strip()
})
```

### Consistency Rules

1. **Always use --census-year** (not --year) in analysis scripts to avoid confusion with election year
2. **Always provide override capability** via --output-dir
3. **Detect from output-dir if possible** instead of requiring all three parameters:

```python
# Good pattern for analysis scripts
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output-dir', type=str,
                       help='Output directory (e.g., outputs/us_2020_v1)')
    # OR individual parameters
    parser.add_argument('--year', type=str, default='2020')
    parser.add_argument('--version', type=str, default='v1')
    args = parser.parse_args()

    # Flexible detection
    if args.output_dir:
        output_dir = Path(args.output_dir)
    else:
        output_dir = Path(f'outputs/us_{args.year}_{args.version}')
```

### Directory Structure Convention

```
outputs/
├── us_2020_v1/              # Standard format: us_{year}_{version}
│   ├── states/
│   │   ├── alabama/
│   │   └── ...
│   ├── us_rounds_hierarchy.csv
│   └── us_national_map.png
├── us_2020_v2/              # Different version (e.g., different params)
├── us_2010_v1/              # Different year
└── dev_test/                # Development override
```

---

## When to Use What

### Data Loading

**Tracts (Geographic)**:
```python
import geopandas as gpd
tracts = gpd.read_parquet('data/tracts/2020/ca_tracts_2020.parquet')
```

**Tabular Data**:
```python
import pandas as pd
df = pd.read_csv('data.csv')
df = pd.read_parquet('data.parquet')  # Faster, smaller
```

**Pickle (Python objects)**:
```python
import pickle
with open('adjacency.pkl', 'rb') as f:
    graph = pickle.load(f)
```

### When to Use Each File Format

- **Parquet**: Tabular data, fast, compressed, typed
- **CSV**: Human-readable, Excel-compatible, larger
- **Pickle**: Python objects (graphs, dicts), not portable
- **Shapefile**: Legacy GIS, don't use (use parquet instead)
- **GeoJSON**: Web mapping, human-readable, large

### Tools vs Direct Commands

**Prefer specialized tools**:
```python
# BAD
subprocess.run(['cat', 'file.txt'])

# GOOD
with open('file.txt', 'r') as f:
    content = f.read()

# BAD
subprocess.run(['grep', 'pattern', 'file.txt'])

# GOOD
import re
with open('file.txt') as f:
    matches = [line for line in f if re.search(pattern, line)]
```

**Use subprocess for**:
- External tools (METIS, gdal)
- Child scripts with progress reporting
- Git operations

---

## Anti-Patterns (Don't Do This!)

### 1. Hardcoded Paths
```python
# BAD
df = pd.read_csv('C:/Users/John/data/file.csv')

# GOOD
from pathlib import Path
data_dir = Path('data/tracts/2020')
df = pd.read_csv(data_dir / 'file.csv')
```

### 2. Ignoring Existing Outputs
```python
# BAD - Always regenerates
def create_map(output):
    # ... expensive computation ...
    plt.savefig(output)

# GOOD - Skip if exists
def create_map(output, force=False):
    if not force and output.exists():
        return
    # ... expensive computation ...
    plt.savefig(output)
```

### 3. Silent Failures
```python
# BAD
try:
    result = risky_operation()
except:
    pass  # Silent failure!

# GOOD
try:
    result = risky_operation()
except Exception as e:
    print(f"ERROR: {e}")
    return 1
```

### 4. Mixing Concerns
```python
# BAD - One giant function
def process_everything():
    data = download_data()
    processed = clean_data(data)
    results = analyze_data(processed)
    create_maps(results)
    send_email(results)

# GOOD - Separate scripts
# scripts/download_data.py
# scripts/process_data.py
# scripts/analyze_data.py
# scripts/visualize_results.py
```

---

## Static HTML Generation

### Problem: Browser CORS Restrictions with Local Files

**Issue**: When opening HTML files locally (`file://` protocol), browsers block `fetch()` calls to load JSON files due to CORS (Cross-Origin Resource Sharing) security restrictions.

**Example of the Problem**:
```html
<!-- This FAILS when opened as file:// -->
<script>
    fetch('./data.json')  // BLOCKED by browser
        .then(response => response.json())
        .then(data => displayData(data));
</script>
```

**Error**: `Access to fetch at 'file:///...' from origin 'null' has been blocked by CORS policy`

### Solution: Embed Data Directly in HTML

Instead of fetching external JSON files, embed the data directly into the HTML during generation.

**Pattern**:

1. **Template HTML** (`web/dashboard.html`):
```html
<script>
    // Data will be embedded by generator script
    const DATA = /* DATA_PLACEHOLDER */;

    // Use data directly (synchronous, no fetch needed)
    function loadData() {
        displayData(DATA);  // Works immediately
    }
</script>
```

2. **Generator Script** (`scripts/web/generate_dashboard.py`):
```python
import json
from pathlib import Path

def generate_dashboard(template_path, output_path, data):
    # Read template
    with open(template_path, 'r', encoding='utf-8') as f:
        html = f.read()

    # Embed data as JSON
    data_json = json.dumps(data, indent=8)
    html = html.replace('/* DATA_PLACEHOLDER */', data_json)

    # Write output
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html)
```

**Result**: The generated HTML file contains all data embedded within it, eliminating any external dependencies and CORS issues.

### Example: Master Dashboard

**Template** (`web/master_dashboard.html`):
```javascript
const RUNS = /* RUNS_DATA_PLACEHOLDER */;
const COMPARISON_DATA = /* COMPARISON_DATA_PLACEHOLDER */;

// Data is immediately available, no async needed
document.addEventListener('DOMContentLoaded', function() {
    displayComparison(COMPARISON_DATA);
    displayRuns(RUNS);
});
```

**Generator** (`scripts/web/generate_master_dashboard.py`):
```python
# Embed run data
runs_json = json.dumps(runs, indent=8)
html = html.replace('/* RUNS_DATA_PLACEHOLDER */', runs_json)

# Embed comparison data from file
comparison_path = Path('outputs/comparison.json')
with open(comparison_path, 'r', encoding='utf-8') as f:
    comparison_data = json.load(f)
comparison_json = json.dumps(comparison_data, indent=8)
html = html.replace('/* COMPARISON_DATA_PLACEHOLDER */', comparison_json)
```

### Best Practices

1. **Use Meaningful Placeholder Names**:
   - `/* DATA_PLACEHOLDER */` - Generic data
   - `/* RUNS_DATA_PLACEHOLDER */` - Run configuration
   - `/* COMPARISON_DATA_PLACEHOLDER */` - Comparison results
   - Use comments `/* ... */` not strings to avoid JSON escaping issues

2. **Handle Missing Data Gracefully**:
```python
if data_file.exists():
    with open(data_file) as f:
        data = json.load(f)
else:
    data = {}  # Provide empty fallback
    print(f"Warning: {data_file} not found")
```

3. **Format JSON for Readability**:
```python
# Use indent for human-readable embedded data
json.dumps(data, indent=8)  # Nice indentation for viewing HTML source
```

4. **Consider Data Size**:
   - Embedding works well for dashboards (typically < 5MB)
   - For very large datasets (> 10MB), consider splitting into multiple pages
   - State-by-state data is fine to embed, tract-level data may be too large

### When to Use This Pattern

**Use Embedded Data When**:
- ✅ Generating static HTML dashboards for local viewing
- ✅ Data size is reasonable (< 5MB)
- ✅ Need self-contained single-file deliverable
- ✅ Avoiding server setup complexity

**Don't Use Embedded Data When**:
- ❌ Building a web application with server (use API instead)
- ❌ Data is extremely large (> 10MB)
- ❌ Need real-time data updates
- ❌ Multiple pages need to share same data (use shared JSON file with proper server)

---

## Summary: Quick Reference

**When starting a new script**:
1. ✅ Add `--dpi` parameter (if creates visualizations)
2. ✅ Add `--force` parameter (for skip logic)
3. ✅ Check `TQDM_POSITION` environment variable
4. ✅ Implement `report_progress()` function
5. ✅ Use `.zfill(11)` for all GEOIDs
6. ✅ Add `STATE_CODE_TO_NAME` mapping if merging state data
7. ✅ Follow file naming conventions
8. ✅ Add skip logic for existing outputs
9. ✅ Use thin white + thick black boundary pattern for maps

**When debugging**:
1. Check GEOID types (str vs int with `.zfill(11)`)
2. Check state code formats (AL vs ALABAMA vs alabama)
3. Check TQDM_POSITION is being passed
4. Check STATUS messages have `flush=True`
5. Check DPI is being threaded through
6. Check file paths are platform-independent (use `Path`)

**When committing**:
1. Use conventional commit format
2. Add detailed description
3. Include Co-Authored-By line
4. Test the change before committing
