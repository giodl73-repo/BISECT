---
name: enhancement-implement
description: Execute enhancement following standard workflow with progress tracking. Creates todo lists, follows phases sequentially, uses STATUS protocol.
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

# Enhancement Implementation

Executes planned enhancements with todo tracking, incremental changes, step-by-step validation.

## Prerequisites
**Read**: context/ENHANCEMENT_WORKFLOW.md (6-phase process)
**Required**: Enhancement in `context/enhancements/active/` with 📋 PLANNED status, detailed phases/files, data deps noted
**Check**: context/enhancements/INDEX.md for status

## When to Use
User says: "implement [enhancement]", "execute enhancement XX" | After `/enhancement-plan` approval

## Workflow

### Step 1: Read Spec
1. Check `context/enhancements/INDEX.md` for location
2. Read `context/enhancements/active/XX_name.md`
3. Extract phases, tasks, file lists, deps, testing plan

### Step 2: Create Todos
**TodoWrite** from enhancement phases:
```python
todos = [
    {"content": "Phase 1: Core Implementation", "status": "pending", "activeForm": "Implementing core"},
    {"content": "Phase 2: Pipeline Integration", "status": "pending", "activeForm": "Integrating pipeline"},
    {"content": "Phase 3: Testing & Validation", "status": "pending", "activeForm": "Testing"},
    {"content": "Phase 4: Documentation", "status": "pending", "activeForm": "Updating docs"}
]
```

### Step 3: Execute Phases (Sequential)
**For each phase**:
1. **Mark in_progress** (TodoWrite)
2. **Implement**: Read files → Make changes incrementally → Follow CODING_PATTERNS.md → Use STATUS protocol
3. **Test**: Print-only → Small state (VT/DE) → Validate outputs
4. **Mark completed** (TodoWrite)
5. **Next phase**

### Step 4: Patterns (CODING_PATTERNS.md)

**STATUS Protocol** (scripts):
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
```

**Skip Logic**:
```python
if output_file.exists() and not force:
    report_progress("[SKIP] Already exists") if not is_standalone else print("[SKIP]...")
    return
```

**Windows-Safe Console**:
**Use**: `[OK]`, `[FAIL]`, `[WARN]`, `->`, `-` | **Never**: ✓, ✗, →

**Paths**:
```python
from pathlib import Path
tract_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')
```

### Step 5: Testing Sequence
1. **Print-only** (catches param threading): `script.py --print-only --year 2020 --version test`
2. **Small state** (30s-2m, VT/DE): `--state VT --year 2020 --version test`
3. **Multi-year** (if year-dependent): Test 2000/2010/2020
4. **Quantitative** (if applicable): Compare metrics before/after, document % improvements
5. **Full spot-check** (optional): `run_redistricting.bat --year 2020 --version test --states "VT,DE,WY"`

### Step 6: Mark In Progress
**Update** `context/enhancements/active/XX_name.md`:
```markdown
**Status**: 🔄 IN PROGRESS | **Started**: [Date]
```
**Also update**: context/enhancements/INDEX.md

## Key Patterns

### Incremental Development
• One component at a time • Manual edits for critical changes • Test after each change • Keep system working

### Common Issues → Solutions

**Unicode (Windows)**: `UnicodeEncodeError` → Replace ✓→[OK], ✗→[FAIL], →→->

**GEOID Type**:
```python
tracts = pd.read_csv(file, dtype={'GEOID': str})  # Force string
```

**Missing Data**:
```python
if not election_file.exists():
    print(f"[SKIP] Election data not available for {year}")
    return  # Skip gracefully, don't fail
```

**Path Compat** (transitions):
```python
file_new = Path(f'data/adjacency/{year}/{state}_adj_{year}.pkl')
file_old = Path(f'data/adjacency/{state}_adj_{year}.pkl')
file = file_new if file_new.exists() else file_old
```

### Scope-Based (analysis scripts)
```python
parser.add_argument('--scope', choices=['state', 'national'], default='state')
if args.scope == 'state': process_single_state(args)  # Parallel
elif args.scope == 'national': aggregate_all_states(args)  # Post-processing
```

## Anti-Patterns

```python
# ❌ Hardcode year
tracts = f'data/raw/{state}_tracts_2020.parquet'
# ✅ Use param
tracts = f'data/tracts/{year}/{state}_tracts_{year}.parquet'

# ❌ Fail on missing optional data
if not file.exists(): raise FileNotFoundError()
# ✅ Skip gracefully
if not file.exists(): print("[SKIP] Not available"); return

# ❌ Unicode console
print("✓ Complete")  # Crashes Windows
# ✅ ASCII
print("[OK] Complete")
```

## After Implementation
1. Run full validation
2. `/enhancement-document` to update all docs
3. Git commit (clear message)
4. Move `active/XX_name.md` → `completed/`
5. Update `context/enhancements/INDEX.md`
6. Mark ✅ COMPLETED

## Output
✅ Complete impl per spec
✅ All files modified/created
✅ Validated outputs (incremental testing)
✅ Todo tracking (full transparency)
✅ Code follows patterns

## Next Steps
**After**: `/enhancement-document` → `/create-session-archive` (optional) → Update CHANGELOG.md
