# Enhancement Manager

A simple web application for viewing, filtering, searching, and editing project enhancement files.

## Overview

The Enhancement Manager provides a visual interface to manage the 32+ enhancement specifications stored in `../../context/enhancements/`. It allows you to:

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
├── app.py                 # Flask server with API endpoints
├── parser.py              # Markdown parsing and validation
├── index_sync.py          # INDEX.md synchronization
├── requirements.txt       # Python dependencies
├── README.md              # This file
├── run.bat                # Windows launcher
└── static/
    └── index.html         # Single-page web application
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

Created as Enhancement 35 for the Congressional Redistricting project.

**Technology Stack**:
- Flask 3.0.0
- Tailwind CSS 3.x (CDN)
- marked.js 11.x (CDN)
- DOMPurify 3.x (CDN)

## License

Part of Congressional Redistricting project. See project root for license information.
