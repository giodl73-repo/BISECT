# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, graph partitioning, multilevel algorithms
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made substantial improvements addressing all major concerns from Round 1. Most notably, the addition of partitioning quality analysis (P1.3), alternative partitioner comparison (P1.4), and recursive bisection justification (P1.5) significantly strengthens the algorithmic contribution. The paper now provides deeper insight into *why* edge weighting works beyond the intuitive "minimize perimeter" explanation.

The partitioning quality analysis (Section 3.2) is particularly valuable—demonstrating the topological vs geometric tradeoff (edge cuts +77%, perimeter -25%) clarifies METIS's behavior and explains why geometric optimization succeeds despite increased edge cuts. The insight that cutting 5 short urban edges (1 km each) beats cutting 1 long rural edge (20 km) is elegant and addresses my original concern about understanding METIS's behavior with geometric weights.

The alternative partitioner comparison (Section 3.5) validates that edge weighting generalizes beyond METIS—all three partitioners (METIS, KaHIP, Scotch) achieve similar compactness (within 0.3% average). This substantially improves the contribution's generality.

The recursive bisection justification (Section 2.3) is convincing: k-way partitioning offers 1.7% better compactness but suffers 20% contiguity failure rate. For redistricting, where contiguity is a legal requirement, the choice of recursive bisection is clearly correct.

## Updated Score

**Score**: 4/4 — **Accept**

*Upgrade from 3/4 in Round 1*

## P1 Items Addressed (My Areas)

### P1.3: Partitioning Quality Analysis ✓ EXCELLENT

The new Section 3.2 provides exactly what I requested:
- Edge-cut values: 2,156 (unweighted) → 3,805 (weighted), +77% increase
- Coarsening statistics: Average coarsening ratio ~5x per level
- Key insight: Topological optimization (minimize cuts) vs geometric optimization (minimize perimeter) are fundamentally different objectives
- The edge-length distribution analysis explains *why* more cuts can yield better compactness

This is the paper's strongest technical addition. It demonstrates deep understanding of METIS's behavior and provides actionable insight for practitioners.

**Remaining minor concern**: The analysis could benefit from showing convergence behavior—does Kernighan-Lin refinement converge differently with geometric weights? This is not blocking.

### P1.4: Alternative Partitioner Comparison ✓ COMPLETE

Section 3.5 compares METIS, KaHIP, and Scotch on 5 representative states:
- All three achieve similar compactness (within 0.3% average, max 1.86%)
- Validates that edge weighting generalizes across multilevel partitioners
- Justifies METIS choice (maturity, ease of use) while acknowledging alternatives work equally well

This fully addresses my concern about METIS-specific results. The contribution is now properly positioned as a general edge-weighting technique, not a METIS-specific hack.

**Note**: The paper could strengthen this by briefly mentioning ParMETIS for very large instances, but this is optional.

### P1.5: Recursive Bisection Justification ✓ CONVINCING

Section 2.3 compares recursive bisection to k-way on 10 states:
- K-way: 1.7% better compactness, 20% contiguity failure rate
- Recursive: 100% contiguity guarantee, 1.7x faster, hierarchical structure
- Clear conclusion: contiguity is absolute legal requirement, so recursive bisection is correct choice

This is excellent analysis. The paper correctly prioritizes robustness over marginal quality gains. For a practical redistricting system, 100% contiguity is non-negotiable.

**Insight**: The hierarchical structure of recursive bisection may also be valuable for nested district design (state legislature + congressional), though the paper doesn't pursue this.

## Observations on Other P1 Items

While not my primary expertise, I note the authors also addressed:

- **P1.1 (Partisan Analysis)**: Added efficiency gap, mean-median, partisan bias metrics. Results show mixed outcomes—compactness doesn't guarantee fairness. This is honest and important.

- **P1.2 (VRA Compliance)**: 68% reduction in majority-minority districts demonstrates fundamental tradeoff between compactness and VRA compliance. The proposed solutions (hybrid objectives, protected communities) are reasonable.

- **P1.6 (Neutrality Claims)**: Language updated throughout to distinguish "political blindness" from "partisan neutrality." The three-part taxonomy (algorithmic neutrality, political blindness, partisan neutrality) is clear and prevents misinterpretation.

## Remaining P2 Items

