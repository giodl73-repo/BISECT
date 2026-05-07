# Review: Slice-Based Cross-Census Validation for Congressional Redistricting Algorithms

**Reviewer**: Ümit V. Çatalyürek (Georgia Tech)
**Expertise**: Hypergraph partitioning, parallel algorithms
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

From a graph partitioning perspective, this paper presents an interesting validation methodology for evaluating recursive bisection algorithms across multiple datasets. The core idea—using stable geographic slices to control for data distribution changes—is sound. The application to METIS recursive bisection is appropriate given METIS's widespread use in redistricting.

However, the paper has significant algorithmic and computational gaps. The abstract mentions "computational efficiency" evolving across census cycles but provides no runtime data, scalability analysis, or complexity discussion. The treatment of graph construction is superficial—how adjacency graphs are built, weighted, and how edge weights affect partitioning quality is not discussed. For a paper targeting an algorithms-aware venue like SIGSPATIAL, this is insufficient.

## Score

**Score**: 3/4 — **Accept**

## Major Issues (Blocking)

### M1: Graph Construction Not Detailed
The paper applies METIS to census tract graphs but doesn't specify: (1) How is the adjacency graph constructed? (queen/rook contiguity? shared boundary length?), (2) Are edges weighted? If so, by what (distance, shared boundary, inverse perimeter)?, (3) How many edges per node on average? Graph structure fundamentally affects partitioning quality, and METIS is sensitive to edge weights.

### M2: Computational Complexity Missing
The abstract claims to evaluate "computational efficiency" but I see no runtime data, scaling analysis, or complexity discussion. For 50 states × 3 census years × multiple slices per state, what is the total computation time? How does runtime scale with graph size (O(n log n)? O(m)? where n=nodes, m=edges)? This is essential for a algorithms paper.

### M3: METIS Parameters Not Specified
METIS has numerous parameters: imbalance tolerance, number of bisection attempts, refinement iterations, coarsening scheme. Which parameters were used? Were they tuned per state or held constant? Parameter choices significantly affect output quality and must be documented.

## Minor Issues

### m1: No Comparison to Alternative Partitioning Methods
The paper validates METIS but doesn't compare against alternatives (spectral partitioning, KaHIP, PUNCH, simulated annealing). Is METIS-specific variance greater or less than inter-algorithm variance? This contextualizes whether cross-census validation is revealing algorithm behavior or partitioning heuristics in general.

### m2: Load Balancing Not Discussed
Congressional districts must have equal population (±0.5% tolerance), which is a strict load balancing constraint. How is this enforced in METIS? Via node weights? Post-processing? The interaction between population balance and compactness (edge-cut) is central to redistricting and should be discussed.

### m3: Parallel Scalability Not Explored
METIS has parallel variants (ParMETIS). Given the 50-state scale, was parallelization used? If not, why? If so, did it affect partitioning quality (parallel METIS can give different results than serial)?

### m4: Graph Metrics Missing
Standard graph partitioning metrics should be reported: edge-cut reduction, bisection quality, imbalance ratios. These are more fundamental than compactness scores and allow comparison to other partitioning contexts (circuit design, mesh partitioning).

## Strengths

1. **Large-scale evaluation**: 50 states across 3 census years is comprehensive and demonstrates serious empirical work.
2. **Relevant algorithm**: METIS is widely used and validating it across realistic datasets is valuable.
3. **Reproducibility potential**: The framework can be applied to other graph partitioning algorithms, enabling systematic comparison.
4. **Algorithmic insight**: The claim that geographic structure dominates temporal changes is interesting if supported by data.

## Questions for Authors

1. What is the range of graph sizes across states (smallest vs largest in terms of nodes and edges)?
2. How does METIS edge-cut quality correlate with geographic compactness metrics (Polsby-Popper, Reock)?
3. Have you compared METIS recursive bisection to METIS k-way partitioning (direct k-way split)?
4. Does the slicing process affect the graph structure (e.g., do slices induce disconnected components)?
5. Can the validation framework detect when METIS gets stuck in local minima?

## Recommendations

- Add "Graph Construction and Partitioning" subsection detailing adjacency definition, edge weighting, METIS parameters
- Include a table of runtime statistics: mean/min/max time per state-year, total computation time, scaling analysis
- Provide graph statistics: nodes, edges, degree distribution, connected components for representative states
- Report edge-cut ratios in addition to compactness metrics
- If possible, compare against one alternative partitioning algorithm to show the framework is not METIS-specific

---

**Verdict**: **Accept with Minor Revisions**

**Confidence**: High — Graph partitioning is my core expertise and I am confident these additions would significantly strengthen the paper.
