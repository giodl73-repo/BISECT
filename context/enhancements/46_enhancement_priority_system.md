# E46: Priority System for Enhancements and Enhancement Manager

**Status**: Completed
**Priority**: Medium
**Created**: January 17, 2026
**Started**: January 17, 2026
**Completed**: January 17, 2026

## Problem Statement

Enhancement proposals include priority in their markdown frontmatter, but:
1. No standardized priority levels
2. Enhancement Manager app doesn't display or filter by priority
3. INDEX.md doesn't sort by priority
4. No way to quickly identify high-priority work

**Current State:**
- Enhancements have freeform priority text ("High", "Medium-Low", "Critical for Publication", etc.)
- Enhancement Manager shows all enhancements equally
- INDEX.md lists enhancements by number, not priority
- Hard to answer: "What should I work on next?"

**User Request (January 17, 2026):**
> "can we add priority to our enhancement system and to our enhancement manager as a enhancement-plan"

## Goals

1. Standardize priority levels across all enhancements
2. Add priority display to Enhancement Manager web app
3. Enable filtering/sorting by priority
4. Update INDEX.md with priority-sorted views
5. Make priority field required in enhancement template

## Proposed Priority Levels

### 5-Tier System (Recommended)

**Critical** - Blocker for publication or core functionality
- Example: E42 (Research Narrative)
- Example: E45 (Baseline Data Organization)
- Timeline: Must do now

**High** - Important for project goals, significant impact
- Example: E11 (Baseline Comparison) ✅ Completed
- Example: E36 (Experimental Variants Config)
- Timeline: Next 1-2 sprints

**Medium** - Valuable but not urgent
- Example: E43 (Longitudinal Analysis)
- Example: E41 (Public Distribution)
- Timeline: Next quarter

**Low** - Nice to have, quality-of-life improvements
- Example: E40 (Corner Adjacency Filter)
- Example: E32 (Reock Compactness)
- Timeline: When time permits

**Research** - Experimental/exploratory, uncertain value
- Example: E22 (National Redistricting - no state boundaries)
- Example: E24 (Party-Based Allocation)
- Timeline: No timeline (pure research)

### Alternative: 3-Tier System (Simpler)

**P0 (Critical)** - Must have for publication
**P1 (High)** - Should have for completeness
**P2 (Low)** - Nice to have

## Implementation Plan

### Phase 1: Standardize Enhancement Template

**Update:** `context/enhancements/templates/enhancement_template.md`

```markdown
# Enhancement XX: [Title]

**Status**: Proposed | In Progress | Completed | Cancelled
**Priority**: Critical | High | Medium | Low | Research
**Created**: YYYY-MM-DD
**Commits**: [471716d](https://github.com/giodl_microsoft/redistricting/commit/471716de6b23c50d37ba4c3ca97524626acb1cf7), [ad542e9](https://github.com/giodl_microsoft/redistricting/commit/ad542e9efb96bd2ee745b6ab6e77f2ee721b1789)
**Size**: M - 1,194 lines changed (7 files)
**Estimated Effort**: Small (< 2h) | Medium (2-8h) | Large (8-40h) | XL (> 40h)
```

**Add Guidance:**
```markdown
### Priority Levels:
- **Critical**: Blocker for publication or core functionality
- **High**: Important for project goals, significant impact
- **Medium**: Valuable but not urgent
- **Low**: Quality-of-life improvements
- **Research**: Experimental/exploratory work
```

### Phase 2: Update Existing Enhancements

**Bulk Update:**
- [ ] Review all 46 enhancements
- [ ] Assign consistent priority levels
- [ ] Update frontmatter with standardized format
- [ ] Document rationale for priority assignments

**Priority Assignment Criteria:**
1. **Critical** → Blocks publication or breaks existing functionality
2. **High** → Answers key research questions or high user impact
3. **Medium** → Improves quality/capabilities but not urgent
4. **Low** → Polish and refinement
5. **Research** → Speculative explorations

### Phase 3: Update Enhancement Manager App

**Modify:** `tools/enhancement_manager/app.py`

**Features to Add:**

1. **Priority Display**
   - Add priority badge to enhancement cards
   - Color-coded: Red (Critical), Orange (High), Yellow (Medium), Blue (Low), Purple (Research)
   - Priority icon/label

2. **Filtering**
   - Dropdown: "Filter by Priority"
   - Options: All | Critical | High | Medium | Low | Research
   - Checkboxes: Select multiple priorities

3. **Sorting**
   - Default sort: Priority (Critical → Research), then by number
   - Alternative sorts: Number, Date, Status
   - User-selectable sort order

4. **Priority Summary**
   - Dashboard widget showing counts by priority
   - Example: "3 Critical | 7 High | 11 Medium | 15 Low | 5 Research"

5. **Search/Filter Combinations**
   - Filter by: Status + Priority + Search text
   - Example: "Show High/Critical enhancements that are In Progress"

