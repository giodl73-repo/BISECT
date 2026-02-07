---
wave_uuid: b45d8d
slug: edit-paper
uuid: 77daf3
---
# E20: Edit-Paper Skill

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium (2-4 hours)
**Created**: January 15, 2026
**Completed**: January 15, 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

We have LaTeX papers in `papers/` directory:
- `papers/03_combined_recursive_bisection/` - Main redistricting paper
- Multiple `.tex` section files (intro, methodology, results, etc.)
- Compiled with `/compile-latex` skill

Currently:
- Writing and editing is manual
- No systematic review process
- No automated condensing or restructuring
- No target page limit enforcement
- No copyediting for academic style
- Inconsistent notation and terminology across sections

When preparing papers for submission:
1. Manual review of entire paper for clarity
2. Manual condensing to meet page limits (typical: 8-12 pages)
3. Manual copyediting for academic tone
4. Manual consistency checks (notation, citations, terminology)
5. Multiple revision cycles without systematic tracking

This is time-consuming and prone to:
- Missing inconsistencies across sections
- Verbose writing that exceeds page limits
- Unclear or overly technical prose
- Notation inconsistencies between sections

## Goal

Create an `/edit-paper` skill that acts as a **journal editor** to:

1. **Proofread** - Fix grammar, spelling, academic style issues
2. **Condense** - Reduce verbosity while preserving technical content
3. **Copyedit** - Ensure consistent notation, terminology, citations
4. **Restructure** - Suggest section reordering or content movement
5. **Page targeting** - Work toward specific page count goals
6. **Quality check** - Flag unclear explanations, missing context, or logical gaps

**Target audience**: Academic peer reviewers and researchers
**Style**: Formal, technical, precise, concise
**Constraints**: Page limits (8-12 pages typical for conference/journal)

### Key Editing Principles

- **Clarity over brevity** - Never sacrifice understanding for space
- **Technical accuracy** - Preserve mathematical and algorithmic details
- **Active voice** - Prefer "We apply METIS" over "METIS is applied"
- **Concrete over abstract** - Use specific examples and numbers
- **Logical flow** - Each paragraph should connect to previous/next
- **Consistent notation** - Same variable names and terminology throughout

## Implementation Plan

### Phase 1: Paper Analysis

- [ ] Use `AskUserQuestion` to gather requirements:
  - Which paper/section to edit?
  - Current page count vs target page count
  - Editing focus (proofread only / condense / full edit)
  - Specific concerns (notation, clarity, length)
  - Keep or revise abstract/introduction
- [ ] Read all `.tex` files in the paper directory
- [ ] Read compiled PDF to understand visual layout (if available)
- [ ] Analyze current structure:
  - Section lengths and proportions
  - Notation and terminology usage
  - Citation patterns
  - Figure/table references
- [ ] Estimate current page count from LaTeX source

### Phase 2: Content Review

- [ ] **Proofread each section**:
  - Grammar and spelling
  - Academic style (avoid colloquialisms, contractions)
  - Consistent verb tense (typically past for methods, present for results)
  - Proper citation format
- [ ] **Check technical accuracy**:
  - Notation consistency (e.g., $G = (V, E)$ throughout)
  - Algorithm descriptions match implementation
  - Mathematical formulas are correct
  - Variable definitions are clear
- [ ] **Identify verbosity**:
  - Redundant phrases ("in order to" → "to")
  - Unnecessary qualifiers ("very", "quite", "somewhat")
  - Repetitive explanations
  - Overly complex sentences

### Phase 3: Condensing & Restructuring

- [ ] **Condense verbose sections**:
  - Simplify complex sentences
  - Remove redundant content
  - Consolidate similar paragraphs
  - Use more efficient phrasing
- [ ] **Restructure if needed**:
  - Move detailed derivations to appendix
  - Combine similar subsections
  - Reorder for logical flow
  - Merge or split sections as appropriate
- [ ] **Optimize space usage**:
  - Suggest figure/table size reductions
  - Identify content for supplementary materials
  - Recommend inline math vs display math
  - Suggest list format over prose where appropriate

### Phase 4: Quality Enhancement

- [ ] **Improve clarity**:
  - Flag jargon that needs definition
  - Suggest clearer explanations
  - Add transitional sentences between sections
  - Ensure first mention of concepts is defined
- [ ] **Strengthen arguments**:
  - Ensure claims are supported by results
  - Add quantitative evidence where missing
  - Clarify causal vs correlational statements
  - Flag unsupported assertions
- [ ] **Polish presentation**:
  - Consistent formatting (italics, bold, math mode)
  - Proper capitalization of terms
  - Consistent hyphenation (e.g., "edge-weighted")
  - Proper use of LaTeX commands

### Phase 5: Documentation & Delivery

- [ ] Apply edits to `.tex` files using `Edit` tool
- [ ] Create edit summary document:
  - List of major changes
  - Page count reduction achieved
  - Sections requiring author review
  - Suggested follow-up improvements
- [ ] Compile with `/compile-latex` to verify no LaTeX errors
- [ ] Provide before/after statistics:
  - Word count change
  - Estimated page count change
  - Number of edits by category

## Files to Modify/Create

### Create

- `.claude/skills/edit-paper/SKILL.md` - Skill definition
- `papers/{paper_name}/EDIT_SUMMARY.md` - Editing report (generated per use)

### Modify

- `papers/{paper_name}/sections/*.tex` - LaTeX section files (edited content)
- `CLAUDE.md` - Add `/edit-paper` to Phase 5 (Editorial) skills
- `SKILLS.md` - Document skill usage and examples
- `enhancements/INDEX.md` - Mark E20 as complete

