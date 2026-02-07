# Round 2 Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Cynthia A. Phillips (Sandia National Labs)
**Expertise**: Combinatorial optimization, graph algorithms
**Round**: 2
**Date**: 2026-02-07

---

## Overall Assessment

The authors have made substantial improvements addressing most algorithmic concerns from Round 1. The additions of partitioning quality analysis (P1.3), alternative partitioner comparison (P1.4), and recursive bisection justification (P1.5) significantly strengthen the algorithmic contribution. The paper now provides deeper insight into solution properties and algorithm behavior.

From a combinatorial optimization perspective, the paper has evolved from pure empirical study to include meaningful algorithmic analysis. The partitioning quality analysis explains *why* edge weighting works (topological vs geometric optimization tradeoff). The recursive bisection comparison demonstrates proper algorithm selection based on robustness vs marginal quality gains. The alternative partitioner validation shows the technique generalizes beyond METIS.

However, the paper still lacks theoretical analysis—no approximation bounds, complexity analysis, or optimality guarantees. Indiana's enacted plan (0.478 vs algorithmic 0.353, +35% gap) suggests significant room for improvement, but we don't know how close algorithmic solutions are to optimal.

For applications venues (KDD, AAAI), the current empirical depth with algorithmic insights is acceptable. For optimization venues (SODA, IPCO), approximation analysis would be required.

## Updated Score

**Score**: 4/4 — **Accept** (for applications venues: KDD, AAAI, JEA)
**Score**: 3/4 — **Accept with Minor Revisions** (for optimization venues: SODA, IPCO)

*Upgrade from 3/4 in Round 1 for applications venues*

## P1 Items Addressed (My Areas)

### P1.3: Partitioning Quality Analysis ✓ EXCELLENT

Section 3.2 (Partitioning Quality Analysis) provides comprehensive analysis:

**Metrics**:
- Edge cuts: 2,156 (unweighted) → 3,805 (weighted), +77%
- Perimeter: 3,627 km (unweighted) → 2,704 km (weighted), -25%
- Average edge length: 397m (unweighted) → 135m (weighted), -66%

**Key insight**: Edge-cut minimization with geometric weights favors many short cuts over few long cuts. This explains why traditional partitioning metrics (edge cuts) don't predict geometric quality.

**Optimization interpretation**: The algorithm effectively optimizes:
$$\min \sum_{(i,j) \in \text{cut}} w_{ij}$$

where $w_{ij}$ = boundary length. This is weighted edge cut, which correlates with perimeter but isn't identical (a district's perimeter includes only external boundaries, not internal cut edges within the district's subgraph).

**Excellent analysis**: This demonstrates understanding of optimization objective and how METIS's heuristics behave with domain-specific weights.

### P1.4: Alternative Partitioner Comparison ✓ GOOD

Section 3.5 compares METIS, KaHIP, Scotch on 5 states:
- All within 0.3% average compactness (max deviation 1.86%)
- Validates edge weighting generalizes to multilevel partitioners

**Optimization interpretation**: All three use similar multilevel frameworks (coarsening + refinement), so similar solution quality is expected. The small variations (0.3%) likely reflect different matching strategies, refinement heuristics, or tie-breaking rules.

**What this validates**: The technique is not METIS-specific but applicable to any multilevel partitioner with edge weight support.

**Minor suggestion**: Brief mention of integer programming comparison (see M1 below) would strengthen positioning, but current comparison is adequate.

### P1.5: Recursive Bisection Justification ✓ STRONG

Section 2.3 compares recursive to k-way:
- K-way: 1.7% better compactness, 20% contiguity failure rate
- Recursive: 100% contiguity guarantee, 1.7x faster

**Optimization interpretation**: K-way's 20% contiguity failure rate is striking. This suggests:
1. METIS's balance constraint doesn't enforce contiguity globally
2. Kernighan-Lin refinement can disconnect components through local vertex swaps
3. Greedy optimization can violate global constraints

**Why recursive succeeds**: By partitioning at each level and operating only on connected subgraphs, recursive bisection maintains contiguity invariant. This is proper algorithm design—guaranteeing hard constraints at expense of marginal objective improvement.

