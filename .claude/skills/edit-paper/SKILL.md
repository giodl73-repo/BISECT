---
name: edit-paper
description: Edit academic papers for journal submission. Acts as journal editor providing proofreading, condensing, copyediting, and page limit targeting. Focuses on clarity, conciseness, technical accuracy, and consistent notation for peer-reviewed publications.
allowed-tools:
  - Read
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
  - Bash
user-invocable: true
---

# Edit Paper

Act as academic journal editor to review and improve LaTeX papers for publication. Provides systematic proofreading, condensing, copyediting, quality enhancement while preserving technical accuracy and meeting page constraints.

**Target audience**: Academic peer reviewers and researchers | **Style**: Formal, technical, precise, concise | **Common use**: Preparing papers for conference/journal submission (8-12 page limits)

## Prerequisites
**Required**: LaTeX paper files (`.tex`) in `papers/` directory, paper uses standard LaTeX structure (sections/figures/citations)
**Recommended**: Compiled PDF available for visual review, clear target page count (e.g., "reduce to 10 pages"), `/compile-latex` skill available for validation

## When to Use
User says "Edit my paper for submission/Proofread the paper/Condense the paper/Help me meet the page limit", user needs consistent notation across sections, user wants academic style review (avoid colloquialisms), paper exceeds journal/conference page limits, preparing for peer review submission

## Editing Levels

**Level 1: Light Proofread** (5-10 min): Grammar/spelling fixes, remove obvious verbosity, fix citation formatting, no condensing or restructuring → Use when: Quick polish before informal review

**Level 2: Standard Edit** (15-30 min): Full proofread + copyedit, light condensing (10-15% reduction), notation consistency checks, improve clarity of unclear passages, no major restructuring → Use when: Standard submission preparation

**Level 3: Heavy Edit** (30-60 min): Comprehensive editing, significant condensing (20-30% reduction), restructuring sections as needed, quality enhancement (clarity/flow/arguments), meet specific page targets → Use when: Paper significantly exceeds page limit

**Level 4: Complete Overhaul** (60+ min): Major restructuring, aggressive condensing (30%+ reduction), complete notation standardization, extensive rewriting for clarity, move content to appendix/supplementary → Use when: Major revision required

## Workflow

### Step 1: Analyze Paper
**Gather requirements via `AskUserQuestion`**: Which paper to edit? (path to paper directory), which sections? (all, or specific sections like methodology), current vs target page count (e.g., "18 pages → 12 pages"), editing level (Light/Standard/Heavy/Complete), specific concerns (notation, clarity, length, style)

**Read paper files**: `ls papers/{paper_name}/*.tex`, `cat papers/{paper_name}/{main}.tex`

**Identify opportunities**: Total page count (current vs target), verbose sections (intro, methodology, results), notation inconsistencies ($G$ vs $G_{adj}$), redundant explanations (repeated concepts), figure/table sizing (can be reduced?), length-reducing moves (appendix candidates)

### Step 2: Proofread and Copyedit
**Grammar/spelling**: Fix typos, grammatical errors, punctuation, remove extra whitespace
**Academic style**: Replace colloquialisms ("we show" → "we demonstrate"), remove contractions ("don't" → "do not"), use formal tone consistently, avoid first person where inappropriate
**Citations**: Consistent format (all \cite{...}), proper ordering (alphabetical/chronological), no broken references

### Step 3: Condense Content
**Sentence-level**: Remove verbose phrases ("due to the fact that" → "because", "in order to" → "to", "it is worth noting that" → [delete]), use active voice ("the algorithm computes" vs "is computed by"), eliminate redundant clauses
**Paragraph-level**: Combine related sentences, remove redundant statements, keep one example per concept, consolidate bullet lists to prose
**Section-level**: Combine introductory subsections, move detailed derivations to appendix, consolidate results tables, merge redundant background sections

### Step 4: Standardize Notation
**Variables**: Consistent symbols ($n$ always = number, $G$ always = graph), consistent formatting (italics for variables $x$, upright for functions $\sin$), define all notation on first use
**Equations**: Consistent numbering style, proper alignment using align/equation environments, explanatory text before/after equations
**Terms**: Pick one term per concept ("redistricting" vs "apportionment" → choose one consistently), define abbreviations (METIS, RBA, PP)

### Step 5: Improve Clarity
**Paragraph structure**: Topic sentence first, supporting sentences follow, conclusion/transition last
**Section transitions**: Clear connectors between sections, forward/backward references where appropriate
**Technical explanations**: Define technical terms on first use, use concrete examples, refer to figures/tables to support arguments
**Arguments**: Clear logical flow (premise → evidence → conclusion), quantitative support for claims, acknowledge limitations where appropriate

### Step 6: Optimize Figures and Tables
**Figure sizing**: Essential figures (0.5 textwidth), supporting figures (0.4 textwidth), multi-panel combinations (2-3 panels per figure)
**Table condensing**: Remove non-essential rows/columns, combine related data, use scientific notation for large numbers, move to appendix if reference-only
**Captions**: Concise (1-2 sentences), explain key takeaway, avoid repeating body text
**Placement**: Near first reference, grouped by topic, avoid orphaning

### Step 7: Apply Specific Targets
**If page limit specified**: Calculate required reduction % ((current-target)/current), prioritize sections by length + verbosity, condense aggressively in verbose sections, consider appendix moves for non-critical content
**If notation focus**: Create notation table, check every symbol usage, update inconsistencies systematically, add notation section if needed
**If clarity focus**: Identify unclear passages, rewrite for clarity, add examples/figures, get feedback on improved sections

