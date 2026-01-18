---
name: refactor-for-pattern
description: Refactor code to follow established coding pattern. Reads CODING_PATTERNS.md for target pattern, identifies code sections not following pattern (progress reporting, path handling, error handling, dual output modes), refactors to match pattern, tests before/after equivalence, and documents pattern in code comments. Use when code doesn't follow patterns in CODING_PATTERNS.md.
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

# Refactor for Pattern

## Overview

Refactor existing code to follow established project patterns documented in CODING_PATTERNS.md. Ensures consistency, maintainability, and adherence to best practices across the codebase.

## Prerequisites

**Required**:
- Target code file(s) to refactor
- CODING_PATTERNS.md documentation
- Git version control (for safety/rollback)

**Recommended**:
- Understanding of the pattern being applied
- Test suite to validate behavior preservation
- Clean working directory

## When to Use This Skill

- User says: "Refactor this to follow project patterns"
- User says: "Make this code consistent with CODING_PATTERNS.md"
- Code review identifies pattern violations
- New script doesn't follow established patterns
- Legacy code needs modernization
- After learning new best practice
- Before merging feature branch

## Common Refactoring Patterns

### Pattern 1: Add STATUS Protocol Support

**Purpose**: Enable coordinated progress reporting in parent-child process hierarchy

**Before**: No progress reporting or direct tqdm usage
```python
# No progress reporting
for state in states:
    process_state(state)

# Or direct tqdm (doesn't coordinate with parent)
from tqdm import tqdm
for state in tqdm(states):
    process_state(state)
```

**After**: STATUS protocol for child processes
```python
import os

position = int(os.environ.get('TQDM_POSITION', '-1'))

for i, state in enumerate(states):
    if position >= 0:
        progress_pct = (i + 1) / len(states) * 100
        print(f"STATUS:{position}:Processing {state} ({progress_pct:.0f}%)", flush=True)
    process_state(state)
```

**References**: CODING_PATTERNS.md Section 4 (Progress Reporting Protocol)

### Pattern 2: Add Scope-Based Architecture

**Purpose**: Handle both state-level and national-level processing in single script

**Before**: State-level only
```python
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--state', required=True)
    parser.add_argument('--year', required=True)
    args = parser.parse_args()

    data = load_state_data(args.state, args.year)
    results = analyze(data)
    save_results(results, args.state)
```

**After**: Scope-based with state and national modes
```python
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--scope', choices=['state', 'national'], default='state')
    parser.add_argument('--state', help='Required for scope=state')
    parser.add_argument('--year', required=True)
    args = parser.parse_args()

    # Validation
    if args.scope == 'state' and not args.state:
        parser.error('--state required for scope=state')

    if args.scope == 'state':
        # Per-state analysis
        data = load_state_data(args.state, args.year)
        results = analyze(data)
        save_state_results(results, args.state, args.year)
    elif args.scope == 'national':
        # National aggregation
        all_data = load_all_states(args.year)
        results = analyze_national(all_data)
        save_national_results(results, args.year)
```

**References**: CODING_PATTERNS.md Section 7 (Scope-Based Analysis Pattern)

### Pattern 3: Add Skip Logic for Missing Data

**Purpose**: Gracefully handle missing optional data without crashing

**Before**: Assumes data exists
```python
# Crashes if election data missing
election_data = pd.read_csv('election_2020.csv')
analyze_political(election_data)
```

**After**: Check availability and skip gracefully
```python
from pathlib import Path

election_file = Path('election_2020.csv')
if election_file.exists() and args.year == '2020':
    election_data = pd.read_csv(election_file)
    analyze_political(election_data)
    print("[OK] Political analysis complete")
else:
    print(f"[SKIP] Political analysis (no 2020 election data for {args.year})")
```

**References**: CODING_PATTERNS.md Section 5 (Conditional Analysis)

### Pattern 4: Replace Unicode with ASCII

**Purpose**: Windows console compatibility (CP1252 encoding)

**Before**: Uses Unicode characters
```python
print("✓ Success")
print("✗ Failed")
print("→ Next step")
```

**After**: Uses ASCII alternatives
```python
print("[OK] Success")
print("[FAIL] Failed")
print("-> Next step")
```

**References**: CLAUDE.md Windows-Specific section

### Pattern 5: Use Path Objects

**Purpose**: Cross-platform path handling

