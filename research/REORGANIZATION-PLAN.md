# Portfolio Reorganization Plan — Track-Based Structure

**Created**: 2026-02-08
**Status**: Planning Phase
**Goal**: Reorganize 20 papers into 5 thematic tracks with narrative flow

---

## Overview

Reorganize portfolio from sequential numbering (01-18) to **track-based hierarchical numbering** (A, A.1, B, B.1, etc.) where **track head papers (A, B, C, D, E) tell the complete story** and sub-papers provide detailed evidence.

**Key Principle**: A reader could read **only the 5 track head papers** and understand the entire research program.

---

## Proposed Track Structure

### Track A — Overview & Synthesis
**Audience**: Interdisciplinary, general scientific community
**Narrative**: What we did and why it matters

| # | Slug | Title | Type | Readiness |
|---|------|-------|------|-----------|
| **A** | synthesis-metapaper | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | Synthesis (Science/Nature) | Planned ✓ |
| **A.1** | portfolio-guide | Portfolio Guide: Detailed Technical Documentation | Guide (300+ pages) | **NEW** |
| **A.2** | portfolio-summary | Portfolio Summary: Executive Brief | Summary (30 pages) | **NEW** |
| **A.3** | portfolio-visualization | Interactive Portfolio Visualization | Web app | **NEW** |
| **A.4** | replication-materials | Replication Materials and Data Archive | Archive | **NEW** |

**Track Head (A) Purpose**: Capstone synthesis for Science/Nature — tells the full story of algorithmic objectivity, draws from all tracks.

---

### Track B — Algorithm Design
**Audience**: Computer scientists, operations researchers, algorithm designers
**Narrative**: How we designed the redistricting algorithm

| # | Slug | Title | Type | Readiness |
|---|------|-------|------|-----------|
| **B** | algorithm-design-overview | Algorithmic Design for Congressional Redistricting: Method Selection and Architectural Decisions | Overview | **NEW** |
| **B.1** | recursive-bisection | Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design | Foundation | Ready ✓ |
| **B.2** | edge-weighted-bisection | Edge-Weighted Graph Partitioning for Compact Congressional Districts | Extension | Ready ✓ |
| **B.3** | nway-vs-recursive-general | N-Way vs Recursive Bisection: General Architectural Comparison | Comparison | Planned ✓ |
| **B.4** | maup-sensitivity | Spatial Resolution Sensitivity in Algorithmic Redistricting: Tracts, Block Groups, and Blocks | Robustness | Planned ✓ |

**Track Head (B) Purpose**:
- **NEW PAPER** explaining why we chose recursive bisection, edge-weighting, and METIS
- Covers: Graph partitioning paradigm, multilevel coarsening, architectural trade-offs (n-way vs recursive)
- Justifies: Census tracts as units, adjacency construction, population balancing
- Evidence: References B.1-B.4 for detailed proofs
- Target: ACM TSAS or SIAM Journal on Scientific Computing

---

### Track C — Validation & Analysis
**Audience**: Political scientists, election law scholars, statisticians
**Narrative**: How we validated that the algorithm works

| # | Slug | Title | Type | Readiness |
|---|------|-------|------|-----------|
| **C** | validation-overview | Validating Algorithmic Redistricting: A Multi-Faceted Approach | Overview | **NEW** |
| **C.1** | efficiency-gap-analysis | Efficiency Gap and Partisan Bias in Algorithmic Redistricting | Partisan fairness | Planned ✓ |
| **C.2** | cross-census-validation | Validating Algorithmic Redistricting Across Census Decades: A Slice-Based Methodology | Cross-census | Ready ✓ |
| **C.3** | temporal-stability | Temporal Stability in Algorithmic Redistricting: Hierarchical Advantage Across Census Cycles | Boundary stability | Ready ✓ |
| **C.4** | longitudinal-analysis | Twenty Years of Congressional Redistricting: Temporal Trends 2000-2020 | Longitudinal | Planned ✓ |

**Track Head (C) Purpose**:
- **NEW PAPER** explaining our comprehensive validation methodology
- Covers: Partisan fairness (efficiency gap), temporal validation (cross-census, stability), longitudinal trends
- Justifies: Why multiple validation dimensions are necessary
- Shows: Algorithm produces fair, stable, reproducible results
- Evidence: References C.1-C.4 for detailed statistical analyses
- Target: Political Analysis or APSR

