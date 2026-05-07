# Review: Quantifying the VRA-Compactness Tradeoff

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, Graph Partitioning, Multilevel Algorithms
**Date**: 2026-02-08
**Round**: 1

---

## Overall Assessment

This paper presents an innovative application of edge-weighted graph partitioning to congressional redistricting, demonstrating that demographic-aware optimization can simultaneously improve Voting Rights Act compliance and district compactness. The use of METIS with minority-population-weighted edges represents a clever adaptation of standard graph partitioning to incorporate domain-specific constraints without the rigidity of multi-constraint optimization.

From a graph partitioning perspective, the paper makes solid algorithmic contributions: edge-weighting dominates multi-constraint approaches (Alabama: 2 MM districts vs 0 with better compactness), and the systematic parameter sweep (5 weight factors × 4 thresholds) provides practical guidance for setting edge weight parameters. The finding that edge-weighted optimization can reduce unweighted edge cut while satisfying demographic objectives (Alabama: -9.3% edge cut) is particularly notable, as it suggests that standard METIS is indeed finding suboptimal solutions when demographic clustering is not leveraged.

However, the paper would benefit from more rigorous treatment of METIS implementation details, computational complexity analysis, validation of graph partitioning quality metrics beyond edge cut, and comparison to alternative partitioning algorithms. Several technical questions about how edge weights interact with METIS's multilevel coarsening and refinement phases remain unanswered, limiting replicability and understanding of why the approach succeeds.

**Score**: 3.25/4 (Accept with Revisions)

---

## Major Strengths

### 1. Novel Application of Edge-Weighting to Demographic Objectives
The edge-weighting strategy (Section 3.3.3) elegantly encodes VRA compliance as a graph partitioning objective without imposing rigid constraints. By making minority-minority edges expensive to cut (5×-100× normal weight), METIS naturally preserves demographic clusters while maintaining flexibility in population balancing. This is algorithmically cleaner than multi-constraint optimization, which forces explicit per-partition demographic targets that may be geometrically infeasible.

### 2. Edge-Weighted Optimization Finds Better Solutions Than Unweighted
The Alabama result (edge cut: 280 → 254, -9.3%) while achieving 2 MM districts demonstrates that edge-weighting accesses a superior region of the solution space. This suggests that standard METIS's unweighted optimization is locally optimal for raw edge cut but globally suboptimal when natural demographic structure is considered. This is a genuine algorithmic insight with implications beyond redistricting.

### 3. Systematic Parameter Exploration
Testing 5 weight factors (1×, 5×, 10×, 50×, 100×) and 4 thresholds (40%-55%) across 5 states provides empirical guidance for practitioners. The convergence on 5× as optimal for 3 states (Alabama, Georgia, Louisiana) suggests this weight factor balances VRA enforcement with compactness preservation, while the threshold variation (40%-55%) shows that optimal thresholds are state-specific, depending on minority population distribution.

### 4. Computational Efficiency Demonstrated
Completing 105 configurations in ~2 hours (Section 3.7) demonstrates METIS's scalability advantage over optimization approaches like integer programming or simulated annealing. This efficiency enables the comprehensive parameter sweeps that produce your Pareto frontier analysis, which would be computationally prohibitive with slower algorithms.

---

## Major Issues (Must Address)

### P1: Incomplete Description of Edge-Weight Integration with METIS

**Issue**: Section 3.3.3 describes the edge weight function conceptually (minority-minority edges weighted w, others weighted 1) but provides insufficient detail on how these weights are integrated into METIS's multilevel algorithm. METIS has three phases—coarsening, initial partitioning, and refinement—and edge weights can be incorporated differently in each:

1. **Coarsening phase**: Are edge weights used in the matching algorithm that collapses nodes? Heavy-edge matching (HEM) would preferentially collapse minority-minority edges, creating coarse-graph supervertices that preserve demographic clusters. Light-edge matching would preferentially collapse low-weight edges, fragmenting minority clusters early. Which matching scheme do you use?

2. **Initial partitioning**: Does the initial bisection at the coarsest level consider edge weights? If initial partitioning is unweighted, the demographic structure may be broken early and only partially recovered during refinement.

3. **Refinement phase**: Kernighan-Lin (KL) or Fiduccia-Mattheyses (FM) refinement considers edge weights naturally when computing cut costs. However, refinement is local search—it can improve an initial partition but may not escape local optima if coarsening or initial partitioning ignored demographics.

