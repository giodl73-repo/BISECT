---
wave_uuid: 2ddc27
slug: wave-phase-validation-fix
uuid: 7f2f5c
---
# E55 (Phase 2): Wave Phase Validation and Title Standardization

**Status**: ✅ COMPLETED
**Completed**: 2026-01-25
**Wave**: Wave 8 (WAVE-MANAGER-IMPROVEMENTS)
**Priority**: Medium
**Estimated Complexity**: Small

---

## Implementation Summary

**Completion Date**: 2026-01-25

**What Was Built**:
- Fixed validator path resolution to use absolute paths from BASE_DIR
- Added support for searching both `context/enhancements/` and `context/enhancements/active/` directories
- Standardized E55 title to include "(Phase 2)" format
- Validator now finds all 47 existing enhancement files

**Files Modified**:
- `tools/wave-manager/validate_phases.py` - Fixed path resolution and added active/ directory search
- `context/enhancements/active/55_wave_phase_validation_fix.md` - Added phase number to title

**Validation Results**:
- ✅ All 47 existing enhancements found
- ✅ All wave phase mappings validated
- ✅ No path resolution errors
- ⚠️ 6 enhancements (56-61) not yet created (expected - future enhancements)

**Decision Made**:
- **Title Format**: Made optional - enhancements can include "(Phase X)" if helpful, but it's not required
- **Wave files remain source of truth** for phase assignments
- **E55 uses title format** as example: `# E55 (Phase 2): Title`

---

## Description

Fix wave phase validation script to find all enhancement files and optionally standardize enhancement titles to include phase numbers for better organization.

---

## Implementation

### Tasks

1. **Fix Validator to Find All Enhancement Files** (10 min)
   - Update `tools/wave-manager/validate_phases.py` to check both directories:
     - `context/enhancements/*.md`
     - `context/enhancements/active/*.md`
   - Fix relative path resolution

2. **Decide on Title Standardization** (5 min)
   - Option A: Keep current format (cleaner, phase info in wave files)
   - Option B: Add phase numbers to titles (more explicit, matches appmanager)
   - Get user preference

3. **If Adding Phase Numbers to Titles** (20 min)
   - Create apportionment-specific `fix_phase_titles.py`
   - Generate FIXES dictionary from wave phase mappings
   - Apply fixes to all 47 enhancements
   - Format: `# Enhancement X (Phase Y): Title`

4. **Validation** (5 min)
   - Run `validate_phases.py` to verify all issues resolved
   - Check that all enhancement files are found
   - Verify phase assignments match wave documents

### File Changes

**Files to Modify**:
- `tools/wave-manager/validate_phases.py` - Fix file discovery

**Files to Create** (if adding phase numbers):
- `tools/wave-manager/fix_phase_titles_apportionment.py` - Auto-generate fixes from waves

---

## Current State

**Validation Issues**:
- 13 enhancement files not found (in `active/` subdirectory)
- 33 enhancements without phase numbers in titles (warnings)

**Enhancement Locations**:
- `context/enhancements/` - 41 files (1-39, 8, 41-46, 49, 51)
- `context/enhancements/active/` - 6 files (47, 48, 50, 52, 53, 54)

---

## Success Criteria

- [x] Validator finds all 47 enhancement files
- [x] No "File not found" errors for existing enhancements
- [x] Decision made on title format (optional, not required)
- [x] E55 uses example title format with phase number
- [x] Validation passes cleanly (only expected errors for uncreat ed enhancements 56-61)

---

## Dependencies

**Prerequisites**:
- E53 (Wave Manager Schema v2.0) ✅
- E54 (Wave Skills Integration) ✅
- All wave files have explicit phase mappings ✅

**Blocking Issues**: None

---

## Related Files

**Validation Scripts**:
- `tools/wave-manager/validate_phases.py` - Phase validation script
- `tools/wave-manager/parser.py` - Wave file parser

**Enhancement Files**:
- `context/enhancements/*.md` - 41 enhancement files
- `context/enhancements/active/*.md` - 6 enhancement files

**Wave Files**:
- `context/waves/WAVE*.md` - 8 wave files with phase mappings

---

## Title Format Options

### Option A: Current Format (Recommended)
```markdown
# E1: Integrate Compactness into Main Pipeline
```

**Pros**:
- Cleaner, more concise
- Phase info available in wave files
- Enhancement number is primary identifier

**Cons**:
- Phase number not immediately visible in enhancement file

### Option B: Include Phase Number
```markdown
# E1 (Phase 1): Integrate Compactness into Main Pipeline
```

**Pros**:
- Phase immediately visible in file
- Matches appmanager convention
- Better for manual organization

**Cons**:
- More verbose
- Redundant with wave file info
- Requires updating 47 files

---

## Notes

- Validator currently only checks `ENHANCEMENTS_DIR` path, not subdirectories
- Wave files already have complete phase mappings (Schema v2.0)
- Adding phase numbers to titles is optional - wave files are source of truth
- Most projects don't include phase in enhancement titles

---

**E55 Summary**: Fix wave phase validator to find all enhancement files and optionally standardize enhancement titles with phase numbers.