---

### Track D — Voting Rights Act Compliance
**Audience**: Constitutional law scholars, civil rights advocates, election officials
**Narrative**: How algorithmic redistricting handles minority representation

| # | Slug | Title | Type | Readiness |
|---|------|-------|------|-----------|
| **D** | vra-compliance | Algorithmic Redistricting Exceeds Voting Rights Act Requirements Without Explicit Racial Targeting | Foundation | Ready ✓ |
| **D.1** | threshold-analysis | The 42% Threshold: State Minority Percentage and Majority-Minority District Feasibility | Feasibility | Ready ✓ |
| **D.2** | adaptive-bisection | Parameter Sensitivity in Recursive Bisection: Tree Structure Irrelevance with Edge-Weighting | Parameter tuning | Ready ✓ |
| **D.3** | nway-vs-recursive-vra | N-Way vs Recursive Bisection: Statistical Equivalence for VRA Compliance | VRA-specific comparison | Ready ✓ |
| **D.4** | multi-vs-edge | Why Single-Objective Edge-Weighting Outperforms Multi-Constraint Optimization | Constraint conflict | Ready ✓ |
| **D.5** | compactness-tradeoff | The VRA-Compactness Tradeoff: Pareto Frontiers in Algorithmic Redistricting | Pareto analysis | Ready ✓ |

**Track Head (D) Purpose**:
- **EXISTING PAPER** — already works as standalone
- Shows: Purely algorithmic approach (no racial data) produces VRA-compliant districts
- Addresses: Constitutional concerns, legal framework, minority representation
- Evidence: References D.1-D.5 for detailed analyses (feasibility limits, parameter sensitivity, trade-offs)
- Target: APSR, Journal of Politics, or Law Review

---

### Track E — Experimental Systems
**Audience**: Political theorists, electoral reform advocates, comparative politics scholars
**Narrative**: What if we redesigned representation from scratch?

| # | Slug | Title | Type | Readiness |
|---|------|-------|------|-----------|
| **E** | experimental-overview | Alternative Representation Systems: Algorithmic Explorations Beyond Single-Member Districts | Overview | **NEW** |
| **E.1** | multi-member-districts | Multi-Member Districts and Proportional Representation | Proportional | Planned ✓ |
| **E.2** | county-representation | Direct County Representation: Using Existing Political Boundaries | Boundary-based | Planned ✓ |
| **E.3** | national-redistricting | National Redistricting Without State Boundaries: A Geometric Baseline | Geographic | Planned ✓ |
| **E.4** | partisan-similarity-districts | Partisan Similarity Districts: Algorithmic Safe Seats | Partisan | Planned ✓ |
| **E.5** | party-based-allocation | Proportional Representation Through Party-Based District Allocation | Party-based | Planned ✓ |

**Track Head (E) Purpose**:
- **NEW PAPER** explaining what we learned from experimenting with alternative systems
- Covers: Multi-member, county-based, national, partisan similarity, party-based approaches
- Justifies: Why experimentation matters for understanding current system's constraints
- Shows: Trade-offs between different representation paradigms
- Evidence: References E.1-E.5 for detailed implementations
- Target: Electoral Studies or Comparative Political Studies

---

## New Papers Required

### 1. B — Algorithm Design Overview
**Estimated effort**: 2-3 weeks
**Content**:
- Section 1: Why graph partitioning? (explains recursive bisection paradigm)
- Section 2: Architectural decisions (METIS, census tracts, adjacency construction)
- Section 3: N-way vs recursive comparison (references B.3)
- Section 4: Edge-weighting for compactness (references B.2)
- Section 5: Robustness to resolution (references B.4)
- Section 6: Implementation details (references B.1)

**Target**: ACM TSAS, SIAM Journal on Scientific Computing
**Length**: 8,000-10,000 words

### 2. C — Validation Overview
**Estimated effort**: 2-3 weeks
**Content**:
- Section 1: Multi-faceted validation framework
- Section 2: Partisan fairness (efficiency gap, references C.1)
- Section 3: Temporal validation (cross-census, references C.2; stability, references C.3)
- Section 4: Longitudinal trends (20-year analysis, references C.4)
- Section 5: Synthesis (algorithm produces fair, stable, reproducible results)