**UI Mockup:**
```
╔══════════════════════════════════════════════════╗
║ Enhancement Manager                               ║
║                                                   ║
║ [Status: All ▼] [Priority: All ▼] [Sort: Priority ▼] [🔍 Search]
║                                                   ║
║ Summary: ⚠️ 3 Critical  🔴 7 High  🟡 11 Medium  🔵 15 Low  🟣 5 Research
║                                                   ║
║ ┌────────────────────────────────────────────┐  ║
║ │ 🔴 [Critical] #42: Research Narrative       │  ║
║ │ Status: Proposed                            │  ║
║ │ Defines research questions and policy...    │  ║
║ │ [View Details] [Edit] [Start]              │  ║
║ └────────────────────────────────────────────┘  ║
║                                                   ║
║ ┌────────────────────────────────────────────┐  ║
║ │ 🔴 [Critical] #45: Baseline Data Org        │  ║
║ │ Status: Proposed                            │  ║
║ │ Organize baseline comparison data...        │  ║
║ │ [View Details] [Edit] [Start]              │  ║
║ └────────────────────────────────────────────┘  ║
╚══════════════════════════════════════════════════╝
```

### Phase 4: Update INDEX.md

**Modify:** `context/enhancements/INDEX.md`

**Add Priority-Based Views:**

```markdown
## By Priority

### 🔴 Critical (3 enhancements)
| # | Title | Status | Files |
|---|-------|--------|-------|
| [42](42_research_narrative_policy_questions.md) | Research Narrative and Policy Questions | Proposed | [View](42_research_narrative_policy_questions.md) |
| [45](45_baseline_data_organization.md) | Baseline Data Organization | Proposed | [View](45_baseline_data_organization.md) |

### 🟠 High (7 enhancements)
...

### 🟡 Medium (11 enhancements)
...

### 🔵 Low (15 enhancements)
...

### 🟣 Research (5 enhancements)
...
```

**Keep Existing Views:**
- Status-based view (Completed | In Progress | Planned)
- Chronological view (by number)

### Phase 5: Documentation

**Update:**
- [ ] `context/ENHANCEMENT_WORKFLOW.md` - Add priority guidance
- [ ] `CONTRIBUTING.md` - Explain priority system
- [ ] `tools/enhancement_manager/README.md` - Document new features

**Add Section: "How to Prioritize Enhancements"**
```markdown
### Prioritization Criteria

**Critical:**
- Blocks paper publication
- Fixes broken functionality
- Legal/compliance requirements
- Example: Missing baseline comparison data

**High:**
- Answers key research questions
- High user/stakeholder impact
- Enables other enhancements
- Example: Experimental variants config system

**Medium:**
- Improves quality or capabilities
- Moderate user impact
- Can be deferred without major consequences
- Example: Longitudinal analysis

**Low:**
- Polish and refinement
- Quality-of-life improvements
- Low urgency
- Example: Corner adjacency filtering

**Research:**
- Exploratory work
- Uncertain value/feasibility
- No immediate application
- Example: National redistricting (no state boundaries)
```

## Files to Create/Modify

### Create
- None (all modifications to existing files)

### Modify
- `context/enhancements/templates/enhancement_template.md` - Add priority guidance
- `tools/enhancement_manager/app.py` - Add priority features
- `tools/enhancement_manager/templates/index.html` - Update UI
- `tools/enhancement_manager/static/style.css` - Add priority colors
- `context/enhancements/INDEX.md` - Add priority-sorted views
- `context/ENHANCEMENT_WORKFLOW.md` - Add prioritization guidance
- All 46 existing enhancement files - Standardize priority field

## Success Criteria

- [ ] All enhancements have standardized priority field
- [ ] Enhancement Manager displays priority badges
- [ ] Can filter enhancements by priority
- [ ] Can sort enhancements by priority
- [ ] INDEX.md includes priority-sorted view
- [ ] Documentation explains priority system
- [ ] Priority assignment rationale documented

## Priority Color Scheme

**Colors for Badges/UI:**
- 🔴 Critical: Red (#DC2626)
- 🟠 High: Orange (#EA580C)
- 🟡 Medium: Yellow (#CA8A04)
- 🔵 Low: Blue (#2563EB)
- 🟣 Research: Purple (#9333EA)

## Benefits

1. **Clarity** - Instantly see what's most important
2. **Focus** - Work on high-impact items first
3. **Communication** - Stakeholders understand priorities
4. **Planning** - Better sprint/milestone planning
5. **Triage** - Quickly decide what to defer

## Related Enhancements

- [E35: Enhancement Manager App](35_enhancement_manager_app.md) - Base app being enhanced
- [E19: Create-Skill Meta-Skill](19_create_skill.md) - Similar meta-tooling enhancement

## Notes

**Why 5-Tier vs. 3-Tier:**
- 5-tier provides more granularity (separates "nice to have" from "research")
- 3-tier is simpler but may force too much into "Low" category
- Recommendation: Start with 5-tier, collapse to 3-tier if too complex

**Migration Strategy:**
- Review existing priorities (many already have High/Medium/Low)
- Map to new system (e.g., "High" → "High", "Medium-Low" → "Low")
- Document any changes in enhancement history

**Estimated Effort:** Medium (4-8 hours)
- Template updates: 30 min
- Bulk priority assignment: 2-3 hours (review all 46)
- Enhancement Manager updates: 2-3 hours (UI + filtering logic)
- INDEX.md updates: 1 hour
- Documentation: 1 hour
