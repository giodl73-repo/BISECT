---
name: enhancement-plan
description: Create a new enhancement specification following project patterns. Use when the user requests a new feature, improvement, or system change. Reads ARCHITECTURE.md, CODING_PATTERNS.md, and existing enhancements to maintain consistency.
allowed-tools:
  - Read
  - Write
  - Grep
  - Glob
  - Edit
user-invocable: true
---

# Enhancement Planning Skill

## Overview

This skill creates comprehensive enhancement specifications following the project's established patterns from 18+ previous enhancements. It ensures consistency with existing patterns and provides detailed implementation roadmaps.

## When to Use This Skill

- User says: "I want to add [feature]"
- User says: "Can we improve [component]"
- User says: "Let's plan [enhancement]"
- Any request that requires planning before implementation

## Workflow

### Step 1: Gather Context

Read the following files to understand project structure and patterns:

**Required reading:**
1. `CLAUDE.md` - Project overview, quick reference, recent changes
2. `docs/ENHANCEMENT_WORKFLOW.md` - Detailed 6-phase workflow process (research, planning, implementation, testing, documentation, completion)
3. `docs/ARCHITECTURE.md` - System design, data flow, component relationships
4. `docs/CODING_PATTERNS.md` - Implementation patterns, progress reporting, file naming
5. `docs/enhancements/INDEX.md` - Review similar past enhancements for patterns
6. `docs/enhancements/templates/enhancement_template.md` - Standard template

**Use Grep to find:**
- Similar past enhancements using keywords from user request
- Existing code patterns that relate to the request
- Data format dependencies if working with census data

### Step 2: Analyze Request

Identify:
- **Affected components**: Which parts of the system will change?
- **Data dependencies**: What data files/formats are required?
- **Integration points**: How does this fit into existing pipeline?
- **Similar enhancements**: What patterns can we reuse from past work?
- **Year compatibility**: Does this work for 2000, 2010, 2020 data?

### Step 3: Create Enhancement Specification

Determine next enhancement number by checking `docs/enhancements/INDEX.md`.

Create a new file in `docs/enhancements/active/` following the template from `docs/enhancements/templates/enhancement_template.md`:

```markdown
## Enhancement XX: [Descriptive Name]

**Status**: 📋 PLANNED
**Proposed**: [Current Date]
**Complexity**: [Low (1-2 hours) | Medium (2-4 hours) | Medium-High (4-8 hours) | High (8-15 hours) | Very High (15+ hours)]

### Current State

[Describe what exists today. Be specific about current capabilities and limitations.]

### Goal

[What we want to achieve. Be specific and measurable. Include quantitative goals if applicable.]

### Implementation Plan

**Phase 1: [Name - e.g., Core Implementation]**
- Task 1: [Specific deliverable]
- Task 2: [Specific deliverable]
- Files: `path/to/file.py` - [Purpose]

**Phase 2: [Name - e.g., Pipeline Integration]**
- Task 1: [Specific deliverable]
- Task 2: [Specific deliverable]
- Files: `path/to/file.py` - [Purpose]

**Phase 3: [Name - e.g., Testing & Validation]**
- Task 1: [Testing approach]
- Task 2: [Validation criteria]

**Phase 4: [Name - e.g., Documentation]**
- Update all relevant documentation files

[Continue phases as needed, typically 3-6 phases]

### Files to Modify/Create

**New Files:**
1. `path/to/new/file.py` - Purpose and functionality
2. `path/to/another/file.py` - Purpose and functionality

**Modified Files:**
1. `path/to/existing/file.py` - Specific changes needed
2. `path/to/another/existing.py` - Specific changes needed

### Testing Plan

Follow project testing pattern:

1. **Print-only mode first** - Validate all parameters thread correctly
   ```bash
   python script.py --print-only --year 2020 --version test
   ```

2. **Small state test** - Quick validation (VT or DE, 30 seconds - 2 minutes)
   ```bash
   python script.py --state VT --year 2020 --version test
   ```

3. **Multi-year test** - If year-dependent, test all supported years
   ```bash
   # Test 2020, 2010, and 2000 if applicable
   ```

4. **Full validation** - Spot-check with subset of states
   ```bash
   python script.py --states "VT,DE,AL" --year 2020 --version test
   ```

5. **Quantitative validation** - If applicable, measure improvements
   - Compare metrics before/after (compactness, performance, etc.)
   - Document percentage improvements

### Benefits

[List concrete benefits, quantify when possible]
- Benefit 1: [e.g., "52.8% improvement in Polsby-Popper compactness"]
- Benefit 2: [e.g., "Reduces processing time from 4 hours to 2 hours"]
- Benefit 3: [e.g., "Eliminates manual step in workflow"]

### Success Criteria

- [ ] Criterion 1: [Specific, testable condition]
- [ ] Criterion 2: [Specific, testable condition]
- [ ] Criterion 3: [Specific, testable condition]
- [ ] All tests pass (print-only, small state, full validation)
- [ ] Documentation updated (CHANGELOG, INDEX.md, CLAUDE.md)
- [ ] Code follows patterns in CODING_PATTERNS.md

### Estimated Complexity

**Effort**: [X-Y hours based on phases]
**Risk**: [Low/Medium/High - explain any risks]
**Dependencies**: [List any prerequisites or blocking items]

### Implementation Notes

[Any additional context, gotchas, or important considerations]
```

