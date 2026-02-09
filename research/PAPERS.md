# Research Paper Portfolio — Track-Based Organization

**Created**: 2026-02-08
**Reorganized**: 2026-02-08
**Source of Truth**: This file defines the canonical track-based organization of all papers in the research program.

---

## Overview

This portfolio comprises **28 papers** organized into **5 thematic tracks** (A-E), where each track has a **head paper** (A.0, B.0, etc.) that tells the complete story, with **sub-papers** providing detailed evidence.

**Key Principle**: A reader can read **only the 5 track head papers** (A.0, B.0, C.0, D.0, E.0) and understand the entire research program.

**Track Structure**:
- **Track A** (Overview & Synthesis): 5 papers — What we did, why it matters, how to use it
- **Track B** (Algorithm Design): 6 papers — How we designed the algorithm and why these choices
- **Track C** (Validation & Analysis): 6 papers — How we validated it works (robustness, stability, fairness)
- **Track D** (VRA Compliance): 4 papers — How it handles minority representation
- **Track E** (Experimental Systems): 6 papers — What we learned from alternative systems

**Total**: 28 papers (5 track heads + 23 sub-papers)

---

## Track A — Overview & Synthesis (5 papers)

**Audience**: Interdisciplinary, general scientific community, policymakers
**Narrative**: What we did, why it matters, how to use it

| Track ID | Slug | Title | Type | Status | Target Venue |
|----------|------|-------|------|--------|--------------|
| **A.0** | synthesis-metapaper | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | Science/Nature synthesis | Planned | Science / Nature |
| **A.1** | portfolio-guide | Portfolio Guide: Reader's Introduction | 2-page guide | Ready | Supplementary material |
| **A.2** | portfolio-summary | Portfolio Summary: Research Statement | 6-page statement | Ready | Supplementary material |
| **A.3** | portfolio-visualization | Interactive Portfolio Visualization | Web app | Planned | GitHub Pages |
| **A.4** | replication-materials | Replication Materials and Data Archive | Archive | Planned | Zenodo / Dataverse |

**Track Head (A.0) Purpose**:
- Capstone synthesis for Science/Nature
- Tells full story: gerrymandering crisis → recursive bisection → impossibility defense → VRA surplus → 42% threshold
- Draws from all tracks (B, C, D, E)
- 2,500 words + 4 figures
- Written LAST (after tracks B-E established)

**Status**: A.1 and A.2 extracted from legacy gerry-portfolio-guide, A.3 and A.4 planned

---

## Track B — Algorithm Design (6 papers)

**Audience**: Computer scientists, operations researchers, algorithm designers
**Narrative**: How we designed the redistricting algorithm and why these architectural choices

| Track ID | Slug | Title | Current Status | Target Venue | Panel Score |
|----------|------|-------|----------------|--------------|-------------|
| **B.0** | algorithm-design-overview | Algorithmic Design for Congressional Redistricting: Method Selection and Architectural Decisions | Plan complete | ACM TSAS / SIAM | TBD |
| **B.1** | recursive-bisection | Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design | Ready (panel-reviewed) | APSR / Science | 8.4/10 (A) |
| **B.2** | edge-weighted-bisection | Edge-Weighted Graph Partitioning for Compact Congressional Districts | Ready (panel-reviewed) | KDD / SIGSPATIAL | 8.2/10 (A-) |
| **B.3** | multi-vs-edge | Why Single-Objective Edge-Weighting Outperforms Multi-Constraint Optimization | Ready (panel-reviewed) | OR/MS journals | 7.3/10 (B) |
| **B.4** | adaptive-bisection | Parameter Sensitivity in Recursive Bisection: Tree Structure Irrelevance with Edge-Weighting | Ready (panel-reviewed) | Algorithmic venues | 7.1/10 (B) |
| **B.5** | nway-vs-recursive-general | N-Way vs Recursive Bisection: General Architectural Comparison | Planned | ACM TSAS / SIAM | TBD |

**Track Head (B.0) Purpose**:
- Comprehensive justification of all algorithmic architecture decisions
- RQ1: Why graph partitioning? RQ2: Why recursive bisection? RQ3: Why edge-weighting?
- RQ4: Why METIS? RQ5: Why census tracts? RQ6: Parameter robustness?
- Evidence: References B.1-B.5 for detailed proofs
- 8,000-10,000 words, 6 figures