**Before**: String concatenation
```python
data_file = 'data/tracts/' + state + '_tracts_' + year + '.parquet'
```

**After**: Path objects
```python
from pathlib import Path

data_file = Path('data') / 'tracts' / year / f'{state}_tracts_{year}.parquet'
```

**References**: CODING_PATTERNS.md Section 3 (Path Handling)

### Pattern 6: Year-Specific Paths

**Purpose**: Support multiple census years consistently

**Before**: Hardcoded year
```python
tracts_file = f'data/tracts/{state}_tracts_2020.parquet'
```

**After**: Parameterized year
```python
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'
```

**References**: CODING_PATTERNS.md Section 2 (File Naming Conventions)

## Workflow

### Step 1: Identify Pattern Violations

**Read CODING_PATTERNS.md**:
```bash
# Review patterns documentation
cat ../../context/CODING_PATTERNS.md
```

**Scan code for violations**:
```bash
# Find Unicode characters in Python files
grep -r "[✓✗→]" scripts/ --include="*.py"

# Find hardcoded 2020 instead of {year}
grep -r "2020" scripts/ --include="*.py" | grep -v "# " | grep -v "default"

# Find string concatenation for paths
grep -r "'/'" scripts/ --include="*.py"

# Find scripts without STATUS protocol
grep -rL "STATUS:" scripts/ --include="*.py" | xargs grep -l "subprocess"
```

**Create checklist**:
```markdown
# Pattern Violations in scripts/political/analyze_districts.py

- [ ] No STATUS protocol support
- [ ] Uses Unicode characters (✓, ✗)
- [ ] String concatenation for paths
- [ ] No skip logic for missing election data
- [ ] Hardcoded 2020 year
```

### Step 2: Read Target Pattern

**Example**: Adding STATUS protocol support

**Read pattern documentation**:
```markdown
From CODING_PATTERNS.md:

## Progress Reporting Protocol

Child processes report progress via STATUS messages:

```python
import os

position = int(os.environ.get('TQDM_POSITION', '-1'))

if position >= 0:
    print(f"STATUS:{position}:{message}", flush=True)
```

Parent processes:
- Set TQDM_POSITION environment variable
- Parse STATUS messages
- Update corresponding progress bar
```

### Step 3: Plan Refactoring

**Use TodoWrite to track**:
```python
todos = [
    {"content": "Add STATUS protocol support", "status": "pending"},
    {"content": "Replace Unicode with ASCII", "status": "pending"},
    {"content": "Convert paths to Path objects", "status": "pending"},
    {"content": "Add skip logic for election data", "status": "pending"},
    {"content": "Parameterize year in paths", "status": "pending"},
    {"content": "Test before/after equivalence", "status": "pending"},
]
```

**Estimate impact**:
- Lines to change: ~15-20
- Functions to modify: 3-4
- Risk level: Low (behavior-preserving)
- Testing needed: Run on VT, DE to verify

### Step 4: Apply Refactoring

#### Refactoring 1: Add STATUS Protocol

**Read current code**:
```python
# Current (no progress reporting)
def process_all_states(states, year, version):
    for state in states:
        process_state(state, year, version)
```

**Refactor to add STATUS**:
```python
# Refactored (with STATUS protocol)
import os

def process_all_states(states, year, version):
    position = int(os.environ.get('TQDM_POSITION', '-1'))

    for i, state in enumerate(states):
        if position >= 0:
            progress_pct = (i + 1) / len(states) * 100
            print(f"STATUS:{position}:Processing {state} ({progress_pct:.0f}%)", flush=True)

        process_state(state, year, version)
```

#### Refactoring 2: Replace Unicode

**Find Unicode usage**:
```bash
grep -n "[✓✗→]" scripts/political/analyze_districts.py
```

**Use Edit tool to replace**:
```python
# Line 47
# Before:
print("✓ Political analysis complete")

# After:
print("[OK] Political analysis complete")

# Line 103
# Before:
print("✗ Election data not found")

# After:
print("[FAIL] Election data not found")
```

#### Refactoring 3: Convert to Path Objects

**Before**:
```python
data_file = 'outputs/us_' + year + '_' + version + '/states/' + state + '/data/political_lean.csv'
```

**After**:
```python
from pathlib import Path

data_file = Path('outputs') / f'us_{year}_{version}' / 'states' / state / 'data' / 'political_lean.csv'
```

