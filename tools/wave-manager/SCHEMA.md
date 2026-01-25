# Wave Manager Schema Documentation

**Last Updated**: 2026-01-25
**Version**: 2.0 (with explicit phase mappings)

---

## Overview

The Wave Manager uses markdown files with structured frontmatter to track development waves and enhancements. This document describes the expected schema after all recent updates.

---

## Wave Document Schema

### File Location
`context/waves/WAVE##-NAME.md`

### Required Format

```markdown
# Wave {ID}: {Title}

**Date**: YYYY-MM-DD [to YYYY-MM-DD]
**Focus**: {Brief description of wave focus}
**Status**: {Status emoji + text}
**Enhancements**: {comma-separated enhancement IDs}
**Phases**:
- Phase {local-ID}: Enhancement {global-ID}
- Phase {local-ID}: Enhancements {global-ID1}, {global-ID2}

---

## Goals

{Wave objectives and what you're trying to achieve}

## Implementation

{How the wave was executed}

## Results

{Quantified outcomes}

## Key Files Changed

{Modified files}
```

### Field Definitions

#### Required Fields

| Field | Format | Example | Description |
|-------|--------|---------|-------------|
| **Title** | `# Wave {ID}: {Name}` | `# Wave 1: Initial POC` | First-level heading with wave ID and name |
| **Date** | `**Date**: YYYY-MM-DD` | `**Date**: 2026-01-21` | Wave start date (can include range) |
| **Focus** | `**Focus**: {text}` | `**Focus**: Foundation establishment` | Brief wave objective |
| **Status** | `**Status**: {emoji} {text}` | `**Status**: ✅ Complete` | Current wave status |
| **Enhancements** | `**Enhancements**: {IDs}` | `**Enhancements**: 1, 2, 3` | Comma-separated enhancement IDs |

#### Optional Fields

| Field | Format | Example | Description |
|-------|--------|---------|-------------|
| **Phases** | See Phase Mapping below | Multiple lines | Explicit phase-to-enhancement mapping |
| **Start Date** | `**Start Date**: YYYY-MM-DD` | `**Start Date**: 2026-01-21` | Alternative date format |
| **End Date** | `**End Date**: YYYY-MM-DD` | `**End Date**: 2026-01-24` | Wave completion date |
| **Duration** | `**Duration**: {text}` | `**Duration**: ~15 hours` | Time spent on wave |
| **Commit** | `**Commit**: {sha}` | `**Commit**: b5ebaac` | Primary git commit |
| **Commits** | `**Commits**: {shas}` | `**Commits**: abc1234, def5678` | Multiple commits |

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

**Alphanumeric Labels with Names:**
```markdown
**Phases**:
- Phase 1A: Enhancement 1 - Data Setup (✅ COMPLETED)
- Phase 1B: Enhancement 2 - Validation (✅ COMPLETED)
```

**Multiple Enhancements per Phase:**
```markdown
**Phases**:
- Phase 1: Enhancement 3 - Foundation
- Phase 2: Enhancements 4, 5, 6 - Core Features
- Phase 3: Enhancement 7 - Polish
```

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

### Status Values

| Status | Emoji | Text | Color | Meaning |
|--------|-------|------|-------|---------|
| Planned | 📋 | PLANNED | Yellow | Not yet started |
| In Progress | 🔄 | IN PROGRESS | Blue | Currently working |
| Complete | ✅ | Complete / COMPLETED | Green | Finished |

**Note**: Parser recognizes both "Complete" and "COMPLETED"

### Complete Wave Example

```markdown
# Wave 3: Table-First Architecture

**Date**: 2026-01-22
**Focus**: Token optimization through AI-friendly formatting
**Commit**: b5ebaac
**Status**: ✅ Complete
**Enhancements**: 7, 8, 9, 10, 11
**Phases**:
- Phase 1: Enhancement 7
- Phase 2: Enhancement 8
- Phase 3: Enhancement 9
- Phase 4: Enhancement 10
- Phase 5: Enhancement 11

---

## Goals

1. Reduce token usage through table-first design
2. Improve AI readability of guidelines
3. Consolidate redundant content

## Implementation

### Phase 1: Quick Wins Tables
- Converted impact levels to table format
- Reduced 2,100 tokens to 800 tokens (62% reduction)

### Phase 2: Level Calibration Tables
- Created unified calibration tables
- Consolidated 18 files to 3 files

## Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Files | 51 | 38 | -13 files |
| Tokens | 35,000 | 22,000 | -37% |
| Load Time | 8.5s | 5.2s | -39% |

## Key Files Changed

- `data/guidelines/rewards-and-impact/impact-levels.md`
- `data/guidelines/scoring-and-calibration/level-calibration.md`
```

---

## Enhancement Document Schema

### File Location
`context/enhancements/##_name.md`

### Required Format

