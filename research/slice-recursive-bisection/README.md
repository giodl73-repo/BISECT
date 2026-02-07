# Recursive Bisection for Congressional Redistricting

**Paper**: Extending Huntington-Hill to Boundary Design
**Author**: Giovanni Della-Libera
**Status**: Draft Complete - Ready for Panel Review
**Target Venue**: APSR (American Political Science Review)

## Overview

This paper extends the Huntington-Hill apportionment precedent to congressional redistricting through recursive graph bisection. The core argument: if mathematical objectivity resolved *how many* seats each state receives, similar principles can determine *where* district boundaries are drawn.

## Core Contribution

A redistricting algorithm with **structural immunity to partisan manipulation**:
- Operates exclusively on census data (population + adjacency)
- Cannot access partisan information (voting patterns, demographics, party affiliation)
- Produces reproducible, verifiable results
- Guarantees contiguity and achieves strong population balance (2.79% mean deviation)
- The **impossibility defense**: algorithm cannot gerrymander because it cannot see partisan data

## Key Findings

1. **Technical Feasibility**: Successfully handles all 50 states (Wyoming's 1 district to California's 52)
2. **Population Balance**: 86% of districts within ±5% of ideal population
3. **Geographic Patterns**: Democratic advantage in district count (56.5%) reflects urban concentration and geographic sorting, not algorithmic bias
4. **Compactness**: Middle range compared to existing plans; improvements possible with edge-weighted optimization

## Structure

- **Section 1**: Introduction and Huntington-Hill precedent as philosophical foundation
- **Section 2**: Detailed examination of why Huntington-Hill succeeded and how principles extend to redistricting
- **Section 3**: Recursive bisection algorithm, METIS, water-based adjacency handling
- **Section 4**: Population balance and compactness results for all 435 districts
- **Section 5**: Political analysis with the "impossibility defense" - algorithm cannot see partisan data
- **Section 6**: Discussion of advantages, limitations, comparisons to alternatives, policy implications
- **Section 7**: Conclusion emphasizing process fairness over outcome fairness

## Compilation

```bash
cd C:/src/apportionment/research/slice-recursive-bisection
pdflatex main.tex
bibtex main
pdflatex main.tex
pdflatex main.tex
```

Or use MiKTeX console to compile with bibliography.

## Figures

All figures are referenced from the existing draft at:
`../../artifacts/papers/01_recursive_bisection/figures/`

Required figures:
- `example_state_round_0.png` / `_1.png` / `_2.png` - Minnesota visualization
- `odd_example_round_0.png` / `_1.png` / `_2.png` - Alabama visualization
- `political_margin_hist.png` - Democratic margin distribution

## Target Reviewers (7 from REVIEWER-DATABASE.md)

**Political Science** (3):
- Jonathan Rodden (Stanford) - Political geography, gerrymandering
- Jowei Chen (Michigan) - Automated redistricting, neutrality
- Moon Duchin (Rutgers) - Metric geometry, fairness

**Algorithms** (2):
- George Karypis (Minnesota) - METIS, graph partitioning [CRITICAL]
- Ümit Çatalyürek (Georgia Tech) - Hypergraph partitioning

**Law** (1):
- Richard Pildes (NYU Law) - Election law, constitutional doctrine

**GIS** (1):
- Michael Goodchild (UCSB) - GIS theory, spatial analysis

## Differences from Original Draft

This rewrite strengthens:

1. **Philosophical framing**: Huntington-Hill as precedent for mathematical governance, not just historical curiosity
2. **The impossibility defense**: Clearer articulation of structural immunity vs. intent-based arguments
3. **Process vs. outcome fairness**: Explicit prioritization with philosophical justification
4. **Political geography framing**: Better integration of Chen/Rodden "unintentional gerrymandering" findings
5. **Discussion of limitations**: More honest about what algorithms cannot achieve
6. **Policy implications**: Clearer adoption pathways and public acceptance strategies
7. **Democratic legitimacy**: Ties throughout to questions of trust, transparency, and procedural fairness

## Next Steps (Panel Review Lifecycle)

1. ✅ **Draft**: Complete - main.tex + 7 sections written
2. ⏳ **Panel Review Round 1**: Assign 7 reviewers, generate individual reviews
3. ⏳ **Synthesis**: Consolidate reviews into P1/P2/P3 priority items
4. ⏳ **Revision**: Address P1 blocking issues
5. ⏳ **Recheck**: Round 2 reviews, score evaluation
6. ⏳ **Ready**: Panel review (cross-portfolio level)
7. ⏳ **Submit**: Target APSR
8. ⏳ **Accepted**: Publication

---

**Reference**: Original draft at `artifacts/papers/01_recursive_bisection/`
**Panel State**: `_panel.yaml`
**Reviewer Database**: `../REVIEWER-DATABASE.md`
