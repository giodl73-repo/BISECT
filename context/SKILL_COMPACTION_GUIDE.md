# Skill Compaction Guide

Guide for compacting Claude Code skills to AI-optimized format (40-60% reduction while preserving 100% information).

## Compaction Rules

### ✅ DO
- **Use symbols**: →, ✅/❌, ⚠️, •, **bold**
- **Inline lists**: `File1, File2, File3` not numbered/bulleted
- **Pattern-first**: Show code with minimal prose
- **Abbreviations**: impl/docs/config/deps/reqs
- **Compact headers**: ## Step 1: Context (not ### Step 1: Gather Context and Review Documentation)
- **Inline descriptions**: `file.py` - Purpose (not separate bullets)
- **Combine sections**: Merge redundant/similar sections

### ❌ DON'T
- **Lose information**: Every fact must remain
- **Sacrifice clarity**: Must be understandable
- **Remove examples**: Keep code samples (compact them)
- **Skip edge cases**: Document gotchas/warnings

## Before → After Examples

### Example 1: File Lists

**Before** (Verbose):
```markdown
### Step 1: Gather Context

Read the following files to understand project structure:

**Required reading:**
1. `CLAUDE.md` - Project overview, quick reference, recent changes
2. `ENHANCEMENT_WORKFLOW.md` - 6-phase workflow process
3. `ARCHITECTURE.md` - System design, data flow
4. `CODING_PATTERNS.md` - Implementation patterns
5. `INDEX.md` - Past enhancements

**Use Grep to find:**
- Similar past enhancements using keywords
- Existing code patterns that relate to request
- Data format dependencies
```

**After** (Compact):
```markdown
## Step 1: Context
**Read**: CLAUDE.md, ENHANCEMENT_WORKFLOW.md, ARCHITECTURE.md, CODING_PATTERNS.md, INDEX.md
**Grep**: Similar enhancements (keywords), existing patterns, data dependencies
```

**Reduction**: 15 lines → 3 lines (80% reduction)

### Example 2: Workflow Steps

**Before** (Verbose):
```markdown
### Step 2: Analyze Request

Carefully analyze the user's request to understand:

- **Affected components**: Which parts of the system will be modified or extended?
- **Data dependencies**: What data files, formats, or external sources are required?
- **Integration points**: How does this feature integrate with the existing pipeline?
- **Similar enhancements**: Are there past enhancements we can learn from?
- **Year compatibility**: Does this work for 2000, 2010, and 2020 census data?

Take time to understand the full scope before proceeding to planning.
```

**After** (Compact):
```markdown
## Step 2: Analyze
**Identify**: Affected components, data deps, integration points, similar enhancements
**Check**: Year compatibility (2000/2010/2020)
```

**Reduction**: 12 lines → 3 lines (75% reduction)

### Example 3: Code Patterns

**Before** (Verbose):
```markdown
### Progress Reporting Pattern

When your script runs as part of the pipeline, it must use the STATUS protocol for progress reporting. This allows the parent process to coordinate progress bars across all running scripts.

Here's how to implement it:

```python
# At the top of your script
position = int(os.environ.get('TQDM_POSITION', '-1'))

# When reporting progress
if position >= 0:
    print(f"STATUS:{position}:{message}", flush=True)
else:
    print(message)  # Standalone mode
```

The parent process sets TQDM_POSITION to coordinate multiple progress bars.
```

**After** (Compact):
```markdown
### Progress (STATUS Protocol)
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)  # Pipeline mode
else: print(msg)  # Standalone
```
**Parent sets TQDM_POSITION** to coordinate progress bars
```

**Reduction**: 15 lines → 6 lines (60% reduction)

### Example 4: Testing Instructions

**Before** (Verbose):
```markdown
### Testing Plan

Follow the standard project testing pattern to validate your changes:

1. **Print-only mode first** - Validate that all parameters thread correctly through the pipeline without actually executing
   ```bash
   python script.py --print-only --year 2020 --version test
   ```

2. **Small state test** - Quick validation using Vermont or Delaware (30 seconds - 2 minutes)
   ```bash
   python script.py --state VT --year 2020 --version test
   ```

3. **Multi-year test** - If your changes are year-dependent, test with all supported census years
   ```bash
   python script.py --year 2000 --version test
   python script.py --year 2010 --version test
   python script.py --year 2020 --version test
   ```

4. **Full validation** - Spot-check with a subset of states representing different district counts
   ```bash
   python script.py --states "VT,DE,AL" --year 2020 --version test
   ```
```

**After** (Compact):
```markdown
### Testing
1. **Print-only**: `script.py --print-only --year 2020 --version test`
2. **Small state** (VT/DE, 30s-2m): `--state VT --year 2020 --version test`
3. **Multi-year** (if year-dependent): Test 2000/2010/2020
4. **Full validation** (subset): `--states "VT,DE,AL" --year 2020 --version test`
```

**Reduction**: 21 lines → 5 lines (76% reduction)

## Compaction Checklist

For each skill file:

- [ ] Replace verbose headers with compact ones (### Step 1: Gather Context → ## Step 1: Context)
- [ ] Convert numbered/bulleted lists to inline format where possible
- [ ] Merge redundant sections
- [ ] Use symbols (→, ✅/❌) instead of words (then, success/failure)
- [ ] Abbreviate common words (implementation → impl, documentation → docs)
- [ ] Inline file descriptions (not separate bullets)
- [ ] Compact code examples (remove comments that state the obvious)
- [ ] Combine similar steps
- [ ] Remove redundant explanations
- [ ] **Verify no information lost** - Every fact must remain

## Target Metrics

- **Line reduction**: 40-60% fewer lines
- **Information preservation**: 100% (no facts lost)
- **Readability**: Still clear and understandable
- **Token efficiency**: Maximized for AI consumption

## Batch Processing

To compact all skills at once:

```bash
# Create backup
cp -r .claude/skills .claude/skills.backup

# Apply compaction pattern to each skill
# (Script TBD - manual for now)
```

## Testing Compacted Skills

After compacting, verify:
1. Skill still loads in Claude Code
2. Skill description is clear
3. Workflow steps are understandable
4. Code examples are correct
5. No information was lost

## Next Steps

1. Apply pattern to high-priority skills first:
   - enhancement-plan, enhancement-implement, enhancement-document
   - run-redistricting, run-tests, debug-tests
   - pipeline-debug
2. Then remaining skills
3. Test all skills after compaction
4. Commit changes

## See Also

- `context/ARCHITECTURE.md` - Example of AI-optimized documentation
- `context/CODING_PATTERNS.md` - Compact pattern-first format
- `context/DATA_FORMATS.md` - Inline schema format
