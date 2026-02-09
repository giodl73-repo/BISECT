# Portfolio Reorganization Plan — Track-Based Structure (REVISED)

**Created**: 2026-02-08
**Revised**: 2026-02-08
**Status**: Planning Phase - Approved Structure
**Goal**: Reorganize portfolio into 5 thematic tracks with narrative flow

---

## Overview

Reorganize portfolio into **track-based hierarchical structure** (A, A.1, B, B.1, etc.) where **track head papers (A, B, C, D, E) tell the complete story** and sub-papers provide detailed evidence.

**Key Principle**: A reader could read **only the 5 track head papers** and understand the entire research program.

**Directory Naming**: Use track letters directly (A+synthesis-metapaper, B.1+recursive-bisection, etc.)

---

## Final Track Structure

### Track A — Overview & Synthesis (5 papers)

**Audience**: Interdisciplinary, general scientific community, policymakers
**Narrative**: What we did, why it matters, how to use it

| Track ID | Slug | Title | Type | Status |
|----------|------|-------|------|--------|
| **A** | synthesis-metapaper | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | Science/Nature synthesis | Planned ✓ |
| **A.1** | portfolio-guide | Portfolio Guide: Reader's Introduction | 2-page guide | **EXISTS** — Extract from gerry-portfolio-guide/guide.tex |
| **A.2** | portfolio-summary | Portfolio Summary: Research Statement | 6-page statement | **EXISTS** — Extract from gerry-portfolio-guide/summary.tex |
| **A.3** | portfolio-visualization | Interactive Portfolio Visualization | Web app | NEW |
| **A.4** | replication-materials | Replication Materials and Data Archive | Archive | NEW |

**Track Head (A) Purpose**:
- Capstone synthesis for Science/Nature
- Tells full story: gerrymandering crisis → recursive bisection → impossibility defense → VRA surplus → 42% threshold
- Draws from all tracks (B, C, D, E)
- 2,500 words + 4 figures
- Target: Science (primary), PNAS (backup)

**Quick Win**: A.1 and A.2 already exist! Just extract and rename directories.

---

### Track B — Algorithm Design (5 papers)

**Audience**: Computer scientists, operations researchers, algorithm designers
**Narrative**: How we designed the redistricting algorithm and why these architectural choices

| Track ID | Slug | Title | Current # | Status |
|----------|------|-------|-----------|--------|
| **B** | algorithm-design-overview | Algorithmic Design for Congressional Redistricting: Method Selection and Architectural Decisions | NEW | NEW |
| **B.1** | recursive-bisection | Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design | 01 | Ready ✓ |
| **B.2** | edge-weighted-bisection | Edge-Weighted Graph Partitioning for Compact Congressional Districts | 02 | Ready ✓ |
| **B.3** | multi-vs-edge | Why Single-Objective Edge-Weighting Outperforms Multi-Constraint Optimization | 08 | Ready ✓ |
| **B.4** | adaptive-bisection | Parameter Sensitivity in Recursive Bisection: Tree Structure Irrelevance with Edge-Weighting | 09 | Ready ✓ |
| **B.5** | nway-vs-recursive-general | N-Way vs Recursive Bisection: General Architectural Comparison | 18 | Planned ✓ |

**Track Head (B) Purpose**:
- **NEW PAPER** explaining why we chose recursive bisection, edge-weighting, METIS
- Section 1: Why graph partitioning? (recursive bisection paradigm)
- Section 2: Why edge-weighting? (single-objective vs multi-constraint)
- Section 3: Why METIS? (multilevel coarsening, near-linear complexity)
- Section 4: Why census tracts? (resolution choice, MAUP awareness)
- Section 5: Architectural trade-offs (n-way vs recursive, Paper B.5)
- Section 6: Parameter robustness (Papers B.3, B.4)
- Evidence: References B.1-B.5 for detailed proofs
- Target: ACM TSAS or SIAM Journal on Scientific Computing
- Length: 8,000-10,000 words

**Why Papers B.3 and B.4 belong here**:
- **Paper 08 (multi-vs-edge)**: Explains core design decision (edge-weighting > multi-constraint)
- **Paper 09 (adaptive-bisection)**: Demonstrates parameter robustness (algorithm design concern)
- Both justify **how we designed the algorithm**, not how we validated it

