---
name: edit-guide
description: Edit educational guides and documentation for general audiences (layman's guides, tutorials, explainers). Ensures clarity, conciseness, and accessibility for non-expert readers.
version: 1.0.0
author: Claude
category: editorial
user-invocable: true
tags:
  - editing
  - documentation
  - layman
  - educational
  - guide
  - tutorial
  - explainer
tools:
  - Read
  - Edit
  - Write
  - Bash
  - Glob
  - Grep
related_skills:
  - edit-paper
  - edit-presentation
  - update-docs
  - compile-latex
---

# Edit Guide

## Overview

Act as an educational editor to review and improve guides, tutorials, and explainers written for general (non-expert) audiences. Focuses on clarity, conciseness, accessibility, and readability while maintaining technical accuracy.

**Target audience**: General readers, laypeople, students, non-experts
**Style**: Clear, conversational, visual-first, jargon-free
**Common use**: Layman's guides, tutorials, explainers, educational documentation

## Prerequisites

**Required**:
- Guide document (Markdown, LaTeX, or other text format)
- Clear target audience definition
- Target page/word count (if applicable)

**Recommended**:
- Figures and diagrams available
- Technical paper/presentation for reference
- Subject matter expert available for validation

## When to Use This Skill

- User says: "Edit my layman's guide" or "Review my tutorial"
- User says: "Make this more readable" or "Condense for general audience"
- User says: "Too wordy" or "Eliminate repetition"
- Guide exceeds target length
- Content is too technical for intended audience
- Figures/examples need repositioning
- Redundant explanations across sections

## Editing Levels

Choose editing intensity based on needs:

### Level 1: Light Polish (15-30 minutes)
**What it includes**:
- Fix clarity issues
- Remove obvious jargon
- Tighten verbose sentences
- No major restructuring

**Use when**: Quick readability pass

### Level 2: Standard Edit (30-60 minutes)
**What it includes**:
- Full clarity review
- Moderate condensing (10-20% reduction)
- Simplify technical language
- Improve flow between sections
- No major restructuring

**Use when**: Standard guide improvement

### Level 3: Heavy Edit (1-3 hours)
**What it includes**:
- Significant condensing (20-40% reduction)
- Reorganize sections for clarity
- Replace verbose explanations
- Move/resize figures
- Eliminate redundancy

**Use when**: Guide significantly too long or technical

### Level 4: Complete Overhaul (3+ hours)
**What it includes**:
- Major restructuring
- Aggressive condensing (40%+ reduction)
- Rewrite technical sections
- Create new examples
- Complete figure reorganization

**Use when**: Major revision required

## Workflow

### Step 1: Analyze Guide

**Gather requirements**:
- Which guide document? (path)
- Target audience level (layman / student / technical non-expert)
- Current vs target length (e.g., "26 pages → 20 pages")
- Editing level (Light / Standard / Heavy / Complete)
- Key concepts that must be preserved
- Specific problem areas (figures too large, too repetitive, etc.)

**Read guide**:
```bash
# Find guide files
find . -name "*guide*.tex" -o -name "*guide*.md"

# Read main guide
Read guide_file.tex
```

**Analyze structure**:
- Total pages/words
- Section breakdown
- Figure count and sizes
- Paragraph lengths
- Technical density
- Repetition patterns

### Step 2: Identify Issues

**Common problems in educational guides**:

**1. Over-explanation**
- Repeating same concept multiple times
- Verbose step-by-step when summary would work
- Too many examples of same principle

**2. Technical creep**
- Jargon without definition
- Technical details unnecessary for understanding
- Math/formulas when intuition suffices

**3. Poor figure sizing**
- Figures too large (waste space)
- Figures too small (unreadable)
- Multiple figures that could be combined

**4. Redundant content**
- Same information in multiple sections
- Overlapping examples
- Repeated introductions to same concept

**5. Poor flow**
- Jumping between topics
- Missing transitions
- Examples before explanation
- Figure placement breaks narrative

### Step 3: Apply Edits

**Condensing strategies**:

**Eliminate repetition**:
```latex
% BEFORE (repetitive)
\subsection{What is X?}
X is a method that does Y...

\subsection{How X Works}
X is a technique that accomplishes Y by...
```

```latex
% AFTER (consolidated)
\subsection{Understanding X}
X is a method that does Y by...
```

**Convert lists to prose** (when more readable):
```latex
% BEFORE (verbose list)
\textbf{Advantages:}
\begin{itemize}
\item First, it eliminates partisan manipulation
\item Second, it provides complete transparency
\item Third, it offers a neutral baseline
\item Fourth, it runs very quickly
\end{itemize}
```

```latex
% AFTER (concise prose)
\textbf{Advantages}: Eliminates partisan manipulation, provides transparency, offers neutral baseline, and runs quickly.
```

**Tighten verbose explanations**:
```latex
% BEFORE (wordy)
In order to understand how this works, we first need
to think about what a graph actually is. A graph, in
computer science terms, is not the same thing as the
x-y coordinate graphs you might remember from math
class. Instead, it's a completely different concept...
```

```latex
% AFTER (concise)
A graph in computer science isn't an x-y plot—it's a
network of connected nodes.
```

**Combine related figures**:
```latex
% BEFORE (3 separate full-page figures)
\begin{figure}[H]
\includegraphics[width=0.65\textwidth]{round_1.png}
\caption{Round 1: 1→2 regions}
\end{figure}

\begin{figure}[H]
\includegraphics[width=0.65\textwidth]{round_2.png}
\caption{Round 2: 2→4 regions}
\end{figure}

\begin{figure}[H]
\includegraphics[width=0.65\textwidth]{round_3.png}
\caption{Round 3: 4→8 regions}
\end{figure}
```

```latex
% AFTER (combined 3-panel figure)
\begin{figure}[H]
\centering
\begin{tabular}{@{}c@{\hspace{0.5cm}}c@{\hspace{0.5cm}}c@{}}
\includegraphics[width=0.3\textwidth]{round_1.png} &
\includegraphics[width=0.3\textwidth]{round_2.png} &
\includegraphics[width=0.3\textwidth]{round_3.png} \\
\textbf{Round 1: 1→2} & \textbf{Round 2: 2→4} & \textbf{Round 3: 4→8}
\end{tabular}
\caption{Three rounds of recursive bisection}
\end{figure}
```

**Remove redundant examples**:
- Keep the BEST example (most clear, most visual)
- Cut others or move to appendix
- One great example > three mediocre ones

**Simplify technical sections**:
```latex
% BEFORE (too technical)
METIS reads a specially-formatted CSR graph file where
the first line contains metadata (n vertices, m edges,
format code, k partitions), followed by n lines each
listing the vertex's neighbors with edge weights...
```

```latex
% AFTER (appropriate for laymen)
METIS reads a file describing the graph structure and
outputs which partition each tract belongs to.
```

### Step 4: Section-Specific Strategies

**Introduction (Section 1)**
- Goal: Hook reader quickly, establish problem
- Cut: Unnecessary background everyone knows
- Keep: Clear problem statement, motivation

**Fundamentals (Sections 2-3)**
- Goal: Build intuition, not technical mastery
- Cut: Redundant explanations, too many examples
- Keep: Best visual examples, core concepts
- Move up: Real examples before abstract ones

**Details (Sections 4-5)**
- Goal: Enough detail to understand, not implement
- Cut: Technical implementation details
- Keep: Key insights, "why it matters"
- Consider: Moving some content to appendix

**Examples (Section 6)**
- Goal: See algorithm in action
- Cut: Repetitive step-by-step explanations
- Keep: Visual progression, final results
- Combine: Multiple similar examples

**Results (Section 7)**
- Goal: Demonstrate effectiveness concisely
- Cut: Redundant tables, verbose lists
- Keep: Key numbers, comparative results
- Consolidate: Multiple tables into summary

**Conclusion (Section 8-9)**
- Goal: Takeaways and implications
- Cut: Repetition of earlier content
- Keep: Main insights, future directions

### Step 5: Quality Checks

**Readability**:
- [ ] Sentences average 15-20 words
- [ ] Paragraphs 3-5 sentences max
- [ ] Jargon defined on first use
- [ ] Technical terms have plain-language equivalents

**Clarity**:
- [ ] Each section has clear purpose
- [ ] Transitions between sections smooth
- [ ] Examples precede or accompany complex concepts
- [ ] Figures support (not repeat) text

**Conciseness**:
- [ ] No redundant explanations
- [ ] No unnecessary examples
- [ ] Lists converted to prose where clearer
- [ ] Verbose explanations tightened

**Visual balance**:
- [ ] Figures appropriately sized
- [ ] No walls of text
- [ ] White space improves readability
- [ ] Related figures combined

**Accessibility**:
- [ ] 8th-10th grade reading level (for layman's guide)
- [ ] Minimal unexplained acronyms
- [ ] Math used sparingly, with intuition
- [ ] Examples from everyday experience

### Step 6: Generate Summary

Create edit summary documenting changes:

```markdown
# Edit Summary: [Guide Name]

**Date**: [Date]
**Editing Level**: [Level]
**Target**: Reduce from X pages to Y pages

## Changes Made

### Condensing (N pages saved)
- Eliminated Section X redundancy
- Combined Figures A, B, C into single 3-panel
- Removed verbose example in Section Y
- Consolidated Results section (3 subsections → 1)

### Clarity Improvements
- Moved real example up to Section 2 (before abstract)
- Removed Alice/Bob social network example (redundant)
- Simplified METIS file format explanation
- Converted advantages/limitations lists to prose

### Figures
- Resized Minnesota maps: 3 pages → 1 page (3-panel)
- Resized Alabama maps: 3 pages → 1 page (3-panel)
- Removed redundant schematic figures (kept best one)

### Technical Simplification
- Removed detailed file format example
- Simplified graph partitioning explanation
- Streamlined recursive bisection description

## Statistics

- **Pages**: X → Y pages (-Z%)
- **Sections modified**: N
- **Figures resized/combined**: M
- **Redundant content removed**: K instances

## Files Modified

- `path/to/guide.tex`

## Next Steps

- Compile to verify final page count
- Verify figures render correctly
- Check that key concepts preserved
- Get subject matter expert review
```

### Step 7: Verify

**Compile (if LaTeX)**:
```bash
/compile-latex path/to/guide
```

**Check**:
- Final page count meets target
- Figures render correctly at new sizes
- No broken cross-references
- PDF compiles without errors

**Report to user**:
- Summary of changes
- Before/after statistics
- Areas needing author review
- Suggestions for further improvement

## Best Practices

### For Layman's Guides

**Use analogies from everyday life**:
- "Like a social network" (for graphs)
- "Like splitting a pie" (for bisection)
- "Like a circle vs a snake" (for compactness)

**Front-load the "why"**:
- Why should reader care?
- What problem does this solve?
- How does it affect them?

**Real before abstract**:
- Show actual example first
- Then explain abstraction
- Not: theory → theory → example

**Visual first**:
- Prefer figures to text
- Use captions effectively
- One good figure > paragraph of explanation

**Progressive disclosure**:
- Simple concepts first
- Build to complexity
- Appendix for deep dives

### Common Mistakes to Avoid

**❌ Don't assume background knowledge**
- Define all technical terms
- Explain acronyms
- Provide context for domain concepts

**❌ Don't over-explain**
- One clear explanation > three repetitive ones
- Trust reader to understand
- Use appendix for extra examples

**❌ Don't bury the lede**
- Put best example first
- Lead with insights
- Results before methodology

**❌ Don't use passive voice excessively**
- "We create a graph" > "A graph is created"
- Active voice more engaging
- Passive okay for formal results

**❌ Don't sacrifice accuracy for simplicity**
- Simplify explanation, not facts
- Add caveats when needed
- Technical paper for full details

## Figure Guidelines

### Sizing Rules

**Full-width figures** (0.9-1.0\textwidth):
- Complex diagrams with labels
- Multi-panel comparisons
- Primary examples

**Half-width figures** (0.5-0.6\textwidth):
- Simple diagrams
- Supporting examples
- Side-by-side comparisons

**Third-width figures** (0.3-0.35\textwidth):
- Multi-panel sets (3+ panels)
- Simple icons/schematics
- When combined in table

### Combining Figures

**When to combine**:
- Showing progression (Step 1, 2, 3)
- Comparing alternatives (Before/After)
- Related examples (Same concept, different data)

**How to combine**:
```latex
\begin{figure}[H]
\centering
\begin{tabular}{@{}c@{\hspace{0.5cm}}c@{\hspace{0.5cm}}c@{}}
\includegraphics[width=0.3\textwidth]{fig1.png} &
\includegraphics[width=0.3\textwidth]{fig2.png} &
\includegraphics[width=0.3\textwidth]{fig3.png} \\
\textbf{Panel A} & \textbf{Panel B} & \textbf{Panel C} \\
\small{Description A} & \small{Description B} & \small{Description C}
\end{tabular}
\caption{Overall caption explaining all panels}
\end{figure}
```

### Caption Writing

**Good captions** (for laymen):
- Short (1-2 sentences)
- Explain what to notice
- No jargon

```latex
% GOOD
\caption{All three rounds fit on one page, showing how the algorithm progresses from 1 state to 8 districts.}

% BAD (too technical)
\caption{Minnesota Round 1: One state becomes two regions. The algorithm found a roughly north-south split that balances population while minimizing the shared boundary length utilizing edge-weighted METIS partitioning to ensure optimal compactness metrics as measured by Polsby-Popper scores.}
```

## Troubleshooting

### Still Too Long After Editing

**More aggressive strategies**:
- Move entire sections to appendix
- Cut background sections (assume knowledge)
- Remove all but one example per concept
- Combine multiple short sections
- Use appendix for examples

### Lost Technical Accuracy

**Balance accuracy and simplicity**:
- Get SME review
- Add footnotes for caveats
- Reference technical paper
- Use "generally" and "typically"

### Figures Don't Fit After Resizing

**Options**:
- Increase to 2-panel instead of 3-panel
- Split across two pages
- Move some to appendix
- Reduce margins slightly
- Rotate to landscape

### Section Feels Incomplete After Cutting

**Add back strategically**:
- One key example
- One visual diagram
- One sentence bridge
- Don't just restore all cuts

## Related Skills

- `/edit-paper` - Edit academic papers (different audience/style)
- `/edit-presentation` - Edit conference presentations
- `/update-docs` - Update technical documentation
- `/compile-latex` - Compile guide after editing
- `/create-pedagogical-example` - Create educational examples

## Example Usage

**Scenario**: 26-page layman's guide needs to be 20 pages

**Steps**:
1. User invokes: `/edit-guide`
2. Skill asks:
   - Guide: `presentations/edge_weighted_bisection/laymen_guide.tex`
   - Target: 26 pages → 20 pages
   - Level: Heavy Edit
   - Audience: General public (layman)
   - Issues: "Round maps too large, too repetitive"
3. Skill analyzes guide structure
4. Applies edits:
   - Combines Minnesota maps (3 pages → 1 page)
   - Combines Alabama maps (3 pages → 1 page)
   - Removes redundant Alice/Bob example
   - Removes verbose METIS file format example
   - Condenses National Results section
   - Tightens introductions and explanations throughout
   - Consolidates advantages/limitations lists to prose
5. Generates summary showing ~6-7 pages saved
6. Compiles to verify 19-20 pages achieved
7. Reports changes and areas for review

**Result**: Readable, concise guide meeting target length while preserving all key concepts.
