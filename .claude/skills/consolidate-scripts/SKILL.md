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

Merge duplicate or similar scripts into unified, parameterized implementations. Reduces code duplication, improves maintainability, follows DRY principle.

## Prerequisites
**Required**: Multiple scripts with similar functionality, git version control (safety/rollback), understanding of script purposes/differences
**Recommended**: Test suite for behavior validation, documentation of script usage, clean working directory

## When to Use
User says "Consolidate duplicate scripts/Merge similar scripts", multiple scripts with 80%+ identical code, scripts differing only in parameters/modes, scripts unifiable with flag (--scope, --mode, --type), code review identifies duplication, after implementing similar features across multiple scripts

## Consolidation Patterns

### Pattern 1: Scope-Based (Most Common)
**Before**: `analyze_state_political.py` + `analyze_national_political.py`
**After**: `analyze_districts.py --scope state` or `--scope national`
```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')
if args.scope == 'state':
    process_single_state(args)
elif args.scope == 'national':
    aggregate_all_states(args)
```
**Benefits**: Single codebase, shared functions (data loading/metrics), consistent behavior, easy to add new scopes

### Pattern 2: Mode-Based
**Before**: `run_weighted.py` + `run_unweighted.py`
**After**: `run_redistricting.py --mode weighted` or `--mode unweighted`

### Pattern 3: Type-Based
**Before**: `visualize_political.py` + `visualize_demographic.py` + `visualize_compactness.py`
**After**: `visualize_state.py --type political|demographic|compactness`

### Pattern 4: Library Extraction
**Before**: Duplicated functions across scripts (e.g., compute_polsby_popper in multiple files)
**After**: Extract to `src/apportionment/compactness.py` + import
```python
# src/apportionment/compactness.py
def compute_polsby_popper(geometry):
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
find scripts/ -name "*.py" | sort            # List all scripts
find scripts/ -name "*_state_*.py"           # Similar names
ls scripts/political/                        # Same directory (likely related)
```

**Analyze similarity**:
```bash
diff -u script1.py script2.py                # Compare
diff script1.py script2.py | wc -l           # Count differences
```

**Quantify duplication** (Python):
```python
import difflib
with open('script1.py') as f1, open('script2.py') as f2:
    matcher = difflib.SequenceMatcher(None, f1.readlines(), f2.readlines())
    ratio = matcher.ratio()
    print(f"Similarity: {ratio * 100:.1f}%")  # If >70%, strong candidate
```

**Example findings**:
```
Consolidation Candidates:
1. Political Analysis (85% similar): analyze_state_political.py (347 lines) + analyze_national_political.py (329 lines) → Difference: Data aggregation scope only
2. Demographic Analysis (90% similar): analyze_state_demographic.py (412 lines) + analyze_national_demographic.py (395 lines) → Difference: Output location, map generation
3. Compactness Analysis (88% similar): analyze_state_compactness.py (298 lines) + analyze_national_compactness.py (285 lines) → Difference: Per-state vs all-states processing
```

### Step 2: Analyze Differences
**Common difference patterns**:
**1. Input source**: State script (`states/{state}/data/political_lean.csv`) vs National (`national/data/political_lean_national.csv`)
**2. Processing loop**: State (single district loop) vs National (nested state→district loop)
**3. Output location**: State (`states/{state}/maps/political.png`) vs National (`national/maps/political_national.png`)
**4. Aggregation**: State (no aggregation) vs National (`pd.concat(state_summaries)`)

**Document differences**:
```markdown
# Consolidation Analysis: Political Analysis Scripts
Similarities (85%): Argument parsing, data loading, metric computation, visualization
Differences (15%): State scope (single state) vs National scope (all states + aggregation), output paths
Consolidation Strategy: Use --scope parameter to branch, extract shared code to helper functions
```

### Step 3: Design Consolidated Script
**Unified argument structure**:
```python
parser.add_argument('--year', type=str, required=True)
parser.add_argument('--version', type=str, required=True)
parser.add_argument('--scope', choices=['state', 'national'], default='state')
parser.add_argument('--state', type=str, help='State name (required for scope=state)')

# Validation
if args.scope == 'state' and not args.state:
    parser.error("--state required when --scope=state")
```