**Target**: Political Analysis, APSR
**Length**: 8,000-10,000 words

### 3. E — Experimental Overview
**Estimated effort**: 1-2 weeks
**Content**:
- Section 1: Why experiment with alternatives?
- Section 2: Multi-member districts (references E.1)
- Section 3: County representation (references E.2)
- Section 4: National redistricting (references E.3)
- Section 5: Partisan similarity (references E.4)
- Section 6: Party-based allocation (references E.5)
- Section 7: Lessons learned (trade-offs, constraints)

**Target**: Electoral Studies, Comparative Political Studies
**Length**: 6,000-8,000 words

### 4. A.1 — Portfolio Guide (Detailed)
**Estimated effort**: 1 week
**Content**:
- Comprehensive technical documentation (300+ pages)
- How to use the codebase, replicate results, extend the work
- API documentation, data formats, pipeline details

**Target**: GitHub repository, Zenodo archive
**Format**: PDF + online documentation

### 5. A.2 — Portfolio Summary (Executive Brief)
**Estimated effort**: 3-4 days
**Content**:
- Executive summary of entire portfolio (30 pages)
- One-page summaries of each track head paper (A, B, C, D, E)
- Key findings, impact, future work

**Target**: Stakeholders, policymakers, journalists
**Format**: PDF

### 6. A.3 — Portfolio Visualization
**Estimated effort**: 1-2 weeks
**Content**:
- Interactive web app showing paper dependencies, findings, data
- Network graph of paper relationships
- Timeline of research program

**Target**: Web-hosted (GitHub Pages or similar)
**Format**: Vue.js web app

### 7. A.4 — Replication Materials
**Estimated effort**: 1 week
**Content**:
- Complete data archive (census data, outputs, intermediate results)
- Replication instructions
- Software environment specification

**Target**: Zenodo, Harvard Dataverse
**Format**: Compressed archive + README

---

## Directory Naming Scheme

### Simplified Approach: Sequential Directories, Track IDs in Documentation

**Filesystem**: Sequential numbering `01+` through `30+` (simple, clean sorting)
**Documentation**: Track-based IDs (A, A.1, B, B.1, etc.) in PAPERS.md, visualizations, cross-references

**Rationale**:
- Filesystem keeps simple sequential ordering
- Track structure maintained in PAPERS.md and visualizations
- Papers cite each other by track ID ("see Paper B.1")
- No filesystem sorting issues (A.10 vs A.2)
- Easier git renames (just renumber sequentially)

**Directory Structure** (filesystem order):
```
01+synthesis-metapaper/              → Track A
02+portfolio-guide/                  → Track A.1
03+portfolio-summary/                → Track A.2
04+portfolio-visualization/          → Track A.3
05+replication-materials/            → Track A.4
06+algorithm-design-overview/        → Track B
07+recursive-bisection/              → Track B.1
08+edge-weighted-bisection/          → Track B.2
09+nway-vs-recursive-general/        → Track B.3
10+maup-sensitivity/                 → Track B.4
11+validation-overview/              → Track C
12+efficiency-gap-analysis/          → Track C.1
13+cross-census-validation/          → Track C.2
14+temporal-stability/               → Track C.3
15+longitudinal-analysis/            → Track C.4
16+vra-compliance/                   → Track D
17+threshold-analysis/               → Track D.1
18+adaptive-bisection/               → Track D.2
19+nway-vs-recursive-vra/            → Track D.3
20+multi-vs-edge/                    → Track D.4
21+compactness-tradeoff/             → Track D.5
22+experimental-overview/            → Track E
23+multi-member-districts/           → Track E.1
24+county-representation/            → Track E.2
25+national-redistricting/           → Track E.3
26+partisan-similarity-districts/    → Track E.4
27+party-based-allocation/           → Track E.5
```

