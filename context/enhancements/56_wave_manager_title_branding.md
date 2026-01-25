# Enhancement 56 (Phase 3): Wave Manager Title Branding

**Status**: ✅ COMPLETED
**Completed**: 2026-01-25
**Wave**: Wave 8 (WAVE-MANAGER-IMPROVEMENTS)
**Priority**: Low
**Estimated Complexity**: Small

---

## Implementation Summary

**Completion Date**: 2026-01-25

**What Was Built**:
- Added `/api/config` endpoint to app.py serving PROJECT_NAME and PROJECT_COLOR
- Updated index.html header with projectTitle element and subtitle
- Added loadConfig() JavaScript function to fetch config and update branding
- Wave Manager now displays "Apportionment - Wave Manager" in blue (#2563eb)

**Files Modified**:
- `tools/wave-manager/app.py` - Added import of PROJECT_NAME, PROJECT_COLOR and /api/config endpoint
- `tools/wave-manager/static/index.html` - Updated header HTML and added loadConfig() function

**Testing Results**:
- ✅ `/api/config` endpoint returns correct JSON
- ✅ Page title shows "Apportionment - Wave Manager" in blue
- ✅ Browser tab title updated correctly
- ✅ Fallback to "Wave Manager" works if fetch fails

---

## Description

Update Wave Manager to display "Apportionment - Wave Manager" in the title with project-specific blue color, making it easy to identify which project's wave manager tab is open in the browser.

---

## Implementation

### Tasks

1. **Update app.py - Add /api/config Endpoint** (5 min)
   - Import PROJECT_NAME and PROJECT_COLOR from config
   - Add `/api/config` endpoint to serve branding configuration
   - Return JSON with projectName and projectColor

2. **Update static/index.html - Header HTML** (5 min)
   - Add `id="pageHeader"` to header
   - Change h1 to have `id="projectTitle"` with inline color style
   - Add subtitle "Project Wave & Enhancement Tracking"
   - Wrap title in div for better layout

3. **Update static/index.html - JavaScript** (5 min)
   - Add `loadConfig()` async function
   - Fetch `/api/config` and update title text and color
   - Update browser tab title (`document.title`)
   - Call `loadConfig()` first in `init()` function

4. **Test** (5 min)
   - Restart wave manager
   - Verify title shows "Apportionment - Wave Manager" in blue
   - Verify browser tab title updated
   - Check fallback works if endpoint fails

### File Changes

**Files to Modify**:
- `tools/wave-manager/app.py` - Add `/api/config` endpoint
- `tools/wave-manager/static/index.html` - Update header and add loadConfig()

**Config Already Has**:
- `tools/wave-manager/config.py` - PROJECT_NAME = "Apportionment", PROJECT_COLOR = "#2563eb" ✅

---

## Current State

**Apportionment config.py**:
```python
PROJECT_NAME = "Apportionment"
PROJECT_COLOR = "#2563eb"  # Blue
```

**Current Title**: "Wave Manager" (generic)
**After Fix**: "Apportionment - Wave Manager" (branded in blue)

---

## Success Criteria

- [x] `/api/config` endpoint added to app.py
- [x] Header HTML updated with projectTitle element
- [x] loadConfig() function added and called in init()
- [x] Title shows "Apportionment - Wave Manager" in blue (#2563eb)
- [x] Browser tab title updated
- [x] Fallback to "Wave Manager" if config fetch fails

---

## Dependencies

**Prerequisites**:
- Enhancement 53 (Wave Manager Schema v2.0) ✅
- Enhancement 54 (Wave Skills Integration) ✅
- Enhancement 55 (Wave Phase Validation) ✅
- config.py has PROJECT_NAME and PROJECT_COLOR ✅

**Blocking Issues**: None

---

## Related Files

**Implementation Guide**:
- `WAVE_MANAGER_TITLE_FIX.md` - Step-by-step instructions

**Wave Manager Files**:
- `tools/wave-manager/app.py` - Flask backend
- `tools/wave-manager/static/index.html` - Frontend HTML/JS
- `tools/wave-manager/config.py` - Configuration (already has PROJECT_NAME and PROJECT_COLOR)

---

## Technical Details

### app.py Changes

**Add import** (line ~27):
```python
from config import WAVES_DIR, ENHANCEMENTS_DIR, PORT, HOST, DEBUG, PROJECT_NAME, PROJECT_COLOR
```

**Add endpoint** (after index route):
```python
@app.route('/api/config', methods=['GET'])
def get_config():
    """Get project configuration for branding"""
    return jsonify({
        'projectName': PROJECT_NAME,
        'projectColor': PROJECT_COLOR
    })
```

### index.html Changes

**Header update** (line ~56-68):
```html
<header class="bg-white shadow-sm" id="pageHeader">
    <div class="max-w-7xl mx-auto px-4 py-6">
        <div class="flex items-center justify-between mb-4">
            <div>
                <h1 class="text-3xl font-bold" id="projectTitle" style="color: #3b82f6;">Wave Manager</h1>
                <p class="text-sm text-gray-500 mt-1">Project Wave & Enhancement Tracking</p>
            </div>
            <!-- ... buttons ... -->
        </div>
    </div>
</header>
```

**JavaScript function** (before init()):
```javascript
// Load project configuration for branding
async function loadConfig() {
    try {
        const response = await fetch('/api/config');
        const config = await response.json();

        // Update page title
        const titleEl = document.getElementById('projectTitle');
        if (titleEl && config.projectName) {
            titleEl.textContent = `${config.projectName} - Wave Manager`;
            titleEl.style.color = config.projectColor || '#3b82f6';
        }

        // Update browser tab title
        document.title = `${config.projectName} - Wave Manager`;
    } catch (error) {
        console.error('Failed to load config:', error);
        // Fallback to default
        document.title = 'Wave Manager';
    }
}

// Initialize
async function init() {
    await loadConfig();  // Add this line
    await loadWaves();
    await loadPhases();
    renderWaves();
    updateLastUpdated();
}
```

---

## Testing

```bash
# Start wave manager
cd tools/wave-manager
python app.py

# Visit http://localhost:5104
# Should see: "Apportionment - Wave Manager" in blue (#2563eb)

# Test endpoint directly
curl http://localhost:5104/api/config
# Should return: {"projectName":"Apportionment","projectColor":"#2563eb"}
```

---

## Visual Result

**Before**:
- Page title: "Wave Manager" (black text)
- Browser tab: "Wave Manager"

**After**:
- Page title: "Apportionment - Wave Manager" (blue #2563eb)
- Subtitle: "Project Wave & Enhancement Tracking"
- Browser tab: "Apportionment - Wave Manager"

---

## Notes

- This is backward compatible - no breaking changes
- Falls back to "Wave Manager" if config fetch fails
- Matches the same branding used in appmanager wave manager
- Makes it easy to distinguish multiple wave manager tabs
- No database or state changes required
- Blue color (#2563eb) matches project theme

---

**Enhancement 56 Summary**: Add project name and color branding to Wave Manager title, showing "Apportionment - Wave Manager" in blue.
