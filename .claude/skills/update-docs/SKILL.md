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

Systematically review and update all project documentation files to ensure they remain accurate, consistent, and helpful. Performs comprehensive checks across all docs and updates any outdated or inconsistent information.

## Prerequisites
No specific prerequisites required. Can be run anytime to ensure documentation quality.

## When to Use
User says "Update the documentation/Are the docs current?", after completing major enhancement, after changing file structure/paths/commands, periodically (monthly) for maintenance, before releases or major milestones, when adding new features or capabilities, after fixing bugs that affect documentation

## Documentation Structure

**AI Context** (`context/` directory) - Optimized for Claude: Dense, token-efficient documentation for AI consumption
1. ARCHITECTURE.md (system design, compact symbolic notation)
2. CODING_PATTERNS.md (coding conventions, pattern-first)
3. DATA_FORMATS.md (file formats, inline schemas)
4. SKILLS.md (skill catalog, category-based)
5. QUICK_REFERENCE.md (commands, troubleshooting)
6. ENHANCEMENT_WORKFLOW.md (enhancement process)
7. TESTING.md (test system documentation)
8. enhancements/ (enhancement specifications)
9. archive/ (historical session notes)

**Human Documentation** (`docs/` directory) - User-friendly: Comprehensive, readable documentation for developers
1. RECURSIVE_BISECTION.md (algorithm explanation, detailed)
2. DEPENDENCIES.md (installation, setup, detailed)
3. CONTRIBUTING.md (contribution guidelines)
4. CHANGELOG.md (version history, feature tracking)
5. API_REFERENCE.md (code documentation)

**Root Documentation** - Quick access:
1. README.md (project overview, getting started)
2. CLAUDE.md (AI instructions, project context - AI-optimized)

## Workflow

### Step 1: Identify Update Scope
**After enhancement**: Read enhancement spec (check what changed), identify affected docs (files/paths/commands/features), prioritize (critical changes first)

**Periodic maintenance**: Check all docs systematically, look for staleness indicators (dates, version numbers, deprecated info)

**After bug fix**: Update troubleshooting sections, fix incorrect command examples, note workarounds if applicable

### Step 2: Check for Common Issues
**Outdated information**: Old version numbers (check for version refs), deprecated commands (replaced by new), obsolete file paths (after reorganization), stale examples (no longer work), old feature descriptions (features changed)

**Broken references**: Links to non-existent files (`grep -r "\[.*\](.*.md)" context/ docs/`), references to renamed files, broken section anchors (`#section-name`), missing related skills

**Missing updates**: New features not documented, changed commands not updated, new files/directories not listed, test additions not noted, skill additions not cataloged

**Inconsistencies**: Different command syntax in different docs, conflicting information, outdated file counts, inconsistent terminology

### Step 3: Update Core Documentation

