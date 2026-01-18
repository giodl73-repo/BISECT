---
name: edit-presentation
description: Edit academic presentations (Beamer LaTeX) for conference talks. Acts as presentation editor ensuring slides are clear, concise (one idea per slide), visually balanced, and fit time constraints. Targets 15-20 slides for 15-20 minute talks with engaging, scannable content.
allowed-tools:
  - Read
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
  - Bash
user-invocable: true
---

# Edit Presentation

Act as conference presentation editor to review and improve Beamer LaTeX slides for academic talks. Ensures slides follow best practices: one idea per slide, visual clarity, minimal text, appropriate pacing for time constraints.

**Target audience**: Conference attendees with limited attention span | **Style**: Conversational, visual-first, clear takeaways | **Common use**: 15-20 minute conference talks (typically 15-20 slides)

## Prerequisites
**Required**: Beamer LaTeX presentation files (`.tex`) in `presentations/` directory, presentation uses standard Beamer structure (frames, sections)
**Recommended**: Compiled PDF available for visual review, clear target duration (e.g., "15-minute talk"), `/compile-latex` skill available for validation

## When to Use
User says "Edit my presentation/Review my slides/My talk is too long/Condense my presentation/Make slides more visual/Improve slide clarity", presentation exceeds time limit (too many slides), slides are text-heavy (>5 bullets per slide), need to enforce "one idea per slide" principle, preparing for conference presentation submission

## Editing Levels

**Level 1: Light Polish** (10-15 min): Fix obvious clarity issues, improve slide titles (make action-oriented), remove excessive text bullets, no condensing or restructuring → Use when: Quick polish before practice talk

**Level 2: Standard Edit** (20-40 min): Full clarity review, light condensing (10-20% slide reduction), improve narrative flow, polish bullet points + titles, no major restructuring → Use when: Standard conference submission preparation

**Level 3: Heavy Edit** (40-90 min): Significant condensing to meet target, restructure sections as needed, split/merge slides for "one idea per slide", enhance visual clarity, add transition slides → Use when: Presentation significantly exceeds time limit

**Level 4: Complete Overhaul** (90+ min): Major restructuring, aggressive condensing (30%+ slides), complete visual redesign, create new flow/narrative, extensive rewriting → Use when: Major revision required

## Workflow

### Step 1: Analyze Presentation
**Gather requirements via `AskUserQuestion`**: Which presentation? (path to presentation directory), target duration? (e.g., "15 minutes", "20 minutes"), current vs target slide count (e.g., "30 slides → 18 slides"), editing level (Light/Standard/Heavy/Complete), key messages (what audience should remember), specific concerns (too text-heavy, unclear flow, over time)

**Read presentation files**: `ls presentations/{name}/*.tex`, `cat presentations/{name}/presentation.tex`

**Identify opportunities**: Total slide count (current vs target), slides per minute (target ~1 slide/minute), text-heavy slides (>5 bullets), slide topics (multiple ideas per slide?), narrative flow (logical progression?), visual balance (text vs figures ratio), title clarity (action-oriented?), redundant content (repeated concepts)

### Step 2: Apply "One Idea Per Slide" Principle
**Identify violations**: Slides with multiple topics, slides with >5 bullets (likely multiple ideas), slides with long explanations (paragraph-style), transition slides that introduce new concepts

**Split slides**: Take multi-topic slide → Create separate slides per topic, keep focused narrative, add transition slides between major sections

**Merge slides**: Two single-bullet slides on same topic → Combine into one clear slide, eliminate redundant intro/outro slides

### Step 3: Improve Titles
**Make action-oriented**: Passive ("Background") → Active ("Why Redistricting Matters"), generic ("Results") → Specific ("Edge Weights Improve Compactness by 53%"), questions ("How does it work?") → Statements ("Algorithm Uses Recursive Bisection")

**Title guidelines**: State the takeaway (not just topic), <8 words ideal, sentence case (not title case in Beamer), avoid "Introduction to..." or "Overview of..."

### Step 4: Reduce Text Density
**Bullet guidelines**: 3-5 bullets per slide (max), <15 words per bullet, complete thoughts (not sentence fragments), parallel structure across bullets

**Text reduction strategies**: Replace bullets with visuals (diagrams, figures, icons), use progressive disclosure (`\pause` between bullets), move details to notes/backup slides, eliminate redundant phrases

### Step 5: Enhance Visual Clarity
**Figure optimization**: One figure per slide (unless comparison), figures sized to be readable from back of room (large fonts/icons), captions below figures (brief 1 sentence), use callouts/arrows to highlight key points

**Visual balance**: 50-70% visual content, 30-50% text (ideal ratio), avoid all-text slides (except section transitions), avoid all-figure slides (need context)

**Color/formatting**: Consistent color scheme, high contrast (dark text on light background), avoid red/green (colorblind issues), use bold/italics sparingly (emphasis only)

### Step 6: Improve Narrative Flow
**Story arc**: Intro (motivation + problem), Methods (approach overview), Results (key findings), Conclusion (takeaways), organize by idea flow (not chronology), add transition slides between major sections

**Transitions**: Clear connectors ("Next, we show...", "This leads to...", "Building on this..."), forward/backward references where appropriate, section dividers (full-screen section titles)

