# A.3 — Portfolio Visualization Guide

**Paper Type**: Portfolio Guide with Visualizations
**Status**: Planned
**Target Venue**: Supplementary Material / Standalone Portfolio Documentation
**Format**: LaTeX document (10-15 pages) with 8-10 figures
**Target Audience**: Researchers, reviewers, students, policymakers navigating the 28-paper portfolio

---

## Purpose

Create a **sequential document with visualizations** that guides readers through the 28-paper congressional redistricting research portfolio. Unlike a single static infographic, this document walks readers page-by-page through the portfolio structure, explaining the five tracks, paper dependencies, narrative flow, and recommended reading orders.

**Key Innovation**: Pedagogical approach to portfolio navigation—similar to how Paper B.2's laymen_guide explains edge-weighted bisection, this guide explains the portfolio architecture itself.

---

## Target Audience

1. **Journal Reviewers**: Understanding the scope and structure before diving into technical papers
2. **New Researchers**: Entering the redistricting field and wanting to understand the landscape
3. **Students**: Graduate students looking for research directions or thesis topics
4. **Policymakers**: Legislative staff or redistricting commissioners seeking evidence-based approaches
5. **Peer Researchers**: Other academics wanting to cite or build on specific components

---

## Document Objectives

1. **Demystify the portfolio**: Explain why 28 papers exist and how they fit together
2. **Enable efficient navigation**: Help readers find the papers most relevant to their interests
3. **Visualize dependencies**: Show which papers build on or reference others
4. **Explain the narrative**: Walk through the five-paper story (track heads A.0→B.0→C.0→D.0→E.0)
5. **Provide reading paths**: Recommend sequences for different audiences and purposes
6. **Summarize contributions**: One-paragraph descriptions of each paper's unique contribution

---

## Document Structure

### Section 1: Introduction (1-2 pages)

**Purpose**: Orient the reader to the portfolio and explain this document's role.

**Content**:
- **The Redistricting Problem**: Brief context on why algorithmic redistricting matters (2-3 paragraphs)
- **Portfolio Scope**: 28 papers across 5 thematic tracks, 3 census years, 435 districts, 50 states
- **How to Use This Guide**: What readers will learn and how to navigate the document
- **Quick Reference**: Table showing paper count by track (A=5, B=5, C=5, D=4, E=5, total=28)

**Visualization**:
- **Figure 1.1**: High-level portfolio overview diagram showing 5 colored tracks with paper counts

### Section 2: The Five Tracks (2-3 pages)

**Purpose**: Explain the portfolio architecture and why papers are organized into tracks.

**Content**:
- **Track Architecture**: Definition of tracks as thematic groupings with hierarchical IDs (A.0, A.1, A.2...)
- **Track Heads (.0 papers)**: Special role as synthesis papers that tell complete stories
- **Track A — Overview & Synthesis** (5 papers): Meta-level portfolio papers
- **Track B — Algorithm Design** (5 papers): How the system was architected and justified
- **Track C — Validation** (5 papers): Multi-faceted robustness testing and cross-validation
- **Track D — Voting Rights Act** (4 papers): Minority representation and VRA compliance
- **Track E — Experimental Alternatives** (5 papers): What-if scenarios and alternative systems

**Visualization**:
- **Figure 2.1**: Track hierarchy diagram showing all 28 papers organized by track
  - Each track colored differently (A=purple, B=blue, C=green, D=orange, E=red)
  - Track heads (.0) visually distinguished (larger boxes or bold borders)
  - Sub-papers (.1, .2, .3...) shown beneath each track head
  - Include paper slugs for identification

**Example layout**:
```
Track A (Overview & Synthesis)
├─ A.0 synthesis-metapaper
├─ A.1 portfolio-guide
├─ A.2 portfolio-summary
├─ A.3 portfolio-visualization (this document)
└─ A.4 replication-materials

Track B (Algorithm Design)
├─ B.0 algorithm-design-overview
├─ B.1 recursive-bisection
├─ B.2 edge-weighted-bisection
├─ B.3 multi-vs-edge
├─ B.4 adaptive-bisection
└─ B.5 nway-vs-recursive-general

[... continue for C, D, E]
```

