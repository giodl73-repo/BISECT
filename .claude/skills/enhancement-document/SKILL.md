---
name: enhancement-document
description: Complete documentation for a finished enhancement. Use when an enhancement is fully implemented and tested. Updates enhancement file, INDEX.md, CHANGELOG.md, CLAUDE.md, and related documentation files.
allowed-tools:
  - Read
  - Write
  - Edit
  - Grep
user-invocable: true
---

# Enhancement Documentation Skill

Complete docs for finished enhancement → ensures all project docs updated consistently.

## Prerequisites
Enhancement fully implemented + tested, all phases complete, outputs validated, metrics measured (if applicable)

## When to Use
User says "Document the completed enhancement", after `/enhancement-implement` completes, when testing/validation complete, before final git commit

## Workflow

### Step 1: Update Enhancement File
**File**: `context/enhancements/active/XX_name.md`

**Update status**:
```markdown
**Status**: ✅ COMPLETED
**Completed**: [Date]
**Complexity**: [Actual based on time]
```

**Capture commits** (automatically adds git commit metadata):
```bash
python tools/enhancement_manager/capture_commits.py XX --verbose
```
This will:
- Find all commits referencing Enhancement XX in git history
- Calculate size metrics (lines changed, files modified)
- Update **Commits** and **Size** fields in the enhancement file

**Add completion summary**:
```markdown
### Implementation Summary
**Completion Date**: [Date]
**What Was Built**: Deliverable 1, Deliverable 2, Deliverable 3
**Quantitative Results** (if applicable):
- Metric 1: [e.g., "52.8% improvement in Polsby-Popper compactness"]
- Metric 2: [e.g., "Reduced runtime 240→120 min (50% faster)"]
**Files Modified**: `path/file1.py` - Changes, `path/file2.py` - Changes
**Files Created**: `path/new/file.py` - Purpose
**Testing Results**: Print-only ✓, Small state (VT/DE) ✓, Multi-year (2000/2010/2020) ✓, Full validation ✓
```

### Step 2: Move Enhancement + Update INDEX
**Move**: `git mv context/enhancements/active/XX_name.md context/enhancements/completed/`
**Update INDEX.md**: Move to Completed section, add completion date, update totals

### Step 3: Update CHANGELOG.md
**Read** `docs/CHANGELOG.md`, update "Last Updated", add entry:
```markdown
## [Date] - Enhancement XX: [Name]
### Added
- New features (list)
### Changed
- Changes with quantitative improvements
### Fixed
- Bug fixes
### Files Modified
- List with brief descriptions
### Metrics
- Quantitative improvements
```

### Step 4: Update CLAUDE.md
**Update**: "Last Updated" (line 5), add to "Recent Major Changes": `[Enhancement Name] (Enhancement XX): Brief description with key metrics`
**Add to completed list**: `✅ Enhancement XX: [Name] ([Date])`
**Update Common Pitfalls** (if lessons learned): Add pitfall + solution

### Step 5-7: Update Architecture Docs (if applicable)
**ARCHITECTURE.md** (if system design changed): Update diagrams, data flow, component relationships, code patterns
**CODING_PATTERNS.md** (if new patterns): Add pattern with description, examples, when to use, anti-patterns
**DATA_FORMATS.md** (if data formats changed): Update specs, directory structure, field names, year differences

## Validation Checklist
- [ ] Enhancement file status → ✅ COMPLETED with summary + dates
- [ ] Enhancement file moved active/ → completed/
- [ ] INDEX.md updated (status, date, totals)
- [ ] Quantitative metrics documented (if applicable)
- [ ] Files modified list accurate + complete
- [ ] CHANGELOG.md updated with dated entry
- [ ] CLAUDE.md "Last Updated" + "Recent Changes" + completed list current
- [ ] ARCHITECTURE.md/CODING_PATTERNS.md/DATA_FORMATS.md updated (if changed)
- [ ] All "Last Updated" dates current

## Documentation Standards

### Quantitative Metrics
✅ **Good**: "52.8% improvement in Polsby-Popper", "240→120 min (50% faster)", "6 examples within 0.5% tolerance"
❌ **Bad**: "Much better", "Faster", "Validated successfully"

### File Lists
✅ **Good**: `file.py` - Specific change description
❌ **Bad**: "Multiple scripts"

### Testing Results
✅ **Good**: Specific test + result + metrics (e.g., "VT: ✓ 28s, all outputs created")
❌ **Bad**: Generic "Passed"

## After Documentation

1. **Review**: `git diff docs/ context/`
2. **Commit**:
```bash
git add docs/ context/
git commit -m "Enhancement XX: [Name]

[Brief description]

Key improvements:
- Improvement 1 (quantified)
- Improvement 2 (quantified)

Files modified: [count] | Files created: [count]

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```
3. **Archive** (if significant): `/create-session-archive` → `context/archive/YYYY-MM-DD_enhancement_XX.md`

## Common Mistakes
❌ Skip quantitative metrics → Numbers make improvements concrete
❌ Forget "Last Updated" dates → Keep docs synchronized
❌ List files without descriptions → Explain what changed
❌ Mark complete without testing results → Document validation
❌ Update only one doc → All 6 standard docs may need updates
❌ Write vague summaries → Be specific