Without clarification, readers cannot assess whether your approach leverages edge weights throughout the multilevel hierarchy (optimal) or only during refinement (potentially suboptimal). The Alabama -9.3% edge cut improvement suggests full multilevel integration, but this needs to be stated explicitly.

**Recommendation**:
1. **Add technical details to Section 3.3.3**: Specify which METIS matching algorithm you use (HEM, LEM, RM, SHEM). If using HEM with weighted edges, state this clearly: "We employ heavy-edge matching during coarsening, which preferentially collapses minority-minority edges (high weight), preserving demographic structure through the multilevel hierarchy."

2. **Verify METIS configuration**: Confirm that your METIS build supports weighted graphs in coarsening. Some METIS configurations only apply edge weights during refinement. If you're using command-line METIS via graph files, specify the file format (edge list with weights) and METIS flags used.

3. **Ablation study**: Run experiments with edge weights applied only during refinement (by providing unweighted graph for coarsening, weighted graph for refinement) vs. full multilevel integration. If performance differs significantly, this validates the importance of multilevel edge-weight integration.

4. **Pseudocode or repository**: Provide pseudocode showing edge-weight computation and METIS integration, or link to a GitHub repository with your implementation. This is essential for replication.

**Severity**: P1 (Blocking) - This is a fundamental technical question about your method. Without clarification, graph partitioning experts cannot assess whether your approach is implemented optimally or whether further improvements are possible.

---

### P2: Missing Computational Complexity and Scalability Analysis

**Issue**: Section 3.7 reports execution times (single configuration: 30-90 seconds, full sweep: 2 hours total) but provides no formal complexity analysis or scaling experiments. From a graph partitioning perspective, several complexity questions are unaddressed:

1. **Time complexity**: METIS recursive bisection has O((|E| + |V|) log k) time complexity for partitioning a graph with |V| vertices, |E| edges into k parts. Do edge weights increase complexity? (No, if edge weights are simply multiplicative factors in cut cost computation.) Confirming this would validate your efficiency claims.

2. **Space complexity**: Storing edge weights requires O(|E|) additional space. For your largest state (Georgia: 2,796 tracts, ~17,000 edges assuming degree 6), this is trivial (~136 KB for float32 weights). Confirming negligible overhead would strengthen efficiency arguments.

3. **Scaling with graph size**: Your states range from 1,323 tracts (SC) to 2,796 tracts (GA), a 2.1× size ratio. Do execution times scale linearly (suggesting O(n) behavior), superlinearly (O(n log n)), or worse? Plot execution time vs. |V| across your 5 states to empirically validate complexity.

4. **Scaling with weight factor**: Do extreme weight factors (100×, 1000×) increase runtime? Theoretically, no—edge weights are multiplicative constants in cut cost. However, high weights may affect METIS's convergence in refinement (more local optima, more iterations to converge). Testing this would be informative.

5. **Scaling with refinement iterations**: You set niter=100 (Section 3.3.2). How does performance (edge cut, MM count) scale with niter ∈ {10, 50, 100, 500, 1000}? Diminishing returns would suggest niter=100 is sufficient; continued improvement would suggest you're underfitting.

Without this analysis, readers cannot assess whether your approach scales to larger problems (e.g., block-level redistricting with 50,000 units, 300,000 edges) or whether it's specific to tract-level graphs.

**Recommendation**:
1. **Add complexity analysis**: In Section 3.7 (Computational Infrastructure) or new subsection 3.8 (Scalability), provide formal complexity analysis confirming that edge-weighting does not increase asymptotic complexity beyond standard METIS.

2. **Scaling experiments**: Plot execution time vs. |V| (number of tracts) across your 5 states. If linear or log-linear, this validates scalability to larger problems. If superlinear, discuss implications for block-level redistricting.

3. **Parameter scaling**: Test how execution time varies with weight factors (1×, 5×, 10×, 50×, 100×, 1000×) for a fixed state (Alabama recommended). If time is constant, this confirms weights are computational no-ops. If time increases, explain why (e.g., refinement convergence).

