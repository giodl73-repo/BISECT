---
wave_uuid: f507d9
slug: validation-framework
uuid: 91efb6
---
# E14: Pipeline Output Validation Framework

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium
**Created**: January 2026
**Completed**: January 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

### Goal
Create a validation script that checks for missing/incomplete outputs from the redistricting pipeline and reports which specific scripts failed to generate their expected outputs.

### Problem
When debugging pipeline runs (especially 2010 and 2000 census data), it was difficult to determine:
- Which specific outputs were missing
- Which scripts failed to generate those outputs
- How to systematically re-run failed components

Without a validation framework, debugging required manual inspection of directory trees and guesswork about which scripts to re-run.

### Solution
Built comprehensive validation framework that:
1. **Maps scripts to outputs**: Maintains `PIPELINE_OUTPUTS` dictionary mapping each pipeline script to its expected output files
2. **Validates per-state outputs**: Checks all 50 states for required and optional analysis outputs
3. **Validates national outputs**: Checks aggregation CSVs, national maps, and dashboard
4. **Dual reporting**: Brief console summary + detailed text report for debugging
5. **Actionable diagnostics**: Groups missing files by generating script with re-run commands

### Implementation

**New Files Created:**
- `scripts/validation/validate_pipeline_outputs.py` - Main validation script (903 lines)

**Files Modified:**
- `scripts/pipeline/run_complete_redistricting.py` - Added validation at end of pipeline
- `docs/ENHANCEMENTS_2026.md` - This documentation

### Script-to-Output Mapping

The validation script maintains a comprehensive mapping of pipeline scripts to their outputs:

**Per-State Core Outputs (Always Required):**
```python
"run_state_redistricting": {
    "outputs": [
        "final_assignments.pkl",
        "{state_name}_{num_districts}_districts.png",
        "{state_name}_{num_districts}_districts_with_cities.png",
        "rounds_hierarchy.csv"
    ],
    "scope": "per-state",
    "required": True
}

"visualize_all_rounds": {
    "outputs": ["maps/rounds/round_{round_num}_{round_regions}_regions.png"],
    "scope": "per-state",
    "required": True
}

"create_individual_district_maps": {
    "outputs": ["maps/districts/district_{district_num:02d}_{city_name}.png"],
    "scope": "per-state",
    "required": True
}
```

**Per-State Optional Outputs (if --run-analysis):**
- Political analysis (year==2020 only)
- Demographic analysis
- Compactness visualization
- Metro area maps

