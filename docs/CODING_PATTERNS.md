# Coding Patterns and Conventions

This document captures critical patterns and conventions used throughout the codebase. Following these patterns ensures consistency, proper progress reporting, and correct behavior.

**Last Updated**: January 17, 2026

## Related Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and architectural decisions
- **[enhancements/INDEX.md](enhancements/INDEX.md)** - Enhancement tracking and specifications
- **[ENHANCEMENT_WORKFLOW.md](ENHANCEMENT_WORKFLOW.md)** - Enhancement implementation workflow
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
14. [Creating Presentation Visualizations with Real Data](#creating-presentation-visualizations-with-real-data)

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

**State-Level Directory Structure:**
```
outputs/us_{year}_{version}/states/{state_name}/
├── data/                                   # All CSV/pickle files
│   ├── final_assignments.pkl               # tract_idx -> district
│   ├── district_summary.csv                # District metrics
│   ├── district_cities.csv                 # Major cities per district
│   └── rounds_hierarchy.csv                # Bisection tree
├── maps/                                   # All visualization files
│   ├── all_districts.png                   # Final district map
│   ├── all_districts_with_cities.png       # With city labels
│   ├── rounds/
│   │   ├── round_01.png                    # Bisection progression (zero-padded)
│   │   ├── round_02.png
│   │   └── round_03.png
│   ├── districts/
│   │   ├── district_01.png                 # Individual district maps (zero-padded)
│   │   ├── district_02.png
│   │   └── district_03.png
│   └── metros/
│       ├── los_angeles.png                 # Metro area maps (2010+ only)
│       └── san_francisco.png
├── political/                              # Political analysis (optional)
│   ├── district_political.csv              # Partisan lean data
│   ├── rounds_political.csv
│   └── maps/
│       ├── partisan_lean.png
│       └── rounds/
│           └── round_01.png
├── demographic/                            # Demographics (optional)
│   ├── district_demographics.csv
│   └── maps/
│       ├── gender_balance.png
│       ├── majority_race.png
│       └── diversity_index.png
└── compactness/                            # Compactness (optional)
    ├── district_compactness.csv            # Fallback if not in district_summary
    └── maps/
        ├── polsby_popper.png
        └── reock.png
```

**National-Level Directory Structure:**
```
outputs/us_{year}_{version}/
├── data/                                   # Aggregated CSV files
│   ├── us_all_districts.csv                # All 435 districts
│   ├── us_district_summary.csv             # Summary statistics
│   └── us_rounds_hierarchy.csv             # National bisection tree
├── maps/                                   # National visualizations
│   ├── us_all_districts.png                # All 435 districts
│   ├── us_all_districts_with_cities.png    # With major cities
│   ├── rounds/
│   │   ├── round_01.png                    # 6 rounds (zero-padded)
│   │   ├── round_02.png
│   │   └── round_06.png
│   ├── political/
│   │   └── partisan_lean.png               # National political map
│   ├── demographic/
│   │   └── majority_demographics.png       # National demographics
│   └── compactness/
│       └── polsby_popper.png               # National compactness
└── index.html                              # Interactive dashboard

```

**Naming Rules:**
1. **No year suffixes** - Year is in directory path `us_{year}_{version}/`
2. **Snake_case only** - All lowercase with underscores
3. **Zero-padded numbers** - `round_01.png`, `district_01.png` (2-digit padding)
4. **Organized by type** - `data/` for CSVs, `maps/` for visualizations
5. **Consistent prefixes** - State: no prefix, National: `us_` prefix for CSVs

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

## Papers and Presentations Figure Management

### Pattern: Figures in outputs/

**All figures for papers and presentations must be generated to and referenced from `outputs/`**, not from source directories. This keeps the repository clean and follows the .gitignore pattern where all generated content goes to outputs/.

### Structure

```
papers/{paper_name}/
├── create_figures.py          # Generates all figures for this paper
├── compile.bat                # Runs create_figures.py, then LaTeX
├── {paper_name}.tex
└── sections/
    └── *.tex                  # Reference: ../../outputs/papers/{paper_name}/figures/

presentations/{presentation_name}/
├── create_figures.py          # Generates all figures for this presentation
├── compile.bat                # Runs create_figures.py, then LaTeX
├── presentation.tex
└── laymen_guide.tex

outputs/
├── papers/
│   └── {paper_name}/
│       ├── figures/
│       │   └── *.png         # Generated figures
│       └── {paper_name}.pdf  # Compiled PDF
└── presentations/
    └── {presentation_name}/
        ├── figures/
        │   └── *.png         # Generated figures
        ├── presentation.pdf
        └── laymen_guide.pdf
```

### Implementation Pattern

**1. Create `create_figures.py` Script**

```python
#!/usr/bin/env python3
"""Create all figures for Paper 3."""

from pathlib import Path
import shutil
import subprocess

# Create figures directory in outputs
figures_dir = Path('../../outputs/papers/03_combined_recursive_bisection/figures')
figures_dir.mkdir(parents=True, exist_ok=True)

# Option A: Copy from pipeline outputs
source = Path('../../outputs/us_2020_v1/states/minnesota/maps/rounds/round_01.png')
dest = figures_dir / 'minnesota_round_1.png'
shutil.copy2(source, dest)

# Option B: Call analysis script to generate figures
subprocess.run([
    'python', 'scripts/baseline/visualize_comparison.py',
    '--output-dir', str(figures_dir)
], check=True)

# Option C: Generate matplotlib figures directly
import matplotlib.pyplot as plt
fig, ax = plt.subplots()
# ... create figure ...
plt.savefig(figures_dir / 'custom_figure.png', dpi=150)
```

**2. Update `compile.bat` to Run Figure Generation First**

```batch
@echo off
REM ... setup code ...

echo [1/2] Generating figures...
python create_figures.py
if errorlevel 1 (
    echo [ERROR] Figure generation failed
    exit /b 1
)

echo [2/2] Compiling LaTeX...
pdflatex -interaction=nonstopmode paper.tex >nul 2>&1
REM ... rest of LaTeX compilation ...
```

**3. Reference Figures from outputs/ in LaTeX**

```latex
\begin{figure}[h]
\centering
\includegraphics[width=0.8\textwidth]{../../outputs/papers/03_combined_recursive_bisection/figures/minnesota_round_1.png}
\caption{Minnesota Round 1}
\end{figure}
```

### Why This Pattern?

**Benefits**:
1. ✅ **Clean repository**: No generated PNGs committed to git
2. ✅ **Consistent with pipeline**: All outputs go to outputs/
3. ✅ **Reproducible**: `compile.bat --reset` regenerates everything from scratch
4. ✅ **Single source of truth**: Pipeline outputs are the authoritative figures
5. ✅ **Easy updates**: Rerun pipeline → recompile papers → figures automatically updated

**Common Scenarios**:

- **Round progression maps**: Copy from `outputs/us_{year}_{version}/states/{state}/maps/rounds/`
- **Analysis charts**: Run visualization scripts with `--output-dir` pointing to outputs/
- **Custom diagrams**: Generate matplotlib figures directly to outputs/

### Figure Sources

**1. Pipeline Outputs (Copy)**
```python
# Copy existing pipeline outputs
pipeline_map = Path('outputs/us_2020_v1/states/alabama/maps/rounds/round_01.png')
paper_fig = Path('outputs/papers/03_combined/figures/alabama_round_1.png')
shutil.copy2(pipeline_map, paper_fig)
```

**2. Analysis Scripts (Generate)**
```python
# Call existing analysis scripts
subprocess.run([
    sys.executable,
    'scripts/baseline/visualize_three_way_comparison.py',
    '--comparison-csv', 'outputs/baseline_comparison_edge/three_way_comparison.csv',
    '--output-dir', 'outputs/papers/03_combined/figures/'
], check=True)
```

**3. Custom Figures (Create)**
```python
# Generate custom matplotlib figures
import matplotlib.pyplot as plt

fig, ax = plt.subplots(figsize=(10, 6))
# ... create visualization ...
plt.savefig('outputs/presentations/edge_weighted/figures/graph_example.png',
            dpi=150, bbox_inches='tight')
```

### Updating Existing Papers

If a paper currently references `figures/` directly:

**Before**:
```latex
\includegraphics[width=\textwidth]{figures/minnesota_round_1.png}
```

**After**:
```latex
\includegraphics[width=\textwidth]{../../outputs/papers/03_combined/figures/minnesota_round_1.png}
```

**Batch update** (in LaTeX files):
```bash
# Replace all figure references
sed -i 's|figures/|../../outputs/papers/03_combined/figures/|g' sections/*.tex
```

---

---

## Creating Presentation Visualizations with Real Data

### Overview

When creating visualizations for presentations or papers that show real METIS partitioning with actual census data, follow this pattern to create publication-quality figures that demonstrate the algorithm's behavior on real geography.

### Use Case

**Goal**: Create side-by-side comparison showing:
1. **Left panel**: Real geographic census tracts with METIS partition
2. **Right panel**: Abstract graph representation with edge weights and cut

**Example**: `presentations/edge_weighted_bisection/create_figures.py`

### Step-by-Step Pattern

#### 1. Select Contiguous Tracts Using BFS

**Never use random tracts** - they may not be contiguous, causing METIS to fail or produce disconnected partitions.

```python
import geopandas as gpd
from collections import deque

def find_contiguous_tracts(tracts_gdf, start_idx, n_tracts):
    """Use BFS to find n_tracts contiguous tracts starting from start_idx."""
    visited = set()
    queue = deque([start_idx])
    contiguous = []

    while queue and len(contiguous) < n_tracts:
        idx = queue.popleft()
        if idx in visited:
            continue

        visited.add(idx)
        contiguous.append(idx)

        # Find neighbors (tracts that touch this one)
        geom = tracts_gdf.iloc[idx].geometry
        for j in range(len(tracts_gdf)):
            if j not in visited and tracts_gdf.iloc[j].geometry.touches(geom):
                queue.append(j)

    return contiguous[:n_tracts]

# Use it
hennepin_tracts = tracts_gdf[tracts_gdf['COUNTYFP'] == '053']  # Hennepin County, MN
start_idx = 0  # Start from first tract
tract_indices = find_contiguous_tracts(hennepin_tracts, start_idx, n_tracts=12)
sample_tracts = hennepin_tracts.iloc[tract_indices].copy()
```

#### 2. Calculate Real Boundary Lengths

**CRITICAL**: Boundary lengths must be calculated using geometry intersection, not just checking if tracts touch.

```python
import numpy as np

# Build adjacency graph with real boundary lengths
adjacency = {i: [] for i in range(n_tracts)}
edge_weights = {}

for i in range(n_tracts):
    geom_i = sample_tracts.iloc[i].geometry
    for j in range(i + 1, n_tracts):
        geom_j = sample_tracts.iloc[j].geometry

        if geom_i.touches(geom_j):
            # Calculate actual shared boundary length
            boundary = geom_i.intersection(geom_j.boundary)
            if not boundary.is_empty:
                length_km = boundary.length / 1000  # meters to km

                # Handle unprojected coordinates (degrees)
                if length_km < 0.1:
                    length_km = boundary.length * 111  # degrees to km

                # Filter corner adjacencies (< 0.1 km)
                if length_km >= 0.1:
                    # Add to BOTH directions (symmetric graph)
                    adjacency[i].append(j)
                    adjacency[j].append(i)
                    edge_weights[(i, j)] = length_km
                    edge_weights[(j, i)] = length_km
```

**Common mistakes:**
- ❌ Only adding edge in one direction → non-symmetric graph
- ❌ Not filtering corner adjacencies → visual clutter
- ❌ Forgetting unprojected coordinate conversion → 0.0 km lengths

#### 3. Run METIS with Contiguity Parameters

**Must match production settings** from `run_state_redistricting.py`:

```python
from apportionment.partition.metis_wrapper import partition_graph

# Prepare inputs
adjacency_list = [adjacency.get(i, []) for i in range(n_tracts)]
vweights = [int(sample_tracts.iloc[i]['population']) for i in range(n_tracts)]
edge_weights_dict = {(i, j): int(w * 1000) for (i, j), w in edge_weights.items()}

# Run METIS with contiguity enforcement
membership = partition_graph(
    adjacency=adjacency_list,
    vertex_weights=vweights,
    nparts=2,
    target_weights=[0.5, 0.5],  # Equal population split
    recursive=True,              # Use recursive bisection
    ufactor=1.005,               # 0.5% imbalance tolerance
    edge_weights=edge_weights_dict,
    debug=False
)

# Identify cut edges (edges crossing partition boundary)
cut_edges = set()
for i in range(n_tracts):
    for j in adjacency.get(i, []):
        if i < j and membership[i] != membership[j]:
            cut_edges.add((i, j))
```

#### 4. Create Side-by-Side Visualization

**Use GridSpec for custom layout**:

```python
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec

fig = plt.figure(figsize=(16, 7))
gs = fig.add_gridspec(1, 2, width_ratios=[1.4, 1], wspace=0.15)
ax1 = fig.add_subplot(gs[0])  # Map
ax2 = fig.add_subplot(gs[1])  # Graph
```

**Left panel: Geographic map**

```python
partition_colors = {0: 'lightblue', 1: 'lightcoral'}

# Plot tracts colored by partition
for idx in range(n_tracts):
    color = partition_colors[membership[idx]]
    sample_tracts.iloc[[idx]].plot(
        ax=ax1,
        facecolor=color,
        edgecolor='black',
        linewidth=1.5,
        alpha=0.7
    )

    # Label each tract
    centroid = sample_tracts.iloc[idx].geometry.centroid
    pop_k = sample_tracts.iloc[idx]['population'] / 1000
    ax1.text(centroid.x, centroid.y, f'{labels[idx]}\n{pop_k:.1f}K',
            ha='center', va='center', fontsize=9, fontweight='bold',
            bbox=dict(boxstyle='round', facecolor='white',
                    edgecolor='black', linewidth=1))

# Highlight cut boundaries with thick red lines
for i, j in cut_edges:
    geom_i = sample_tracts.iloc[i].geometry
    geom_j = sample_tracts.iloc[j].geometry
    boundary = geom_i.intersection(geom_j.boundary)
    gpd.GeoSeries([boundary]).plot(ax=ax1, color='red',
                                    linewidth=4, alpha=0.9, zorder=10)

    # Label cut edges with yellow background
    length_km = edge_weights.get((i, j), 0)
    if length_km > 0.1:
        mid_point = boundary.centroid
        ax1.text(mid_point.x, mid_point.y, f'{length_km:.1f}',
                ha='center', va='center', fontsize=7, fontweight='bold',
                bbox=dict(boxstyle='round', facecolor='yellow',
                        edgecolor='red', linewidth=1.5, alpha=0.9),
                zorder=11)

ax1.axis('off')
```

**Right panel: Abstract graph**

```python
from matplotlib.patches import Circle

# Create node positions from tract centroids
centroids = sample_tracts.geometry.centroid
xs = [c.x for c in centroids]
ys = [c.y for c in centroids]

# Normalize to [0, 4] range
x_min, x_max = min(xs), max(xs)
y_min, y_max = min(ys), max(ys)
positions = {}
for i in range(n_tracts):
    x_norm = 4 * (xs[i] - x_min) / (x_max - x_min)
    y_norm = 4 * (ys[i] - y_min) / (y_max - y_min)
    positions[i] = (x_norm, y_norm)

# Draw edges (colored by cut status)
for i in range(n_tracts):
    for j in adjacency.get(i, []):
        if i < j:
            x1, y1 = positions[i]
            x2, y2 = positions[j]
            length_km = edge_weights.get((i, j), 0)

            if length_km > 0.1:  # Skip corner adjacencies
                is_cut = (i, j) in cut_edges

                if is_cut:
                    # Cut edge: thick red dashed with X marker
                    ax2.plot([x1, x2], [y1, y2], 'r--',
                            linewidth=4, alpha=0.9, zorder=2)
                    mid_x, mid_y = (x1 + x2) / 2, (y1 + y2) / 2
                    ax2.plot(mid_x, mid_y, 'rX', markersize=12,
                            markeredgewidth=3, zorder=3)
                else:
                    # Non-cut edge: gray, thickness by weight
                    thickness = min(6, max(1.5, length_km / 3))
                    ax2.plot([x1, x2], [y1, y2], 'k-',
                            linewidth=thickness, alpha=0.4, zorder=1)

# Draw nodes (vertices)
for i in range(n_tracts):
    x, y = positions[i]
    color = partition_colors[membership[i]]
    pop_k = sample_tracts.iloc[i]['population'] / 1000

    # Circle
    circle = Circle((x, y), 0.3, facecolor=color,
                   edgecolor='black', linewidth=2.5, alpha=0.8, zorder=4)
    ax2.add_patch(circle)

    # Label inside circle
    ax2.text(x, y + 0.06, labels[i], ha='center', va='center',
            fontsize=9, fontweight='bold', zorder=5)
    ax2.text(x, y - 0.09, f'{pop_k:.1f}K', ha='center', va='center',
            fontsize=9, fontweight='bold', zorder=5)

ax2.set_xlim(-0.5, 4.5)
ax2.set_ylim(-0.7, 5.0)
ax2.set_aspect('equal')
ax2.axis('off')
```

### Styling Guidelines

**Font sizes (publication quality)**:
- Tract labels: 9pt bold
- Edge weight labels: 7pt (internal), 7pt bold (cuts)
- Region labels: 11pt bold
- Summary boxes: 9pt bold

**Colors**:
- Partition 1 (blue): `lightblue` fill, `blue` border
- Partition 2 (red): `lightcoral` fill, `red` border
- Cut edges: Red with yellow label backgrounds
- Non-cut edges: Gray with white label backgrounds

**Z-ordering** (critical for overlaps):
- Base edges: zorder=1
- Cut edges: zorder=2, 10
- Vertices/circles: zorder=4
- Labels: zorder=5, 11 (for labels on red lines)

**Layout best practices**:
- Use GridSpec with width_ratios=[1.4, 1] (more space for map)
- No titles or captions - let visualization speak for itself
- Add yellow summary box with total cut length below each panel
- Position region labels to avoid tract overlaps

### Complete Example Structure

```python
#!/usr/bin/env python3
"""Create real tracts to graph visualization."""

from pathlib import Path
import geopandas as gpd
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec
from matplotlib.patches import Circle
from collections import deque

def find_contiguous_tracts(tracts_gdf, start_idx, n_tracts):
    """BFS to find contiguous tracts."""
    # ... implementation ...

def main():
    # 1. Load data
    tracts_gdf = gpd.read_file('tracts.shp')
    population_df = pd.read_csv('population.csv')
    tracts_gdf = tracts_gdf.merge(population_df, on='GEOID')

    # 2. Select contiguous tracts
    county_tracts = tracts_gdf[tracts_gdf['COUNTYFP'] == '053']
    tract_indices = find_contiguous_tracts(county_tracts, 0, n_tracts=12)
    sample_tracts = county_tracts.iloc[tract_indices].copy()

    # 3. Build adjacency with real boundary lengths
    adjacency, edge_weights = build_adjacency(sample_tracts)

    # 4. Run METIS
    membership = run_metis(sample_tracts, adjacency, edge_weights)

    # 5. Identify cut edges
    cut_edges = find_cut_edges(adjacency, membership)

    # 6. Create visualization
    fig = create_visualization(sample_tracts, adjacency, edge_weights,
                              membership, cut_edges)

    # 7. Save
    plt.savefig('output.png', dpi=150, bbox_inches='tight')
    plt.close()

if __name__ == '__main__':
    main()
```

### Testing and Validation

**Verify correctness**:
1. ✅ All tracts are contiguous (no disconnected components)
2. ✅ Adjacency graph is symmetric (edges in both directions)
3. ✅ Cut edges correctly identified (different partitions)
4. ✅ Boundary lengths reasonable (0.1-10 km typical for tracts)
5. ✅ No labels overlapping tracts or edges
6. ✅ Total cut matches sum of individual cut edge weights

**Common issues**:
- Non-contiguous tracts → Use BFS
- Asymmetric graph → Add edges in both directions
- 0.0 km boundaries → Convert degrees to km
- Overlapping labels → Adjust z-order, reposition region labels
- Corner adjacencies cluttering → Filter edges < 0.1 km

### When to Use This Pattern

✅ **Use for**:
- Presentation figures showing real algorithm behavior
- Paper figures demonstrating METIS on actual geography
- Educational materials explaining graph partitioning
- Comparison of geographic vs abstract representation

❌ **Don't use for**:
- Production pipeline (use dedicated scripts)
- Analysis requiring all 50 states (too slow)
- Simple illustrations (use synthetic examples)

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
