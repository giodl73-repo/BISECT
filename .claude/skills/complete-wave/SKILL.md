---
name: complete-wave
description: Finalize a completed wave with documentation updates, metrics recording, architecture diagram updates, archival, and git commit.
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
  - TaskUpdate
  - TaskList
  - AskUserQuestion
user-invocable: true
---

# Complete Wave

Finalize a completed wave with documentation updates, metrics recording, architecture diagrams, archival, and git commit.

## Purpose

Close out a development wave by updating all documentation, recording achievements, updating diagrams, organizing artifacts, and committing changes to git.

## When to Use

Use when:
- All phase tasks (enhancements) complete
- All tests passing
- Features working as expected
- Ready to commit and move to next wave

## What This Skill Does

### Phase 1: Wave Summary (15-20 min)
- Update wave document: Status → "Complete"
- Fill in "Actual" column for all success metrics
- Calculate % improvement/change
- Document lessons learned
- Verify all tasks complete in TaskList

### Phase 2: Architecture Diagram Updates (15-30 min)
**IMPORTANT**: Always check if architecture changed and update ALL diagrams

**Diagram Update Process**:
1. **Review what changed**: Identify all architectural changes made in the wave
2. **Update existing diagrams**: Modify diagrams in architecture.md to reflect changes
3. **Create new diagrams**: Generate new diagrams for significant features added
4. **Before/after comparison**: Create side-by-side diagrams if architecture significantly evolved

**Specific Diagram Types to Check**:
- **Port Diagram**: Update if any port assignments changed
- **High-Level Architecture**: Update if services added/removed or data flows changed
- **Component Architecture**: Update if new components or integrations added
- **Deployment Architecture**: Update if deployment topology changed
- **Database Schema**: Update if data models changed
- **Sequence Diagrams**: Create for complex new workflows

**Diagram Storage**:
- Save new diagrams to `figures/wave##/` directory
- Update references in architecture.md
- Include diagram descriptions explaining what changed and why

### Phase 3: Documentation Updates (20-30 min)
**IMPORTANT**: Every wave MUST update all relevant documentation to reflect progress made

**Core Documentation** (Always review and update):
- Update `MASTER_PLAN.md` - Mark phase complete, update progress indicators
- Update `context/README.md` - Index the new wave, update wave status badges
- Update `README.md` - Add new features/capabilities to feature list
- Update `architecture.md` - Document architectural changes (from Phase 2)
- Update `CLAUDE.md` - Add new patterns, update Quick Reference if commands changed
- Update `coding_patterns.md` - Document new code patterns established

**Project-Specific Documentation** (Update if applicable):
- Update relevant `EXECUTE_*.md` files if procedures changed
- Update `deployment.md` if deployment process changed
- Update `testing.md` if testing strategies changed
- Update `troubleshooting.md` if new issues/solutions discovered

**Enhancement Documentation** (Always do):
- Verify all enhancement files have Implementation Summary sections
- Verify all enhancement files have commit SHAs recorded
- Update wave document with links to all completed enhancements

### Phase 4: Wave Manager Updates (10 min)
- Ensure all enhancement files updated with commits
- Update enhancement status to "Completed"
- Verify wave manager (port 5104) shows correct data
- Check integration with appmanager wave manager (port 5101)

### Phase 5: Archive & Cleanup (10-15 min)
- Move temporary scripts → `scripts/archive/wave##/`
- Move working docs → `context/archive/wave##/`
- Delete debugging outputs
- Delete backup files
- Delete test artifacts
- Update references in wave doc

### Phase 6: Git Commit (5 min)
- Stage all changes across affected repositories
- Create structured commit message
- Push to remote

## Completion Checklist

Before running `/complete-wave`:

### Development Complete
- [ ] Wave document shows all enhancements complete
- [ ] All TaskList tasks marked completed
- [ ] All tests passing
- [ ] Features working in all affected repos
- [ ] No temporary debug code left
- [ ] All TODOs addressed or documented