**Why Papers B.3 and B.4 belong in this track**:
- B.3 (multi-vs-edge): Explains core design decision (edge-weighting > multi-constraint)
- B.4 (adaptive-bisection): Demonstrates parameter robustness (algorithm design concern)
- Both justify **how we designed the algorithm**, not how we validated it

---

## Track C — Validation & Analysis (6 papers)

**Audience**: Political scientists, statisticians, election law scholars
**Narrative**: How we validated that the algorithm works across multiple dimensions

| Track ID | Slug | Title | Current Status | Target Venue | Panel Score |
|----------|------|-------|----------------|--------------|-------------|
| **C.0** | validation-overview | Validating Algorithmic Redistricting: A Multi-Faceted Approach | Plan complete | Political Analysis / APSR | TBD |
| **C.1** | maup-sensitivity | Spatial Resolution Sensitivity in Algorithmic Redistricting: Tracts, Block Groups, and Blocks | Planned | IJGIS / Political Analysis | TBD |
| **C.2** | cross-census-validation | Validating Algorithmic Redistricting Across Census Decades: A Slice-Based Methodology | Ready (panel-reviewed) | SIGSPATIAL | 7.8/10 (B+) |
| **C.3** | temporal-stability | Temporal Stability in Algorithmic Redistricting: Hierarchical Advantage Across Census Cycles | Ready (panel-reviewed) | Political Analysis | 7.5/10 (B+) |
| **C.4** | longitudinal-analysis | Twenty Years of Congressional Redistricting: Temporal Trends 2000-2020 | Planned | APSR / Political Analysis | TBD |
| **C.5** | efficiency-gap-analysis | Efficiency Gap and Partisan Bias in Algorithmic Redistricting | Ready (panel-reviewed) | Election Law Journal | TBD |

**Track Head (C.0) Purpose**:
- Multi-faceted validation framework across 5 dimensions
- RQ1: Spatial robustness (MAUP), RQ2: Cross-census consistency, RQ3: Temporal stability
- RQ4: Longitudinal trends (20 years), RQ5: Partisan fairness (efficiency gap)
- Evidence: References C.1-C.5 for detailed statistical analyses
- 8,000-10,000 words, 6 figures

**Validation Dimensions**:
1. **C.1 (MAUP)**: Robustness across spatial resolutions (tracts, block groups, blocks)
2. **C.2-C.4 (Temporal)**: Cross-census methodology, boundary stability, longitudinal trends
3. **C.5 (Partisan)**: Efficiency gap analysis (62% bias reduction vs enacted plans)

---

## Track D — Voting Rights Act Compliance (4 papers)

**Audience**: Constitutional law scholars, civil rights advocates, election officials
**Narrative**: How algorithmic redistricting handles minority representation

| Track ID | Slug | Title | Current Status | Target Venue | Panel Score |
|----------|------|-------|----------------|--------------|-------------|
| **D.0** | vra-compliance | Algorithmic Redistricting Exceeds Voting Rights Act Requirements Without Explicit Racial Targeting | Ready (panel-reviewed) | APSR / JOP | 8.0/10 (A-) |
| **D.1** | threshold-analysis | The 42% Threshold: State Minority Percentage and Majority-Minority District Feasibility | Ready (panel-reviewed) | APSR / Law Review | 7.9/10 (B+) |
| **D.2** | nway-vs-recursive-vra | N-Way vs Recursive Bisection: Statistical Equivalence for VRA Compliance | Ready (panel-reviewed) | Comparative study | 6.8/10 (B-) |
| **D.3** | compactness-tradeoff | The VRA-Compactness Tradeoff: Pareto Frontiers in Algorithmic Redistricting | Ready (panel-reviewed) | APSR | 7.7/10 (B+) |

**Track Head (D.0) Purpose**:
- **EXISTING PAPER** — vra-compliance already works as standalone track head
- Shows: Purely algorithmic approach (no racial data) produces VRA-compliant districts
- Key findings: +69 MM district surplus, exceeds VRA requirements
- Addresses: Constitutional concerns, legal framework, minority representation
- Evidence: References D.1-D.3 for detailed analyses

**Why only 4 papers (smallest track)**:
- D.0 (vra-compliance) is comprehensive and stands alone strongly
- D.1: Feasibility limits (42% demographic threshold)
- D.2: Method robustness for VRA (n-way vs recursive comparison)
- D.3: VRA-compactness trade-off analysis (Pareto frontiers)

