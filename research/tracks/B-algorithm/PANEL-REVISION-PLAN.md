# Track B — Algorithm: Panel Revision Plan

**Track**: B-algorithm
**Generated**: 2026-05-07
**Based on**: REVIEW_PANEL.md (2026-05-07), _panel.yaml files for 14 papers, paper abstracts
**Target module score**: 8.5 / 10 (current: 7.3 / 10)

---

## Overview

Track B's revision work falls into three distinct categories:

1. **Administrative fixes** (immediate, low effort): MODULE.md description error for B.6, B.22b naming/scope decision
2. **Missing panel reviews** (3–6 weeks each): B.5, B.6, B.16, B.17 have no panel review history; B.18, B.7 are also unreviewed
3. **Paper-level revisions** (varies): B.8 R1 blocking item, B.15 Proposition relabeling, B.12 positioning

The foundations and extensions sub-tracks are strong. The search sub-track (B.16, B.17) is the module's weakest link: both papers are foundational for downstream tracks (G.0, G.14, H.0 all depend on B.16's T=600 formula) but neither has been panel-reviewed.

---

## Immediate Actions (This Week)

### Action B-IMMED-1: Correct MODULE.md B.6 description

**File**: `research/tracks/B-algorithm/MODULE.md`
**Priority**: Critical (PP1)

Current text in MODULE.md paper table for B.6:
> | B.6+computational-complexity | foundations | O(n log² n) / O(√log n) approximation guarantee | draft | JACM |

This description is incorrect. The paper proves NP-hardness and characterizes METIS runtime. Read B.6's actual main.tex and determine:
- What is the actual theorem proved?
- Is any approximation bound derived?
- What is the paper's actual contribution?

Update MODULE.md to reflect the correct result. Propagate the correction to any Track A synthesis documents that cite "O(n log² n) / O(√log n)."

**Effort**: 1 hour (read abstract + introduction, correct description)

---

### Action B-IMMED-2: Resolve B.22b vs B.22 naming and scope

**Files**: `research/B.22b+cvd-geographic/` directory
**Priority**: PP1

B.22's abstract previews a "Phase 2 Geographic CVD" extension. B.22b appears to deliver that extension with 2–6% PP improvements over the basic CVD approach.

Choose one of:

**Option A (Merge)**: Incorporate B.22b's geographic CVD results as a new section or appendix in B.22. Retitle B.22 as "Centroidal Voronoi Districts: Geometric and Geographic Extensions." Remove B.22b as a standalone paper.

**Option B (Rename and differentiate)**: Rename B.22b as B.24 with a title that clearly distinguishes it from B.22. In B.24's introduction, explicitly state the contribution that B.22 does not contain. In B.22's Section 7 or conclusion, note that geographic extension is covered in B.24.

**Option C (Status quo clarification)**: If B.22b is genuinely distinct (different algorithm, not just Phase 2 of B.22), keep the current naming but add a clear "Relationship to B.22" section in B.22b's introduction.

Option A is recommended if the geographic extension content is not independently publishable; Option B if it is.

**Effort**: 2–4 hours to compare and decide; 1–2 days to implement chosen option

---

## Panel Review Queue (3–6 weeks each)

These papers have no panel review history and need full 5-reviewer panel reviews before any Track B papers citing them can be submitted.

### B.16 — ConvergenceSweep (Highest Priority)

**Why critical**: T=600 is the program's statutory standard cited by B.0, G.0, G.14, and H.0. If the T=600 derivation fails review, all downstream citations need revision.

**Recommended panel composition**:
- Karypis (computational: convergence analysis for graph partitioning)
- Rodden (political science: whether T=600 seeds provide meaningful partisan variance reduction)
- Duchin (mathematical: convergence guarantees)
- Chen (methodology: empirical derivation validity)
- Liang (statistics: whether the empirical methodology is adequately powered)

