# Debugging App Manager Backend - Session 2026-02-07

## Original Problem
User reported "network error" when trying to start an apportionment run through the App Manager frontend after not running the app in a long time.

## Root Causes Discovered

### 1. PostgreSQL Database Not Running
**Problem**: The `postgres-apportionment` container on port 5434 was not started.

**Fix Applied**:
```bash
cd C:\src\appmanager\infrastructure
docker-compose up -d postgres-apportionment
```

**Status**: ✅ Fixed - Container now running and healthy

---

### 2. PM2 Configuration Issues
**Problem**: Backend was configured with `--reload` flag which conflicts with PM2's process management, causing the server process to never complete startup.

**Fix Applied** (C:\src\appmanager\infrastructure\ecosystem.config.js):
```javascript
// BEFORE:
args: '-m uvicorn app.main:app --reload --host 0.0.0.0 --port 8002',

// AFTER:
args: 'app.main:app --host 0.0.0.0 --port 8002',
script: 'uvicorn',
interpreter: 'python',
```

**Status**: ✅ Fixed - Backend starts successfully

---

### 3. Database Tables Not Created
**Problem**: The lifespan event in FastAPI wasn't creating tables because the database connection was failing silently.

**Temporary Fix**:
Created `C:\src\Apportionment\backend\init_db.py`:
```python
"""Initialize database tables."""
from app.database import engine, Base
import app.models.run

print("Creating database tables...")
Base.metadata.create_all(bind=engine)
print("[OK] Tables created successfully!")

# Verify
from sqlalchemy import inspect
inspector = inspect(engine)
tables = inspector.get_table_names()
print(f"[OK] Found tables: {tables}")
```

**Status**: ⚠️ Workaround - Tables created manually, but lifespan event needs investigation

---

### 4. Windows Long Path DLL Issue (CRITICAL)
**Problem**: psycopg2-binary and psycopg (v3) both fail to load on Windows due to the extremely long virtual environment path:
```
C:/Users/giodl/AppData/Local/Packages/PythonSoftwareFoundation.Python.3.13_qbz5n2kfra8p0/
LocalCache/Local/pypoetry/Cache/virtualenvs/apportionment-backend-qBsAbpb_-py3.13/
```

**Error**:
```
ImportError: DLL load failed while importing _psycopg: The filename or extension is too long.
```

**Temporary Workaround**:
Switched to SQLite for now by updating `.env` and `ecosystem.config.js`:
```bash
# .env
DATABASE_URL=sqlite:///./apportionment.db

# ecosystem.config.js
env: {
  DATABASE_URL: 'sqlite:///./apportionment.db',
  ...
}
```

**Status**: ⚠️ **TEMPORARY** - PostgreSQL still needs to be fixed

**Permanent Solutions (NOT YET IMPLEMENTED)**:
1. **Option A: Shorter Virtual Environment Path**
   ```bash
   # Create venv in shorter location
   cd C:\src\Apportionment\backend
   poetry config virtualenvs.path C:\venvs
   poetry install
   # Update PM2 config with new path
   ```

2. **Option B: Use subst Command**
   ```bash
   # Create shorter drive mapping
   subst V: "C:\Users\giodl\AppData\Local\Packages\..."
   # Update PM2 config to use V:\ path
   ```

3. **Option C: Docker Backend**
   ```dockerfile
   # Run backend in Docker container
   # Eliminates Windows path issues entirely
   ```

4. **Option D: Alternative PostgreSQL Driver**
   - Try `pg8000` (pure Python, no DLL)
   - Or `asyncpg` (async driver)

---

### 5. Pipeline Argument Format Mismatch
**Problem**: Backend was passing `--years 2000,2010,2020` but the pipeline script only accepts single year values or "all".

**Error**:
```
run_complete_redistricting.py: error: unrecognized arguments: --years 2000,2010,2020
```

**Fix Applied** (C:\src\Apportionment\backend\app\services\execution_service.py):
```python
# BEFORE:
command = [
    "python",
    "scripts/pipeline/run_complete_redistricting.py",
    "--years", ",".join(years),  # WRONG: --years 2000,2010,2020
    ...
]

# AFTER:
if len(years) > 1:
    year_arg = "all"
else:
    year_arg = years[0]

command = [
    "python",
    "scripts/pipeline/run_complete_redistricting.py",
    "-y", year_arg,  # CORRECT: -y all or -y 2020
    "-w", str(workers),
    "--dpi", str(dpi),
    "-pm", partition_mode,
    "-v", version,
]

# Also fixed states argument:
if states:
    command.extend(["-st"] + states)  # CORRECT: -st CA TX NY
```

**Status**: ✅ Fixed - Pipeline arguments now match script expectations

---

## Current State