### Section 3: The Five-Paper Story (2-3 pages)

**Purpose**: Explain the narrative arc through the five track head papers.

**Content**:
- **Why Track Heads Matter**: Someone can read ONLY the five .0 papers and understand the entire research program
- **The Narrative Flow**: Sequential story from A.0 through E.0

**For each track head (.0 paper)**:
1. **A.0 (Synthesis Metapaper)**: "What we built"
   - Big-picture findings, national results, policy implications
   - Target venue: Science/Nature
   - Key message: Algorithmic redistricting is feasible, transparent, and fairer than status quo

2. **B.0 (Algorithm Design Overview)**: "How we designed it"
   - Architectural decisions and justifications (why METIS, why recursive bisection, why census tracts, etc.)
   - Target venue: ACM TSAS / SIAM Journal on Applied Mathematics
   - Key message: Every design choice was deliberate and justified

3. **C.0 (Validation Overview)**: "How we validated it"
   - Multi-faceted testing framework (MAUP, cross-census, temporal, longitudinal, partisan)
   - Target venue: Political Analysis / APSR
   - Key message: The algorithm is robust, stable, and fair across multiple dimensions

4. **D.0 (VRA Compliance)**: "How it handles minority representation"
   - Voting Rights Act requirements, majority-minority districts, community preservation
   - Target venue: Election Law Journal / Journal of Politics
   - Key message: Algorithmic redistricting can meet VRA mandates while maintaining compactness

5. **E.0 (Experimental Overview)**: "What alternatives we explored"
   - Five experimental systems with different trade-offs (multi-member, county-based, national, partisan, party-based)
   - Target venue: Electoral Studies
   - Key message: No perfect system—all have trade-offs, but our baseline is a strong default

**Visualization**:
- **Figure 3.1**: Narrative flow diagram showing the five track heads connected by arrows
  - Each track head as a box with title, venue, and 2-3 bullet key findings
  - Arrows showing logical progression (A.0→B.0→C.0→D.0→E.0)
  - Visual metaphor: pipeline or progression from "what" to "how" to "validation" to "VRA" to "alternatives"

### Section 4: Track Deep Dives (3-4 pages)

**Purpose**: Show how sub-papers (.1, .2, .3...) feed into track heads (.0).

**Content**: One subsection per track showing paper dependencies.

#### 4.1 Track B: Algorithm Design
- **B.0 Overview**: Synthesizes findings from B.1-B.5
- **B.1 (Recursive Bisection)**: Core algorithm, multilevel partitioning
- **B.2 (Edge-Weighted Bisection)**: Geographic compactness via boundary-length optimization
- **B.3 (Multi-Member vs Edge-Weighted)**: Why edge weights beat multi-member optimization
- **B.4 (Adaptive Bisection)**: Population imbalance tolerance and geographic preservation
- **B.5 (N-way vs Recursive)**: Why recursive bisection beats simultaneous n-way partitioning
- **Dependency**: B.0 synthesizes B.1-B.5 into cohesive design narrative

#### 4.2 Track C: Validation
- **C.0 Overview**: Multi-faceted validation framework
- **C.1 (MAUP Sensitivity)**: Robustness to scale changes (tracts vs blocks)
- **C.2 (Cross-Census Validation)**: Consistency across 2000/2010/2020 census years
- **C.3 (Temporal Stability)**: Geographic stability over time
- **C.4 (Longitudinal Analysis)**: How districts evolve across decades
- **C.5 (Efficiency Gap)**: Partisan fairness metrics (wasted votes analysis)
- **Dependency**: C.0 argues algorithm is robust, stable, and fair by synthesizing C.1-C.5

