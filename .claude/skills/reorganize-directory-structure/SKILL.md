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

## Overview

Systematically restructure project directories following best practices and established patterns. Ensures consistency, maintainability, and scalability as the project grows.

## Prerequisites

**Required**:
- Full project codebase access
- Git version control (for safety/rollback)
- Clean working directory (no uncommitted changes)

**Recommended**:
- Backup or git commit before starting
- Full test suite to validate paths after changes
- Documentation of current structure

## When to Use This Skill

- User says: "Organize the directories" or "Clean up the file structure"
- User says: "Restructure the data directories"
- Directory structure has grown organically without plan
- Year-specific data mixed with generic data
- Scripts scattered across multiple locations
- Outputs not consistently organized
- After major feature additions that changed organization needs

## Target Directory Patterns

### 1. Year-Specific Data

**Pattern**: `data/{type}/{year}/`

**Structure**:
```
data/
├── tracts/
│   ├── 2000/
│   │   ├── alabama_tracts_2000.parquet
│   │   └── ...
│   ├── 2010/
│   │   └── ...
│   └── 2020/
│       └── ...
├── adjacency/
│   ├── 2000/
│   ├── 2010/
│   └── 2020/
├── election/
│   └── 2020/  # Only 2020 has presidential data
└── demographics/
    ├── 2000/
    ├── 2010/
    └── 2020/
```

**Benefits**:
- Clear year separation
- Easy to add new census years
- Supports multi-year comparisons
- Reduces path complexity

### 2. Analysis Outputs

**Pattern**: `outputs/{year}_{version}/{analysis_type}/`

**Structure**:
```
outputs/
└── us_2020_v1/
    ├── states/
    │   ├── alabama/
    │   │   ├── data/
    │   │   │   ├── districts.csv
    │   │   │   ├── compactness.csv
    │   │   │   └── ...
    │   │   └── maps/
    │   │       ├── districts.png
    │   │       └── ...
    │   └── ...
    ├── national/
    │   ├── maps/
    │   └── data/
    ├── metro/
    └── index.html  # Dashboard
```

**Benefits**:
- Versioned outputs (can compare v1 vs v2)
- Year-tagged (can compare 2010 vs 2020)
- Hierarchical (state → national → metro)

### 3. Scripts Organization

**Pattern**: `scripts/{function}/{script}.py`

**Structure**:
```
scripts/
├── pipeline/          # Orchestration
│   ├── run_complete_redistricting.py
│   └── process_single_state.py
├── data/             # Data preparation
│   ├── download_census.py
│   └── build_adjacency.py
├── political/        # Political analysis
│   └── analyze_districts.py
├── demographic/      # Demographic analysis
│   └── analyze_demographics.py
├── compactness/      # Compactness metrics
│   └── analyze_compactness.py
├── visualization/    # Visualization
│   ├── visualize_state.py
│   └── visualize_rounds.py
└── validation/       # Validation
    └── validate_pipeline_outputs.py
```

**Benefits**:
- Function-based grouping
- Easy to find related scripts
- Clear responsibility boundaries

### 4. Papers & Presentations

**Pattern**: `papers/{paper_name}/` and `presentations/{presentation_name}/`

**Structure**:
```
papers/
├── 01_recursive_bisection/
│   ├── recursive_bisection.tex
│   ├── references.bib
│   ├── sections/
│   ├── figures/
│   └── compile.bat
└── ...

presentations/
├── edge_weighted_bisection/
│   ├── presentation.tex
│   ├── figures/
│   └── compile.bat
└── ...
```

**Benefits**:
- Self-contained papers/presentations
- Version control friendly
- Easy to archive or share

## Workflow

### Step 1: Analyze Current Structure

**Document current state**:
```bash
# Generate directory tree
tree -d -L 3 > ../../context/archive/directory_structure_before.txt

# Count files per directory
find . -type f | cut -d/ -f1-3 | sort | uniq -c | sort -rn > ../../context/archive/file_counts.txt

# Identify duplicates or inconsistencies
find . -name "*.py" | grep -E "(old|backup|temp|copy)" > ../../context/archive/cleanup_candidates.txt
```

**Identify issues**:
- Mixed year-specific and generic paths
- Scattered scripts
- Inconsistent output organization
- Duplicate or redundant files
- Deep nesting (>4 levels)

