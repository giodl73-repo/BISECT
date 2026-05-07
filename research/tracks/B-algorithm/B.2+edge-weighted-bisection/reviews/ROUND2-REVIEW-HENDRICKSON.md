# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Bruce Hendrickson (Sandia National Labs)
**Expertise**: Graph partitioning, load balancing, spectral methods
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made substantial improvements addressing all major algorithmic concerns from Round 1. The addition of partitioning quality analysis (P1.3), alternative partitioner comparison (P1.4), and recursive bisection justification (P1.5) significantly strengthens the technical depth. The paper now provides valuable insights into partition structure and algorithmic behavior beyond pure empirical validation.

The partitioning quality analysis (Section 3.2) is the paper's strongest technical addition—demonstrating that edge cuts increase while perimeter decreases clarifies the fundamental tension between topological and geometric optimization. This insight is valuable for the graph partitioning community: traditional metrics (minimize edge cuts) don't predict geometric solution quality.

The recursive bisection justification is particularly convincing: k-way partitioning's 20% contiguity failure rate vs recursive bisection's 100% success demonstrates that greedy refinement can violate global connectivity constraints. This is an important practical observation.

However, the paper still lacks theoretical analysis—no approximation bounds, optimality analysis, or structural characterization of solutions. For applications venues (KDD, AAAI), the current empirical depth is acceptable. For theory venues (SODA), more work would be needed.

## Updated Score

**Score**: 4/4 — **Accept** (for KDD/AAAI)
**Score**: 3/4 — **Accept with Minor Revisions** (for SODA)

*Upgrade from 3/4 in Round 1 for applications venues*

## P1 Items Addressed (My Areas)

### P1.3: Partitioning Quality Analysis ✓ EXCELLENT

Section 3.2 provides comprehensive partitioning quality analysis:
- Edge cuts: +77% increase (2,156 → 3,805)
- Perimeter: -25% decrease
- Average edge length: -66% reduction
- Key insight: Cutting 5 short edges (1 km each) beats 1 long edge (20 km)

This is exactly what I requested. The analysis reveals *why* edge weighting works: METIS's edge-cut minimization with geometric weights naturally favors many short cuts over few long cuts, which translates to lower perimeter.

**Theoretical perspective**: This suggests the algorithm is optimizing a weighted edge cut that correlates with perimeter, not perimeter directly. The paper could formalize this relationship, but the empirical insight is valuable.

**Minor gap**: The paper doesn't show how this behavior emerges from METIS's multilevel algorithm (coarsening + refinement). Does geometric weighting affect matching quality during coarsening? This is beyond typical application papers.

### P1.4: Alternative Partitioner Comparison ✓ GOOD

Section 3.5 compares METIS, KaHIP, Scotch on 5 states:
- All three within 0.3% average compactness (max deviation 1.86%)
- Validates edge weighting generalizes to multilevel partitioners
- Justifies METIS choice while acknowledging alternatives work equally

This addresses my concern about whether results are METIS-specific or general to multilevel partitioning. The answer is clear: edge weighting benefits any multilevel partitioner that supports weighted edges.

**Theoretical note**: All three partitioners use similar multilevel frameworks (coarsening + refinement), so similar performance is expected. The paper could briefly mention this.

### P1.5: Recursive Bisection Justification ✓ STRONG

Section 2.3 compares recursive to k-way:
- K-way: 1.7% better compactness, 20% contiguity failure rate
- Recursive: 100% contiguity guarantee, 1.7x faster
- Clear conclusion: Robustness justifies recursive choice

This is excellent analysis and addresses my Round 1 concern. The 20% contiguity failure rate for k-way is striking—it suggests that METIS's greedy Kernighan-Lin refinement can move boundary vertices in ways that disconnect components without global connectivity checks.

**Theoretical insight**: This demonstrates a fundamental limitation of local-search refinement: optimizing a global objective (perimeter) with local moves (vertex swaps) can violate global constraints (contiguity). Recursive bisection avoids this by maintaining contiguity at each level.

**For theory venues**: This observation could motivate theoretical analysis of when local refinement preserves global properties.

## Major Theoretical Gaps (Not Blocking for KDD/AAAI)

### M1: Approximation Bounds Still Missing

The paper provides no approximation analysis or optimality bounds:
- What's the approximation ratio vs optimal compactness?
- Are there lower bounds on achievable perimeter?
- How far from optimal is recursive bisection vs optimal k-way?

Indiana's enacted plan (0.478 vs algorithmic 0.353, +35% gap) suggests significant room for improvement, but we don't know if this represents:
1. Suboptimality of recursive bisection
2. Suboptimality of METIS heuristics
3. Actual optimality being achieved by Indiana

**For SODA**: This would need to be addressed. For KDD/AAAI, it's acceptable to lack bounds.

### M2: Structural Properties Still Not Characterized

The paper doesn't characterize partition structure:
- What graph properties (diameter, conductance, expansion) distinguish weighted from unweighted partitions?
- Do weighted partitions have lower cut ratio or better balance?
- Are there identifiable geometric patterns?

