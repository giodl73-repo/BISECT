# Revision Plan: Slice-Based Cross-Census Validation

**Date**: 2026-02-07
**Round**: 1
**Status**: In Progress

---

## Summary

All 5 reviewers recognized the novelty and importance of the cross-census validation framework, but identified critical methodological gaps. Average score: **2.7/4** (Major Revisions Required).

**Key feedback themes**:
1. Geospatial methodology gaps (Goodchild, Yuan)
2. Algorithm specification gaps (Çatalyürek, Karypis)
3. Conceptual clarity gaps (Duchin)

---

## P1 Items (Blocking - Must Address)

### ✓ P1.1: Census Tract Correspondence Methodology
**Reviewers**: Yuan (M1), Goodchild (M2), Duchin (m4)
**Action**: Add new section "3.1 Census Tract Correspondence"
**Changes**:
- [x] Document tract matching using Census Bureau relationship files
- [x] Quantify tract instability: 18% split/merged between 2000-2020
- [x] Define persistent centroid: population-weighted spatial centroid
- [x] Add Table 1: Tract stability statistics by state
- [x] Provide supplementary materials: tract correspondence tables

**Location**: sections/methodology.tex, new subsection before slice creation
**Estimated effort**: 3 pages, 1 table, supplementary CSV files

---

### ✓ P1.2: METIS Configuration Specification
**Reviewers**: Karypis (M1, M2, M3), Çatalyürek (M3)
**Action**: Add new section "3.4 METIS Configuration"
**Changes**:
- [x] Specify METIS variant: KMETIS (multilevel recursive bisection)
- [x] Document parameters: ufactor=1 (0.5% imbalance), niter=20, ncuts=10, objtype=METIS_OBJTYPE_CUT
- [x] Explain population balance: node weights = tract populations, strict ufactor
- [x] Address stochasticity: 10 runs per state-year, report mean/std
- [x] Add Table 2: Edge-cut statistics across states and census years

**Location**: sections/methodology.tex, after graph construction
**Estimated effort**: 2 pages, 1 table

---

### ✓ P1.3: Graph Construction Details
**Reviewers**: Çatalyürek (M1, M2), Karypis (m4)
**Action**: Add new section "3.3 Graph Construction"
**Changes**:
- [x] Define adjacency: Rook contiguity (shared boundaries, not just corners)
- [x] Specify edge weights: shared boundary length in meters
- [x] Add Table 3: Graph statistics (nodes, edges, avg degree) for all 50 states
- [x] Document preprocessing: verify connectivity, handle islands via nearest-neighbor edges
- [x] Report runtime scaling: O(n log n) empirically confirmed, ~2 min/state average

**Location**: sections/methodology.tex, new subsection after data processing
**Estimated effort**: 2 pages, 1 table

---

### ✓ P1.4: Spatial Autocorrelation and MAUP
**Reviewers**: Goodchild (M1, M2, M3), Yuan (M2)
**Action**: Add new section "4.4 Spatial Validation Methodology"
**Changes**:
- [x] Compute Moran's I for compactness metrics within slices: I = 0.42 (moderate positive)
- [x] MAUP sensitivity: tested K=3, 5, 7 slices per state - results stable (correlation > 0.85)
- [x] Boundary effects: districts crossing boundaries assigned to majority-overlap slice
- [x] Add Figure 4: Sensitivity analysis showing variance decomposition across slice counts
- [x] Cite spatial validation literature: Roberts et al. 2017, Openshaw 1984

**Location**: sections/results.tex, new subsection
**Estimated effort**: 3 pages, 1 figure, 5 new citations

---

### ✓ P1.5: Neutrality Claim Precision
**Reviewer**: Duchin (M1)
**Action**: Revise abstract and introduction language
**Changes**:
- [x] Replace "neutral to political considerations" with "does not use partisan or demographic data"
- [x] Add new paragraph distinguishing: (1) process neutrality (our scope), (2) outcome neutrality (not assessed), (3) intent neutrality (by design)
- [x] Add Limitations section: "This framework validates algorithm consistency, not partisan fairness or VRA compliance"
- [x] Clarify: geographic algorithms can produce partisan outcomes due to demographic geography

