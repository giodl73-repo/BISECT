---
name: enhancement-plan
description: Create enhancement specification following project patterns. Reads ARCHITECTURE.md, CODING_PATTERNS.md, existing enhancements for consistency.
allowed-tools:
  - Read
  - Write
  - Grep
  - Glob
  - Edit
user-invocable: true
---

# Enhancement Planning

Creates comprehensive enhancement specs following patterns from 26+ past enhancements.

## When to Use
User requests: "add [feature]", "improve [component]", "plan [enhancement]"

## Workflow

### Step 1: Context
**Read**: CLAUDE.md, context/ENHANCEMENT_WORKFLOW.md, context/ARCHITECTURE.md, context/CODING_PATTERNS.md, context/enhancements/INDEX.md, context/enhancements/templates/enhancement_template.md
**Grep**: Similar enhancements (keywords), existing patterns, data deps

### Step 2: Analyze
**Identify**: Affected components, data deps, integration points, similar enhancements
**Check**: Year compatibility (2000/2010/2020)

### Step 3: Create Spec
**Get next number**: Check `context/enhancements/INDEX.md`
**Create**: `context/enhancements/active/XX_name.md` following template

**Template structure**:
```markdown
## Enhancement XX: [Name]
**Status**: 📋 PLANNED | **Proposed**: [Date] | **Complexity**: [Low|Medium|Medium-High|High|Very High]

### Current State
[What exists. Be specific about capabilities/limitations.]

### Goal
[What we want. Specific, measurable, quantifiable.]

### Implementation Plan
**Phase 1: [Name]** • Task 1 • Task 2 • Files: `path/file.py` - purpose
**Phase 2: [Name]** • Task 1 • Task 2 • Files: `path/file.py` - purpose
**Phase 3: Testing** • Print-only • Small state • Full validation
**Phase 4: Docs** • Update CHANGELOG, INDEX.md, CLAUDE.md
[3-6 phases typical]

### Files to Modify/Create
**New**: `path/new.py` - purpose, `path/another.py` - purpose
**Modified**: `path/existing.py` - changes, `path/other.py` - changes

### Testing Plan
1. **Print-only**: `script.py --print-only --year 2020 --version test`
2. **Small state** (VT/DE, 30s-2m): `--state VT --year 2020 --version test`
3. **Multi-year** (if year-dependent): Test 2000/2010/2020
4. **Full validation** (subset): `--states "VT,DE,AL" --year 2020 --version test`
5. **Quantitative** (if applicable): Compare metrics before/after, document % improvements

### Benefits
• [Quantified benefit 1: "52.8% improvement in X"]
• [Quantified benefit 2: "Reduces time from 4h → 2h"]
• [Benefit 3: "Eliminates manual step"]

### Success Criteria
- [ ] [Specific testable condition]
- [ ] All tests pass (print-only, small state, full)
- [ ] Docs updated (CHANGELOG, INDEX.md, CLAUDE.md)
- [ ] Code follows CODING_PATTERNS.md

### Estimated Complexity
**Effort**: [X-Y hours] | **Risk**: [Low/Medium/High + explain] | **Deps**: [Prerequisites/blockers]

### Implementation Notes
[Context, gotchas, important considerations]
```

### Step 4: Update & Present
1. Add entry to `context/enhancements/INDEX.md` (Planned section)
2. Summarize key points for user
3. Highlight risks/deps
4. Get user approval
5. Suggest: `/enhancement-implement` to execute

## Key Patterns

### STATUS Protocol (CODING_PATTERNS.md)
```python
pos = int(os.environ.get('TQDM_POSITION', '-1'))
if pos >= 0: print(f"STATUS:{pos}:{msg}", flush=True)
```

### Per-Stage Skip Logic
```python
if output_file.exists() and not force:
    report_progress("[SKIP] Already exists") if not is_standalone else print("[SKIP]...")
    return
```

### Windows Compatibility
**Never**: Unicode (✓, ✗, →) | **Always**: ASCII ([OK], [FAIL], [WARN], ->, -)

### Path Handling
**Use**: `Path` objects from `pathlib` (not string concat)
**Support**: Year-specific paths `data/tracts/{year}/{state}_tracts_{year}.parquet`

### State Names
**Always**: lowercase with underscores (`california`, `new_york`)

## Complexity Estimates (from 26+ enhancements)
• **Low** (20-60m): Simple integrations, text annotations
• **Medium** (2-4h): New visualizations, modest refactoring
• **Medium-High** (4-8h): Multi-file refactoring, new analysis types
• **High** (8-15h): New algorithms, major architectural changes
• **Very High** (15+h): Multi-year data support, block-level data

## Common Enhancement Types
1. **Integration**: Add existing functionality to pipeline (Low)
2. **Visualization**: New maps/charts (Medium)
3. **Analysis**: New metrics/statistics (Medium-High)
4. **Algorithm**: Core redistricting changes (High)
5. **Data**: New census year support (Very High)

## Output
✅ Complete spec in `context/enhancements/active/XX_name.md`
✅ Entry in `context/enhancements/INDEX.md`
✅ Detailed phases with tasks/files
✅ Testing approach following project patterns
✅ Success criteria for validation
✅ User approval before impl

## Next Steps
**After user approval**: `/enhancement-implement` → Follow phases → TodoWrite tracking → `/enhancement-document` when complete

## Examples
See `context/enhancements/completed/`:
• `01_compactness_integration.md` - Simple integration
• `07_edge_weighted_bisection.md` - Algorithm + quantitative validation
• `13_directory_unification.md` - Large refactoring
• `18_figure_quality.md` - Validation + retry logic
