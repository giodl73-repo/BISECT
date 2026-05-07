# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Ümit V. Çatalyürek (Georgia Tech)
**Expertise**: Hypergraph partitioning, parallel algorithms
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made significant improvements addressing most Round 1 concerns. The addition of partitioning quality analysis (P1.3), alternative partitioner comparison (P1.4), and recursive bisection justification (P1.5) substantially strengthens the technical contribution. The paper now provides deeper algorithmic insight and validates generalization beyond METIS.

The partitioning quality analysis is particularly valuable—demonstrating that edge cuts increase 77% while perimeter decreases 25% clarifies the fundamental tradeoff between topological and geometric optimization. This explains why traditional graph partitioning metrics (edge cuts) don't predict compactness quality for this domain.

The alternative partitioner comparison validates that edge weighting generalizes across multilevel partitioners (METIS, KaHIP, Scotch within 0.3% average). This addresses my primary concern that results might be METIS-specific.

However, my major concern about hypergraph formulation (M1 from Round 1) remains unaddressed. The paper still doesn't justify why pairwise graph edges are sufficient when census tracts have multi-way adjacencies. This is a fundamental modeling choice that deserves analysis.

## Updated Score

**Score**: 3/4 — **Accept with Minor Revisions**

*No change from Round 1, though close to 4/4*

## P1 Items Addressed (My Areas)

### P1.3: Partitioning Quality Analysis ✓ EXCELLENT

Section 3.2 provides comprehensive analysis:
- Edge cuts: +77% increase (unweighted → weighted)
- Perimeter: -25% decrease
- Average edge length: -66% reduction
- Key insight: Cutting many short edges beats cutting few long edges for geometric compactness

This is exactly what I requested. The analysis demonstrates that traditional partitioning metrics (minimize edge cuts) don't translate to geometric quality. The edge-length distribution insight is valuable for the partitioning community.

**Minor observation**: The paper could discuss whether METIS's coarsening algorithm behaves differently with geometric weights, but this is not critical.

### P1.4: Alternative Partitioner Comparison ✓ GOOD

Section 3.5 compares METIS, KaHIP, Scotch on 5 states:
- All achieve similar compactness (within 0.3% average)
- Validates edge weighting generalizes to other multilevel partitioners
- Justifies METIS choice while acknowledging alternatives work equally well

This addresses my concern about METIS-specific results. The technique is now properly positioned as a general approach applicable to any multilevel graph partitioner with edge weight support.

**Suggestion**: Brief mention of hypergraph partitioners (PaToH, Zoltan) would strengthen positioning, even if not experimentally evaluated.

### P1.5: Recursive Bisection Justification ✓ ACCEPTABLE

Section 2.3 compares recursive to k-way:
- K-way: 1.7% better compactness, 20% contiguity failure rate
- Recursive: 100% contiguity, 1.7x faster
- Conclusion: Contiguity guarantee justifies recursive choice

This is reasonable justification. For redistricting, where contiguity is legally mandatory, the recursive approach's robustness is correct prioritization.

**From parallel algorithms perspective**: The 20% contiguity failure rate for k-way is concerning and suggests the balance constraint alone doesn't guarantee contiguity. The paper could briefly discuss why k-way fails contiguity while recursive succeeds (likely due to greedy refinement moving boundary vertices without global connectivity checks).

## Major Concern Still Unaddressed

### M1: Hypergraph Formulation Not Explored (STILL UNRESOLVED)

My primary Round 1 concern remains: **Why graphs instead of hypergraphs?**

Census tracts have multi-way adjacencies—a single boundary vertex often connects 3-6 tracts simultaneously. Hypergraph partitioning could model this more naturally:
- Each boundary segment = hyperedge connecting all adjacent tracts
- Hyperedge weight = segment length
- Cutting hyperedge = breaking that boundary

This might better optimize perimeter than pairwise edges. The paper still provides no:
- Conceptual comparison to hypergraph formulation
- Justification for why graphs suffice
- Discussion of whether hypergraphs could improve results

**Why this matters**: Hypergraph partitioning is my expertise area. Without justification for graph vs hypergraph choice, a fundamental modeling decision remains unexplained. This is the only reason I'm not upgrading to 4/4.

**Recommendation**: Add 1-2 paragraphs in Section 2 or Section 6 (Discussion) explaining:
1. Why pairwise graph edges adequately capture geometric constraints despite multi-way adjacencies
2. Whether hypergraph formulation was considered and if so, why graph model was chosen
3. Brief discussion of whether hyperedges could improve results (speculation acceptable)

