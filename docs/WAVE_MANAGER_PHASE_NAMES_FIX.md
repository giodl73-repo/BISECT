# Wave Manager Phase Names Fix

This guide shows the changes needed to add phase name support to wave manager.

## What This Adds

**Before**: Phases show as "Phase 1", "Phase 2", "Phase 3"
**After**: Phases show as "Phase 1: Foundation", "Phase 2: Configuration", "Phase 3: UI Fixes"

---

## Fix 1: Update SCHEMA.md

**File**: `tools/wave-manager/SCHEMA.md`

**Find this section** (around line 60-90):

```markdown
### Phase Mapping Schema (NEW in v2.0)

The `**Phases**:` field explicitly maps local phase numbers/labels to global enhancement IDs.

#### Format

```markdown
**Phases**:
- Phase {label}: Enhancement {ID}
- Phase {label}: Enhancements {ID1}, {ID2}
```

#### Examples

**Sequential Numbering:**
```markdown
**Phases**:
- Phase 1: Enhancement 7
- Phase 2: Enhancement 8
- Phase 3: Enhancement 9
```
```

**Replace with**:

```markdown
### Phase Mapping Schema (NEW in v2.0)

The `**Phases**:` field explicitly maps local phase numbers/labels to global enhancement IDs. Optionally includes descriptive phase names.

#### Format

```markdown
**Phases**:
- Phase {label}: Enhancement {ID} [- {Name}] [(status)]
- Phase {label}: Enhancements {ID1}, {ID2} [- {Name}] [(status)]
```

**Note**: Phase names (after `-`) are optional but recommended for clarity.

#### Examples

**Sequential Numbering:**
```markdown
**Phases**:
- Phase 1: Enhancement 7
- Phase 2: Enhancement 8
- Phase 3: Enhancement 9
```

**With Phase Names (Recommended):**
```markdown
**Phases**:
- Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
- Phase 2: Enhancements 54, 55 - Configuration (✅ COMPLETED 2026-01-25)
- Phase 3: Enhancements 56, 57 - UI Fixes (✅ COMPLETED 2026-01-25)
```
```

**Also find the Parser Behavior section** (around line 100-110):

```markdown
#### Parser Behavior

- **With Phases field**: Uses explicit phase labels (supports 1A, 1B, etc.)
- **Without Phases field**: Falls back to sequential numbering (1, 2, 3...)
- **Display**: Wave cards show "Phase 1A", clicking opens "Enhancement 1" modal
```

**Replace with**:

```markdown
#### Parser Behavior

- **With Phases field**: Uses explicit phase labels (supports 1A, 1B, etc.)
- **Without Phases field**: Falls back to sequential numbering (1, 2, 3...)
- **Phase Names**: Extracts optional name after `-` separator
  - Format: `Phase 1: Enhancement 53 - Foundation` → name = "Foundation"
  - Without name: `Phase 1: Enhancement 53` → name = null
- **Display**:
  - With name: "Phase 1: Foundation"
  - Without name: "Phase 1"
  - Wave cards show phase label and name, clicking opens enhancement modals
```

---

## Fix 2: Update parser.py

**File**: `tools/wave-manager/parser.py`

**Find this section** (around line 100-130):

```python
    # Extract phases mapping (explicit phase definitions)
    # Format: **Phases**:
    #         - Phase 1: Enhancement 1
    #         - Phase 2: Enhancements 3, 4
    phases_section = re.search(
        r'\*\*Phases\*\*:\s*\n((?:[-*]\s+Phase\s+\d+[A-Z]?:.*\n?)+)',
        content,
        re.MULTILINE
    )
    if phases_section:
        phases_text = phases_section.group(1)
        phase_mappings = []
        # Parse each line: - Phase 1: Enhancement 1 or - Phase 1: Enhancements 1, 2
        for line in phases_text.split('\n'):
            phase_match = re.match(
                r'[-*]\s+Phase\s+(\d+[A-Z]?):\s+Enhancement[s]?\s+([\d,\s]+)',
                line.strip()
            )
            if phase_match:
                phase_num = phase_match.group(1)
                enh_nums = [int(n.strip()) for n in phase_match.group(2).split(',') if n.strip().isdigit()]
                phase_mappings.append({
                    'phase': phase_num,
                    'enhancements': enh_nums
                })
        metadata['phase_mappings'] = phase_mappings
```

**Replace with**:

