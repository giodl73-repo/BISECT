# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: George Karypis (University of Minnesota)
**Expertise**: METIS, graph partitioning, multilevel algorithms
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

As the developer of METIS, I'm pleased to see the library applied to congressional redistricting—a novel application domain that showcases graph partitioning's versatility beyond scientific computing and VLSI design. The paper demonstrates that METIS's multilevel framework handles the specific requirements of redistricting (contiguity, balance, compactness) effectively.

The recursive bisection approach is sound and the implementation appears technically competent. The philosophical framing around Huntington-Hill is creative, though perhaps more relevant for political science audiences than algorithmists.

However, as a graph partitioning researcher, I have significant concerns about: (1) parameter choices that seem arbitrary without justification, (2) incomplete exploration of METIS's capabilities, (3) missing comparison to state-of-the-art partitioning methods, and (4) insufficient performance analysis.

## Score: 3.0/4.0

**Accept with Revisions**

The application of graph partitioning to redistricting is novel and the results demonstrate feasibility. The philosophical contribution ("impossibility defense") may appeal to political science venues but is less relevant for algorithmic evaluation. With revisions addressing the technical issues below—particularly parameter justification and performance analysis—this would be suitable for publication in interdisciplinary venues (not core CS theory/algorithms venues like SODA, but applied venues like KDD or applications tracks).

## Major Issues (Must Address)

### M1. Parameter Choices Inadequately Justified

**Issue**: Section 3.5 lists METIS parameters but doesn't justify choices or analyze sensitivity. As METIS's designer, I can tell you these parameters significantly affect output quality—arbitrary choices undermine reproducibility claims.

**Specific parameter concerns**:

1. **ufactor=5 (0.5% imbalance tolerance)**:
   - Why 0.5%? Legal standard is "as nearly equal as practicable"—why not ufactor=1 (0.1%) or ufactor=10 (1%)?
   - How does imbalance tolerance affect edge cuts? (Tighter balance usually increases edge cuts)
   - Did you try other values empirically?

2. **niter=100 (refinement iterations)**:
   - METIS default is 10. You use 100 (10× more).
   - Justification: "higher quality"—but did you measure this? What's the quality improvement from 10→100?
   - Runtime impact? (More iterations = slower, but by how much?)
   - Why not 1000? Or adaptive stopping when quality plateaus?

3. **objtype=cut (edge-cut minimization)**:
   - METIS also supports minimizing total communication volume (objtype=vol)
   - For redistricting, volume minimization might produce different district shapes
   - Did you try it? Why prefer cuts over volume?

4. **No random seed specified**:
   - METIS uses randomization in coarsening (maximal matching) and initial partitioning
   - You mention <1% variation across runs but provide NO DATA
   - How many runs? What's the distribution? Min/max variation?
   - For "reproducibility" claims, you MUST fix random seed or empirically characterize variation

5. **minconn=1 (minimize subdomain connectivity)**:
   - This forces each partition to be connected through a minimum spanning tree
   - Interaction with contig flag? (redundant or complementary?)
   - Impact on edge cuts?

**Recommendation**: Add Section 4.5 "Parameter Sensitivity Analysis" with systematic exploration:

1. **Parameter sweep**: For 5 representative states (small/medium/large, even/odd districts):
   - Vary ufactor: 1, 5, 10, 20 (keep others fixed)
   - Vary niter: 10, 50, 100, 200 (keep others fixed)
   - Try objtype=vol vs. objtype=cut

2. **Quality vs. runtime trade-offs**: Plot Pareto frontier (edge cuts vs. runtime) for different parameter settings

3. **Random seed ensemble**: For each state:
   - Run 100 times with different random seeds (seed=1 to 100)
   - Report distribution of edge cuts, Polsby-Popper scores, partisan outcomes
   - Show variation is <1% as claimed (with actual data)

4. **Default parameter justification**: Based on empirical results, explain why your default choices (ufactor=5, niter=100) are reasonable

This is ESSENTIAL for reproducibility claims. Without parameter sensitivity analysis, readers cannot verify your results or apply your method.

### M2. Incomplete Exploration of METIS Capabilities

**Issue**: You use a small subset of METIS's features while ignoring capabilities that could improve results significantly.

**Underutilized features**:

1. **Edge weights**: METIS supports edge-weighted graphs. You use unweighted edges (all census tract adjacencies treated equally).

   - **Natural weighting**: edge weight = shared boundary length between tracts
   - **Effect**: Minimizing weighted edge cuts directly minimizes total district perimeter (better proxy for compactness than unweighted edge counts)
   - **Why not use this?** Section 3.8 mentions "edge-weighted optimization" as future work, but this is a standard METIS feature you could implement NOW

