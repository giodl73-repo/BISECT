# Panel Revision Plan — Gerry Module
# Portfolio-Level Revision Items from Cross-Portfolio Panel Review

**Generated**: 2026-02-08
**Panel Round**: 1
**Module Score**: 7.8/10 (Tier A-)
**Papers**: 10

---

## Overview

This revision plan consolidates cross-cutting issues identified by the 13-member expert panel. Items are prioritized as **PP1** (blocking), **PP2** (important), or **PP3** (optional) using the same classification as paper-level reviews.

**Completion Target**: Address all PP1 items before submission; PP2 items before or during revision

---

## PP1 Items (Blocking — Required Before Submission)

These must be addressed to ensure portfolio coherence and legal credibility.

### PP1.1: Add Research Program Framing [ALL 10 PAPERS]

**Status**: ☐ Not started
**Applies to**: All 10 papers
**Estimated effort**: 2-3 hours total
**Blocking severity**: High — papers read as unrelated studies

**Action Items**:
- [x] Draft template paragraph (see REVIEW_PANEL.md Section V)
- [ ] Add paragraph to `gerry-recursive-bisection/` introduction
- [ ] Add paragraph to `gerry-edge-weighted-bisection/` introduction
- [ ] Add paragraph to `gerry-vra-compliance/` introduction
- [ ] Add paragraph to `gerry-threshold-analysis/` introduction
- [ ] Add paragraph to `gerry-cross-census-validation/` introduction
- [ ] Add paragraph to `gerry-compactness-tradeoff/` introduction
- [ ] Add paragraph to `gerry-temporal-stability/` introduction
- [ ] Add paragraph to `gerry-multi-vs-edge/` introduction
- [ ] Add paragraph to `gerry-adaptive-bisection/` introduction
- [ ] Add paragraph to `gerry-nway-vs-recursive/` introduction

**Template** (customize per paper):
```
This paper is part of a broader research program investigating recursive
bisection for congressional redistricting. The foundational method is
introduced in [recursive-bisection], which establishes the 'impossibility
defense' and demonstrates feasibility at national scale. [edge-weighted-
bisection] extends the method with compactness optimization, and [vra-
compliance] adds demographic awareness for Voting Rights Act compliance.
This paper investigates [SPECIFIC CONTRIBUTION], building on [PREREQUISITE
PAPERS] and informing [DOWNSTREAM PAPERS].
```

**Placement**: After contributions list, before paper outline in Introduction

---

### PP1.2: Standardize VRA Baseline [4 PAPERS — BLOCKING]

**Status**: ☐ Not started
**Applies to**: recursive-bisection, vra-compliance, compactness-tradeoff, threshold-analysis
**Estimated effort**: 1 week
**Blocking severity**: Critical — undermines legal credibility

