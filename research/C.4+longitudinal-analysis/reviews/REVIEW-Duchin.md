# Review: Twenty Years of Congressional Redistricting

**Reviewer**: Moon Duchin (Tufts University / MGGG Redistricting Lab)
**Expertise**: Geometric algorithms for redistricting, ensemble methods, compactness
**Date**: 2026-02-08
**Round**: 1

---

## Overall Assessment

Strong methodological paper demonstrating temporal consistency of METIS-based redistricting across 2000-2020. The algorithmic approach is sound and the longitudinal design is valuable. However, the paper would benefit from ensemble comparison (not just single METIS runs), more discussion of compactness metric choice, and acknowledgment of the impossibility-of-gerrymandering claims that require partisan data you explicitly exclude.

**Verdict**: Accept with moderate revisions

---

## Score

**3.5 / 4.0** — Strong paper, minor revisions needed

---

## Major Issues

### M1. Single-Run Determinism vs Ensemble Methods

**Issue**: METIS is deterministic given fixed parameters, but redistricting scholarship increasingly uses ensemble methods (Markov chains, recombination) to characterize the *distribution* of possible plans, not just a single point estimate.

**Evidence**: You claim algorithmic redistricting is "durable" (abstract, discussion), but durability relative to what baseline? A single METIS run is just one of many possible algorithmically-generated plans.

**Impact**: Without ensemble comparison, we cannot assess:
- Whether METIS compactness (0.46 PP) is optimal or merely acceptable
- How much variance exists in algorithmic plans
- Whether temporal stability is algorithm-specific or general

**Recommendation**: Generate N=1000 ensemble plans per state/year using ReCom (recombination Markov chain). Compare METIS compactness to ensemble median/mean. If METIS is near the ensemble center, it validates the approach. If it's an outlier, it raises questions.

---

### M2. Polsby-Popper Limitations

**Issue**: Exclusive use of Polsby-Popper (PP) compactness without discussing its known limitations or comparing to alternatives (Reock, Convex Hull, Cut Edges).

**Evidence**: PP penalizes perimeter length but is sensitive to coastlines and jagged boundaries. Hawaii scores 0.18 (Section 5.3)—is this "less compact" or just coastal geography?

**Impact**: PP may conflate geography with gerrymandering. States with irregular boundaries (coastlines, mountains) score low even with fair districts.

**Recommendation**:
1. Add Reock compactness (circle-based, less perimeter-sensitive)
2. Normalize by state geography: compare enacted vs algorithmic *within* each state
3. Discuss metric choice in methodology and limitations

---

## Minor Issues

### m1. Graph Construction Details Missing

METIS partitions graphs, but you don't specify:
- How census tracts become graph nodes (centroids?)
- How edges are defined (Rook adjacency? Queen? Shared vertices?)
- How edge weights are computed (α = 5.0 for minority-minority, but what about other edges?)

**Recommendation**: Add explicit graph construction pseudocode or point to reproducibility supplement.

---

### m2. Population Balance Tolerance

You specify ±0.5% population balance (Section 3.2) but don't justify this choice. SCOTUS has upheld stricter standards (near-zero deviation for congressional districts). Why not ±0.1%?

**Recommendation**: Justify tolerance or test sensitivity (rerun with ±0.1% and ±1% to show robustness).

---

### m3. Impossibility Defense Contradiction

You claim "impossibility defense maintained" (no partisan data) but then discuss partisan implications extensively (red vs blue state gains, Section 4.2). This is internally inconsistent.

**Recommendation**: Either fully commit to non-partisan analysis (remove political color coding) or acknowledge that post-hoc partisan analysis doesn't violate impossibility defense for *generating* districts.

---

## Strengths

1. **Methodological consistency**: Same algorithm + parameters across 20 years is exemplary for longitudinal study

2. **IoU metric**: Geographic stability via Intersection-over-Union is clever and well-executed

3. **Transparency**: Clear description of METIS approach, population balance, edge weighting

4. **Scale**: All 50 states, 3 census years, 435 districts—comprehensive

---

## Questions for Authors

1. Have you generated ensemble plans to validate METIS as representative?
2. How sensitive is compactness to METIS parameters (different α values)?
3. Can you share graph construction code for reproducibility?

---

**Bottom Line**: Solid algorithmic work with good longitudinal design. The METIS-exclusive focus is defensible but limits generality. Ensemble comparison would substantially strengthen claims about "durable foundation" for algorithmic redistricting.