**National Outputs:**
- Aggregate CSVs (us_all_districts.csv, us_district_summary.csv)
- National maps (us_all_districts.png)
- National round progression (us_rounds/*.png)
- Dashboard (index.html)

### Usage Examples

**Test complete 2020 run:**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2020 --version v3
```

**Test incomplete 2010 run (with optional analysis):**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1 --check-optional
```

**Test single state:**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1 --state CA
```

**Force re-validation:**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1 --force
```

**Generate CSV report:**
```bash
python scripts/validation/validate_pipeline_outputs.py --year 2010 --version v1 --csv
```

### Output Format

**Console Summary (Always Displayed):**
```
============================================================
  Pipeline Validation Summary
  Run: outputs\us_2010_v1
============================================================
[OK] 0 states complete (0%)
[WARN] 43 states partially complete (86%)
[FAIL] 7 states with missing core outputs (14%)

National outputs: 3/6 present (50%)

Top script failures:
  - Add Cities To Districts (50 states, 50 files)
  - Create Individual District Maps (50 states, 50 files)
  - Run State Redistricting (43 states, 43 files)

Detailed report: outputs\us_2010_v1_validation.txt
```

**Detailed Report File (`us_{year}_{version}_validation.txt`):**
```
======================================================================
PIPELINE OUTPUT VALIDATION REPORT
======================================================================

Run: outputs\us_2010_v1
Generated: 2026-01-14 12:25:50
Year: 2010, Version: v1
Total Checks: 491
Passed: 472 (96.1%)
Failed: 19 (3.9%)

======================================================================
PER-STATE VALIDATION
======================================================================

States Checked: 50
  Fully Complete: 0 (0.0%)
  Partially Complete: 43 (86.0%)
  Missing Core Outputs: 7 (14.0%)

----------------------------------------------------------------------
CALIFORNIA (95.7% complete)
----------------------------------------------------------------------
Expected: 23 outputs
Found: 22
Missing: 1

Missing Files:
  [X] district_cities.csv
    Script: scripts/pipeline/add_cities_to_districts.py
    Condition: After final_assignments.pkl exists

======================================================================
SCRIPTS WITH FAILURES
======================================================================

scripts/pipeline/add_cities_to_districts.py:
  States Affected: 50 (all states)
  Files Missing: 50
  Condition: After final_assignments.pkl exists

======================================================================
RECOMMENDED ACTIONS
======================================================================

1. Re-run per-state scripts for affected states:
   python scripts/pipeline/add_cities_to_districts.py --year 2010 --version v1 --force

2. Re-run national post-processing:
   python scripts/web/generate_dashboard.py --year 2010 --version v1
```

### Special Cases Handled

**1. Single-District States**
States with 1 district (AK, DE, MT, ND, SD, VT, WY) don't need recursive bisection:
- Skips round map validation
- Skips rounds_hierarchy.csv check
- Only validates core district outputs

**2. District Map Filenames**
District maps include city names: `district_01_west_covina.png`
- Validation uses glob patterns to count files instead of predicting names
- Reports: `maps/districts/ (52/52 files)` ✓ or `(45/52 files)` ✗

**3. Inconsistent Filename Conventions**
Some files use underscores (`new_hampshire_2_districts.png`), others use spaces (`new hampshire_2_districts_with_cities.png`):
- Validation tries both patterns to handle pipeline inconsistency
- Documented as future cleanup (E15)

**4. Round Map Naming**
Round maps use format `round_{N}_{REGIONS}_regions.png`:
- round_1_2_regions.png
- round_2_4_regions.png
- round_6_52_regions.png (final round = exact district count, not 2^N)

### Integration with Pipeline

Validation automatically runs at the end of `run_complete_redistricting.py`:

```python
# After post-processing steps complete
if not args.print_only:
    print("\n" + "="*70)
    print("  Validating Pipeline Outputs")
    print("="*70)

    validation_result = subprocess.run([
        sys.executable,
        'scripts/validation/validate_pipeline_outputs.py',
        '--year', args.year,
        '--version', args.version,
        '--output-dir', str(output_dir)
    ])

    if validation_result.returncode != 0:
        print("\nWARNING: Some pipeline outputs are missing.")
        print(f"Review detailed report at: {output_dir.name}_validation.txt")
```

### Testing Results

**2020 Census (v3):**
- ✅ All 50 states: 100% complete (required outputs)
- ⚠️ National outputs: 3/6 present (missing rounds_hierarchy, us_all_districts.png, us_rounds/)

**2010 Census (v1):**
- ❌ 0 states complete (0%)
- ⚠️ 43 states partially complete (86%)
- ❌ 7 single-district states with missing core outputs (14%)
- **Major Issues Identified:**
  - All 50 states missing `district_cities.csv`
  - All 50 states missing individual district maps
  - 43 states missing `_with_cities.png` files

This validated the user's observation that 2010/2000 runs had missing outputs and provided exact diagnosis for fixing them (see E15).

### Benefits

1. **Quick Diagnosis**: Immediately identify which scripts are failing
2. **Actionable**: Provides exact commands to re-run failed scripts
3. **Completeness Tracking**: Shows % complete per state and overall
4. **Debugging Aid**: Essential for investigating missing maps in 2010/2000 runs
5. **Quality Assurance**: Can be run after pipeline to verify completeness
6. **Documentation**: Serves as canonical list of expected pipeline outputs

### Future Improvements

---
