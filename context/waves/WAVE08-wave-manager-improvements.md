# Wave 8: Wave Manager Improvements

**Date**: 2026-01-25
**Focus**: Upgrade wave manager with schema v2.0, fix bugs, and add project branding across all projects
**Status**: ✅ COMPLETED
**Started**: 2026-01-25
**Completed**: 2026-01-25
**Estimated Duration**: 1 day
**Actual Duration**: 1 day
**Enhancements**: 53, 54, 55, 56, 57, 58
**Phases**:
- Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
- Phase 2: Enhancements 54, 55 - Configuration (✅ COMPLETED 2026-01-25)
- Phase 3: Enhancements 56, 57 - UI Fixes (✅ COMPLETED 2026-01-25)
- Phase 4: Enhancement 58 - Phase Names (✅ COMPLETED 2026-01-25)

---

## Goals

1. Upgrade wave manager to schema v2.0 with phase mapping support
2. Reorganize all waves to reflect accurate chronological history
3. Configure wave management skills and validation
4. Add project-specific branding to wave manager (PROJECT_NAME and PROJECT_COLOR)
5. Fix phase display bug showing "No phases assigned to this wave"
6. Apply fixes to all projects (apportionment, appmanager, TCM, NHL, Performance)
7. Fix enhancement file organization (move from active/ subdirectory)
8. Add cache-busting meta tags to prevent stale JavaScript

---

## Success Metrics

| Metric | Target | Actual | Notes |
|--------|--------|--------|-------|
| Schema upgrade | v2.0 | ✅ | Phase mapping support added |
| Wave reorganization | 7 historical + 7 future | ✅ | WAVE01-07, WAVE-F2 through F7 |
| Wave skills | 3 skills | ✅ | /start-wave, /complete-enhancement, /complete-wave |
| Validation | All 57 enhancements | ✅ | Fixed to search flat directory |
| Branding endpoint | /api/config | ✅ | Returns PROJECT_NAME and PROJECT_COLOR |
| Title display | Project-specific | ✅ | Shows "Apportionment - Wave Manager" in blue |
| Phase display | Accurate counts | ✅ | Shows "2/2 phases completed" instead of "0/0" |
| Projects updated | 5 total | ✅ | apportionment, appmanager, TCM, NHL, Performance |
| Enhancement organization | Flat directory | ✅ | Moved from active/ to enhancements/ |
| Cache prevention | Meta tags | ✅ | No-cache headers prevent stale JS |

---

## Architecture Overview

**Backend Changes**:
- Added `/api/config` endpoint to serve PROJECT_NAME and PROJECT_COLOR
- Updated API to search flat enhancements/ directory (removed active/ subdirectory)

**Frontend Changes**:
- Added `loadConfig()` function to fetch and apply branding
- Added `linkPhasesToWaves()` function to populate wave.phases from phase_ids
- Added cache-busting meta tags to prevent browser caching
- Updated `init()` and `refreshData()` to call linking function

**File Organization**:
- Moved 10 enhancement files from active/ to enhancements/
- Updated INDEX.md to remove active/ references
- Updated validate_phases.py to remove active/ directory search

---

## Phases

### Phase 1: Enhancement 53 - Schema v2.0 & Wave Reorganization (Foundation)
**Status**: ✅ COMPLETED
**Completed**: 2026-01-25
**Effort**: 4 hours

Upgraded wave manager tool with schema v2.0 and reorganized all waves to reflect accurate chronological history.

**Completed**:
- ✅ Updated wave manager from appmanager with phase mapping support
- ✅ Reorganized 7 historical waves (WAVE01-07) chronologically
- ✅ Created 7 prioritized future waves (WAVE-F2 through F7)
- ✅ Fixed INDEX.md status discrepancies (Enh 47, 50)
- ✅ Validated all phase mappings
- ✅ Added GitHub integration support
- ✅ Updated schema documentation (SCHEMA.md)

