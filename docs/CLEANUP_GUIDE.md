# File Organization Guide

## Current Structure After Cleanup (2026-02-07)

### Backend Files (C:\src\Apportionment\backend\)
```
backend/
├── app/                          # FastAPI application
├── tests/                        # Backend tests
├── tools/                        # 🆕 Debugging scripts (not in git)
│   ├── README.md
│   ├── init_db.py               # Manual DB initialization
│   └── test_db_connection.py    # DB connection diagnostics
├── .env                          # Environment config (not in git)
├── .gitignore                    # 🆕 Ignores *.db and tools/
├── apportionment.db              # 🆕 SQLite database (temporary, not in git)
└── pyproject.toml
```

### Root Debugging Files
```
apportionment/
├── DEBUGGING_APP_MANAGER_BACKEND.md  # 🆕 Full debugging session notes
└── CLEANUP_GUIDE.md                  # 🆕 This file
```

### Scripts Organization
```
scripts/
├── data/
│   └── test_census_check.py      # 🆕 Moved from root - census data validation
├── pipeline/
│   ├── test_edge_weighting_comprehensive.py   # Pipeline tests (existing)
│   └── test_minority_edge_weighting.py        # Pipeline tests (existing)
└── ...
```

### Research Files (Leave As-Is)
```
research/
└── gerry-vra-compliance/         # User research work - don't touch
```

---

## File Categories

### 🔧 Temporary/Debugging (Not Committed)
- `backend/tools/*.py` - Debugging scripts
- `backend/apportionment.db` - SQLite database (temporary)
- `backend/.env` - Environment config
- `.claude/waves.json` - Claude plugin state

### 📝 Documentation (Should Commit)
- `DEBUGGING_APP_MANAGER_BACKEND.md` - Important session notes
- `CLEANUP_GUIDE.md` - This organization guide
- `backend/tools/README.md` - Explains debugging tools

### 🧪 Test Scripts (Evaluate Before Committing)
- `scripts/data/test_census_check.py` - Might be useful, add to git if needed
- `scripts/pipeline/test_*.py` - Pipeline-specific tests

---

## Git Status Summary

**Untracked files you'll see**:
```bash
$ git status --short

?? .claude/waves.json                          # Claude plugin - don't commit
?? DEBUGGING_APP_MANAGER_BACKEND.md            # SHOULD COMMIT - important notes
?? CLEANUP_GUIDE.md                            # SHOULD COMMIT - organization guide
?? backend/.gitignore                          # SHOULD COMMIT - ignores temp files
?? research/gerry-vra-compliance/              # User research - decide separately
?? scripts/data/test_census_check.py           # Decide if useful
?? scripts/pipeline/test_edge_weighting_comprehensive.py
?? scripts/pipeline/test_minority_edge_weighting.py
```

**Modified files**:
- `backend/.env` - Has DATABASE_URL change (SQLite temporarily)
- `backend/app/services/execution_service.py` - Fixed pipeline args
- `infrastructure/ecosystem.config.js` - Fixed PM2 config

---

## What To Commit (Recommendations)

### Definitely Commit ✅
```bash
git add DEBUGGING_APP_MANAGER_BACKEND.md
git add CLEANUP_GUIDE.md
git add backend/.gitignore
git add backend/tools/README.md
git add backend/app/services/execution_service.py  # Pipeline arg fix
```

### Maybe Commit (Decide Based on Value)
```bash
# If you want to keep backend/.env changes (SQLite temporarily)
git add backend/.env

# If PM2 changes are permanent
git add ../appmanager/infrastructure/ecosystem.config.js

# If test scripts are useful
git add scripts/data/test_census_check.py
git add scripts/pipeline/test_edge_weighting_comprehensive.py
git add scripts/pipeline/test_minority_edge_weighting.py
```

### Never Commit ❌
```bash
# Already in .gitignore
backend/apportionment.db
backend/tools/*.py  # Debugging scripts - temporary

# Claude plugin state
.claude/waves.json
```

---

## Files by Repository Section

### Main Apportionment Repo
- **Core Logic**: `src/apportionment/` (untouched)
- **Pipeline Scripts**: `scripts/pipeline/` (execution_service.py modified)
- **Data Scripts**: `scripts/data/` (added test_census_check.py)
- **Backend**: `backend/` (see Backend Files section above)
- **Frontend**: `frontend/` (untouched)
- **Docs**: Root level (added debugging docs)

### App Manager Repo (C:\src\appmanager\)
- **Infrastructure**: `infrastructure/ecosystem.config.js` (modified)
- **Shared Packages**: `packages/common-backend-utils/` (untouched)

---

## Cleanup Commands

### If you want to remove temporary files:
```bash
cd C:\src\Apportionment

# Remove SQLite database
rm backend/apportionment.db

# Remove debugging scripts (if no longer needed)
rm -rf backend/tools/

# Reset .env to example (if you want to start fresh)
cd backend
cp .env.example .env
```

### If you want to commit the important stuff:
```bash
cd C:\src\Apportionment

git add DEBUGGING_APP_MANAGER_BACKEND.md
git add CLEANUP_GUIDE.md
git add backend/.gitignore
git add backend/tools/README.md
git add backend/app/services/execution_service.py

git commit -m "Fix: App Manager backend revival - pipeline args, PM2 config, debugging docs

- Fixed pipeline argument format (--years -> -y with single value)
- Removed --reload from PM2 config (conflicts with process management)
- Added backend debugging tools directory
- Documented PostgreSQL long path DLL issue and SQLite workaround
- Added comprehensive debugging session notes

See DEBUGGING_APP_MANAGER_BACKEND.md for full context"
```

---

## Future Cleanup Tasks

1. **Fix PostgreSQL Connection** (see DEBUGGING_APP_MANAGER_BACKEND.md)
   - Once fixed, delete `backend/apportionment.db`
   - Update `.env` back to PostgreSQL

2. **Remove Debugging Scripts** (after PostgreSQL working)
   - Delete `backend/tools/` directory
   - Remove from `.gitignore`

3. **Decide on Test Scripts**
   - Review `scripts/pipeline/test_*.py`
   - Either commit or delete

4. **Research Files**
   - Review `research/gerry-vra-compliance/`
   - Commit if valuable or clean up

---

## Quick Reference

**Backend debugging**: `backend/tools/`
**Session notes**: `DEBUGGING_APP_MANAGER_BACKEND.md`
**File organization**: This file
**PM2 config**: `../appmanager/infrastructure/ecosystem.config.js`
**Backend .env**: `backend/.env`