```python
    # Extract phases mapping (explicit phase definitions)
    # Format: **Phases**:
    #         - Phase 1: Enhancement 1
    #         - Phase 2: Enhancements 3, 4
    #         - Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
    phases_section = re.search(
        r'\*\*Phases\*\*:\s*\n((?:[-*]\s+Phase\s+\d+[A-Z]?:.*\n?)+)',
        content,
        re.MULTILINE
    )
    if phases_section:
        phases_text = phases_section.group(1)
        phase_mappings = []
        # Parse each line: - Phase 1: Enhancement 1 or - Phase 1: Enhancements 1, 2 [- Name] [(status)]
        for line in phases_text.split('\n'):
            # Match format: Phase {num}: Enhancement(s) {ids} [- {name}] [(status)]
            phase_match = re.match(
                r'[-*]\s+Phase\s+(\d+[A-Z]?):\s+Enhancement[s]?\s+([\d,\s]+)(?:\s+-\s+([^(]+?))?(?:\s+\([^)]+\))?\s*$',
                line.strip()
            )
            if phase_match:
                phase_num = phase_match.group(1)
                enh_nums = [int(n.strip()) for n in phase_match.group(2).split(',') if n.strip().isdigit()]
                phase_name = phase_match.group(3).strip() if phase_match.group(3) else None
                phase_mappings.append({
                    'phase': phase_num,
                    'name': phase_name,
                    'enhancements': enh_nums
                })
        metadata['phase_mappings'] = phase_mappings
```

---

## Fix 3: Update app.py

**File**: `tools/wave-manager/app.py`

**Find the get_wave() function** (around line 109-121):

```python
        return jsonify({
            'id': wave_id,
            'name': wave['name'],
            'title': wave['title'],
            'status': wave['status'],
            'goal': wave['goal'],
            'success_metrics': wave['success_metrics'],
            'start_date': wave['start_date'],
            'end_date': wave['end_date'],
            'phases': wave['phases'],
            'content': content,
            'file_path': str(wave_file.relative_to(WAVES_DIR.parent))
        })
```

**Replace with**:

```python
        return jsonify({
            'id': wave_id,
            'name': wave['name'],
            'title': wave['title'],
            'status': wave['status'],
            'goal': wave['goal'],
            'success_metrics': wave['success_metrics'],
            'start_date': wave['start_date'],
            'end_date': wave['end_date'],
            'phases': wave['phases'],
            'phase_mappings': wave.get('phase_mappings', []),
            'content': content,
            'file_path': str(wave_file.relative_to(WAVES_DIR.parent))
        })
```

---

## Fix 4: Update static/index.html (Part 1 - Phase Mapping Extraction)

**File**: `tools/wave-manager/static/index.html`

**Find this section** (around line 538-559):

```javascript
            // Use phase_mappings if available, otherwise use sequential numbering
            let phasesList = '';
            if (wave.phase_mappings && wave.phase_mappings.length > 0) {
                // Build lookup map of enhancement ID to phase label
                const enhToPhase = {};
                wave.phase_mappings.forEach(mapping => {
                    mapping.enhancements.forEach(enhId => {
                        enhToPhase[enhId] = mapping.phase;
                    });
                });

                phasesList = (wave.phases || []).map(p => {
                    const phaseLabel = enhToPhase[p.id] || (wave.phases.indexOf(p) + 1);
                    return createPhaseCardInWave(p, phaseLabel);
                }).join('');
            } else {
                // Fall back to sequential numbering
                phasesList = (wave.phases || []).map((p, index) => createPhaseCardInWave(p, index + 1)).join('');
            }
```

**Replace with**:

```javascript
            // Use phase_mappings if available, otherwise use sequential numbering
            let phasesList = '';
            if (wave.phase_mappings && wave.phase_mappings.length > 0) {
                // Build lookup map of enhancement ID to phase label and name
                const enhToPhase = {};
                const enhToPhaseName = {};
                wave.phase_mappings.forEach(mapping => {
                    mapping.enhancements.forEach(enhId => {
                        enhToPhase[enhId] = mapping.phase;
                        enhToPhaseName[enhId] = mapping.name || null;
                    });
                });

                phasesList = (wave.phases || []).map(p => {
                    const phaseLabel = enhToPhase[p.id] || (wave.phases.indexOf(p) + 1);
                    const phaseName = enhToPhaseName[p.id] || null;
                    return createPhaseCardInWave(p, phaseLabel, phaseName);
                }).join('');
            } else {
                // Fall back to sequential numbering
                phasesList = (wave.phases || []).map((p, index) => createPhaseCardInWave(p, index + 1, null)).join('');
            }
```

---

## Fix 5: Update static/index.html (Part 2 - Phase Card Display)

**File**: `tools/wave-manager/static/index.html`

**Find the createPhaseCardInWave() function** (around line 632-686):

