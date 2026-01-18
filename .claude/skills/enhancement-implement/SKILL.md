---
name: enhancement-implement
description: Execute an enhancement following the standard workflow with progress tracking. Use when implementing a planned enhancement from ../../context/enhancements/. Creates todo lists, follows phases sequentially, uses STATUS protocol for progress reporting.
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

# Enhancement Implementation Skill

## Overview

This skill executes planned enhancements following the project's standard workflow. It creates todo lists from enhancement phases, implements changes incrementally, and validates at each step.

## Prerequisites

- **Read `../../context/ENHANCEMENT_WORKFLOW.md`** - Detailed 6-phase process (research, planning, implementation, testing, documentation, completion)
- Enhancement must be documented in `../../context/enhancements/active/` with status 📋 PLANNED
- Enhancement spec includes detailed phases and file lists
- All data dependencies are available or noted
- Check `../../context/enhancements/INDEX.md` for enhancement status

## When to Use This Skill

- User says: "Let's implement [enhancement name]"
- User says: "Execute enhancement XX"
- After `/enhancement-plan` skill completes and user approves
- When ready to start coding a planned feature

## Workflow

### Step 1: Read Enhancement Specification

1. Check `../../context/enhancements/INDEX.md` to locate the enhancement file
2. Read the enhancement file from `../../context/enhancements/active/XX_name.md`
3. Extract all phases, tasks, and file lists
4. Note any dependencies or prerequisites
5. Review testing plan

### Step 2: Create Todo List

Use TodoWrite tool to create task list from enhancement phases:

```python
todos = [
    {"content": "Phase 1: Core Implementation", "status": "pending", "activeForm": "Implementing core functionality"},
    {"content": "Phase 2: Pipeline Integration", "status": "pending", "activeForm": "Integrating with pipeline"},
    {"content": "Phase 3: Testing & Validation", "status": "pending", "activeForm": "Testing and validating"},
    {"content": "Phase 4: Documentation", "status": "pending", "activeForm": "Updating documentation"}
]
```

### Step 3: Execute Phases Sequentially

For each phase:

1. **Mark phase as in_progress** using TodoWrite
2. **Implement the phase**:
   - Read files that need modification
   - Make changes incrementally (one component at a time)
   - Follow patterns from `../../context/CODING_PATTERNS.md`
   - Use STATUS protocol for child processes
3. **Test the phase**:
   - Run print-only mode if applicable
   - Test with small state (VT/DE)
   - Validate outputs
4. **Mark phase as completed** using TodoWrite
5. **Move to next phase**

### Step 4: Follow Implementation Best Practices

**From CODING_PATTERNS.md:**

**Progress Reporting (if creating scripts)**:
```python
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0:
    print(f"STATUS:{position}:{msg}", flush=True)
```

**Skip Logic Pattern**:
```python
if output_file.exists() and not force:
    if is_standalone:
        print(f"[SKIP] Output already exists: {output_file}")
    else:
        report_progress(f"[SKIP] Already exists")
    return
```

**Windows-Safe Console Output**:
- Use ASCII: `[OK]`, `[FAIL]`, `[WARN]` not Unicode (✓/✗)
- All print statements must be Windows CP1252 compatible

**Path Handling**:
```python
from pathlib import Path
tract_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')
```

### Step 5: Testing Sequence

Follow the standard testing pattern:

1. **Print-Only First** (catches parameter threading issues):
   ```bash
   python script.py --print-only --year 2020 --version test
   ```

2. **Small State Test** (quick validation, 30 sec - 2 min):
   ```bash
   python script.py --state VT --year 2020 --version test
   python script.py --state DE --year 2010 --version test
   ```

3. **Multi-Year Test** (if year-dependent):
   ```bash
   # Test all supported census years (2000, 2010, 2020)
   ```

4. **Quantitative Validation** (if applicable):
   - Compare metrics before/after
   - Document improvements with specific numbers
   - Test statistical significance if needed

5. **Full Pipeline Spot-Check** (optional):
   ```bash
   run_redistricting.bat --year 2020 --version test --states "VT,DE,WY"
   ```

### Step 6: Mark Enhancement In Progress

Update the enhancement file in `../../context/enhancements/active/XX_name.md`:
```markdown
**Status**: 🔄 IN PROGRESS
**Started**: [Date]
```

Also update `../../context/enhancements/INDEX.md` to reflect the status change.

## Key Patterns

### Incremental Development

- Make changes one component at a time
- Prefer safe, manual edits for critical changes
- Test after each significant change
- Keep system in working state

### Error Handling

Common issues and solutions:

**Unicode Errors (Windows)**:
- Problem: `UnicodeEncodeError: 'charmap' codec can't encode character`
- Solution: Replace Unicode with ASCII: ✓ → [OK], ✗ → [FAIL], → → ->

**GEOID Type Mismatches**:
```python
# Force GEOID as string type
tracts = pd.read_csv(file, dtype={'GEOID': str})
```

**Missing Data**:
```python
# Check data availability before adding analysis
election_data_available = (year == '2020' and election_file.exists())
if election_data_available:
    # Run analysis
else:
    print(f"[SKIP] Election data not available for {year}")
```

**Path Compatibility**:
```python
# Support both old and new paths during transitions
graph_file_new = Path(f'data/adjacency/{year}/{state}_adjacency_{year}.pkl')
graph_file_old = Path(f'data/adjacency/{state}_adjacency_{year}.pkl')
graph_file = graph_file_new if graph_file_new.exists() else graph_file_old
```

### Scope-Based Pattern (for analysis scripts)

If creating new analysis script:
```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')

if args.scope == 'state':
    # Per-state processing (runs in parallel)
    process_single_state(args)
elif args.scope == 'national':
    # National aggregation (runs in post-processing)
    aggregate_all_states(args)
```

## What NOT to Do

From project anti-patterns:

```python
# ❌ DON'T: Hardcode census year
tracts_file = f'data/raw/{state}_tracts_2020.parquet'

# ✅ DO: Use year parameter
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'

# ❌ DON'T: Fail on missing optional data
if not election_data_file.exists():
    raise FileNotFoundError("Election data not found")

# ✅ DO: Skip gracefully
if not election_data_file.exists():
    print(f"[SKIP] Election data not available")
    return

# ❌ DON'T: Use Unicode in console output
print(f"✓ Complete")  # Crashes on Windows

# ✅ DO: Use ASCII
print(f"[OK] Complete")
```

## After Implementation

Once all phases complete:
1. Run full validation suite
2. Use `/enhancement-document` skill to update all documentation
3. Create git commit with clear message
4. Move enhancement file from `active/` to `completed/`
5. Update `../../context/enhancements/INDEX.md`
6. Mark enhancement as ✅ COMPLETED

## What You'll Get

- Complete implementation of planned enhancement
- All files modified/created per specification
- Validated outputs (tested incrementally)
- Todo list tracking for full transparency
- Code following established patterns

## Next Steps

After implementation:
- Use `/enhancement-document` to complete documentation
- Consider creating session archive with `/create-session-archive`
- Update CHANGELOG.md with dated entry