#### 4.3 Track D: Voting Rights Act
- **D.0 Overview**: VRA compliance and minority representation
- **D.1 (Threshold Analysis)**: Impact of minority concentration thresholds
- **D.2 (N-way vs Recursive for VRA)**: Which approach better preserves minority districts
- **D.3 (Compactness Tradeoff)**: Tension between compactness and VRA compliance
- **Dependency**: D.0 shows VRA compliance is achievable, with D.1-D.3 exploring constraints

#### 4.4 Track E: Experimental Alternatives
- **E.0 Overview**: "No free lunch" — all systems have trade-offs
- **E.1 (Multi-Member Districts)**: Proportional representation via MMDs
- **E.2 (County Representation)**: Preserving county boundaries
- **E.3 (National Redistricting)**: Single national system vs 50 state systems
- **E.4 (Partisan Similarity)**: Districts with balanced partisan composition
- **E.5 (Party-Based Allocation)**: Proportional representation by party vote share
- **Dependency**: E.0 compares E.1-E.5 to show trade-offs in alternative designs

**Visualization**:
- **Figure 4.1**: Track B dependency diagram (B.0 at top, B.1-B.5 feeding into it)
- **Figure 4.2**: Track C dependency diagram (C.0 at top, C.1-C.5 feeding into it)
- **Figure 4.3**: Track D dependency diagram (D.0 at top, D.1-D.3 feeding into it)
- **Figure 4.4**: Track E dependency diagram (E.0 at top, E.1-E.5 feeding into it)

**Diagram style**: Flowchart with track head at top, arrows pointing up from sub-papers, brief labels on arrows explaining contribution (e.g., "edge weights → compactness", "MAUP → robustness")

### Section 5: Reading Orders (2 pages)

**Purpose**: Recommend reading paths for different audiences and purposes.

**Content**: Multiple reading paths with explanations.

#### 5.1 The Quick Path (5 papers, ~2-3 hours)
**Audience**: Reviewers, busy policymakers, anyone wanting the full story quickly
**Papers**: A.0, B.0, C.0, D.0, E.0 (track heads only)
**Rationale**: Track heads are self-contained and tell the complete story

#### 5.2 The Algorithmic Deep Dive (6 papers, ~1 day)
**Audience**: Computer scientists, algorithm researchers
**Papers**: A.0 → B.0 → B.1 → B.2 → B.3 → B.5
**Rationale**: Focuses on algorithm design, optimization techniques, and performance comparisons

#### 5.3 The Validation & Robustness Path (6 papers, ~1 day)
**Audience**: Political scientists, statisticians, reviewers concerned with methodology
**Papers**: A.0 → C.0 → C.1 → C.2 → C.3 → C.5
**Rationale**: Demonstrates multi-faceted validation and partisan fairness

#### 5.4 The VRA & Minority Representation Path (5 papers, ~4-5 hours)
**Audience**: Election law scholars, civil rights advocates, VRA experts
**Papers**: A.0 → D.0 → D.1 → D.2 → D.3
**Rationale**: Focuses specifically on minority representation and VRA compliance

#### 5.5 The Policy Explorer Path (6 papers, ~1 day)
**Audience**: Policymakers, legislative staff, redistricting commissioners
**Papers**: A.0 → B.0 → C.0 → D.0 → E.1 → E.3
**Rationale**: Covers core system, validation, VRA, plus alternative systems for policy comparison

#### 5.6 The Complete Researcher Path (28 papers, ~2-3 weeks)
**Audience**: PhD students, peer researchers building on this work
**Papers**: All 28 papers in dependency order
**Rationale**: Full context for extending or building on the research program
**Order**: A.1 → A.2 → (tracks in parallel) → A.0 → A.3 (this doc) → A.4

**Visualization**:
- **Figure 5.1**: Six reading paths shown as flowcharts with paper boxes and arrows
  - Color-coded by track (matching earlier figures)
  - Time estimates shown for each path
  - Difficulty indicators (beginner/intermediate/advanced)

### Section 6: Paper Summaries (3-4 pages)

**Purpose**: One-paragraph description of each paper's unique contribution.

