# Coding Patterns and Conventions

This document captures critical patterns and conventions used throughout the codebase. Following these patterns ensures consistency, proper progress reporting, and correct behavior.

## Table of Contents
1. [Progress Bar Protocol](#progress-bar-protocol)
2. [Skip Logic Pattern](#skip-logic-pattern)
3. [DPI Threading](#dpi-threading)
4. [GEOID Handling](#geoid-handling-critical)
5. [Map Visualization](#map-visualization)
6. [Subprocess Communication](#subprocess-communication)
7. [File Naming Conventions](#file-naming-conventions)
8. [Git Commit Messages](#git-commit-messages)
9. [Error Handling](#error-handling)
10. [When to Use What](#when-to-use-what)

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

### Standard Pattern

**All pipeline scripts should check for existing outputs and skip unless forced.**

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
        return 0

    # ... do work ...
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

**Why**: Prevents expensive re-computation, enables pipeline resumption after failures.

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
tracts = gpd.read_parquet('data/raw/ca_tracts_2020.parquet')
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
data_dir = Path('data/raw')
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

## Summary: Quick Reference

**When starting a new script**:
1. ✅ Add `--dpi` parameter (if creates visualizations)
2. ✅ Add `--force` parameter (for skip logic)
3. ✅ Check `TQDM_POSITION` environment variable
4. ✅ Implement `report_progress()` function
5. ✅ Use `.zfill(11)` for all GEOIDs
6. ✅ Follow file naming conventions
7. ✅ Add skip logic for existing outputs
8. ✅ Use thin white + thick black boundary pattern for maps

**When debugging**:
1. Check GEOID types (str vs int)
2. Check TQDM_POSITION is being passed
3. Check STATUS messages have `flush=True`
4. Check DPI is being threaded through
5. Check file paths are platform-independent (use `Path`)

**When committing**:
1. Use conventional commit format
2. Add detailed description
3. Include Co-Authored-By line
4. Test the change before committing
