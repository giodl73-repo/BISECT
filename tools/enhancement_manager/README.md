# Enhancement Manager

Comprehensive suite of tools for managing, tracking, and analyzing project enhancements with GitHub integration.

**Last Updated**: January 18, 2026

## Overview

The Enhancement Manager provides:
- **Web UI** for viewing/editing enhancements (Flask app)
- **Git integration** for tracking commits and code changes
- **Size metrics** for effort estimation and analytics
- **Automated workflows** for capturing commit metadata

### Web Interface Features

The web application allows you to:

- **View all enhancements** in one place with status badges
- **Filter** by status (Completed / In Progress / Planned)
- **Search** by title or description
- **View** full markdown content with formatting
- **Edit** enhancement files inline
- **Update status** with automatic INDEX.md synchronization
- **View statistics** (totals, completion rate, complexity distribution)

## Features

### View & Browse
- Grid view of all enhancements sorted by ID (newest first)
- Status badges with color coding (green=completed, blue=in-progress, yellow=planned)
- Summary cards showing title, dates, and complexity
- Real-time statistics dashboard

### Filter & Search
- One-click filtering by status
- Real-time search across titles and descriptions
- Client-side filtering for instant results

### Edit & Update
- Inline markdown editor with syntax preservation
- Status dropdown for quick updates
- Automatic INDEX.md synchronization when status changes
- Content validation before saving

### Statistics
- Total enhancements count
- Status breakdown (completed/in-progress/planned)
- Completion rate percentage
- Complexity distribution
- **Size distribution** (XS/S/M/L/XL) - shows code change metrics
- **Size filtering** - filter enhancements by code size category
- **Commit tracking** - view GitHub commit links for each enhancement

## Git Integration Tools

### 1. Git Analyzer (`git_analyzer.py`)

Analyzes git commit history to match commits with enhancements.

**Usage**:
```bash
python git_analyzer.py [--output FILE] [--verbose]
```

**What it does**:
- Scans entire git log for commits mentioning enhancements
- Extracts enhancement numbers using regex patterns
- Calculates size metrics per commit (lines added/deleted, files modified)
- Handles batch proposals (e.g., "Enhancement 42-46" → splits to 5 enhancements)
- Outputs to `enhancement_commits.json`

**Patterns matched**:
- `Enhancement N: Title`
- `Enhancement N Phase M: Title`
- `Mark Enhancement N as Completed`
- `Add Enhancements N-M` (batch)
- And more...

**Example output** (enhancement_commits.json):
```json
{
  "48": {
    "enhancement_id": 48,
    "commit_count": 2,
    "commits": [...],
    "total_lines_changed": 1550,
    "total_files_modified": 15
  }
}
```

### 2. Update Enhancements (`update_enhancements.py`)

Batch updates all enhancement files with commit metadata.

**Usage**:
```bash
python update_enhancements.py [--dry-run] [--verbose]
```

**What it does**:
- Reads `enhancement_commits.json`
- Finds all enhancement markdown files
- Adds **Commits** and **Size** fields to each file
- Calculates size category (XS/S/M/L/XL)

**Size categories**:
- **XS**: < 100 lines (< 5 files)
- **S**: 100-500 lines (5-15 files)
- **M**: 500-1500 lines (15-30 files)
- **L**: 1500-5000 lines (30-60 files)
- **XL**: > 5000 lines (> 60 files)

### 3. Capture Commits (`capture_commits.py`)

Automatically captures commits for a specific enhancement.

**Usage**:
```bash
python capture_commits.py <enhancement_id> [--commit SHA] [--dry-run] [--verbose]
```

**Examples**:
```bash
# Capture all new commits for Enhancement 48
python capture_commits.py 48 --verbose

# Add a specific commit
python capture_commits.py 48 --commit abc123

# Preview without writing
python capture_commits.py 48 --dry-run
```

**When to use**:
- After committing changes for an enhancement
- During enhancement completion workflow
- To add a missed commit manually

