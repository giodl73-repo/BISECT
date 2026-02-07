# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, graph partitioning, multilevel algorithms
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper presents a straightforward but effective application of edge-weighted graph partitioning to congressional redistricting, using METIS as the underlying partitioner. The core idea—weighting edges by boundary lengths to optimize perimeter—is conceptually simple yet yields impressive empirical results: 56% compactness improvement over unweighted baseline and 20% over enacted districts across all 50 U.S. states.

From a graph partitioning perspective, this is a textbook application of weighted partitioning. The novelty lies not in the algorithm (METIS has supported edge weights since its inception) but in the problem formulation and empirical validation. The authors correctly identify that standard edge-cut minimization optimizes topology rather than geometry, and show that incorporating geometric information dramatically improves solution quality for this domain.

The paper is well-executed with thorough experimental validation. The national-scale evaluation (435 districts, 50 states) demonstrates practical viability. However, the algorithmic contribution is incremental—this is fundamentally METIS with domain-specific preprocessing. The paper would be strengthened by deeper analysis of METIS's behavior with geometric edge weights and exploration of alternative partitioning strategies.

## Score

**Score**: 3/4 — **Accept**

## Major Issues (Blocking)

### M1: Missing Partitioning Quality Analysis

The paper reports compactness improvements but doesn't analyze METIS's actual partitioning quality. Key questions:
- What are the edge-cut values (weighted vs unweighted)?
- How do multilevel coarsening ratios change with geometric weights?
- Does the Kernighan-Lin refinement converge differently?
- Are there states where METIS struggles due to geometric weight distribution?

Without this analysis, we don't understand *why* edge weighting works beyond the intuitive "minimize perimeter" explanation. Include edge-cut statistics, convergence metrics, and failure case analysis.

### M2: Incomplete Comparison to Alternative Partitioners

The paper only evaluates METIS. Modern partitioners (KaHIP, Scotch, Zoltan) have different optimization strategies and may behave differently with geometric weights. At minimum:
- Compare to one alternative partitioner (KaHIP recommended—supports edge weights, open source)
- Justify why METIS is the appropriate choice
- Discuss whether results generalize to other multilevel partitioners

The claim that edge weighting improves redistricting is only validated for METIS. This limits the contribution's generality.

### M3: Integer Weight Precision Analysis Missing

Edge weights are scaled to integer centimeters (100x meters). The paper states this is "adequate" but provides no evidence. Critical analysis needed:
- Sensitivity analysis: What happens with 1m, 10cm, 1cm, 1mm precision?
- Does precision affect METIS convergence or solution quality?
- Are there numerical stability issues at boundaries?
- Block-level data claim (1M+ units) requires justification—will centimeter precision suffice?

This is important for practitioners adopting the method.

## Minor Issues

### m1: METIS Parameter Selection Unjustified

Why `-niter=100`? Why `-ufactor=1.005` (0.5%)? These parameters significantly affect results but are presented without justification or sensitivity analysis. Include ablation study or cite tuning methodology.

### m2: Bridge Edge Weight Heuristic Not Validated

County-based median boundary length for water crossings is heuristic. Why median? Why county-level? Compare to alternatives:
- Mean vs median vs percentile choices
- State-level vs county-level scaling
- Shortest path distances
- Population-weighted centroids

Show that median/county is best or at least reasonable.

### m3: Recursive Bisection vs K-way Partitioning

METIS supports direct k-way partitioning (`gpmetis` vs `kmetis`). Why recursive bisection? Does k-way produce different (better/worse) compactness? Recursive bisection has $O(\log K)$ levels but may produce suboptimal global cuts. Justify this choice.

### m4: Subgraph Extraction Implementation

Section 3.3 describes index remapping for subgraph extraction but doesn't discuss implementation complexity. For large states (California: 9,213 tracts), this happens $O(\log K)$ times. What's the overhead? Can this be optimized with better data structures?

### m5: Memory Consumption Underreported

"Full state graphs: 1-5 MB" seems low. METIS uses multiple graph copies during coarsening. What's peak memory consumption during partitioning? Large states may hit memory limits on modest hardware.

## Strengths

1. **Strong empirical validation**: National-scale evaluation (50 states, 435 districts) with multiple baselines (unweighted, enacted) demonstrates practical effectiveness.

2. **Clear problem formulation**: The topology-vs-geometry mismatch is well-articulated with concrete examples (150m vs 12.5km boundary).

3. **Comprehensive methodology**: Graph construction pipeline (adjacency detection, boundary computation, bridge connections) is thorough and addresses real-world complexity.

4. **Reproducibility**: Open-source implementation promise and detailed METIS configuration enable replication.

5. **Domain impact**: 56% improvement and superiority over enacted districts in 37/50 states is impressive. The gerrymandering analysis (Illinois +174%) is compelling.

6. **Honest limitations**: Discussion acknowledges single-objective optimization, integer precision constraints, and metric specificity.

## Questions for Authors

1. **METIS behavior**: How do coarsening ratios, refinement iterations, and convergence behavior differ between weighted and unweighted modes? Provide statistics from actual runs.

2. **Theoretical bounds**: Can you establish approximation guarantees? What's the worst-case compactness ratio relative to optimal? Does recursive bisection have inherent limitations compared to optimal partitioning?

3. **Sensitivity to graph structure**: Do results depend on adjacency graph properties (degree distribution, clustering coefficient)? Are there states where geometric weighting fails?

4. **Alternative objectives**: Have you experimented with other edge weight formulations? E.g., boundary length squared (penalizes long boundaries more), inverse boundary length (encourages long cuts), or combinations with topological edge count?

5. **Indiana outlier**: Indiana's enacted plan (0.478) exceeds algorithmic result (0.353) by 35%. What does their commission do differently? Can we reverse-engineer their approach to improve the algorithm?

6. **Scalability**: You mention block-level data (1M+ units nationally). Will METIS scale? Have you tested on block-level graphs for pilot states?

## Recommendations

- **Add partitioning quality metrics**: Report edge-cut values, coarsening statistics, refinement convergence for representative states.

- **Compare to alternative partitioner**: Run KaHIP or Scotch on 3-5 states to validate that edge weighting generalizes beyond METIS.

- **Precision sensitivity analysis**: Test 1m, 10cm, 1cm, 1mm scaling; show results are stable or identify sweet spot.

- **Ablation study**: Vary METIS parameters (`-niter`, `-ufactor`, `-minconn`) to show robustness.

- **Theoretical analysis**: Provide approximation bounds or lower bounds on achievable compactness. Even informal worst-case examples would strengthen the paper.

- **Block-level pilot**: Test on 2-3 states with block-level data to validate scalability claims.

- **Code release**: Ensure METIS integration code, graph construction pipeline, and edge weight computation are well-documented and runnable.

---

**Verdict**: Accept with Minor Revisions

**Confidence**: High — I designed METIS and understand its behavior intimately. This is a solid application paper with good empirical results. The algorithmic contribution is incremental (standard weighted partitioning), but the problem formulation, domain-specific engineering (bridge connections, boundary computation), and national-scale validation justify publication. Addressing the major issues (partitioning quality analysis, alternative partitioners, precision justification) would elevate this from a good application paper to an excellent one.
