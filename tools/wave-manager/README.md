# Wave Manager - Unified Enhancement & Wave Tracking

A Flask-based web application for managing development waves across all projects in the unified app manager system.

## Overview

The Wave Manager provides a unified interface for tracking:
- **Waves**: Collections of related phases (stored in `context/waves/`)
- **Phases**: Individual enhancements/work items (stored in `context/enhancements/`)
- **Commits**: Git commits associated with each phase
- **Metrics**: Progress, completion rates, and statistics

## Architecture

The Wave Manager implements a hierarchical model:

```
Wave = High-level collection of related phases
├── Phase 1 = Enhancement 01 (with metadata & commits)
├── Phase 2 = Enhancement 02 (with metadata & commits)
└── Phase 3 = Enhancement 03 (with metadata & commits)

File Structure:
context/
├── waves/
│   ├── WAVE01-foundation.md
│   ├── WAVE02-features.md
│   └── ...
└── enhancements/
    ├── 1_project-setup.md
    ├── 2_database-schema.md
    └── ...
```

## Port Assignment

Each project runs its own Wave Manager instance:

- **Port 5101**: AppManager Wave Manager (this instance)
- **Port 5102**: TCM Wave Manager
- **Port 5103**: NHL Wave Manager
- **Port 5104**: Apportionment Wave Manager

The App Manager Dashboard (port 9000) integrates all wave managers via API calls.

## Features

### Dual View Modes

**1. Waves View** (default)
- Shows all waves with nested phases
- Each wave card displays:
  - Wave status, start/end dates
  - Goals and objectives
  - List of all phases in the wave
  - Progress tracking (X/Y phases completed)
- Filter waves by status
- Search waves by name or goal
- Click any phase to open detail modal

**2. All Phases View**
- Shows all phases in grid layout
- Filter by status (Completed/In Progress/Planned)
- Sort by number, status, or title
- Search by title or summary
- Click any phase to open detail modal

### Phase Detail Modal

Three tabs:
- **View**: Rendered markdown content
- **Edit**: Edit status and full content
- **Commits**: View git commits with GitHub links

### Statistics Dashboard

Separate stats for waves and phases:
- Wave stats: Total waves, completed, in progress, planned
- Phase stats: Total phases, completed, in progress, planned

### Backward Compatibility

- Works without waves (show phases only)
- Phases can exist independently
- All original enhancement manager features preserved

## Directory Structure

```
tools/wave-manager/
├── app.py                  # Flask application with wave & phase endpoints
├── parser.py               # Markdown parsing for waves and phases
├── config.py               # Configuration (paths, port, debug)
├── static/
│   └── index.html          # Web UI with dual view mode
├── requirements.txt        # Dependencies
└── README.md              # This file
```

## Installation

```bash
cd tools/wave-manager
pip install -r requirements.txt
```

## Usage

### Run Wave Manager

```bash
# From tools/wave-manager/
python app.py

# Or use launcher
run.bat
```

Access at: **http://localhost:5101**

### Configuration

The `config.py` file contains all configuration:

```python
# Base directories
WAVES_DIR = CONTEXT_DIR / 'waves'
ENHANCEMENTS_DIR = CONTEXT_DIR / 'enhancements'

# Server configuration
PORT = 5101
HOST = 'localhost'
DEBUG = True
```

Directories are automatically created on startup.

### For Other Projects

Copy this wave-manager to other projects and update `config.py`:

```python
# TCM project - port 5102
PORT = 5102
PROJECT_ROOT = 'C:/src/TCM'
ENHANCEMENTS_DIR = 'context/enhancements'

# NHL project - port 5103
PORT = 5103
PROJECT_ROOT = 'C:/src/NHL'
ENHANCEMENTS_DIR = 'context/enhancements'  # To be created

# Apportionment project - port 5104
PORT = 5104
PROJECT_ROOT = 'C:/src/apportionment'
ENHANCEMENTS_DIR = 'context/enhancements'
```

## API Endpoints

The Wave Manager exposes REST API endpoints for integration:

### Waves
- `GET /api/waves` - List all waves with metadata
- `GET /api/waves/{id}` - Get wave details with linked phases
- `GET /api/waves/{id}/phases` - Get all phases in a wave

### Phases (Enhancements)
- `GET /api/enhancements` - List all phases
- `GET /api/enhancements/{id}` - Get phase details with full content
- `PUT /api/enhancements/{id}` - Update phase content
- `POST /api/enhancements/{id}/status` - Update phase status

### Statistics
- `GET /api/stats` - Get project statistics (from original enhancement manager)

### Utility
- `POST /api/shutdown` - Shutdown the server

## Integration with App Manager Dashboard

