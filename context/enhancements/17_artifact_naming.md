# Enhancement 17: Standardize Artifact Naming Conventions

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026

### Goal

Create clean, consistent naming conventions for all pipeline artifacts (maps, CSVs, analysis outputs) across state and national levels, removing year suffixes where appropriate and organizing artifacts in logical top-level directories.

### Status

**COMPLETED** - January 14, 2026

### Problem

The pipeline had inconsistent naming conventions that created confusion and validation false negatives:

| Issue | Old Behavior | New Behavior |
|-------|--------------|--------------|
| **Year in filenames** | `polsby_popper_districts_2020.png` | `polsby_popper.png` (year in directory) |
| **National map naming** | `US_National_Map_435_Districts_2020.png` (PascalCase + year) | `us_all_districts.png` (snake_case, no year) |
| **File organization** | CSVs and maps mixed in root directory | Organized in `data/` and `maps/` subdirectories |
| **Round maps** | `round_1_2_regions.png` (no padding, includes count) | `round_01.png` (zero-padded, clean) |
| **District maps** | `district_01_los_angeles.png` (city slug) | `district_01.png` (no city slug) |
| **Analysis paths** | `political_analysis/`, `demographic_analysis/` | `political/`, `demographic/` (shorter) |

### Solution

Implemented comprehensive naming standardization:

**Naming Rules:**
1. **No year suffixes** - Year is in directory path `us_{year}_{version}/`
2. **Snake_case only** - All lowercase with underscores (no PascalCase)
3. **Zero-padded numbers** - `round_01.png`, `district_01.png` (consistent 2-digit padding)
4. **Organized by type** - `data/` for CSVs, `maps/` for visualizations
5. **Consistent prefixes** - State: no prefix, National: `us_` prefix for CSVs
6. **Descriptive names** - `all_districts.png` not `california_52_districts.png`

**State-Level Structure:**
```
outputs/us_{year}_{version}/states/{state_name}/
├── data/                    # All CSV/pickle files
│   ├── final_assignments.pkl
│   ├── district_summary.csv
│   ├── district_cities.csv
│   └── rounds_hierarchy.csv
├── maps/                    # All visualizations
│   ├── all_districts.png
│   ├── all_districts_with_cities.png
│   ├── rounds/round_01.png
│   ├── districts/district_01.png
│   └── metros/los_angeles.png
├── political/               # Political analysis
│   ├── district_political.csv
│   └── maps/partisan_lean.png
├── demographic/             # Demographics
│   ├── district_demographics.csv
│   └── maps/majority_race.png
└── compactness/             # Compactness
    └── maps/polsby_popper.png
```

**National-Level Structure:**
```
outputs/us_{year}_{version}/
├── data/                         # Aggregated data
│   ├── us_all_districts.csv
│   ├── us_district_summary.csv
│   └── us_rounds_hierarchy.csv
├── maps/                         # National maps
│   ├── us_all_districts.png
│   ├── rounds/round_01.png
│   ├── political/partisan_lean.png
│   ├── demographic/majority_demographics.png
│   └── compactness/polsby_popper.png
└── index.html
```

### Implementation

**Scripts Updated (19 files):**

**Core Pipeline (8 files):**
1. `run_state_redistricting.py` - State maps to `maps/`, data to `data/`
2. `add_cities_to_districts.py` - Cities CSV/maps paths
3. `create_individual_district_maps.py` - Removed city slug from filenames
4. `visualize_all_rounds.py` - Zero-padded, removed region count
5. `create_final_district_summary.py` - CSVs to `data/` subdirectory
6. `create_us_national_map.py` - National maps to `maps/`
7. `create_us_national_rounds_progression.py` - Zero-padded rounds
8. `create_us_aggregate.py` + `create_us_rounds_hierarchy.py` - CSVs to `data/`

**Analysis Scripts (8 files):**
9. `analyze_districts.py` - Political CSVs (no year suffix)
10. `visualize_partisan_lean.py` - Political maps to `political/maps/`
11. `create_us_national_political_map.py` - National political map
12. `analyze_district_demographics.py` - Demographic directory
13. `visualize_district_demographics.py` - Demographic maps
14. `create_us_national_demographic_map.py` - National demographic map
15. `visualize_compactness.py` - Compactness maps (no year suffix)
16. `calculate_compactness_metrics.py` - Compactness CSV

**Supporting Files (3 files):**
17. `validate_pipeline_outputs.py` - Updated PIPELINE_OUTPUTS dictionary
18. `web/dashboard.html` - Updated all JavaScript path construction
19. `scripts/web/generate_dashboard.py` - Path handling (if needed)

