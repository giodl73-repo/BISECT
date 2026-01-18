---
name: create-session-archive
description: Create comprehensive historical documentation of significant work sessions, capturing rationale, decisions, challenges, and solutions. Archives go to ../../context/archive/ for future reference. Use after major enhancements or when significant decisions are made.
allowed-tools:
  - Read
  - Write
  - Glob
  - Grep
user-invocable: true
---

# Create Session Archive

Create detailed historical documentation of significant work sessions, capturing not just what was done but WHY decisions were made. These archives serve as valuable context for future development, helping understand reasoning behind implementation choices.

## Prerequisites
No specific prerequisites. Can be used after any significant work session.

## When to Use
User says "Archive this session/Document these decisions", after completing major enhancement (Enhancement workflow Phase 6), after making significant architectural decisions, after solving complex technical problems, after debugging difficult issues, when establishing new patterns/conventions, before major refactoring (document current state), periodically after productive sessions

## What Makes a Good Session Archive

**DO document**: WHY decisions were made (not just WHAT), alternative approaches considered and rejected, challenges encountered + solutions, lessons learned + key insights, mistakes made + fixes, surprising discoveries or unexpected behaviors, context that would help future developers

**DON'T document**: Line-by-line code changes (that's what git diffs are for), trivial details easily inferred from code, information already in other docs, temporary debugging attempts, conversation transcripts verbatim

**Structure**: (1) Overview - brief summary, (2) Context - why work needed, (3) Key Decisions - important choices + rationale, (4) Challenges - problems + solutions, (5) Lessons Learned - insights for future, (6) Related Work - links to enhancements/commits/issues

## Workflow

### Step 1: Determine Scope
**After enhancement**: Decisions not captured in enhancement doc, unexpected challenges, alternatives considered, deviations from plan
**After debugging**: Root cause analysis, why fix works, how to prevent recurrence, related issues
**After architectural decision**: Options considered, tradeoffs evaluated, rationale for choice, future implications
**After establishing pattern**: Why this pattern, when to use it, examples of correct usage, anti-patterns to avoid

### Step 2: Choose Archive Filename
```
../../context/archive/YYYY-MM-DD_topic_description.md
```

**Examples**: `2026-01-12_enhancement_7_edge_weighted_bisection.md`, `2026-01-13_historical_tract_data_pipeline.md`, `2026-01-14_directory_unification_decisions.md`, `2026-01-15_metis_connectivity_debugging.md`

**Guidelines**: Date (ISO format YYYY-MM-DD), Topic (brief identifier - enhancement number/feature name), Description (lowercase with underscores), Extension (always .md)

### Step 3: Write Archive Content
Use this template structure:
```markdown
# [Title]: Brief Description

**Date**: [Month DD, YYYY]
**Context**: [Enhancement N / Bug Fix / Architectural Decision / etc.]
**Duration**: [Time spent]
**Status**: [Completed / In Progress / Blocked]

## Overview
[2-3 paragraph summary of what was accomplished and why it matters]

## Context and Motivation
[Why was this work needed? What problem does it solve?]

## Key Decisions

### Decision 1: [Decision Name]
**Options considered**:
- Option A: [Description] - Pros: [...] - Cons: [...]
- Option B: [Description] - Pros: [...] - Cons: [...]

**Choice**: [Chosen option]
**Rationale**: [Why we chose this option]
**Implications**: [What this means for future work]

## Implementation Details
[Only include details that provide useful context, not obvious from code]

### Key Technical Insights
- Insight 1: [...]
- Insight 2: [...]

## Challenges and Solutions

### Challenge 1: [Problem Description]
**Attempted solutions**:
1. [First attempt] - Failed because [...]
2. [Second attempt] - Failed because [...]
3. [Final solution] - Worked because [...]

**Key insight**: [What we learned from this]

## Lessons Learned
1. **[Lesson title]**: [Description and implications]
2. **[Lesson title]**: [Description and implications]

## Future Considerations
- [Thing to watch out for]
- [Potential improvements]
- [Related work needed]

## Related Resources
- **Enhancement**: [Enhancement N: Title]
- **Commits**: [git commit hashes]
- **Files Modified**: [Key files changed]
- **Related Archives**: [Links to related archive docs]

## Metrics (if applicable)
[Quantitative results: performance improvements, code reduction, etc.]

---
*This archive documents the rationale and context for decisions made during this work. For code changes, see git history.*
```