### Step 4: Update Index and Present to User

After writing the enhancement specification:
1. Add entry to `docs/enhancements/INDEX.md` in the "Planned" section
2. Summarize the key points for the user
3. Highlight any risks or dependencies
4. Get user approval before proceeding to implementation
5. Suggest next step: Use `/enhancement-implement` skill to execute

## Key Patterns to Follow

### From CODING_PATTERNS.md:

**Progress Reporting (STATUS Protocol)**:
```python
position = int(os.environ.get('TQDM_POSITION', '-1'))
if position >= 0:
    print(f"STATUS:{position}:{msg}", flush=True)
```

**Per-Stage Skip Logic**:
```python
if output_file.exists() and not force:
    if is_standalone:
        print(f"[SKIP] Output already exists: {output_file}")
    else:
        report_progress(f"[SKIP] Already exists")
    return
```

**Windows Compatibility**:
- NEVER use Unicode characters in console output (✓, ✗, →, etc.)
- ALWAYS use ASCII: [OK], [FAIL], [WARN], ->, -

**Path Handling**:
- Use `Path` objects from `pathlib`, not string concatenation
- Support year-specific paths: `data/tracts/{year}/{state}_tracts_{year}.parquet`

**State Names**:
- Always lowercase with underscores: `california`, `new_york`

### From Enhancement History:

**Complexity Estimates** (from 18+ past enhancements):
- **Low** (20-60 min): Simple integrations, text annotations
- **Medium** (2-4 hours): New visualizations, modest refactoring
- **Medium-High** (4-8 hours): Multi-file refactoring, new analysis types
- **High** (8-15 hours): New algorithms, major architectural changes
- **Very High** (15+ hours): Multi-year data support, block-level data

**Common Enhancement Types**:
1. **Integration**: Add existing functionality to pipeline (Low)
2. **Visualization**: New maps or charts (Medium)
3. **Analysis**: New metrics or statistics (Medium-High)
4. **Algorithm**: Core redistricting changes (High)
5. **Data**: New census year support (Very High)

## What You'll Get

After completing this skill, you will have:

1. **Complete enhancement specification** in `docs/enhancements/active/XX_name.md`
2. **Entry added** to `docs/enhancements/INDEX.md`
3. **Detailed implementation phases** with specific tasks and files
4. **Testing approach** following project patterns
5. **Success criteria** for validation
6. **User approval** before proceeding to implementation

## Next Steps

After user approves the plan:
- Use `/enhancement-implement` skill to execute the enhancement
- Follow the phases sequentially
- Mark each phase complete using TodoWrite tool
- Update documentation using `/enhancement-document` skill when finished

## Examples

See `docs/enhancements/completed/` for 14 completed enhancement examples:
- [01_compactness_integration.md](../../docs/enhancements/completed/01_compactness_integration.md) - Simple integration
- [07_edge_weighted_bisection.md](../../docs/enhancements/completed/07_edge_weighted_bisection.md) - Algorithm improvement with quantitative validation
- [13_directory_unification.md](../../docs/enhancements/completed/13_directory_unification.md) - Large refactoring
- [18_figure_quality.md](../../docs/enhancements/completed/18_figure_quality.md) - Validation + retry logic