**Convenience wrapper** (`add_commit.bat`):
```bash
add_commit 48           # Capture all new commits
add_commit 48 abc123    # Add specific commit
```

## Workflows

### Initial Setup (One-Time)

Generate commit metadata for all existing enhancements:

```bash
cd tools/enhancement_manager

# 1. Analyze git history
python git_analyzer.py --verbose

# 2. Update all enhancement files (preview first)
python update_enhancements.py --dry-run
python update_enhancements.py --verbose

# 3. Commit the changes
git add context/enhancements/*.md
git commit -m "Add commit metadata to all enhancements"
```

### Completing a New Enhancement

When finishing Enhancement N:

```bash
# 1. Make implementation commits
git commit -m "Enhancement N: Implementation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# 2. Capture commits automatically
python tools/enhancement_manager/capture_commits.py N --verbose

# 3. Review changes
git diff context/enhancements/*N_*.md

# 4. Complete documentation (use /enhancement-document skill)

# 5. Final commit
git commit -m "Document Enhancement N completion"
```

## Installation

### Prerequisites
- Python 3.8+
- Project enhancements stored in `../../context/enhancements/`

### Setup

1. **Install dependencies**:
   ```bash
   pip install -r requirements.txt
   ```

2. **Run the application**:
   ```bash
   python app.py
   ```

   Or use the launcher script:
   ```bash
   run.bat
   ```

3. **Open in browser**:
   Navigate to http://localhost:5001

## Usage

### Starting the Server

**Option 1: Direct Python**
```bash
cd tools/enhancement_manager
python app.py
```

**Option 2: Launcher Script** (Recommended)
```bash
cd tools/enhancement_manager
run.bat
```

The server will start on http://localhost:5001 and automatically display the URL.

### Viewing Enhancements

1. Open http://localhost:5001 in your browser
2. All enhancements load automatically
3. Use filter buttons to show specific statuses
4. Type in search box to find enhancements by keyword
5. Click any enhancement card to view details

### Editing Enhancements

1. Click an enhancement card to open detail view
2. Click "Edit" tab at the top
3. Modify the markdown content in the textarea
4. Change status using the dropdown (optional)
5. Click "Save Changes"
6. INDEX.md will be automatically updated if status changed

### Understanding Status Updates

When you change an enhancement status:
- **PLANNED → IN PROGRESS**: Adds "Started" date, moves to In Progress section in INDEX.md
- **IN PROGRESS → COMPLETED**: Adds "Completed" date, moves to Completed section in INDEX.md
- **Any status change**: Updates INDEX.md counts and table entries automatically

## Architecture

### Backend (Flask)
- **app.py**: Main Flask server with REST API
- **parser.py**: Markdown parsing and metadata extraction
- **index_sync.py**: INDEX.md synchronization logic
- **requirements.txt**: Python dependencies

### Frontend (HTML/CSS/JS)
- **static/index.html**: Single-page application
- **Tailwind CSS** (CDN): Styling
- **marked.js** (CDN): Markdown rendering
- **DOMPurify** (CDN): HTML sanitization

### Data Storage
- **No database**: File system is the source of truth
- **Direct file I/O**: Reads/writes markdown files in `../../context/enhancements/`
- **Automatic backups**: INDEX.md.backup created before updates

## API Endpoints

### GET /api/enhancements
Returns list of all enhancements with metadata.

**Response**:
```json
{
  "enhancements": [
    {
      "id": 35,
      "title": "Enhancement Manager Web App",
      "status": "🔄 IN PROGRESS",
      "priority": "Medium",
      "complexity": "Medium (3-5 hours)",
      "created": "January 16, 2026",
      "started": "January 16, 2026",
      "file": "active/35_enhancement_manager_app.md",
      "summary": "Create a simple web application..."
    }
  ]
}
```

### GET /api/enhancements/<id>
Returns full content of a single enhancement.