**Content**: For each of the 28 papers, provide:
- **Track ID** (e.g., B.2)
- **Title** (e.g., "Edge-Weighted Recursive Bisection")
- **Target Venue** (e.g., SIAM Journal on Applied Mathematics)
- **Status** (Ready / Planned / In Progress)
- **Core Contribution** (2-4 sentences): What this paper uniquely contributes that others don't
- **Key Result** (1 sentence): Main finding or metric
- **Dependencies** (if any): Which papers does this build on?

**Organization**: Group by track (A.0-A.4, B.0-B.5, C.0-C.5, D.0-D.3, E.0-E.5)

**Example entry**:
> **B.2 — Edge-Weighted Recursive Bisection**
> *Target*: SIAM Journal on Applied Mathematics | *Status*: Ready (Panel Score: 8.5/10)
> **Contribution**: Introduces geographic compactness optimization by assigning edge weights equal to real boundary lengths (measured in meters). Shows that encoding geography into the graph structure improves compactness by 56% over unweighted baseline and 20% over enacted 2020 maps, with strongest gains in gerrymandered states (Illinois +174%, Louisiana +104%). Demonstrates cross-census consistency, proving geography—not politics—drives the algorithm.
> **Key Result**: Mean Polsby-Popper 0.367 nationally, beating 37 of 50 state enacted maps.
> **Dependencies**: Builds on B.1 (recursive bisection foundation).

**Visualization**:
- **Figure 6.1**: Summary table with all 28 papers showing Track ID, Title, Venue, Status, Score (if reviewed)
  - Color-coded rows by track
  - Status icons (✓ Ready, ⧗ In Progress, ○ Planned)

### Section 7: Cross-Paper Dependencies (1-2 pages)

**Purpose**: Show which papers cite, reference, or build on others.

**Content**:
- **Dependency Types**:
  - **Foundational**: Paper X provides methodology used in Paper Y
  - **Synthesis**: Paper X aggregates findings from Papers Y, Z, W
  - **Extension**: Paper X extends Paper Y's approach to new domain
  - **Comparison**: Paper X compares approaches from Papers Y and Z
  - **Validation**: Paper X validates claims from Paper Y

**Major Dependencies**:
- A.0 (synthesis) depends on ALL other papers (23 papers)
- B.0 synthesizes B.1-B.5 (5 papers)
- C.0 synthesizes C.1-C.5 (5 papers)
- D.0 synthesizes D.1-D.3 (3 papers)
- E.0 synthesizes E.1-E.5 (5 papers)
- B.2 (edge-weighted) builds on B.1 (recursive bisection)
- B.3 (multi-vs-edge) compares B.2 with alternative
- B.4 (adaptive) extends B.2 with population tolerance
- C.5 (efficiency-gap) validates partisan fairness claims from B.0
- D.2 (nway-vs-recursive-vra) extends D.0 comparison to VRA context

**Visualization**:
- **Figure 7.1**: Full portfolio dependency graph
  - All 28 papers as nodes
  - Arrows showing dependencies
  - Track heads (A.0, B.0, C.0, D.0, E.0) prominently positioned
  - Color-coded by track
  - Arrow styles indicating dependency type (solid=foundational, dashed=synthesis, dotted=extension)

**Note**: This is the most complex figure—may require full-page or fold-out format.

### Section 8: Timeline & Status (1 page)

**Purpose**: Show the research progression and current completion status.

**Content**:
- **Development Phases**:
  - Phase 1 (2023-2024): Core algorithm development (Track B papers)
  - Phase 2 (2024): Validation framework (Track C papers)
  - Phase 3 (2024-2025): VRA analysis (Track D papers)
  - Phase 4 (2025): Experimental alternatives (Track E papers)
  - Phase 5 (2025-2026): Synthesis and portfolio documentation (Track A papers)

- **Current Status** (as of February 2026):
  - **Ready for Submission** (10 papers): Panel-reviewed with scores 7.0-8.8
  - **In Progress** (11 papers): Being written or under panel review
  - **Planned** (5 papers): Specifications complete, writing not started
  - **Extracted/Support** (2 papers): A.1, A.2 extracted from portfolio-guide

