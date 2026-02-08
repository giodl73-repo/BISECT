# Review: Slice-Based Cross-Census Validation (Round 3)

**Reviewer**: Ümit V. Çatalyürek (Georgia Tech)
**Expertise**: Hypergraph partitioning, parallel algorithms
**Round**: 3 (Full Paper Review)
**Date**: 2026-02-08

---

## Overall Assessment

From a graph partitioning perspective, this paper does an excellent job of adapting METIS to the redistricting domain and properly documenting the configuration. Section 3.4 (METIS Configuration) is one of the most complete METIS specifications I've seen in an applications paper—the authors clearly understand both the algorithm and its implementation details.

The slice-based validation framework is clever and addresses a real challenge in evaluating partition quality across evolving graphs. The variance decomposition provides interpretable metrics that could generalize beyond redistricting to other dynamic graph partitioning problems.

**Score**: **3.5/4** (Accept - Strong)

---

## Strengths

### 1. Complete METIS Specification

Section 3.4 is exemplary. The authors specify:
- Algorithm variant: KMETIS (recursive bisection) with clear justification
- All parameters: `ufactor=1`, `niter=20`, `ncuts=10`, `objtype=METIS_OBJTYPE_CUT`
- Coarsening and initial partitioning strategies
- Stochasticity handling: 10 runs with median selection by edge-cut

This is exactly what's needed for reproduction. Many papers just say "we used METIS" without configuration details, making results impossible to replicate.

### 2. Appropriate Graph Representation

Section 3.3 (Graph Construction) properly defines:
- Adjacency as Rook contiguity (shared boundaries, not corners)
- Edge weights as shared boundary length in meters
- Island handling via nearest-neighbor synthetic edges

The edge weight choice (boundary length) is well-motivated: minimizing cut edges is equivalent to minimizing total boundary length, which correlates with compactness. This is a natural objective for redistricting.

### 3. Scalability Analysis

Table 6 reports runtime statistics:
- Mean time per state-year: 253 seconds
- Total for 50 states × 3 years: 5.4 hours
- Empirical complexity: $O(n \log n)$ (confirmed by r=0.94 correlation between tract count and runtime)

This demonstrates computational feasibility at scale. METIS is well-suited for this problem size (85K nodes, 266K edges).

### 4. Stochasticity Analysis

Table 3 (METIS edge-cut variability) shows coefficient of variation <2% across 10 runs. This is consistent with METIS behavior on well-structured graphs—the randomness in coarsening and initial partitioning has minimal impact on final solution quality for graphs with clear community structure.

The authors correctly use the median run rather than mean, which is more robust to occasional poor initializations.

---

## Weaknesses / Areas for Improvement

### M1: Limited Algorithm Comparison (Major)

The paper evaluates only METIS recursive bisection. From a graph partitioning perspective, several alternatives would be valuable to compare:

**Direct k-way partitioning**: METIS also offers direct k-way partitioning (`METIS_PartGraphKway`). How does recursive bisection compare to direct k-way for district creation?

**Multilevel vs. flat partitioning**: Are the multilevel coarsening benefits significant for redistricting graphs? A comparison to flat spectral partitioning would quantify this.

**Alternative objectives**: METIS supports communication volume minimization (`METIS_OBJTYPE_VOL`). For redistricting, this might better capture compactness than edge-cut.

**Hypergraph partitioning**: Redistricting could be formulated as a hypergraph problem with tracts as vertices and districts as hyperedges. Tools like PaToH or KaHyPar might better model the multi-constraint redistricting problem.

**Recommendation**: Even a small pilot comparison (5 states) with direct k-way or alternative objectives would strengthen the paper.

### M2: Multi-Constraint Formulation

Congressional redistricting requires only population balance (single constraint). However, state legislative redistricting often requires multiple constraints:
- Total population balance
- Voting-age population balance
- Minority population quotas (VRA compliance)

METIS supports multi-constraint partitioning via `vwgt` (multi-dimensional vertex weights). The paper doesn't discuss whether the slice-based validation framework extends to multi-constraint settings.

**Questions**:
1. How would variance decomposition change with multiple constraints?
2. Would geographic vs. temporal variance differ for VRA-constrained partitioning?
3. Does METIS performance degrade with multiple constraints (it typically does)?

This deserves discussion in limitations or future work.

### m1: Edge Weight Normalization