### Documentation Complete
- [ ] All enhancement files have Implementation Summary
- [ ] All enhancement files have commit SHAs recorded
- [ ] Architecture diagrams updated to reflect changes
- [ ] New diagrams created for significant features
- [ ] MASTER_PLAN.md updated with phase completion
- [ ] README.md updated with new capabilities
- [ ] CLAUDE.md updated with new patterns/commands
- [ ] architecture.md updated with changes
- [ ] All relevant EXECUTE_*.md files updated
- [ ] coding_patterns.md updated with new patterns

### Wave Manager Complete
- [ ] All enhancements show COMPLETED status
- [ ] Wave progress shows 100% complete
- [ ] Enhancement commit SHAs recorded
- [ ] Wave manager accessible at http://localhost:5104

## Git Commit Format

```
Wave ##: [Wave Name]

[Brief description of what was accomplished]

Repositories:
- appmanager: [Changes summary]
- TCM: [Changes summary]
- NHL: [Changes summary]
- apportionment: [Changes summary]

Metrics:
- [Metric 1]: [Baseline] → [Actual] ([% improvement])
- [Metric 2]: [Baseline] → [Actual] ([% improvement])

Architecture Changes:
- [Port changes, service additions, etc.]

Files:
- [X files modified]
- [Y files added]
- [Z files archived]

Wave Manager: http://localhost:5104 (view wave details)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Interactive Prompts

During execution, will ask:

1. **Architecture Assessment**:
   - "Did the architecture change?" (ports, services, data flow)
   - If yes: "What changed? Update architecture.md now."

2. **Diagram Creation**:
   - "Do we need before/after architecture diagrams?"
   - If yes: "Describe changes for diagram generation"

3. **Documentation Updates**:
   - "Which EXECUTE_*.md files changed?"
   - "Did Claude workflows change?"

4. **Multi-Repo Commits**:
   - "Commit order: appmanager first, then dependencies?"
   - "Document commit SHAs for cross-reference?"

5. **Archive Strategy**:
   - "Any temporary scripts to archive?"
   - "What can be deleted?"

6. **Lessons Learned**:
   - "What went well?"
   - "What would you do differently?"
   - "Any patterns to reuse?"

## What Gets Updated

### Always Updated
- `context/WAVE##-NAME.md` - Status and metrics
- `context/README.md` - Wave index
- `MASTER_PLAN.md` - Phase completion
- `architecture.md` - If architecture changed
- Wave Manager (port 5104) - Enhancement status
- Git repository - New commit

### Conditionally Updated
- `README.md` - If features/capabilities changed
- `claude.md` - If Claude instructions changed
- `coding_patterns.md` - If new patterns established
- `EXECUTE_*.md` - If procedures changed
- `figures/wave##/` - If diagrams created

## Architecture Diagram Update Guide

When architecture changes:

### Port Changes
Update port alignment table in architecture.md

### Service Additions
Update high-level diagram with new service boxes, ports, connections

### Data Flow Changes
Update component architecture section with new integration points

### Before/After Diagrams
Create figures/wave##/architecture-before.png and architecture-after.png when significant changes made

## Example Output

After running `/complete-wave`:

```
✅ Wave document updated
✅ Architecture updated
✅ Documentation updated
✅ Wave Manager updated
✅ Archive organized
✅ Git committed

Wave 1 complete! 🎉
Ready to start Wave 2 with /start-wave
```

## Cross-Repository Coordination

When wave affects multiple repos:

1. **Commit Order**: appmanager first, then dependent repos
2. **Testing**: Test each repo independently, then integration
3. **Documentation**: Primary wave doc in appmanager

## Best Practices

1. Complete all enhancements before marking wave complete
2. Always check if architecture diagrams need updating
3. Document deviations from targets
4. Preserve knowledge in lessons learned
5. Clean archive - delete what's not needed
6. Test before commit
7. Keep Wave Manager in sync