- **Submission Timeline**:
  - Q1 2026: Track A papers (A.0, A.3, A.4)
  - Q2 2026: Track B papers (B.0-B.5) to algorithm/CS venues
  - Q3 2026: Track C papers (C.0-C.5) to political science venues
  - Q4 2026: Tracks D and E to specialized venues

**Visualization**:
- **Figure 8.1**: Timeline Gantt chart showing development phases and paper completion
- **Figure 8.2**: Status pie chart (10 ready, 11 in progress, 5 planned, 2 extracted)

### Section 9: How to Navigate the Codebase (1 page)

**Purpose**: Connect papers to code artifacts and replication materials.

**Content**:
- **Repository Structure**: Overview of `research/` directory with track-based organization
- **Paper Directories**: Format `TRACK.INDEX+slug/` (e.g., `B.2+edge-weighted-bisection/`)
- **Artifacts**: Where to find figures, data, and supplementary materials
- **Replication**: Pointer to A.4 (replication-materials) for running experiments
- **Code Mapping**: Which papers correspond to which modules in `src/apportionment/`

**Example mapping**:
- B.1 → `src/apportionment/partition/recursive_bisection.py`
- B.2 → `src/apportionment/data/adjacency.py` (edge weights)
- C.1 → `scripts/analysis/maup_sensitivity.py`
- C.5 → `scripts/political/efficiency_gap.py`

**Visualization**:
- **Figure 9.1**: Directory tree showing track-based organization
- **Figure 9.2**: Code-to-paper mapping table

### Section 10: Conclusion & Next Steps (1 page)

**Purpose**: Wrap up and point readers to next actions.

**Content**:
- **Portfolio Summary**: 28 papers, 5 tracks, comprehensive algorithmic redistricting research program
- **Key Takeaway**: Track heads tell the complete story; sub-papers provide depth and validation
- **For Reviewers**: Recommended starting with A.0 (synthesis) then track heads
- **For Researchers**: Identify relevant papers using reading paths (Section 5)
- **For Policymakers**: Focus on A.0, B.0, C.0, D.0 for evidence-based recommendations
- **Feedback Welcome**: Contact information and how to engage with the research

**Call to Action**:
- Download papers from [repository URL]
- Explore interactive maps at [dashboard URL]
- Run code yourself using A.4 (replication materials)
- Cite specific papers using provided BibTeX

---

## Visualizations Summary

The document requires 8-10 high-quality figures:

1. **Figure 1.1**: Portfolio overview (5 tracks, 28 papers)
2. **Figure 2.1**: Track hierarchy diagram (all 28 papers organized by track)
3. **Figure 3.1**: Five-paper story narrative flow (A.0→B.0→C.0→D.0→E.0)
4. **Figure 4.1-4.4**: Track dependency diagrams (B, C, D, E tracks showing .0 ← .1-.5)
5. **Figure 5.1**: Six reading paths flowchart
6. **Figure 6.1**: Summary table (28 papers with status/scores)
7. **Figure 7.1**: Full portfolio dependency graph (all 28 papers, all dependencies)
8. **Figure 8.1**: Timeline Gantt chart
9. **Figure 8.2**: Status pie chart
10. **Figure 9.1**: Directory tree
11. **Figure 9.2**: Code-to-paper mapping table

**Visual Style**:
- Color-coded by track (A=purple, B=blue, C=green, D=orange, E=red)
- Track heads (.0 papers) visually distinguished (bold borders, larger boxes)
- Clean, professional diagrams (TikZ or similar for LaTeX integration)
- Consistent iconography (✓ ready, ⧗ in progress, ○ planned)

---

## Writing Guidelines

### Tone and Style
- **Accessible but precise**: Explain portfolio architecture clearly without oversimplifying
- **Pedagogical**: Similar to B.2's laymen_guide, but for portfolio navigation not algorithms
- **Action-oriented**: Help readers quickly find what they need
- **Visual-first**: Every major concept should have a supporting figure

