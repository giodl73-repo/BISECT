---
name: reorganize-directory-structure
description: Restructure directories following best practices. Analyzes current structure, proposes improvements following project patterns (year-specific data, organized outputs, function-based scripts), creates migration plan with validation, updates all path references, and documents changes. Use when directory structure has grown organically and needs cleanup.
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

# Reorganize Directory Structure

Systematically restructure project directories following best practices and established patterns. Ensures consistency, maintainability, and scalability as project grows.

## Prerequisites
**Required**: Full project codebase access, git version control (for safety/rollback), clean working directory (no uncommitted changes)
**Recommended**: Backup or git commit before starting, full test suite to validate paths after changes, documentation of current structure

## When to Use
User says "Organize the directories/Clean up the file structure/Restructure the data directories", directory structure has grown organically without plan, year-specific data mixed with generic data, scripts scattered across multiple locations, outputs not consistently organized, after major feature additions that changed organization needs

## Target Directory Patterns

**1. Year-Specific Data**: Pattern `data/{type}/{year}/`
Structure: `data/tracts/2020/`, `data/adjacency/2020/`, `data/districts/2020/`
Why: Separate data by census year, easy year comparison, clear data dependencies, scalable for additional years

**2. Function-Based Scripts**: Pattern `scripts/{function}/`
Structure: `scripts/pipeline/` (orchestration), `scripts/political/` (analysis), `scripts/demographic/` (analysis), `scripts/compactness/` (analysis), `scripts/data/` (download/build)
Why: Group by functionality (not chronology), clear separation of concerns, easier navigation, maintainable codebase

**3. Organized Outputs**: Pattern `outputs/us_{year}_{version}/`
Structure: `outputs/us_2020_v1/states/` (per-state results), `outputs/us_2020_v1/national/` (national aggregates), `outputs/us_2020_v1/figures/` (visualizations)
Why: Version control for outputs, compare across versions, isolate experiments, avoid overwriting results

**4. Artifacts by Type**: Pattern `artifacts/{type}/`
Structure: `papers/` (LaTeX papers), `presentations/` (Beamer presentations), `guides/` (educational guides), `figures/` (publication figures)
Why: Separate docs/code, clear publication pipeline, version controlled separately

## Workflow

### Step 1: Analyze Current Structure
Map existing directories: `find . -type d -maxdepth 3 | sort`, identify issues (mixed years, scattered scripts, inconsistent naming, redundant directories), document current structure (record where files are now)

### Step 2: Design Target Structure
Based on project patterns: Year-specific data (`data/{type}/{year}/`), function-based scripts (`scripts/{function}/`), organized outputs (`outputs/us_{year}_{version}/`), artifacts by type (`artifacts/{type}/`)

**Create directory map**:
```
# Current → Target
data/tracts_2020/ → data/tracts/2020/
data/tracts_2010/ → data/tracts/2010/
scripts/analyze_political.py → scripts/political/analyze_districts.py
scripts/analyze_demographic.py → scripts/demographic/analyze_districts.py
outputs/2020_v1/ → outputs/us_2020_v1/
```

### Step 3: Create Migration Plan
Use TodoWrite to track: [ ] Create new directory structure, [ ] Move data files, [ ] Move script files, [ ] Update path references in code, [ ] Update path references in docs, [ ] Run tests to validate, [ ] Remove old directories, [ ] Document changes

**Prioritize moves**: Non-breaking first (unused dirs), data files (before code), scripts (after data), test after each major category

### Step 4: Create New Directory Structure
```bash
mkdir -p data/{tracts,adjacency,districts}/{2000,2010,2020}
mkdir -p scripts/{pipeline,political,demographic,compactness,data,web,utils}
mkdir -p outputs
mkdir -p artifacts/{papers,presentations,guides,figures}
```

### Step 5: Move Files Systematically
**Use git mv** (preserves history): `git mv old/path new/path`

**Move data files**:
```bash
git mv data/tracts_2020/ data/tracts/2020/
git mv data/adjacency_2020/ data/adjacency/2020/
```