**Shared functions**:
```python
def load_political_data(year, version, state=None):
    if state:
        return pd.read_csv(f'outputs/us_{year}_{version}/states/{state}/data/political_lean.csv')
    else:
        # Aggregate all states
        data = []
        for state_dir in Path(f'outputs/us_{year}_{version}/states').iterdir():
            df = pd.read_csv(state_dir / 'data' / 'political_lean.csv')
            df['state'] = state_dir.name
            data.append(df)
        return pd.concat(data, ignore_index=True)
```

**Main execution logic**:
```python
def main():
    if args.scope == 'state':
        data = load_political_data(args.year, args.version, args.state)
        output_path = f'outputs/us_{args.year}_{args.version}/states/{args.state}/maps/political_lean.png'
        visualize_political_map(data, output_path, 'state')
    elif args.scope == 'national':
        data = load_political_data(args.year, args.version)
        output_path = f'outputs/us_{args.year}_{args.version}/national/maps/political_lean_national.png'
        visualize_political_map(data, output_path, 'national')
```

### Step 4: Implement Consolidation
**Create consolidated script**:
```bash
cp scripts/political/analyze_state_political.py scripts/political/analyze_districts.py
# Edit to add --scope logic
```

**Use TodoWrite to track**:
```python
[
  {"content": "Create consolidated analyze_districts.py", "status": "completed"},
  {"content": "Add --scope parameter", "status": "in_progress"},
  {"content": "Extract shared functions", "status": "pending"},
  {"content": "Implement state scope logic", "status": "pending"},
  {"content": "Implement national scope logic", "status": "pending"},
  {"content": "Test both scopes", "status": "pending"}
]
```

**Extract shared code** (if widely used):
```python
# Move to src/apportionment/political.py
def compute_partisan_lean(dem_votes, rep_votes):
    total_votes = dem_votes + rep_votes
    return (dem_votes / total_votes) * 100 if total_votes > 0 else 50.0

# In consolidated script:
from apportionment.political import compute_partisan_lean
```

### Step 5: Maintain Backward Compatibility
**Create wrapper scripts** (temporary):
```python
#!/usr/bin/env python3
# scripts/political/analyze_state_political.py (DEPRECATED)
"""DEPRECATED: Use analyze_districts.py --scope state instead. Wrapper maintained for backward compatibility."""
import sys, os
from pathlib import Path

script_dir = Path(__file__).parent
consolidated_script = script_dir / 'analyze_districts.py'
args = sys.argv[1:] + ['--scope', 'state']
os.execv(sys.executable, [sys.executable, str(consolidated_script)] + args)
```

**Add deprecation warning**:
```python
import warnings
warnings.warn("analyze_state_political.py is deprecated. Use: python analyze_districts.py --scope state",
              DeprecationWarning, stacklevel=2)
```

### Step 6: Update Callers
**Find all callers**:
```bash
grep -r "analyze_state_political.py" scripts/ --include="*.py"
grep -r "analyze_state_political.py" . --include="*.bat"
```

**Update pipeline scripts**:
```python
# Before:
subprocess.run([sys.executable, 'scripts/political/analyze_state_political.py', '--year', year, '--version', version, '--state', state])

# After:
subprocess.run([sys.executable, 'scripts/political/analyze_districts.py', '--year', year, '--version', version, '--scope', 'state', '--state', state])
```

### Step 7: Test Side-by-Side
**Run both old and new**:
```bash
python scripts/political/analyze_state_political.py --year 2020 --version test --state alabama
python scripts/political/analyze_districts.py --year 2020 --version test --scope state --state alabama

# Compare outputs
diff outputs/us_2020_test/states/alabama/data/political_lean.csv.old \
     outputs/us_2020_test/states/alabama/data/political_lean.csv
```

**Verify equivalence** (Python):
```python
import pandas as pd
old = pd.read_csv('political_lean_old.csv')
new = pd.read_csv('political_lean_new.csv')
pd.testing.assert_frame_equal(old, new, rtol=1e-6)  # Allow small float diffs
```