### LaTeX Structure
```latex
\documentclass[11pt]{article}
\usepackage[margin=1in]{geometry}
\usepackage{graphicx}
\usepackage{tikz}
\usepackage{booktabs}
\usepackage{hyperref}
\usepackage{enumitem}

\title{Navigating the Congressional Redistricting Research Portfolio}
\author{A Visual Guide to 28 Papers Across Five Thematic Tracks}
\date{February 2026}
```

### Section Templates
Each section should follow:
1. **Section title** (e.g., "Section 3: The Five-Paper Story")
2. **Purpose statement** (1-2 sentences explaining why this section exists)
3. **Content** (explanatory text with examples)
4. **Visualization** (referenced figure with caption)
5. **Transition** (1 sentence leading to next section)

### Figure Guidelines
- All figures saved to `research/A.3+portfolio-visualization/figures/`
- LaTeX figure blocks with descriptive captions
- Reference figures in text: "Figure 2.1 shows the complete track hierarchy..."
- Figure captions should be self-explanatory (reader can understand figure without reading main text)

---

## Dependencies

**This paper depends on**:
- **PAPERS.md**: Source of truth for all 28 papers, their tracks, status, and venues
- **Track head plan.md files**: B.0, C.0, E.0 plans provide detailed track descriptions
- **A.1 (portfolio-guide)**: 2-page guide with paper summaries and glossary
- **A.2 (portfolio-summary)**: 6-page research statement with detailed descriptions

**Papers that depend on this**:
- **A.0 (synthesis-metapaper)**: Will reference A.3 as supplementary navigation material
- **A.4 (replication-materials)**: Will link to A.3 for understanding which papers correspond to which experiments

---

## Success Criteria

This document succeeds if:

1. ✓ A reviewer can read it in 30 minutes and understand the portfolio structure
2. ✓ A researcher can identify relevant papers for their specific interest (VRA, algorithms, validation)
3. ✓ A policymaker can follow a reading path without getting lost
4. ✓ A student can understand dependencies and know which papers to read in which order
5. ✓ All 28 papers are accurately represented with up-to-date status and descriptions
6. ✓ Visualizations are clear, professional, and publication-ready
7. ✓ The document stands alone—no need to consult other materials to understand the portfolio

---

## Target Metrics

- **Length**: 10-15 pages (excluding figures)
- **Figures**: 8-10 high-quality diagrams/charts
- **Reading Time**: 30-45 minutes for complete read, 5-10 minutes for quick scan
- **Audience Reach**: Useful for 5+ distinct audience types (reviewers, researchers, students, policymakers, peers)
- **Reusability**: Can be updated as papers progress through submission/publication

---

## Next Steps for /panel:author

When writing this document:

1. **Start with Section 2 (Track hierarchy diagram)**: This is the foundation—get it right first
2. **Create all figures early**: Having visuals helps structure the narrative
3. **Use PAPERS.md as ground truth**: Don't guess paper details—reference authoritative source
4. **Test reading paths**: Make sure recommended sequences actually work
5. **Get feedback on figures**: Share drafts of key diagrams before finalizing
6. **Update regularly**: As papers progress, keep status and scores current
7. **Link to actual papers**: Use relative paths so readers can navigate to paper directories

**Estimated writing time**: 2-3 days for initial draft + 1 day for figure creation + 1 day for revisions

---

## Notes

- This document should be **living documentation**—update as portfolio evolves
- Consider generating some figures programmatically from PAPERS.md (e.g., status charts)
- May want to create both PDF (for printing) and HTML (for web dashboard) versions
- Could extract key figures for use in presentations or conference posters
- Track head papers (B.0, C.0, E.0) will likely reference this document in their introductions

**Related Documents**:
- A.1 (portfolio-guide): 2-page quick reference → complements this
- A.2 (portfolio-summary): 6-page research statement → provides more narrative context
- This document (A.3): 10-15-page visual guide → deepest portfolio navigation resource