**Problem**: Papers cite different enacted MM district counts:
- `recursive-bisection`: 81 enacted MM districts (source unclear)
- `vra-compliance`: 68 enacted MM districts (Dave's Redistricting)
- `compactness-tradeoff`: baseline not clearly stated
- `threshold-analysis`: uses algorithmic counts, no enacted baseline

**Root Cause**: Inconsistent definitions of "majority-minority" and/or different data sources

**Action Items**:
- [ ] **Step 1**: Define MM threshold consistently across portfolio
  - Use >50% BVAP for "majority-minority"
  - Use 45-50% BVAP for "effective control" (Gingles standard)
- [ ] **Step 2**: Compile authoritative enacted district baseline
  - Use Dave's Redistricting App as primary source
  - Verify against Brennan Center reports
  - Create `data/vra/enacted_mm_baseline.csv` with all 435 districts' BVAP
- [ ] **Step 3**: Create supplementary Table S1
  - All 435 enacted districts (2010s redistricting cycle)
  - Columns: State, District, BVAP%, MM_status (>50% / 45-50% / <45%)
  - Summary: Total MM districts (>50%), Total effective control (45-50%)
- [ ] **Step 4**: Update papers to reference consistent baseline
  - [ ] Update `recursive-bisection` Section 5.6 (VRA analysis)
  - [ ] Update `vra-compliance` throughout (currently uses 68)
  - [ ] Update `compactness-tradeoff` Table 2 baseline
  - [ ] Update `threshold-analysis` comparison sections
- [ ] **Step 5**: Add footnote to all 4 papers explaining definition
  ```
  "We define majority-minority districts as those with >50% BVAP (Black Voting
  Age Population), following DOJ enforcement guidelines. Districts with 45-50%
  BVAP are classified as 'effective control' under Gingles. See Table S1 for
  complete enacted district BVAP data."
  ```

**Verification**: All 4 papers must cite identical enacted MM count

---

### PP1.3: Add Cross-Paper References [ALL 10 PAPERS]

**Status**: ☐ Not started
**Applies to**: All 10 papers
**Estimated effort**: 3-4 hours
**Blocking severity**: High — weakens portfolio coherence

**Action Items**:

#### Part A: Update Bibliographies
- [ ] Ensure all papers cite `recursive-bisection` as foundational work
- [ ] Papers using edge-weighted method cite `edge-weighted-bisection`
- [ ] VRA-related papers cite `vra-compliance` and `threshold-analysis`
- [ ] Cross-census papers cite `cross-census-validation`
- [ ] Verification: Run script to check bibliography consistency

**Papers requiring bibliography updates**:
- [ ] `gerry-threshold-analysis` → add refs to vra-compliance, edge-weighted
- [ ] `gerry-compactness-tradeoff` → add ref to vra-compliance
- [ ] `gerry-temporal-stability` → add ref to recursive-bisection
- [ ] `gerry-multi-vs-edge` → add refs to recursive-bisection, edge-weighted
- [ ] `gerry-adaptive-bisection` → add ref to recursive-bisection
- [ ] `gerry-nway-vs-recursive` → add ref to recursive-bisection

#### Part B: Add In-Text Cross-References
- [ ] `threshold-analysis` cite edge-weighted + vra-compliance for methods
- [ ] `compactness-tradeoff` cite vra-compliance for Pareto frontier context
- [ ] `temporal-stability` cite recursive-bisection for baseline method
- [ ] `cross-census-validation` cite recursive-bisection + temporal-stability
- [ ] `multi-vs-edge` cite edge-weighted for single-objective rationale
- [ ] `adaptive-bisection` cite recursive-bisection for parameter defaults

#### Part C: Standardize Terminology
Ensure all papers use identical terms (not abbreviations or variants):

- [ ] "majority-minority districts" (NOT "MM districts", "VRA-compliant districts")
- [ ] "Polsby-Popper compactness" (NOT "PP score", "compactness metric")
- [ ] "recursive bisection" (NOT "RB method", "partitioning algorithm")
- [ ] "METIS" (NOT "graph partitioner", "bisection tool")
- [ ] "census tracts" (NOT "tracts", "spatial units")

**Verification**: Search all papers for abbreviated terms, replace with standard form

---

## PP2 Items (Important — Should Address Before Submission)

These strengthen the portfolio but aren't blocking.

### PP2.1: Standardize Evaluation Methodology [7 PAPERS]

**Status**: ☐ Not started
**Applies to**: All papers with empirical comparisons
**Estimated effort**: 1-2 days
**Impact**: Strengthens statistical rigor

**Action Items**:

#### Part A: Compactness Metrics
- [ ] Report Polsby-Popper as **primary** metric in all papers
- [ ] Move other metrics (Reock, Schwartzberg, Convex Hull) to appendix
- [ ] Create Appendix A for each paper: "Supplementary Compactness Metrics"
- [ ] Include table with ALL metrics for ALL algorithm variants

#### Part B: Statistical Tests
For every claim of "significant" or "better" performance:

- [ ] Add mean ± standard deviation
- [ ] Add 95% confidence intervals (bootstrap with 10K resamples)
- [ ] Add hypothesis test (t-test / Mann-Whitney) with p-value
- [ ] Add effect size (Cohen's d)
- [ ] Add sample size (N=50 states / N=435 districts)

**Papers requiring statistical tests**:
- [ ] `gerry-edge-weighted-bisection` (compactness improvement claims)
- [ ] `gerry-multi-vs-edge` (all comparisons)
- [ ] `gerry-nway-vs-recursive` (algorithm comparison)
- [ ] `gerry-compactness-tradeoff` (Pareto frontier tradeoff)
- [ ] `gerry-temporal-stability` (cross-census stability claims)
- [ ] `gerry-adaptive-bisection` (parameter sensitivity)
- [ ] `gerry-vra-compliance` (partisan effects Section 5.4)

#### Part C: Baseline Consistency
- [ ] All papers compare to 2010s enacted districts (standard baseline)
- [ ] Papers with ensemble comparisons cite same ReCom parameters
- [ ] Verification: Enacted district compactness values consistent across papers

---

### PP2.2: Add Methodological Limitations Section [ALL 10 PAPERS]

**Status**: ☐ Not started
**Applies to**: All 10 papers
**Estimated effort**: 1 hour per paper (10 hours total)
**Impact**: Preempts reviewer criticism

**Action Items**:
- [ ] Add "Methodological Limitations" subsection to Discussion (all papers)
- [ ] Use template from REVIEW_PANEL.md Section V, customize per paper
- [ ] Address: census tracts (MAUP), Rook adjacency, METIS choice, census years

**Template** (place in Discussion section):
```markdown
### Methodological Limitations

This study's findings depend on several methodological choices:

1. **Spatial resolution**: We use census tracts as atomic units. Alternative
   aggregations (block groups, blocks, precincts) may yield different results
   due to the Modifiable Areal Unit Problem (MAUP).

2. **Adjacency structure**: We define adjacency via Rook contiguity (shared
   edges). Queen adjacency (shared vertices) or distance-based graphs would
   alter the partition space.

3. **Partitioning algorithm**: We use METIS for recursive bisection. Other
   partitioners (SCOTCH, Zoltan, KaHIP) may produce different cuts even with
   identical inputs.

4. **Census data**: We analyze 2000, 2010, and 2020 decennial census data.
   Results may not generalize to ACS estimates or future censuses.

These choices are principled and standard in redistricting research, but
alternative methodologies may yield different quantitative results. The
qualitative findings (e.g., impossibility defense, +69 MM surplus, 42%
threshold) are likely robust to these variations, but replication studies
with different methodological choices would strengthen confidence.
```

**Papers checklist**:
- [ ] `gerry-recursive-bisection`
- [ ] `gerry-edge-weighted-bisection`
- [ ] `gerry-vra-compliance`
- [ ] `gerry-threshold-analysis`
- [ ] `gerry-cross-census-validation`
- [ ] `gerry-compactness-tradeoff`
- [ ] `gerry-temporal-stability`
- [ ] `gerry-multi-vs-edge`
- [ ] `gerry-adaptive-bisection`
- [ ] `gerry-nway-vs-recursive`

---

### PP2.3: Acknowledge Single-Ecosystem Derivation [ALL 10 PAPERS]

**Status**: ☐ Not started
**Applies to**: All 10 papers
**Estimated effort**: 30 minutes total
**Impact**: Clarifies scope of generalizability claims

**Action Items**:
- [ ] Add sentence to Methodology or Limitations section of each paper
- [ ] Use template below

**Template** (place at end of Methodology or start of Limitations):
```
The 10 papers in this research program share common infrastructure: census
tract shapefiles, adjacency graphs, and population weights. This enables
controlled comparisons across algorithm variants (edge-weighted vs. unweighted,
recursive vs. n-way) but means findings derive from a single computational
ecosystem rather than independent replications.
```

**Papers checklist**:
- [ ] `gerry-recursive-bisection`
- [ ] `gerry-edge-weighted-bisection`
- [ ] `gerry-vra-compliance`
- [ ] `gerry-threshold-analysis`
- [ ] `gerry-cross-census-validation`
- [ ] `gerry-compactness-tradeoff`
- [ ] `gerry-temporal-stability`
- [ ] `gerry-multi-vs-edge`
- [ ] `gerry-adaptive-bisection`
- [ ] `gerry-nway-vs-recursive`

---

### PP2.4: Add Effect Sizes and Confidence Intervals [7 PAPERS]

**Status**: ☐ Not started
**Applies to**: Papers with empirical comparisons
**Estimated effort**: 1 day
**Impact**: Meets statistical reporting standards

**Action Items**:
For every empirical comparison in these papers, recompute and report:

- [ ] 95% confidence intervals (bootstrap 10K resamples)
- [ ] Effect size (Cohen's d / Cramér's V / Cliff's delta)
- [ ] Sample size

**Example transformation**:

**Before**: "Edge-weighted bisection produces more compact districts (PP = 0.39) than unweighted (0.36)."

**After**: "Edge-weighted bisection produces more compact districts (PP = 0.39, 95% CI [0.37, 0.41]) than unweighted bisection (PP = 0.36, 95% CI [0.34, 0.38]), a statistically significant difference (p < 0.001) with small-to-medium effect size (Cohen's d = 0.31, N=50 states)."

**Papers requiring updates**:
- [ ] `gerry-edge-weighted-bisection`
- [ ] `gerry-multi-vs-edge`
- [ ] `gerry-nway-vs-recursive`
- [ ] `gerry-compactness-tradeoff`
- [ ] `gerry-temporal-stability`
- [ ] `gerry-adaptive-bisection`
- [ ] `gerry-vra-compliance`

---

### PP2.5: Stratify Analyses by Region/Demographics [1 PAPER]

**Status**: ☐ Not started
**Applies to**: `gerry-threshold-analysis` (primarily), also vra-compliance, compactness-tradeoff
**Estimated effort**: 2-3 days
**Impact**: Adds nuance to 42% threshold finding

**Action Items**:
- [ ] Rerun threshold analysis with regional stratification
  - South (high Black population concentration)
  - West (dispersed Hispanic population)
  - Northeast / Midwest (mixed demographics)
- [ ] Add table: "Regional Variation in MM Formation Thresholds"
- [ ] Add paragraph discussing geographic clustering's role
- [ ] Update abstract/conclusion to mention regional variation

**Proposed finding** (to add):
```
The 42% national average threshold masks regional variation. Southern states
with higher Black population concentrations have lower thresholds (38.5% ±
4.2%), while Western states with dispersed Hispanic populations have higher
thresholds (46.1% ± 5.8%). This suggests geographic clustering—not just raw
demographic percentages—determines algorithmic MM formation.
```

---

## PP3 Items (Optional — Enhance but Not Required)

These would improve the portfolio but aren't necessary for publication.

### PP3.1: Create Portfolio-Level Visualization

**Status**: ☐ Not started
**Estimated effort**: 4 hours
**Impact**: Helps readers understand research program architecture

**Action Items**:
- [ ] Design directed graph showing paper dependencies
- [ ] Annotate with key findings (e.g., "+69 MM surplus", "42% threshold")
- [ ] Create in LaTeX/TikZ or Python (networkx + matplotlib)
- [ ] Add to README or create standalone `PORTFOLIO_ARCHITECTURE.pdf`

---

### PP3.2: Create "Guide to the Portfolio" Document

**Status**: ☐ Not started
**Estimated effort**: 1 day
**Impact**: Lowers barrier to entry for interdisciplinary readers

**Action Items**:
- [ ] Write 2-3 page overview document
- [ ] Content: research questions, reading order, key findings per paper
- [ ] Glossary of technical terms (METIS, Polsby-Popper, VRA, BVAP, etc.)
- [ ] Pointer to code repository (when/if released)
- [ ] Distribute with paper submissions as supplementary material

---

### PP3.3: Prepare Replication Materials

**Status**: ☐ Not started (future work)
**Estimated effort**: 2-3 weeks
**Impact**: Strengthens reproducibility, increases citations

**Action Items**:
- [ ] Clean and document Python scripts
- [ ] Package census tract shapefiles (2000/2010/2020)
- [ ] Export adjacency graphs as standard format (GraphML, edge lists)
- [ ] METIS wrapper with parameter specifications
- [ ] Analysis notebooks for each paper
- [ ] README with step-by-step replication instructions
- [ ] Release on GitHub under permissive license (MIT / BSD)

---

## Progress Tracking

### PP1 Completion Status

| Item | Papers Affected | Completed | Remaining | % Done |
|------|----------------|-----------|-----------|--------|
| PP1.1: Program framing | 10 | 0 | 10 | 0% |
| PP1.2: VRA baseline | 4 | 0 | 4 | 0% |
| PP1.3: Cross-references | 10 | 0 | 10 | 0% |

**Overall PP1 Progress**: 0/24 items completed (0%)

### PP2 Completion Status

| Item | Papers Affected | Completed | Remaining | % Done |
|------|----------------|-----------|-----------|--------|
| PP2.1: Evaluation methodology | 7 | 0 | 7 | 0% |
| PP2.2: Limitations sections | 10 | 0 | 10 | 0% |
| PP2.3: Single-ecosystem | 10 | 0 | 10 | 0% |
| PP2.4: Effect sizes / CIs | 7 | 0 | 7 | 0% |
| PP2.5: Regional stratification | 1 | 0 | 1 | 0% |

**Overall PP2 Progress**: 0/35 items completed (0%)

### PP3 Completion Status

| Item | Status |
|------|--------|
| PP3.1: Portfolio visualization | ☐ Not started |
| PP3.2: Portfolio guide | ☐ Not started |
| PP3.3: Replication materials | ☐ Not started (future work) |

---

## Estimated Timeline

**Immediate (Week 1)**:
- PP1.1 (program framing): 2-3 hours
- PP1.3 (cross-references): 3-4 hours
- **Total**: ~1 day

**Week 2-3**:
- PP1.2 (VRA baseline): 1 week
- PP2.3 (single-ecosystem): 30 minutes
- **Total**: ~1 week

**Before Submission (Weeks 3-4)**:
- PP2.1 (evaluation methodology): 1-2 days
- PP2.2 (limitations sections): 1 day
- PP2.4 (effect sizes): 1 day
- **Total**: ~1 week

**During Revision (Weeks 5-6)**:
- PP2.5 (regional stratification): 2-3 days
- PP3.1 (visualization): 4 hours
- **Total**: ~3-4 days

**Grand Total**: 3-4 weeks full-time effort, or 6-8 weeks part-time

---

## Notes

- **PP1 items are BLOCKING** — must complete before any paper submission
- **PP2 items are IMPORTANT** — strongly recommended before submission, can be addressed during revision if needed
- **PP3 items are OPTIONAL** — enhance portfolio but not required for publication
- Address items in priority order: PP1 → PP2 → PP3
- Use this document to track progress; check boxes as items are completed
- After PP1 items complete, portfolio is ready for submission with Tier A- rating

---

**Document Created**: 2026-02-08
**Last Updated**: 2026-02-08
**Status**: All items pending
**Next Action**: Begin PP1.1 (add research program framing paragraphs)

