# E21: Edit-Presentation Skill

**Status**: ✅ COMPLETED
**Priority**: Medium
**Estimated Complexity**: Medium (2-4 hours)
**Created**: January 15, 2026
**Completed**: January 15, 2026
**Commits**: (Not yet implemented)
**Size**: (Not yet implemented)

## Current State

We have LaTeX Beamer presentations in `presentations/` directory:
- `presentations/edge_weighted_bisection/` - Edge-weighted algorithm presentation
- `.tex` source file with frames
- Compiled with `/compile-latex` skill

Currently:
- Slide content and structure is manually created
- No systematic review for presentation best practices
- No enforcement of slide count limits (typical: 15-20 slides for 15-min talk)
- No checks for visual clarity (text density, bullet points)
- No narrative flow validation
- Inconsistent terminology across slides

When preparing presentations:
1. Manual slide creation and content writing
2. Manual condensing to fit time constraints
3. Manual checks for "one idea per slide" principle
4. Manual narrative flow adjustments
5. Manual visual design review

This leads to:
- Dense slides with too much text
- Slides trying to cover multiple ideas
- Poor narrative flow between sections
- Inconsistent terminology
- Unclear visual hierarchy
- Presentations that run over time

## Goal

Create an `/edit-presentation` skill that acts as a **conference presentation editor** to:

1. **Condense** - Reduce slide count to target (e.g., 20 slides for 15 min)
2. **Clarify** - Ensure one main idea per slide
3. **Visual polish** - Check text density, bullet formatting, visual hierarchy
4. **Narrative flow** - Ensure logical progression and transitions
5. **Time targeting** - Match slide count to presentation duration
6. **Accessibility** - Ensure readability and clarity for varied audiences

**Target audience**: Conference attendees with limited attention span
**Style**: Conversational, visual-first, clear takeaways
**Constraints**: Time limits (10-20 min typical), slide count limits

### Key Presentation Principles

- **One idea per slide** - Each slide should have one main point
- **Visual first** - Images and diagrams over text
- **Minimal text** - 3-5 bullets max, short phrases not sentences
- **Progressive disclosure** - Build complex ideas across multiple slides
- **Clear transitions** - Each slide connects to previous/next
- **Memorable takeaways** - Key points audience will remember
- **Action titles** - "Districts become more compact" not "Compactness Analysis"

## Implementation Plan

### Phase 1: Presentation Analysis

- [ ] Use `AskUserQuestion` to gather requirements:
  - Which presentation to edit?
  - Target duration (10/15/20 minutes)
  - Current slide count vs target slide count
  - Audience level (experts / general researchers / students)
  - Editing focus (condense / clarity / visual / full)
  - Key messages that must be preserved
- [ ] Read presentation `.tex` file(s)
- [ ] Read compiled PDF to understand visual layout (if available)
- [ ] Analyze current structure:
  - Slide count by section
  - Text density per slide
  - Figure/diagram usage
  - Bullet point depth (nested lists)
  - Transition quality
- [ ] Calculate slides per minute (target: ~1 slide per minute)

### Phase 2: Content Review

- [ ] **Check presentation structure**:
  - Opening: Clear motivation and roadmap
  - Body: Logical progression through ideas
  - Conclusion: Clear takeaways and next steps
  - Balance: Sections appropriately sized
- [ ] **Review slide-level content**:
  - One main idea per slide?
  - Title reflects slide content?
  - Text is minimal and scannable?
  - Bullets are concise phrases?
  - Visual elements support content?
- [ ] **Identify issues**:
  - Dense slides (>5 bullets or >50 words)
  - Slides covering multiple ideas
  - Poor transitions between slides
  - Unclear or generic titles
  - Missing visual elements
  - Inconsistent terminology

### Phase 3: Condensing & Restructuring

- [ ] **Condense to target slide count**:
  - Merge slides covering related ideas
  - Remove tangential content
  - Convert detailed slides to simplified overview
  - Move technical details to backup slides
- [ ] **Improve slide structure**:
  - Split multi-idea slides into focused slides
  - Break dense slides into progressive builds
  - Add transition slides between sections
  - Create overview/roadmap slides
- [ ] **Optimize visual flow**:
  - Convert text to diagrams where possible
  - Suggest figure placements
  - Recommend build animations (e.g., \pause)
  - Balance text and visual elements