---

### Track C — Validation & Analysis (5 papers)

**Audience**: Political scientists, statisticians, election law scholars
**Narrative**: How we validated that the algorithm works across multiple dimensions

| Track ID | Slug | Title | Current # | Status |
|----------|------|-------|-----------|--------|
| **C** | validation-overview | Validating Algorithmic Redistricting: A Multi-Faceted Approach | NEW | NEW |
| **C.1** | maup-sensitivity | Spatial Resolution Sensitivity in Algorithmic Redistricting: Tracts, Block Groups, and Blocks | 11b | Planned ✓ |
| **C.2** | cross-census-validation | Validating Algorithmic Redistricting Across Census Decades: A Slice-Based Methodology | 05 | Ready ✓ |
| **C.3** | temporal-stability | Temporal Stability in Algorithmic Redistricting: Hierarchical Advantage Across Census Cycles | 07 | Ready ✓ |
| **C.4** | longitudinal-analysis | Twenty Years of Congressional Redistricting: Temporal Trends 2000-2020 | 12 | Planned ✓ |
| **C.5** | efficiency-gap-analysis | Efficiency Gap and Partisan Bias in Algorithmic Redistricting | 11a | Ready ✓ |

**Track Head (C) Purpose**:
- **NEW PAPER** explaining comprehensive validation methodology
- Section 1: Why multiple validation dimensions?
- Section 2: Robustness testing (MAUP sensitivity, Paper C.1)
- Section 3: Temporal validation (cross-census methodology, Paper C.2; boundary stability, Paper C.3)
- Section 4: Longitudinal analysis (20-year trends, Paper C.4)
- Section 5: Partisan fairness (efficiency gap, Paper C.5)
- Section 6: Synthesis (algorithm produces fair, stable, reproducible results)
- Evidence: References C.1-C.5 for detailed statistical analyses
- Target: Political Analysis or APSR
- Length: 8,000-10,000 words

**Why this ordering**:
1. **C.1 (MAUP)**: Methodological foundation (does resolution matter?)
2. **C.2-C.4 (Temporal)**: Three temporal validation papers (cross-census, stability, longitudinal)
3. **C.5 (Efficiency gap)**: Partisan fairness (political validation)

**Note**: efficiency-gap-analysis (11a) is a COMPLETE paper (panel-reviewed, ready status). The duplicate #11 is with maup-sensitivity (11b).

---

### Track D — Voting Rights Act Compliance (3 papers)

**Audience**: Constitutional law scholars, civil rights advocates, election officials
**Narrative**: How algorithmic redistricting handles minority representation

| Track ID | Slug | Title | Current # | Status |
|----------|------|-------|-----------|--------|
| **D** | vra-compliance | Algorithmic Redistricting Exceeds Voting Rights Act Requirements Without Explicit Racial Targeting | 03 | Ready ✓ |
| **D.1** | threshold-analysis | The 42% Threshold: State Minority Percentage and Majority-Minority District Feasibility | 04 | Ready ✓ |
| **D.2** | nway-vs-recursive-vra | N-Way vs Recursive Bisection: Statistical Equivalence for VRA Compliance | 10 | Ready ✓ |
| **D.3** | compactness-tradeoff | The VRA-Compactness Tradeoff: Pareto Frontiers in Algorithmic Redistricting | 06 | Ready ✓ |

**Track Head (D) Purpose**:
- **EXISTING PAPER** — vra-compliance already works as standalone
- Shows: Purely algorithmic approach (no racial data) produces VRA-compliant districts
- Key findings: +69 MM district surplus, exceeds VRA requirements
- Addresses: Constitutional concerns, legal framework, minority representation
- Evidence: References D.1-D.3 for detailed analyses
- Target: APSR, Journal of Politics, or Law Review

**Why only 3 sub-papers**:
- vra-compliance is comprehensive and stands alone strongly
- Papers 08 (multi-vs-edge) and 09 (adaptive-bisection) moved to Track B (algorithm design)
- Remaining papers directly support VRA narrative:
  - D.1: Feasibility limits (42% threshold)
  - D.2: Method robustness for VRA (n-way vs recursive)
  - D.3: VRA-compactness trade-off analysis

