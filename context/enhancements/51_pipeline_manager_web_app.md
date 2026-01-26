# E51: Pipeline Manager Web Application

**Status**: Proposed
**Priority**: High
**Estimated Effort**: Medium (6-8h)
**Created**: 2026-01-19
**Architecture**: Simplified - Static HTML + Minimal Flask API (like dashboard + enhancement manager hybrid)

## Priority Levels

- **High**: Significantly improves developer experience and eliminates painful terminal output monitoring

## Current State

Running the redistricting pipeline requires:
- Terminal access with scrolling text output
- Manual monitoring of hierarchical STATUS protocol messages
- No visual progress feedback (just text)
- No centralized version management
- Manual navigation to dashboard after completion
- Difficult to track multiple runs or past runs

**Current workflow**:
```bash
run -v v1 -y 2020          # Start run, watch terminal scroll
# ... wait 1-4 hours watching text ...
# Open browser manually to outputs/v1/2020/index.html
```

**Pain points**:
- Terminal output is noisy and hard to parse visually
- No persistent progress view (can't close terminal and check back)
- No run history or version management
- Manual dashboard navigation
- Can't easily compare or delete old versions

## Goal

Create a web application for managing and monitoring redistricting pipeline runs with:

1. **Run Management**:
   - Start new runs with configuration UI (year, version, states, workers, DPI, mode)
   - View active runs with real-time hierarchical progress visualization
   - View run history (completed, failed, in-progress)
   - Cancel running jobs

2. **Visual Progress Display**:
   - Hierarchical progress bars (replacing STATUS protocol text)
   - Real-time updates via WebSocket/Server-Sent Events
   - Year-level progress (2020, 2010, 2000)
   - Worker-level progress (state processing, post-processing tasks)
   - Census data processing progress (if applicable)
   - Estimated time remaining

3. **Version Management**:
   - List all versions in outputs/ directory
   - View version metadata (date, status, size, years, states)
   - Delete old versions
   - Quick link to dashboard for each version
   - Disk usage statistics

4. **Run Details**:
   - View error logs (outputs/{version}/{year}/error.log)
   - View run configuration
   - Download run artifacts
   - View run duration and performance metrics

**Quantifiable goals**:
- Zero terminal monitoring required
- One-click access to dashboard when run completes
- View progress from any device/browser
- Run history for all past executions

## Implementation Plan

### Phase 1: Backend API (Minimal Flask - ~200 lines)
- [ ] Create `tools/pipeline_manager/` directory structure
- [ ] Create minimal Flask backend (`app.py`) - similar to Enhancement Manager
- [ ] **Simple API endpoints** (no WebSocket):
  - `GET /` - Serve static index.html
  - `GET /api/versions` - List all versions with metadata
  - `POST /api/runs/start` - Start new pipeline run in background
  - `GET /api/runs/active` - Get active run progress (from JSON file)
  - `GET /api/runs/history` - Get past runs (from JSON file)
  - `DELETE /api/runs/<run_id>` - Cancel active run
  - `DELETE /api/versions/<version>` - Delete version directory
  - `GET /api/logs/<version>/<year>` - Get error logs
  - `POST /api/shutdown` - Shutdown server
- [ ] **Background run manager**:
  - Spawn subprocess for run_complete_redistricting.py
  - Parse STATUS protocol messages in real-time
  - Write progress to `outputs/runs_progress.json` (like dashboard uses runs.json)
  - Update on each STATUS message
- [ ] **Simple JSON format** (no database needed):
  ```json
  {
    "active_run": {
      "run_id": "run_20260119_143022",
      "version": "v2",
      "config": {...},
      "start_time": "2026-01-19T14:30:22",
      "years": {
        "2020": {"completed": 24, "total": 50, "status": "running"},
        "2010": {"completed": 5, "total": 50, "status": "running"}
      },
      "workers": {
        "2020_0": {"state": "california", "stage": "3/7", "desc": "Political viz"},
        "2020_1": {"state": "florida", "stage": "5/7", "desc": "Compactness"}
      }
    },
    "history": [...]
  }
  ```

### Phase 2: Frontend (Static HTML + Polling - like Dashboard)
- [ ] **NO WebSocket** - just poll `/api/runs/active` every 2 seconds
- [ ] Single static `index.html` (like dashboard.html)
- [ ] Minimal CSS (reuse dashboard styles where possible)
- [ ] Simple JavaScript (no framework, like dashboard)
- [ ] Poll for updates:
  ```javascript
  setInterval(() => {
    fetch('/api/runs/active')
      .then(r => r.json())
      .then(data => updateProgressBars(data));
  }, 2000);
  ```

### Phase 3: UI Tabs (Simple, like Dashboard)
- [ ] Create single-file static web interface (`static/index.html`)
- [ ] Reuse dashboard.html CSS patterns and structure
- [ ] **Start Run Tab**:
  - Configuration form (year, version, states selector, workers, DPI, partition mode)
  - Validation (check if version exists, warn about overwrite)
  - Start button → POST /api/runs/start
- [ ] **Active Runs Tab**:
  - List running jobs with live progress bars
  - Hierarchical display:
    ```
    [2020] ████████████████░░░░ 24/50 states complete
      ├─ Worker 1: [12/50] California      | Stage 3/7: Political visualization
      └─ Worker 2: [12/50] Florida         | Stage 5/7: Compactness analysis
    [2010] ████░░░░░░░░░░░░░░░░ 5/50 states complete
      ├─ Worker 1: [3/50] Texas            | Stage 2/7: District maps
      └─ Worker 2: [2/50] New York         | Stage 1/7: Redistricting
    ```
  - Real-time updates via WebSocket
  - Cancel button for each run
  - ETA based on progress rate
- [ ] **Version Manager Tab**:
  - Table of all versions (name, date, size, years, status)
  - Delete button with confirmation
  - View dashboard link (if completed)
  - Disk usage chart
  - Sort/filter by date, size, status
- [ ] **Run History Tab**:
  - Past runs with metadata (start/end time, duration, states, status)
  - View logs button
  - Re-run button (same configuration)
- [ ] Use polling for live updates (simpler than WebSocket)
- [ ] Responsive design (works on mobile)
- [ ] **Port**: Default 5100 (configurable via --port flag)

### Phase 4: Version Management Logic
- [ ] Scan outputs/ directory for versions
- [ ] Extract metadata:
  - Version name (directory name)
  - Creation date (directory mtime)
  - Size (du -sh equivalent)
  - Years present (check for 2020/, 2010/, 2000/ subdirs)
  - Completion status (check for .states_complete markers)
  - Dashboard existence (check for index.html)
- [ ] Delete version:
  - Confirmation dialog (prevent accidents)
  - Recursive delete of outputs/{version}/
  - Update run history (mark as deleted)
- [ ] Disk usage calculation:
  - Total size of outputs/ directory
  - Per-version breakdown
  - Visual chart (pie or bar chart)

### Phase 5: Integration & Testing
- [ ] Integration with run_complete_redistricting.py:
  - Subprocess spawning with environment variables
  - STATUS protocol message forwarding
  - Write progress to JSON file (not database)
  - Error handling and logging
- [ ] Entry point script (`run_manager.bat`):
  ```batch
  @echo off
  cd tools/pipeline_manager
  python app.py --port 5100
  start http://localhost:5100
  ```
- [ ] Test scenarios:
  - Start single-year run (2020 only)
  - Start multi-year run (2020, 2010, 2000)
  - Start subset run (specific states)
  - Monitor progress in real-time
  - Cancel mid-run
  - View logs after error
  - Delete old versions
- [ ] Error handling:
  - Subprocess crashes
  - Invalid configuration
  - Port already in use (suggest --port 5101)
  - Disk full
  - JSON file corruption
- [ ] Documentation:
  - Update CLAUDE.md with run_manager.bat
  - Update QUICK_REFERENCE.md with web UI instructions
  - Add screenshots to docs/

### Phase 6: Polish & Enhancements
- [ ] Progress estimation:
  - Calculate ETA based on states/hr rate
  - Show elapsed time
  - Show estimated completion time
- [ ] Notifications:
  - Browser notification when run completes
  - Email notification (optional)
- [ ] Performance:
  - Lazy loading for large run history
  - Pagination for version list
  - WebSocket connection management
- [ ] Visual polish:
  - Color-coded status (running=blue, complete=green, error=red)
  - Smooth progress bar animations
  - Loading spinners
  - Error messages with helpful hints
- [ ] Optional features:
  - Dark mode
  - Run comparison (compare two versions)
  - Export run metadata as JSON
  - Schedule runs (cron-like)

## Files to Modify/Create

### Create (New Application - Simplified)
- `tools/pipeline_manager/app.py` - **Minimal Flask backend (200-300 lines)** - like enhancement manager
- `tools/pipeline_manager/run_manager.py` - Run subprocess + JSON writer (200-300 lines)
- `tools/pipeline_manager/version_manager.py` - Version scanning/deletion (150-200 lines)
- `tools/pipeline_manager/static/index.html` - **Single-file web interface (600-800 lines)** - like dashboard
- `tools/pipeline_manager/requirements.txt` - **Dependencies (Flask only, no SocketIO)**
- `tools/pipeline_manager/README.md` - Documentation
- `run_manager.bat` - Windows entry point
- `run_manager.sh` - Linux entry point

### Modify (Documentation)
- `CLAUDE.md` - Add run_manager.bat to common commands
- `context/QUICK_REFERENCE.md` - Add web UI instructions
- `docs/CONTRIBUTING.md` - Mention pipeline manager for running tests

**Total**: 8 new files (simplified from 10), 3 modified files

**Key Simplifications**:
- NO WebSocket (just polling like dashboard)
- NO separate CSS/JS files (inline in HTML like dashboard)
- NO database (just JSON file like dashboard's runs.json)
- Minimal Flask (200 lines vs 500-700)
- Port 5100+ (configurable)

## Testing Plan

1. **Unit Tests**:
   - `tests/unit/test_run_manager.py` - Test run subprocess management
   - `tests/unit/test_version_manager.py` - Test version scanning/deletion
   - Mock STATUS protocol messages

2. **Integration Tests**:
   - `tests/integration/test_pipeline_manager_api.py` - Test Flask API endpoints
   - Test WebSocket message flow
   - Test run lifecycle (start → progress → complete)

3. **Manual Testing**:
   - Start web UI: `run_manager.bat`
   - Start test run: year=2020, version=test_ui, states=VT
   - Monitor progress in browser
   - Verify real-time updates
   - Cancel run mid-execution
   - View error logs
   - Delete test version
   - Check dashboard link works

4. **E2E Tests** (optional):
   - Playwright tests for web UI
   - Test full run workflow
   - Test version management

## Success Criteria

- [ ] Web UI starts on http://localhost:5002
- [ ] Can start pipeline run from browser
- [ ] Real-time progress bars update correctly
- [ ] Year-level and worker-level progress visible
- [ ] Run completes and dashboard link appears
- [ ] Can delete old versions from UI
- [ ] Run history persists between sessions
- [ ] Can view error logs from UI
- [ ] No terminal monitoring required
- [ ] All STATUS protocol messages parsed correctly
- [ ] Works on Windows and Linux
- [ ] Mobile-responsive design
- [ ] Documentation updated

## Benefits

1. **Developer Experience**: Eliminates painful terminal monitoring - no more scrolling text
2. **Accessibility**: Check progress from any device/browser, don't need to keep terminal open
3. **Version Management**: Centralized UI for managing outputs/ directory, easy cleanup
4. **Run History**: Track all past runs, see what configurations were used
5. **Error Visibility**: Easy access to error logs without navigating file system
6. **Dashboard Access**: One-click link to generated dashboard
7. **Multi-Tasking**: Start run and work on other things, check back in browser
8. **Collaboration**: Share progress URL with team members
9. **Professional**: Polished UI vs terminal output

**Quantified benefits**:
- Zero terminal monitoring time (was: constant attention for 1-4 hour runs)
- Instant dashboard access (was: manual file navigation)
- Version cleanup time: 10 seconds (was: 1-2 minutes with file explorer)
- Run history tracking: Automatic (was: manual notes/screenshots)

## Dependencies

- **E50** (STATUS Protocol Unification): Provides unified parse_status_message()
- **E37** (Parallel Multi-Year Pipeline): Existing hierarchical progress structure
- **Python packages**: Flask only (no SocketIO needed with polling approach)

## Risks & Mitigations

- **Risk 1**: Polling delay (2 second lag in progress updates)
  - *Mitigation*: Acceptable trade-off for simplicity, can reduce to 1 second if needed

- **Risk 2**: Long-running subprocess management (1-4 hour runs)
  - *Mitigation*: Use robust subprocess handling, store progress state to survive restarts

- **Risk 3**: Port conflicts (5100 already in use)
  - *Mitigation*: Make port configurable via --port flag, default 5100

- **Risk 4**: Browser notification permissions
  - *Mitigation*: Make notifications optional, graceful degradation

- **Risk 5**: Large run history in JSON file
  - *Mitigation*: Limit to last 50 runs, archive older runs to separate file

## Implementation Notes

### Architecture (Hybrid: Dashboard + Enhancement Manager)

**Simplified - NO WebSocket, NO separate CSS/JS**:
```
tools/pipeline_manager/
├── app.py                 # Minimal Flask backend (200 lines) - like enhancement manager
├── run_manager.py         # Subprocess + JSON writer (200 lines)
├── version_manager.py     # Version scanning/deletion (150 lines)
├── requirements.txt       # Flask only
├── README.md             # Setup instructions
└── static/
    └── index.html        # Single-file web UI (600-800 lines) - like dashboard.html
                          # Includes inline CSS and JS (no separate files)
```

**Key Design**: Like dashboard.html (static single file) + Enhancement Manager (minimal Flask API)

### Polling API (Simpler than WebSocket)

**Frontend polls every 2 seconds**:
```javascript
// No WebSocket needed!
setInterval(() => {
  fetch('/api/runs/active')
    .then(r => r.json())
    .then(data => {
      if (data.active_run) {
        updateProgressBars(data.active_run);
      }
    });
}, 2000);
```

**Backend writes JSON file** (like dashboard's runs.json):
```json
// outputs/runs_progress.json
{
  "active_run": {
    "run_id": "run_20260119_143022",
    "version": "v2",
    "start_time": "2026-01-19T14:30:22",
    "years": {
      "2020": {"completed": 24, "total": 50, "status": "running"},
      "2010": {"completed": 5, "total": 50, "status": "queued"}
    },
    "workers": {
      "2020_0": {"state": "california", "stage": "3/7", "desc": "Political viz"},
      "2020_1": {"state": "florida", "stage": "5/7", "desc": "Compactness"}
    },
    "eta_seconds": 4980
  },
  "history": [...]
}
```

### STATUS Protocol Integration

Leverage E50's unified parser:
```python
from scripts.utils.status_protocol import parse_status_message
import json

# In subprocess stdout reader
progress_data = load_progress_json()  # Current state

for line in process.stdout:
    msg_type, data = parse_status_message(line.strip())
    if msg_type:
        # Update progress data structure
        update_progress_data(progress_data, msg_type, data)
        # Write to JSON file (frontend polls this)
        write_progress_json(progress_data)
```

### Version Metadata Example

```json
{
  "name": "v1",
  "path": "outputs/v1",
  "created": "2026-01-17T21:15:00",
  "size_bytes": 15728640000,
  "size_human": "14.6 GB",
  "years": ["2020", "2010", "2000"],
  "states_complete": {
    "2020": 50,
    "2010": 50,
    "2000": 50
  },
  "status": "complete",
  "has_dashboard": true,
  "dashboard_url": "/outputs/v1/2020/index.html"
}
```

## Related Documentation

- E35: [Enhancement Manager Web App](../35_enhancement_manager_app.md)
- E37: [Parallel Multi-Year Pipeline](../37_parallel_multi_year_pipeline.md)
- E50: [STATUS Protocol Unification](../50_status_protocol_unification.md)
- Architecture: [ARCHITECTURE.md](../../ARCHITECTURE.md)
- Coding patterns: [CODING_PATTERNS.md](../../CODING_PATTERNS.md)