**Key questions for reviewers**:
1. Is T=600 derived theoretically, empirically, or by convention?
2. What statistical test establishes T=600 as adequate (vs. T=599 or T=601)?
3. Is the convergence definition (variance across seeds below threshold) the right criterion?
4. Does the T=600 formula generalize across states, or is it calibrated to specific state sizes?

**Target score**: 3.0+/4 required for the downstream program to rely on it.

---

### B.17 — Parameter Sensitivity (High Priority)

**Why critical**: The null finding (partisan outcomes robust to all parameter tuning) is used throughout the program's impossibility-defense claims.

**Recommended panel composition**:
- Karypis (METIS parameters: ufactor, niter, objtype)
- Chen (computational redistricting methodology)
- Duchin (null finding validity)
- Liang (statistical power analysis for null findings)
- Goodchild (geographic robustness)

**Key questions for reviewers**:
1. Is the null finding adequately powered to detect relevant effect sizes (e.g., 1-seat changes)?
2. What parameter ranges were tested? Are default METIS parameters included?
3. Is the 400+ run ensemble sufficient for all 50 states?
4. Is the claim "partisan outcomes invariant to tuning" qualified appropriately (vs. "partisan outcomes not significantly different from a null")?

---

### B.5 — N-Way vs. Recursive General Comparison

**Recommended panel**: Karypis, Çatalyürek, Chen, Duchin, Goodchild

**Key questions**: Does B.5 add value beyond B.4 (which proves equivalence under standard conditions)? What are the conditions under which n-way outperforms recursive?

---

### B.6 — Computational Complexity (After MODULE.md Correction)

**Conduct after B-IMMED-1**: Review only after the MODULE.md description is corrected to reflect the paper's actual results.

**Recommended panel**: Karypis, Çatalyürek, Phillips, Chen, Liang

**Key questions**: What is the actual complexity result? Is the NP-hardness reduction correct? Is the METIS runtime characterization O(niter × n log k) a new result or a restatement of known METIS analysis?

---

## Paper-Level Revision Items

### B.8 — GeoSection: R1 Blocking Item

**Priority**: PP2 (blocks submission)

**Blocking item from R1 review**: Formal definition of the bisection ratio and its relationship to the isoperimetric number of the partition is missing.

**Required for submission**:
- Add Definition 1 in Section 2: "We define the ratio-optimal bisection as the partition minimizing r = max(|E(V₁,V₂)|/|E(V₁,V₁)|, |E(V₁,V₂)|/|E(V₂,V₂)|) subject to balance tolerance ε."
- Or alternatively, specify the ratio in terms of the METIS ncon objective: "The ratio r corresponds to METIS ncon=2 with the second constraint being |V₁|/|V₂| ≤ threshold."
- Add Section 2.2: relationship between this ratio and the isoperimetric constant λ(G) = min_{S: |S|≤|V|/2} |E(S,V\S)| / (|S| × (1 - |S|/|V|))

**Effort**: 1–2 days (mathematical definition + relationship proof)

---

### B.15 — StabilitySection: Proposition Relabeling

**Priority**: PP2

**Item**: Proposition 1 states stability is guaranteed under mild conditions but is derived empirically across 50 states, not proved mathematically.

**Required**:
- Change "Proposition 1" to "Conjecture 1" and add a footnote: "This conjecture is supported empirically across all 50 states across three census decades (2000, 2010, 2020). A formal mathematical proof under the stated regularity conditions is left as an open problem."
- Alternatively: provide a formal proof. The conjecture may be provable under the condition that the bisection tree structure is determined by the census block graph topology, which changes slowly between censuses due to the US Census block boundary stability policy.

**Effort**: 15 minutes (rename) or 1–4 weeks (prove)

---

### B.12 — ProportionalSection: Positioning

**Priority**: PP2

**Item**: The proportionality paradox (sigma → 0 for competitive states) directly challenges the efficiency-gap-based partisan fairness literature. The paper should be explicitly positioned against this literature, not just as a neutral empirical observation.