**Response**:
```json
{
  "id": 35,
  "content": "# Enhancement 35: Enhancement Manager...",
  "metadata": {
    "status": "🔄 IN PROGRESS",
    "priority": "Medium",
    "complexity": "Medium (3-5 hours)",
    "created": "January 16, 2026",
    "started": "January 16, 2026"
  },
  "file_path": "active/35_enhancement_manager_app.md"
}
```

### PUT /api/enhancements/<id>
Updates enhancement content.

**Request Body**:
```json
{
  "content": "# Enhancement 35: Updated content..."
}
```

**Response**:
```json
{
  "success": true,
  "message": "Enhancement updated"
}
```

### POST /api/enhancements/<id>/status
Updates enhancement status and syncs INDEX.md.

**Request Body**:
```json
{
  "status": "✅ COMPLETED",
  "completed_date": "Jan 16, 2026"
}
```

**Response**:
```json
{
  "success": true,
  "message": "Status updated"
}
```

### GET /api/stats
Returns enhancement statistics.

**Response**:
```json
{
  "total": 32,
  "completed": 21,
  "in_progress": 1,
  "planned": 11,
  "completion_rate": 65.6,
  "complexity_distribution": {
    "Low": 5,
    "Medium": 15,
    "High": 12
  },
  "recent_completions": [...]
}
```

## File Structure

```
tools/enhancement_manager/
├── README.md                      # This file
├── app.py                         # Flask web server
├── parser.py                      # Markdown parser
├── index_sync.py                  # INDEX.md synchronization
├── run.bat                        # Windows startup script
├── git_analyzer.py                # Git history analyzer (NEW)
├── update_enhancements.py         # Batch enhancement updater (NEW)
├── capture_commits.py             # Individual enhancement commit capturer (NEW)
├── add_commit.bat                 # Convenience wrapper (NEW)
├── test_git_analyzer.py           # Unit tests (NEW)
├── enhancement_commits.json       # Generated commit mapping (not in git)
├── requirements.txt               # Python dependencies
└── static/
    └── index.html                 # Web UI (single-page app)
```

## Configuration

### File Paths
The application expects the following directory structure:

```
project_root/
├── docs/
│   └── enhancements/
│       ├── INDEX.md
│       ├── active/          # In-progress and planned enhancements
│       │   └── XX_name.md
│       └── completed/       # Completed enhancements
│           └── XX_name.md
└── tools/
    └── enhancement_manager/ # This application
```

Paths are configured in `app.py` lines 21-24:
```python
BASE_PATH = Path(__file__).parent.parent.parent / 'docs' / 'enhancements'
ACTIVE_PATH = BASE_PATH / 'active'
COMPLETED_PATH = BASE_PATH / 'completed'
INDEX_PATH = BASE_PATH / 'INDEX.md'
```

### Server Settings
Default configuration runs on localhost:5001. To change:

**Port**: Edit `app.py` line 324:
```python
app.run(debug=True, port=5000, host='localhost')
```

**Debug Mode**: Set `debug=False` for production use

## Troubleshooting

### Server won't start
**Issue**: `ImportError: No module named flask`
**Solution**: Install dependencies with `pip install -r requirements.txt`

### Enhancements not loading
**Issue**: Empty list or error message
**Solution**:
1. Check that `../../context/enhancements/` directory exists
2. Verify markdown files are in `active/` and `completed/` subdirectories
3. Check console for error messages

### Cannot edit enhancement
**Issue**: Save fails or validation error
**Solution**:
1. Ensure enhancement has required frontmatter fields (Status, Priority, Created)
2. Check that file is not read-only
3. Verify markdown syntax is valid

### INDEX.md not updating
**Issue**: Status change doesn't update INDEX.md
**Solution**:
1. Check that INDEX.md is not read-only
2. Verify INDEX.md.backup is created (automatic backup)
3. Check console logs for sync errors
4. Restore from backup if needed

