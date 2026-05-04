# Review — R-9 (Joseph Gonzalez)
**Score: 3.0/4.0**

## Summary
PFR provides a well-motivated hierarchical graph partitioning framework, and the empirical 50-state sweep demonstrates that the algorithm's outputs are geographically rather than algorithmically determined. The hierarchical partitioning design is sound and the partition cache is a reasonable memoization strategy, but the paper leaves key graph-algorithmic questions unanswered: the communication pattern between levels, load balance across subgraph sizes, and behavior of METIS edge-weighting under the prime factorization structure.

## Strengths
- The hierarchical partitioning structure (factorization tree → sequence of k-way METIS calls on subgraphs) is a natural and principled approach to multi-level graph partitioning. PFR exploits METIS's strength at small-to-medium graph partitioning while avoiding the instability of direct k-way METIS for large k.
- The partition cache (region_hash, n_parts, seed) → assignment is a clean memoization over the factorization tree, essentially a dynamic programming approach where subproblems are identified by their graph content. This is exactly the right design for a system where the same subgraphs appear across different seat counts.
- The NC/GA finding provides a compelling empirical validation of geographic determinism. From a graph-theoretic standpoint, this is equivalent to showing that the graph's community structure determines partisan outcomes when partitioning is done optimally.

## Weaknesses
- The paper does not characterize the subgraph size distribution across the factorization tree. For CA (k=52=4×13), the 13-way top-level partition produces 13 subgraphs; if these are highly imbalanced, the METIS calls at the second level will have very different runtimes. The paper should report subgraph size statistics.
- The paper uses METIS edge weights but does not describe how these interact with hierarchical partitioning. If edge weights were calibrated for the full-state graph (e.g., normalized by total boundary length), they may be poorly calibrated for subgraph partitioning.
- The independence of subproblems at each level is asserted but not verified. PFR minimizes edge cut level by level, not globally. The paper should acknowledge this approximation and compare global edge cut of PFR partitions to direct k-way METIS for small states.

## Detailed Comments

The factorization tree structure maps naturally onto a tree decomposition of the partitioning problem. From a graph-processing perspective, PFR is computing a hierarchical clustering of the census tract graph where cluster sizes are constrained by the factorization of n. The key question for graph-processing audiences is whether the hierarchical approach produces partitions competitive with direct k-way partitioning — the paper does not directly address this. Adding even a 5-state comparison of PFR vs. direct k-way METIS for states with k ≤ 10 would significantly strengthen the paper's algorithmic claims.

The communication pattern in PFR's hierarchical partitioning is sequential: top-level partition runs first, then second-level partitions run on each subgraph independently (in parallel). This is a tree-structured computation with no cross-level communication, excellent for parallelism. The paper should make this explicit: PFR's factorization tree enables embarrassing parallelism at each level after the first, a practical advantage over monolithic k-way METIS for large k.

The edge-weighting interaction is a real concern. When PFR's top-level partition assigns census tracts to regions, the subgraph inherits edge weights from the original graph. If weights were calibrated for the full-state graph, they may be poorly calibrated for subgraph partitioning. The NC/GA results suggest the algorithm works well in practice, but the paper should verify that weights are correctly scaled within subgraphs or confirm that raw boundary lengths (naturally subgraph-consistent) are used.

## P1 Items (must fix)
- Add a comparison of PFR vs. direct k-way METIS for states with small k (k ≤ 10) to validate that hierarchical approximation does not significantly degrade partition quality.
- Report subgraph size distribution (mean, std, min, max tract count per subgraph) for the top-level partition across all 50 states.

## P2 Items (should fix)
- Describe the edge-weight inheritance strategy for subgraphs and assess whether this affects partition quality.
- Explicitly describe the parallelism structure and estimate expected speedup vs. serial METIS for large-k states.
- Add a discussion of whether level-by-level edge-cut minimization approximates global edge-cut minimization, with an empirical bound on the approximation ratio.
