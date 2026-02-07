# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Bruce Hendrickson (Sandia National Labs)
**Expertise**: Graph partitioning, load balancing, spectral methods
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper demonstrates that incorporating geometric edge weights into recursive bisection substantially improves redistricting compactness. The empirical results are strong: 56% improvement over unweighted baseline and superiority over enacted districts in 37 of 50 states. The work is technically competent and addresses a real-world problem with practical impact.

From a theoretical perspective, however, the paper is primarily empirical. There is no analysis of approximation guarantees, optimality bounds, or structural properties of the solutions. The recursive bisection approach is reasonable but known to be suboptimal compared to optimal k-way partitioning—the greedy top-down strategy makes locally optimal cuts that may be globally poor. The paper doesn't quantify this gap or explore alternative strategies.

More fundamentally, the paper lacks insight into *why* edge weighting works so well. The intuitive explanation (minimize perimeter = minimize Polsby-Popper denominator) is correct but shallow. What properties of geographic graphs make weighted partitioning effective? Are there theoretical conditions under which edge weighting guarantees improvement? The 76% tract reassignment rate (Alabama) suggests fundamentally different partition structures, but the paper doesn't characterize these differences formally.

This is a good application paper with strong empirical validation. However, it misses opportunities for theoretical analysis that would elevate it to an excellent contribution. For a venue like SODA, theoretical depth is expected; for KDD or AAAI, the current empirical focus may be acceptable.

## Score

**Score**: 3/4 — **Accept** (for KDD/AAAI); **Weak Accept** (for SODA)

## Major Issues (Blocking)

### M1: No Approximation Analysis or Optimality Bounds

The paper reports compactness improvements but doesn't establish:
- Approximation ratio: How far from optimal compactness are the solutions?
- Lower bounds: What's the best achievable Polsby-Popper for each state?
- Recursive bisection gap: How much compactness is lost vs optimal k-way partitioning?

Without bounds, we can't assess whether 0.367 national mean is near-optimal or has significant room for improvement. Indiana's enacted plan (0.478) suggests algorithmic solutions may be far from optimal. Provide:
- Theoretical lower bounds (even loose ones)
- Comparison to optimal solutions for small problem instances (synthetic or small states)
- Gap analysis between recursive bisection and k-way partitioning

### M2: Structural Properties of Solutions Not Characterized

The 76% tract reassignment rate (Alabama) indicates qualitatively different partitions, but the paper doesn't characterize this formally:
- What graph-theoretic properties do edge-weighted partitions have?
- Are they more balanced in degree distribution, diameter, or cut ratio?
- Do districts have higher clustering coefficients or lower conductance?
- Are there identifiable patterns in partition boundaries?

Include structural analysis: compute graph metrics (diameter, conductance, modularity, expansion) for weighted vs unweighted partitions. This would provide insight into *why* edge weighting produces better compactness beyond "minimizes perimeter."

### M3: Alternative Partitioning Strategies Not Explored

Recursive bisection is one of many partitioning strategies. Others may produce superior compactness:
- **Spectral partitioning**: Eigenvectors of Laplacian matrix (continuous relaxation of discrete problem)
- **Direct k-way partitioning**: METIS's k-way mode avoids greedy bisection
- **Multilevel without coarsening**: Flat refinement from random initialization

The paper doesn't justify recursive bisection vs alternatives or explore whether different strategies benefit more/less from edge weighting. At minimum:
- Compare recursive bisection to direct k-way METIS
- Discuss why recursive bisection is appropriate for this problem
- Provide intuition for when recursive bisection vs k-way is preferable

## Minor Issues

### m1: Spectral Properties of Weighted Graphs Not Discussed

Graph Laplacian eigenvalues reveal partitionability. Do geometric edge weights change spectral properties?
- Compare Laplacian eigenvalue distributions (weighted vs unweighted)
- Plot Fiedler vector (2nd eigenvector) for representative states
- Analyze eigengap (spectral gap between 2nd and 3rd eigenvalues)

This would show whether geometric weighting makes graphs "easier" to partition well.

### m2: Unequal Partition Sizes Not Analyzed

