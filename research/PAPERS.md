# Research Paper Portfolio — Canonical Index

**Created**: 2026-02-08
**Source of Truth**: This file defines the canonical numbering and ordering of all papers in the research program.

---

## Overview

This portfolio comprises **17 papers** organized into 5 categories:
- **Synthesis** (1 paper): Meta-analysis for Science/Nature
- **Core Portfolio** (10 papers): Foundation + extensions + investigations
- **Validation** (2 papers): Robustness testing
- **Experimental** (4 papers): Alternative representation systems

---

## Canonical Paper Ordering

### 00. Synthesis Meta-Paper

| # | Slug | Title | Status | Target Venue | Score |
|---|------|-------|--------|--------------|-------|
| **00** | synthesis-metapaper | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | Planned | Science / Nature | TBD |

**Role**: Capstone paper synthesizing findings from papers 01-17 for interdisciplinary audience. Written LAST but numbered 00 as conceptual overview.

---

### 01-10. Core Portfolio (Panel-Ranked)

Ordered by panel consensus ranking (contribution novelty, technical rigor, impact potential, readiness).

| # | Slug | Title | Panel Rank | Score | Tier | Venue | Status |
|---|------|-------|------------|-------|------|-------|--------|
| **01** | recursive-bisection | Recursive Bisection for Congressional Redistricting: Extending Huntington-Hill to Boundary Design | 1 | 8.4/10 | A | APSR / Science | Ready |
| **02** | edge-weighted-bisection | Edge-Weighted Graph Partitioning for Compact Congressional Districts | 2 | 8.2/10 | A- | KDD / SIGSPATIAL | Ready |
| **03** | vra-compliance | Algorithmic Redistricting Exceeds Voting Rights Act Requirements Without Explicit Racial Targeting | 3 | 8.0/10 | A- | APSR / JOP | Ready |
| **04** | threshold-analysis | The 42% Threshold: State Minority Percentage and Majority-Minority District Feasibility | 4 | 7.9/10 | B+ | APSR / Law Review | Ready |
| **05** | cross-census-validation | Validating Algorithmic Redistricting Across Census Decades: A Slice-Based Methodology | 5 | 7.8/10 | B+ | SIGSPATIAL | Ready |
| **06** | compactness-tradeoff | The VRA-Compactness Tradeoff: Pareto Frontiers in Algorithmic Redistricting | 6 | 7.7/10 | B+ | APSR | Ready |
| **07** | temporal-stability | Temporal Stability in Algorithmic Redistricting: Hierarchical Advantage Across Census Cycles | 7 | 7.5/10 | B+ | Political Analysis | Ready |
| **08** | multi-vs-edge | Why Single-Objective Edge-Weighting Outperforms Multi-Constraint Optimization | 8 | 7.3/10 | B | OR/MS journals | Ready |
| **09** | adaptive-bisection | Parameter Sensitivity in Recursive Bisection: Tree Structure Irrelevance with Edge-Weighting | 9 | 7.1/10 | B | Algorithmic venues | Ready |
| **10** | nway-vs-recursive | N-Way vs Recursive Bisection: Statistical Equivalence for VRA Compliance | 10 | 6.8/10 | B- | Comparative study | Ready |

**Status Key**:
- **Ready**: Paper complete, panel-reviewed, ready for submission after PP1/PP2 revisions
- **Planned**: Paper directory exists with plan.md, not yet written

---

### 11-12. Validation & Temporal Analysis

| # | Slug | Title | Status | Target Venue | Priority |
|---|------|-------|--------|--------------|----------|
| **11** | maup-sensitivity | Spatial Resolution Sensitivity in Algorithmic Redistricting: Tracts, Block Groups, and Blocks | Planned | IJGIS / Political Analysis | High |
| **12** | longitudinal-analysis | Twenty Years of Congressional Redistricting: Temporal Trends 2000-2020 | Planned | APSR / Political Analysis | High |

**Purpose**: Test robustness of core findings (MAUP) and analyze temporal evolution (longitudinal).

---

### 13-16. Experimental / Alternative Systems

