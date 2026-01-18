---
name: update-docs
description: Systematically review and update all project documentation to ensure accuracy and consistency. Checks for outdated information, broken references, and missing updates. Use proactively after major changes or periodically for maintenance.
allowed-tools:
  - Read
  - Edit
  - Glob
  - Grep
user-invocable: true
---

# Update Documentation

## Overview

Systematically review and update all project documentation files to ensure they remain accurate, consistent, and helpful. This skill performs comprehensive checks across all docs and updates any outdated or inconsistent information.

## Prerequisites

No specific prerequisites required. This skill can be run at any time to ensure documentation quality.

## When to Use This Skill

- User says: "Update the documentation" or "Are the docs current?"
- After completing a major enhancement
- After changing file structure, paths, or commands
- Periodically (monthly) for maintenance
- Before releases or major milestones
- When adding new features or capabilities
- After fixing bugs that affect documentation

## Documentation Files Covered

### Core Documentation (Required Review)
1. **`CLAUDE.md`** - AI assistant guide, project overview, quick reference
2. **`README.md`** - User-facing project description, setup, usage
3. **`../../context/ARCHITECTURE.md`** - System design, data flow, components
4. **`../../context/CODING_PATTERNS.md`** - Coding conventions, patterns, protocols
5. **`../../context/DATA_FORMATS.md`** - File formats, schemas, data structures
6. **`docs/CHANGELOG.md`** - Version history, changes, improvements
7. **`docs/ENHANCEMENTS_2026.md`** - Enhancement tracking, status updates

### Secondary Documentation (Conditional Review)
8. **`docs/DEPENDENCIES.md`** - Package requirements, setup instructions
9. **`docs/CONTRIBUTING.md`** - Development workflow, contribution guidelines
10. **`../../context/SKILLS.md`** - Anthropic MCP skills documentation
11. **`../../context/archive/`** - Historical session notes (review recent only)

## Workflow

### Step 1: Identify What Changed

Determine scope of review:

**After enhancement**:
```bash
# Check what files were modified
git diff --name-only HEAD~1

# Review enhancement specification
grep -A 20 "Enhancement N" docs/ENHANCEMENTS_2026.md
```

**Periodic maintenance**:
```bash
# Check when docs were last updated
grep "Last Updated" docs/*.md

# Find docs older than 30 days
find docs/ -name "*.md" -mtime +30
```

**After major change**:
- Directory restructuring → Update all path references
- New features → Update ARCHITECTURE.md, CLAUDE.md
- Pattern changes → Update CODING_PATTERNS.md
- Data format changes → Update DATA_FORMATS.md

### Step 2: Review Each Document

For each documentation file, check:

#### CLAUDE.md

**Check for**:
- [ ] "Last Updated" date is current
- [ ] Recent Major Changes section reflects latest work
- [ ] File paths still accurate
- [ ] Command examples still work
- [ ] Enhancement status list up-to-date
- [ ] Project structure diagram matches reality
- [ ] Common pitfalls reflect learnings
- [ ] Links to other docs still valid

**Update if needed**:
```markdown
**Last Updated**: January 15, 2026  <!-- Update this -->

## Recent Major Changes (Jan 2026)
- Enhancement N: Description  <!-- Add completed enhancements -->
- Directory Unification (Enhancement 13): Complete
```

#### README.md

**Check for**:
- [ ] Feature list matches current capabilities
- [ ] Setup instructions still accurate
- [ ] Example commands still work
- [ ] Screenshots/examples reflect current UI
- [ ] Links to documentation valid
- [ ] Version numbers current
- [ ] Requirements list up-to-date

**Update if needed**:
```markdown
## Features
- ✅ Feature that now exists
- ✅ New capability added in Enhancement N
```

#### ../../context/ARCHITECTURE.md

**Check for**:
- [ ] Diagrams reflect current structure
- [ ] Component descriptions accurate
- [ ] Data flow matches implementation
- [ ] File paths current
- [ ] New components documented
- [ ] Deprecated components removed
- [ ] Integration patterns up-to-date

**Update if needed**:
```markdown
## Pipeline Stages

### Stage N: New Stage (Added 2026-01-15)
Description of new stage added in Enhancement N
```

#### ../../context/CODING_PATTERNS.md

**Check for**:
- [ ] Patterns match current code
- [ ] Examples use current file paths
- [ ] New patterns documented
- [ ] Anti-patterns updated with learnings
- [ ] Progress reporting protocol current
- [ ] Scope-based pattern examples accurate