### Phase 4: Clarity & Polish

- [ ] **Improve titles**:
  - Make titles action-oriented
  - Ensure titles convey main message
  - Use parallel structure across similar slides
  - Add section titles for context
- [ ] **Polish bullet points**:
  - Use parallel grammatical structure
  - Start with action verbs when possible
  - Keep phrases short (5-10 words max)
  - Remove complete sentences
  - Use 2-tier bullets max (avoid deep nesting)
- [ ] **Enhance narrative flow**:
  - Add transition phrases
  - Ensure each slide follows logically from previous
  - Create clear signposts (e.g., "Three main challenges...")
  - Build toward conclusions

### Phase 5: Documentation & Delivery

- [ ] Apply edits to `.tex` file(s) using `Edit` tool
- [ ] Create edit summary document:
  - Slide count before/after
  - Major structural changes
  - Slides requiring author review
  - Suggested visual improvements (if figures needed)
  - Estimated presentation duration
- [ ] Compile with `/compile-latex` to verify no LaTeX errors
- [ ] Provide presentation statistics:
  - Total slide count
  - Slides per section
  - Estimated duration (1 min per slide guideline)
  - Text density metrics

## Files to Modify/Create

### Create

- `.claude/skills/edit-presentation/SKILL.md` - Skill definition
- `presentations/{presentation_name}/EDIT_SUMMARY.md` - Editing report (generated per use)

### Modify

- `presentations/{presentation_name}/*.tex` - Beamer LaTeX files (edited content)
- `CLAUDE.md` - Add `/edit-presentation` to Phase 5 (Editorial) skills
- `SKILLS.md` - Document skill usage and examples
- `enhancements/INDEX.md` - Mark E21 as complete

## Testing Plan

1. **Test on single section** - Small subset (5-7 slides)
   - Clarity mode only
   - Verify one idea per slide
   - Check LaTeX compiles

2. **Test with condensing target** - Full presentation
   - Current: 30 slides, target: 20 slides
   - Verify key content preserved
   - Check narrative flow maintained

3. **Test on dense technical slides** - Complex methodology section
   - Convert text to more visual format
   - Split multi-idea slides
   - Verify technical accuracy preserved

4. **Test with time constraint** - Strict duration limit
   - 15-minute talk (target: 15 slides)
   - Ensure critical content fits
   - Check pacing recommendations

5. **Compilation test** - Ensure LaTeX validity
   - Run `/compile-latex` after edits
   - Verify no Beamer errors introduced
   - Check PDF renders correctly

## Success Criteria

- [ ] `/edit-presentation` skill created in `.claude/skills/edit-presentation/`
- [ ] Can analyze presentation structure and identify issues
- [ ] Provides clear, actionable editing suggestions
- [ ] Can condense slides to target count while preserving key messages
- [ ] Enforces "one idea per slide" principle
- [ ] Improves visual clarity and text density
- [ ] Enhances narrative flow and transitions
- [ ] Generates comprehensive edit summary
- [ ] Edited Beamer files compile without errors
- [ ] Slide count and duration estimates are accurate
- [ ] Can handle section-level or full-presentation edits
- [ ] User can specify editing focus (condense/clarity/visual/full)

## Benefits

- **Time savings**: Reduce manual editing from 3-6 hours to 1-2 hours per presentation
- **Clarity**: Systematic enforcement of presentation best practices
- **Pacing**: Algorithmic condensing to meet time constraints
- **Engagement**: More visual, less text-heavy slides
- **Consistency**: Standardized terminology and structure
- **Accessibility**: Clear, scannable content for all audience levels

## Dependencies

- **Existing `/compile-latex` skill** - Verify edits don't break compilation
- **LaTeX Beamer presentations** - Requires `.tex` files to edit
- **Read/Edit tools** - Core functionality
- **Presentation design knowledge** - Built into skill prompts

## Risks & Mitigations

- **Risk 1**: Condensing might remove important technical details
  - *Mitigation*: Move details to backup slides; flag deletions for author review

- **Risk 2**: Over-simplification could lose technical rigor
  - *Mitigation*: Preserve key technical points; simplify presentation not content

- **Risk 3**: Beamer syntax errors from editing (e.g., \pause, overlays)
  - *Mitigation*: Run `/compile-latex` after editing to verify; careful with Beamer commands