| # | Slug | Title | Status | Target Venue | Priority |
|---|------|-------|--------|--------------|----------|
| **13** | national-redistricting | National Redistricting Without State Boundaries: A Geometric Baseline | Planned | Comparative Politics | Research |
| **14** | county-representation | Direct County Representation: Using Existing Political Boundaries | Planned | State Politics & Policy | Research |
| **15** | party-based-allocation | Proportional Representation Through Party-Based District Allocation | Planned | Electoral Studies | Research |
| **16** | multi-member-districts | Multi-Member Districts and Proportional Representation | Planned | Electoral Studies | Research |

**Status**: All experimental papers planned, ready to implement at any time.

**Purpose**: Explore alternative representation systems, generate novel research ideas for future conferences/publications.

---

## Paper Categories by Function

### Foundation (1 paper)
- **01**: recursive-bisection — Establishes core method and impossibility defense

### Primary Extensions (2 papers)
- **02**: edge-weighted-bisection — Adds compactness optimization
- **03**: vra-compliance — Adds demographic awareness

### Systematic Investigations (7 papers)
- **04**: threshold-analysis — VRA feasibility limits (42%)
- **05**: cross-census-validation — Tract correspondence methodology
- **06**: compactness-tradeoff — VRA-compactness Pareto frontiers
- **07**: temporal-stability — Hierarchical advantage (recursive > n-way)
- **08**: multi-vs-edge — Constraint conflict theory
- **09**: adaptive-bisection — Parameter sensitivity (tree structure)
- **10**: nway-vs-recursive — Architectural comparison (statistical equivalence)

### Validation (1 paper)
- **11**: maup-sensitivity — Tests generalizability across spatial resolutions

### Temporal Analysis (1 paper)
- **12**: longitudinal-analysis — 20-year trends (2000/2010/2020)

### Experimental (4 papers)
- **13-16**: Alternative representation systems (national, county, party-based, multi-member)

### Synthesis (1 paper)
- **00**: synthesis-metapaper — Capstone for Science/Nature

---

## Dependency Graph

```
00 (synthesis) ← depends on all papers 01-17

Foundation:
  01 (recursive-bisection)
      ↓
  ┌───┴───┐
  ↓       ↓
02 (edge)  03 (vra)
  ↓         ↓
  ├─→ 06 (compactness-tradeoff)
  ├─→ 08 (multi-vs-edge)
  ├─→ 09 (adaptive-bisection)
  └─→ 10 (nway-vs-recursive)
            ↓
          04 (threshold)
            ↓
          05 (cross-census)
            ↓
          07 (temporal-stability)

Validation:
  11 (maup) ← tests all 01-10

Temporal:
  12 (longitudinal) ← uses data from 01-10

Experimental:
  13-16 ← use infrastructure from 01-02
```

---

## Directory Naming Convention

All paper directories follow the format: `##+slug/`

Examples:
- `00+synthesis-metapaper/`
- `01+recursive-bisection/`
- `11+maup-sensitivity/`
- `13+national-redistricting/`

**Rationale**: Numbered prefixes ensure correct ordering in file listings and make dependencies clear. Plus sign separator matches wave naming convention.

---

## Usage Notes

### For Claude Code
When referencing papers, use either:
- **Full path**: `research/01-recursive-bisection/`
- **Number**: "Paper 01" or "Paper #01"
- **Slug**: "recursive-bisection paper"

### For Panel Review
Papers 01-10 have been panel-reviewed (see `REVIEW_PANEL.md`). Rankings in this file reflect consensus scores.

### For New Papers
When adding papers:
1. Assign next available number (18, 19, ...)
2. Create directory: `##-slug/`
3. Add entry to this file in appropriate section
4. Update dependency graph if applicable

---

## Paper Status Summary

| Status | Count | Papers |
|--------|-------|--------|
| Ready for submission | 10 | 01-10 (after PP1/PP2 revisions) |
| Planned | 7 | 00, 11-16 |
| **Total** | **17** | |

---

## Related Documents

- **REVIEW_PANEL.md**: Panel review findings and scores for papers 01-10
- **PANEL-REVISION-PLAN.md**: Revision checklist (PP1/PP2/PP3 items)
- **Individual plan.md files**: Detailed plans for papers 00, 11-17 (in each directory)

---

**Last Updated**: 2026-02-08
**Maintained By**: Research program lead
**Canonical Source**: This file is the single source of truth for paper numbering and ordering
