# Review: Slice-Based Cross-Census Validation (Round 3)

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, graph partitioning, multilevel algorithms
**Round**: 3 (Full Paper Review)
**Date**: 2026-02-08

---

## Overall Assessment

As the creator of METIS, I'm pleased to see such a thorough and correct application of the algorithm to redistricting. Section 3.4 (METIS Configuration) is one of the best specifications I've seen in an applications paper—every parameter is documented, stochasticity is properly handled, and the authors demonstrate deep understanding of the algorithm's behavior.

The slice-based validation framework is interesting and novel. The idea of creating temporally stable validation regions to assess algorithm performance across evolving graphs could generalize beyond redistricting to other dynamic partitioning problems.

**Score**: **4.0/4** (Strong Accept)

---

## Strengths

### 1. Exemplary METIS Configuration

Section 3.4 is exactly what I wish every METIS applications paper would include:

**Algorithm variant**: KMETIS (recursive bisection) with clear justification—you correctly note it produces more balanced partition trees than direct k-way

**All parameters specified**:
- `ufactor=1` (0.1% imbalance) - conservative but appropriate for legal requirements
- `niter=20` - standard refinement iterations
- `ncuts=10` - properly handling stochasticity
- `objtype=METIS_OBJTYPE_CUT` - standard edge-cut minimization
- `ctype=METIS_CTYPE_SHEM` - sorted heavy-edge matching (good for weighted graphs)
- `iptype=METIS_IPTYPE_RANDOM` - random initial partition

**Stochasticity handling**: Running 10 times and selecting median by edge-cut is exactly the right approach. Many papers run once and don't acknowledge METIS's randomness.

**Version specified**: METIS 5.1.0 - this matters because 5.x has different coarsening heuristics than 4.x

This level of detail enables exact reproduction, which is rare and commendable.

### 2. Appropriate Graph Representation

Section 3.3 properly constructs the graph:
- Rook contiguity (shared boundaries, not corners) - correct for redistricting
- Edge weights as boundary length - natural objective for compactness
- Island handling via nearest-neighbor edges - pragmatic solution

The edge weight choice (boundary length) is well-motivated: minimizing weighted edge-cut is equivalent to minimizing total district boundary length, which correlates with compactness. This is a more natural objective than unweighted edge-cut.

### 3. Understanding of Algorithm Behavior

Table 3 (METIS edge-cut variability) shows coefficient of variation <2%, and the authors correctly interpret this as "high algorithmic stability." This is consistent with METIS behavior on well-structured graphs—redistricting graphs have clear community structure (geographic clustering), so random initialization has minimal impact on final quality.

The authors also correctly note that `ufactor=1` (strict population balance) provides "safety margin for floating-point precision"—showing they understand METIS's internal tolerance handling.

### 4. Scalability Confirmation

Table 6 reports:
- 253 seconds mean per state-year
- Empirical $O(n \log n)$ complexity (r=0.94 correlation)

This matches METIS's theoretical complexity for recursive bisection. The authors properly validate that the algorithm scales as expected, which not all applications papers bother to check.

### 5. Novel Validation Framework

The slice-based approach is interesting. While I don't work on redistricting specifically, the problem of validating partitioning algorithms on evolving graphs is general. This framework could apply to:
- Social network partitioning (graphs evolve over time)
- Scientific mesh partitioning (meshes refined adaptively)
- Circuit partitioning (netlists change across design iterations)

The variance decomposition (geographic vs. temporal) provides an interpretable metric for algorithm stability.

---

## Weaknesses / Areas for Improvement

### M1: Limited Algorithm Exploration

The paper evaluates only recursive bisection. METIS offers several variants that would be worth comparing:

**Direct k-way partitioning** (`METIS_PartGraphKway`): Creates k districts in one pass rather than recursive bisection. Typically produces higher edge-cut but more balanced partitions and better load balance in parallel computing contexts.

**Volume minimization** (`METIS_OBJTYPE_VOL`): Minimizes communication volume rather than edge-cut. For redistricting, this might better capture "interaction" between districts (tracts on district boundaries interact with multiple districts).

**Edge-cut vs. volume**: Table 2 reports edge-cut statistics, but volume (sum of subdomain interface sizes) might be a better metric for compactness. Edge-cut counts cut edges; volume counts boundary nodes. For redistricting, volume corresponds to "number of tracts on district boundaries," which might be more interpretable.

**Recommendation**: Include a small comparison (5 states) testing:
- Recursive bisection vs. direct k-way
- Edge-cut vs. volume objectives
- Report both metrics even if optimizing only one

### m1: Multi-Constraint Potential

METIS supports multi-constraint partitioning (multiple balance objectives simultaneously). Congressional redistricting has only one constraint (population), but state legislative redistricting often requires:
- Total population balance
- Voting-age population balance
- Minority population thresholds (VRA)

The paper doesn't discuss whether slice-based validation extends to multi-constraint settings.

**Question**: Would variance decomposition differ for multi-constraint problems? My experience suggests multi-constraint partitioning has higher variance because the feasible space is smaller.

### m2: Coarsening Strategy Sensitivity