```javascript
        // Create phase card within wave (using local phase numbering)
        function createPhaseCardInWave(phase, localPhaseNumber) {
            const statusClass =
                phase.status.includes('✅') ? 'status-completed' :
                phase.status.includes('🔄') ? 'status-in-progress' :
                'status-planned';

            const priorityClass =
                phase.priority && phase.priority.includes('Critical') ? 'priority-critical' :
                phase.priority && phase.priority.includes('High') ? 'priority-high' :
                phase.priority && phase.priority.includes('Medium') ? 'priority-medium' :
                phase.priority && phase.priority.includes('Low') ? 'priority-low' :
                '';

            const priorityText = phase.priority ? phase.priority.split('|')[0].trim() : '';

            // Build commits list
            const commits = phase.commits || [];
            const githubUrls = phase.github_urls || [];
            const commitsList = commits.length > 0 ? commits.map((sha, index) => {
                const url = githubUrls[index] || '';
                return url
                    ? `<a href="${url}" target="_blank" rel="noopener noreferrer" onclick="event.stopPropagation()" class="text-blue-600 hover:text-blue-800 font-mono text-xs">${sha}</a>`
                    : `<span class="text-gray-600 font-mono text-xs">${sha}</span>`;
            }).join(', ') : '';

            // Clean up title - remove "Wave X Phase Y - " or "Wave X - " prefix since it's redundant in this context
            // Keep "Enhancement X:" since that's the unique global identifier
            let cleanTitle = phase.title;
            cleanTitle = cleanTitle.replace(/Wave \d+ Phase [0-9A-Z]+\s*-\s*/, ''); // Remove "Wave X Phase Y - "
            cleanTitle = cleanTitle.replace(/Wave \d+\s*-\s*/, ''); // Remove "Wave X - "
            cleanTitle = cleanTitle.replace(/:\s*-\s*/, ': '); // Clean up "Enhancement X: - Description" to "Enhancement X: Description"
            cleanTitle = cleanTitle.trim(); // Clean up whitespace

            return `
                <div class="phase-card" onclick="openPhase(${phase.id})">
                    <div class="flex items-start justify-between">
                        <div class="flex-1">
                            <div class="flex items-center gap-2 mb-1">
                                <span class="text-sm font-semibold text-gray-700">Phase ${localPhaseNumber}</span>
                                ${priorityClass ? `<span class="priority-badge ${priorityClass}">${priorityText}</span>` : ''}
                                <span class="status-badge ${statusClass} text-xs">
                                    ${phase.status.replace(/[📋🔄✅]/g, '').trim()}
                                </span>
                            </div>
                            <div class="text-sm font-medium text-gray-900 mb-1">${cleanTitle}</div>
                            ${commitsList ? `<div class="text-xs text-gray-600">Commits: ${commitsList}</div>` : ''}
                        </div>
                    </div>
                </div>
            `;
        }
```

**Replace with**:

```javascript
        // Create phase card within wave (using local phase numbering)
        function createPhaseCardInWave(phase, localPhaseNumber, phaseName) {
            const statusClass =
                phase.status.includes('✅') ? 'status-completed' :
                phase.status.includes('🔄') ? 'status-in-progress' :
                'status-planned';

            const priorityClass =
                phase.priority && phase.priority.includes('Critical') ? 'priority-critical' :
                phase.priority && phase.priority.includes('High') ? 'priority-high' :
                phase.priority && phase.priority.includes('Medium') ? 'priority-medium' :
                phase.priority && phase.priority.includes('Low') ? 'priority-low' :
                '';

            const priorityText = phase.priority ? phase.priority.split('|')[0].trim() : '';

            // Build commits list
            const commits = phase.commits || [];
            const githubUrls = phase.github_urls || [];
            const commitsList = commits.length > 0 ? commits.map((sha, index) => {
                const url = githubUrls[index] || '';
                return url
                    ? `<a href="${url}" target="_blank" rel="noopener noreferrer" onclick="event.stopPropagation()" class="text-blue-600 hover:text-blue-800 font-mono text-xs">${sha}</a>`
                    : `<span class="text-gray-600 font-mono text-xs">${sha}</span>`;
            }).join(', ') : '';

            // Clean up title - remove "Wave X Phase Y - " or "Wave X - " prefix since it's redundant in this context
            // Keep "Enhancement X:" since that's the unique global identifier
            let cleanTitle = phase.title;
            cleanTitle = cleanTitle.replace(/Wave \d+ Phase [0-9A-Z]+\s*-\s*/, ''); // Remove "Wave X Phase Y - "
            cleanTitle = cleanTitle.replace(/Wave \d+\s*-\s*/, ''); // Remove "Wave X - "
            cleanTitle = cleanTitle.replace(/:\s*-\s*/, ': '); // Clean up "Enhancement X: - Description" to "Enhancement X: Description"
            cleanTitle = cleanTitle.trim(); // Clean up whitespace

            // Format phase label with name if available
            const phaseLabel = phaseName ? `Phase ${localPhaseNumber}: ${phaseName}` : `Phase ${localPhaseNumber}`;

            return `
                <div class="phase-card" onclick="openPhase(${phase.id})">
                    <div class="flex items-start justify-between">
                        <div class="flex-1">
                            <div class="flex items-center gap-2 mb-1">
                                <span class="text-sm font-semibold text-gray-700">${phaseLabel}</span>
                                ${priorityClass ? `<span class="priority-badge ${priorityClass}">${priorityText}</span>` : ''}
                                <span class="status-badge ${statusClass} text-xs">
                                    ${phase.status.replace(/[📋🔄✅]/g, '').trim()}
                                </span>
                            </div>
                            <div class="text-sm font-medium text-gray-900 mb-1">${cleanTitle}</div>
                            ${commitsList ? `<div class="text-xs text-gray-600">Commits: ${commitsList}</div>` : ''}
                        </div>
                    </div>
                </div>
            `;
        }
```