**Example findings**:
```
Issues identified:
1. data/tracts/ has flat structure (all years mixed)
2. outputs/ has inconsistent naming (us_2020_v1 vs us_2010_test)
3. Scripts in both scripts/ and src/apportionment/
4. Old backup files (*.old, *.backup) present
5. Temporary directories (temp/, old/, backup/) present
```

### Step 2: Design New Structure

**Create proposal document**:
```markdown
# Directory Reorganization Proposal

## Current Issues
1. Year-specific data not separated
2. Scripts scattered across locations
3. Outputs inconsistently named

## Proposed Structure
[Include tree diagram of new structure]

## Migration Plan
- Phase 1: Create new directories
- Phase 2: Copy files (preserve originals)
- Phase 3: Update path references
- Phase 4: Validate all paths
- Phase 5: Delete old structure

## Affected Files
- 47 Python scripts with hardcoded paths
- 12 batch files with paths
- 5 LaTeX documents with figure paths

## Risk Assessment
- Low risk: All changes can be rolled back via git
- Testing: Run full pipeline after Phase 3
```

**Get user approval**:
- Present proposal
- Discuss any concerns
- Confirm timeline
- Agree on rollback plan

### Step 3: Phase 1 - Create New Directories

**Create target structure**:
```bash
# Data directories
mkdir -p data/tracts/{2000,2010,2020}
mkdir -p data/adjacency/{2000,2010,2020}
mkdir -p data/election/2020
mkdir -p data/demographics/{2000,2010,2020}

# Output directories (examples)
mkdir -p outputs/us_2020_v1/{states,national,metro}
mkdir -p outputs/us_2010_v1/{states,national,metro}

# Scripts (already organized, but verify)
ls scripts/  # Check structure
```

**Use TodoWrite to track**:
```python
todos = [
    {"content": "Create new data directories", "status": "completed"},
    {"content": "Create new output directories", "status": "completed"},
    {"content": "Verify script organization", "status": "completed"},
]
```

### Step 4: Phase 2 - Copy Files

**Copy files to new locations** (preserve originals):
```bash
# Copy tract data
cp data/tracts/alabama_tracts_2020.parquet data/tracts/2020/
cp data/tracts/alabama_tracts_2010.parquet data/tracts/2010/

# Or use script
python scripts/migration/migrate_tract_data.py --dry-run
python scripts/migration/migrate_tract_data.py --execute
```

**Verify copies**:
```bash
# Check file counts
find data/tracts/2020 -name "*.parquet" | wc -l
# Should match old structure

# Verify file integrity
md5sum data/tracts/alabama_tracts_2020.parquet
md5sum data/tracts/2020/alabama_tracts_2020.parquet
# Should match
```

**Safety notes**:
- Never move files, always copy first
- Verify copies before modifying anything
- Keep originals until fully validated

### Step 5: Phase 3 - Update Path References

**Find all hardcoded paths**:
```bash
# Find Python files with old paths
grep -r "data/tracts/[a-z]" scripts/ --include="*.py" > path_updates_needed.txt

# Find batch files with old paths
grep -r "data\\tracts\\" . --include="*.bat" >> path_updates_needed.txt

# Find LaTeX files with old paths
grep -r "figures/" papers/ --include="*.tex" >> path_updates_needed.txt
```

**Update paths systematically**:

**Option A: Manual (safer for critical files)**:
```python
# Example: Update single file
# From:
tracts_file = f'data/tracts/{state}_tracts_2020.parquet'

# To:
tracts_file = f'data/tracts/2020/{state}_tracts_2020.parquet'
```

**Option B: Batch (for many similar changes)**:
```bash
# Use sed (be very careful!)
find scripts/ -name "*.py" -exec sed -i "s|data/tracts/\([a-z_]*\)_tracts_\(20[0-9][0-9]\)|data/tracts/\2/\1_tracts_\2|g" {} +

# Always test on a copy first!
```

**Track changes**:
```python
# Use TodoWrite to track each file updated
todos.append({"content": "Update paths in run_complete_redistricting.py", "status": "completed"})
todos.append({"content": "Update paths in download_census.py", "status": "completed"})
# etc.
```