#### Refactoring 4: Add Skip Logic

**Before**:
```python
# Assumes election data exists
election_file = f'data/election/2020/{state}_election.csv'
election_data = pd.read_csv(election_file)
```

**After**:
```python
from pathlib import Path

election_file = Path(f'data/election/{year}/{state}_election.csv')

if election_file.exists() and year == '2020':
    election_data = pd.read_csv(election_file)
    analyze_political_lean(election_data)
    print(f"[OK] Political analysis complete: {state}")
else:
    print(f"[SKIP] Political analysis (no election data for {year})")
```

#### Refactoring 5: Parameterize Year

**Find hardcoded years**:
```bash
grep -n "2020" scripts/political/analyze_districts.py | grep -v "#" | grep -v "default"
```

**Replace with parameter**:
```python
# Before:
tracts_file = f'data/tracts/2020/{state}_tracts_2020.parquet'

# After:
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'
```

### Step 5: Update Documentation

**Add pattern comments**:
```python
"""
Analyze political districts at state or national scope.

Follows project patterns:
- STATUS protocol for progress reporting (CODING_PATTERNS.md §4)
- Scope-based architecture (CODING_PATTERNS.md §7)
- Graceful skipping for missing data (CODING_PATTERNS.md §5)
- Path objects for cross-platform compatibility (CODING_PATTERNS.md §3)
- ASCII output for Windows compatibility (CLAUDE.md)
"""
```

**Update function docstrings**:
```python
def process_state(state, year, version):
    """
    Process political analysis for a single state.

    Reports progress via STATUS protocol if TQDM_POSITION set.
    Skips gracefully if election data not available.

    Args:
        state: State name (lowercase with underscores)
        year: Census year (2000, 2010, 2020)
        version: Pipeline version tag
    """
```

### Step 6: Test Before/After

**Save original**:
```bash
# Create backup
cp scripts/political/analyze_districts.py scripts/political/analyze_districts.py.orig
```

**Run original**:
```bash
# Test original version
python scripts/political/analyze_districts.py.orig \
  --state vermont --year 2020 --version test_original \
  --scope state
```

**Run refactored**:
```bash
# Test refactored version
python scripts/political/analyze_districts.py \
  --state vermont --year 2020 --version test_refactored \
  --scope state
```

**Compare outputs**:
```bash
# Should be identical (except for timestamps/messages)
diff outputs/us_2020_test_original/states/vermont/data/political_lean.csv \
     outputs/us_2020_test_refactored/states/vermont/data/political_lean.csv
```

**Test STATUS protocol**:
```python
# Test that STATUS messages work
import subprocess
import os

env = os.environ.copy()
env['TQDM_POSITION'] = '2'

result = subprocess.run(
    [sys.executable, 'scripts/political/analyze_districts.py',
     '--state', 'vermont', '--year', '2020', '--version', 'test',
     '--scope', 'state'],
    env=env,
    capture_output=True,
    text=True
)

# Should see STATUS messages in stdout
assert 'STATUS:2:' in result.stdout
```

### Step 7: Document Changes