**Files Modified**:
- `tools/wave-manager/parser.py` - Enhanced with phase mapping
- `tools/wave-manager/SCHEMA.md` - v2.0 schema documentation
- `tools/wave-manager/static/index.html` - Updated UI
- `tools/wave-manager/config.py` - Added GITHUB_REPO field
- All wave files (WAVE01-07, WAVE-F2 through F7)
- `context/enhancements/INDEX.md` - Fixed status discrepancies

**Enhancement**: [53](../enhancements/53_wave_manager_schema_v2.md)

### Phase 2: Enhancements 54, 55 - Skills Integration & Validation (Configuration)
**Status**: ✅ COMPLETED
**Completed**: 2026-01-25
**Effort**: 1 hour

Configured wave management skills and fixed validation to work with new directory structure.

**Enhancement 54 - Wave Skills Integration**:
- ✅ Updated port references (5101 → 5104) in wave skills
- ✅ Configured /start-wave, /complete-enhancement, /complete-wave
- ✅ Updated EXECUTE_APPORTIONMENT.md with wave workflow
- ✅ Tested wave management skills
- ✅ Documented in CLAUDE.md

**Enhancement 55 - Wave Phase Validation and Title Standardization**:
- ✅ Fixed validator to search both enhancements/ and enhancements/active/
- ✅ Updated path resolution to use absolute paths
- ✅ Standardized title format (made optional)
- ✅ Validated all 47 existing enhancements found
- ✅ Documented title format decision

**Files Modified**:
- `.claude/skills/start-wave/`, `complete-enhancement/`, `complete-wave/`
- `EXECUTE_APPORTIONMENT.md` - Wave workflow documentation
- `CLAUDE.md` - Skills documentation
- `tools/wave-manager/validate_phases.py` - Path resolution fixes

**Enhancements**: [54](../enhancements/54_wave_skills_integration.md), [55](../enhancements/55_wave_phase_validation_fix.md)

### Phase 3: Enhancements 56, 57 - Title Branding & Phase Display Fix (UI Fixes)
**Status**: ✅ COMPLETED
**Completed**: 2026-01-25
**Effort**: 2 hours

Added project branding and fixed critical phase display bug across all projects.