**Common path updates**:
```python
# Data paths
OLD: f'data/tracts/{state}_tracts_{year}.parquet'
NEW: f'data/tracts/{year}/{state}_tracts_{year}.parquet'

OLD: f'data/adjacency/{state}_adjacency_{year}.pkl'
NEW: f'data/adjacency/{year}/{state}_adjacency_{year}.pkl'

# Output paths (already correct, verify)
CURRENT: f'outputs/us_{year}_{version}/states/{state}/data/'
```

### Step 6: Phase 4 - Test All Paths

**Run validation script**:
```python
# scripts/validation/validate_paths.py
import sys
from pathlib import Path

errors = []

# Check all expected data files exist
states = ['alabama', 'alaska', ...]  # All 50 states
for state in states:
    for year in [2000, 2010, 2020]:
        tract_file = Path(f'data/tracts/{year}/{state}_tracts_{year}.parquet')
        if not tract_file.exists():
            errors.append(f"Missing: {tract_file}")

if errors:
    print(f"Found {len(errors)} missing files:")
    for err in errors:
        print(f"  {err}")
    sys.exit(1)
else:
    print("[OK] All paths validated")
```

**Run test pipeline**:
```bash
# Test with small states
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version test_paths \
  --states "VT,DE" \
  --print-only

# If print-only succeeds, run for real
python scripts/pipeline/run_complete_redistricting.py \
  --year 2020 --version test_paths \
  --states "VT,DE"
```

**Check outputs**:
```bash
# Verify outputs created in correct location
ls outputs/us_2020_test_paths/states/vermont/data/
ls outputs/us_2020_test_paths/states/delaware/data/

# Run full validation
python scripts/validation/validate_pipeline_outputs.py \
  --year 2020 --version test_paths
```

### Step 7: Phase 5 - Clean Up Old Structure

**Only after full validation**:

**Create backup**:
```bash
# Archive old structure
tar -czf old_structure_backup_$(date +%Y%m%d).tar.gz \
  data/tracts/*.parquet \
  data/adjacency/*.pkl

# Move to archive
mv old_structure_backup_*.tar.gz ../../context/archive/
```

**Remove old files**:
```bash
# Remove old tract files (NEW structure has them)
rm data/tracts/*.parquet

# Remove old adjacency files
rm data/adjacency/*.pkl

# Remove temporary/backup files
rm -rf temp/
rm -rf backup/
find . -name "*.old" -delete
find . -name "*.backup" -delete
```

**Verify clean state**:
```bash
# Should only have year-organized directories
ls data/tracts/
# Should show: 2000/ 2010/ 2020/

ls data/adjacency/
# Should show: 2000/ 2010/ 2020/
```

### Step 8: Update Documentation