**Required addition to Introduction** (~1 paragraph):
Add a paragraph engaging with the efficiency gap critique: the ProportionalSection result is not evidence that algorithmic redistricting fails partisan fairness, but rather that the partisan fairness goal is geometrically infeasible for competitive states under neutral optimization. The Rodden gap is a Lorenz feasibility constraint — the geographic distribution of partisan voters sets a floor on the achievable efficiency gap, independent of map-drawing methodology. Cite Stephanopoulos & McGhee (2015) and Rodden (2019).

**Effort**: 2–3 hours

---

### B.11 — ApportionRegions: Generalizability Qualification

**Priority**: PP3

**Item**: The NC 7D/7R result is NC-specific. The 223D/209R national result is more robust but rests on all states using prime-factor bisection, which may not be the legally mandated approach.

**Required addition** to Discussion (~1 paragraph):
"The NC 7D/7R result reflects the specific interaction between North Carolina's geographic partisan sorting and the prime-factor bisection tree structure that divides NC into a balanced 7×7 grid. This result does not generalize to 'every state produces proportional outcomes under ApportionRegions' — it reflects NC-specific geography. The 223D/209R national result is more robust, representing the aggregate outcome when all 50 states use prime-factor bisection, but individual state outcomes will vary based on geographic partisan concentration."

**Effort**: 1–2 hours

---

### B.1/B.2 — Framing Refinement

**Priority**: PP3

**Item**: B.1 and B.2 frame their contribution as "recursive bisection for redistricting" without sufficiently emphasizing the adaptation from the METIS literature.

**Required**: Brief paragraph in each paper's Introduction explicitly distinguishing the contribution from prior METIS applications (Karypis & Kumar 1998, Hendrickson & Leland 1995) and noting that the redistricting adaptation involves: (a) geographic edge weighting for community cohesion; (b) strict balance constraints for population equality; (c) 50-state empirical validation; (d) legal defensibility framing.

---

## Module Documentation Items

### Doc-1: Cross-track dependency table

Add to MODULE.md a "Cross-Track Dependencies" section documenting:

| Paper | Depends on / Depended on by |
|-------|----------------------------|
| B.6 | B.1, B.2 (runtime characterization) |
| B.11 | H.1 (cites prime-factor tree), A.0 (NC 7D/7R headline) |
| B.14 | D.0–D.3 (VRA methodology), F.6 (state legislative VRA) |
| B.16 | G.0, G.14, H.0 (T=600 formula citation) |

---

## Submission Priority Order

Given the revision landscape, recommended submission sequence:

1. **B.1** → APSR (ready; framing refinement optional)
2. **B.2** → APSR (ready; framing refinement optional)
3. **B.4** → APSR (recheck; near ready)
4. **B.9** → APSR (recheck; near ready)
5. **B.13, B.14, B.15** → APSR/Yale LJ (ready; B.15 Proposition rename required first)
6. **B.19–B.23** → NeurIPS/ICML/KDD/AAAI (accepted externally; submit now after B.22b decision)
7. **B.8** → APSR (after R1 blocking item resolved)
8. **B.3, B.11, B.12** → APSR (after recheck revisions)
9. **B.16, B.17** → After panel review completion
10. **B.5, B.6** → After panel review completion
11. **B.7, B.18** → After panel review completion
12. **B.22b / B.24** → After scope decision (B-IMMED-2)

---

## Timeline

| Week | Action |
|------|--------|
| 1 | B-IMMED-1: Correct MODULE.md B.6 description; B-IMMED-2: Decide B.22b scope |
| 1–2 | B.15: Proposition → Conjecture rename; B.12 positioning paragraph; B.8 blocking item |
| 2–4 | Initiate B.16 panel review (highest priority) |
| 3–5 | Initiate B.17, B.5 panel reviews (in parallel with B.16) |
| 5–8 | Initiate B.6 panel review (after MODULE.md correction) |
| 6–10 | Submit B.1, B.2, B.4, B.9, B.13, B.14, B.15 to target venues |
| 8–12 | Complete B.16/B.17 panel review cycles; submit or revise based on scores |
| 12+ | Submit B.5, B.6, B.8, B.18 based on review outcomes |
