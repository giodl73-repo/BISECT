---
name: edit-guide
description: Edit educational guides and documentation for general audiences (layman's guides, tutorials, explainers). Ensures clarity, conciseness, and accessibility for non-expert readers.
allowed-tools:
  - Read
  - Edit
  - Bash
  - Glob
  - Grep
user-invocable: true
---

# Edit Guide

Act as educational editor to review and improve guides, tutorials, explainers written for general (non-expert) audiences. Focuses on clarity, conciseness, accessibility, readability while maintaining technical accuracy.

**Target audience**: General readers, laypeople, students, non-experts | **Style**: Clear, conversational, visual-first, jargon-free | **Common use**: Layman's guides, tutorials, explainers, educational documentation

## Prerequisites
**Required**: Guide document (Markdown/LaTeX/text format), clear target audience definition, target page/word count (if applicable)
**Recommended**: Figures/diagrams available, technical paper/presentation for reference, subject matter expert available for validation

## When to Use
User says "Edit layman's guide/Review tutorial/Make this more readable/Condense for general audience/Too wordy/Eliminate repetition", guide exceeds target length, content too technical for intended audience, figures/examples need repositioning, redundant explanations across sections

## Editing Levels

**Level 1: Light Polish** (15-30 min): Fix clarity issues, remove obvious jargon, tighten verbose sentences, no major restructuring → Use when: Quick readability pass

**Level 2: Standard Edit** (30-60 min): Full clarity review, moderate condensing (10-20% reduction), simplify technical language, improve flow between sections, no major restructuring → Use when: Standard guide improvement

**Level 3: Heavy Edit** (1-3h): Significant condensing (20-40% reduction), reorganize sections for clarity, replace verbose explanations, move/resize figures, eliminate redundancy → Use when: Guide significantly too long or technical

**Level 4: Complete Overhaul** (3+h): Major restructuring, aggressive condensing (40%+ reduction), rewrite technical sections, create new examples, complete audience realignment → Use when: Guide fundamentally misses target audience

## Workflow

### Step 1: Analyze Current State
Read full guide noting: Total length (pages/words), target audience (actual vs intended), technical depth, figure count + sizes, redundant sections, jargon density, flow issues, target length (if any)

### Step 2: Identify Opportunities
Common issues: Too technical (jargon, equations, citations), too long (verbose explanations, redundant examples, oversized figures), poor flow (disconnected sections, unclear transitions, redundant info), unclear structure (buried main points, too many subsections, confusing organization)

### Step 3: Apply Condensing Strategies
**Paragraph-level**: Cut verbose intros (get to point faster), eliminate redundant clauses ("In order to" → "To"), combine related sentences, remove obvious statements, use active voice
**Section-level**: Combine similar sections, move details to appendix, keep one example per concept (cut redundant examples), consolidate bullet lists to prose
**Figure-level**: Combine multi-figure sequences (3 separate → 1 multi-panel), reduce size (0.8 textwidth → 0.6 textwidth), move to appendix (if not critical), eliminate if redundant

### Step 4: Simplify Language
**Jargon removal**: Replace technical terms with plain language (if term necessary, define once then use consistently), avoid abbreviations unless standard, remove citations from body (move to footnotes)
**Sentence clarity**: One idea per sentence, active voice preferred, short sentences (<25 words), concrete examples over abstractions

### Step 5: Improve Structure
**Section organization**: Intro states goal + outcome upfront, body organized by concept (not chronology), examples follow explanations (not before), conclusions summarize key takeaways
**Transitions**: Clear connectors between sections ("Next, we...", "This leads to..."), forward references ("as shown later"), no abrupt topic changes

### Step 6: Optimize Figures
**Sizing**: Critical figures (0.8 textwidth), supporting figures (0.6 textwidth), examples (0.5 textwidth), multi-panel combinations (3 panels @ 0.3 textwidth each)
**Captions**: Short (1-2 sentences), explain what to notice, no jargon
**Placement**: Near first reference, not interrupting flow, grouped by topic (not scattered)

### Step 7: Handle Tables
**Condensing**: Remove non-essential rows/columns, combine categories, use prose instead (if <5 rows), move to appendix (if reference only)
**Formatting**: Minimal lines (booktabs style), clear headers, units in headers (not cells), consistent alignment

### Step 8: Verify Changes
**Quality checks**: [ ] Meets target length, [ ] Maintains technical accuracy, [ ] Clear for target audience, [ ] No jargon without definition, [ ] All figures necessary, [ ] Smooth flow between sections, [ ] Key concepts preserved, [ ] Examples support main points

### Step 9: Generate Change Summary
Document: Sections removed/combined, figures resized/combined/moved, content condensed (% reduction), jargon simplified (examples), structure improvements, verification (compile + check page count)

### Step 10: Compile and Validate
Run LaTeX compilation, check page count matches target (±0.5 pages), verify figures render correctly, check for orphaned references, review one final time for clarity

## Condensing Strategies

**Verbose → Concise Examples**:
- "In order to achieve this goal, we need to..." → "To achieve this, we..."
- "It is important to note that..." → "Note that..." or remove entirely
- "The algorithm works by performing..." → "The algorithm performs..."
- "As can be seen in the figure..." → "Figure X shows..."
- "One of the key advantages of this approach is..." → "This approach..."

**Redundancy Elimination**:
- Alice/Bob example + Minnesota example → Pick one (strongest example)
- Three separate round figures (1 page each) → One 3-panel figure (1 page total)
- Advantages list (5 bullets) + Limitations list (4 bullets) → Prose paragraph (0.5 page)
- Background section + Introduction → Combine (both cover motivation)

**Jargon Simplification**:
- "Polsby-Popper compactness metric" → "compactness score"
- "METIS recursive bisection partitioning" → "METIS algorithm"
- "Census tract geometries" → "census tracts"
- "Adjacency graph structure" → "connections between tracts"

## Figure Strategies

**Multi-panel combinations** (LaTeX):
```latex
\begin{figure}[H]
\centering
\begin{tabular}{@{}c@{\hspace{0.5cm}}c@{\hspace{0.5cm}}c@{}}
\includegraphics[width=0.3\textwidth]{fig1.png} &
\includegraphics[width=0.3\textwidth]{fig2.png} &
\includegraphics[width=0.3\textwidth]{fig3.png} \\
\textbf{Panel A} & \textbf{Panel B} & \textbf{Panel C}
\end{tabular}
\caption{Overall caption explaining all panels}
\end{figure}
```

**Caption writing** (good for laymen): Short (1-2 sentences), explain what to notice, no jargon
- Good: "All three rounds fit on one page, showing how the algorithm progresses from 1 state to 8 districts."
- Bad: "Minnesota Round 1: One state becomes two regions. The algorithm found a roughly north-south split that balances population while minimizing the shared boundary length utilizing edge-weighted METIS partitioning..."

## Troubleshooting

**Still too long after editing**: Move entire sections to appendix, cut background sections (assume knowledge), remove all but one example per concept, combine multiple short sections, use appendix for examples

**Lost technical accuracy**: Get SME review, add footnotes for caveats, reference technical paper, use "generally" and "typically"

**Figures don't fit after resizing**: Increase to 2-panel instead of 3-panel, split across two pages, move some to appendix, reduce margins slightly, rotate to landscape

**Section feels incomplete after cutting**: Add back strategically (one key example, one visual diagram, one sentence bridge), don't just restore all cuts

## Example Usage

**Scenario**: 26-page layman's guide needs to be 20 pages

**Steps**:
1. User invokes: `/edit-guide`
2. Skill asks: Guide (`presentations/edge_weighted_bisection/laymen_guide.tex`), Target (26 pages → 20 pages), Level (Heavy Edit), Audience (General public), Issues ("Round maps too large, too repetitive")
3. Skill analyzes guide structure
4. Applies edits: Combines Minnesota maps (3 pages → 1 page), Combines Alabama maps (3 pages → 1 page), Removes redundant Alice/Bob example, Removes verbose METIS file format example, Condenses National Results section, Tightens introductions/explanations throughout, Consolidates advantages/limitations lists to prose
5. Generates summary showing ~6-7 pages saved
6. Compiles to verify 19-20 pages achieved
7. Reports changes and areas for review

**Result**: Readable, concise guide meeting target length while preserving all key concepts

## What You'll Get
Condensed guide meeting target length (if specified), simplified language for target audience (jargon removed/defined), improved flow and structure (clear transitions, logical organization), optimized figures and tables (appropriately sized, well-placed), preserved technical accuracy (all key concepts retained), change summary documenting edits (for review/validation), compiled output verifying page count (LaTeX guides)

## Next Steps
Review changes with subject matter expert (technical accuracy check), compile final version (LaTeX → PDF), share with test readers (target audience), incorporate feedback if needed (iterative refinement), finalize and publish (once validated)

## Related Skills
`/edit-paper` (edit academic papers - different audience/style), `/edit-presentation` (edit conference presentations), `/update-docs` (update technical documentation), `/compile-latex` (compile guide after editing), `/create-pedagogical-example` (create educational examples)