4. **Refinement scaling**: Plot compactness and MM count vs. niter for representative configurations (Alabama 5×@45%, Georgia 5×@55%). Identify elbow points where diminishing returns begin. If niter=100 is beyond the elbow, you're overfitting; if before, you're underfitting.

**Severity**: P2 (Important) - Scalability is critical for practical adoption. Without this analysis, practitioners cannot assess whether your approach generalizes to larger problems (block-level redistricting, larger states).

---

### P3: Partition Quality Metrics Beyond Edge Cut

**Issue**: You evaluate compactness using four metrics (edge cut, Polsby-Popper, Reock, convex hull) but only edge cut is directly optimized by METIS. The other three are geometric post-hoc measurements. From a graph partitioning perspective, additional quality metrics would strengthen your evaluation:

1. **Balance constraint satisfaction**: You set ufactor=1.005 (±0.5% population tolerance). Do all 105 configurations satisfy this constraint? Report min/max population imbalance across all partitions. If some configurations violate balance, this indicates feasibility issues.

2. **Communication volume**: In parallel computing, communication volume (sum of edge weights crossing partitions, weighted by partition sizes) measures load balancing quality. For redistricting, this translates to "how much minority population is fragmented across district boundaries?" Computing weighted edge cut (summing weights of cut edges) provides this metric, but you don't report it consistently.

3. **Partition size variation**: Do all districts have roughly equal population (coefficient of variation < 0.01) or are some systematically larger/smaller? Standard deviation of district populations would quantify balance quality.

4. **Internal connectivity**: Are districts well-connected (low diameter, high internal edge density) or are some districts barely contiguous (long "tendrils" connecting distant clusters)? Computing average diameter or average shortest path within districts would quantify internal compactness.

5. **Separator size**: Recursive bisection creates a hierarchy of bisections. At each level, the edge separator (edges cut) can be small (good) or large (bad). Reporting average separator size across bisection levels would diagnose whether your edge-weighting improves cuts at all hierarchy levels or only at final partition.

Without these metrics, your evaluation is limited to end-state geometry (Polsby-Popper) without assessing the graph-theoretic quality of the partition.

**Recommendation**:
1. **Add balance metrics**: Report min/max/stddev population imbalance for all 105 configurations in supplementary table. Confirm all satisfy ufactor=1.005 constraint. If violations occur, discuss implications.