**Update CHANGELOG.md**:
```markdown
## [Unreleased]

### Changed
- Reorganized directory structure for year-specific data
  - Moved `data/tracts/*.parquet` → `data/tracts/{year}/*.parquet`
  - Moved `data/adjacency/*.pkl` → `data/adjacency/{year}/*.pkl`
- Updated all path references in scripts (47 files)
- Removed temporary and backup directories

### Migration
- Existing data: Run `scripts/migration/migrate_data.py` to move files
- Scripts: All paths automatically updated
- Outputs: New structure used going forward
```

**Update DATA_FORMATS.md**:
```markdown
## Directory Structure

### Data Files
\`\`\`
data/
├── tracts/{year}/          # Census tract geometries and population
│   ├── {state}_tracts_{year}.parquet
│   └── ...
├── adjacency/{year}/       # Adjacency graphs
│   ├── {state}_adjacency_{year}.pkl
│   └── ...
...
\`\`\`
```

**Update ARCHITECTURE.md**:
- Update directory structure diagrams
- Update data flow descriptions
- Update code examples with new paths

**Update CODING_PATTERNS.md**:
- Add pattern for year-specific paths
- Document path construction patterns
- Update examples

### Step 9: Commit Changes

**Create comprehensive commit**:
```bash
git add .
git status  # Review all changes

git commit -m "Reorganize directory structure to year-specific pattern

- Move tract data to data/tracts/{year}/
- Move adjacency graphs to data/adjacency/{year}/
- Update all path references in scripts (47 files)
- Update batch files with new paths
- Remove old temporary/backup directories
- Update documentation (CHANGELOG, DATA_FORMATS, ARCHITECTURE)

Tested: Full pipeline run for VT, DE with new structure
Validated: All 50 states have data in new locations

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

## Common Reorganization Scenarios

### Scenario 1: Year-Specific Data Separation

**Problem**: All census years mixed in flat structure

**Solution**: Create year subdirectories

**Migration**:
```python
# scripts/migration/separate_by_year.py
from pathlib import Path
import shutil

source_dir = Path('data/tracts')
years = [2000, 2010, 2020]

for year in years:
    target_dir = source_dir / str(year)
    target_dir.mkdir(exist_ok=True)

    # Move year-specific files
    for file in source_dir.glob(f'*_{year}.parquet'):
        target = target_dir / file.name
        shutil.copy2(file, target)
        print(f"Copied: {file} → {target}")
```

### Scenario 2: Consolidate Scattered Scripts

**Problem**: Scripts in multiple locations (scripts/, src/, root/)

**Solution**: Consolidate to scripts/{function}/

**Migration**:
1. Identify all script locations
2. Group by function
3. Move to appropriate subdirectory
4. Update imports and paths
5. Test each script

### Scenario 3: Flatten Deep Nesting

**Problem**: Paths like `data/processed/census/2020/tracts/geometries/`

**Solution**: Simplify to `data/tracts/2020/`

**Benefits**:
- Shorter paths
- Less cognitive overhead
- Easier to type/remember

## Safety Protocols

**Always**:
1. ✅ Commit clean working directory first
2. ✅ Create backup before starting
3. ✅ Copy files (never move) in early phases
4. ✅ Validate thoroughly before deleting originals
5. ✅ Test pipeline after path updates
6. ✅ Document all changes
7. ✅ Can rollback via git

**Never**:
1. ❌ Delete files before validation
2. ❌ Batch update paths without testing
3. ❌ Skip documentation updates
4. ❌ Reorganize during active development
5. ❌ Mix reorganization with feature work

## Troubleshooting

### Broken Imports After Reorganization

**Symptom**: `ModuleNotFoundError` or `ImportError`

**Cause**: Python imports depend on directory structure

**Solution**:
- Keep source code structure (`src/apportionment/`) unchanged
- Only reorganize data/ and outputs/
- If moving scripts, update imports

### Tests Fail After Path Updates

**Symptom**: Tests can't find fixture files

**Cause**: Test fixtures have hardcoded paths

**Solution**:
```python
# Update test fixtures
# From:
fixture_path = 'tests/fixtures/sample_tracts.parquet'

# To:
import os
fixture_path = os.path.join(os.path.dirname(__file__), 'fixtures', 'sample_tracts.parquet')
```

### Git Recognizes as New Files (Not Moves)

**Symptom**: Git shows file deletions + additions instead of renames

**Cause**: Using copy instead of `git mv`

**Solution**: Expected behavior when using copy-validate-delete approach
- Git will track content similarity
- Use `git log --follow` to see file history
- Worth it for safety

### Old Paths Still Work (Shadowing)

**Symptom**: Code runs but uses wrong files

**Cause**: Old files still present, new paths not actually used

**Solution**:
- Verify new paths in code
- Check which files were accessed: `strace -e open python script.py 2>&1 | grep parquet`
- Delete old files once validated

## Related Skills

- `/enhancement-implement` - Implement as formal enhancement
- `/update-docs` - Update all documentation after reorganization
- `/consolidate-scripts` - Consolidate scripts after reorganization
- `/refactor-for-pattern` - Refactor code to use new patterns

## Best Practices

1. **Plan first, execute second**: Design full structure before moving anything
2. **Incremental validation**: Test after each phase
3. **Document decisions**: Explain why structure chosen
4. **Communicate changes**: Notify team of upcoming reorganization
5. **Version control**: Commit each phase separately for easier rollback
6. **Keep backups**: Archive old structure even after deletion
7. **Update CI/CD**: Ensure automation uses new paths

## Next Steps

After reorganization:
- Run full test suite
- Update CI/CD pipelines
- Notify collaborators of changes
- Archive old structure documentation
- Monitor for any path-related issues
- Consider adding path validation to CI