**Trade-off analysis**: 1.7% quality loss for 100% vs 80% success rate is correct choice for redistricting where contiguity is legal requirement.

This is excellent optimization analysis demonstrating proper constraint handling vs objective optimization tradeoff.

## Major Theoretical Gaps (Venue-Dependent)

### M1: Approximation Bounds Still Missing (Not Blocking for KDD/AAAI)

The paper still provides no approximation analysis or optimality bounds:
- **Approximation ratio**: How far from optimal are solutions?
- **Lower bounds**: What's minimum achievable perimeter for each state?
- **Recursive bisection gap**: How much is lost vs optimal k-way partitioning?

**Evidence of gap**: Indiana (0.478 vs 0.353, +35%) suggests significant suboptimality, but we don't know if:
1. Optimal is 0.48+ (Indiana is near-optimal, METIS is far)
2. Optimal is ~0.37 (METIS is near-optimal, Indiana is exceptional)
3. Optimal is somewhere between

**For applications venues**: Not required. Empirical validation suffices.

**For optimization venues**: Would need at least:
- MILP solutions for small states (Delaware, Vermont, Wyoming: 1-2 districts)
- Informal lower bounds (geometric packing arguments)
- Discussion of problem complexity (NP-hardness or polynomial cases)

### M2: Problem Formulation (Acceptable but Could Be Improved)

The paper informally describes the optimization problem but doesn't provide formal statement.

**Current**: "Minimize total perimeter subject to population balance and contiguity"

**Standard optimization formulation**:
- Variables: $x_{it} \in \{0,1\}$ (tract $i$ assigned to district $t$)
- Objective: $\min \sum_{t=1}^K \sum_{(i,j) \in E} w_{ij} \cdot \mathbb{1}[x_{it} \neq x_{jt}]$
- Constraints:
  - Balance: $|\sum_i p_i x_{it} - \bar{p}| \leq \epsilon$ for all $t$
  - Contiguity: Districts form connected subgraphs
  - Assignment: $\sum_t x_{it} = 1$ for all $i$

**For applications venues**: Current informal statement is acceptable.

**For optimization venues**: Formal statement would be expected.

### M3: Complexity Analysis Missing (Not Critical)

The paper doesn't state whether the perimeter minimization problem is NP-hard or discuss polynomial special cases.

**Likely NP-hard** (by analogy to graph partitioning), but should cite proof or provide reduction.

**Not blocking**: This is deep theoretical analysis beyond applications venues.

## Minor Optimization Issues

### m1: Population Tolerance Sensitivity (Still Not Analyzed)

`-ufactor=1.005` (0.5% imbalance) is constraint relaxation. The paper doesn't show compactness-balance tradeoff:
- Pareto frontier at different tolerances
- Shadow prices (compactness improvement per % relaxation)

**Acceptable**: Standard redistricting practice is 0.5% for congressional districts. Sensitivity analysis would strengthen but isn't critical.

### m2: Edge Weight Quantization (Still Not Analyzed)

Integer centimeter scaling may introduce quantization error. No sensitivity analysis on precision (1m, 10cm, 1cm, 1mm).

**Acceptable**: Results suggest centimeter precision is adequate. Future work could validate.

### m3: Bridge Weight Optimization (Still Heuristic)

Median boundary length for bridges is heuristic, not optimized. Could bridge weights be learned?

**Acceptable**: Pragmatic heuristic with good results. Optimization would be marginal improvement.

### m4: Multi-Start Strategy (Not Discussed)

Standard optimization practice: run multiple times with different seeds, keep best.

**Does the paper do this?** Not stated. If not, why not? Multi-start is embarrassingly parallel and could improve results.

**Recommendation**: Clarify whether single run or multi-start. If single, justify. If multi-start, report variability.

### m5: Local Optima Analysis (Not Discussed)

METIS's Kernighan-Lin refinement can get stuck in local optima. Solution variability across random seeds not reported.

**Recommendation**: Run 10x with different seeds, report best/worst/mean to show stability.

