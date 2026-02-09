# Guide to the Portfolio — Plan

**Artifact Type**: PP3.2 (Optional Enhancement)
**Goal**: 2-3 page PDF overview for readers new to the research program
**Estimated Effort**: 1 day (8 hours)
**Status**: Not started

---

## Objective

Create a concise, accessible introduction to the 10-paper research program that:
- Orients interdisciplinary readers (political scientists, computer scientists, legal scholars)
- Provides recommended reading order
- Summarizes key findings from each paper in 1-2 sentences
- Defines technical terms used across papers
- Points to code repository for replication

**Target Audience**:
- Reviewers encountering portfolio for the first time
- Practitioners (state redistricting commissions)
- Policymakers and legal scholars
- Graduate students interested in algorithmic redistricting

---

## Proposed Structure

### Page 1: Overview

**Section 1: Research Program Summary** (1 paragraph)
- What problem does this portfolio address?
- What's the core innovation (recursive bisection + impossibility defense)?
- What's the scope (10 papers, 50 states, 3 census decades)?

**Section 2: The Big Picture** (2-3 bullet points)
- **Philosophical contribution**: Impossibility defense against gerrymandering
- **Technical contribution**: Edge-weighted graph partitioning for VRA compliance
- **Empirical contribution**: National-scale validation across 50 states × 3 decades
- **Legal contribution**: 42% threshold for VRA feasibility

**Section 3: Paper Architecture** (visual reference)
- Include the portfolio visualization from gerry-portfolio-visualization/
- Or a simplified text version:
  ```
  Foundation: recursive-bisection → establishes core method
  Extensions: edge-weighted (compactness), vra-compliance (demographics)
  Investigations: 7 papers exploring properties and alternatives
  ```

---

### Page 2: Paper Summaries

**Format**: Table with 4 columns

| # | Paper | Key Finding | Venue | Score |
|---|-------|-------------|-------|-------|
| 1 | recursive-bisection | Impossibility defense: algorithm can't see partisan data | APSR / Science | 8.4/10 (A) |
| 2 | edge-weighted-bisection | +56% compactness improvement with edge weighting | KDD / SIGSPATIAL | 8.2/10 (A-) |
| 3 | vra-compliance | +69 MM district surplus over enacted plans | APSR / JOP | 8.0/10 (A-) |
| 4 | threshold-analysis | 42% state minority threshold for VRA feasibility | APSR | 7.9/10 (B+) |
| 5 | cross-census-validation | 3.2× variance geography > time (algorithm stable) | SIGSPATIAL | 7.8/10 (B+) |
| 6 | compactness-tradeoff | Non-MM districts gain +7.5% compactness (not sacrifice) | APSR | 7.7/10 (B+) |
| 7 | temporal-stability | +14pt tract retention: recursive > n-way stability | Political Analysis | 7.5/10 (B+) |
| 8 | multi-vs-edge | Constraint conflict: edge-weighting beats multi-constraint | OR/MS | 7.3/10 (B) |
| 9 | adaptive-bisection | Tree structure irrelevant with α≥5 edge-weighting | Algorithm venues | 7.1/10 (B) |
| 10 | nway-vs-recursive | Statistical equivalence: both methods viable | Comparative study | 6.8/10 (B-) |

**Below table**: One-sentence description of each paper's contribution

---

### Page 3: Getting Started

**Section 1: Recommended Reading Order**

**For Political Scientists**:
1. Start with **recursive-bisection** (impossibility defense, philosophy)
2. Then **vra-compliance** (demographic representation)
3. Then **threshold-analysis** (42% finding, legal implications)

**For Computer Scientists**:
1. Start with **edge-weighted-bisection** (algorithm innovation)
2. Then **multi-vs-edge** (constraint conflict theory)
3. Then **adaptive-bisection** (parameter sensitivity)

**For Legal Scholars**:
1. Start with **recursive-bisection** (Huntington-Hill precedent)
2. Then **vra-compliance** (+69 MM surplus)
3. Then **threshold-analysis** (42% threshold, Gingles implications)
4. Then **compactness-tradeoff** (VRA-compactness myths)

**For Practitioners** (redistricting commissions):
1. Start with **recursive-bisection** (core method)
2. Then **edge-weighted-bisection** (implementation details)
3. Then **vra-compliance** (VRA compliance approach)
4. Skim others as needed for specific questions