**Enhancement 56 - Wave Manager Title Branding**:
- ✅ Added `/api/config` endpoint to app.py
- ✅ Imported PROJECT_NAME and PROJECT_COLOR from config
- ✅ Updated index.html header with projectTitle element
- ✅ Added loadConfig() JavaScript function
- ✅ Title now shows "Apportionment - Wave Manager" in blue (#2563eb)
- ✅ Browser tab title updated correctly
- ✅ Fallback to "Wave Manager" if config fetch fails

**Enhancement 57 - Wave Manager Phase Display Fix**:
- ✅ Added `linkPhasesToWaves()` function to populate wave.phases
- ✅ Called linking after both loadWaves() and loadPhases() complete
- ✅ Fixed "No phases assigned" bug for all waves
- ✅ Phase counts now accurate (e.g., "2/2 phases completed")
- ✅ Moved enhancement files from active/ to enhancements/ directory
- ✅ Updated INDEX.md to remove active/ references
- ✅ Updated validate_phases.py to remove active/ search
- ✅ Added cache-busting meta tags to prevent stale JavaScript
- ✅ Created WAVE_MANAGER_LINKING_FIX.md guide
- ✅ Applied fixes to all 5 projects (apportionment, appmanager, TCM, NHL, Performance)

**Files Modified - Apportionment**:
- `tools/wave-manager/static/index.html` - Added linkPhasesToWaves() and cache-busting
- `context/enhancements/47_*.md` through `57_*.md` - Moved from active/ to enhancements/
- `context/enhancements/INDEX.md` - Removed active/ references
- `tools/wave-manager/validate_phases.py` - Removed active/ directory search
- `WAVE_MANAGER_LINKING_FIX.md` - Created comprehensive fix guide

**Cross-Project Commits**:
- **appmanager**: Commit 7e00df3 - Added linkPhasesToWaves() and cache-busting
- **TCM**: Commit b7fe4d0 - Added linkPhasesToWaves() and cache-busting
- **NHL**: Commit 71e1952 - Added linkPhasesToWaves() and cache-busting
- **Performance**: Commit 34edc88 - Added linkPhasesToWaves() and cache-busting

---

## Dependencies

**Prerequisites**:
- Enhancement 53 (Wave Manager Schema v2.0) ✅
- Enhancement 54 (Wave Skills Integration) ✅
- Enhancement 55 (Wave Phase Validation) ✅
- config.py has PROJECT_NAME and PROJECT_COLOR ✅

**Blocking Issues**: None

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Browser caching prevents updates | High | Added cache-busting meta tags |
| Enhancement file organization | Medium | Moved to flat directory structure |
| Cross-project sync | Low | Created comprehensive guide (WAVE_MANAGER_LINKING_FIX.md) |
| Phase linking timing | Medium | Called after both async loads complete |

---

## Related Enhancements

- [Enhancement 56](../enhancements/56_wave_manager_title_branding.md) - Wave Manager Title Branding
- [Enhancement 57](../enhancements/57_wave_manager_phase_display_fix.md) - Wave Manager Phase Display Fix

---

## Implementation Details

### Bug Root Causes

**Phase Display Bug**:
1. **File Organization**: Enhancement files 47+ were in `active/` subdirectory, but API only searched main `enhancements/` directory
2. **Frontend Linking**: JavaScript didn't populate `wave.phases` array from `phase_ids`
3. **Browser Caching**: Aggressive browser caching prevented JavaScript updates from loading

**Solutions**:
1. Moved all enhancement files to flat `enhancements/` directory
2. Added `linkPhasesToWaves()` function to map phase_ids to full phase objects
3. Added cache-busting meta tags to force browser to reload JavaScript

### Code Changes

**linkPhasesToWaves() Function**:
```javascript
function linkPhasesToWaves() {
    waves.forEach(wave => {
        if (wave.phase_ids && Array.isArray(wave.phase_ids)) {
            wave.phases = wave.phase_ids.map(id => {
                return phases.find(p => p.id === id);
            }).filter(p => p !== undefined);
        } else {
            wave.phases = [];
        }
    });
}
```

**Cache-Busting Meta Tags**:
```html
<meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate">
<meta http-equiv="Pragma" content="no-cache">
<meta http-equiv="Expires" content="0">
```

**Calling Pattern**:
```javascript
async function init() {
    await loadConfig();
    await loadWaves();
    await loadPhases();
    linkPhasesToWaves();  // Called AFTER both async loads
    renderWaves();
    updateLastUpdated();
}
```

---

## Testing Results

**Apportionment**:
- ✅ Title shows "Apportionment - Wave Manager" in blue
- ✅ Wave 7 shows "2/2 phases completed" (was "0/0")
- ✅ Phases expand/collapse correctly
- ✅ All 57 enhancements load from flat directory

**Cross-Project**:
- ✅ appmanager wave manager displays phases correctly
- ✅ TCM wave manager displays phases correctly
- ✅ NHL wave manager displays phases correctly
- ✅ Performance wave manager displays phases correctly

**Browser Testing**:
- ✅ Hard refresh (Ctrl+Shift+R) loads new JavaScript
- ✅ Incognito mode shows updated version
- ✅ Cache-busting prevents stale code

---

## Notes

- This wave was extracted from original Wave 8 (API Migration) to separate concerns
- Wave manager improvements are now tracked independently
- All fixes applied consistently across all 5 projects
- Enhanced file organization eliminates subdirectory confusion
- Cache-busting ensures reliable updates across browser sessions
- WAVE_MANAGER_LINKING_FIX.md serves as reference for future projects

---

**Wave 8 Summary**: Fixed wave manager phase display bug and added project branding across all projects, improving development workflow and visual consistency.