Section 3.3.2 uses boundary length in meters as edge weights. However, tract sizes vary enormously (urban tracts: ~0.5 km², rural tracts: ~500 km²). Should edge weights be normalized by tract size or perimeter to avoid bias?

**Concern**: Long boundaries between large rural tracts might be overweighted compared to short boundaries between small urban tracts, even if the latter represent more significant district boundary irregularity.

**Recommendation**: Test sensitivity to edge weight normalization (e.g., boundary length / geometric mean of tract perimeters).

### m2: Population Balance Tolerance

The paper uses `ufactor=1` (0.1% max imbalance), which is stricter than the legal requirement (0.5% per Karcher v. Daggett). While conservative, this might unnecessarily constrain partitioning and reduce achievable compactness.

**Question**: How does compactness vary with `ufactor` ∈ {1, 5, 10}? Is there a compactness-balance tradeoff that the slice-based framework could quantify?

### m3: Graph Properties

Table 3 reports average degree (mean 6.2 across all states). However, degree distributions matter for partitioning quality. High-degree nodes (urban tracts with many neighbors) behave differently than low-degree nodes (rural tracts) during coarsening.

**Recommendation**: Report degree distribution statistics (median, 90th percentile, max degree). Are there power-law properties? How does degree heterogeneity affect METIS performance?

---

## Minor Issues

1. **Section 3.4.2** (Population Balance): The formula uses ufactor/1000, but ufactor=1 means 0.1%, not 0.001%. Check if this is a typo or misunderstanding of METIS units.

2. **Section 3.3.3** (Handling Islands): "237 synthetic edges across all 50 states and 3 years" - were these edges given special weights (e.g., high cost to discourage use)? Or are water crossings penalized equally to land boundaries?

3. **Table 3**: "Edge-cut measured in total meters" - is this the sum of weights of cut edges? Clarify that this is weighted edge-cut, not unweighted cut size.

4. **Section 3.4.3**: You specify `ctype=METIS_CTYPE_SHEM` (sorted heavy-edge matching). Did you test alternative coarsening strategies (RM, SHEM, HEM)? SHEM is good for weighted graphs but may not be optimal for all graph types.

5. **Section 4.5** (Computational Performance): "AMD Ryzen 9 5950X, 64GB RAM" - was METIS run with multiple threads? METIS 5.1.0 supports OpenMP parallelism, which could substantially reduce runtime.

6. **Algorithm 1, Line 8**: "K-means clustering" - K-means finds spherical clusters, but geographic regions may be elongated (e.g., coastal slices). Did you consider spectral clustering or DBSCAN which handle non-spherical clusters better?

---

## Questions for Authors

1. Have you tested direct k-way partitioning vs. recursive bisection? Theory suggests recursive bisection produces more balanced partition trees but may have higher edge-cut.

2. Did you use METIS's multi-constraint partitioning features? Even for single-constraint problems, METIS quality can be sensitive to how constraints are specified.

3. Section 3.4.4 mentions using median run by edge-cut. How much do other metrics (compactness, runtime) vary across the 10 runs? Are edge-cut and compactness strongly correlated?

4. The paper focuses on METIS 5.1.0. Are you aware that METIS 5.2.0 was released in 2023 with improved recursive bisection heuristics? Would results differ?

---

## Recommendation

**Accept (Strong) - With Suggestions for Extensions**

This is a strong applications paper that demonstrates appropriate use of graph partitioning for a complex real-world problem. The METIS configuration is properly documented, the graph representation is well-motivated, and the validation framework is novel.

My main suggestion is to include some algorithm comparison (even limited) to demonstrate that the slice-based validation framework can differentiate between algorithms. Currently, we know the framework exists and is specified, but we don't know whether it provides discriminative power.

For publication, I recommend:
1. Add a brief pilot comparison (5 states) with direct k-way or alternative METIS objectives
2. Discuss multi-constraint extension (even if not implemented)
3. Report degree distribution statistics (not just mean degree)

Even without these extensions, the paper makes a solid contribution. The methodology is sound, and the METIS application is appropriate for the problem domain.

---

## Summary

This paper successfully bridges graph partitioning algorithms and redistricting validation. The authors demonstrate strong understanding of METIS internals and appropriate problem formulation as a graph partitioning task. The slice-based validation framework could generalize to other dynamic graph partitioning problems (social networks, transportation networks, scientific computing meshes).

My score reflects strong methodology and appropriate algorithm application, with room for deeper algorithmic exploration. This is solid work that advances the state of redistricting validation.