**Test all scopes**:
```bash
python analyze_districts.py --scope state --state alabama --year 2020 --version test
python analyze_districts.py --scope national --year 2020 --version test
```

### Step 8: Document Changes
**Update script docstring**:
```python
"""
Analyze political districts at state or national scope.

This consolidated script replaces:
- analyze_state_political.py (deprecated)
- analyze_national_political.py (deprecated)

Usage:
    python analyze_districts.py --scope state --state california --year 2020 --version v1
    python analyze_districts.py --scope national --year 2020 --version v1

See CODING_PATTERNS.md for scope-based pattern documentation.
"""
```

**Update CHANGELOG.md**:
```markdown
### Changed
- Consolidated political analysis scripts into analyze_districts.py
  - analyze_state_political.py → analyze_districts.py --scope state (deprecated wrapper remains)
  - analyze_national_political.py → analyze_districts.py --scope national (deprecated wrapper remains)

### Deprecated
- analyze_state_political.py (use analyze_districts.py --scope state)
- analyze_national_political.py (use analyze_districts.py --scope national)
```

**Update CODING_PATTERNS.md**:
```markdown
## Scope-Based Analysis Pattern
Use single script with --scope parameter instead of separate scripts.
Example: `scripts/political/analyze_districts.py`
```

### Step 9: Deprecate Old Scripts
**After validation period** (e.g., 1 month):
```bash
git rm scripts/political/analyze_state_political.py
git rm scripts/political/analyze_national_political.py
# Or move to archive: git mv scripts/political/analyze_state_political.py scripts/deprecated/
```

## Common Consolidation Scenarios

**1. State vs National Analysis**: Scope-based consolidation (add --scope parameter, extract shared functions, branch on scope in main, test both)
- Project examples: Political analysis (analyze_districts.py), Demographic analysis (analyze_demographics.py), Compactness analysis (analyze_compactness.py), Round visualization (visualize_rounds.py)

**2. Multiple Visualization Types**: Type-based consolidation (visualize_political.py + visualize_demographic.py + visualize_compactness.py → visualize_state.py --type political|demographic|compactness)

**3. Duplicated Utility Functions**: Library extraction (move to src/apportionment/{module}.py, import from library, write unit tests, update all callers)

**4. Parameter Variants**: Parameterize (run_with_tolerance_0.5.py + run_with_tolerance_1.0.py + run_with_tolerance_2.0.py → run_redistricting.py --tolerance 0.5|1.0|2.0)

## Troubleshooting

**Scripts not exactly equivalent**: Outputs differ slightly → Document expected differences, set random seeds explicitly, sort outputs for deterministic order, allow tolerance in comparisons
**Callers still use old scripts**: Old scripts still called → Search entire codebase (`grep -r "old_script.py"`), add deprecation warnings, create issue tracker for migration, update documentation
**Backward compatibility breaks**: Old usage patterns fail → Keep wrapper scripts longer, add aliases for old parameter names, document migration path clearly, version bump to signal breaking change
**Performance regression**: Consolidated script slower → Profile before/after, lazy import modules, optimize hot paths, consider separate scripts if performance critical

## Best Practices
Consolidate incrementally (one pair at a time), test thoroughly (compare outputs side-by-side), maintain compatibility (keep wrappers during transition), document changes (CHANGELOG/docstrings/patterns), extract libraries (move shared code to src/apportionment/), validate with users (ensure no workflow breakage), set deprecation timeline (clear communication about removal)

## Performance Notes
**Consolidation benefits**: Reduced maintenance (one codebase vs N), consistent behavior (shared functions), easier testing (single test suite), better documentation (one place)
**Runtime impact**: Usually negligible (<1% overhead), branching on --scope is fast, shared functions may be slightly slower due to generality, consider specialization if performance critical (rare)

## Next Steps
Update all callers, run full test suite, update documentation, set deprecation timeline, monitor for issues, remove old scripts after validation period

## Related Skills
`/refactor-for-pattern` (refactor to follow scope-based pattern), `/reorganize-directory-structure` (reorganize after consolidation), `/update-docs` (update documentation), `/enhancement-implement` (implement as formal enhancement)