2. **Multi-constraint partitioning**: METIS supports balancing multiple constraints simultaneously (e.g., population + racial demographics for VRA compliance)

   - **Application**: Balance total population AND Black voting-age population to create majority-minority districts
   - **Why not explore this?** Section 5.6 claims VRA compliance is "technically feasible" but you don't demonstrate it

3. **Fixed vertices**: METIS can fix certain vertices to specific partitions (useful for preserving existing boundaries or respecting county lines)

   - **Application**: Fix county boundary tracts to not cross districts (respecting traditional boundaries)
   - **Why not try this?**

4. **k-way partitioning**: METIS's direct k-way is often higher quality than recursive bisection

   - **Question**: Did you compare direct k-way to recursive bisection empirically?
   - **Trade-off**: Recursive = hierarchical structure, k-way = better optimality
   - **Your claim** (Section 3.8): "Recursive bisection for hierarchical interpretability"—but no empirical comparison of quality gap

**Recommendation**: Add Section 3.9 "Alternative METIS Configurations" exploring:

1. **Edge-weighted graphs**: Implement for 3-5 states, compare compactness to unweighted. If 50-60% improvement as claimed in Section 6.4.1, show actual results.

2. **Direct k-way vs. recursive bisection**: Empirical comparison on 5-10 states showing edge-cut quality gap (if any)

3. **Multi-constraint for VRA**: Demonstrate VRA-constrained optimization for 2-3 covered states (AL, MS, LA)

These are not "future work"—they're standard METIS features you should evaluate to justify your design choices.

### M3. Missing Comparison to State-of-the-Art Graph Partitioning Methods

**Issue**: Section 6.2 compares to other redistricting methods (MCMC, genetic algorithms) but NOT to alternative graph partitioning libraries. METIS is 20+ years old; newer methods exist.

**Modern alternatives**:

1. **KaHIP (Karlsruhe High-Quality Partitioning)**: Sanders et al., 2013
   - Often achieves 5-15% better edge cuts than METIS
   - Supports same constraints (balance, contiguity)
   - Why not compare?

2. **Scotch/PT-Scotch**: Pellegrini, 2012
   - Comparable quality to METIS, different algorithmic approach
   - Used in some redistricting research

3. **Zoltan**: Sandia hypergraph partitioner
   - Can model more complex relationships (hypergraphs vs. graphs)

4. **Recent METIS improvements**: My research group has developed enhanced versions (hMETIS, MT-METIS) with improved quality

**Why this matters**: If KaHIP produces 10% better edge cuts, your compactness gap with enacted districts (0.220 vs. 0.305) might narrow. You claim compactness gap is due to "block vs. tract granularity" and "manual optimization," but maybe it's also because METIS isn't the best available partitioner.

**Recommendation**: Add Section 6.2.2 "Comparison to Alternative Graph Partitioners":

1. **Empirical comparison**: For 5-10 representative states:
   - Run METIS, KaHIP, Scotch with identical settings
   - Compare edge cuts, Polsby-Popper scores, runtime

2. **Results interpretation**:
   - If METIS is comparable: Validates your choice, shows results robust to partitioner
   - If METIS is worse: Explains some compactness gap, suggests improvement path
   - If METIS is better: Great! Strengthens contribution

3. **Practical justification**: Even if KaHIP is slightly better, METIS's maturity, stability, and widespread availability may justify its use

This comparison is standard in graph partitioning research and should be included.

### M4. Insufficient Performance Analysis

**Issue**: Section 4.6 mentions "2.5 hours on 6-core desktop" but provides minimal performance analysis. For a computational method, this is inadequate.

**Missing performance data**:

1. **Per-state timing**: You mention "seconds (single-district) to 20 minutes (California)" but no detailed table

2. **Scalability analysis**: How does runtime scale with graph size?
   - Theoretically: METIS is O((|V|+|E|) log k)
   - Empirically: Does your implementation match theoretical complexity?
   - Plot: runtime vs. |V| (number of tracts) showing complexity trend

3. **Memory consumption**: You mention "<2GB per state" but:
   - Peak memory for largest state?
   - Memory vs. graph size scaling?
   - Is memory a bottleneck for block-level implementation?

4. **Parallelization**: You use "4-8 parallel workers" for state-level parallelism, but:
   - METIS itself is serial (per-state)
   - Have you tried parallel METIS variants (MT-METIS, ParMETIS)?
   - For block-level graphs (100× larger), parallel partitioning may be necessary