## Observations on Other P1 Items

While not my primary expertise:

- **P1.1 (Partisan Analysis)**: Shows compactness ≠ fairness—important for redistricting
- **P1.2 (VRA Compliance)**: 68% reduction demonstrates representation tradeoff
- **P1.6 (Neutrality)**: Language appropriately softened

These strengthen the paper's honesty about limitations.

## P2 Items Completed

- **P2.4 (County Preservation)**: Shows modest tradeoff
- **P2.5 (Geographic Sorting)**: Quantifies geographic vs intentional effects

Both outside my expertise but appear thorough.

## Remaining P2 Items (My Area)

- **P2.1 (Approximation Analysis)**: My primary concern—see M1 above. Not blocking for applications venues, needed for optimization venues.

Other P2 items (multi-objective, MCMC, Indiana, census tracts, hypergraph) are for other reviewers.

## Strengths (Updated)

In addition to Round 1 strengths:

1. **Partitioning quality insight**: Topological vs geometric tradeoff explains algorithm behavior

2. **Algorithmic robustness analysis**: K-way contiguity failure vs recursive success demonstrates proper constraint handling

3. **Generalization validation**: Alternative partitioners show technique is not METIS-specific

4. **Honest evaluation**: Partisan/VRA analysis shows single-objective limitations

5. **Proper trade-off analysis**: 1.7% quality loss for 100% contiguity guarantee demonstrates engineering judgment

## For Optimization Venues (If Targeting)

**Required additions for SODA/IPCO**:
1. Formal problem statement with variables, objective, constraints
2. Complexity analysis (NP-hardness or polynomial cases)
3. Approximation bounds or lower bounds (even informal)
4. MILP comparison for small states to estimate optimality gap

**Current gap**: Paper is empirical study with algorithmic insights, not theoretical contribution.

**Recommendation**: Target applications venues (KDD, AAAI, JEA) where current depth is excellent.

## Final Recommendations

**For applications venues (KDD, AAAI, JEA)**: Paper is ready for publication.

**Optional enhancements** (not blocking):
- Multi-start experiments (10x runs with different seeds)
- Local optima analysis (solution variability)
- Sensitivity analysis (population tolerance, edge weight precision)
- Clarify whether single run or multi-start in methodology

**For optimization venues (SODA, IPCO)**: Add approximation analysis, formal problem statement, complexity discussion.

---

## Verdict

**Accept** — Ready for publication at applications venues

**Changes from Round 1**: Upgraded from "Accept with Minor Revisions" (3/4) to "Accept" (4/4) for applications venues

**Rationale**: The paper now provides:
1. **Algorithmic insight**: Partitioning quality analysis explains why edge weighting works
2. **Robustness analysis**: Recursive vs k-way comparison demonstrates proper constraint handling
3. **Generalization**: Alternative partitioners validate technique applicability
4. **Honest evaluation**: Partisan/VRA analysis shows limitations

For **applications venues** (KDD, AAAI, JEA), this represents excellent work:
- Strong empirical validation (50 states, 435 districts)
- Meaningful algorithmic insights (not just black-box optimization)
- Proper engineering trade-offs (robustness vs marginal quality)
- Comprehensive evaluation (compactness, partisan effects, VRA, counties)

The remaining theoretical gaps (approximation bounds, formal problem statement) are not expected for applications venues. The paper makes valuable contributions:
- **For optimization community**: Demonstrates domain-specific edge weights dramatically improve solution quality
- **For redistricting community**: Provides scalable, practical method with honest assessment of limitations

**Venue recommendation**: Target KDD, AAAI, or Journal of Experimental Algorithmics where empirical algorithmic contributions with practical impact are valued. The current depth is excellent for these venues.

**Confidence**: High — I have extensive experience with combinatorial optimization and graph algorithms. This paper has evolved from pure empirical study (Round 1) to include meaningful algorithmic analysis (Round 2). The quality is now appropriate for top-tier applications venues. For optimization theory venues, additional theoretical work would be needed, but this is venue-dependent, not a fundamental flaw.
