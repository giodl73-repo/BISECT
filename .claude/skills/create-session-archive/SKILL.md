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

## Overview

Create detailed historical documentation of significant work sessions, capturing not just what was done but WHY decisions were made. These archives serve as valuable context for future development, helping understand the reasoning behind implementation choices.

## Prerequisites

No specific prerequisites. This skill can be used after any significant work session.

## When to Use This Skill

- User says: "Archive this session" or "Document these decisions"
- After completing a major enhancement (Enhancement workflow Phase 6)
- After making significant architectural decisions
- After solving complex technical problems
- After debugging difficult issues
- When establishing new patterns or conventions
- Before major refactoring (document current state)
- Periodically after productive sessions

## What Makes a Good Session Archive

### Content Focus

**DO document**:
- **WHY** decisions were made, not just WHAT was done
- Alternative approaches considered and rejected
- Challenges encountered and how they were solved
- Lessons learned and key insights
- Mistakes made and how they were fixed
- Surprising discoveries or unexpected behaviors
- Context that would help future developers

**DON'T document**:
- Line-by-line code changes (that's what git diffs are for)
- Trivial details easily inferred from code
- Information already in other docs
- Temporary debugging attempts
- Conversation transcripts verbatim

### Structure

Good archives follow this template:
1. **Overview**: Brief summary of what was accomplished
2. **Context**: Why this work was needed
3. **Key Decisions**: Important choices and their rationale
4. **Challenges**: Problems encountered and solutions
5. **Lessons Learned**: Insights for future work
6. **Related Work**: Links to enhancements, commits, issues

## Workflow

### Step 1: Determine Scope

Identify what should be archived:

**After enhancement**:
- Focus on decisions not captured in enhancement doc
- Document unexpected challenges
- Capture alternatives considered
- Note any deviations from plan

**After debugging session**:
- Root cause analysis
- Why fix works
- How to prevent recurrence
- Related issues to watch for

**After architectural decision**:
- Options considered
- Tradeoffs evaluated
- Rationale for choice
- Future implications

**After establishing pattern**:
- Why this pattern
- When to use it
- Examples of correct usage
- Anti-patterns to avoid

### Step 2: Choose Archive Filename

Follow naming convention:
```
../../context/archive/YYYY-MM-DD_topic_description.md
```

**Examples**:
- `2026-01-12_enhancement_7_edge_weighted_bisection.md`
- `2026-01-13_historical_tract_data_pipeline.md`
- `2026-01-14_directory_unification_decisions.md`
- `2026-01-15_metis_connectivity_debugging.md`

**Guidelines**:
- Date: ISO format YYYY-MM-DD
- Topic: Brief identifier (enhancement number, feature name)
- Description: Lowercase with underscores
- Extension: Always `.md` (Markdown)

### Step 3: Write Archive Content

Use this template structure:

```markdown
# [Title]: Brief Description

**Date**: [Month DD, YYYY]
**Context**: [Enhancement N / Bug Fix / Architectural Decision / etc.]
**Duration**: [Time spent on this work]
**Status**: [Completed / In Progress / Blocked]

## Overview

[2-3 paragraph summary of what was accomplished and why it matters]

## Context and Motivation

[Why was this work needed? What problem does it solve?]

## Key Decisions

### Decision 1: [Decision Name]

**Options considered**:
- Option A: [Description]
  - Pros: [...]
  - Cons: [...]
- Option B: [Description]
  - Pros: [...]
  - Cons: [...]

**Choice**: [Chosen option]

**Rationale**: [Why we chose this option]

**Implications**: [What this means for future work]

[Repeat for other major decisions...]

## Implementation Details

[Only include details that provide useful context, not obvious from code]

### Key Technical Insights

- Insight 1: [...]
- Insight 2: [...]

### Code Examples (if illustrative)

\`\`\`python
# Example showing important pattern or technique
# with comments explaining WHY, not just WHAT
\`\`\`

## Challenges and Solutions

### Challenge 1: [Problem Description]

**Attempted solutions**:
1. [First attempt] - Failed because [...]
2. [Second attempt] - Failed because [...]
3. [Final solution] - Worked because [...]

**Key insight**: [What we learned from this]

[Repeat for other challenges...]

## Lessons Learned

1. **[Lesson title]**: [Description and implications]
2. **[Lesson title]**: [Description and implications]
3. **[Lesson title]**: [Description and implications]

## Future Considerations

- [Thing to watch out for]
- [Potential improvements]
- [Related work needed]

## Related Resources

- **Enhancement**: [Enhancement N: Title]
- **Commits**: [git commit hashes]
- **Files Modified**: [Key files changed]
- **Related Archives**: [Links to related archive docs]
- **External References**: [Papers, blog posts, documentation cited]

## Metrics (if applicable)

[Quantitative results: performance improvements, code reduction, etc.]

## Timeline

[Brief timeline of major milestones if relevant]

---

*This archive documents the rationale and context for decisions made during this work. For code changes, see git history. For current status, see [relevant tracking document].*
```

### Step 4: Gather Information

Collect information to include:

**From current session**:
```bash
# Review recent git commits
git log --oneline -20

# See what files changed
git diff --stat HEAD~10..HEAD

# Check enhancement specification
cat ../../context/enhancements/active/enhancement_N.md
```

**From conversations**:
- Key decisions discussed
- Alternatives considered
- User feedback and preferences
- Unexpected discoveries

**From testing**:
- Issues encountered
- Performance measurements
- Before/after comparisons

### Step 5: Write the Archive

**Focus on clarity**:
- Use clear, concise language
- Include code examples only when illuminating
- Link to other documentation rather than repeating
- Organize with clear headings

**Focus on future utility**:
- Write for someone who wasn't in the conversation
- Explain context that might not be obvious later
- Include enough detail to understand rationale
- But not so much that key points are buried

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

```bash
# Create the archive file
# File: ../../context/archive/2026-01-15_session_topic.md
```

**Location**: Always in `../../context/archive/`
**Format**: Markdown (`.md`)
**Naming**: Follow YYYY-MM-DD_topic_description.md pattern

## Archive Types and Examples

### Enhancement Archive

**When**: After completing major enhancement

**Focus**:
- Why enhancement was needed
- Design decisions and tradeoffs
- Implementation challenges
- Quantitative results
- Lessons learned

**Example**: `2026-01-12_enhancement_7_edge_weighted_bisection.md`
```markdown
# Enhancement 7: Edge-Weighted Recursive Bisection

## Key Decisions

### Decision: Use Physical Boundary Lengths as Edge Weights

**Rationale**: Physical distance provides objective, measurable compactness metric. Census tracts with longer shared boundaries are "closer" geographically.

**Alternative considered**: Population-weighted distances
- Rejected because: Less intuitive, harder to interpret, doesn't directly optimize for compactness

**Implementation**: Scale meters to centimeters (integers for METIS)

### Decision: Change Default Mode to Edge-Weighted

**Rationale**: Alabama test showed 52.8% improvement in Polsby-Popper scores with minimal computation overhead (<2x slower)

**Evidence**: Quantitative analysis on representative state
```

### Debugging Archive

**When**: After solving complex bug

**Focus**:
- Symptoms and root cause
- Debugging process
- Why fix works
- How to prevent recurrence

**Example**: `2026-01-15_metis_connectivity_debugging.md`
```markdown
# Debugging METIS Graph Connectivity Failures

## Root Cause

METIS requires all nodes to form single connected component. Our code was only adding nodes that had edges, missing isolated tracts.

## Solution

Explicitly add ALL nodes before adding edges:
\`\`\`python
for i in range(num_nodes):
    graph.add_node(i)  # Even isolated nodes
for i, neighbors in enumerate(adjacency):
    for j in neighbors:
        graph.add_edge(i, j)
\`\`\`

## Prevention

Add connectivity validation step in adjacency building pipeline.
```

### Architectural Decision Record (ADR)

**When**: After significant architecture choice

**Focus**:
- Problem being solved
- Options evaluated
- Decision and rationale
- Consequences and tradeoffs

**Example**: `2026-01-12_scope_based_analysis_pattern.md`
```markdown
# ADR: Scope-Based Analysis Pattern

## Problem

Analysis scripts had parallel per-state and sequential national versions with code duplication.

## Decision

Single script with `--scope state|national` parameter

## Rationale

- Eliminates duplication
- Ensures consistency
- Easier to maintain
- Fits pipeline architecture (per-state parallel, national sequential)

## Consequences

- Must pass scope parameter through pipeline
- Script must handle both cases
- **Benefit**: 300+ minute sequential bottleneck eliminated
```

### Pattern Establishment Archive

**When**: Introducing new coding pattern

**Focus**:
- Why pattern needed
- When to use it
- How to implement
- Examples and anti-patterns

**Example**: `2026-01-10_progress_reporting_protocol.md`
```markdown
# Progress Reporting Protocol (STATUS)

## Rationale

Pipeline runs 50+ parallel processes. Need coordinated progress output without interleaved tqdm bars.

## Pattern

Child processes send STATUS messages to parent:
\`\`\`python
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0:
    print(f"STATUS:{position}:{message}", flush=True)
\`\`\`

Parent collects and updates tqdm bars.

## When to Use

- Any script called from pipeline
- Any parallel processing
- NOT for standalone scripts (use tqdm directly)
```

## Best Practices

### Content Guidelines

1. **Be concise**: Aim for 2-5 pages, not 20
2. **Focus on "why"**: Code shows "what," archives explain "why"
3. **Use examples**: Show, don't just tell
4. **Link liberally**: Reference other docs rather than repeating
5. **Organize clearly**: Use headings and lists
6. **Write for future**: Assume reader doesn't know context

### Maintenance

1. **Don't update archives**: They're historical snapshots
2. **Create new archive**: If situation evolves, write new one and link
3. **Reference in docs**: Link from current docs to relevant archives
4. **Index archives**: Keep ../../context/archive/README.md with index

### Timing

**Best time to create**:
- Right after work completes (while fresh in mind)
- During documentation phase of enhancement workflow
- After user validation (include their feedback)

**Don't wait**:
- Memory fades quickly
- Context gets lost
- Details become fuzzy

## Integration with Enhancement Workflow

### Enhancement Workflow Phase 6: Completion

Archives are part of completion phase:

1. Create git commit
2. **Create session archive** (this skill)
3. Update main documentation
4. Mark enhancement complete

### What Goes Where

**Enhancement spec** (../../context/enhancements/):
- Current status
- Implementation plan
- Success criteria

**Session archive** (../../context/archive/):
- WHY decisions made
- Challenges and solutions
- Lessons learned
- Historical context

**Main docs** (docs/*.md, CLAUDE.md):
- Current state of system
- How to use features
- Coding patterns
- Quick reference

## Common Pitfalls

**Too verbose**:
```
Issue: Archive is 30 pages of detailed conversation transcript
Solution: Summarize key points, link to git commits for details
```

**Too sparse**:
```
Issue: "We did enhancement N. See git log."
Solution: Explain WHY, document decisions, capture insights
```

**Wrong focus**:
```
Issue: Detailed code walkthrough of obvious changes
Solution: Focus on non-obvious decisions and rationale
```

**Not timely**:
```
Issue: Trying to recreate archive weeks later
Solution: Write while work is fresh, even if brief
```

**No future value**:
```
Issue: Archive documents only temporary debugging steps
Solution: Focus on permanent insights and learnings
```

## Automation Helpers

### Archive Template Generator

```bash
# generate_archive.sh
#!/bin/bash
DATE=$(date +%Y-%m-%d)
TOPIC=$1
OUTPUT="../../context/archive/${DATE}_${TOPIC}.md"

cat > $OUTPUT <<'EOF'
# [Title]: [Brief Description]

**Date**: $(date +"%B %d, %Y")
**Context**: [Enhancement N / Bug Fix / etc.]
**Status**: [Completed / In Progress]

## Overview

[Summary of what was accomplished]

## Key Decisions

### Decision 1: [Name]

**Options considered**:
- Option A
- Option B

**Choice**: [...]
**Rationale**: [...]

## Lessons Learned

1. [Lesson]

## Related Resources

- **Enhancement**: [...]
- **Commits**: [...]

EOF

echo "Created: $OUTPUT"
```

### Archive Index Generator

```python
# generate_archive_index.py
from pathlib import Path
import re

archives = sorted(Path('../../context/archive/').glob('*.md'))

for archive in archives:
    # Extract title from first heading
    content = archive.read_text()
    match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
    title = match.group(1) if match else archive.stem

    print(f"- [{title}]({archive.name})")
```

## Related Skills

- `/enhancement-document` - Complete enhancement documentation (includes archiving)
- `/update-docs` - Update current documentation
- `/create-architecture-diagram` - Visual documentation

## What You'll Get

After creating session archive:
- **Historical record** of decisions and rationale
- **Context preservation** for future developers
- **Lessons captured** to avoid repeating mistakes
- **Pattern documentation** showing real-world usage
- **Debugging insights** for similar issues
- **Architectural rationale** explaining system design
- **Knowledge transfer** from this session to future sessions

## Next Steps

- Reference archive from relevant documentation
- Link from enhancement specification
- Update ../../context/archive/README.md index
- Share insights with collaborators
- Use learnings in future work