If authors provide this justification, I would upgrade to 4/4 even without experimental hypergraph comparison.

## Minor Issues from Round 1

### M2: Parallelization (STATE-LEVEL ONLY) — ACCEPTABLE

The paper still parallelizes only at state level (6 workers), not within states. For large states (California: 15 min), finer-grained parallelism could help.

**However**: The recursive vs k-way comparison (P1.5) indirectly addresses this—recursive bisection is 1.7x faster than k-way, suggesting the current approach is efficient enough. Runtime results (2-3 hours for 50 states) demonstrate practical performance.

**Verdict**: Acceptable. Not blocking, though sibling-bisection parallelism or ParMETIS for large states could improve throughput in future work.

### M3: Scalability (BLOCK-LEVEL UNVALIDATED) — ACCEPTABLE

Block-level scalability claims remain unvalidated. The paper mentions 1M+ units as future work but provides no experiments.

**However**: Tract-level validation (50 states, 435 districts) is comprehensive enough for publication. Block-level claims are appropriately positioned as future work rather than current contributions.

**Verdict**: Acceptable. Future work should include block-level pilot studies.

## Minor Technical Observations

### Subgraph Extraction Overhead

My Round 1 concern about subgraph extraction efficiency (m1) remains unaddressed. For large states with O(log K) recursion levels, index remapping and file I/O could be significant overhead.

**Not blocking**: Runtime results suggest overhead is acceptable. Future optimization could use METIS's in-memory API.

### Load Balancing

My Round 1 concern about load balancing (m2) across workers is not discussed. With vastly different state sizes (Delaware vs California), static assignment may have imbalance.

**Not blocking**: 2-3 hour total runtime suggests load balancing is adequate, even if not optimal.

### Bridge Edge Strategy

My Round 1 concern about bridge connection overhead (m3) remains unaddressed.

**Not blocking**: The median boundary length heuristic is pragmatic and results suggest it works well.

## Observations on Other P1 Items

While not my primary expertise, I note:

- **P1.1 (Partisan Analysis)**: Added, shows compactness ≠ fairness
- **P1.2 (VRA Compliance)**: Added, shows 68% reduction in minority districts
- **P1.6 (Neutrality Claims)**: Language softened appropriately

These additions strengthen the paper's honesty about limitations.

## P2 Items Completed

- **P2.4 (County Preservation)**: Added, shows modest tradeoff
- **P2.5 (Geographic Sorting)**: Added, quantifies geographic vs gerrymandering effects

Both are outside my expertise but appear thorough.

## Remaining P2 Items

Six P2 items remain incomplete, including:
- **P2.8 (Hypergraph Formulation)**: My area—see M1 above

The others (approximation analysis, multi-objective, MCMC, Indiana, census tracts) are for other reviewers to assess.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Partitioning quality insight**: Topological-geometric tradeoff is valuable for partitioning community
2. **Generalization validation**: Alternative partitioner comparison demonstrates technique is not METIS-specific
3. **Honest evaluation**: Partisan/VRA analysis shows limitations of single-objective optimization
4. **Robust algorithm**: Recursive bisection's 100% contiguity guarantee vs k-way's 20% failure rate justifies choice

## Final Recommendations

**For 4/4 upgrade**: Add brief (1-2 paragraph) justification for graph vs hypergraph modeling choice. This doesn't require experiments—conceptual discussion suffices.

**For future work** (not blocking):
- Hypergraph partitioner comparison (PaToH, Zoltan) on pilot states
- Block-level scalability validation
- Sibling-bisection parallelism or ParMETIS for large states
- In-memory METIS API to eliminate file I/O overhead

---

## Verdict

**Accept with Minor Revisions** — Very close to ready, needs hypergraph justification

**Changes from Round 1**: No score change (still 3/4), but much closer to 4/4

**Rationale**: All P1 items in my expertise area are addressed:
- Partitioning quality analysis is excellent
- Alternative partitioner comparison validates generalization
- Recursive bisection justification is sound

**Only remaining concern**: Hypergraph vs graph modeling choice is still unjustified. This is the sole reason for not upgrading to 4/4. If authors add 1-2 paragraphs discussing this (Section 2 or Section 6), I would immediately upgrade to Accept (4/4).

The paper makes a solid contribution demonstrating that domain-specific edge weights dramatically improve geometric partitioning quality. This is valuable for both redistricting and the broader graph partitioning community.

**Confidence**: High — I have extensive experience with graph and hypergraph partitioning. The technical quality is excellent. The hypergraph question is a legitimate modeling concern that deserves brief discussion.