---

### Track E — Experimental Systems (5 papers)

**Audience**: Political theorists, electoral reform advocates, comparative politics scholars
**Narrative**: What if we redesigned representation from scratch?

| Track ID | Slug | Title | Current # | Status |
|----------|------|-------|-----------|--------|
| **E** | experimental-overview | Alternative Representation Systems: Algorithmic Explorations Beyond Single-Member Districts | NEW | NEW |
| **E.1** | multi-member-districts | Multi-Member Districts and Proportional Representation | 16 | Planned ✓ |
| **E.2** | county-representation | Direct County Representation: Using Existing Political Boundaries | 14 | Planned ✓ |
| **E.3** | national-redistricting | National Redistricting Without State Boundaries: A Geometric Baseline | 13 | Planned ✓ |
| **E.4** | partisan-similarity-districts | Partisan Similarity Districts: Algorithmic Safe Seats | 17 | Planned ✓ |
| **E.5** | party-based-allocation | Proportional Representation Through Party-Based District Allocation | 15 | Planned ✓ |

**Track Head (E) Purpose**:
- **NEW PAPER** explaining what we learned from experimenting with alternative systems
- Section 1: Why experiment with alternatives? (understanding current system via counterfactuals)
- Section 2: Multi-member districts (proportional representation, Paper E.1)
- Section 3: County representation (existing boundaries, Paper E.2)
- Section 4: National redistricting (federalism cost, Paper E.3)
- Section 5: Partisan similarity (algorithmic safe seats, Paper E.4)
- Section 6: Party-based allocation (overlapping districts, Paper E.5)
- Section 7: Lessons learned (trade-offs, constraints revealed)
- Evidence: References E.1-E.5 for detailed implementations
- Target: Electoral Studies or Comparative Political Studies
- Length: 6,000-8,000 words

---

## Complete Directory Structure

### Filesystem Naming: Track Letters with Plus Sign Separator

```
A+synthesis-metapaper/              [00] → Track A
A.1+portfolio-guide/                [extract from gerry-portfolio-guide/guide.tex]
A.2+portfolio-summary/              [extract from gerry-portfolio-guide/summary.tex]
A.3+portfolio-visualization/        [NEW]
A.4+replication-materials/          [NEW]
B+algorithm-design-overview/        [NEW]
B.1+recursive-bisection/            [01]
B.2+edge-weighted-bisection/        [02]
B.3+multi-vs-edge/                  [08]
B.4+adaptive-bisection/             [09]
B.5+nway-vs-recursive-general/      [18]
C+validation-overview/              [NEW]
C.1+maup-sensitivity/               [11b]
C.2+cross-census-validation/        [05]
C.3+temporal-stability/             [07]
C.4+longitudinal-analysis/          [12]
C.5+efficiency-gap-analysis/        [11a]
D+vra-compliance/                   [03]
D.1+threshold-analysis/             [04]
D.2+nway-vs-recursive-vra/          [10]
D.3+compactness-tradeoff/           [06]
E+experimental-overview/            [NEW]
E.1+multi-member-districts/         [16]
E.2+county-representation/          [14]
E.3+national-redistricting/         [13]
E.4+partisan-similarity-districts/  [17]
E.5+party-based-allocation/         [15]
```

**Total**: 28 papers (5 track heads + 23 sub-papers)

**Breakdown**:
- Track A: 5 papers (1 head + 4 supporting artifacts)
- Track B: 6 papers (1 head + 5 algorithm papers)
- Track C: 6 papers (1 head + 5 validation papers)
- Track D: 4 papers (1 head + 3 VRA papers)
- Track E: 6 papers (1 head + 5 experimental papers)

---

## Migration Mapping (Old → New)

