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

## Documentation Structure

The project has TWO distinct documentation systems:

### AI Context (`context/` directory) - Optimized for Claude
Dense, token-efficient documentation for AI consumption:
1. **`context/ARCHITECTURE.md`** - System design (compact, symbolic notation)
2. **`context/CODING_PATTERNS.md`** - Coding conventions (pattern-first)
3. **`context/DATA_FORMATS.md`** - File formats (inline schemas)
4. **`context/SKILLS.md`** - Skill catalog (category-based)
5. **`context/QUICK_REFERENCE.md`** - Commands, troubleshooting
6. **`context/ENHANCEMENT_WORKFLOW.md`** - Enhancement process
7. **`context/TESTING.md`** - Test system documentation
8. **`context/enhancements/`** - Enhancement specifications
9. **`context/archive/`** - Historical session notes

### Human Documentation (`docs/` directory) - User-friendly
Comprehensive, readable documentation for developers:
1. **`docs/RECURSIVE_BISECTION.md`** - Algorithm explanation (detailed)
2. **`docs/DEPENDENCIES.md`** - Package requirements, setup
3. **`docs/CONTRIBUTING.md`** - Development workflow, git practices
4. **`docs/CHANGELOG.md`** - Version history, changes
5. **`docs/DATA_DICTIONARY.md`** - *(TODO: Create)* Field definitions for users
6. **`docs/TROUBLESHOOTING.md`** - *(TODO: Create)* Common issues, solutions
7. **`docs/DEPLOYMENT.md`** - *(TODO: Create)* Production deployment guide

### Root Documentation
1. **`CLAUDE.md`** - AI assistant guide (references both context/ and docs/)
2. **`README.md`** - User-facing project overview

## Workflow

### Step 0: Check for Human Documentation Gaps

**IMPORTANT**: Ensure human docs exist for user-facing features:

```bash
# Check what exists in docs/ vs context/
ls docs/
ls context/

# Identify missing human docs
# Compare topic coverage between directories
```

**Common Gaps to Check**:
- [ ] Algorithm explanations (context/ has compact version, docs/ needs detailed)
- [ ] Setup/installation guides (DEPENDENCIES.md complete?)
- [ ] Troubleshooting guides (common errors documented?)
- [ ] Data dictionary (field definitions for users?)
- [ ] Deployment procedures (production setup documented?)
- [ ] API/CLI reference (command documentation?)

**If gaps found**: Create human-friendly versions in `docs/` that expand on AI context

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

#### context/ARCHITECTURE.md (AI-optimized)

**Check for**:
- [ ] Compact notation still clear
- [ ] Symbolic data flow accurate (→ arrows, etc.)
- [ ] File paths current
- [ ] New components documented
- [ ] Deprecated components removed
- [ ] Design decisions use ✅/❌ format

**Update if needed**:
```markdown
### Stage N: New Stage
**Input**: data → **Process**: algorithm → **Output**: results
**Added**: 2026-01-15 (Enhancement N)
```

**Note**: Keep compact for token efficiency, use symbols not prose

#### docs/RECURSIVE_BISECTION.md (Human-readable)

**Check for**:
- [ ] Algorithm explanation clear for humans
- [ ] Examples illustrative and detailed
- [ ] Mathematical notation explained
- [ ] Step-by-step walkthroughs included
- [ ] Diagrams aid understanding
- [ ] References to papers included

**Update if needed**:
```markdown
## Algorithm Overview

The recursive bisection algorithm works by...

### Step 1: Initial Partitioning
[Detailed explanation with examples]

### Step 2: Refinement
[Step-by-step process]
```

**Note**: Prioritize clarity over brevity, explain concepts thoroughly

#### context/CODING_PATTERNS.md (AI-optimized)

**Check for**:
- [ ] Patterns match current code
- [ ] Examples concise (pattern-first)
- [ ] Anti-patterns use ❌ notation
- [ ] Progress reporting (STATUS protocol) current
- [ ] Inline comments used (not verbose)

**Update if needed**:
```markdown
## Pattern: New Pattern
**Use when**: Brief trigger
**Code**:
```python
pos = int(os.environ.get('VAR'))  # Brief inline comment
```
**Anti-pattern** ❌: Don't do this
```

