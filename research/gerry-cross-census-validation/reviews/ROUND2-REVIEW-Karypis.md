# Round 2 Review: Slice-Based Cross-Census Validation

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, graph partitioning, multilevel algorithms
**Round**: 2
**Date**: 2026-02-07

---

## Assessment of Revisions

The new section 3.4 (METIS Configuration) is comprehensive and demonstrates proper understanding of METIS. The authors correctly specify KMETIS (multilevel recursive bisection), document all key parameters (ufactor=1 for tight balance, niter=20, ncuts=10, edge-cut objective), and explain population balance enforcement via node weights.

The 10-run stochasticity analysis is excellent—this addresses my primary concern about METIS's non-determinism. Reporting mean/std across runs is the right approach. The edge-cut statistics table provides METIS's native quality metric, which complements the geometric compactness scores nicely.

The graph construction details (Rook contiguity, boundary-length weights) are appropriate for redistricting. The runtime scaling confirmation (O(n log n)) matches METIS's theoretical complexity.

## Updated Score

**Score**: 4/4 — **Strong Accept**

(Increased from 3/4)

## Remaining Minor Issues

1. **METIS version**: The authors don't specify which METIS version (5.x has different defaults than 4.x). A footnote would help future reproducibility.

2. **Coarsening scheme**: While most parameters are documented, the coarsening matching algorithm is not mentioned. Default is sorted heavy-edge matching, but confirming this would be thorough.

3. **k-way comparison**: The authors note recursive vs k-way comparison as future work. I agree it would be interesting, but not essential for this paper's contribution.

## Verdict

**Strong Accept** — This is now a rigorous, reproducible evaluation of METIS for redistricting. The methodological documentation is exemplary and will enable other researchers to build on this work.

**Confidence**: High — The METIS-specific aspects are now properly treated. This paper can serve as a reference for how to rigorously evaluate graph partitioning algorithms on real-world geographic data.