| Current Directory | New Directory | Track | Notes |
|-------------------|---------------|-------|-------|
| `00+synthesis-metapaper` | `A+synthesis-metapaper` | A | Rename |
| (gerry-portfolio-guide/guide.tex) | `A.1+portfolio-guide` | A | **Extract** |
| (gerry-portfolio-guide/summary.tex) | `A.2+portfolio-summary` | A | **Extract** |
| (none) | `A.3+portfolio-visualization` | A | NEW |
| (none) | `A.4+replication-materials` | A | NEW |
| (none) | `B+algorithm-design-overview` | B | NEW |
| `01+recursive-bisection` | `B.1+recursive-bisection` | B | Rename |
| `02+edge-weighted-bisection` | `B.2+edge-weighted-bisection` | B | Rename |
| `08+multi-vs-edge` | `B.3+multi-vs-edge` | B | Rename |
| `09+adaptive-bisection` | `B.4+adaptive-bisection` | B | Rename |
| `18+nway-vs-recursive-general` | `B.5+nway-vs-recursive-general` | B | Rename |
| (none) | `C+validation-overview` | C | NEW |
| `11+maup-sensitivity` | `C.1+maup-sensitivity` | C | Rename |
| `05+cross-census-validation` | `C.2+cross-census-validation` | C | Rename |
| `07+temporal-stability` | `C.3+temporal-stability` | C | Rename |
| `12+longitudinal-analysis` | `C.4+longitudinal-analysis` | C | Rename |
| `11+efficiency-gap-analysis` | `C.5+efficiency-gap-analysis` | C | Rename (resolve duplicate #11) |
| `03+vra-compliance` | `D+vra-compliance` | D | Rename |
| `04+threshold-analysis` | `D.1+threshold-analysis` | D | Rename |
| `10+nway-vs-recursive` | `D.2+nway-vs-recursive-vra` | D | Rename |
| `06+compactness-tradeoff` | `D.3+compactness-tradeoff` | D | Rename |
| (none) | `E+experimental-overview` | E | NEW |
| `16+multi-member-districts` | `E.1+multi-member-districts` | E | Rename |
| `14+county-representation` | `E.2+county-representation` | E | Rename |
| `13+national-redistricting` | `E.3+national-redistricting` | E | Rename |
| `17+partisan-similarity-districts` | `E.4+partisan-similarity-districts` | E | Rename |
| `15+party-based-allocation` | `E.5+party-based-allocation` | E | Rename |
| `gerry-portfolio-guide` | (extract into A.1, A.2) | A | Split |

**Old total**: 20 papers (including duplicate #11, plus gerry-portfolio-guide)
**New total**: 28 papers (5 tracks × 4-6 papers each)
**Net new**: 6 papers (3 track heads B/C/E, 2 artifacts A.3/A.4, plus extraction of A.1/A.2)

---

## New Papers Required (6 total)

### 1. B — Algorithm Design Overview
**Estimated effort**: 2-3 weeks
**Content**:
- Section 1: Why graph partitioning? (recursive bisection paradigm)
- Section 2: Why edge-weighting? (single-objective > multi-constraint, references B.3)
- Section 3: Why METIS? (multilevel coarsening, complexity analysis)
- Section 4: Why census tracts? (resolution choice, MAUP awareness)
- Section 5: Architectural comparison (n-way vs recursive, references B.5)
- Section 6: Parameter robustness (references B.4)
**Target**: ACM TSAS, SIAM Journal on Scientific Computing
**Length**: 8,000-10,000 words

### 2. C — Validation Overview
**Estimated effort**: 2-3 weeks
**Content**:
- Section 1: Multi-faceted validation framework
- Section 2: Robustness (MAUP sensitivity, references C.1)
- Section 3: Temporal validation (cross-census C.2, stability C.3, longitudinal C.4)
- Section 4: Partisan fairness (efficiency gap, references C.5)
- Section 5: Synthesis (fair, stable, reproducible results)
**Target**: Political Analysis, APSR
**Length**: 8,000-10,000 words

### 3. E — Experimental Overview
**Estimated effort**: 1-2 weeks
**Content**:
- Section 1: Why experiment with alternatives?
- Sections 2-6: Each experimental system (E.1-E.5)
- Section 7: Lessons learned (trade-offs, constraints)
**Target**: Electoral Studies, Comparative Political Studies
**Length**: 6,000-8,000 words

### 4. A.1 — Portfolio Guide (EXTRACT, not create!)
**Estimated effort**: 1 day
**Action**: Extract gerry-portfolio-guide/guide.tex → A.1+portfolio-guide/
**Status**: File already exists, just needs directory creation + extraction

### 5. A.2 — Portfolio Summary (EXTRACT, not create!)
**Estimated effort**: 1 day
**Action**: Extract gerry-portfolio-guide/summary.tex → A.2+portfolio-summary/
**Status**: File already exists, just needs directory creation + extraction

### 6. A.3 — Portfolio Visualization
**Estimated effort**: 1-2 weeks (check if gerry-portfolio-visualization/ already exists?)
**Content**: Interactive web app showing paper dependencies, findings, data
**Format**: Vue.js or D3.js visualization

### 7. A.4 — Replication Materials
**Estimated effort**: 1 week (check if gerry-replication-materials/ already exists?)
**Content**: Complete data archive, replication instructions, environment spec
**Format**: Zenodo/Harvard Dataverse archive + README

---

## Systematic Changes Required

### Phase 1: Extract Existing Artifacts (1 day)
1. Create `A.1+portfolio-guide/` directory
2. Copy `gerry-portfolio-guide/guide.tex` → `A.1+portfolio-guide/guide.tex`
3. Copy associated files (Makefile, guide.pdf, etc.)
4. Create `A.2+portfolio-summary/` directory
5. Copy `gerry-portfolio-guide/summary.tex` → `A.2+portfolio-summary/summary.tex`
6. Copy associated files
7. Archive `gerry-portfolio-guide/` (mark as legacy)

### Phase 2: Create New Track Head Papers (2-3 weeks, parallelizable)
1. **B+algorithm-design-overview** (2-3 weeks)
2. **C+validation-overview** (2-3 weeks)
3. **E+experimental-overview** (1-2 weeks)

These can be written **in parallel** by different authors or AI sessions.

### Phase 3: Directory Reorganization (1 day)
1. Rename all 20 existing paper directories (git mv operations)
   - Use track-based naming (A+, B.1+, C.2+, etc.)
2. Create directories for new papers (B+, C+, E+, A.3+, A.4+)
3. Verify filesystem sorting is correct

### Phase 4: Update PAPERS.md (1 day)
1. Complete rewrite with track-based organization
2. Add track head paper descriptions
3. Update all cross-references (old #01 → new B.1)
4. Update dependency graph with track structure

### Phase 5: Reference Updates (2-3 days)
1. Update all inter-paper references in plan.md files
   - "Paper 01" → "Paper B.1 (Recursive Bisection)"
   - "Paper 08" → "Paper B.3 (Multi vs Edge)"
2. Update panel review files (REVIEW_PANEL.md, individual reviews)
3. Update wave files that reference papers
4. Update CLAUDE.md and other documentation
5. Update scripts/notebooks that reference paper numbers

### Phase 6: Create Supporting Artifacts (1-2 weeks, parallelizable)
1. **A.3+portfolio-visualization** (1-2 weeks) — Check if gerry-portfolio-visualization/ exists first
2. **A.4+replication-materials** (1 week) — Check if gerry-replication-materials/ exists first

### Phase 7: Panel Re-Review (1 week, optional)
- Panel reviews new track head papers (B, C, E)
- Panel reviews new Track A artifacts (A.3, A.4)
- Adjusts track organization based on feedback

### Phase 8: Synthesis Paper (A) Update (1 week)
- Update synthesis metapaper to reference new track structure
- Ensure it tells narrative using track heads (A, B, C, D, E)

---

## Timeline Estimate

| Phase | Duration | Parallel? | Dependencies |
|-------|----------|-----------|--------------|
| Phase 1: Extract artifacts (A.1, A.2) | 1 day | No | - |
| Phase 2: New track heads (B, C, E) | 2-3 weeks | **Yes** (3 papers in parallel) | Phase 1 complete |
| Phase 3: Directory rename | 1 day | No | Phase 1 complete |
| Phase 4: PAPERS.md update | 1 day | No | Phase 3 complete |
| Phase 5: Reference updates | 2-3 days | No | Phase 4 complete |
| Phase 6: Artifacts (A.3, A.4) | 1-2 weeks | **Yes** (in parallel with Phase 2) | Phase 1 complete |
| Phase 7: Panel re-review (optional) | 1 week | No | Phase 2, 6 complete |
| Phase 8: Synthesis update | 1 week | No | Phase 7 complete |
| **Total (parallelized)** | **3-4 weeks** | - | - |
| **Total (sequential)** | **6-8 weeks** | - | - |

**Recommended**: Parallelize Phases 2 and 6 → **3-4 weeks total**

---

## Narrative Flow Check

**Can someone read only A, B, C, D, E and understand the full story?**

**A (Synthesis)**:
"We built an algorithmic system for congressional redistricting that is objective, transparent, and reproducible. It creates +69 more MM districts than enacted plans, identifies a 42% demographic threshold, and achieves 80% boundary stability across decades."

**B (Algorithm Design)**:
"Here's how we designed it: recursive bisection (hierarchical splitting), edge-weighting (compactness optimization), METIS (multilevel coarsening), census tracts (resolution choice), architectural trade-offs (n-way vs recursive), parameter robustness."

**C (Validation)**:
"Here's how we validated it works: spatial resolution testing (MAUP), cross-census methodology, boundary stability (80% retention), longitudinal trends (20 years), partisan fairness (62% bias reduction)."

**D (VRA Compliance)**:
"Here's how it handles minority representation: +69 MM district surplus without explicit racial targeting, 42% demographic threshold for feasibility, method robustness, VRA-compactness trade-off analysis."

**E (Experimental Systems)**:
"Here's what we learned from exploring alternatives: multi-member districts (proportional representation), county representation (existing boundaries), national redistricting (federalism cost), partisan similarity (safe seats), party-based allocation (overlapping districts)."

**Flow**: A (what) → B (how designed) → C (how validated) → D (special case: VRA) → E (alternatives explored)

✅ **This narrative works perfectly!**

---

## Success Criteria

- [ ] All 28 papers have directories in track-based naming (A+, A.1+, B+, B.1+, etc.)
- [ ] PAPERS.md reflects track-based organization
- [ ] All 3 new track head papers (B, C, E) have complete plan.md files
- [ ] A.1 and A.2 extracted from gerry-portfolio-guide (quick win!)
- [ ] A.3 and A.4 directories created with plans
- [ ] No broken references in plan.md files, panel reviews, waves
- [ ] Track head papers (A, B, C, D, E) tell cohesive narrative
- [ ] Filesystem sorts correctly (A, A.1, A.2, ..., B, B.1, ...)
- [ ] Git history preserved (renames detected, not deletions)
- [ ] All changes committed with clear commit messages
- [ ] Duplicate #11 issue resolved (efficiency-gap = C.5, maup = C.1)

---

## Risk Mitigation

### Risk 1: Breaking references
**Mitigation**:
- Create reference mapping file (old → new) before migration
- Automated script to update all references
- Manual verification of panel reviews

### Risk 2: Losing track of existing papers
**Mitigation**:
- Migration mapping table clearly shows old # → new Track ID
- Keep PAPERS.md updated throughout process

### Risk 3: Scope creep (6 new papers)
**Mitigation**:
- Prioritize track head papers (B, C, E) first
- A.1 and A.2 are quick wins (extraction only)
- A.3 and A.4 can be created incrementally

### Risk 4: Narrative inconsistency
**Mitigation**:
- Draft all 5 track head abstracts first
- Ensure they read coherently as a set (A → B → C → D → E)
- User review after Phase 2

---

## Next Steps

### Immediate:
1. ✅ **User approval** of refined track structure (APPROVED)
2. **Phase 1**: Extract A.1 and A.2 from gerry-portfolio-guide (1 day)
3. **Phase 2**: Create plan.md files for 3 new track heads (B, C, E)
4. **Phase 3**: Directory reorganization (rename all to track IDs)

### After Phase 3:
5. **Phase 4-5**: Update PAPERS.md and all references
6. **Phase 6**: Create A.3 and A.4 artifacts
7. **Phase 7-8**: Optional panel review and synthesis update

---

**Created**: 2026-02-08
**Revised**: 2026-02-08
**Author**: Research program lead + Claude Sonnet 4.5
**Status**: **Approved - Ready to Execute**
**Estimated Completion**: 3-4 weeks (with parallelization)