```markdown
# Enhancement {ID}: {Title}

**Status**: {Status emoji + text}
**Wave**: Wave {ID} ({WAVE-NAME})
**Priority**: {Priority level}
**Completed**: YYYY-MM-DD

---

## Description

{What was done}

---

## Implementation

{How it was implemented}

---

## Git Commits

- `{sha}` - {commit message}
- `{sha}` - {commit message}

---

## Results

{Quantified impact}

---

## Impact

{Long-term effects}

---

## Related Files

{Links to related documentation}

---

**Enhancement {ID} Summary**: {One-line summary}
```

### Field Definitions

#### Required Fields

| Field | Format | Example | Description |
|-------|--------|---------|-------------|
| **Title** | `# Enhancement {ID}: {Name}` | `# Enhancement 1: Initial POC` | First-level heading with ID |
| **Status** | `**Status**: {emoji} {text}` | `**Status**: ✅ Completed` | Current status |
| **Wave** | `**Wave**: Wave {ID} ({NAME})` | `**Wave**: Wave 1 (INITIAL-POC)` | Parent wave reference |
| **Priority** | `**Priority**: {level}` | `**Priority**: High` | Priority level |

#### Optional Fields

| Field | Format | Example | Description |
|-------|--------|---------|-------------|
| **Completed** | `**Completed**: YYYY-MM-DD` | `**Completed**: 2026-01-21` | Completion date |
| **Created** | `**Created**: YYYY-MM-DD` | `**Created**: 2026-01-20` | Creation date |
| **Started** | `**Started**: YYYY-MM-DD` | `**Started**: 2026-01-20` | Start date |
| **Estimated Complexity** | `**Estimated Complexity**: {level}` | `**Estimated Complexity**: Medium` | Complexity estimate |
| **Size** | `**Size**: {category} - {lines} lines ({files} files)` | `**Size**: M - 1,250 lines (15 files)` | Code change size |
| **Commits** | `**Commits**: [{sha}](url), [{sha}](url)` | See below | Git commits with links |

### Git Commits Section

#### Format 1: Section with backticks (Recommended)

```markdown
## Git Commits

- `abc1234` - Initial commit message
- `def5678` - Second commit message
```

**Parser behavior**:
- Extracts SHAs from backticks
- Auto-generates GitHub URLs: `{GITHUB_REPO}/commit/{sha}`
- Displays clickable links in UI

#### Format 2: Frontmatter with links

```markdown
**Commits**: [abc1234](https://github.com/org/repo/commit/abc1234), [def5678](https://github.com/org/repo/commit/def5678)
```

**Note**: Format 1 is preferred for readability

### Status Values

Same as Wave status values (see above)

### Priority Values

| Priority | Badge Color | Typical Use |
|----------|-------------|-------------|
| Critical | Red | Blocking issues, security fixes |
| High | Orange | Important features, major bugs |
| Medium | Yellow | Standard features |
| Low | Blue | Nice-to-haves, minor improvements |
| Research | Purple | Investigation, exploration |

### Size Categories

| Category | Lines Changed | Files Modified | Example |
|----------|---------------|----------------|---------|
| XS | <200 | 1-2 | Small bug fix |
| S | 200-500 | 2-5 | Simple feature |
| M | 500-1500 | 5-15 | Standard feature |
| L | 1500-5000 | 15-30 | Large feature |
| XL | >5000 | 30+ | Major refactor |

### Complete Enhancement Example

