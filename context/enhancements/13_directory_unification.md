# Enhancement 13: Unify Directory Structure Across Census Years

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: [1944a24](https://github.com/giodl_microsoft/redistricting/commit/1944a24d3afa09e55a04870628647bdf13c98e16)
**Size**: XS - 12 lines changed (1 files)

**Completion Date:** January 14, 2026

### Priority
**LOW-MEDIUM** - Code maintainability and simplification

### Motivation
Currently, 2020 data uses flat structure (`data/raw/`) while 2010/2000 use subdirectories (`data/tracts/{year}/`, `data/adjacency/{year}/`). This inconsistency requires conditional path logic in ~15+ scripts throughout the codebase. Unifying the structure would eliminate this complexity and make the codebase more maintainable.

### Current State
- **2020 Structure**: `data/raw/{state}_tracts_2020.parquet`
- **2010/2000 Structure**: `data/tracts/{year}/{state}_tracts_{year}.parquet`
- Every script that loads tract/adjacency/places data needs year-specific path logic
- ~15-20 scripts affected across pipeline, analysis, and visualization

### Implementation Plan

#### Phase 1: Move 2020 Data (30 minutes)
1. Create new directory structure:
   ```
   data/tracts/2020/
   data/adjacency/2020/
   ```
2. Move all 2020 tract/places files from `data/raw/` to `data/tracts/2020/`
3. Move all 2020 adjacency files from `data/adjacency/` to `data/adjacency/2020/`
4. Keep `data/raw/` for truly raw downloaded data only

#### Phase 2: Update Scripts (1 hour)
Remove all conditional path logic and use uniform pattern:
```python
# Before (conditional):
if args.year == '2020':
    tracts_file = f'data/raw/{state}_tracts_{year}.parquet'
else:
    tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'

# After (unified):
tracts_file = f'data/tracts/{year}/{state}_tracts_{year}.parquet'
```

**Scripts to Update** (~15-20 files):
- `scripts/pipeline/` - 10 scripts
- `scripts/political/` - 3 scripts
- `scripts/demographic/` - 2 scripts
- `scripts/compactness/` - 3 scripts
- `scripts/data/geography/` - 1 script
- Other visualization/analysis scripts as needed

#### Phase 3: Update Documentation (15 minutes)
- Update `DATA_FORMATS.md` with new structure
- Update `ARCHITECTURE.md` data flow diagrams
- Update `CLAUDE.md` and `README.md` if needed
- Update `.gitignore` patterns if needed

### Benefits
- **Simplified Code**: Remove ~50-100 lines of conditional path logic
- **Consistency**: All census years follow same pattern
- **Maintainability**: Future census years (2030+) just drop in
- **Clarity**: Logical separation of processed data by year
- **Less Error-Prone**: No more forgetting to add year conditionals

### Risks
- **Minimal**: Just file moves + find/replace path updates
- **Testing**: Verify one full 2020 run after changes
- **Rollback**: Git commit before changes, easy to revert

### Testing Plan
1. Move 2020 files to new structure
2. Update all path logic in scripts
3. Run full 2020 pipeline: `run_redistricting.bat --year 2020 --version test`
4. Verify outputs identical to previous run
5. Run spot checks for 2010/2000 to ensure still working

### Timeline
**Estimated**: 2-3 hours total
- 30 min: Move files
- 1 hour: Update scripts
- 15 min: Update documentation
- 30 min: Testing and verification

### Dependencies
- None (independent enhancement)
- Can be done anytime between census runs
- Ideally after current 2000 run completes

### Future-Proofing
Once implemented, adding 2030 Census data will follow same pattern with zero code changes needed for path logic.

### Implementation Summary

**Status:** Completed January 14, 2026 (~2 hours total)

**Phase 1: File Movement (30 minutes)**
- Created `data/tracts/2020/` and `data/adjacency/2020/` directories
- Moved 50 tract files from `data/raw/` to `data/tracts/2020/`
- Moved 50 places files from `data/raw/` to `data/tracts/2020/`
- Moved 50 adjacency files from `data/adjacency/` to `data/adjacency/2020/`
- Verified file naming consistency across all three census years
- **Result:** All 150 data files (50 states × 3 years) now use uniform structure

**Phase 2: Script Updates (60 minutes)**
- Updated 19 scripts to remove year-specific path conditionals
- **Pipeline Scripts (7):** run_complete_redistricting.py, create_final_district_summary.py, add_cities_to_districts.py, create_individual_district_maps.py, visualize_all_rounds.py, create_us_national_map.py, create_us_national_rounds_progression.py
- **Geography Scripts (4):** build_all_adjacency_graphs.py (4 conditionals removed), check_graph_connectivity.py, check_isolated_tracts.py, diagnose_components.py
- **Political Scripts (3):** analyze_districts.py, visualize_partisan_lean.py, create_us_national_political_map.py
- **Demographic Scripts (3):** analyze_district_demographics.py, visualize_district_demographics.py, create_us_national_demographic_map.py
- **Compactness Scripts (2):** visualize_compactness.py, create_us_national_compactness_map.py
- **Pattern Changed:** Removed all `if args.year == '2020': ... else: ...` conditionals for file paths
- **Pattern Preserved:** Config imports remain conditional (intentionally)
- **Result:** Simplified ~80 lines of conditional path logic

**Phase 3: Documentation Updates (15 minutes)**
- Updated `CODING_PATTERNS.md` (4 path references)
- Updated `DATA_FORMATS.md` (3 sections: directory structure, data sizes, adjacency graphs)
- Updated `ARCHITECTURE.md` (2 code examples)
- Updated `CLAUDE.md` (Enhanced Enhancement Workflow section with comprehensive guide)
- Created `UNIFICATION_STATUS.md` tracking document

**Phase 4: Testing (User Completed)**
- Tested 2020 adjacency rebuild
- Tested 2010 adjacency rebuild
- Tested 2000 adjacency rebuild
- All three census years verified working with new structure

**Benefits Achieved:**
- ✅ Simplified Code: Removed ~80 lines of conditional path logic
- ✅ Consistency: All census years follow same pattern
- ✅ Maintainability: Future census years (2030+) just drop in
- ✅ Clarity: Logical separation of processed data by year
- ✅ Less Error-Prone: No more forgetting to add year conditionals

**Key Learnings:**
- Manual editing safer for critical path changes (avoided batch operations)
- Status documents helpful for tracking multi-phase enhancements
- Preserving intentional conditionals while removing redundant ones is important
- Testing all census years essential after directory structure changes

---

**Date Added**: January 13, 2026
