---
name: consolidate-scripts
description: Merge duplicate or similar scripts. Identifies scripts with similar functionality, analyzes differences (same logic with different parameters or modes), proposes consolidation (single script with mode flags, shared library functions), implements refactoring while maintaining backward compatibility, tests old and new side-by-side, and deprecates old scripts after validation. Use when multiple scripts do similar things with slight variations.
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - TodoWrite
user-invocable: true
---

# Consolidate Scripts

## Overview

Merge duplicate or similar scripts into unified, parameterized implementations. Reduces code duplication, improves maintainability, and follows DRY (Don't Repeat Yourself) principle.

## Prerequisites

**Required**:
- Multiple scripts with similar functionality
- Git version control (for safety/rollback)
- Understanding of script purposes and differences

**Recommended**:
- Test suite to validate behavior preservation
- Documentation of script usage
- Clean working directory

## When to Use This Skill

- User says: "Consolidate these duplicate scripts"
- User says: "Merge similar scripts into one"
- Multiple scripts with 80%+ identical code
- Scripts that differ only in parameters or modes
- Scripts that could be unified with a flag (--scope, --mode, --type)
- Code review identifies duplication
- After implementing similar features across multiple scripts

## Consolidation Patterns

### Pattern 1: Scope-Based Consolidation (Most Common)

**Before**: Separate scripts for state-level and national-level

```
scripts/political/analyze_state_political.py
scripts/political/analyze_national_political.py
```

**After**: Single script with `--scope` parameter

```
scripts/political/analyze_districts.py --scope state
scripts/political/analyze_districts.py --scope national
```

**Implementation**:
```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')

if args.scope == 'state':
    # Per-state processing
    process_single_state(args)
elif args.scope == 'national':
    # National aggregation
    aggregate_all_states(args)
```

**Benefits**:
- Single codebase to maintain
- Shared functions (data loading, metrics computation)
- Consistent behavior across scopes
- Easy to add new scopes (e.g., regional)

### Pattern 2: Mode-Based Consolidation

**Before**: Separate scripts for different modes

```
scripts/redistricting/run_weighted.py
scripts/redistricting/run_unweighted.py
```

**After**: Single script with `--mode` parameter

```
scripts/redistricting/run_redistricting.py --mode weighted
scripts/redistricting/run_redistricting.py --mode unweighted
```

### Pattern 3: Type-Based Consolidation

**Before**: Separate visualization scripts

```
scripts/visualization/visualize_political.py
scripts/visualization/visualize_demographic.py
scripts/visualization/visualize_compactness.py
```

**After**: Single script with `--type` parameter

```
scripts/visualization/visualize_state.py --type political
scripts/visualization/visualize_state.py --type demographic
scripts/visualization/visualize_state.py --type compactness
```

### Pattern 4: Library Extraction

**Before**: Duplicated functions across scripts

```python
# In multiple scripts:
def compute_polsby_popper(geometry):
    area = geometry.area
    perimeter = geometry.boundary.length
    return (4 * np.pi * area) / (perimeter ** 2)
```

**After**: Shared library function

```python
# src/apportionment/compactness.py
def compute_polsby_popper(geometry):
    """Compute Polsby-Popper compactness score."""
    area = geometry.area
    perimeter = geometry.boundary.length
    return (4 * np.pi * area) / (perimeter ** 2)

# In scripts:
from apportionment.compactness import compute_polsby_popper
```

## Workflow

### Step 1: Identify Candidates

**Find similar scripts**:
```bash
# List all Python scripts
find scripts/ -name "*.py" | sort

# Find scripts with similar names
find scripts/ -name "*_state_*.py"
find scripts/ -name "*_national_*.py"

# Find scripts in same directory (likely related)
ls scripts/political/
ls scripts/demographic/
ls scripts/compactness/
```

**Analyze similarity**:
```bash
# Compare two scripts
diff -u scripts/political/analyze_state_political.py \
        scripts/political/analyze_national_political.py

# Count differences
diff scripts/political/analyze_state_political.py \
     scripts/political/analyze_national_political.py | wc -l
```

**Quantify duplication**:
```python
# Calculate similarity ratio
import difflib

with open('script1.py') as f1, open('script2.py') as f2:
    lines1 = f1.readlines()
    lines2 = f2.readlines()

matcher = difflib.SequenceMatcher(None, lines1, lines2)
ratio = matcher.ratio()
print(f"Similarity: {ratio * 100:.1f}%")

# If ratio > 0.7 (70%), strong consolidation candidate
```

**Example findings**:
```
Consolidation Candidates:

1. Political Analysis Scripts (85% similar)
   - analyze_state_political.py (347 lines)
   - analyze_national_political.py (329 lines)
   Difference: Data aggregation scope only

2. Demographic Analysis Scripts (90% similar)
   - analyze_state_demographic.py (412 lines)
   - analyze_national_demographic.py (395 lines)
   Difference: Output location, map generation

3. Compactness Analysis Scripts (88% similar)
   - analyze_state_compactness.py (298 lines)
   - analyze_national_compactness.py (285 lines)
   Difference: Per-state vs all-states processing
```

### Step 2: Analyze Differences

**Read both scripts** to understand key differences.

**Common difference patterns**:

**1. Input source**:
```python
# State script:
data_file = f'outputs/us_{year}_{version}/states/{state}/data/political_lean.csv'

# National script:
data_file = f'outputs/us_{year}_{version}/national/data/political_lean_national.csv'
```

**2. Processing loop**:
```python
# State script:
for district in districts:
    # Process single district

# National script:
for state in states:
    for district in state_districts:
        # Process all districts
```

**3. Output location**:
```python
# State script:
output_file = f'outputs/us_{year}_{version}/states/{state}/maps/political.png'

# National script:
output_file = f'outputs/us_{year}_{version}/national/maps/political_national.png'
```

**4. Aggregation**:
```python
# State script:
# No aggregation, process as-is

# National script:
# Aggregate across all states
national_summary = pd.concat(state_summaries)
```

**Document differences**:
```markdown
# Consolidation Analysis: Political Analysis Scripts

## Similarities (85%)
- Argument parsing (--year, --version)
- Data loading logic
- Metric computation (partisan lean)
- Visualization generation

## Differences (15%)
- State scope: Processes single state
- National scope: Loops over all states
- Output paths differ by scope
- National aggregates state results

## Consolidation Strategy
Use --scope parameter to branch between state and national logic.
Extract shared code to helper functions.
```

### Step 3: Design Consolidated Script

**Unified argument structure**:
```python
import argparse

parser = argparse.ArgumentParser(description='Analyze political districts')
parser.add_argument('--year', type=str, required=True)
parser.add_argument('--version', type=str, required=True)
parser.add_argument('--scope', choices=['state', 'national'], default='state')
parser.add_argument('--state', type=str,  # Required if scope=state
                   help='State name (required for scope=state)')

args = parser.parse_args()

# Validation
if args.scope == 'state' and not args.state:
    parser.error("--state required when --scope=state")
```

**Shared functions**:
```python
def load_political_data(year, version, state=None):
    """Load political lean data for state or all states."""
    if state:
        # Single state
        file_path = f'outputs/us_{year}_{version}/states/{state}/data/political_lean.csv'
        return pd.read_csv(file_path)
    else:
        # All states
        data = []
        for state_dir in Path(f'outputs/us_{year}_{version}/states').iterdir():
            file_path = state_dir / 'data' / 'political_lean.csv'
            if file_path.exists():
                df = pd.read_csv(file_path)
                df['state'] = state_dir.name
                data.append(df)
        return pd.concat(data, ignore_index=True)

def compute_partisan_metrics(df):
    """Compute partisan metrics from political lean data."""
    # Shared computation logic
    return metrics

def visualize_political_map(df, output_path, scope):
    """Generate political map visualization."""
    # Shared visualization logic
    pass
```

**Main execution logic**:
```python
def main():
    args = parse_args()

    if args.scope == 'state':
        # State-level analysis
        data = load_political_data(args.year, args.version, args.state)
        metrics = compute_partisan_metrics(data)
        output_path = f'outputs/us_{args.year}_{args.version}/states/{args.state}/maps/political_lean.png'
        visualize_political_map(data, output_path, 'state')
        print(f"[OK] State analysis complete: {args.state}")

    elif args.scope == 'national':
        # National analysis
        data = load_political_data(args.year, args.version)
        metrics = compute_partisan_metrics(data)
        output_path = f'outputs/us_{args.year}_{args.version}/national/maps/political_lean_national.png'
        visualize_political_map(data, output_path, 'national')
        print(f"[OK] National analysis complete")

if __name__ == '__main__':
    main()
```

### Step 4: Implement Consolidation

**Create new consolidated script**:
```bash
# Create from one of the existing scripts
cp scripts/political/analyze_state_political.py \
   scripts/political/analyze_districts.py

# Edit to add --scope logic
# (Use Edit tool to modify)
```

**Use TodoWrite to track**:
```python
todos = [
    {"content": "Create consolidated analyze_districts.py", "status": "completed"},
    {"content": "Add --scope parameter", "status": "in_progress"},
    {"content": "Extract shared functions", "status": "pending"},
    {"content": "Implement state scope logic", "status": "pending"},
    {"content": "Implement national scope logic", "status": "pending"},
    {"content": "Test both scopes", "status": "pending"},
]
```

**Extract shared code**:
```python
# Move common functions to library if widely used
# src/apportionment/political.py
def compute_partisan_lean(dem_votes, rep_votes):
    """Compute partisan lean percentage."""
    total_votes = dem_votes + rep_votes
    if total_votes == 0:
        return 50.0  # Neutral
    return (dem_votes / total_votes) * 100

# In consolidated script:
from apportionment.political import compute_partisan_lean
```

### Step 5: Maintain Backward Compatibility

**Create wrapper scripts** (temporary):
```python
#!/usr/bin/env python3
# scripts/political/analyze_state_political.py (DEPRECATED)
"""
DEPRECATED: Use analyze_districts.py --scope state instead

This wrapper maintained for backward compatibility.
Will be removed in next major version.
"""
import sys
import os
from pathlib import Path

# Call consolidated script
script_dir = Path(__file__).parent
consolidated_script = script_dir / 'analyze_districts.py'

# Forward arguments and add --scope state
args = sys.argv[1:] + ['--scope', 'state']
os.execv(sys.executable, [sys.executable, str(consolidated_script)] + args)
```

**Add deprecation warning**:
```python
# In wrapper script
import warnings
warnings.warn(
    "analyze_state_political.py is deprecated. "
    "Use: python analyze_districts.py --scope state",
    DeprecationWarning,
    stacklevel=2
)
```

### Step 6: Update Callers

**Find all callers**:
```bash
# Find scripts that call the old script
grep -r "analyze_state_political.py" scripts/ --include="*.py"
grep -r "analyze_state_political.py" . --include="*.bat"
```

**Update pipeline scripts**:
```python
# Before:
subprocess.run([sys.executable, 'scripts/political/analyze_state_political.py',
               '--year', year, '--version', version, '--state', state])

# After:
subprocess.run([sys.executable, 'scripts/political/analyze_districts.py',
               '--year', year, '--version', version,
               '--scope', 'state', '--state', state])
```

**Update batch files**:
```batch
REM Before:
python scripts\political\analyze_state_political.py --year 2020 --version v1 --state california

REM After:
python scripts\political\analyze_districts.py --year 2020 --version v1 --scope state --state california
```

### Step 7: Test Side-by-Side

**Run both old and new**:
```bash
# Run old script
python scripts/political/analyze_state_political.py \
  --year 2020 --version test --state alabama

# Run new consolidated script
python scripts/political/analyze_districts.py \
  --year 2020 --version test --scope state --state alabama

# Compare outputs
diff outputs/us_2020_test/states/alabama/data/political_lean.csv.old \
     outputs/us_2020_test/states/alabama/data/political_lean.csv
```

**Verify equivalence**:
```python
# Compare CSV outputs
import pandas as pd

old = pd.read_csv('political_lean_old.csv')
new = pd.read_csv('political_lean_new.csv')

# Should be identical
assert old.equals(new), "Outputs differ!"

# Or allow for small floating point differences
pd.testing.assert_frame_equal(old, new, rtol=1e-6)
```

**Test all scopes**:
```bash
# State scope
python analyze_districts.py --scope state --state alabama --year 2020 --version test

# National scope
python analyze_districts.py --scope national --year 2020 --version test
```

### Step 8: Document Changes

**Update script docstring**:
```python
#!/usr/bin/env python3
"""
Analyze political districts at state or national scope.

This consolidated script replaces:
- analyze_state_political.py (deprecated)
- analyze_national_political.py (deprecated)

Usage:
    # State-level analysis
    python analyze_districts.py --scope state --state california --year 2020 --version v1

    # National analysis
    python analyze_districts.py --scope national --year 2020 --version v1

See CODING_PATTERNS.md for scope-based pattern documentation.
"""
```

**Update CHANGELOG.md**:
```markdown
## [Unreleased]

### Changed
- Consolidated political analysis scripts into analyze_districts.py
  - analyze_state_political.py → analyze_districts.py --scope state (deprecated wrapper remains)
  - analyze_national_political.py → analyze_districts.py --scope national (deprecated wrapper remains)
- Extracted shared functions to src/apportionment/political.py

### Deprecated
- analyze_state_political.py (use analyze_districts.py --scope state)
- analyze_national_political.py (use analyze_districts.py --scope national)
```

**Update CODING_PATTERNS.md**:
```markdown
## Scope-Based Analysis Pattern

When implementing analysis that operates at multiple scopes (state, national, regional),
use a single script with --scope parameter instead of separate scripts.

Example: `scripts/political/analyze_districts.py`

```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')
if args.scope == 'state':
    process_state(args)
elif args.scope == 'national':
    process_national(args)
```
```

### Step 9: Deprecate Old Scripts

**After validation period** (e.g., 1 month):

**Remove old scripts**:
```bash
# Remove deprecated scripts
git rm scripts/political/analyze_state_political.py
git rm scripts/political/analyze_national_political.py

# Or move to archive
mkdir -p scripts/deprecated/
git mv scripts/political/analyze_state_political.py scripts/deprecated/
git mv scripts/political/analyze_national_political.py scripts/deprecated/
```

**Update documentation**:
```markdown
## Removed
- analyze_state_political.py (replaced by analyze_districts.py --scope state)
- analyze_national_political.py (replaced by analyze_districts.py --scope national)
```

## Common Consolidation Scenarios

### Scenario 1: State vs National Analysis

**Pattern**: Scope-based consolidation

**Steps**:
1. Identify state and national variants
2. Add --scope parameter
3. Extract shared functions
4. Branch on scope in main()
5. Test both scopes

**Project examples**:
- Political analysis (analyze_districts.py)
- Demographic analysis (analyze_demographics.py)
- Compactness analysis (analyze_compactness.py)
- Round visualization (visualize_rounds.py)

### Scenario 2: Multiple Visualization Types

**Pattern**: Type-based consolidation

**Before**:
```
visualize_political.py
visualize_demographic.py
visualize_compactness.py
```

**After**:
```
visualize_state.py --type political
visualize_state.py --type demographic
visualize_state.py --type compactness
```

### Scenario 3: Duplicated Utility Functions

**Pattern**: Library extraction

**Consolidate**:
- Move to src/apportionment/{module}.py
- Import from library
- Write unit tests
- Update all callers

### Scenario 4: Parameter Variants

**Before**:
```
run_with_tolerance_0.5.py
run_with_tolerance_1.0.py
run_with_tolerance_2.0.py
```

**After**:
```
run_redistricting.py --tolerance 0.5
run_redistricting.py --tolerance 1.0
run_redistricting.py --tolerance 2.0
```

## Troubleshooting

### Scripts Not Exactly Equivalent

**Symptom**: Outputs differ slightly after consolidation

**Causes**:
- Floating point differences
- Random seed not fixed
- Order of operations changed

**Solutions**:
- Document expected differences
- Set random seeds explicitly
- Sort outputs for deterministic order
- Allow tolerance in comparisons

### Callers Still Use Old Scripts

**Symptom**: Old scripts still being called

**Causes**:
- Missed updates in pipeline
- External tools/documentation reference old scripts

**Solutions**:
- Search entire codebase: `grep -r "old_script.py"`
- Add deprecation warnings
- Create issue tracker for migration
- Update documentation

### Backward Compatibility Breaks

**Symptom**: Old usage patterns fail

**Causes**:
- Changed default behavior
- Different parameter names
- Missing backward compatibility wrappers

**Solutions**:
- Keep wrapper scripts longer
- Add aliases for old parameter names
- Document migration path clearly
- Version bump to signal breaking change

### Performance Regression

**Symptom**: Consolidated script slower

**Causes**:
- Added branching overhead
- Loaded unnecessary modules
- Inefficient data structures

**Solutions**:
- Profile before and after
- Lazy import modules
- Optimize hot paths
- Consider separate scripts if performance critical

## Best Practices

1. **Consolidate incrementally**: One pair at a time, not all at once
2. **Test thoroughly**: Compare outputs side-by-side
3. **Maintain compatibility**: Keep wrappers during transition
4. **Document changes**: Update CHANGELOG, docstrings, patterns
5. **Extract libraries**: Move shared code to src/apportionment/
6. **Validate with users**: Ensure no workflow breakage
7. **Set deprecation timeline**: Clear communication about removal

## Related Skills

- `/refactor-for-pattern` - Refactor to follow scope-based pattern
- `/reorganize-directory-structure` - Reorganize after consolidation
- `/update-docs` - Update documentation after consolidation
- `/enhancement-implement` - Implement as formal enhancement

## Performance Notes

**Consolidation benefits**:
- Reduced maintenance: One codebase instead of N
- Consistent behavior: Shared functions ensure consistency
- Easier testing: Single test suite
- Better documentation: One place to document

**Runtime impact**:
- Usually negligible (< 1% overhead)
- Branching on --scope is fast
- Shared functions may be slightly slower due to generality
- Consider specialization if performance critical (rare)

## Next Steps

After consolidation:
- Update all callers
- Run full test suite
- Update documentation
- Set deprecation timeline
- Monitor for issues
- Remove old scripts after validation period
