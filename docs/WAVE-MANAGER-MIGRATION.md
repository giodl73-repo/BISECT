# Wave Manager Migration - Apportionment

**Date**: January 26, 2026
**Status**: ✅ COMPLETED

## What Changed

The Apportionment-specific wave-manager has been **removed** and replaced with the **unified wave-manager** located in the App Manager repository.

## Old Structure (Removed)
```
C:/src/apportionment/tools/wave-manager/        ❌ DELETED
C:/src/apportionment/tools/enhancement_manager/  ❌ DELETED (obsolete, pre-wave-manager)
```

**Note**: Both wave-manager and enhancement_manager were removed. Enhancement managers were the precursor to wave-managers and are no longer needed.

## New Structure
All Apportionment waves are now managed by the unified wave-manager:
```
C:/src/appmanager/tools/wave-manager/  ✅ ACTIVE
└── Serves all 5 projects (app-manager, tcm, nhl, apportionment, performance)
```

## How to Access Apportionment Waves

### Option 1: Wave Launcher (Recommended)
1. Open App Manager: http://localhost:9000
2. Click "Waves" button in the header
3. Select "Apportionment" from the dropdown

### Option 2: Direct URL
- http://localhost:5100?project=apportionment

## Benefits

✅ Single wave-manager instance (5 → 1)
✅ Consistent UI across all projects
✅ Centralized maintenance
✅ Shared port 5100 (no more port conflicts)
✅ Project switcher for easy navigation

## Related Changes

- **Wave 4 - E42**: Backend multi-project support
- **Wave 4 - E43**: Frontend project switcher UI
- **Wave 4 - E44**: AppManager integration
- **Wave 4 - E45**: Wave Launcher redesign

## Questions?

See: `C:/src/appmanager/WAVE4-COMPLETION-SUMMARY.md`