### ✅ Working Components
- **Frontend**: http://localhost:3002 (Vite dev server)
- **Backend**: http://localhost:8002 (Uvicorn with SQLite)
- **Database**: SQLite at `C:\src\Apportionment\backend\apportionment.db`
- **PM2 Services**: All running except wave-manager-frontend (separate issue)
- **Health Check**: `/health` endpoint returns 200 OK with database connected

### ⚠️ Known Issues

1. **PostgreSQL Not Used** (High Priority)
   - Currently using SQLite as workaround
   - PostgreSQL container running but backend can't connect
   - Needs one of the permanent solutions above

2. **Database Initialization** (Medium Priority)
   - Tables currently created via manual script
   - Lifespan event should handle this automatically
   - Need to debug why `Base.metadata.create_all()` isn't working on startup

3. **Pipeline Execution** (Unknown - Needs Testing)
   - Arguments fixed, but actual run not tested yet
   - May encounter other issues when pipeline starts
   - Need to verify Python environment can find apportionment module

---

## Files Modified

1. `C:\src\appmanager\infrastructure\ecosystem.config.js`
   - Changed script/args/interpreter for apportionment-backend
   - Removed `--reload` flag
   - Updated DATABASE_URL to SQLite temporarily

2. `C:\src\Apportionment\backend\.env`
   - Updated DATABASE_URL from PostgreSQL to SQLite

3. `C:\src\Apportionment\backend\app\services\execution_service.py`
   - Fixed `_build_command()` method
   - Changed from `--years` to `-y` with proper single value
   - Fixed states argument format

4. `C:\src\Apportionment\backend\init_db.py` (NEW)
   - Manual database initialization script
   - Workaround for lifespan event issue

5. `C:\src\Apportionment\backend\test_db_connection.py` (NEW)
   - Diagnostic script for testing database connections
   - Helped identify the DLL load error

---

## Testing Checklist

### Immediate Testing Needed
- [ ] Try creating a run through frontend UI
- [ ] Verify run starts without immediate failure
- [ ] Check if pipeline actually executes
- [ ] Monitor backend logs for errors
- [ ] Verify SQLite database gets updated with run info

### After PostgreSQL Fix
- [ ] Switch back to PostgreSQL connection
- [ ] Verify tables are created automatically
- [ ] Test run creation with PostgreSQL
- [ ] Verify data persists across backend restarts

---

## Next Steps (Priority Order)

### 1. Test Current Setup (Immediate)
```bash
# Access frontend
open http://localhost:3002

# Try creating a run with:
# - Year: 2020
# - States: VT (small state for quick test)
# - Workers: 2
# - Version: test

# Monitor backend logs
cd C:\src\appmanager
pm2 logs apportionment-backend --lines 50
```

### 2. Fix PostgreSQL Connection (High Priority)
Choose and implement one of the permanent solutions from issue #4 above.

**Recommended**: Option A (Shorter venv path) or Option C (Docker)

### 3. Fix Database Initialization (Medium Priority)
Debug why lifespan event isn't creating tables automatically.

### 4. Additional Issues to Investigate
- Why wave-manager-frontend is errored in PM2
- Whether pipeline can actually find/import apportionment module
- If output directories have correct permissions
- Whether METIS is installed and accessible

---

## Useful Commands

```bash
# Check PM2 status
cd C:\src\appmanager
pm2 list

# View backend logs
pm2 logs apportionment-backend

# Restart backend
pm2 restart apportionment-backend

# Check database
cd C:\src\Apportionment\backend
poetry run python -c "from app.database import engine; from sqlalchemy import inspect; print(inspect(engine).get_table_names())"

# Test database connection
poetry run python test_db_connection.py

# Initialize database manually
poetry run python init_db.py

# Check PostgreSQL container
docker ps | grep postgres-apportionment

# Check if backend is responding
curl http://localhost:8002/health
curl http://localhost:8002/api/v1/runs
```

---

## Important Notes

1. **Windows CP1252 Encoding**: Always use ASCII characters (`[OK]`, `[FAIL]`) instead of Unicode (✓, ✗) in Python output to avoid encoding errors.

2. **Path Separators**: The long Windows path issue affects many Python packages that use native extensions (psycopg2, psycopg3, potentially others).

3. **PM2 + Python**: The combination requires careful configuration. Using `interpreter: 'python'` and letting PM2 find Python in PATH works better than long absolute paths.

4. **SQLite Limitations**: While SQLite works for testing, PostgreSQL is needed for production due to:
   - Better concurrent access
   - JSON field support (used for run config/progress)
   - Production-grade reliability

---

## Session End State

**Date**: 2026-02-07 20:30 PST

**Backend Status**: ✅ Running on port 8002 with SQLite
**Frontend Status**: ✅ Running on port 3002
**Database Status**: ⚠️ SQLite (temporary), PostgreSQL needs fixing
**Pipeline Args**: ✅ Fixed
**Ready for Testing**: ✅ Yes, but with SQLite limitations

**Next Session Priority**: Test run creation, then fix PostgreSQL connection.