**Documentation (2 files):**
20. `CODING_PATTERNS.md` - Updated Section 9 (File Naming Conventions)
21. `DATA_FORMATS.md` - Added comprehensive directory structure examples

### Benefits

1. **Consistency**: Parallel naming between state and national artifacts
2. **Clarity**: Year redundancy eliminated (already in directory path)
3. **Organization**: Files grouped in logical subdirectories by type
4. **Maintainability**: Single source of truth for naming conventions
5. **Discoverability**: Predictable file locations
6. **Documentation Alignment**: Implementation matches documented conventions
7. **Validation Accuracy**: No false negatives from naming mismatches
8. **Dashboard Reliability**: No broken image links from path changes

### Migration Strategy

Used **Clean Regeneration** approach:
- Update all scripts with new naming conventions
- Re-run pipelines with new structure (2020, 2010, 2000)
- No legacy compatibility layer needed
- Ensures complete consistency across all outputs

### Testing

**Unit Testing**: Each modified script tested individually
**Integration Testing**: Full pipeline run for single state (California 2010)
**Validation Testing**: `validate_pipeline_outputs.py` reports 100% completion
**Dashboard Testing**: All images load correctly, no broken links
**Full Pipeline Testing**: All three census years (2020, 2010, 2000)

### Success Criteria

- ✅ All artifacts follow consistent naming convention
- ✅ No year suffixes in filenames
- ✅ National maps organized in `maps/` subdirectory
- ✅ State and national naming patterns are parallel
- ✅ CSVs organized in `data/` subdirectory
- ✅ Analysis outputs organized by type (political/, demographic/, compactness/)
- ✅ Validation script reports 100% completion
- ✅ Dashboard loads all images correctly
- ✅ Documentation matches implementation
- ✅ Works for all census years (2000, 2010, 2020)

### Estimated Effort

**Actual Time**: ~3-4 hours
- Script Updates: 16 scripts × 5-10 min = 2 hours
- Validation Script: 30 minutes
- Dashboard Updates: 45 minutes
- Documentation: 30 minutes
- Summary/notes: 15 minutes

**Total**: 3-4 hours (as estimated)

### Priority

**CRITICAL** - Core infrastructure improvement:
- Fixes validation false negatives
- Improves maintainability
- Establishes foundation for future enhancements
- Eliminates confusion from inconsistent naming

### Related Enhancements

- **Enhancement 14**: Pipeline Output Validation Framework (validation updates needed)
- **Enhancement 15**: Fix 2010/2000 Pipeline Completeness (re-run with new naming)
- **Enhancement 13**: Unify Directory Structure (this completes that work)

---

**Date Added**: January 14, 2026
**Date Completed**: January 14, 2026
**Status**: COMPLETED
**Commits**: [2eb7ed7](https://github.com/giodl_microsoft/redistricting/commit/2eb7ed768a8bcd8b4ad72421a4b50255a1c6bcd6), [9e35aaf](https://github.com/giodl_microsoft/redistricting/commit/9e35aafaecb39078640165d0757dccc2723ff558), [54bf030](https://github.com/giodl_microsoft/redistricting/commit/54bf030c615359f04df80c6601a033b9b53b6dbc), [5ab531a](https://github.com/giodl_microsoft/redistricting/commit/5ab531ad0fc676b9276fef17638b4bf96aa301e5), [67ed998](https://github.com/giodl_microsoft/redistricting/commit/67ed99896b0a67f1346d225b4d8150873e2fa1b8), [cf3b655](https://github.com/giodl_microsoft/redistricting/commit/cf3b65576b7b5931efbf20675b06401ca26da22f), [22a9f53](https://github.com/giodl_microsoft/redistricting/commit/22a9f532c759350f2f8903a501c15f4fa5fd8171)
**Size**: L - 3,141 lines changed (44 files)
**Actual Implementation Time**: 3-4 hours

### Built-in Validation

Added automatic validation at two pipeline levels:

**State-Level** (`run_state_redistricting.py`):
- Validates outputs immediately after processing each state
- Only prints warnings if files are missing
- Prints success message in standalone mode
- Respects progress bar protocol (quiet in parallel mode)

**National-Level** (`run_complete_redistricting.py`):
- Already integrated - validates all outputs at end of pipeline
- Comprehensive check of all 50 states + national aggregations
- Generates detailed validation report
- Returns non-zero exit code if outputs missing

**Benefits:**
- Immediate feedback during pipeline execution
- Catches missing outputs before wasting time on later stages
- No separate validation step required
- Fail-fast option for automated workflows

---