**Total**: 30 papers (directory #01-#30, track IDs A through E.5)

**Cross-Reference Style**:
- In papers: "As shown in Paper B.1 (Recursive Bisection)..."
- In PAPERS.md: Shows both directory # and track ID
- In code/scripts: Use directory number (`07+recursive-bisection`)
- In visualizations: Use track IDs (A, B, C, D, E with hierarchy)

---

## Migration Mapping

### From Current → To New

| Current | New | Track | Type |
|---------|-----|-------|------|
| `00+synthesis-metapaper` | `A+synthesis-metapaper` | A | Track head |
| (none) | `A.1+portfolio-guide` | A | NEW |
| (none) | `A.2+portfolio-summary` | A | NEW |
| (none) | `A.3+portfolio-visualization` | A | NEW |
| (none) | `A.4+replication-materials` | A | NEW |
| (none) | `B+algorithm-design-overview` | B | NEW (track head) |
| `01+recursive-bisection` | `B.1+recursive-bisection` | B | Move |
| `02+edge-weighted-bisection` | `B.2+edge-weighted-bisection` | B | Move |
| `18+nway-vs-recursive-general` | `B.3+nway-vs-recursive-general` | B | Move |
| `11+maup-sensitivity` | `B.4+maup-sensitivity` | B | Move |
| (none) | `C+validation-overview` | C | NEW (track head) |
| `11+efficiency-gap-analysis` | `C.1+efficiency-gap-analysis` | C | Move |
| `05+cross-census-validation` | `C.2+cross-census-validation` | C | Move |
| `07+temporal-stability` | `C.3+temporal-stability` | C | Move |
| `12+longitudinal-analysis` | `C.4+longitudinal-analysis` | C | Move |
| `03+vra-compliance` | `D+vra-compliance` | D | Move (becomes track head) |
| `04+threshold-analysis` | `D.1+threshold-analysis` | D | Move |
| `09+adaptive-bisection` | `D.2+adaptive-bisection` | D | Move |
| `10+nway-vs-recursive` | `D.3+nway-vs-recursive-vra` | D | Move + rename |
| `08+multi-vs-edge` | `D.4+multi-vs-edge` | D | Move |
| `06+compactness-tradeoff` | `D.5+compactness-tradeoff` | D | Move |
| (none) | `E+experimental-overview` | E | NEW (track head) |
| `16+multi-member-districts` | `E.1+multi-member-districts` | E | Move |
| `14+county-representation` | `E.2+county-representation` | E | Move |
| `13+national-redistricting` | `E.3+national-redistricting` | E | Move |
| `17+partisan-similarity-districts` | `E.4+partisan-similarity-districts` | E | Move |
| `15+party-based-allocation` | `E.5+party-based-allocation` | E | Move |

**Old total**: 20 papers (including duplicate #11)
**New total**: 30 papers (5 track heads + 25 sub-papers)
**Net new**: 10 papers (7 new artifacts, 3 new track head papers)

---

## Systematic Changes Required

### Phase 1: Create New Track Head Papers (1-2 weeks)
1. **B+algorithm-design-overview** (2-3 weeks)
2. **C+validation-overview** (2-3 weeks)
3. **E+experimental-overview** (1-2 weeks)

Run these **in parallel** (can be done by different authors or AI sessions).

### Phase 2: Create New Track A Artifacts (1-2 weeks, in parallel with Phase 1)
1. **A.1+portfolio-guide** (1 week)
2. **A.2+portfolio-summary** (3-4 days)
3. **A.3+portfolio-visualization** (1-2 weeks)
4. **A.4+replication-materials** (1 week)

### Phase 3: Directory Reorganization (1-2 days)
1. Rename all 20 existing directories (git mv operations)
2. Create 10 new directories for new papers
3. Update PAPERS.md with new structure
4. Verify all directories sort correctly

### Phase 4: Reference Updates (2-3 days)
1. Update all inter-paper references in plan.md files
2. Update panel review files (REVIEW_PANEL.md, individual reviews)
3. Update wave files that reference papers
4. Update CLAUDE.md and other documentation
5. Update scripts/notebooks that reference paper numbers

### Phase 5: Panel Re-Review (1 week, optional)
- Panel reviews new track head papers (B, C, E)
- Panel reviews new Track A artifacts (A.1-A.4)
- Adjusts track organization based on feedback

### Phase 6: Synthesis Paper (A) Update (1 week)
- Update synthesis metapaper to reference new track structure
- Ensure it tells the narrative using track heads

---

## Risks & Mitigation

### Risk 1: Breaking references
**Impact**: High (could break panel reviews, waves, documentation)
**Mitigation**:
- Comprehensive grep for all paper references before migration
- Create reference mapping file (old → new)
- Automated script to update all references

### Risk 2: Directory sorting issues
**Impact**: Medium (could confuse filesystem navigation)
**Mitigation**:
- Test sorting on both Windows and Unix filesystems
- Use clear separators (A+, A.1+, etc.)

### Risk 3: Scope creep (10 new papers)
**Impact**: High (could delay entire portfolio)
**Mitigation**:
- Prioritize track head papers (B, C, E) first
- Track A artifacts can be created incrementally
- Set clear deadlines for each phase

### Risk 4: Narrative inconsistency
**Impact**: Medium (track heads might not tell cohesive story)
**Mitigation**:
- Draft all 5 track head abstracts first
- Ensure they read coherently as a set (A → B → C → D → E)
- User review after Phase 1

### Risk 5: Panel score invalidation
**Impact**: Medium (track structure changes what panel reviewed)
**Mitigation**:
- Map existing panel scores to new track IDs
- Note in PAPERS.md which papers were panel-reviewed under old structure
- Optional: Re-review track heads after creation

---

## Success Criteria

- [ ] All 30 papers have directories in new naming scheme
- [ ] PAPERS.md reflects track-based organization
- [ ] All 3 new track head papers (B, C, E) have complete plan.md files
- [ ] All 7 new Track A artifacts have directories/plans
- [ ] No broken references in plan.md files, panel reviews, waves
- [ ] Track head papers (A, B, C, D, E) tell cohesive narrative
- [ ] Filesystem sorts correctly (A, A.1, A.2, ..., B, B.1, ...)
- [ ] Git history preserved (renames detected, not deletions)
- [ ] All changes committed with clear commit messages

---

## Timeline Estimate

| Phase | Duration | Parallel? | Blocker |
|-------|----------|-----------|---------|
| Phase 1: New track heads (B, C, E) | 2-3 weeks | Yes (3 papers in parallel) | - |
| Phase 2: Track A artifacts | 1-2 weeks | Yes (in parallel with Phase 1) | - |
| Phase 3: Directory rename | 1-2 days | No | Phase 1+2 complete |
| Phase 4: Reference updates | 2-3 days | No | Phase 3 complete |
| Phase 5: Panel re-review (optional) | 1 week | No | Phase 4 complete |
| Phase 6: Synthesis update | 1 week | No | Phase 5 complete |
| **Total (parallelized)** | **4-5 weeks** | - | - |
| **Total (sequential)** | **7-9 weeks** | - | - |

**Recommended**: Parallelize Phases 1 and 2 → **4-5 weeks total**

---

## Narrative Flow Check

**Question**: Can someone read only A, B, C, D, E and understand the full story?

**A (Synthesis)**: "We built an algorithmic system for congressional redistricting that is objective, transparent, and reproducible."

**B (Algorithm Design)**: "Here's how we designed the algorithm — recursive bisection, edge-weighting, architectural choices, robustness."

**C (Validation)**: "Here's how we validated it works — partisan fairness, temporal stability, cross-census reproducibility, longitudinal trends."

**D (VRA Compliance)**: "Here's how it handles minority representation — exceeds VRA requirements without explicit racial targeting, feasibility limits, trade-offs."

**E (Experimental Systems)**: "Here's what we learned from exploring alternatives — multi-member, county-based, national, partisan similarity, party-based approaches."

**Flow**: A (what) → B (how) → C (validation) → D (special case: VRA) → E (alternatives)

✅ **This narrative works!**

---

## Next Steps

### Immediate (before proceeding):
1. **User approval** of track structure and narrative flow
2. **Refinement** of track assignments (are papers in the right tracks?)
3. **Decision**: Create all 10 new papers, or prioritize track heads only?
4. **Decision**: Sequential or parallel approach?

### After approval:
1. Create detailed plans for 3 new track head papers (B, C, E)
2. Create directories for all 30 papers
3. Begin Phase 1 (track head papers) and Phase 2 (Track A artifacts) in parallel
4. Systematic migration and reference updates

---

**Created**: 2026-02-08
**Author**: Research program lead + Claude Sonnet 4.5
**Status**: **Awaiting user approval**
**Estimated Completion**: 4-5 weeks (with parallelization)