**Note**: Pattern-first, minimal prose, inline documentation

#### docs/CONTRIBUTING.md (Human-readable)

**Check for**:
- [ ] Git workflow explained clearly
- [ ] Branch naming conventions
- [ ] Commit message guidelines
- [ ] Code review process
- [ ] Testing requirements before PR
- [ ] Documentation update requirements

**Update if needed**:
```markdown
## Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following coding patterns...

[Detailed step-by-step guide]
```

**Note**: Assume reader is new to project, explain thoroughly

#### context/DATA_FORMATS.md (AI-optimized)

**Check for**:
- [ ] Schemas use inline format
- [ ] Field descriptions concise
- [ ] URLs inline (no separate reference section)
- [ ] Examples compact

**Update if needed**:
```markdown
### `file.csv`
**Path**: `outputs/{year}/file.csv`
**Fields**: state(str), pop(int), pct(float)
```

**Note**: Inline everything, use abbreviations, compact tables

#### docs/DATA_DICTIONARY.md (Human-readable) *[CREATE IF MISSING]*

**Should contain**:
- [ ] Every CSV field explained in detail
- [ ] Data types and ranges
- [ ] Example values
- [ ] Where data comes from
- [ ] How fields are calculated
- [ ] Common use cases

**Example structure**:
```markdown
# Data Dictionary

## District Summary CSV

### Field: `polsby_popper_score`

**Type**: Float (0.0 to 1.0)

**Description**: Measures district compactness using the Polsby-Popper metric, which compares the district's area to the area of a circle with the same perimeter.

**Formula**: 4π × Area / Perimeter²

**Interpretation**:
- 1.0 = Perfect circle (most compact)
- 0.5 = Moderately compact
- 0.1 = Highly irregular (gerrymandered?)

**Example Values**:
- Wyoming District 1: 0.67 (fairly compact)
- Maryland District 3: 0.14 (very irregular)

**Data Source**: Calculated from district geometries
```

**Note**: Write for users who need to understand the data

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

#### docs/TROUBLESHOOTING.md (Human-readable) *[CREATE IF MISSING]*

**Should contain**:
- [ ] Common error messages and solutions
- [ ] Platform-specific issues (Windows/Linux/Mac)
- [ ] Data quality issues
- [ ] Performance problems
- [ ] Installation issues

**Example structure**:
```markdown
# Troubleshooting Guide

## Installation Issues

### Error: "METIS not found"

**Problem**: Pipeline fails with "gpmetis command not found"

**Solution**:
1. Install METIS: `brew install metis` (Mac) or download from...
2. Add to PATH: `export PATH=/path/to/metis:$PATH`
3. Verify: `gpmetis --help`

**Why this happens**: METIS is required for graph partitioning...

## Runtime Errors

### Error: "UnicodeEncodeError: 'charmap' codec"

**Problem**: Console output fails on Windows

**Solution**: Never use Unicode characters (✓, →, etc.) in console output. Use ASCII: [OK], ->, etc.

**Code fix**:
```python
# Bad
print("✓ Complete")

# Good
print("[OK] Complete")
```
```

**Note**: Write for users troubleshooting real problems

#### context/enhancements/INDEX.md

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

### Creating Missing Human Documentation

**When**: User reports docs/ directory has gaps compared to context/