The authors completed P2.4 (County Preservation) and P2.5 (Geographic Sorting), which are outside my primary expertise. Six P2 items remain incomplete:

- P2.1: Approximation analysis / optimality bounds (Phillips/Hendrickson concern)
- P2.2: Multi-objective formulation (Duchin/Chen concern)
- P2.3: MCMC ensemble comparison (Duchin/Chen concern)
- P2.6: Indiana case study (multiple reviewers)
- P2.7: Census tract boundary limitations (Goodchild concern)
- P2.8: Hypergraph formulation (Çatalyürek concern)

These are all "important" rather than "blocking." The paper is publishable in its current form, though addressing these would strengthen it further.

## Minor Issues Resolved

From my Round 1 review:

- **m1 (METIS parameters)**: Still not fully justified, but the recursive vs k-way comparison validates `-niter=100` choice implicitly
- **m2 (Bridge weights)**: Not explicitly validated, but results are robust enough that this is acceptable
- **m3 (Recursive vs k-way)**: ✓ Fully addressed in Section 2.3
- **m4 (Subgraph extraction)**: Not discussed, but runtime results suggest overhead is acceptable
- **m5 (Memory consumption)**: Not reported, but no practical issues mentioned

These are all minor enough to not affect acceptance.

## Minor Issues Remaining

### m1: Precision Analysis Still Missing

My Round 1 concern about integer weight precision (centimeter vs millimeter) remains unaddressed. The paper claims centimeter precision is "adequate" but provides no sensitivity analysis.

**Recommendation**: For future work, test 1m, 10cm, 1cm, 1mm scaling and show results are stable.

### m2: Coarsening Behavior Not Fully Characterized

While edge-cut statistics are provided, the paper doesn't discuss how geometric weights affect METIS's coarsening phase. Do weighted graphs coarsen differently than unweighted?

**Not blocking**: This is deep algorithmic analysis that goes beyond typical application papers.

### m3: Block-Level Scalability Unvalidated

The paper mentions block-level data (1M+ units) as future work but provides no validation. Will METIS scale to 100K+ vertex graphs for large states?

**Recommendation**: Pilot study on 2-3 states at block level would strengthen scalability claims, but current tract-level validation is sufficient for publication.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Deep partitioning analysis**: The topological-geometric tradeoff explanation (Section 3.2) demonstrates expert understanding of METIS's behavior.

2. **Generalization validation**: Alternative partitioner comparison shows edge weighting is technique-agnostic, not METIS-specific.

3. **Honest empirical results**: Partisan analysis shows compactness ≠ fairness. VRA analysis shows compactness-representation tradeoff. This intellectual honesty strengthens credibility.

4. **Correct algorithm choice**: Recursive bisection justification demonstrates proper engineering judgment—prioritizing robustness over marginal gains.

5. **Comprehensive evaluation**: With partisan metrics, VRA compliance, partitioning quality, and alternative partitioners, the paper now provides thorough validation.

## Final Recommendations

**For publication acceptance**: None. The paper is ready for publication.

**For future work** (not blocking):
- Precision sensitivity analysis (1m, 10cm, 1cm, 1mm)
- Block-level scalability validation (2-3 pilot states)
- Coarsening phase analysis with geometric weights
- ParMETIS evaluation for very large instances

---

## Verdict

**Accept** — Ready for publication

**Changes from Round 1**: Upgraded from "Accept with Minor Revisions" (3/4) to "Accept" (4/4)

**Rationale**: All major algorithmic concerns have been addressed:
1. Partitioning quality analysis explains METIS's behavior with geometric weights
2. Alternative partitioner comparison validates generalization
3. Recursive bisection justification demonstrates correct algorithm choice

The remaining P2 items are important enhancements but not blocking for publication. The paper makes a solid contribution to both the graph partitioning and redistricting communities:
- **For graph partitioning**: Demonstrates that domain-specific edge weights can dramatically improve solution quality for geometric problems
- **For redistricting**: Provides practical, scalable method that outperforms human redistricting in compactness

The intellectual honesty about limitations (compactness ≠ fairness, VRA tradeoffs) strengthens rather than weakens the contribution.

**Confidence**: High — I designed METIS and can definitively assess this work's algorithmic quality. The revisions demonstrate expert-level understanding of graph partitioning. This is excellent application research.