**Update commit message**:
```bash
git commit -m "Refactor analyze_districts.py to follow project patterns

- Add STATUS protocol support for progress reporting
- Replace Unicode characters with ASCII ([OK], [FAIL], ->)
- Convert string paths to Path objects
- Add skip logic for missing election data (non-2020 years)
- Parameterize hardcoded 2020 year
- Add pattern reference comments in docstrings

Tested: VT, DE produce identical outputs
Follows: CODING_PATTERNS.md §3, §4, §5, §7

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

## Common Refactoring Scenarios

### Scenario 1: Add STATUS Protocol to Script

**Target**: Script that's called by pipeline but has no progress reporting

**Steps**:
1. Check if script called by parent with subprocess
2. Add `position = int(os.environ.get('TQDM_POSITION', '-1'))`
3. Add STATUS messages at key progress points
4. Test with TQDM_POSITION set

**Testing**:
```bash
TQDM_POSITION=2 python script.py --args
# Should see STATUS:2: messages
```

### Scenario 2: Modernize Legacy Script

**Target**: Old script without year parameterization or skip logic

**Steps**:
1. Add --year parameter
2. Replace hardcoded 2020 with {year}
3. Add availability checks for year-specific data
4. Add skip messages for unavailable data
5. Test with 2010, 2020

### Scenario 3: Fix Windows Compatibility

**Target**: Script with Unicode that crashes on Windows

**Steps**:
1. Find Unicode: `grep -n "[^\x00-\x7F]" script.py`
2. Replace with ASCII equivalents
3. Test on Windows or with CP1252 encoding

### Scenario 4: Consolidate Duplicate Code

**Target**: Similar code blocks across multiple functions

**Steps**:
1. Extract common code to helper function
2. Move helper to library if used across scripts
3. Update all call sites
4. Test each caller

### Scenario 5: Add Scope-Based Architecture

**Target**: State-only script that needs national capability

**Steps**:
1. Add --scope parameter
2. Refactor into process_state() and process_national()
3. Branch in main() based on scope
4. Test both scopes

## Refactoring Checklist

Before refactoring:
- [ ] Read relevant sections of CODING_PATTERNS.md
- [ ] Create backup of original file
- [ ] Commit clean working directory
- [ ] Write test plan

During refactoring:
- [ ] Use TodoWrite to track changes
- [ ] Make one pattern change at a time
- [ ] Test after each change
- [ ] Add pattern reference comments

After refactoring:
- [ ] Test with multiple states/years
- [ ] Compare outputs before/after
- [ ] Update docstrings
- [ ] Document changes in commit
- [ ] Update CODING_PATTERNS.md if new pattern

## Troubleshooting

### Refactoring Breaks Functionality

**Symptom**: Outputs differ or script crashes

**Causes**:
- Logic error in refactoring
- Missed edge case
- Changed behavior unintentionally

**Solutions**:
- Revert to backup: `cp script.py.orig script.py`
- Apply changes more incrementally
- Add more tests for edge cases
- Review diff carefully

### STATUS Messages Don't Appear

**Symptom**: No progress updates in parent

**Causes**:
- TQDM_POSITION not set
- Missing `flush=True`
- Output captured instead of displayed

**Solutions**:
```python
# Always flush
print(f"STATUS:{position}:{msg}", flush=True)

# Check environment
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0:
    # Only print if position set
    print(f"STATUS:{position}:{msg}", flush=True)
```

### Path Refactoring Causes File Not Found

**Symptom**: Files not found after Path refactoring

**Causes**:
- Incorrect path construction
- Forward slash vs backslash on Windows
- Relative vs absolute paths

**Solutions**:
```python
# Use Path objects correctly
from pathlib import Path

# Correct
data_file = Path('data') / 'tracts' / year / f'{state}_tracts_{year}.parquet'

# Also correct (f-string)
data_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')

# Verify path exists
assert data_file.exists(), f"File not found: {data_file}"
```

### Unicode Replacement Changes Output

**Symptom**: Tests fail after Unicode replacement

**Causes**:
- Output comparison includes console messages
- Checkmark/cross used in data (not just display)

**Solutions**:
- Only replace console output Unicode
- Keep Unicode in data files if needed
- Update test comparisons to ignore messages

## Best Practices

1. **One pattern at a time**: Don't refactor multiple patterns simultaneously
2. **Test incrementally**: Verify after each pattern application
3. **Preserve behavior**: Refactoring should not change outputs
4. **Document patterns**: Add comments referencing CODING_PATTERNS.md
5. **Backup first**: Always save original before refactoring
6. **Commit frequently**: Each pattern change = one commit
7. **Update tests**: Ensure tests still pass after refactoring

## Related Skills

- `/consolidate-scripts` - Consolidate after refactoring to patterns
- `/update-docs` - Update CODING_PATTERNS.md with new patterns
- `/enhancement-implement` - Implement pattern as part of enhancement
- `/reorganize-directory-structure` - Reorganize to match patterns

## Performance Notes

**Refactoring impact**:
- STATUS protocol: Negligible (<0.1% overhead)
- Path objects: Negligible (modern Python optimized)
- Skip logic: Faster (avoids errors, unnecessary work)
- Scope-based: Negligible branching overhead

**Benefits**:
- Maintainability: Consistent code easier to understand
- Reliability: Pattern-following code more robust
- Scalability: Patterns designed for growth
- Collaboration: Team follows same patterns

## Next Steps

After refactoring:
- Run full test suite
- Update documentation
- Share patterns with team
- Apply same refactoring to similar scripts
- Consider automation (linters, formatters)
- Add patterns to code review checklist