**Update if needed**:
```markdown
### Pattern: New Pattern Name

**When to use**: Description

**Example**:
\`\`\`python
# Current implementation pattern
def example():
    pass
\`\`\`
```

#### ../../context/DATA_FORMATS.md

**Check for**:
- [ ] Directory structure diagrams current
- [ ] File format specifications accurate
- [ ] Field names match actual data
- [ ] CSV schemas up-to-date
- [ ] Year-specific differences documented
- [ ] New file types documented

**Update if needed**:
```markdown
### `new_file.csv`

**Location**: `outputs/us_{year}_{version}/data/new_file.csv`

**Schema**:
| Column | Type | Description |
|--------|------|-------------|
| field1 | str  | Description |
```

#### docs/CHANGELOG.md

**Check for**:
- [ ] Recent changes documented
- [ ] Dates accurate
- [ ] Enhancement completions added
- [ ] Bug fixes listed
- [ ] Format consistent

**Update if needed**:
```markdown
## [Unreleased]

### Added
- Enhancement N: Feature description (2026-01-15)

### Changed
- Updated thing based on Enhancement M (2026-01-14)

### Fixed
- Bug fix description (2026-01-13)
```

#### docs/ENHANCEMENTS_2026.md

**Check for**:
- [ ] Completed enhancements marked ✅
- [ ] Completion dates added
- [ ] Implementation summaries complete
- [ ] Status accurate (📋 PLANNED / 🔄 IN PROGRESS / ✅ COMPLETED)
- [ ] New enhancements added

**Update if needed**:
```markdown
### Enhancement N: Feature Name

**Status**: ✅ COMPLETED  <!-- Update from 🔄 or 📋 -->
**Completion Date**: January 15, 2026  <!-- Add this -->
**Implementation Summary**: Brief summary of what was done
```

### Step 3: Check Consistency Across Docs

Verify consistency between documents:

**File paths**:
```bash
# Extract all file paths from docs
grep -r "scripts/" docs/*.md > /tmp/paths.txt
grep -r "data/" docs/*.md >> /tmp/paths.txt
grep -r "outputs/" docs/*.md >> /tmp/paths.txt

# Verify paths exist
# (Manual verification or script)
```

**Command examples**:
```bash
# Extract commands from docs
grep -r "python scripts/" docs/*.md > /tmp/commands.txt

# Test each command with --help or --print-only
```

**Cross-references**:
```bash
# Find broken internal links
grep -r "\[.*\](.*.md)" docs/*.md

# Verify referenced sections exist
```

### Step 4: Update "Last Updated" Dates

For any document you modified:
```markdown
**Last Updated**: January 15, 2026  <!-- Today's date -->
```

**Date format**: `Month DD, YYYY` (e.g., "January 15, 2026")

### Step 5: Verify Changes

Review all changes before committing:
```bash
# View all doc changes
git diff docs/

# Check that changes are intentional
git diff CLAUDE.md
git diff ../../context/ARCHITECTURE.md
git diff docs/CHANGELOG.md
```

## Common Update Scenarios

### After Directory Restructure

**Files to update**:
- All docs with path references
- ARCHITECTURE.md (directory structure diagrams)
- DATA_FORMATS.md (file location tables)
- CLAUDE.md (project structure section)

**Find all path references**:
```bash
grep -r "data/tracts/2020/" docs/*.md
# Update to: data/tracts/{year}/
```

### After Adding New Feature

**Files to update**:
- CLAUDE.md (Recent Major Changes, features list)
- README.md (Features section)
- ARCHITECTURE.md (if new component)
- CODING_PATTERNS.md (if new pattern)
- CHANGELOG.md (Add entry)
- ENHANCEMENTS_2026.md (Mark complete)

### After Renaming Commands

**Files to update**:
- All docs with command examples
- CLAUDE.md (command reference sections)
- README.md (usage examples)
- SKILLS.md (skill command examples)

**Find command references**:
```bash
grep -r "old_command" docs/*.md
# Update to: new_command
```

### After Changing Data Formats

**Files to update**:
- DATA_FORMATS.md (schema tables)
- ARCHITECTURE.md (data flow diagrams)
- CODING_PATTERNS.md (data handling examples)

### After Bug Fixes Affecting Docs

**Files to update**:
- CHANGELOG.md (document fix)
- CLAUDE.md (update anti-patterns if relevant)
- CODING_PATTERNS.md (add "Don't Do This" if needed)

## Automation Helpers

### Check for Outdated Dates

