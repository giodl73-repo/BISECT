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

## Overview

This skill completes the documentation for a finished enhancement, ensuring all project documentation is updated consistently. It's a critical final step that ensures future agents have complete context.

## Prerequisites

- Enhancement fully implemented and tested
- All phases from enhancement spec completed
- Outputs validated
- Quantitative results measured (if applicable)

## When to Use This Skill

- User says: "Document the completed enhancement"
- After `/enhancement-implement` completes successfully
- When enhancement testing and validation are complete
- Before creating final git commit

## Workflow

### Step 1: Update Enhancement File

1. **Locate** enhancement file in `../../context/enhancements/active/XX_name.md`
2. **Update status section**:
```markdown
**Status**: ✅ COMPLETED
**Completed**: [Current Date]
**Complexity**: [Actual complexity based on time spent]
```

3. **Add completion summary**:
```markdown
### Implementation Summary

**Completion Date**: [Date]

**What Was Built**:
- Deliverable 1
- Deliverable 2
- Deliverable 3

**Quantitative Results** (if applicable):
- Metric 1: [e.g., "52.8% improvement in Polsby-Popper compactness"]
- Metric 2: [e.g., "Reduced runtime from 4 hours to 2 hours"]
- Metric 3: [e.g., "All 50 states validated successfully"]

**Files Modified**:
- `path/to/file1.py` - Changes made
- `path/to/file2.py` - Changes made

**Files Created**:
- `path/to/new/file.py` - Purpose

**Testing Results**:
- Print-only: ✓ Passed
- Small state (VT/DE): ✓ Passed
- Multi-year test: ✓ Passed (2000, 2010, 2020)
- Full validation: ✓ Passed
```

### Step 2: Move Enhancement File and Update INDEX.md

1. **Move** enhancement file from `../../context/enhancements/active/` to `../../context/enhancements/completed/`
   ```bash
   git mv ../../context/enhancements/active/XX_name.md ../../context/enhancements/completed/XX_name.md
   ```

2. **Update** `../../context/enhancements/INDEX.md`:
   - Move enhancement from "In Progress" or "Planned" section to "Completed" section
   - Add completion date
   - Update total counts in headers

### Step 3: Update CHANGELOG.md

1. **Read** `docs/CHANGELOG.md`
2. **Update "Last Updated" date** at top
3. **Add new entry** after "## [Unreleased]" section:

```markdown
## [Date] - Enhancement XX: [Name]

### Added
- New feature 1
- New feature 2

### Changed
- Change 1 with quantitative improvement
- Change 2

### Fixed
- Bug fix 1
- Bug fix 2

### Files Modified
- List of modified files with brief descriptions

### Metrics
- Quantitative improvements if applicable
```

### Step 4: Update CLAUDE.md

1. **Read** `CLAUDE.md`
2. **Update "Last Updated" date** (line 5)
3. **Add to "Recent Major Changes" section**:
```markdown
- **[Enhancement Name]** (Enhancement XX): Brief description with key metrics
```

4. **Add to completed enhancements list**:
```markdown
- ✅ Enhancement XX: [Name] ([Date])
```

5. **Update "Common Pitfalls" if lessons learned**:
```markdown
X. **[New Pitfall]** - Description and solution
```

### Step 5: Update ARCHITECTURE.md (if applicable)

Only if enhancement changed system design:

1. **Read** `../../context/ARCHITECTURE.md`
2. **Update relevant sections**:
   - System design diagrams
   - Data flow descriptions
   - Component relationships
   - Code examples with new patterns

### Step 6: Update CODING_PATTERNS.md (if applicable)

Only if enhancement introduced new patterns:

1. **Read** `../../context/CODING_PATTERNS.md`
2. **Add new pattern section** or update existing:
   - Pattern description
   - Code examples
   - When to use
   - Anti-patterns (what NOT to do)

### Step 7: Update DATA_FORMATS.md (if applicable)

Only if enhancement changed data formats:

1. **Read** `../../context/DATA_FORMATS.md`
2. **Update affected sections**:
   - File format specifications
   - Directory structure diagrams
   - Field name references
   - Year-specific differences

## Validation Checklist

Before marking documentation complete, verify:

- [ ] Enhancement file status updated to ✅ COMPLETED
- [ ] Enhancement file includes completion summary with dates
- [ ] Enhancement file moved from `active/` to `completed/`
- [ ] INDEX.md updated with new status and completion date
- [ ] Quantitative metrics documented (if applicable)
- [ ] Files modified list is accurate and complete
- [ ] CHANGELOG.md updated with dated entry
- [ ] CLAUDE.md "Last Updated" date current
- [ ] CLAUDE.md "Recent Major Changes" includes enhancement
- [ ] CLAUDE.md completed enhancements list updated
- [ ] ARCHITECTURE.md updated (if system design changed)
- [ ] CODING_PATTERNS.md updated (if new patterns added)
- [ ] DATA_FORMATS.md updated (if data formats changed)
- [ ] All "Last Updated" dates current in modified docs

## Documentation Standards

### Quantitative Metrics

Always include specific numbers when documenting improvements:

**Good**:
- "52.8% improvement in Polsby-Popper compactness"
- "Reduced pipeline runtime from 240 minutes to 120 minutes (50% faster)"
- "All 6 examples validated within 0.5% tolerance"

**Not as good**:
- "Much better compactness"
- "Faster pipeline"
- "Examples validated successfully"

### File Lists

Be specific about what changed:

**Good**:
```markdown
**Files Modified**:
- `scripts/pipeline/run_complete_redistricting.py` - Added parallel analysis integration
- `scripts/political/analyze_districts.py` - Added --scope parameter for state/national modes
```

**Not as good**:
```markdown
**Files Modified**:
- Multiple pipeline scripts
- Analysis scripts
```

### Testing Results

Document what was tested and results:

**Good**:
```markdown
**Testing Results**:
- Print-only mode: ✓ Parameter threading validated
- Small state test (VT): ✓ 28 seconds, all outputs created
- Multi-year test: ✓ Tested 2000, 2010, 2020 - all passed
- Quantitative validation: 52.8% PP improvement (43 state sample, p < 0.001)
```

## What You'll Get

After completing this skill:

1. **Complete documentation trail** - Future agents understand what changed and why
2. **Quantitative record** - Specific improvements documented with numbers
3. **Pattern documentation** - New patterns available for reuse
4. **Updated quick references** - CLAUDE.md and CHANGELOG.md current
5. **Consistent timestamps** - All "Last Updated" dates synchronized
6. **Ready for commit** - Documentation complete, ready for git commit

## Next Steps

After documentation complete:

1. **Review all changes**:
   ```bash
   git diff docs/
   ```

2. **Create git commit**:
   ```bash
   git add docs/ .claude/
   git commit -m "Enhancement XX: [Name]

   [Brief description]

   Key improvements:
   - Improvement 1 (quantified)
   - Improvement 2 (quantified)

   Files modified: [count]
   Files created: [count]

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```

3. **Consider session archive** (for significant enhancements):
   - Use `/create-session-archive` to document rationale and decisions
   - Create `../../context/archive/YYYY-MM-DD_enhancement_XX_description.md`

## Common Mistakes to Avoid

1. **Don't skip quantitative metrics** - Numbers make improvements concrete
2. **Don't forget "Last Updated" dates** - Keep all docs synchronized
3. **Don't list files without descriptions** - Explain what changed in each file
4. **Don't mark complete without testing results** - Document what was validated
5. **Don't update only one doc** - All 6 standard docs may need updates
6. **Don't write vague summaries** - Be specific about what was accomplished