---

## Track E — Experimental Systems (6 papers)

**Audience**: Political theorists, electoral reform advocates, comparative politics scholars
**Narrative**: What if we redesigned representation from scratch?

| Track ID | Slug | Title | Current Status | Target Venue | Priority |
|----------|------|-------|----------------|--------------|----------|
| **E.0** | experimental-overview | Alternative Representation Systems: Algorithmic Explorations Beyond Single-Member Districts | Plan complete | Electoral Studies | Research |
| **E.1** | multi-member-districts | Multi-Member Districts and Proportional Representation | Planned | Electoral Studies | Research |
| **E.2** | county-representation | Direct County Representation: Using Existing Political Boundaries | Planned | State Politics & Policy | Research |
| **E.3** | national-redistricting | National Redistricting Without State Boundaries: A Geometric Baseline | Planned | Comparative Politics | Research |
| **E.4** | partisan-similarity-districts | Partisan Similarity Districts: Algorithmic Safe Seats | Planned | AJPS | Research |
| **E.5** | party-based-allocation | Proportional Representation Through Party-Based District Allocation | Planned | Electoral Studies | Research |

**Track Head (E.0) Purpose**:
- Synthesis of lessons learned from 5 experimental alternative systems
- RQ1: Multi-member districts (proportionality), RQ2: County representation (fixed boundaries)
- RQ3: National redistricting (federalism cost), RQ4: Partisan similarity (safe seats)
- RQ5: Party-based allocation (overlapping districts)
- Key insight: **"No free lunch"** — all systems have trade-offs
- 6,000-8,000 words, 5 figures

**Experimental Papers**:
- All explore alternatives to single-member districts
- Counterfactual analysis reveals trade-offs in representation design
- Ready to implement at any time (no deferral restrictions)

---

## Narrative Flow — The Five-Paper Story

**Can someone read only A.0, B.0, C.0, D.0, E.0 and understand the full research program?**

### A.0 (Synthesis)
"We built an algorithmic system for congressional redistricting that is objective, transparent, and reproducible. It creates +69 more MM districts than enacted plans, identifies a 42% demographic threshold, and achieves 80% boundary stability across decades."

### B.0 (Algorithm Design)
"Here's how we designed it: recursive bisection (hierarchical splitting), edge-weighting (compactness optimization), METIS (multilevel coarsening), census tracts (resolution choice), architectural trade-offs (n-way vs recursive), parameter robustness."

### C.0 (Validation)
"Here's how we validated it works: spatial resolution testing (MAUP), cross-census methodology, boundary stability (80% retention), longitudinal trends (20 years), partisan fairness (62% bias reduction)."

### D.0 (VRA Compliance)
"Here's how it handles minority representation: +69 MM district surplus without explicit racial targeting, 42% demographic threshold for feasibility, method robustness, VRA-compactness trade-off analysis."

### E.0 (Experimental Systems)
"Here's what we learned from exploring alternatives: multi-member districts (proportional representation), county representation (existing boundaries), national redistricting (federalism cost), partisan similarity (safe seats), party-based allocation (overlapping districts)."

**Flow**: A.0 (what) → B.0 (how designed) → C.0 (how validated) → D.0 (special case: VRA) → E.0 (alternatives explored)

✅ **This narrative works perfectly!**

---

## Track Dependency Graph

```
A.0 (synthesis) ← depends on all tracks B, C, D, E

Track B (Algorithm Design):
  B.0 (design overview)
    ↓
  ┌─┴─┬───┬───┬───┐
  ↓   ↓   ↓   ↓   ↓
 B.1 B.2 B.3 B.4 B.5
 (recursive, edge, multi-vs-edge, adaptive, nway-general)

Track C (Validation):
  C.0 (validation overview)
    ↓
  ┌─┴─┬───┬───┬───┐
  ↓   ↓   ↓   ↓   ↓
 C.1 C.2 C.3 C.4 C.5
 (maup, cross-census, temporal, longitudinal, efficiency-gap)

Track D (VRA):
  D.0 (vra-compliance) ← strong standalone
    ↓
  ┌─┴─┬───┬───┐
  ↓   ↓   ↓
 D.1 D.2 D.3
 (threshold, nway-vra, compactness-tradeoff)

Track E (Experimental):
  E.0 (experimental overview)
    ↓
  ┌─┴─┬───┬───┬───┐
  ↓   ↓   ↓   ↓   ↓
 E.1 E.2 E.3 E.4 E.5
 (multi-member, county, national, partisan, party-based)
```