For K=7 districts, recursive bisection uses splits (4,3), (2,2), (2,1), ... The paper doesn't analyze how unequal splits affect compactness:
- Do larger partitions have worse compactness?
- Is there systematic bias introduced by split order?
- Would different recursion trees (e.g., (3,4) instead of (4,3)) improve results?

Show that split ordering doesn't significantly affect compactness or use optimal ordering.

### m3: Bridge Edge Weights Heuristic

Median land boundary length for bridge edges is heuristic. Alternatives:
- Shortest path distance between disconnected components
- Minimum spanning tree cost
- Area-weighted centroid distance

Why median? Provide sensitivity analysis showing results are robust to bridge weight choice.

### m4: Point Adjacency Weight (0.1m) Unjustified

Tracts sharing only a vertex receive 0.1m edge weight. Why 0.1m? Why not 0.01m or 1m? Does this affect results for states with many point adjacencies?

### m5: Population Balance Imbalance Factor Selection

`-ufactor=1.005` allows 0.5% population imbalance. Why 0.5%? Constitutional requirement is stricter for congressional districts (<1 person deviation in some states). Does tighter tolerance (0.1%, 0.001%) improve or degrade compactness? Trade-off analysis needed.

## Strengths

1. **Strong empirical validation**: National scale (50 states, 435 districts) with multiple baselines demonstrates practical effectiveness.

2. **Clear problem formulation**: Topology vs geometry mismatch is well-articulated; weighted edge cuts = total perimeter is elegant.

3. **Comprehensive methodology**: Graph construction pipeline handles real-world complexity (water crossings, islands).

4. **Reproducibility**: Detailed configuration and open-source promise enable verification.

5. **Practical impact**: 20% improvement over enacted districts, 37/50 states surpassed, gerrymandering analysis is policy-relevant.

6. **Computational efficiency**: O(N log K) complexity, 2-3 hours for 50 states is excellent scalability.

## Questions for Authors

1. **Optimality gap**: What's the approximation ratio? Can you bound the gap between algorithmic compactness and optimal? Even loose bounds would help.

2. **Recursive bisection vs k-way**: Does direct k-way METIS produce better/worse compactness? Why choose recursive bisection?

3. **Spectral analysis**: How do Laplacian eigenvalues differ between weighted and unweighted graphs? Does weighting improve spectral gap?

4. **Partition structure**: What graph properties (diameter, conductance, modularity) characterize weighted partitions? Why do they have better compactness?

5. **Indiana outlier**: Their enacted plan (0.478) is 35% better than algorithmic (0.353). What structural differences explain this? Can we learn from their approach?

6. **Small instance optimality**: For small states (Delaware: 1 district, Vermont: 1 district, Wyoming: 1 district), can you compute optimal compactness and measure gap?

7. **Bridge sensitivity**: How sensitive are results to bridge edge weights? Would shortest-path distances instead of median boundaries change compactness significantly?

## Recommendations

- **Add approximation analysis**: Provide lower bounds on achievable compactness (even informal ones) and estimate gap from optimal.

- **Structural characterization**: Compute graph metrics (diameter, conductance, expansion) for weighted vs unweighted partitions; show qualitative differences.

- **Compare k-way partitioning**: Run METIS direct k-way mode on representative states to validate recursive bisection choice.

- **Spectral analysis**: Plot Laplacian eigenvalues and Fiedler vectors for 2-3 states; show edge weighting improves spectral properties.

- **Small instance optimality**: For states with 2-4 districts, compute or approximate optimal compactness; measure algorithmic gap.

- **Sensitivity analysis**: Vary bridge weights, point adjacency weights, population tolerance; show results are robust.

- **Theoretical discussion**: Even without formal proofs, provide intuition for when/why edge weighting should help. Are there graph classes where weighting provably improves cuts?

---

**Verdict**: Accept with Minor Revisions (for KDD/AAAI); Major Revisions (for SODA)

**Confidence**: High — I have deep experience with graph partitioning theory and spectral methods. This paper is a solid empirical application with strong results. For applications-focused venues (KDD, AAAI), the current empirical depth suffices with minor additions. For theory venues (SODA), substantial theoretical analysis is needed: approximation bounds, structural characterization, and comparison to optimal solutions. The authors should target their revisions to the intended venue's expectations.
