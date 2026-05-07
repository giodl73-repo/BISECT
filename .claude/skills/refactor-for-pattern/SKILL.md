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

# Refactor For Pattern

Refactor existing code to follow established patterns documented in CODING_PATTERNS.md. Ensures consistency, maintainability, and best practices across codebase.

## Prerequisites
**Required**: Target code file(s) to refactor, CODING_PATTERNS.md documentation, git version control (for safety/rollback)
**Recommended**: Understanding of pattern being applied, test suite to validate behavior preservation, clean working directory

## When to Use
User says "Refactor this to follow project patterns/Make this code consistent with CODING_PATTERNS.md", code review identifies pattern violations, new script doesn't follow established patterns, legacy code needs modernization, after learning new best practice, before merging feature branch

## Common Patterns

**1. STATUS Protocol** (Progress Reporting): Child processes send STATUS messages to parent, parent collects + updates tqdm bars, coordinated progress output without interleaved bars
**When**: Any script called from pipeline, any parallel processing scripts
**Pattern**: `pos = int(os.environ.get('TQDM_POSITION', '-1'))` + `if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)`

**2. Path Handling**: Use Path objects (not strings), resolve paths early, use relative imports for package code, absolute paths for scripts
**When**: All Python files with file I/O
**Pattern**: `from pathlib import Path` + `path = Path(args.input).resolve()` + `output_path = base_path / 'subdir' / 'file.txt'`

**3. Dual Output Modes** (State vs National): Single script with --scope parameter, shared functions for both modes, consistent behavior, efficient implementation
**When**: Analysis scripts (political, demographic, compactness)
**Pattern**: `parser.add_argument('--scope', choices=['state', 'national'])` + `if args.scope == 'state': ...` + `elif args.scope == 'national': ...`

**4. Error Handling**: Specific exceptions (not bare except), meaningful error messages, cleanup on failure, exit codes (0=success, 1=error)
**When**: All scripts (especially pipeline)
**Pattern**: `try: ... except SpecificError as e: logging.error(f"Failed: {e}"); sys.exit(1)` + `finally: cleanup()`

**5. Configuration Pattern**: Config modules (scripts/config_YYYY.py), import configs (not hardcode), type hints for clarity, immutable constants (CAPS)
**When**: Pipeline scripts, analysis scripts
**Pattern**: `from scripts.config_2020 import STATES, DISTRICT_COUNTS` + `target_districts = DISTRICT_COUNTS[state]`

## Workflow

### Step 1: Identify Pattern
**Via user request**: User specifies pattern ("refactor to use STATUS protocol")
**Via code review**: Grep for anti-patterns (`grep -r "print(" --include="*.py"` finds non-STATUS prints)
**Via CODING_PATTERNS.md**: Read context/CODING_PATTERNS.md to understand target pattern

### Step 2: Analyze Current Code
Read target file(s), identify sections violating pattern, document violations (file:line, what's wrong, why it matters), assess impact (how many files? how invasive? need tests?)

### Step 3: Plan Refactoring
Use TodoWrite to track: [ ] Read pattern documentation, [ ] Identify all violations, [ ] Refactor file 1, [ ] Refactor file 2, [ ] Test equivalence, [ ] Update documentation

**Prioritize**: High-impact files first (pipeline scripts), group related changes (all path handling together), minimize test breakage (preserve public APIs)

### Step 4: Implement Refactoring
**For each violation**: (1) Understand current behavior, (2) Apply pattern from CODING_PATTERNS.md, (3) Preserve exact behavior (no functional changes), (4) Add comments explaining pattern (reference CODING_PATTERNS.md)

**Example** (STATUS protocol):
```python
# Before
print(f"Processing {state}...")

# After
pos = int(os.environ.get('TQDM_POSITION', '-1'))
def status(msg):
    if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
status(f"Processing {state}...")  # Using STATUS protocol - see CODING_PATTERNS.md
```

**Example** (Path handling):
```python
# Before
output_file = f"outputs/{year}_{version}/states/{state}/data.csv"
with open(output_file, 'w') as f: f.write(data)

# After
from pathlib import Path  # Path handling pattern - see CODING_PATTERNS.md
output_dir = Path(f"outputs/{year}_{version}/states/{state}")
output_dir.mkdir(parents=True, exist_ok=True)
(output_dir / "data.csv").write_text(data)
```

### Step 5: Test Equivalence
**Before/after comparison**: Run old code → save output, run new code → save output, diff outputs (should be identical)
**Automated tests**: Run test suite (`pytest tests/`), verify all pass, add new tests for pattern compliance
**Manual validation**: Run refactored code on small test case, verify behavior unchanged, check edge cases

### Step 6: Document Pattern Usage
**In-code comments**: Add comments explaining pattern (`# Using STATUS protocol - see CODING_PATTERNS.md`)
**Update CODING_PATTERNS.md** (if needed): Add examples of correct usage, note common pitfalls, link to refactored files as examples
**Commit message**: Reference pattern (`Refactor to follow STATUS protocol`)

### Step 7: Update Related Code
Find similar violations (`grep -r "similar_anti_pattern"`), apply same refactoring, update callers (if function signatures changed)

### Step 8: Mark Complete
Update todo list, document in enhancement (if part of enhancement), archive old approach (if pattern changed)

## Refactoring Safety Checklist

**Before**: [ ] Code under version control, [ ] Tests exist and pass, [ ] Understand current behavior, [ ] Read target pattern docs, [ ] Plan changes (TodoWrite)
**During**: [ ] One pattern at a time, [ ] Preserve exact behavior, [ ] Add explanatory comments, [ ] Test incrementally
**After**: [ ] All tests pass, [ ] Behavior equivalent, [ ] No regressions, [ ] Code more maintainable, [ ] Pattern documented

## Common Pitfalls

**Mixing functional changes with refactoring**: Refactor AND fix bug at same time → Hard to debug → Refactor first (preserve behavior), then fix bugs in separate commit

**Breaking public APIs**: Change function signature used by other code → Breaks callers → Keep public APIs stable, refactor internals only, or update all callers systematically

**Over-refactoring**: Apply pattern where not needed → Code more complex → Only refactor where pattern clearly improves code

**Insufficient testing**: Refactor without verifying equivalence → Introduces bugs → Test before/after, run test suite, verify on real data

**Ignoring pattern documentation**: Guess at pattern → Implement incorrectly → Read CODING_PATTERNS.md carefully, follow examples exactly

## Performance Notes
**Typical time**: Simple pattern (single file, 15-30 min), moderate pattern (multiple files, 1-2h), complex pattern (many files + tests + callers, 3-6h)
**What takes time**: Understanding current code (20-30%), planning refactoring (10-15%), implementing changes (30-40%), testing equivalence (20-30%), documentation (10%)

## What You'll Get
Refactored code following established pattern (consistent with project standards), preserved behavior (verified equivalence), improved maintainability (easier to understand/modify), documented pattern usage (in-code comments + CODING_PATTERNS.md), test coverage (verified pattern works correctly)

## Next Steps
Review refactored code (final sanity check), run full test suite (verify no regressions), commit changes (with descriptive message referencing pattern), update CODING_PATTERNS.md (if pattern evolved), identify other files needing same refactoring (systematic cleanup)

## Related Skills
`/consolidate-scripts` (merge duplicate scripts), `/reorganize-directory-structure` (restructure directories), `/enhancement-implement` (implement enhancement following patterns), `/run-tests` (verify refactoring doesn't break tests)
