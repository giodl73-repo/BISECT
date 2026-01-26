# Execute: Apportionment Repository

Step-by-step instructions for adding FastAPI API layer and React dashboard to the Apportionment project.

**Repository:** `C:\src\apportionment`
**Open VS Code:** `code C:\src\apportionment`

---

## Wave Management System

### Wave System Overview

This project uses a **wave-based development system** where:
- **Wave** = Collection of related enhancements grouped by theme and time
- **Phase** = Logical grouping of one or more enhancements within a wave
- **Enhancement** = Specific feature or improvement with tasks and commits

### Wave Organization

**Historical Waves** (WAVE01-07): Completed work organized chronologically
**Active Wave** (WAVE08): Current API Migration wave
**Future Waves** (WAVE-F2 through F7): Planned work, prioritized and ready to promote

### Wave Workflow

**Starting a New Wave**:
1. Use `/start-wave` to initialize wave planning document
2. Create enhancement files for each phase
3. Begin Phase 1 implementation

**During Wave Execution**:
1. Implement enhancement following ENHANCEMENT_WORKFLOW.md
2. Test thoroughly (unit, integration, e2e)
3. Run `/complete-enhancement X` when done
4. Move to next enhancement in wave

**Completing a Wave**:
1. Verify all enhancements completed
2. Run `/complete-wave` to finalize documentation, metrics, and archival
3. Create git commit for the wave
4. Move to next wave

### Wave Skills

**Three core skills for wave management**:

- **`/start-wave`** - Initialize new development wave
  - Creates `context/waves/WAVE##-NAME.md` planning document
  - Sets up task structure and phase breakdown
  - Identifies dependencies and risks

- **`/complete-enhancement X`** - Finalize completed enhancement
  - Updates enhancement status to COMPLETED
  - Records git commit SHA
  - Creates structured git commit
  - Syncs with Wave Manager

- **`/complete-wave`** - Finalize completed wave
  - Updates wave document (status, metrics, lessons learned)
  - Updates architecture diagrams if needed
  - Updates all affected documentation
  - Archives temporary files
  - Creates final git commit

### Wave Manager

**Web UI for tracking wave and enhancement progress**:
- **URL**: http://localhost:5104
- **Features**: View waves, enhancements, progress, metrics
- **Integration**: Auto-syncs with enhancement file status updates

**Running Wave Manager**:
```bash
cd tools/wave-manager
python app.py
```

**Configuration**:
- Port: 5104 (apportionment-specific)
- Project: "Apportionment"
- Color: Blue (#2563eb)
- Schema: v2.0 with phase mapping support

---

## Current State Analysis

### What Exists
- **Pipeline:** Python CLI for redistricting analysis
- **Core Modules:** `src/apportionment/` (census, partition, visualization)
- **Scripts:** `scripts/pipeline/` for running analyses
- **Outputs:** Generated maps, CSVs, dashboards
- **Documentation:** Extensive markdown docs and skills
- **Enhancement System:** 52 enhancements, 30 completed

### What to Add
- **Backend:** FastAPI wrapper around existing pipeline
- **Frontend:** React dashboard for running/viewing analyses
- **Database:** PostgreSQL for storing run metadata

### What NOT to Change
- Keep all existing `src/` code intact
- Keep all existing `scripts/` intact
- Keep all existing outputs and documentation

---

## Prerequisites

1. Complete `EXECUTE_APPMANAGER.md`
2. Ensure Docker is running with postgres-apportionment on port 5434
3. Complete wave migration above
4. Existing pipeline should still work