### Step 4: Gather Information
**From current session**: `git log --oneline -20`, `git diff --stat HEAD~10..HEAD`, `cat ../../context/enhancements/active/enhancement_N.md`
**From conversations**: Key decisions discussed, alternatives considered, user feedback/preferences, unexpected discoveries
**From testing**: Issues encountered, performance measurements, before/after comparisons

### Step 5: Write the Archive
**Focus on clarity**: Use clear concise language, include code examples only when illuminating, link to other documentation rather than repeating, organize with clear headings
**Focus on future utility**: Write for someone who wasn't in conversation, explain context that might not be obvious later, include enough detail to understand rationale, but not so much that key points are buried

### Step 6: Review and Refine
**Self-review checklist**:
- [ ] Does it explain WHY, not just WHAT?
- [ ] Would this help someone understand decisions later?
- [ ] Are all major decisions documented?
- [ ] Are lessons learned clearly stated?
- [ ] Is it organized and readable?
- [ ] Are links to related docs included?
- [ ] Is it concise (not a transcript)?

### Step 7: Save to ../../context/archive/
**Location**: Always in `../../context/archive/`
**Format**: Markdown (`.md`)
**Naming**: Follow YYYY-MM-DD_topic_description.md pattern

## Archive Types and Examples

**Enhancement Archive** (after completing major enhancement): Focus on why enhancement needed, design decisions + tradeoffs, implementation challenges, quantitative results, lessons learned
Example: `2026-01-12_enhancement_7_edge_weighted_bisection.md` - Key decision: Use physical boundary lengths as edge weights (rationale: objective measurable compactness metric)

**Debugging Archive** (after solving complex bug): Focus on symptoms + root cause, debugging process, why fix works, how to prevent recurrence
Example: `2026-01-15_metis_connectivity_debugging.md` - Root cause: METIS requires connected component, code only added nodes with edges (missing isolated tracts)

**Architectural Decision Record (ADR)** (after significant architecture choice): Focus on problem being solved, options evaluated, decision + rationale, consequences + tradeoffs
Example: `2026-01-12_scope_based_analysis_pattern.md` - Decision: Single script with `--scope state|national` parameter (eliminates duplication, ensures consistency)

**Pattern Establishment Archive** (introducing new coding pattern): Focus on why pattern needed, when to use it, how to implement, examples + anti-patterns
Example: `2026-01-10_progress_reporting_protocol.md` - STATUS protocol for coordinated progress output in parallel pipeline

## Best Practices

**Content Guidelines**: Be concise (2-5 pages, not 20), focus on "why" (code shows "what"), use examples (show don't tell), link liberally (reference rather than repeating), organize clearly (headings + lists), write for future (assume reader doesn't know context)

**Maintenance**: Don't update archives (they're historical snapshots), create new archive (if situation evolves, write new one + link), reference in docs (link from current docs to relevant archives), index archives (keep ../../context/archive/README.md with index)

**Timing**: Best time to create (right after work completes while fresh, during documentation phase of enhancement workflow, after user validation including feedback), don't wait (memory fades quickly, context gets lost, details become fuzzy)

## Integration with Enhancement Workflow

**Enhancement Workflow Phase 6: Completion**: Archives are part of completion phase → (1) Create git commit, (2) Create session archive (this skill), (3) Update main documentation, (4) Mark enhancement complete

**What Goes Where**:
- **Enhancement spec** (../../context/enhancements/): Current status, implementation plan, success criteria
- **Session archive** (../../context/archive/): WHY decisions made, challenges + solutions, lessons learned, historical context
- **Main docs** (docs/*.md, CLAUDE.md): Current state of system, how to use features, coding patterns, quick reference

## Common Pitfalls
**Too verbose**: Archive is 30 pages of detailed conversation transcript → Summarize key points, link to git commits for details
**Too sparse**: "We did enhancement N. See git log." → Explain WHY, document decisions, capture insights
**Wrong focus**: Detailed code walkthrough of obvious changes → Focus on non-obvious decisions and rationale
**Not timely**: Trying to recreate archive weeks later → Write while work is fresh, even if brief
**No future value**: Archive documents only temporary debugging steps → Focus on permanent insights and learnings

## What You'll Get
Historical record of decisions + rationale, context preservation for future developers, lessons captured to avoid repeating mistakes, pattern documentation showing real-world usage, debugging insights for similar issues, architectural rationale explaining system design, knowledge transfer from this session to future sessions

## Next Steps
Reference archive from relevant documentation, link from enhancement specification, update ../../context/archive/README.md index, share insights with collaborators, use learnings in future work

## Related Skills
`/enhancement-document` (complete enhancement documentation including archiving), `/update-docs` (update current documentation), `/create-architecture-diagram` (visual documentation)