5. **Bottleneck analysis**: Where does time go?
   - Graph construction (reading Census data, computing adjacency)?
   - METIS partitioning?
   - Post-processing (computing compactness, overlaying election data)?

**Recommendation**: Add Section 4.6 "Performance Analysis" with:

1. **Detailed timing table**: All 50 states with |V|, |E|, districts, runtime (graph construction vs. METIS vs. post-processing)

2. **Scalability plot**: Runtime vs. |V| showing O(|V| log k) trend

3. **Memory profile**: Peak memory vs. graph size

4. **Block-level projections**: Based on empirical scaling, estimate runtime and memory for block-level graphs. Feasible on single machine or requires HPC?

5. **Comparison to manual redistricting**: 2.5 hours algorithmic vs. months of committee deliberations—emphasize efficiency advantage

This performance analysis is standard for computational papers and provides practical guidance for implementation.

## Minor Issues (Should Address)

### m1. Graph Construction Details Missing

**Section 3.2 describes adjacency** but lacks implementation details:
- How do you compute queen contiguity efficiently? (Naive pairwise comparison is O(n²))
- Do you use spatial indexing (R-trees, quadtrees)?
- What about sliver polygons and numerical precision issues?
- Runtime for adjacency computation vs. METIS runtime?

**Recommendation**: Add paragraph explaining efficient adjacency computation (likely using spatial libraries like GEOS or GeoPandas STRtree).

### m2. METIS Version and Compilation

**Section 3.3**: You mention "METIS 5.1.0" but:
- Compiled with which flags? (compiler optimizations affect performance)
- Using gpmetis command-line or METIS library API?
- Any modifications to METIS source code?

**Reproducibility**: Provide exact compilation instructions or Docker container for perfect reproducibility.

### m3. Water-Based Adjacency Hack Needs Validation

**Section 3.2 county bridging**: Clever solution but raises algorithmic concerns:
- How many bridge edges added per state?
- Do bridges become district boundaries (creating water-crossing districts)?
- Are bridges ever cut by METIS, or do they stay internal to districts?
- Alternative: Could you use METIS with disconnected components (partition each component separately, then combine)?

**Recommendation**: Empirical analysis of bridge impact: For 3-5 island states, report number of bridges added and whether any become district boundaries.

### m4. Contiguity Verification Missing

**You rely on METIS's contig flag** but:
- METIS guarantees graph connectivity, not geometric contiguity
- With bridge edges, graph-contiguous ≠ geographically-contiguous
- Do you verify output districts are geometrically contiguous (all points in district form connected region)?

**Recommendation**: Post-process verification that all districts are geometrically contiguous without relying solely on METIS flag.

## Strengths (Preserve These)

1. **Novel application domain**: Applying graph partitioning to redistricting showcases METIS versatility

2. **Technically sound implementation**: Recursive bisection with target weights for odd districts is correct

3. **Computational efficiency**: 2.5 hours for 50 states demonstrates scalability

4. **Clean methodology**: Reproducible and understandable (with parameter sensitivity analysis)

5. **Practical impact**: If adopted, this could actually be implemented operationally (unlike some theoretical proposals)

## Recommendation

**Revise and Resubmit**

This paper demonstrates a novel application of graph partitioning to redistricting. The implementation is technically competent and the results show feasibility.

However, four critical additions are needed:

1. **M1**: Parameter sensitivity analysis (essential for reproducibility)
2. **M2**: Exploration of underutilized METIS features (edge weights, multi-constraint, k-way)
3. **M3**: Comparison to state-of-the-art partitioners (KaHIP, Scotch)
4. **M4**: Comprehensive performance analysis (scalability, memory, bottlenecks)

These additions would strengthen the paper from "proof of concept" to "rigorous computational study." They're standard expectations in graph partitioning research.

**Target venue**: With revisions, suitable for interdisciplinary venues (KDD applications track, SIAM Applied Math, PLOS Computational Biology applied algorithms). Not suitable for pure theory venues (SODA, FOCS) due to limited algorithmic novelty, but the application contribution is significant.

---

**Personal note**: I'm gratified to see METIS applied to socially important problems like redistricting. The library was designed for flexibility across application domains, and your work demonstrates this well. My critiques focus on ensuring the graph partitioning aspects are rigorous enough to withstand scrutiny from the algorithms community. With the suggested additions, you'll have a solid contribution that bridges CS and political science effectively.