Section 3.4.3 specifies `ctype=METIS_CTYPE_SHEM` (sorted heavy-edge matching). This is good for weighted graphs, but did you test alternatives?

**METIS coarsening options**:
- `RM`: Random matching (fastest, lowest quality)
- `HEM`: Heavy-edge matching (good for unweighted graphs)
- `SHEM`: Sorted heavy-edge matching (best for weighted graphs, slower)

SHEM is the right choice for your weighted graph, but reporting sensitivity to coarsening strategy would strengthen the validation.

**Recommendation**: Test RM, HEM, SHEM on 2-3 states. I expect SHEM to win (as you used), but quantifying the improvement would validate the choice.

### m3: Imbalance Tolerance Trade-off

You use `ufactor=1` (0.1% max imbalance), which is stricter than the legal requirement (0.5%). While conservative, this constrains partitioning and might reduce achievable compactness.

**Question**: What's the compactness-imbalance Pareto frontier? Varying `ufactor` ∈ {1, 2, 5, 10} (0.1%, 0.2%, 0.5%, 1.0%) could reveal whether the legal limit (0.5%) binds compactness.

If compactness is insensitive to `ufactor` in this range, it suggests population constraints don't significantly limit compactness—which is itself an interesting finding.

---

## Minor Issues

1. **Section 3.4.2**: The formula `|P_i - P̄| / P̄ ≤ ufactor/1000` - verify the denominator. METIS interprets `ufactor` as parts per thousand, so `ufactor=1` means 0.1% = 1/1000. The formula is correct, but double-check you're passing `ufactor=1` (not `ufactor=1000`) to METIS.

2. **Section 3.4.4**: "Median run by edge-cut" - good choice. Mean can be distorted by occasional poor initializations, while median is robust. But did you check other metrics (runtime, compactness) for the median run? Is the median-edge-cut run also median-compactness?

3. **Table 3**: "Edge-cut measured in total meters of district boundaries" - clarify this is the sum of *cut* edge weights, not total edge weight in the graph. The terminology "edge-cut" is standard in graph partitioning but might be unclear to redistricting readers.

4. **Section 4.5**: "AMD Ryzen 9 5950X" - was METIS compiled with OpenMP? METIS 5.1.0 supports multi-threaded coarsening and refinement. If you used single-threaded METIS, note that parallel METIS could reduce runtime 3-5×.

5. **Algorithm 1, Line 8**: You use k-means for slice creation. K-means is fast but assumes spherical clusters. For elongated geographic regions (coastal slices, river valleys), spectral clustering might better capture geography. Did you test alternatives?

6. **Section 5.6.1** (Multi-Algorithm Comparison): You mention testing GerryChain and simulated annealing as future work. Be aware that these are *stochastic* algorithms, while METIS (with fixed seed) is deterministic. Comparing deterministic vs. stochastic algorithms requires care—you'd need to compare METIS variance (10 runs) to ensemble spread.

---

## Questions for Authors

1. You chose recursive bisection over direct k-way. Did you empirically test this, or was it theoretical preference? My experience is that recursive bisection has lower edge-cut but direct k-way has better balance—what did you find?

2. Table 3 shows CV < 2% for edge-cut. What about compactness (PP, Reock) variance across 10 runs? Is compactness more or less stable than edge-cut?

3. You mention METIS 5.1.0. Are you aware that 5.2.0 (released 2023) has improved refinement for highly constrained partitions (like your strict `ufactor=1`)? Would results differ?

4. Did you use METIS's `seed` parameter to control randomness across the 10 runs? If not, runs might be correlated (e.g., if run on same machine state).

5. Section 3.3.2 uses boundary length in meters as edge weights. Did you normalize by UTM zone distortion? In wide East-West states (e.g., Montana), UTM distortion can be 1-2%, which might bias edge weights.

---

## Recommendation

**Strong Accept (4.0/4)**

This is an exemplary applications paper that demonstrates correct, thorough, and insightful use of METIS for a complex real-world problem. The METIS configuration is complete and reproducible—I could replicate your results exactly from the specification in Section 3.4. The stochasticity analysis (Table 3) shows you understand the algorithm's behavior, and the scalability validation (Table 6) confirms expected complexity.

The slice-based validation framework is novel and well-motivated. While I'd like to see some algorithm comparisons (recursive vs. direct k-way, edge-cut vs. volume objectives), these are enhancements rather than requirements. The paper makes a solid contribution as-is.

I have no hesitation recommending acceptance. This is the kind of applications paper I enjoy reviewing: the authors understand the algorithm deeply, apply it appropriately, validate its behavior, and create a novel evaluation framework. Well done.

---

## Summary

As the creator of METIS, I'm pleased to see such thoughtful and correct application. The slice-based validation framework is a clever solution to the temporal validation problem, and the variance decomposition provides interpretable metrics. The methodology is sound, the METIS specification is complete, and the paper makes a valuable contribution to both redistricting and dynamic graph partitioning.

This is a strong accept with no reservations. Suggested improvements (algorithm comparisons, multi-constraint discussion) would strengthen the paper further, but it's publication-ready as-is.