**Section 2: Glossary of Key Terms**

- **Census tracts**: Geographic units used as atomic partitioning units (~4,000 residents)
- **METIS**: Multilevel graph partitioning library developed by Karypis & Kumar
- **Edge-weighted partitioning**: Assigning higher costs to edges between similar nodes
- **Majority-minority (MM) districts**: Districts where minority voters ≥50% of VAP
- **Polsby-Popper compactness**: 4π × Area / Perimeter², range [0,1], higher = more compact
- **Recursive bisection**: Hierarchical method splitting graph into two parts repeatedly
- **N-way partitioning**: Flat method dividing graph into k parts simultaneously
- **Impossibility defense**: Algorithm cannot gerrymander because it cannot see partisan data
- **VRA (Voting Rights Act)**: Federal law prohibiting minority vote dilution

**Section 3: Code and Data**

- **Replication repository**: [Link to GitHub when/if released]
- **Census data sources**: U.S. Census Bureau redistricting files (2000/2010/2020)
- **METIS library**: [http://glaros.dtc.umn.edu/gkhome/metis/metis/overview](http://glaros.dtc.umn.edu/gkhome/metis/metis/overview)
- **Contact**: [Author email for questions]

**Section 4: Citation Guide**

If citing the entire research program:
```
Della-Libera, G. (2026). Recursive bisection for congressional redistricting:
A 10-paper research program. [Details TBD based on publication status]
```

If citing specific papers:
```
Della-Libera, G. (2026). Recursive bisection for congressional redistricting:
Extending Huntington-Hill to boundary design. American Political Science Review.
```

---

## Design Specifications

**Format**:
- PDF, 2-3 pages max
- Single-column, 11pt font
- Generous margins (1" all sides)
- Section headers bold 14pt
- Body text 11pt, tables 10pt

**Style**:
- Professional but accessible (avoid jargon)
- Use active voice
- Short sentences and paragraphs
- Minimal math/equations
- Visual elements (table, small diagram)

**Tone**:
- Welcoming to interdisciplinary readers
- Emphasizes breadth and coherence of program
- Highlights practical implications
- Points readers to right papers based on interests

---

## Deliverables

1. **Source**: `guide-to-portfolio.tex` or `guide-to-portfolio.md`
2. **Output**: `guide-to-portfolio.pdf` (2-3 pages)
3. **Web version** (optional): Convert to HTML for website

---

## Distribution Channels

This guide will be:
- Included as supplementary material with paper submissions
- Posted on research website/GitHub repository
- Sent to potential collaborators and reviewers
- Distributed at conference presentations
- Shared with state redistricting commissions

---

## Examples to Reference

**Existing portfolio guides**:
- MGGG Redistricting Lab: [https://mggg.org/](https://mggg.org/) (project overviews)
- Dave's Redistricting App: Documentation section
- Brennan Center redistricting guides

**Academic paper overviews**:
- NeurIPS tutorial papers (concise, accessible)
- JMLR survey papers (structured summaries)
- ACM SIGMOD "Database Principles" tutorials

---

## Implementation Steps

1. **Draft outline**: Expand structure above with actual content
2. **Extract key findings**: Pull 1-sentence summaries from abstracts
3. **Create table**: Paper metadata + scores from REVIEW_PANEL.md
4. **Write glossary**: Define 10-15 most common technical terms
5. **Design layout**: LaTeX template or Markdown → PDF
6. **Review**: Get feedback from 2-3 readers (different backgrounds)
7. **Iterate**: Revise based on clarity feedback
8. **Finalize**: Export PDF, post online

---

## Next Actions

- [ ] Draft Section 1 (Research Program Summary)
- [ ] Extract paper summaries from abstracts
- [ ] Create recommended reading order for each audience
- [ ] Write glossary entries for key terms
- [ ] Design LaTeX/Markdown template
- [ ] Get feedback from interdisciplinary reviewer
- [ ] Finalize and distribute

---

**Created**: 2026-02-08
**Panel Reference**: REVIEW_PANEL.md Section V, PP3.2
**Related Artifacts**: gerry-portfolio-visualization/, gerry-replication-materials/
