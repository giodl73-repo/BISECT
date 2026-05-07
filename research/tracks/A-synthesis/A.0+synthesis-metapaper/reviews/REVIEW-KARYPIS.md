# Quality Assessment: Algorithmic Objectivity for Congressional Redistricting

**AI Persona**: George Karypis (based on work at University of Minnesota)
**Expertise Area**: METIS algorithm, graph partitioning, parallel computing
**Round**: 1
**Date**: 2026-02-08

> **Simulation Notice**: This is AI-generated feedback for quality improvement, not a real peer review. Use these insights to strengthen your work.

---

**Content Mode**: full

---

## Overall Assessment

As the creator of METIS, I'm gratified to see the algorithm applied to congressional redistricting at this scale. The technical execution is generally sound: the authors correctly implement recursive bisection with edge-weighted partitioning, and the population balance constraints (±0.5%) demonstrate proper use of the balance parameter. The claim of "near-linear time O(n log k)" is accurate for the partitioning phase, though graph construction and validation add overhead not captured in this complexity bound.

However, the paper has significant technical gaps that need addressing before publication in a venue like Science. First, the edge-weighting scheme is underspecified—how are weights normalized? What prevents pathological cases where high-weight edges dominate the optimization? Second, the paper doesn't report key quality metrics (edge-cut, communication volume) that would allow algorithmic comparison. Third, there's no sensitivity analysis on METIS parameters (niter, seed, refinement strategy)—are results deterministic or do random initializations matter?

Most critically, the paper conflates "what METIS minimizes" (weighted edge-cut) with "what redistricting should optimize" (compactness, VRA compliance). The algorithm minimizes edge-cut because that's its objective function, not because edge-cut is normatively correct. The paper needs to justify why minimizing edge-cut is the right objective for redistricting, not just assert that it produces good results.

## Score

**Score**: 3/4 — Accept (with substantial technical revisions)

## Major Issues (Blocking)

### M1: Edge-Weighting Scheme Underspecified

Section 3 states edges receive weights "inversely proportional to distance and proportional to demographic similarity" but provides no formula. How is demographic similarity measured? Hamming distance on racial categories? Euclidean distance in demographic space? How are distance and similarity combined—multiplicatively? Additively? What prevents weight distributions from being pathologically skewed (e.g., one high-weight edge pulling the partition)?

**Required**: Provide the actual weight function w(e) = f(distance, demographics). Report weight distribution statistics (min, max, median, 90th percentile) to show weights are well-behaved. If weight distributions are highly skewed, discuss normalization strategy.

### M2: Missing Algorithmic Metrics

The paper reports population deviation and compactness but not the objective function METIS actually minimizes: weighted edge-cut. Without reporting cut weight, we can't assess whether METIS is converging properly or whether the optimization landscape is well-structured. Also missing: runtime breakdown (graph construction vs. partitioning vs. validation), memory footprint, number of refinement passes.

**Required**: Add a table with columns [State, n_tracts, k_districts, edge-cut, runtime, memory]. This allows readers to assess scaling and reproducibility. Also report niter value used—you mention "niter=100" in one place but don't confirm this is used throughout.

### M3: Parameter Sensitivity Not Explored

METIS has several parameters that affect output: niter (refinement iterations), seed (random initialization), ufactor (unbalance factor), refinement strategy (greedy vs. random moves). The paper treats METIS as deterministic but different seeds can produce different partitions. Are results stable across seeds? If not, how much variation exists?

**Required**: For 2-3 states, run with 10 different random seeds and report coefficient of variation on key metrics (population deviation, edge-cut, MM district count). If CV > 5%, acknowledge this and discuss implications for reproducibility.

## Minor Issues

### m1: Complexity Bound Incomplete

The claim of "near-linear time O(n log k)" describes METIS multilevel partitioning but not the full pipeline. Graph construction requires adjacency detection (likely O(n log n) with spatial indexing), and validation requires contiguity checks (BFS/DFS per district = O(n) total). Report wall-clock runtime breakdown to show which phases dominate.

### m2: Recursive vs. Direct k-way Partitioning

The paper uses recursive bisection but doesn't justify this choice. METIS also supports direct k-way partitioning (gpmetis -ptype=kway), which typically produces better edge-cuts. Did you test both? If recursive is comparable for redistricting despite being suboptimal for edge-cut, this suggests redistricting objectives differ from graph partitioning—which is worth discussing.

### m3: Balance Constraint Too Tight?

Population deviation of ±0.5% is tighter than Supreme Court requirements ("as nearly equal as practicable" typically allows ±1%). A tighter constraint makes METIS's optimization problem harder and may force suboptimal cuts. Did you experiment with relaxing ufactor? If ±1% allows substantially better edge-cuts, this tradeoff should be reported.

### m4: Adjacency Graph Construction

The paper mentions "queen contiguity" but doesn't specify how adjacency is computed from shapefile geometries. Are you using Shapely/GeoPandas intersection tests? R-tree spatial indexing? Naive O(n²) pairwise checks? This matters for reproducibility and affects the graph construction bottleneck.

### m5: Contiguity Guarantee Claim

Section 4 states "100% geographic contiguity (guaranteed by algorithm)" but METIS doesn't inherently guarantee connected partitions—it minimizes edge-cut, which encourages but doesn't enforce connectivity. You likely post-process to enforce contiguity. Clarify: does METIS produce contiguous partitions directly, or do you repair disconnected districts afterward?

## Strengths

1. **Proper use of METIS**: The authors correctly use recursive bisection with balance constraints, which is appropriate for this application.

2. **Impressive scale**: Partitioning 85,000 tracts into 435 districts across 50 states demonstrates that METIS scales well to real-world redistricting problems.

3. **Edge-weighting innovation**: Using weighted edges to encode VRA objectives is clever and shows the authors understand METIS's flexibility beyond uniform graphs.

4. **Reproducibility**: The paper provides enough detail that someone familiar with METIS could likely replicate the approach (though the missing weight function is a gap).

## Questions for Authors

1. What METIS variant did you use? Original C library (http://glaros.dtc.umn.edu/gkhome/metis/metis/overview)? PyMETIS Python bindings? Ensure citations point to correct version.

2. Did you use METIS's vertex weight feature to encode population? If so, how did you discretize continuous population values?

3. For edge weights: are they computed once (static) or recomputed during refinement? METIS assumes static weights—if you update them, this could affect convergence.

4. Have you compared to other graph partitioners (SCOTCH, KaHIP, ParMETIS)? If METIS isn't uniquely suited, this should be acknowledged.

5. The 56% compactness improvement over unweighted baseline—is this comparing weighted METIS to unweighted METIS? Or to a different baseline? Clarify comparison.

## Recommendations

- Specify edge weight function mathematically and report weight statistics
- Add algorithmic metrics table (edge-cut, runtime, memory per state)
- Conduct seed sensitivity analysis for 2-3 states, report variation
- Provide runtime breakdown (construction vs. partitioning vs. validation)
- Compare recursive bisection to direct k-way partitioning
- Clarify adjacency construction and contiguity enforcement details
- If using balance constraints tighter than required, explore relaxation
- Cite specific METIS version and variant used
- Consider releasing code and preprocessed graphs for full reproducibility

---

**Verdict**: This is solid applied graph partitioning work that demonstrates METIS's versatility beyond its traditional HPC domain. The technical gaps are fixable with more detailed reporting on algorithm internals. Once these details are added, this will be a valuable contribution showing how multilevel graph partitioning can scale to real-world redistricting. The algorithmic work is sound; the exposition needs more technical depth for readers who want to reproduce or extend this work.