### Port already in use
**Issue**: `Address already in use` error
**Solution**:
1. Kill existing Flask process
2. Change port in `app.py` line 324
3. Or use `run.bat` which checks for running processes

## Security Considerations

### Intended Use
This application is designed for **local development use only**:
- Runs on localhost (not accessible from network)
- No authentication required
- Direct file system access
- No encryption

### Do NOT:
- Expose to public internet
- Run on shared/production servers
- Use with sensitive/confidential data without additional security
- Allow untrusted users to access

### File System Access
- Only reads/writes to `../../context/enhancements/` directory
- Input validation on file paths (prevents directory traversal)
- Markdown content is sanitized before HTML rendering (DOMPurify)

## Performance

### Expected Performance
- Load all 32 enhancements: <100ms
- Search across all content: <50ms (client-side)
- Save enhancement: <100ms (file write)
- Update INDEX.md: <200ms (read, modify, write)

### Optimization
- Client-side filtering and search (no server round-trips)
- Cached parsed enhancements in memory
- Lazy loading of full content (only when viewing details)

## Development

### Adding Features

**New API Endpoint**:
1. Add route decorator in `app.py`
2. Implement handler function
3. Update README API documentation
4. Test with curl or browser

**Frontend Changes**:
1. Edit `static/index.html`
2. Refresh browser (no build step required)
3. Check browser console for errors

### Testing

**Backend API**:
```bash
# Start server
python app.py

# Test endpoints
curl http://localhost:5001/api/enhancements
curl http://localhost:5001/api/enhancements/35
curl http://localhost:5001/api/stats
```

**Frontend**:
1. Open http://localhost:5001 in browser
2. Test filtering (All/Completed/In Progress/Planned)
3. Test search functionality
4. Test viewing enhancement details
5. Test editing and saving
6. Verify INDEX.md updates correctly

### Common Development Tasks

**Add new metadata field**:
1. Update `parser.py` `extract_metadata()` to parse new field
2. Update `app.py` `parse_enhancement()` to include in response
3. Update `static/index.html` to display new field

**Change UI styling**:
1. Edit Tailwind classes in `static/index.html`
2. Or add custom CSS in `<style>` section
3. Refresh browser to see changes

## Troubleshooting Git Tools

### Git analyzer finds no commits

**Cause**: Commits don't mention "Enhancement N" in message

**Solution**:
- Ensure commit messages follow patterns
- Use `git log --grep="Enhancement"` to verify
- Manually add commits using `capture_commits.py --commit SHA`

### Unicode encoding errors (Windows)

**Cause**: Git log contains non-ASCII characters

**Solution**: Already handled in git_analyzer.py with `encoding='utf-8', errors='replace'`

### Enhancement file not found

**Cause**: Looking in wrong directory or file renamed

**Solution**:
- Check both `context/enhancements/` and `context/enhancements/active/`
- Ensure filename follows pattern: `NN_description.md`

### Size metrics seem incorrect

**Cause**: Binary files or generated files inflating counts

**Solution**: Review `git diff-tree` output for the commit

## Future Enhancements

Potential additions (not in current scope):
- **Version control integration**: Show git history for enhancements
- **Diff view**: Compare versions side-by-side
- **Templates**: Create new enhancements from template
- **Export**: Generate reports (PDF, HTML)
- **Charts**: Visualize completion over time with graphs
- **Tags**: Categorize enhancements by type
- **Links**: Cross-reference between enhancements
- **File upload**: Attach documents or images

## Credits

Created for the Congressional Redistricting project:
- **Enhancement 35** (Jan 2026): Web UI for viewing/editing enhancements
- **Enhancement 48** (Jan 2026): Git integration with commit tracking and size metrics

**Technology Stack**:
- Flask 3.0.0
- Tailwind CSS 3.x (CDN)
- marked.js 11.x (CDN)
- DOMPurify 3.x (CDN)
- Python standard library (subprocess, pathlib, re, json)

## License

Part of Congressional Redistricting project. See project root for license information.