---

## Summary of Changes

**Total changes needed**:
1. Update SCHEMA.md - Add phase name field documentation
2. Update parser.py - Extract phase names from wave documents
3. Update app.py - Include phase_mappings in API response
4. Update static/index.html - Two sections:
   - Build phase name lookup map
   - Display phase names in phase cards

**No changes needed to**:
- Any other backend files
- config.py

**Backward Compatible**: Waves and phases without names continue to work correctly.

---

## Apply to Projects

### App Manager
```bash
cd /c/src/appmanager/tools/wave-manager
# Apply all 5 fixes above to the corresponding files
git add SCHEMA.md parser.py app.py static/index.html
git commit -m "Add phase name support to wave manager

Added support for descriptive phase names in wave manager.
Phases can now display as 'Phase 1: Foundation' instead of 'Phase 1'.

Changes:
- Updated SCHEMA.md with phase name format
- Modified parser.py to extract phase names
- Updated app.py to include phase_mappings in API
- Enhanced frontend to display phase names

Backward compatible with phases without names.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### TCM
```bash
cd /c/src/TCM/tools/wave-manager
# Apply all 5 fixes above to the corresponding files
git add SCHEMA.md parser.py app.py static/index.html
git commit -m "Add phase name support to wave manager

Added support for descriptive phase names in wave manager.
Phases can now display as 'Phase 1: Foundation' instead of 'Phase 1'.

Changes:
- Updated SCHEMA.md with phase name format
- Modified parser.py to extract phase names
- Updated app.py to include phase_mappings in API
- Enhanced frontend to display phase names

Backward compatible with phases without names.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### NHL
```bash
cd /c/src/NHL/tools/wave-manager
# Apply all 5 fixes above to the corresponding files
git add SCHEMA.md parser.py app.py static/index.html
git commit -m "Add phase name support to wave manager

Added support for descriptive phase names in wave manager.
Phases can now display as 'Phase 1: Foundation' instead of 'Phase 1'.

Changes:
- Updated SCHEMA.md with phase name format
- Modified parser.py to extract phase names
- Updated app.py to include phase_mappings in API
- Enhanced frontend to display phase names

Backward compatible with phases without names.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

### Performance
```bash
cd /c/src/Performance/tools/wave_manager
# Note: different directory name (wave_manager not wave-manager)
# Apply all 5 fixes above to the corresponding files
git add SCHEMA.md parser.py app.py static/index.html
git commit -m "Add phase name support to wave manager

Added support for descriptive phase names in wave manager.
Phases can now display as 'Phase 1: Foundation' instead of 'Phase 1'.

Changes:
- Updated SCHEMA.md with phase name format
- Modified parser.py to extract phase names
- Updated app.py to include phase_mappings in API
- Enhanced frontend to display phase names

Backward compatible with phases without names.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Test After Applying

For each project:
1. Restart wave manager
2. Open in browser (hard refresh with Ctrl+Shift+R)
3. Find a wave with phase names in the markdown
4. Expand the phases section
5. Verify phase names display (e.g., "Phase 1: Foundation")

---

## Next Steps

After applying to all projects, each project can update their wave documents to add phase names:

**Example format**:
```markdown
**Phases**:
- Phase 1: Enhancement 53 - Foundation (✅ COMPLETED 2026-01-25)
- Phase 2: Enhancements 54, 55 - Configuration (✅ COMPLETED 2026-01-25)
- Phase 3: Enhancements 56, 57 - UI Fixes (✅ COMPLETED 2026-01-25)
```