**CLAUDE.md** (AI instructions): Update file paths (scripts/*, data/*, outputs/*), update command examples (ensure accurate), update skill list (31 total skills currently), update recent changes section (last 3-5 updates), verify coding patterns section (reference CODING_PATTERNS.md), update quick reference (common commands/troubleshooting)

**README.md** (user-facing): Update quick start commands (ensure work), update feature list (new capabilities), update installation steps (if dependencies changed), verify example outputs (screenshots/descriptions), update contribution guidelines (link to CONTRIBUTING.md)

**CHANGELOG.md** (version history): Add new enhancement entries, note major changes (API breaking/non-breaking), document bug fixes, update version numbers, maintain reverse chronological order

### Step 4: Update AI Context Docs

**context/ARCHITECTURE.md**: Update system diagrams (Mermaid), note architectural changes, update component descriptions, verify file structure diagrams

**context/CODING_PATTERNS.md**: Add new patterns (if established), update pattern examples (with real file references), note deprecated patterns

**context/SKILLS.md**: Update skill count (currently 31), add new skills (name + description), update skill categories, verify all skills documented

**context/QUICK_REFERENCE.md**: Update command examples, add new troubleshooting items, update file paths, note new shortcuts

**context/ENHANCEMENT_WORKFLOW.md**: Update workflow steps (if changed), note new phases/requirements, update example enhancements

**context/TESTING.md**: Update test counts (unit/integration/e2e), note new test types, update test commands, document new fixtures

### Step 5: Update Human Docs

**docs/RECURSIVE_BISECTION.md**: Update algorithm description (if changed), add new optimization techniques, update performance metrics, note edge cases discovered

**docs/DEPENDENCIES.md**: Update Python version (3.13+), update package versions (requirements.txt), add new dependencies, note installation changes

**docs/CONTRIBUTING.md**: Update workflow (if changed), note new coding standards, update PR checklist, add new testing requirements

**docs/API_REFERENCE.md** (if exists): Update function signatures, add new public APIs, deprecate old APIs, update examples

### Step 6: Verify Cross-References
**Check internal links**: `grep -r "\[.*\](.*\.md)" context/ docs/ | grep -v "^Binary"`, verify all linked files exist, update broken links, fix anchors to renamed sections

**Check command examples**: Extract all code blocks with bash/python commands, verify commands work (run or check syntax), update outdated commands, note deprecated usage

**Check file path references**: `grep -r "scripts/" context/ docs/ | grep -v "^Binary"`, verify referenced files exist, update moved/renamed files, note new file locations

### Step 7: Update Metrics and Counts
**Skill counts**: Total skills (31 currently), skills per category (verify SKILLS.md), user-invocable skills

**Test counts**: Total tests (unit/integration/e2e), test runtime (~18s currently), coverage metrics

**File counts**: Scripts (by category), data files (by year), output directories

**Performance metrics**: Pipeline runtime (single year ~1h, multi-year ~2-4h), test suite runtime, state processing times

### Step 8: Verify Formatting
**Markdown formatting**: Headers properly nested, code blocks with language tags, lists properly formatted, tables aligned correctly

**Consistency**: Consistent terminology (redistricting vs apportionment), consistent code style (inline vs blocks), consistent file path format (use Path notation), consistent command format (bash blocks)

**Readability**: Proper line wrapping, clear section divisions, informative headers, concise descriptions

### Step 9: Generate Update Summary
Document: **Files updated** (list all modified docs), **Major changes** (significant updates), **Minor changes** (typo fixes, formatting), **Verification status** (links checked, commands verified), **Remaining issues** (known doc gaps, future updates needed)

### Step 10: Commit Documentation Changes
**Commit message format**:
```
Update documentation (post-Enhancement N)

- Updated CLAUDE.md: new skill list, updated commands
- Updated CHANGELOG.md: Enhancement N entry
- Updated context/SKILLS.md: added skill X
- Fixed broken links in README.md
- Updated test counts in TESTING.md
```

## Update Checklist

**Core docs**: [ ] CLAUDE.md (AI instructions current), [ ] README.md (user-facing accurate), [ ] CHANGELOG.md (recent changes documented)

**AI context**: [ ] ARCHITECTURE.md (system design current), [ ] CODING_PATTERNS.md (patterns documented), [ ] SKILLS.md (all skills cataloged), [ ] QUICK_REFERENCE.md (commands work)

**Human docs**: [ ] DEPENDENCIES.md (installations accurate), [ ] CONTRIBUTING.md (workflow current), [ ] Algorithm docs (accurate)

**Cross-references**: [ ] Internal links work, [ ] File paths exist, [ ] Commands verified, [ ] Examples work

**Metrics**: [ ] Skill counts correct, [ ] Test counts current, [ ] Performance metrics accurate, [ ] File counts updated

## Common Update Patterns

**After enhancement**: Update CLAUDE.md (recent changes), update CHANGELOG.md (add entry), update SKILLS.md (if new skill), update relevant docs (feature-specific)

**After refactoring**: Update file paths (moved scripts/data), update command examples (changed invocations), update ARCHITECTURE.md (if structure changed), update CODING_PATTERNS.md (if patterns changed)

**After directory reorganization**: Update all file path references (grep for old paths), update ARCHITECTURE.md (directory structure diagrams), update README.md (file locations), verify all links work

**Periodic maintenance**: Check dates (update stale timestamps), verify counts (skills/tests/files), test commands (ensure work), check links (no 404s), update metrics (performance numbers)

## Troubleshooting

**Can't find where to update**: Use grep to find references (`grep -r "old_info" context/ docs/`), check CLAUDE.md first (central doc), check CHANGELOG.md for history, review recent enhancements (context/enhancements/)

**Conflicting information**: Determine source of truth (code > docs), update all instances consistently, note why conflict occurred (prevent recurrence)

**Too many updates needed**: Prioritize critical docs (CLAUDE.md, README.md, CHANGELOG.md), batch related updates, create tracking issues for minor updates, document known gaps for later

**Broken links after reorganization**: Run comprehensive link check (`grep -r "\[.*\](.*\.md)"`), create mapping of old→new paths, update systematically, test all critical links

## Performance Notes
**Typical time**: Quick update (single file, 5-10 min), standard update (3-5 files after enhancement, 15-30 min), comprehensive update (all docs, periodic, 1-2h)
**What takes time**: Finding all references (30%), updating content (40%), verifying accuracy (20%), testing examples (10%)

## What You'll Get
Current documentation (accurate information), fixed broken references (working links/commands), updated metrics (correct counts/performance), consistent information (no conflicts), verified examples (commands work), maintenance tracking (documented in CHANGELOG.md)

## Next Steps
Commit documentation changes (with descriptive message), notify team (if collaborative), monitor for feedback (user-reported issues), schedule next review (monthly maintenance), update enhancement tracker (mark documentation complete)

## Related Skills
`/enhancement-document` (document completed enhancements), `/create-session-archive` (archive major decisions), `/create-architecture-diagram` (update architecture visuals), `/reorganize-directory-structure` (reorganization triggers doc updates)