**Move script files**:
```bash
git mv scripts/analyze_political.py scripts/political/analyze_districts.py
git mv scripts/analyze_demographic.py scripts/demographic/analyze_districts.py
```

**Verify moves**: `git status` (check staged moves), `ls -R` (verify new structure)

### Step 6: Update Path References in Code
**Find all path references**: `grep -r "data/tracts_2020" --include="*.py"`, `grep -r "scripts/analyze_political" --include="*.py"`

**Update imports**:
```python
# Before
from scripts.analyze_political import analyze

# After
from scripts.political.analyze_districts import analyze
```

**Update file paths**:
```python
# Before
data_path = "data/tracts_2020/california_tracts.parquet"

# After
data_path = f"data/tracts/{year}/california_tracts.parquet"
```

**Update script invocations**:
```bash
# Before
python scripts/analyze_political.py --state CA

# After
python scripts/political/analyze_districts.py --state CA --scope state
```

### Step 7: Update Path References in Docs
**Find documentation references**: `grep -r "data/tracts_2020" --include="*.md"`, `grep -r "scripts/analyze" --include="*.md"`

**Update docs**: README.md (update file paths in examples), ARCHITECTURE.md (update directory structure diagrams), CLAUDE.md (update script references), docs/*.md (update all documentation)

### Step 8: Run Tests to Validate
**Run full test suite**: `pytest tests/` (verify all tests pass), check for import errors, check for file not found errors

**Manual testing**: Run key scripts (`python scripts/pipeline/run_complete_redistricting.py --print-only`), verify outputs created in correct locations, compare with pre-move outputs (ensure equivalence)

### Step 9: Remove Old Directories
**After validation passes**: `git rm -r old/directory/`, verify old paths completely gone (`find . -name "old_dir_name"`), commit changes with descriptive message

### Step 10: Document Changes
Update ARCHITECTURE.md with new structure, update CLAUDE.md with new script paths, update CHANGELOG.md (note directory reorganization), create migration guide (if others need to adapt), commit documentation changes

## Migration Safety Checklist

**Before migration**: [ ] Full git commit (clean state), [ ] Document current structure, [ ] Backup critical data (if not in git), [ ] Plan complete migration (TodoWrite)

**During migration**: [ ] Use `git mv` (preserves history), [ ] One category at a time (data, then scripts, then outputs), [ ] Update references immediately (after each move), [ ] Test incrementally (after each category)

**After migration**: [ ] All tests pass, [ ] No broken imports, [ ] No file not found errors, [ ] Documentation updated, [ ] Old directories removed

## Common Pitfalls

**Breaking imports**: Move file without updating imports → Import errors → Update imports in same commit as move

**Forgetting documentation**: Update code but not docs → Docs out of date → Update README/ARCHITECTURE/CLAUDE.md systematically

**Incomplete path updates**: Update some paths but miss others → Broken scripts → Use grep to find ALL references before updating

**Testing too late**: Move everything then test → Hard to debug → Test after each major category of moves

**Losing git history**: Use `mv` instead of `git mv` → Breaks git history → Always use `git mv` for files in git

## Performance Notes
**Typical time**: Small reorganization (1-2 dirs, 30-60 min), moderate reorganization (5-10 dirs, 2-4h), major reorganization (entire project, 4-8h)
**What takes time**: Planning structure (15%), moving files (20%), updating references (40%), testing (20%), documentation (5%)

## What You'll Get
Organized directory structure (following project patterns), updated code references (all paths corrected), updated documentation (reflects new structure), validated functionality (tests pass), preserved git history (using git mv), migration documentation (for future reference)

## Next Steps
Review new structure (ensure meets needs), run full pipeline (end-to-end validation), update team (if collaborative project), monitor for issues (fix any missed references), document patterns (update CODING_PATTERNS.md if new patterns established)

## Related Skills
`/refactor-for-pattern` (apply patterns after reorganization), `/consolidate-scripts` (merge scripts during reorganization), `/update-docs` (update documentation systematically), `/run-tests` (validate reorganization doesn't break tests)