```markdown
# Enhancement 7: Wave 3 Phase 1 - Quick Wins Tables

**Status**: ✅ Completed
**Wave**: Wave 3 (TABLE-FIRST-ARCHITECTURE)
**Priority**: High
**Completed**: 2026-01-22
**Size**: S - 450 lines changed (8 files)

---

## Description

Converted impact levels and fiscal calendar from prose to table format, achieving significant token reduction while improving readability.

---

## Implementation

### Core Changes

**1. Impact Levels Table** (`data/guidelines/rewards-and-impact/impact-levels.md`)
- Converted 8 prose paragraphs to single table
- Added bonus/stock percentages in columns
- Included criteria and action columns

**2. Fiscal Calendar Table** (`data/guidelines/rewards-and-impact/fiscal-calendar.md`)
- Converted timeline to quarterly table
- Added promotion volume percentages
- Clear decision timing guidance

---

## Git Commits

- `abc1234` - Convert impact levels to table format
- `def5678` - Convert fiscal calendar to table format
- `ghi9012` - Update documentation with table references

---

## Results

### Quantified Impact

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Impact Levels Tokens | 2,100 | 800 | -62% |
| Fiscal Calendar Tokens | 1,800 | 650 | -64% |
| Total Files Changed | 2 | 2 | - |
| Lines Changed | 450 | - | - |

### Capabilities Enabled
- ✅ AI can parse tables faster than prose
- ✅ Reduced hallucination on level criteria
- ✅ Clearer quarterly promotion timing

---

## Impact

This phase established the "table-first" pattern that was applied across 13 subsequent guideline conversions in Phases 2-5.

---

## Related Files

- Wave document: `context/waves/WAVE03-TABLE-FIRST-ARCHITECTURE.md`
- Guidelines: `data/guidelines/rewards-and-impact/*.md`

---

**Enhancement 7 Summary**: Converted impact levels and fiscal calendar to tables, achieving 62-64% token reduction.
```

---

## GitHub Integration

### Configuration

Set in `tools/wave_manager/config.py`:

```python
GITHUB_REPO = "https://github.com/org/repo"
```

### Commit URL Generation

**Enhancement commits** in `## Git Commits` section:
- Parser extracts: `` `abc1234` ``
- Generates URL: `https://github.com/org/repo/commit/abc1234`
- Displays in UI with clickable links

**Wave commits tab**:
- Aggregates all enhancement commits
- Shows phase/enhancement association
- Links to individual enhancement modals

---

## Parser Recognition Patterns

### Wave Phase References

The parser recognizes multiple formats (backwards compatible):

1. **Explicit Phases field** (v2.0 - Recommended)
   ```markdown
   **Phases**:
   - Phase 1: Enhancement 7
   ```

2. **Enhancements field** (v1.0 - Still supported)
   ```markdown
   **Enhancements**: 7, 8, 9
   ```

3. **List items with links** (v1.0)
   ```markdown
   - [Enhancement 7](../enhancements/7_name.md)
   ```

### Enhancement Status

Parser recognizes these patterns:
- `**Status**: ✅ COMPLETED`
- `**Status**: ✅ Completed`
- `**Status**: 🔄 IN PROGRESS`
- `**Status**: 📋 PLANNED`

**Note**: Status can appear in frontmatter or title (backwards compatible)

---

## Migration Guide

### Updating Existing Waves to v2.0 Schema

1. **Add Phases field**:
   ```markdown
   **Enhancements**: 1, 2, 3
   **Phases**:
   - Phase 1: Enhancement 1
   - Phase 2: Enhancement 2
   - Phase 3: Enhancement 3
   ```

2. **Use custom labels** (optional):
   ```markdown
   **Phases**:
   - Phase 1A: Enhancement 1
   - Phase 1B: Enhancement 2
   ```

3. **Group enhancements** (optional):
   ```markdown
   **Phases**:
   - Phase 1: Enhancements 1, 2
   - Phase 2: Enhancement 3
   ```

### Backwards Compatibility

**Without Phases field**:
- Parser falls back to Enhancements field
- UI displays sequential numbers (1, 2, 3...)
- All features still work

**With Phases field**:
- UI displays custom labels (1A, 1B, etc.)
- Enhanced user experience
- Better documentation clarity

---

## Best Practices

### Wave Documents

1. **Use explicit Phases field** for clarity
2. **Number phases sequentially** unless there's a reason for custom labels
3. **Include quantified results** in tables
4. **Link to related enhancements** in text
5. **Update status emoji** when wave completes

### Enhancement Documents

1. **Use Git Commits section** with backticks (auto-generates URLs)
2. **Include Size field** for tracking code changes
3. **Write quantified results** in tables
4. **Link back to wave** in Wave field
5. **One-line summary** at end for quick reference

### Commits

1. **Link commits to enhancements** immediately after merging
2. **Use descriptive messages** that explain "why"
3. **Group related commits** under single enhancement
4. **Reference enhancement ID** in commit messages for traceability

---

## Validation

### Wave Validation Checklist

- [ ] Filename matches `WAVE##-NAME.md` pattern
- [ ] Has required frontmatter (Date, Focus, Status, Enhancements)
- [ ] Status uses emoji (📋/🔄/✅)
- [ ] Enhancement IDs are comma-separated numbers
- [ ] Phases field (if present) matches format
- [ ] Has ## Goals section
- [ ] Has ## Results section with metrics

### Enhancement Validation Checklist

- [ ] Filename matches `##_name.md` pattern
- [ ] Has required frontmatter (Status, Wave, Priority)
- [ ] Status uses emoji
- [ ] Wave reference includes ID and name
- [ ] Has ## Description section
- [ ] Has ## Git Commits section
- [ ] Commits use backtick format
- [ ] Has summary at end

---

## File Naming Conventions

### Waves
- Pattern: `WAVE##-NAME.md`
- Zero-padded: `WAVE01`, `WAVE02`, etc.
- Uppercase WAVE prefix
- Hyphens in name: `WAVE01-INITIAL-POC.md`

### Enhancements
- Pattern: `##_name.md`
- No zero-padding: `1_`, `2_`, `10_`
- Underscore separator
- Lowercase name with hyphens: `1_wave01-phase1a-initial-poc.md`

---

## Version History

### v2.0 (2026-01-25)
- Added explicit **Phases** field for wave documents
- Added GitHub integration for commit URLs
- Added wave detail modal with commits tab
- Added expand/collapse for phases in wave cards
- Improved backwards compatibility

### v1.0 (Initial)
- Basic wave and enhancement tracking
- Status indicators
- Simple phase references
- File-based architecture

---

## See Also

- `README.md` - Installation and usage guide
- `config.py` - Configuration reference
- `parser.py` - Parser implementation
- `context/waves/PHASE-SCHEMA.md` - Phase mapping examples