**Process**:
1. **Identify gaps**: Compare topic coverage (context/ has it, docs/ doesn't)
2. **Determine need**: Is this user-facing? Do external users need this?
3. **Create human version**: Expand AI context into readable documentation
4. **Cross-reference**: Link from context/ to docs/ and vice versa

**Example - Creating docs/TROUBLESHOOTING.md**:
```bash
# 1. Check what exists
ls context/QUICK_REFERENCE.md  # Has troubleshooting section (compact)
ls docs/TROUBLESHOOTING.md     # Doesn't exist

# 2. Read AI version
# context/QUICK_REFERENCE.md has:
# "UnicodeEncodeError → Use ASCII not Unicode"

# 3. Create human version
# docs/TROUBLESHOOTING.md should have:
# - Detailed error message
# - Why it happens (Windows CP1252)
# - Step-by-step solution
# - Code examples (before/after)
# - Platform-specific notes
```

**Files typically needing human versions**:
- [ ] Algorithm explanations (context/ARCHITECTURE.md → docs/RECURSIVE_BISECTION.md)
- [ ] Troubleshooting (context/QUICK_REFERENCE.md → docs/TROUBLESHOOTING.md)
- [ ] Data dictionary (context/DATA_FORMATS.md → docs/DATA_DICTIONARY.md)
- [ ] Deployment guide (context/ARCHITECTURE.md → docs/DEPLOYMENT.md)

### After Directory Restructure

**Files to update**:
- **AI context**: context/ARCHITECTURE.md, context/DATA_FORMATS.md (compact updates)
- **Human docs**: README.md, docs/CONTRIBUTING.md (detailed explanations)
- **Root**: CLAUDE.md (project structure section, references both)

**Find all path references**:
```bash
# Check both directories
grep -r "data/tracts/2020/" context/*.md docs/*.md
# Update to: data/tracts/{year}/
```

### After Adding New Feature

**AI context updates**:
- context/ARCHITECTURE.md (compact component description)
- context/CODING_PATTERNS.md (pattern-first examples)
- context/enhancements/INDEX.md (mark complete)

**Human docs updates**:
- README.md (features section with explanations)
- docs/CHANGELOG.md (detailed entry)
- docs/RECURSIVE_BISECTION.md (if algorithm affected)
- docs/TROUBLESHOOTING.md (if new errors possible)

**Root updates**:
- CLAUDE.md (Recent Major Changes, quick reference)

### After Renaming Commands

**Files to update** (both AI and human):
- CLAUDE.md (command reference, examples)
- README.md (usage examples)
- context/QUICK_REFERENCE.md (command list)
- context/SKILLS.md (skill command examples)
- docs/CONTRIBUTING.md (workflow commands)

**Find command references**:
```bash
grep -r "old_command" context/*.md docs/*.md README.md CLAUDE.md
# Update to: new_command
```

### After Changing Data Formats

**AI context updates**:
- context/DATA_FORMATS.md (inline schema, compact)

**Human docs updates**:
- docs/DATA_DICTIONARY.md (detailed field explanations)
- README.md (if user-visible format change)

**Both**:
- context/ARCHITECTURE.md (data flow diagrams)
- context/CODING_PATTERNS.md (data handling examples)

### After Bug Fixes Affecting Docs

**Updates needed**:
- docs/CHANGELOG.md (document fix for humans)
- docs/TROUBLESHOOTING.md (add solution for this error)
- context/CODING_PATTERNS.md (add anti-pattern ❌ if relevant)
- CLAUDE.md (update common pitfalls if major)

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

## Documentation Philosophy

**Two Audiences, Two Formats**:

### AI Context (context/)
- **Audience**: Claude Code and AI assistants
- **Goal**: Maximum information density, minimal tokens
- **Style**: Compact, symbolic, pattern-first, inline
- **When to use**: Working with AI, need quick reference

### Human Docs (docs/)
- **Audience**: Developers, researchers, users
- **Goal**: Clarity, completeness, accessibility
- **Style**: Detailed explanations, examples, walkthroughs
- **When to use**: Onboarding, learning, troubleshooting

**Key Principle**: Never sacrifice human documentation for AI optimization. Both must exist and complement each other.

## What You'll Get

After successful documentation update:
- **Accurate documentation** reflecting current state (both AI and human)
- **Consistent information** across all docs
- **No gaps** - human docs exist for all user-facing features
- **Current examples** that work as written
- **Updated timestamps** on modified docs
- **AI-optimized context** for efficient Claude Code usage
- **Human-friendly guides** for developers and users
- **Improved maintainability** for future changes
- **Better user experience** for new contributors
- **Audit trail** of what changed and why

## Next Steps

- Commit documentation changes
- Archive session notes if significant changes
- Update skills documentation if skills affected
- Share updated docs with collaborators
- Consider creating documentation automation scripts