The App Manager Dashboard (port 9000) can call these APIs to display:
- Combined wave status across all 4 projects
- Recent activity feed from all projects
- Completion metrics dashboard
- Quick links to each project's wave manager

Example API call from App Manager:
```javascript
// Fetch TCM waves
const tcmWaves = await fetch('http://localhost:5102/api/waves').then(r => r.json());

// Fetch all project stats
const stats = await Promise.all([
  fetch('http://localhost:5101/api/stats').then(r => r.json()), // AppManager
  fetch('http://localhost:5102/api/stats').then(r => r.json()), // TCM
  fetch('http://localhost:5103/api/stats').then(r => r.json()), // NHL
  fetch('http://localhost:5104/api/stats').then(r => r.json())  // Apportionment
]);
```

## Wave Document Format

Create `context/waves/WAVE##-NAME.md` files:

```markdown
# Wave 1: Foundation Setup

**Status**: 📋 PLANNED
**Start Date**: January 2026
**End Date**: March 2026

## Goals

Description of wave objectives and what you're trying to achieve.

## Success Metrics

- Metric 1: Description
- Metric 2: Description
- Metric 3: Description

## Phases

Enhancements: 1, 2, 3

### Phase Breakdown

- Phase 1: Project Setup and Configuration
- Phase 2: Database Schema Design
- Phase 3: Core API Development
```

### Phase Reference Formats

The parser recognizes multiple formats:

1. **Comma-separated**: `Enhancements: 1, 2, 3`
2. **List items**: `- Enhancement 1: Description`
3. **Links**: `- [Enhancement 1](../enhancements/1_name.md)`

All formats work and can be mixed.

## Phase Document Format

Create `context/enhancements/##_name.md` files:

```markdown
# Enhancement 1: Project Setup and Configuration

**Status**: 🔄 IN PROGRESS
**Priority**: Critical
**Estimated Complexity**: Medium
**Created**: January 24, 2026

## Current State

Current situation or problem description.

## Goal

What you're trying to achieve with this phase.

## Technical Details

### Requirements
- Requirement 1
- Requirement 2

### Implementation Plan
1. Step 1
2. Step 2

## Testing
- Test case 1
- Test case 2
```

### Status Values

Both waves and phases use these status indicators:
- `📋 PLANNED` - Not yet started
- `🔄 IN PROGRESS` - Currently being worked on
- `✅ COMPLETED` - Finished

## Best Practices

1. **Plan Waves Upfront**: Create all wave documents at project start
2. **Group Thematically**: Waves should represent logical units of work (e.g., "Foundation", "Core Features", "Polish")
3. **3-5 Phases per Wave**: Keep waves manageable
4. **Update Status Regularly**: Keep wave and phase statuses in sync
5. **Document Learnings**: Add notes about what worked/didn't work

## Workflow

### Starting a New Project

1. **Create Wave Documents** in `context/waves/`
   - WAVE01-foundation.md
   - WAVE02-core-features.md
   - WAVE03-advanced-features.md
   - etc.

2. **Create Phase Documents** in `context/enhancements/`
   - 1_project-setup.md
   - 2_database-schema.md
   - 3_core-api.md
   - etc.

3. **Link Phases to Waves**
   - In each wave document, list phase IDs: `Enhancements: 1, 2, 3`

4. **Start Development**
   - Open Wave Manager at http://localhost:5101
   - See all waves with their phases
   - Click any phase to open detail view
   - Update status as you work

### During Development

1. **Update Phase Status** via the Edit tab in the detail modal
2. **Track Progress** in the Waves view (shows X/Y phases completed)
3. **Search and Filter** to find specific waves or phases
4. **Switch Views** between Waves (hierarchical) and All Phases (flat)

## Troubleshooting

**Port already in use:**
- Update `PORT` in config.py
- Or stop conflicting service

**Phases not showing:**
- Check that files exist in `context/enhancements/`
- Verify filenames match pattern: `##_name.md`
- Check file permissions

**Waves not showing:**
- Check that files exist in `context/waves/`
- Verify filenames match pattern: `WAVE##-name.md`
- Ensure phases are referenced correctly

**Phases not appearing in wave:**
- Check phase ID references in wave document
- Verify phase files exist with matching IDs
- Refresh the page

## Example Project

See the sample files created in:
- `context/waves/WAVE01-foundation.md`
- `context/enhancements/1_project-setup.md`
- `context/enhancements/2_database-schema.md`
- `context/enhancements/3_core-api.md`

## Technologies

- **Backend**: Flask (Python)
- **Frontend**: Vanilla JavaScript, Tailwind CSS
- **Markdown**: Marked.js + DOMPurify
- **Architecture**: REST API with static file serving

## License

Internal tool for the unified app manager system.
