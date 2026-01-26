---
name: complete-enhancement
description: Complete a finished enhancement with git commit, status updates, and wave manager sync. Use when an enhancement is fully implemented and tested.
allowed-tools:
  - Read
  - Edit
  - Bash
  - Grep
user-invocable: true
---

# Complete Enhancement

Finalize a completed enhancement by updating status, recording commits, creating git commit, and syncing with wave manager.

## Purpose

Ensures consistent completion workflow for enhancements within waves:
- Update enhancement status to COMPLETED
- Record git commits in enhancement file
- Create structured git commit
- Sync with wave manager
- Track progress toward wave completion

## When to Use

Use when:
- Enhancement fully implemented and tested
- All tasks in enhancement file completed
- Code reviewed and working correctly
- Ready to commit changes to git

## What This Skill Does

### Phase 1: Update Enhancement File (5 min)

**Update status and dates**:
```markdown
**Status**: ✅ COMPLETED
**Completed**: January 24, 2026
```

**Add completion summary**:
```markdown
## Implementation Summary

**Completion Date**: January 24, 2026

**What Was Built**:
- Deliverable 1
- Deliverable 2
- Deliverable 3

**Files Modified**:
- `path/file1.ts` - Description of changes
- `path/file2.py` - Description of changes

**Files Created**:
- `path/new/file.ts` - Purpose

**Testing Results**:
- Test 1: ✓ Passed
- Test 2: ✓ Passed
- Integration test: ✓ Passed
```

### Phase 2: Git Commit (5 min)

**Create structured commit**:
```bash
git add .
git commit -m "Enhancement XX: [Enhancement Name]

[Brief description of what was accomplished]

Changes:
- Change 1
- Change 2
- Change 3

Testing:
- Test description and results

Files modified: X | Files created: Y

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**Record commit SHA**:
- Get commit SHA: `git log -1 --format=%H`
- Update enhancement file:
  ```markdown
  **Commits**: abc123d
  ```

### Phase 3: Wave Manager Sync (2 min)

- Verify enhancement shows COMPLETED status in wave manager
- Check wave progress: X/Y enhancements completed
- Refresh wave manager: http://localhost:5104

## Interactive Prompts

During execution, will ask:

1. **Completion Verification**:
   - "Is the enhancement fully implemented and tested?"
   - "Have you run all necessary tests?"

2. **Files Changed**:
   - "What files were modified?"
   - "What files were created?"

3. **Testing Results**:
   - "What tests did you run?"
   - "Did all tests pass?"

4. **Commit Message**:
   - "Brief description of what was accomplished?"
   - "Any key improvements or metrics to note?"

## Commit Message Format

```
Enhancement XX: [Enhancement Name]

[1-2 sentence description of what was accomplished]

Changes:
- Specific change 1
- Specific change 2
- Specific change 3

Testing:
- Tests run and results

Files modified: X | Files created: Y

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Example Output

After running `/complete-enhancement 1`:

```
✅ Enhancement file updated (status → COMPLETED)
✅ Git commit created (abc123d)
✅ Commit SHA recorded in enhancement file
✅ Wave manager synced

Enhancement 1 complete! 🎉

Wave 1 progress: 1/6 enhancements completed
View at: http://localhost:5104
```

## Git Commit Best Practices

1. **Stage all changes** before committing
2. **Descriptive messages** that explain the "why"
3. **List key changes** in bullet points
4. **Include testing info** so others know it's validated
5. **Record metrics** if applicable (performance improvements, etc.)
6. **Always include** Co-Authored-By line

## What Gets Updated

### Always Updated
- Enhancement file status → ✅ COMPLETED
- Enhancement file completion date
- Enhancement file commits field
- Git repository (new commit)
- Wave manager (status sync)

### Optionally Updated
- Enhancement file with implementation summary
- Enhancement file with testing results
- Enhancement file with files changed list

## Wave Progress Tracking

After completing each enhancement:
- Wave manager shows updated progress (e.g., "2/6 completed")
- Each enhancement's status badge updates
- Wave card shows completion percentage
- Ready to start next enhancement

When all enhancements complete:
- Wave shows as completable
- Use `/complete-wave` to finalize the wave

## Best Practices

1. **Complete enhancements fully** before marking COMPLETED
2. **Test thoroughly** - don't rush to completion
3. **Commit frequently** during work, final commit at end
4. **Descriptive commit messages** help future debugging
5. **Update status immediately** after committing
6. **Check wave manager** to verify sync

## Troubleshooting

**Git commit fails:**
- Check for staged changes: `git status`
- Ensure no merge conflicts
- Verify git configured: `git config user.name`

**Wave manager doesn't update:**
- Refresh browser: http://localhost:5104
- Check file saved: Read enhancement file
- Verify status format: `**Status**: ✅ COMPLETED`

**Can't find enhancement file:**
- Check context/enhancements/ directory
- List files: `ls context/enhancements/`
- Verify file naming: `#_name.md`

## Notes

### Multiple Commits

If enhancement took multiple commits during development:
- List all commit SHAs: `**Commits**: abc123d, def456e, ghi789f`
- Final commit ties everything together
- Earlier commits show incremental progress

### Cross-Repository Changes

If enhancement affects multiple repos (TCM, NHL, Apportionment):
- Commit in each repo separately
- Record all commit SHAs in enhancement file
- Note which repo each commit is in

### Enhancement Dependencies

If next enhancement depends on this one:
- Verify this enhancement fully complete
- Check next enhancement's prerequisites met
- Update wave manager before starting next

## After Completion

1. **Check wave progress**: Visit http://localhost:5104
2. **Start next enhancement**: Select next planned enhancement in wave
3. **Or complete wave**: If all enhancements done, use `/complete-wave`