2. **Report weighted edge cut**: For edge-weighted configurations, report both unweighted edge cut (traditional compactness) and weighted edge cut (METIS's optimization objective). Plot weighted vs. unweighted to show tradeoff. If weighted cut decreases while unweighted increases, this demonstrates explicit compactness sacrifice; if both decrease (Alabama), this shows win-win.

3. **District connectivity analysis**: For best configurations (Alabama 5×@45%, Georgia 5×@55%), compute average district diameter (longest shortest path within district) and internal edge density (edges within district / possible edges). Compare to baseline to show whether edge-weighting improves internal connectivity.

4. **Separator size analysis**: For recursive bisection hierarchy, report average separator size (edges cut) at each level (log₂ k levels, where k is number of districts). If edge-weighting reduces separators at all levels, this validates multilevel effectiveness.

**Severity**: P2 (Important) - Comprehensive partition quality metrics are standard in graph partitioning evaluation. Without these, your paper reads as application-focused (redistricting) rather than algorithmically rigorous.

---

## Minor Issues

### m1: Comparison to Alternative Partitioning Algorithms Missing
You use METIS recursive bisection exclusively but don't compare to alternative algorithms: k-way partitioning (METIS direct k-way, which partitions into k parts directly rather than recursive bisection), spectral partitioning, or multilevel k-way refinement. METIS's direct k-way often produces better cuts than recursive bisection for k > 4 because it avoids the constraint that bisection imposes (each bisection must be balanced, limiting flexibility). Testing Alabama with direct k-way (k=7) vs. recursive bisection (log₂ 7 ≈ 3 levels) could reveal whether your wins are specific to recursive bisection or general to METIS.

### m2: Random Seed and Determinism Not Fully Explored
Section 3.6.3 mentions "fixed random seeds" for reproducibility but doesn't report which seeds you use or how sensitive results are to seed choice. METIS's stochastic elements (initial partition randomization at coarsest level, random tie-breaking in refinement) can affect final partitions. Testing 10 different seeds per configuration (as you mention but don't execute) would quantify variation. If CV < 5% as claimed, report this explicitly with data.

### m3: Population Tolerance (ufactor) Trade-off Not Analyzed
You set ufactor=1.005 (±0.5% population) uniformly across all configurations. Looser tolerances (ufactor=1.01, ±1%) might enable better compactness or higher MM counts by allowing METIS more flexibility in balancing population vs. edge cut. Conversely, tighter tolerances (ufactor=1.001, ±0.1%) might enforce stricter equality at compactness cost. Testing ufactor ∈ {1.001, 1.005, 1.01, 1.02} for representative configurations would reveal whether your results are sensitive to this parameter.

### m4: Contiguity Enforcement Details Unclear
Section 3.3.1 mentions "water-based bridge connections for disconnected islands" but doesn't specify the algorithm. Are bridges added between nearest tracts across water (<1km)? Are bridge weights adjusted (e.g., higher than normal adjacencies to discourage cuts)? Contiguity is critical for redistricting legality, and the bridge mechanism affects graph structure. Clarify this in Methodology or provide a reference to your bridge construction algorithm.

### m5: Comparison to Integer Programming or Other Exact Methods
Your approach is heuristic (METIS is not guaranteed optimal). Exact methods like integer programming could provide optimality guarantees, at least for small instances. For Mississippi (4 districts, 1,323 tracts, smallest state), solving the exact edge-weighted partitioning problem via IP would establish whether METIS finds optimal or near-optimal solutions. If METIS is within 1-2% of optimal, this validates heuristic quality. If 10-20% suboptimal, this suggests room for improvement.

---

## Recommendations for Revision

### High Priority (Must Address)
1. **Clarify edge-weight integration with METIS multilevel algorithm** (P1) - Specify coarsening, initial partitioning, refinement treatment.
2. **Add computational complexity and scalability analysis** (P2) - Formal complexity analysis, scaling experiments (time vs. |V|, niter, weight factors).
3. **Expand partition quality metrics** (P3) - Report balance metrics, weighted edge cut, internal connectivity, separator sizes.

### Medium Priority (Strengthen Paper)
4. **Compare to direct k-way partitioning** (m1) - Test whether recursive bisection is optimal or if k-way improves results.
5. **Execute robustness analysis with multiple seeds** (m2) - Quantify variation across 10 seeds per configuration.
6. **Test ufactor sensitivity** (m3) - Evaluate whether population tolerance affects results.

### Low Priority (Polish)
7. **Clarify contiguity bridge construction** (m4) - Specify algorithm for water-based bridges.
8. **Compare to exact methods for small instances** (m5) - Validate METIS heuristic quality via IP on Mississippi (smallest state).

---

## Detailed Comments by Section

### Introduction (Section 1)
- **Strength**: Clear motivation—VRA-compactness tradeoff has practical importance for redistricting law and policy.
- **Suggestion**: Add one sentence noting that edge-weighting is a general graph partitioning technique applicable beyond redistricting (load balancing, circuit partitioning, social network community detection with demographic objectives).

### Background (Section 2)
- **Strength**: Comprehensive review of compactness metrics (2.1), VRA law (2.2), multi-objective optimization (2.3), graph partitioning (2.4).
- **Weakness**: Section 2.4 (Graph Partitioning) is brief (1.5 pages) and doesn't discuss METIS's multilevel architecture in detail. Readers unfamiliar with multilevel methods may not appreciate why edge-weighting is non-trivial.
- **Suggestion**: Expand Section 2.4 to ~2 pages, adding subsections on:
  - Multilevel coarsening (matching algorithms: HEM, LEM, SHEM)
  - Initial partitioning (recursive bisection vs. k-way)
  - Refinement (KL/FM algorithms, move-based vs. swap-based)
  - Edge weights in multilevel context (when/how they affect each phase)

### Methodology (Section 3)
- **Strength**: Experimental design is systematic—105 configurations, 5 states, 4 compactness metrics, 3 VRA metrics.
- **Weakness**: Edge-weighting description (3.3.3) is conceptually clear but technically incomplete (P1). Add pseudocode or algorithmic details.
- **Weakness**: No complexity analysis (P2). Add Section 3.8 on scalability.
- **Weakness**: Partition quality metrics (3.1-3.2) focus on edge cut and geometric measures but omit graph-theoretic metrics like balance, diameter, separator size (P3).
- **Suggestion**: Add pseudocode for edge weight computation:
  ```
  Algorithm: Edge Weight Computation
  Input: Tract adjacency graph G=(V,E), minority percentages M, threshold θ, weight factor w
  Output: Weighted adjacency graph G'=(V,E,W)

  For each edge (u,v) ∈ E:
      if M[u] >= θ and M[v] >= θ:
          W[(u,v)] = w  // Minority-minority edge
      else:
          W[(u,v)] = 1  // Other edge
  Return G'
  ```

### Results (Section 4)
- **Strength**: Comprehensive results across 5 states with clear patterns (win-win, MM sac/non-MM gain, both sac, infeasible).
- **Strength**: Alabama -9.3% edge cut improvement (Section 4.3) is compelling evidence that edge-weighting accesses better solution space.
- **Weakness**: No weighted edge cut reported. For edge-weighted configs, readers want to see both unweighted edge cut (traditional compactness) and weighted edge cut (METIS's objective). Add column to Tables 1, 3, 4.
- **Weakness**: No balance metrics reported. Are all partitions within ufactor=1.005 tolerance? Report min/max population imbalance.
- **Suggestion**: Add Table 5 (supplementary) with comprehensive partition quality metrics for best configurations: unweighted edge cut, weighted edge cut, max population imbalance, avg district diameter, internal edge density.

### Discussion (Section 5)
- **Strength**: Three mechanisms (5.1) provide causal explanations grounded in graph partitioning theory. Mechanism 3 (baseline suboptimality) is particularly compelling from an algorithmic perspective—edge-weighting guides METIS to a different region of solution space that's better for both objectives.
- **Weakness**: Doesn't discuss algorithmic alternatives (direct k-way, spectral, other multilevel schemes). Adding subsection "Why Recursive Bisection with Edge-Weighting?" would justify your algorithmic choices.
- **Suggestion**: In Section 5.6 (Implications for Algorithmic Redistricting), add discussion of computational tradeoffs: METIS is O(n log k) but heuristic (no optimality guarantee), IP is exact but exponential (infeasible for large n), ensemble methods are comprehensive but expensive (millions of samples). Position edge-weighted METIS as efficient targeted optimization.

### Limitations (Section 6)
- **Strength**: Acknowledges multiple limitations (single minority group, fixed k, tract resolution, population-only, compactness-only).
- **Weakness**: Doesn't mention algorithmic limitations: METIS is heuristic (not optimal), recursive bisection may be suboptimal compared to k-way, no ensemble comparison.
- **Suggestion**: Add Section 6.6 "Algorithmic Limitations" discussing heuristic vs. exact methods, recursive bisection vs. k-way, lack of ensemble comparison.

### Computational Infrastructure (Section 3.7)
- **Strength**: Provides execution times (30-90 sec per config, ~2 hours total).
- **Weakness**: No complexity analysis, no scaling experiments, no hardware utilization metrics (CPU usage, memory).
- **Suggestion**: Expand to full "Computational Analysis" section with:
  - Formal complexity (O(n log k) for METIS recursive bisection)
  - Empirical scaling (time vs. |V| plot across 5 states)
  - Memory usage (peak RAM, graph storage size)
  - Parallelization (METIS uses 4 workers—does edge-weighting benefit from parallelization?)

---

## Technical Validation and Reproducibility

**Strengths**:
- Detailed methodology with parameter values specified (weight factors, thresholds, ufactor, niter).
- Data sources documented (2020 Census, TIGER/Line).
- Multiple metrics (4 compactness, 3 VRA) for triangulation.

**Weaknesses**:
- No code repository or pseudocode for edge-weighting implementation (P1).
- Robustness analysis mentioned but not executed (m2).
- No validation against exact methods or alternative algorithms (m1, m5).

**Recommendations**:
1. **Create public repository**: GitHub repo with Python/C++ code for edge-weighted METIS, including:
   - Edge weight computation (Python)
   - METIS integration (C++ wrapper or graph file generation)
   - Post-processing (metric computation, visualization)
   - Raw results (CSV files with all 105 configurations)

2. **Validation suite**: For Mississippi (smallest state), solve exact edge-weighted partitioning via integer programming. Compare METIS solution to IP optimal to quantify heuristic quality (e.g., "METIS achieves 98.3% of optimal edge cut in 90 seconds vs. 6 hours for IP").

3. **Ablation studies**: Test edge weights in refinement-only vs. full multilevel (P1), recursive bisection vs. k-way (m1), multiple random seeds (m2).

---

## Algorithm Design Assessment

From a graph partitioning perspective, your edge-weighting approach is algorithmically sound and makes several smart design choices:

### Strengths:
1. **Edge-weighting over multi-constraint**: By encoding VRA objectives in edge weights rather than partition constraints, you avoid the rigidity that causes multi-constraint to fail (Alabama: 0 MM districts). This is the right algorithmic choice.

2. **Weight factor sweep**: Testing 1×-100× (log scale) provides empirical guidance for practitioners. The convergence on 5× for three states suggests this balances VRA enforcement with compactness, while 100× is too aggressive (likely causes compactness degradation).

3. **Threshold discretization**: Testing 40%-55% in 5% increments is reasonable. Finer grids (2.5% or 1%) might find marginal improvements but would increase computation 2-4×.

4. **Recursive bisection**: For k=4-14 districts (your state range), recursive bisection is efficient (log₂ k = 2-4 levels). For larger k (e.g., state legislature redistricting with k=100), direct k-way might be preferable.

### Potential Improvements:
1. **Adaptive weight factors**: Rather than uniform weights (all minority-minority edges weighted w), adaptive schemes could weight edges proportional to minority concentration: w(u,v) = w × min(M[u], M[v]) / θ. This would create gradients rather than binary weight classes.

2. **Hierarchical weighting**: Different weight factors at different bisection levels. Early bisections (creating 2 regions) might need higher weights to preserve major demographic clusters, while late bisections (creating 8→16 districts) might need lower weights for fine-tuning.

3. **Dynamic threshold adjustment**: Rather than fixed θ per state, adjust θ per bisection based on regional minority concentration. Urban bisections might use θ=50%, rural bisections θ=40%.

These are extensions for future work, not criticisms of your current approach. Your design is sound and demonstrates clear advantages over alternatives (multi-constraint, unweighted baseline).

---

## Significance and Impact

### For Graph Partitioning Research:
This paper demonstrates that edge-weighting can improve unweighted edge cut (Alabama: -9.3%) while satisfying domain constraints (2 MM districts). This is a counterintuitive finding—typically, adding constraints degrades primary objective. The mechanistic explanation (edge-weighting exploits natural clustering that baseline misses) has implications for other graph partitioning applications where node features exhibit spatial autocorrelation (social networks with demographic attributes, circuit partitioning with component types, etc.).

### For Redistricting Practice:
Your Pareto frontier framework and optimal parameter guidance (5× weight, state-specific thresholds) are immediately actionable for redistricting commissions. The computational efficiency (~2 hours for comprehensive sweep) makes this practical for time-constrained redistricting cycles.

### For METIS Development:
Your findings suggest opportunities for METIS enhancement: incorporating node/edge attributes directly into the multilevel algorithm (rather than requiring users to manually compute edge weights) would broaden applicability. A "demographic-aware mode" or "attribute-based weighting" option in METIS could serve redistricting and other applications with feature-rich graphs.

---

## Final Recommendation

**Accept with Revisions**

The paper makes solid algorithmic contributions (edge-weighting dominates multi-constraint, systematic parameter guidance, Pareto frontier framework) and demonstrates practical utility for redistricting. However, three technical issues require clarification:

1. Edge-weight integration with METIS multilevel algorithm (P1)
2. Computational complexity and scalability analysis (P2)
3. Comprehensive partition quality metrics (P3)

These are substantial but feasible revisions. P1 requires algorithmic clarification (may be simple documentation fix if you already use full multilevel weighting). P2 requires additional experiments (scaling tests, ~1-2 days computation). P3 requires additional metric computation (graph-theoretic measures, ~1 week implementation + analysis).

**Estimated revision time**: 2-3 weeks for comprehensive technical additions.

With these revisions, the paper will be suitable for publication in top-tier venues for algorithms (ALENEX, ACM JEA) or interdisciplinary venues combining algorithms and applications (AAAI, AISTATS, KDD). The combination of algorithmic rigor, systematic empirical evaluation, and practical impact makes this a strong contribution to both graph partitioning and computational social science.
