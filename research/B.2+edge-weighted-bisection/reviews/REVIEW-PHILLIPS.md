> **AI Simulation Disclosure**: This review is an AI-generated simulation. The named researcher is not an actual reviewer of this work. Their name and expertise are used to construct an AI persona that emulates the perspective and priorities they are known for, based on their published work and documented research philosophy. No endorsement, affiliation, or participation by this individual is implied. All reviews are synthetic outputs produced by a large language model (Claude, Anthropic).

---

# Review: Edge-Weighted Recursive Bisection for Compact Congressional Districts

**Reviewer**: Cynthia A. Phillips (Sandia National Labs)
**Expertise**: Combinatorial optimization, graph algorithms
**Round**: 1
**Date**: 2026-02-07

---

## Overall Assessment

This paper applies weighted graph partitioning to congressional redistricting with strong empirical results: 56% compactness improvement over unweighted baseline and 20% over enacted districts nationally. The core idea—using boundary lengths as edge weights to optimize perimeter—is straightforward and well-executed.

From a combinatorial optimization perspective, this is a heuristic application of weighted partitioning with no optimality guarantees. The paper uses METIS as a black box without analyzing solution quality, comparing to alternative algorithms, or establishing bounds. Recursive bisection is known to be suboptimal compared to optimal k-way partitioning, but this gap is not quantified. The paper reports good empirical results but provides no insight into how close these are to optimal or under what conditions the method might fail.

The problem formulation (minimize total perimeter subject to population balance and contiguity) is a natural optimization problem, but the paper doesn't position it within the broader optimization literature. Is this problem NP-hard? Are there special cases with polynomial-time optimal algorithms? What approximation guarantees exist? These fundamental questions are not addressed.

For an applications-focused venue (KDD, AAAI), the empirical results may suffice. For an optimization venue (SODA, IPCO), the lack of theoretical analysis and algorithmic alternatives is problematic. The paper reads more as a computational study than an optimization contribution.

## Score

**Score**: 3/4 — **Accept** (for applied venues); **2/4 — Weak Accept** (for optimization venues)

## Major Issues (Blocking)

### M1: No Optimality Analysis or Bounds

The paper reports compactness values but never establishes:
- **Problem complexity**: Is minimizing perimeter subject to balance/contiguity NP-hard? If so, cite proof or provide reduction. If not, what's the optimal algorithm?
- **Lower bounds**: What's the minimum achievable perimeter for each state? Even loose bounds (LP relaxation, geometric packing arguments) would help.
- **Approximation ratio**: How far from optimal are METIS solutions? Can you prove or experimentally estimate the approximation factor?
- **Recursive bisection gap**: How much optimality is lost vs optimal k-way partitioning?

Without bounds, we can't assess solution quality. Indiana's enacted plan (0.478) suggests algorithmic solutions may be far from optimal. Provide lower bounds or at least discuss optimality.

### M2: Alternative Algorithms Not Compared

The paper only evaluates METIS recursive bisection. Other optimization approaches may produce superior compactness:
- **Integer programming**: Formulate as MILP with boundary minimization objective, solve with Gurobi/CPLEX for small states
- **Constraint programming**: CP solvers (Gecode, OR-Tools) for combinatorial constraints
- **Metaheuristics**: Simulated annealing, tabu search, genetic algorithms with perimeter objective
- **Direct k-way METIS**: Avoid recursive bisection suboptimality
- **Alternative partitioners**: KaHIP, Scotch, Zoltan

At minimum:
- Compare to MILP solutions for small states (Delaware, Vermont, Wyoming, Alaska) to estimate optimality gap
- Compare recursive bisection to direct k-way METIS
- Justify why METIS is the appropriate choice vs other optimization methods

### M3: Problem Formulation Doesn't Match Standard Optimization Framework

The paper formulates redistricting as:
- Minimize: $\sum_{j=1}^K P(D_j)$ (total perimeter)
- Subject to: Population balance ($\pm 0.5\%$), contiguity

This is informal. Standard optimization formulation would be:
- Decision variables: $x_{it} \in \{0,1\}$ (tract $i$ assigned to district $t$)
- Objective: $\min \sum_{t=1}^K \sum_{(i,j) \in E} w_{ij} \cdot \mathbb{1}[x_{it} \neq x_{jt}]$
- Constraints: Balance ($|\sum_i p_i x_{it} - \bar{p}| \leq \epsilon$ for all $t$), contiguity (graph connectivity), assignment ($\sum_t x_{it} = 1$ for all $i$)

Formalizing this way would:
- Enable MILP implementation for small instances
- Allow standard optimization techniques (cutting planes, column generation, Lagrangian relaxation)
- Connect to broader optimization literature

Include formal problem statement and discuss why MILP is intractable at scale.

## Minor Issues

### m1: No Sensitivity Analysis on Population Tolerance

`-ufactor=1.005` allows 0.5% population imbalance. This is a constraint relaxation that may significantly affect objective value. Standard optimization analysis would include:
- Pareto frontier: Plot compactness vs population tolerance
- Shadow prices: How much does compactness improve per 0.1% relaxation?
- Tightness analysis: Are optimal solutions on the boundary (exactly 0.5% imbalance) or interior?

Show the compactness-balance tradeoff quantitatively.