**Redundancy elimination**: Cut redundant background (assume expert audience), remove repeated explanations, keep examples to 1-2 maximum, consolidate similar results

### Step 7: Meet Time Target
**Duration estimation**: ~1 slide per minute (general rule), title slides (0.5 min), content slides (1 min), complex figures (1.5 min), calculate total estimated duration

**If over time**: Remove non-critical slides, merge similar slides, move details to backup, simplify complex slides, reduce examples

**Backup slides**: Move detailed derivations, extra results, technical details, reference materials → Place after main presentation for Q&A

### Step 8: Polish and Refine
**Quality checks**: [ ] Slide count matches target (±2 slides), [ ] All titles action-oriented, [ ] <5 bullets per slide, [ ] Good visual balance, [ ] Narrative flows smoothly, [ ] Key messages clear, [ ] Duration estimate on target

### Step 9: Generate Edit Summary
Document: **Slide changes** (XX slides → YY slides, ZZ% reduction), **Title improvements** (list key changes), **Text reductions** (bullets per slide avg), **Visual enhancements** (figures added/improved), **Narrative improvements** (flow changes), **Duration estimate** (X minutes at 1 slide/min), **Backup slides created** (list), **Suggestions** (new figures to create, areas for review)

### Step 10: Compile and Validate
Run `/compile-latex` to verify, check slide count matches target (±2 slides), verify figures render correctly, review visual balance (not too text-heavy), practice talk with timer (verify duration estimate)

## Best Practices

**Slide design**: One idea per slide (focused message), action-oriented titles (state takeaway), 3-5 bullets maximum (scannable), large text (readable from back), high-contrast colors (accessible), consistent formatting (professional)

**Content**: Visual-first (figures > text), concrete examples (not abstract), quantitative claims (specific numbers), minimal jargon (accessible language), clear transitions (logical flow)

**Timing**: ~1 slide per minute (general rule), practice with timer (verify duration), leave 20% buffer (questions/transitions), create backup slides (extra details for Q&A)

**Common mistakes to avoid**: Too many slides (rushing/skipping), text-heavy slides (unreadable), multiple ideas per slide (confusing), poor titles (generic "Results"), no visual balance (all text or all figures)

## Troubleshooting

**Still over time after editing**: More aggressive slide removal (cut entire sections), simplify complex slides (reduce bullets), remove all examples (keep only key results), move methodology to backup

**Lost key messages during condensing**: Restore critical slides, add summary slide highlighting key takeaways, use bold/color for emphasis, add "Key Point:" callouts

**Slides feel incomplete after cutting**: Add one concrete example back, add one visual diagram, keep transition slide, but don't restore all cuts

**Narrative feels disjointed**: Add transition slides between sections, improve slide titles for flow, add forward references ("as we'll see"), ensure logical progression

## Performance Notes

| Level | Time | Reduction |
|-------|------|-----------|
| Light Polish | 10-15 min | 0-10% slides |
| Standard Edit | 20-40 min | 10-20% slides |
| Heavy Edit | 40-90 min | 20-30% slides |
| Complete Overhaul | 90-120 min | 30-40%+ slides |

**Factors affecting time**: Presentation length (more slides = more time), current density (dense slides take longer to simplify), visual needs (suggesting new figures takes time), restructuring required (major changes slower)

## What You'll Get
Edited Beamer files (all `.tex` files updated with improvements), edit summary (comprehensive report of changes made), before/after metrics (slide count, bullets per slide, duration estimate), compiled PDF (verified presentation compiles correctly), visual suggestions (list of figures/diagrams to create), slides for review (flagged areas needing author attention)

## Next Steps
Review flagged slides (check areas marked for author review), create suggested visuals (generate new figures/diagrams as recommended), recompile presentation (generate final PDF to see visual result), practice talk (rehearse with timer to verify duration target), get feedback (present to colleague for flow/clarity check), submit (if on target and quality high, submit to conference), archive version (save edited version with date for future reference)

## Example Usage

**Scenario**: 30-slide presentation needs to be 18 slides for 15-minute conference talk

**Steps**:
1. User invokes: `/edit-presentation`
2. Skill asks: Presentation (`presentations/edge_weighted_bisection/`), Target (30 slides → 18 slides for 15-minute talk), Level (Heavy Edit), Audience (General researchers), Key messages ("Edge weights improve compactness by 53%")
3. Skill analyzes presentation structure
4. Applies edits: Merges Introduction slides 1-2, splits dense Algorithm slide into 3 focused slides, removes detailed Background slides, consolidates 6 Result slides into 3 comparative slides, moves technical details to backup slides, improves all titles to be action-oriented, reduces bullets from 6-8 to 3-5 per slide
5. Generates summary showing 40% slide reduction
6. Compiles successfully, now 18 slides
7. Flags Algorithm and Results sections for author review
8. Suggests 3 new figures for visual improvement

**Result**: Presentation ready for conference, on time target, clear narrative flow

## Related Skills
`/compile-latex` (compile Beamer after editing to verify no errors), `/create-presentation-figures` (generate new figures as suggested by edits), `/edit-paper` (edit academic papers - different style), `/edit-guide` (edit layman's guides - different audience)