**I requested this in Round 1 but acknowledge it's deep analysis beyond typical application papers.**

### M3: Spectral Properties Not Analyzed

My Round 1 request for Laplacian eigenvalue analysis remains unaddressed. Do geometric weights improve spectral gap? How do Fiedler vectors differ?

**Not blocking**: This is theoretical depth appropriate for specialized graph theory venues, not general applications venues.

## Minor Issues from Round 1

Most minor issues from Round 1 remain unaddressed but are acceptable:

- **m1 (Spectral properties)**: Not analyzed — acceptable for applications venues
- **m2 (Unequal partition sizes)**: Not analyzed — acceptable, unlikely to significantly affect results
- **m3 (Bridge weights)**: Still heuristic — acceptable, results suggest it works
- **m4 (Point adjacency 0.1m)**: Still unjustified — acceptable, low impact
- **m5 (Population balance tolerance)**: Not analyzed — acceptable, 0.5% is standard

None of these are blocking for applications venues.

## Observations on Other P1 Items

While not my primary expertise:

- **P1.1 (Partisan Analysis)**: Excellent addition showing compactness ≠ fairness. Mixed results (54% states improved MMD, 36% EG) demonstrate that geographic sorting dominates geometric optimization.

- **P1.2 (VRA Compliance)**: 68% reduction in majority-minority districts is striking—shows fundamental tension between compactness and representation. The proposed solutions (hybrid objectives, protected communities) are reasonable.

- **P1.6 (Neutrality Claims)**: Language updated appropriately. Three-part taxonomy (algorithmic neutrality, political blindness, partisan neutrality) clearly distinguishes what the algorithm guarantees vs what it doesn't.

These additions substantially strengthen the paper's intellectual honesty.

## P2 Items Completed

- **P2.4 (County Preservation)**: Added county split analysis, shows modest tradeoff
- **P2.5 (Geographic Sorting)**: Quantifies geographic vs gerrymandering effects

Both are outside my expertise but appear thorough.

## Remaining P2 Items (My Area)

- **P2.1 (Approximation Analysis)**: My primary remaining concern—see M1 above. For KDD/AAAI, not blocking. For SODA, needed.

Other P2 items (multi-objective, MCMC, Indiana, census tracts, hypergraph) are for other reviewers.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Partitioning quality insight**: Topological vs geometric tradeoff is valuable for partitioning community—demonstrates that traditional metrics don't predict geometric quality

2. **Algorithmic robustness**: K-way's 20% contiguity failure rate is important practical observation about local refinement limitations

3. **Generalization validation**: Alternative partitioner comparison shows edge weighting is technique-agnostic

4. **Honest empirical assessment**: Partisan and VRA analysis show limitations, strengthening credibility

5. **Clear algorithmic choices**: Recursive bisection justification demonstrates proper engineering judgment

## Target Venue Recommendations

**For KDD/AAAI (applications focus)**: The paper is ready for publication (4/4). The empirical validation is comprehensive, algorithmic insights are valuable, and limitations are honestly discussed.

**For SODA (theory focus)**: Minor revisions needed (3/4). Add:
- Approximation bounds or lower bounds on achievable compactness
- Structural characterization of solutions (graph metrics)
- At minimum, theoretical discussion of when/why edge weighting should help

**For JEA (experimental algorithms)**: The paper is strong (4/4). The experimental methodology is rigorous and insights are valuable.

## Final Recommendations

**For acceptance at KDD/AAAI**: None. Ready for publication.

**For SODA submission** (if targeting theory venue):
- Add approximation analysis or lower bounds (even informal)
- Characterize partition structure with graph metrics
- Provide theoretical discussion of edge weighting effectiveness

**For future work** (not blocking):
- Spectral analysis of weighted vs unweighted graphs
- Sensitivity analysis on population tolerance, edge weights
- Small-instance optimality via MILP for states with 2-4 districts
- Theoretical conditions under which edge weighting guarantees improvement

---

## Verdict

**Accept** — Ready for publication at applications venues (KDD, AAAI, JEA)
**Accept with Minor Revisions** — For theory venues (SODA), needs approximation bounds

**Changes from Round 1**: Upgraded from "Accept with Minor Revisions" (3/4) to "Accept" (4/4) for applications venues

**Rationale**: All major algorithmic concerns for applications venues are addressed:
1. Partitioning quality analysis reveals why edge weighting works
2. Alternative partitioner comparison validates generalization
3. Recursive bisection justification demonstrates robustness priority is correct
4. Honest assessment of limitations (compactness ≠ fairness, VRA tradeoffs) strengthens paper

For applications venues (KDD, AAAI), the empirical depth and algorithmic insights are excellent. The paper makes valuable contributions to both graph partitioning (domain-specific weights improve geometric solutions) and redistricting (automated methods can outperform human redistricting on compactness).

For theory venues (SODA), additional approximation analysis would be needed, but this is venue-dependent, not a fundamental flaw.

**Confidence**: High — I have deep experience with graph partitioning theory and spectral methods. This is now a strong empirical paper with good algorithmic insights. The quality is appropriate for top-tier applications venues.