- **Risk 4**: Visual suggestions might require new figures
  - *Mitigation*: Flag these as "author action items"; provide guidance on what to create

- **Risk 5**: Time estimates might be inaccurate (speaking pace varies)
  - *Mitigation*: Use 1 min/slide as guideline; note assumes moderate pacing

## Implementation Notes

### Editing Levels

**Level 1: Light Polish** (10-15 minutes)
- Fix obvious clarity issues
- Improve titles
- Remove excessive text
- No condensing or restructuring

**Level 2: Standard Edit** (20-40 minutes)
- Full clarity review
- Light condensing (10-20% reduction)
- Improve narrative flow
- Polish bullets and titles

**Level 3: Heavy Edit** (40-90 minutes)
- Significant condensing to meet target
- Restructuring sections
- Split/merge slides as needed
- Enhance visual clarity

**Level 4: Complete Overhaul** (90+ minutes)
- Major restructuring
- Aggressive condensing (30%+ reduction)
- Complete visual redesign recommendations
- Full narrative rebuild

### Presentation Style Guidelines

**Slide Titles**:
- **Action-oriented**: "Edge weights improve compactness" (not "Results")
- **Specific**: "52.8% improvement in Polsby-Popper" (not "Better districts")
- **Parallel structure**: If one title is a question, consider questions for related slides

**Bullet Points**:
- **Short phrases**: "METIS recursive bisection" (not "We use the METIS recursive bisection algorithm")
- **Parallel structure**: All bullets start with verbs OR all bullets are nouns
- **3-5 bullets max**: If more, split into multiple slides
- **Action verbs**: "Apply edge weights" (not "Edge weights are applied")

**Visual Balance**:
- **50/50 rule**: Aim for 50% text, 50% visual (figure/diagram/whitespace)
- **Font size**: Never go below 20pt for body text
- **Bullet depth**: 2 levels max (main bullets and sub-bullets only)
- **Whitespace**: Don't fill every inch of the slide

### Common Condensing Patterns

**Merge slides**:
- Two slides covering same topic → One slide with combined message
- Introduction + motivation → Single opening slide
- Multiple result slides → One overview + backup details

**Split slides**:
- Slide with 7+ bullets → Two focused slides
- Slide covering 2 concepts → One slide per concept
- Complex diagram + text → Diagram slide, then text slide

**Remove content**:
- Background material audience likely knows
- Excessive methodology details
- Redundant examples
- Tangential observations

**Move to backup**:
- Detailed derivations
- Full algorithm pseudocode
- Extensive numerical results
- Technical edge cases

### Time Estimation Guidelines

**Standard pacing** (~1 minute per slide):
- 10-minute talk: 10-12 slides
- 15-minute talk: 15-18 slides
- 20-minute talk: 20-24 slides

**Adjustments**:
- Title/intro slides: ~30 seconds
- Dense technical slides: ~90-120 seconds
- Simple diagram slides: ~45 seconds
- Build animations: +15 seconds per build

**Slide count targets**:
- Intro (10%): 1-2 slides
- Motivation (10%): 1-2 slides
- Background (15%): 2-3 slides
- Methodology (25%): 4-6 slides
- Results (30%): 5-7 slides
- Conclusion (10%): 1-2 slides

### Beamer-Specific Considerations

**Commands to preserve**:
- `\pause` - Build animations
- `\only<>`, `\uncover<>` - Overlay specifications
- `\alert{}` - Highlighting
- `\begin{block}` - Text blocks
- `\column{}` - Multi-column layouts

**Commands to use**:
- `\frametitle{}` - Consistent title formatting
- `\begin{itemize}` - Bullet lists
- `\includegraphics` - Figures
- `\section{}` - Section divisions (for navigation)

**Avoid**:
- Deep nesting (>2 bullet levels)
- Tiny fonts (`\tiny`, `\scriptsize`)
- Wall of text paragraphs
- Excessive animations (distracting)

## Related Documentation

- Enhancement #19: Create-Skill Meta-Skill (prerequisite)
- Enhancement #20: Edit-Paper Skill (parallel development)
- `/compile-latex` skill - Verify edited presentations compile correctly
- `/create-presentation-figures` skill - May need new figures for visual slides
- [Beamer User Guide](https://ctan.org/pkg/beamer)
- [Presentation Zen Principles](https://www.presentationzen.com/)