### Step 8: Verify Technical Accuracy
**Equations**: Check all derivations, verify equation numbers referenced correctly, ensure notation consistent with definitions
**Claims**: Verify quantitative statements match results, check citations support claims, flag unsupported assertions
**Code/algorithms**: If included, check syntax + logic, verify matches paper descriptions, ensure reproducibility

### Step 9: Generate Edit Summary
Document: **Sections edited** (list affected sections), **Word/page reduction** (XX pages → YY pages, ZZ% reduction), **Notation changes** (list standardizations), **Major condensing moves** (sections combined, content moved to appendix), **Areas for review** (flag passages needing author attention), **Quality improvements** (clarity enhancements, flow improvements)

### Step 10: Compile and Validate
Run `/compile-latex` to verify, check page count matches target (±0.5 pages), verify figures/tables render correctly, check for orphaned references, review one final time for consistency

## Condensing Strategies

**Verbose → Concise Examples**:
- "due to the fact that" → "because"
- "in order to" → "to"
- "it is worth noting that" → [delete]
- "we utilize the approach of" → "we use"
- "the algorithm is capable of" → "the algorithm can"

**Section Condensing**:
- Multiple intro subsections → Combine into single introduction
- Detailed background → Keep only essential context, cite surveys for details
- Verbose methodology → Focus on novel contributions, reference standard methods
- Redundant results → Combine related findings, move tables to appendix
- Long conclusions → Concise summary of key contributions only

**Notation Standardization**:
- $G$ vs $G_{adj}$ vs $G_a$ → Pick one ($G$)
- $n$ vs $N$ vs $|V|$ → Pick one ($n$)
- PP vs $PP$ vs \text{PP} → Pick one (PP or Polsby-Popper)
- Tracts vs census tracts vs $T$ → Define once, use consistently

## Quality Checklist

**Clarity**: [ ] All technical terms defined, [ ] Notation consistent throughout, [ ] Clear argument structure, [ ] No ambiguous pronouns

**Conciseness**: [ ] No verbose phrases, [ ] One example per concept, [ ] Redundant sections combined, [ ] Appropriate detail level

**Technical accuracy**: [ ] All equations correct, [ ] Claims supported by results, [ ] Citations accurate, [ ] Reproducible descriptions

**Style**: [ ] Active voice where appropriate, [ ] Concrete language (specific numbers/examples), [ ] No colloquialisms or contractions, [ ] Proper academic tone maintained

**Format**: [ ] LaTeX compiles without errors, [ ] No formatting issues introduced, [ ] Line breaks and spacing preserved, [ ] Math mode used correctly

## Troubleshooting

**Still over page limit after editing**: Move entire sections to appendix (detailed derivations, extra results, literature review), aggressive figure/table reduction (combine into multi-panel, reduce sizes), cut background sections (assume expert audience), consider supplementary materials

**Lost technical accuracy during condensing**: Restore critical explanations, add clarifying sentences back strategically, consult with author/co-authors, mark sections for expert review

**Notation changes break references**: Search/replace carefully with context, check all equation references updated, verify figure/table references still correct, recompile to catch orphaned refs

**Unclear after condensing**: Add one concrete example back, restore one explanatory sentence, add forward/backward references, consider keeping original if clarity lost

## Performance Notes

| Level | Time | Reduction |
|-------|------|-----------|
| Light Proofread | 5-10 min | Minimal |
| Standard Edit | 15-30 min | 10-15% |
| Heavy Edit | 30-60 min | 20-30% |
| Complete Overhaul | 60-120 min | 30-40%+ |

**Factors affecting time**: Paper length (longer = more time), current verbosity (more verbose = more to condense), technical density (highly technical = slower to verify accuracy), notation inconsistencies (more issues = more fixes needed)

## What You'll Get
Edited LaTeX files (all `.tex` files updated with improvements), edit summary (comprehensive report of changes made), before/after metrics (word count, page count reduction), compiled PDF (verified paper compiles correctly), quality assurance (technical accuracy preserved), sections for review (flagged areas needing author attention)

## Next Steps
Review flagged sections (check areas marked for author review), recompile paper (generate final PDF to see visual result), read through once (do final pass for flow and coherence), get co-author feedback (share edited version with collaborators), submit (if target page count met and quality high, submit to journal/conference), archive version (save edited version with date for future reference)

## Example Usage

**Scenario**: 18-page paper needs to be 12 pages for conference submission

**Steps**:
1. User invokes: `/edit-paper`
2. Skill asks: Paper (`papers/03_combined_recursive_bisection/`), Sections (All), Target (18 pages → 12 pages, 33% reduction), Level (Heavy Edit), Concerns ("Too verbose in intro and methodology")
3. Skill analyzes paper structure
4. Applies edits: Condenses Introduction (3 pages → 1.5 pages), simplifies Methodology explanations, consolidates Results subsections, moves detailed derivation to Appendix
5. Generates summary showing 35% word count reduction
6. Compiles paper successfully, now 11.5 pages
7. Flags Methodology Section 3.2 for author review

**Result**: Paper ready for submission, under page limit, technical accuracy preserved

## Related Skills
`/compile-latex` (compile LaTeX after editing to verify no errors), `/create-presentation-figures` (may need to regenerate figures for paper), `/update-docs` (similar editing workflow for documentation), `/create-session-archive` (archive major paper revisions)