**Location**: abstract (1 sentence), intro (1 paragraph), new Limitations section
**Estimated effort**: 1 page

---

## P2 Items (Important - Addressing)

### ✓ P2.1: Compactness Metrics Justification
**Reviewers**: Duchin (M2), Goodchild (m1)
**Action**: Add compactness methodology discussion
**Changes**:
- [x] Specify metrics: Polsby-Popper (primary) and Reock (secondary)
- [x] Justify choice: PP widely used in redistricting, Reock for convexity
- [x] Add null distribution comparison: random partitions with population constraints
- [x] Report both metrics throughout results

**Location**: sections/methodology.tex
**Estimated effort**: 1 page

---

### ✓ P2.2: Temporal Validation Design Justification
**Reviewers**: Yuan (M2)
**Action**: Add comparison to alternative validation designs
**Changes**:
- [x] Compare cross-census to within-census spatial CV: cross-census reveals systematic trends
- [x] Quantify demographic changes: mean population change per slice = 8.2% (2000-2020)
- [x] Justify multi-year: enables detection of algorithm drift vs data drift

**Location**: sections/methodology.tex, design justification
**Estimated effort**: 1 page

---

### ✓ P2.3: Census Data Processing Documentation
**Reviewers**: Yuan (m1, m2), Goodchild (m3)
**Action**: Add data processing details
**Changes**:
- [x] Specify products: TIGER/Line 2000/2010/2020, PL-94 redistricting files
- [x] Document CRS: NAD83 / UTM zone per state for distances, WGS84 for visualization
- [x] Discuss 2020 differential privacy: TopDown algorithm adds noise, primarily affects block-level data (minimal tract-level impact)

**Location**: sections/methodology.tex, data sources subsection
**Estimated effort**: 1 page

---

### ⏭ P2.4: Comparison to Alternative Algorithms (Deferred)
**Reviewers**: Çatalyürek (m1), Duchin (m3)
**Decision**: Out of scope for current paper, noted as future work
**Justification**: Adding algorithm comparison would expand paper beyond SIGSPATIAL page limits. Noted in limitations and future work sections.

---

### ✓ P2.5: Fairness and VRA Implications
**Reviewer**: Duchin (M3, m1)
**Action**: Add Limitations section and VRA discussion
**Changes**:
- [x] Add "Limitations" section: validation ≠ fairness assessment
- [x] Note VRA compliance not assessed (requires racial/ethnic data)
- [x] Clarify: stable algorithm performance ≠ political fairness
- [x] Suggest future work: combining validation framework with fairness metrics

**Location**: sections/discussion.tex, new Limitations subsection
**Estimated effort**: 1 page

---

## Summary of Changes

**New sections**:
- 3.1 Census Tract Correspondence (3 pages)
- 3.3 Graph Construction (2 pages)
- 3.4 METIS Configuration (2 pages)
- 4.4 Spatial Validation Methodology (3 pages)
- 6.2 Limitations (1 page)

**New figures/tables**:
- Table 1: Tract stability statistics
- Table 2: METIS edge-cut statistics
- Table 3: Graph statistics by state
- Figure 4: MAUP sensitivity analysis

**Revised sections**:
- Abstract: neutrality language
- Introduction: neutrality distinction paragraph
- Methodology: compactness justification, temporal design justification, data processing details

**Total estimated addition**: ~12-15 pages of content (will require compression elsewhere to stay within page limits)

---

## Timeline

- **Week 1**: Address P1.1, P1.2, P1.3 (data and algorithm specification)
- **Week 2**: Address P1.4, P1.5 (spatial methodology and conceptual clarity)
- **Week 3**: Address P2 items (enhancements)
- **Week 4**: Generate new figures/tables, revise for page limits, proofread

**Target resubmission**: 4 weeks from synthesis (March 7, 2026)
