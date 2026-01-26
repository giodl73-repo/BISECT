# Wave Manager Quick Start

## Updated Features (v2.0)

The wave manager has been updated with bug fixes and new features from the performance project:

### 🆕 New Features

1. **Explicit Phase Mappings** - Define custom phase labels in wave documents:
   ```markdown
   **Phases**:
   - Phase 1: Enhancement 7
   - Phase 1A: Enhancement 1
   - Phase 2: Enhancements 3, 4, 5
   ```

2. **Git Commits Section** - Auto-generates GitHub URLs from commit SHAs:
   ```markdown
   ## Git Commits
   - `abc1234` - Commit message
   - `def5678` - Another commit
   ```

3. **Validation Tools** - Scripts to validate and fix phase assignments
4. **Wave Body Content** - Properly extracts and displays wave document content in UI
5. **GitHub Integration** - Automatic commit URL generation

### 🔧 Configuration

Edit `config.py`:

```python
PORT = 5101                # Wave manager port
PROJECT_NAME = "App Manager"
PROJECT_COLOR = "#3b82f6"  # Blue
GITHUB_REPO = "https://github.com/giodl_microsoft/appmanager"
```

## Quick Start

### 1. Install Dependencies

```bash
cd tools/wave-manager
pip install -r requirements.txt
```

### 2. Start Wave Manager

```bash
python app.py
```

Access at: http://localhost:5101

### 3. Validate Phase Assignments

```bash
python validate_phases.py
```

This checks that:
- All waves have proper phase-to-enhancement mappings
- Phase numbers in titles match wave definitions
- All enhancement files exist

## Schema Documentation

See `SCHEMA.md` for complete documentation on:
- Wave document structure
- Enhancement document structure
- Phase mapping format
- Git commits format
- Validation requirements

## Wave Document Format

```markdown
# Wave 1: Foundation Setup

**Date**: 2026-01-24
**Focus**: Brief wave objective
**Status**: ✅ COMPLETED
**Enhancements**: 1, 2, 3
**Phases**:
- Phase 1: Enhancement 1
- Phase 2: Enhancement 2
- Phase 3: Enhancement 3

---

## Goals

1. Primary goal
2. Secondary goal

## Implementation

What was done...

## Results

Quantified outcomes...
```

## Enhancement Document Format

```markdown
# Enhancement 1: Feature Name

**Status**: ✅ COMPLETED
**Wave**: Wave 1 (FOUNDATION)
**Priority**: High
**Created**: 2026-01-24
**Completed**: 2026-01-24
**Size**: M - 1,250 lines (15 files)

---

## Description

What was done...

---

## Git Commits

- `abc1234` - Initial implementation
- `def5678` - Bug fixes

---

## Results

Quantified impact...
```

## Utility Scripts

### validate_phases.py

Validates that phase assignments are consistent between waves and enhancements.

```bash
python validate_phases.py
```

Output:
- ✅ Shows which phases are correctly mapped
- ⚠️ Warns about missing phase info in titles
- ❌ Errors on mismatches

### fix_phase_titles.py

Bulk fixes phase numbers in enhancement titles (edit script first to define fixes).

```bash
python fix_phase_titles.py
```

## Migration from v1.0

### Add Explicit Phase Mappings

Old format (still works):
```markdown
**Enhancements**: 1, 2, 3
```

New format (recommended):
```markdown
**Enhancements**: 1, 2, 3
**Phases**:
- Phase 1: Enhancement 1
- Phase 2: Enhancement 2
- Phase 3: Enhancement 3
```

### Convert Commits to Git Commits Section

Old format (still works):
```markdown
**Commits**: abc1234, def5678
```

New format (auto-generates GitHub URLs):
```markdown
## Git Commits

- `abc1234` - Commit message
- `def5678` - Another commit
```

## API Endpoints

- `GET /api/waves` - List all waves
- `GET /api/waves/:id` - Get wave details with phases
- `GET /api/enhancements` - List all enhancements
- `GET /api/enhancements/:id` - Get enhancement details
- `PUT /api/enhancements/:id` - Update enhancement
- `POST /api/enhancements/:id/status` - Update status

## Integration with App Manager Dashboard

The App Manager dashboard (port 9000) includes quick links to:
- App Manager Wave Manager: http://localhost:5101
- TCM Wave Manager: http://localhost:5102
- NHL Wave Manager: http://localhost:5103
- Apportionment Wave Manager: http://localhost:5104

Each wave manager runs independently and tracks its own project's waves and enhancements.

## Troubleshooting

### Wave not showing up
- Check filename format: `WAVE01-NAME.md` (zero-padded)
- Verify required fields: Date, Focus, Status, Enhancements

### Enhancement not showing up
- Check filename format: `##_name.md` (no zero-padding)
- Verify has title with "Enhancement" in it

### Commits not linking to GitHub
- Check `GITHUB_REPO` in config.py is set correctly
- Verify commit SHAs are in backticks in Git Commits section

### Port already in use
- Change `PORT` in config.py
- Make sure no other wave manager is running on 5101