### m2: Edge Weight Scaling Arbitrary

Boundary lengths are scaled to integer centimeters: $w_{ij}^{\text{METIS}} = \max(1, \lfloor w_{ij} \times 100 \rfloor)$. This introduces quantization error. Optimization perspective:
- What's the approximation error from integerization?
- Does quantization affect solution quality?
- Would higher precision (millimeters) improve compactness?
- Are there systematic biases from rounding?

Provide sensitivity analysis or at least discuss quantization effects.

### m3: Bridge Edge Weights Not Optimized

Median land boundary length for water-crossing bridge edges is heuristic. From an optimization perspective:
- What weight minimizes total perimeter?
- Is there an optimal bridge weight that balances permitting crossings vs discouraging splits?
- Can bridge weights be learned or optimized?

This is a tunable parameter that affects objective value but is not optimized.

### m4: Subgraph Extraction Overhead Not Quantified

Recursive bisection requires extracting subgraphs at each level ($O(\log K)$ extractions). This involves:
- Index remapping
- Edge filtering
- File I/O for METIS

For California (9,213 tracts, 52 districts), this happens 6 times ($\lceil \log_2 52 \rceil = 6$ levels). What's the overhead? Is this a significant fraction of runtime?

From an optimization perspective, this suggests using METIS's in-memory API to eliminate file I/O.

### m5: No Discussion of Local Optima

METIS uses Kernighan-Lin refinement, which can get stuck in local optima. Does this happen in practice?
- Compare multiple METIS runs with different random seeds
- Measure solution variability
- Report best/worst/mean compactness across runs

Show whether solutions are stable or highly dependent on initialization.

### m6: Multi-Start or Restart Strategies Not Explored

Standard optimization practice for heuristics: run multiple times with different initializations, keep best solution. Does the paper do this? If not, why not? Multi-start could improve results with minimal cost (embarrassingly parallel).

## Strengths

1. **Clear objective function**: Minimizing total perimeter is a well-defined optimization objective that aligns with geometric goals.

2. **Strong empirical results**: 56% improvement demonstrates practical effectiveness at scale.

3. **Scalability**: O(N log K) complexity and 2-3 hours for 50 states shows computational efficiency.

4. **Handles real-world constraints**: Population balance ($\pm 0.5\%$), contiguity, water crossings are properly addressed.

5. **Reproducibility**: Detailed configuration enables replication.

6. **National validation**: 50 states with diverse characteristics demonstrates robustness.

## Questions for Authors

1. **Problem complexity**: Is the perimeter minimization problem NP-hard? If so, cite proof or provide reduction. If not, what's the optimal polynomial algorithm?

2. **Lower bounds**: Can you establish lower bounds on achievable compactness (LP relaxation, geometric arguments)? What's the gap between algorithmic and lower bound?

3. **MILP solutions**: For small states (1-4 districts), can you solve optimally with Gurobi/CPLEX? What's the optimality gap for METIS?

4. **Recursive bisection gap**: How much compactness is lost vs optimal k-way partitioning? Compare recursive to direct k-way METIS.

5. **Local optima**: Does METIS get stuck in local optima? What's solution variability across random seeds?

6. **Sensitivity to parameters**: How does compactness vary with population tolerance, edge weight precision, bridge weights?

7. **Multi-start strategies**: Would running METIS multiple times and keeping the best solution improve compactness? What's the expected gain?

8. **Alternative objectives**: Have you tried other objective functions (squared perimeter, boundary-to-area ratio)? Do they produce better compactness?

## Recommendations

- **Add formal problem statement**: Use standard optimization notation (decision variables, objective, constraints). Discuss MILP formulation and why it's intractable.

- **Complexity analysis**: State whether problem is NP-hard (with reference) or discuss polynomial cases.

- **Lower bounds**: Provide lower bounds on compactness (even loose ones) to estimate optimality gap.

- **MILP comparison**: Solve 3-5 small states optimally with Gurobi/CPLEX; measure METIS approximation ratio.

- **Compare recursive to k-way**: Run direct k-way METIS on representative states; quantify recursive bisection gap.

- **Multi-start experiments**: Run METIS 10x with different seeds; report best/worst/mean compactness; show variability.

- **Sensitivity analysis**: Vary population tolerance, edge weight precision, bridge weights; plot Pareto frontiers.

- **Discuss optimality**: Even without formal bounds, discuss how close solutions likely are to optimal based on Indiana comparison, local optima tests, etc.

- **Alternative algorithms**: Compare to at least one other optimization method (SA, GA, CP, or direct k-way) to validate METIS choice.

---

**Verdict**: Accept with Minor Revisions (for applied venues); Major Revisions (for optimization venues)

**Confidence**: High — I have deep experience with combinatorial optimization and graph algorithms. This paper presents a practical heuristic with strong empirical results, suitable for applications venues. However, it lacks the theoretical depth (complexity, bounds, approximation analysis) expected in optimization venues. For KDD/AAAI, the current empirical focus is appropriate with minor additions (multi-start experiments, sensitivity analysis, comparison to k-way). For SODA/IPCO, substantial theoretical work is needed: formal problem statement, complexity analysis, lower bounds, MILP comparison, and approximation guarantees. The authors should target revisions to their intended venue's standards.