```bash
# Find docs not updated recently
grep -H "Last Updated" docs/*.md CLAUDE.md | \
  grep -v "2026-01" | \
  cut -d: -f1
```

### Validate File Path References

```python
# validate_doc_paths.py
import re
import os
from pathlib import Path

def validate_paths_in_docs():
    """Check that all file paths mentioned in docs exist"""
    docs_dir = Path('docs')
    path_pattern = re.compile(r'`([^`]*(?:scripts|data|outputs)/[^`]+)`')

    errors = []
    for doc in docs_dir.glob('**/*.md'):
        content = doc.read_text()
        for match in path_pattern.finditer(content):
            path = match.group(1)
            # Remove {year}, {version}, etc. placeholders
            path = re.sub(r'\{[^}]+\}', '*', path)
            if not any(Path('.').glob(path)):
                errors.append(f"{doc.name}: {path} not found")

    return errors
```

### Check Internal Links

```bash
# Find markdown links
grep -rn "\[.*\](.*\.md" docs/*.md

# Verify target files exist
# (manual or script)
```

## Documentation Quality Checklist

### Completeness
- [ ] All major features documented
- [ ] All command-line tools have examples
- [ ] All data formats specified
- [ ] All coding patterns captured

### Accuracy
- [ ] Examples work as written
- [ ] Paths point to actual files
- [ ] Commands produce expected output
- [ ] Statistics/numbers current

### Consistency
- [ ] Terminology used consistently
- [ ] File path formats consistent
- [ ] Date formats consistent
- [ ] Code examples follow same style

### Clarity
- [ ] Explanations are clear
- [ ] Examples are illustrative
- [ ] Diagrams aid understanding
- [ ] No ambiguous statements

### Maintenance
- [ ] "Last Updated" dates current
- [ ] Changelog entries added
- [ ] Deprecated info removed
- [ ] TODOs resolved or updated

## Integration with Enhancement Workflow

### During Enhancement Implementation

**Phase 5: Documentation Updates** (from enhancement workflow):
1. Use this skill to update all affected docs
2. Follow enhancement's documentation plan
3. Update completion status

### After Enhancement Completion

**Use `/enhancement-document` skill** which includes:
1. Running `/update-docs` as part of completion
2. Marking enhancement complete
3. Updating changelog
4. Ensuring consistency

## Best Practices

1. **Update incrementally**: Don't let docs get stale, update after each change
2. **Test examples**: Run commands to verify they work
3. **Use consistent formatting**: Follow existing style in each doc
4. **Link related sections**: Use cross-references between docs
5. **Remove deprecated info**: Don't just add, also remove outdated content
6. **Update diagrams**: Visual aids quickly become outdated
7. **Check user perspective**: Ensure docs help new users understand
8. **Preserve history**: Don't remove useful archive notes

## Troubleshooting

**Common Issues**:

**Too many changes to update**:
```
Issue: Dozens of path references need updating
Solution: Use find-and-replace carefully with regex
          Or write script to automate bulk changes
```

**Uncertain if doc is outdated**:
```
Issue: Don't know if specific section still accurate
Solution: Test commands/examples to verify
          Check git history to see when last validated
          Ask user for clarification
```

**Conflicting information**:
```
Issue: Different docs say different things
Solution: Verify actual implementation behavior
          Update all docs to match reality
          Note discrepancies in session notes
```

**Large volume of changes**:
```
Issue: Many docs need updates after major change
Solution: Use TodoWrite to track each document
          Update systematically one by one
          Review changes before committing
```

## Performance Notes

**Typical time**:
| Scope | Time |
|-------|------|
| Single doc minor update | ~5 minutes |
| All docs after enhancement | ~20-30 minutes |
| Full documentation audit | ~1-2 hours |

**Efficiency tips**:
- Use grep/find to locate all instances quickly
- Use multi-cursor editing in IDE for bulk changes
- Create scripts for common validation tasks
- Maintain documentation checklist

## Related Skills

- `/enhancement-document` - Complete documentation after enhancement (includes this skill)
- `/create-session-archive` - Create detailed historical notes
- `/create-architecture-diagram` - Update visual documentation

## What You'll Get

After successful documentation update:
- **Accurate documentation** reflecting current state
- **Consistent information** across all docs
- **Current examples** that work as written
- **Updated timestamps** on modified docs
- **Improved maintainability** for future changes
- **Better user experience** for new contributors
- **Audit trail** of what changed and why

## Next Steps

- Commit documentation changes
- Archive session notes if significant changes
- Update skills documentation if skills affected
- Share updated docs with collaborators
- Consider creating documentation automation scripts
