# Archived Documentation

This directory contains historical session notes and development documentation that has been consolidated into the main project documentation.

## Purpose

These files document the development process, decisions, and issues encountered during the initial implementation phases (January 2026). They have been archived to reduce clutter in the root directory while preserving the historical record.

## Consolidated Into

The information from these files has been reorganized and consolidated into:

- **docs/CHANGELOG.md** - Project history, features, and changes
- **docs/DATA_FORMATS.md** - Data format specifications
- **docs/DEPENDENCIES.md** - Installation and setup
- **docs/CODING_PATTERNS.md** - Coding conventions and patterns
- **docs/ARCHITECTURE.md** - System design and algorithms
- **scripts/*/README.md** - Directory-specific documentation

## Archive Contents

### Pipeline Development (2026-01-10)
- `PROGRESS_BAR_GUIDE.md` - Progress bar protocol documentation
- `PROGRESS_BAR_UPDATES_2026-01-10.md` - Progress bar enhancement notes
- `STACKED_PROGRESS_BARS_2026-01-10.md` - Stacked progress bar implementation
- `PRINT_ONLY_ENHANCEMENTS_2026-01-10.md` - Print-only mode implementation
- `SCRIPT_FIXES_2026-01-10.md` - Script bug fixes
- `SCRIPT_FIXES_COMPLETE_2026-01-10.md` - Script fix completion
- `FINAL_SCRIPT_FIXES_2026-01-10.md` - Final script fix notes

### Session Notes (2026-01-09 - 2026-01-10)
- `claude_session_notes.md` - Initial session notes
- `SESSION_PROGRESS_2026-01-09.md` - Session 1 progress
- `SESSION_SUMMARY_2026-01-10.md` - Session 2 summary
- `SUMMARY.md` - General summary
- `STATUS.md` - Project status snapshot
- `PRODUCTION_STATUS.md` - Production readiness status
- `PRODUCTION_RUN_READY.md` - Production run preparation

### Technical Documentation (Consolidated)
- `METIS_COMPILE_GUIDE.md` → **docs/DEPENDENCIES.md**
- `PL94171_FORMAT.md` → **docs/DATA_FORMATS.md**
- `HISTORICAL_DATA_REQUIREMENTS.md` → **docs/DATA_FORMATS.md**
- `COMPACTNESS_IMPROVEMENTS.md` → **docs/CHANGELOG.md**
- `PHASE1_COMPACTNESS_IMPLEMENTATION.md` → **docs/CHANGELOG.md**

### 2010 Census Work (In Progress)
- `2010_CENSUS_PIPELINE_STATUS.md` - 2010 census pipeline status
- `2010_REDOWNLOAD_INSTRUCTIONS.md` - 2010 data download instructions

### Miscellaneous
- `IMPROVEMENTS_TODO.md` - Future improvements (now in docs/CHANGELOG.md Future Work)
- `DPI_CHANGES.md` - DPI configuration changes
- `BATCH_FILES.md` - Windows batch file documentation
- `SCRIPT_HIERARCHY_ANALYSIS.md` - Script organization analysis

## Accessing Archived Files

These files remain in version control for reference:

```bash
# View archived files
ls docs/archive/

# Read a specific archived file
cat docs/archive/PROGRESS_BAR_GUIDE.md

# Search across archived files
grep -r "pattern" docs/archive/
```

## When to Use Archive vs Current Docs

**Use Current Documentation** (root *.md and scripts/*/README.md) for:
- Understanding system architecture
- Learning coding patterns
- Setting up development environment
- Understanding data formats
- Implementing new features

**Use Archived Files** for:
- Understanding specific historical decisions
- Investigating why something was done a certain way
- Detailed session-by-session development history
- Context on specific bugs and their fixes

## Maintenance

Files are archived when:
1. Their content has been consolidated into main documentation
2. They contain dated session notes that are no longer current
3. They document temporary states during development

Files are NOT archived when:
1. They document current features or design
2. They contain unique technical information not yet consolidated
3. They are actively referenced by other documentation

---

**Archive Created**: 2026-01-11
**Reason**: Documentation consolidation and root directory cleanup
**Preserves**: 25 files documenting development from 2026-01-09 to 2026-01-10