**Cross-Track Dependencies**:
- **B.1 (recursive-bisection)**: Foundation for all tracks
- **B.2 (edge-weighted-bisection)**: Used in C (validation), D (VRA), E (experimental)
- **C.3 (temporal-stability)**: Shows recursive advantage (references B.5 nway comparison)
- **D.0 (vra-compliance)**: Uses edge-weighting methodology (references B.2)

---

## Directory Structure

### Filesystem Organization (Sorted)

```
research/
├── A.0+synthesis-metapaper/
├── A.1+portfolio-guide/
├── A.2+portfolio-summary/
├── A.3+portfolio-visualization/         [planned]
├── A.4+replication-materials/           [planned]
├── B.0+algorithm-design-overview/
├── B.1+recursive-bisection/
├── B.2+edge-weighted-bisection/
├── B.3+multi-vs-edge/
├── B.4+adaptive-bisection/
├── B.5+nway-vs-recursive-general/
├── C.0+validation-overview/
├── C.1+maup-sensitivity/
├── C.2+cross-census-validation/
├── C.3+temporal-stability/
├── C.4+longitudinal-analysis/
├── C.5+efficiency-gap-analysis/
├── D.0+vra-compliance/
├── D.1+threshold-analysis/
├── D.2+nway-vs-recursive-vra/
├── D.3+compactness-tradeoff/
├── E.0+experimental-overview/
├── E.1+multi-member-districts/
├── E.2+county-representation/
├── E.3+national-redistricting/
├── E.4+partisan-similarity-districts/
└── E.5+party-based-allocation/
```

**Naming Convention**: `TRACK.INDEX+slug`
- Track heads: `.0` (e.g., `A.0+`, `B.0+`)
- Sub-papers: `.1`, `.2`, `.3`, etc. (e.g., `B.1+`, `B.2+`)
- Plus sign separator (`+`) between ID and slug

**Sorting**: Filesystem naturally sorts track heads first (A.0 before A.1), then sub-papers in order.

---

## Migration Mapping (Old → New)

### Papers That Changed Track IDs

