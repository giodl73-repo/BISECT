---
name: start-wave
description: Initialize a new development wave with planning document, task structure, and phase breakdown for the App Manager system.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
  - TaskCreate
  - AskUserQuestion
user-invocable: true
---

# Start Wave

Initialize a new development wave for the App Manager system with proper planning, goals, and task structure.

## Purpose

Create structured planning documents for a new wave of development work across the unified app manager system. Each wave represents a cohesive set of changes affecting one or more repositories.

## What This Skill Does

1. **Creates Wave Planning Document** (`context/WAVE##-NAME.md`)
2. **Sets Up Task Structure** via TaskCreate
3. **References Master Plan** and EXECUTE_*.md files
4. **Identifies Dependencies** and risks

## Interactive Prompts

If information is not provided, will prompt for:
1. Wave number (e.g., 1, 2, 3...)
2. Wave name (e.g., "Foundation Setup")
3. Primary goal (measurable)
4. Success metrics (what improves, by how much)
5. Repositories affected (appmanager, TCM, NHL, apportionment)
6. Estimated duration
7. Dependencies

## Wave Document Template

Creates `context/WAVE##-NAME.md` with:

```markdown
# Wave N: [Name]

**Date**: [Start Date]
**Focus**: [Brief description]
**Status**: Planning
**Repositories**: appmanager, [others]
**Estimated Duration**: [X hours/days]

---

## Goals

1. [Primary goal - specific and measurable]
2. [Secondary goals]

---

## Success Metrics

| Metric | Baseline | Target | Actual | Status |
|--------|----------|--------|--------|--------|
| Deploy time | 30s | <5s | - | 🔄 |
| Services running | 0 | 7 | - | 🔄 |

---

## Repositories Affected

- **appmanager**: [What changes]
- **TCM**: [What changes]
- **NHL**: [What changes]
- **apportionment**: [What changes]

---

## Phases (Enhancements)

Each phase maps 1:1 to an enhancement.

### Phase 1: Enhancement 01 - [Name]
**Repository**: [Which repo]
**Reference**: [Link to EXECUTE_*.md section]
**Estimated Effort**: [Hours]

**Tasks**:
- [ ] Task 1
- [ ] Task 2

**Testing**: [Strategy]

**Success Criteria**: [Acceptance criteria]

### Phase 2: Enhancement 02 - [Name]
[Same structure]

---

## Dependencies

- **Prerequisite Waves**: [None or list]
- **External Dependencies**: [Docker, PostgreSQL, etc.]
- **Blocking Issues**: [Any blockers]

---

## Risks & Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| PM2 Windows issues | High | Medium | Test early, have batch script fallback |

---

## Port Configuration

| Service | Port | Notes |
|---------|------|-------|
| [If any ports change] | [Port] | [Why] |

---

## Related Documents

- [MASTER_PLAN.md](../MASTER_PLAN.md)
- [EXECUTE_*.md files for relevant repos]
- [architecture.md](../architecture.md) if architecture changes
```

## Example Usage

```
User: /start-wave
Claude: [Prompts for details, creates wave document, sets up tasks]
```

## After Creating Wave

1. Review wave planning document
2. Create enhancement files for each phase
3. Start with Phase 1
4. Use `/complete-wave` when all phases done