## Testing Plan

1. **Test on introduction section** - Small file, clear target (1-2 pages)
   - Proofread only mode
   - Verify no technical errors introduced
   - Check LaTeX compiles correctly

2. **Test on methodology section** - More technical, notation-heavy
   - Full edit mode with condensing
   - Verify notation remains consistent
   - Check algorithm descriptions preserved

3. **Test on complete paper** - Full paper edit
   - Target: Reduce by 1-2 pages
   - Verify all sections edited
   - Check cross-references still work

4. **Test with specific constraints** - Edge cases
   - "Only proofread, don't condense"
   - "Condense by 20% without losing technical detail"
   - "Fix notation inconsistencies only"

5. **Compilation test** - Ensure LaTeX validity
   - Run `/compile-latex` after edits
   - Verify no LaTeX errors introduced
   - Check PDF renders correctly

## Success Criteria

- [ ] `/edit-paper` skill created in `.claude/skills/edit-paper/`
- [ ] Can analyze paper structure and identify issues
- [ ] Provides clear, actionable editing suggestions
- [ ] Can condense text while preserving technical accuracy
- [ ] Fixes grammar, spelling, and style issues
- [ ] Maintains notation and terminology consistency
- [ ] Respects target page limits
- [ ] Generates comprehensive edit summary
- [ ] Edited LaTeX files compile without errors
- [ ] Before/after statistics are accurate
- [ ] Can handle section-level or full-paper edits
- [ ] User can specify editing focus (proofread/condense/full)

## Benefits

- **Time savings**: Reduce manual editing from 4-8 hours to 1-2 hours per paper
- **Consistency**: Automated notation and terminology checking
- **Quality**: Systematic review catches issues manual review might miss
- **Page targeting**: Algorithmic condensing to meet submission limits
- **Accessibility**: Makes academic writing more approachable
- **Iterative**: Can run multiple editing passes with different focuses

## Dependencies

- **Existing `/compile-latex` skill** - Verify edits don't break compilation
- **LaTeX papers** - Requires `.tex` files to edit
- **Read/Edit tools** - Core functionality
- **Academic writing knowledge** - Built into skill prompts

## Risks & Mitigations

- **Risk 1**: Editing might change technical meaning unintentionally
  - *Mitigation*: Flag all technical changes for author review; prefer clarification over rewriting

- **Risk 2**: Condensing might remove important details
  - *Mitigation*: Provide edit summary showing all deletions; user can review and restore

- **Risk 3**: LaTeX syntax errors from editing
  - *Mitigation*: Run `/compile-latex` after editing to verify; careful with math mode

- **Risk 4**: Over-editing could lose author's voice
  - *Mitigation*: Preserve author's style; suggest rather than enforce changes

- **Risk 5**: Notation changes might introduce inconsistencies
  - *Mitigation*: Track all notation uses; make changes globally, not piecemeal

## Implementation Notes

### Editing Levels

**Level 1: Light Proofread** (5-10 minutes)
- Grammar and spelling only
- No condensing or restructuring
- No technical changes

**Level 2: Standard Edit** (15-30 minutes)
- Proofread + copyedit
- Light condensing (remove obvious verbosity)
- Notation consistency checks
- No restructuring

**Level 3: Heavy Edit** (30-60 minutes)
- Full proofread + copyedit
- Significant condensing to meet page targets
- Restructuring suggestions
- Quality enhancement

**Level 4: Complete Overhaul** (1-2 hours)
- Comprehensive editing
- Major restructuring
- Aggressive condensing
- Complete notation standardization

### Academic Style Guidelines

**Preferred**:
- Active voice: "We apply" not "is applied"
- Concrete: "reduces by 52.8%" not "significantly reduces"
- Direct: "The algorithm fails when" not "It appears that the algorithm might fail when"
- Specific: "Census tract 12345" not "a certain tract"

**Avoid**:
- Contractions: "don't" → "do not"
- Colloquialisms: "pretty good" → "effective"
- Hedging: "somewhat", "rather", "quite"
- Passive voice when active is clearer
- Vague quantifiers: "many", "some", "several"

### Common Condensing Patterns

- "in order to" → "to"
- "due to the fact that" → "because"
- "it is important to note that" → [delete]
- "as can be seen in Figure X" → "Figure X shows"
- "We use METIS algorithm" → "We use METIS" (context implied)
- Multiple sentences → One concise sentence
- Separate paragraphs → Combined if related

### Page Estimation Heuristics

**Typical LaTeX article class**:
- ~450-500 words per page (double column)
- ~750-800 words per page (single column)
- Figures: ~0.3-0.5 pages for small, ~0.7-1.0 for large
- Tables: ~0.2-0.4 pages for typical size
- Equations (display): ~0.1-0.2 pages per equation

**Condensing targets**:
- 10% reduction: Remove obvious verbosity
- 20% reduction: Significant condensing, preserve all content
- 30% reduction: Aggressive condensing, move some content to appendix
- 40%+ reduction: Major restructuring required

## Related Documentation

- Enhancement #19: Create-Skill Meta-Skill (prerequisite)
- Enhancement #21: Edit-Presentation Skill (parallel development)
- `/compile-latex` skill - Verify edited papers compile correctly
- [Academic Writing Guidelines](https://writing.wisc.edu/handbook/)
- [LaTeX Best Practices](https://www.overleaf.com/learn/latex/Best_practices)