| Old Directory | New Directory | Track | Notes |
|---------------|---------------|-------|-------|
| `00+synthesis-metapaper` | `A.0+synthesis-metapaper` | A | Track head |
| (gerry-portfolio-guide/guide.tex) | `A.1+portfolio-guide` | A | Extracted |
| (gerry-portfolio-guide/summary.tex) | `A.2+portfolio-summary` | A | Extracted |
| (none) | `A.3+portfolio-visualization` | A | NEW |
| (none) | `A.4+replication-materials` | A | NEW |
| (none) | `B.0+algorithm-design-overview` | B | NEW (track head) |
| `01+recursive-bisection` | `B.1+recursive-bisection` | B | Moved from #01 |
| `02+edge-weighted-bisection` | `B.2+edge-weighted-bisection` | B | Moved from #02 |
| `08+multi-vs-edge` | `B.3+multi-vs-edge` | B | Moved from #08 |
| `09+adaptive-bisection` | `B.4+adaptive-bisection` | B | Moved from #09 |
| `18+nway-vs-recursive-general` | `B.5+nway-vs-recursive-general` | B | Moved from #18 |
| (none) | `C.0+validation-overview` | C | NEW (track head) |
| `11+maup-sensitivity` | `C.1+maup-sensitivity` | C | Moved from #11 |
| `05+cross-census-validation` | `C.2+cross-census-validation` | C | Moved from #05 |
| `07+temporal-stability` | `C.3+temporal-stability` | C | Moved from #07 |
| `12+longitudinal-analysis` | `C.4+longitudinal-analysis` | C | Moved from #12 |
| `11+efficiency-gap-analysis` | `C.5+efficiency-gap-analysis` | C | Moved from #11 (duplicate resolved) |
| `03+vra-compliance` | `D.0+vra-compliance` | D | Track head (was #03) |
| `04+threshold-analysis` | `D.1+threshold-analysis` | D | Moved from #04 |
| `10+nway-vs-recursive` | `D.2+nway-vs-recursive-vra` | D | Moved from #10, renamed |
| `06+compactness-tradeoff` | `D.3+compactness-tradeoff` | D | Moved from #06 |
| (none) | `E.0+experimental-overview` | E | NEW (track head) |
| `16+multi-member-districts` | `E.1+multi-member-districts` | E | Moved from #16 |
| `14+county-representation` | `E.2+county-representation` | E | Moved from #14 |
| `13+national-redistricting` | `E.3+national-redistricting` | E | Moved from #13 |
| `17+partisan-similarity-districts` | `E.4+partisan-similarity-districts` | E | Moved from #17 |
| `15+party-based-allocation` | `E.5+party-based-allocation` | E | Moved from #15 |

**Key Changes**:
- Duplicate #11 issue resolved: maup-sensitivity = C.1, efficiency-gap-analysis = C.5
- Papers 08 and 09 moved from VRA track (D) to Algorithm Design track (B)
- Paper 11 (maup-sensitivity) moved from Algorithm Design to Validation track (C)
- All directories renamed to track-based IDs with .0 for track heads

---

## Paper Status Summary

| Status | Count | Papers |
|--------|-------|--------|
| **Ready** (panel-reviewed) | 10 | B.1, B.2, B.3, B.4, C.2, C.3, C.5, D.0, D.1, D.2, D.3 |
| **Planned** (plan.md exists) | 11 | A.0, B.0, B.5, C.0, C.1, C.4, E.0, E.1, E.2, E.3, E.4, E.5 |
| **Extracted** (from legacy) | 2 | A.1, A.2 |
| **To be created** | 2 | A.3, A.4 |
| **Total** | **25** | 28 directories (25 papers + 3 artifacts) |

**Readiness by Track**:
- Track A: 1 planned (A.0), 2 extracted (A.1, A.2), 2 to be created (A.3, A.4)
- Track B: 1 planned (B.0), 4 ready (B.1-B.4), 1 planned (B.5)
- Track C: 1 planned (C.0), 3 ready (C.2, C.3, C.5), 2 planned (C.1, C.4)
- Track D: 4 ready (D.0-D.3) — **fully ready track!**
- Track E: 6 planned (E.0-E.5) — experimental papers

**Track D is the only fully complete track** (all papers panel-reviewed and ready).

---

## Usage Notes

### For Claude Code

When referencing papers, use **Track IDs**:
- "Paper B.1 (Recursive Bisection)"
- "Paper C.5 (Efficiency Gap Analysis)"
- "Track head B.0 (Algorithm Design Overview)"

When writing cross-references in papers:
- "As shown in Paper B.1 (Recursive Bisection, Section 3)..."
- "See Track C (Validation) for comprehensive validation framework"

### For Panel Review

**Panel-reviewed papers** (10 total):
- Track B: B.1, B.2, B.3, B.4
- Track C: C.2, C.3, C.5
- Track D: D.0, D.1, D.2, D.3 (was #10)

**Rankings reflect panel scores** (from REVIEW_PANEL.md):
- A tier: B.1 (8.4), B.2 (8.2), D.0 (8.0)
- B+ tier: D.1 (7.9), C.2 (7.8), D.3 (7.7), C.3 (7.5)
- B tier: B.3 (7.3), B.4 (7.1)
- B- tier: D.2 (6.8)

### For New Papers

When adding papers:
1. Identify track (A, B, C, D, or E)
2. Assign next index within track (e.g., B.6 if Track B needs another paper)
3. Create directory: `TRACK.INDEX+slug/`
4. Add entry to this file under appropriate track
5. Update dependency graph if applicable

---

## Related Documents

- **REORGANIZATION-PLAN.md**: Detailed reorganization plan (Phases 1-8)
- **REVIEW_PANEL.md**: Panel review findings and scores for ready papers
- **PANEL-REVISION-PLAN.md**: Revision checklist (PP1/PP2/PP3 items)
- **Individual plan.md files**: Detailed plans for each paper (in each directory)

---

**Last Updated**: 2026-02-08
**Reorganization Complete**: Phase 3 (directory renaming) complete
**Next Phase**: Phase 4 (update cross-references) and Phase 5 (reference updates)
**Maintained By**: Research program lead
**Canonical Source**: This file is the single source of truth for track-based organization
