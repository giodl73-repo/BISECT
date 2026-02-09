# Review: Recursive Bisection for Congressional Redistricting

**Reviewer**: Ümit V. Çatalyürek (Georgia Institute of Technology)
**Expertise**: Hypergraph partitioning, parallel algorithms, scalability
**Date**: 2026-02-07
**Round**: 1

---

## Overall Assessment

This paper applies graph partitioning to congressional redistricting with reasonable technical competence. The recursive bisection approach using METIS is straightforward and produces credible results. The philosophical framing around "impossibility defense" is interesting, though more relevant to political science than algorithms research.

From a high-performance computing and scalability perspective, I see opportunities for significant improvement. The paper treats redistricting as 50 independent state-level problems with embarrassing parallelism, but doesn't explore more sophisticated parallel approaches or hypergraph formulations that could capture complex relationships beyond pairwise adjacency.

## Score: 3.0/4.0

**Accept with Revisions (Technical Improvements)**

The application demonstrates feasibility and the results are credible. With additional technical depth on scalability and advanced partitioning methods (hypergraphs, parallel algorithms), this would be a solid contribution to computational redistricting literature.

## Major Issues (Must Address)

### M1. Hypergraph Formulation Unexplored

**Issue**: Census tracts have complex relationships beyond pairwise adjacency that hypergraphs can capture more naturally than graphs.

**Hypergraph opportunities**:

1. **Multi-way adjacency**: Tracts meeting at a single point (corner adjacency) could be modeled as hyperedges rather than forcing pairwise graph edges

2. **Community structure**: Tracts within the same county, municipality, or school district could be connected by hyperedges, enabling natural preservation of these communities

3. **Geographic features**: Rivers, highways, mountain ranges that separate multiple tracts could be modeled as cut-cost hyperedges

**Why hypergraphs matter**: Hypergraph partitioning (hMETIS, Zoltan, PaToH) can optimize multi-way relationships simultaneously, potentially improving compactness and community preservation.

**Recommendation**: Add subsection 3.9 "Hypergraph Formulation" that:
- Formally defines hypergraph model for redistricting
- Implements for 2-3 representative states using hMETIS or PaToH
- Compares compactness and community preservation to graph-based approach
- Discusses computational complexity trade-offs

### M2. Scalability to Block-Level Inadequately Addressed

**Issue**: You claim block-level implementation is "feasible" (Sections 4.1, 6.1) but provide no evidence. As someone who works on large-scale graph partitioning, I'm skeptical.

**Scalability challenges**:

1. **Graph size**: California blocks = ~5 million nodes, ~15 million edges
   - METIS is serial for single graphs
   - Runtime: O(|V| log k) = O(5M × log 52) ≈ 30M operations
   - Even at 1M ops/sec, this is 30+ seconds per bisection, ×51 bisections = 25+ minutes just for California
   - Memory: Large graphs require significant RAM (geometric data, adjacency structures, coarsening hierarchy)

2. **Adjacency computation**: For 5M blocks, computing pairwise adjacency naively is O(25 trillion) operations
   - Requires spatial indexing (R-trees, STRtree)
   - Still expensive for millions of polygons

3. **I/O bottlenecks**: Loading 5M block geometries from shapefiles is slow

**Missing analysis**: No empirical data on even small-state block-level implementation (VT, DE have ~10K blocks)

**Recommendation**: Add Section 6.4.2 "Block-Level Implementation Feasibility Study":
- Implement for 3 small states (VT, DE, WY) at block level
- Report detailed performance: graph construction time, METIS time, memory usage
- Project to large states based on empirical scaling
- Discuss whether parallel METIS variants (ParMETIS, MT-METIS) are necessary

Without empirical evidence, claiming "feasibility" is speculation.

### M3. Parallel Algorithms Underutilized

**Issue**: You use state-level task parallelism (4-8 states simultaneously) but ignore graph-level parallelism within METIS.

**Parallel opportunities**:

1. **ParMETIS**: Distributed-memory parallel graph partitioning
   - For large states (CA, TX), could parallelize single-state partitioning
   - Enables block-level implementation at scale

2. **MT-METIS**: Shared-memory parallel METIS
   - Thread-level parallelism for coarsening, refinement
   - Speedup: 4-8× on modern multicores

3. **GPU acceleration**: Recent work on GPU graph partitioning (CuMETIS)
   - 10-100× speedup for specific kernels
   - May enable block-level implementation without HPC clusters

**Current approach**: Sequential METIS + task parallelism is fine for proof-of-concept but limits scalability

**Recommendation**: Add subsection 6.4.3 "Parallel Implementations" discussing:
- Which parallel METIS variants are applicable (ParMETIS for distributed, MT-METIS for shared)
- Projected speedups based on literature (cite ParMETIS papers)
- Whether GPU acceleration could enable real-time redistricting (interactive tools for commissions)

This would strengthen the scalability claims and show you've considered modern HPC approaches.

## Minor Issues (Should Address)

### m1. Alternative Adjacency Models

**Section 3.2 uses queen contiguity** (edge OR corner touching). But:
- **Rook contiguity** (only edge touching): stricter, may reduce spurious connections
- **Distance-based** (connect tracts within threshold): handles gaps from precision errors

**Sensitivity**: Does adjacency model choice affect outcomes?

**Recommendation**: Brief sensitivity analysis on 2-3 states comparing queen vs. rook contiguity.

### m2. Load Balancing Across States

**Section 4.6**: You mention 4-8 parallel workers for 50 states, but state runtimes vary 100× (Wyoming seconds, California 20 minutes).

**Load balancing**: How do you assign states to workers? Random? By size?

**Optimal assignment**: Bin-packing problem to minimize makespan

**Recommendation**: Paragraph on task scheduling strategy for state-level parallelism.

### m3. Multi-Constraint Partitioning for VRA

**Section 5.6**: You mention VRA compliance requires demographic constraints but don't discuss METIS's multi-constraint partitioning feature.

**METIS capability**: Can balance multiple vertex weights simultaneously (total population + Black VAP + Hispanic VAP)

**Application**: Create districts with balanced total population AND specified racial composition ranges

**Recommendation**: Demonstrate multi-constraint partitioning for 1-2 VRA-covered states showing it's "technically feasible" as claimed.

## Strengths

1. **Clean recursive bisection implementation**: Target weights for odd districts handled correctly
2. **Reasonable computational efficiency**: 2.5 hours for tract-level national run
3. **Novel application**: Redistricting is interesting new domain for graph partitioning
4. **Practical impact**: If adopted, could actually be deployed operationally

## Recommendation

**Revise and Resubmit (with Technical Enhancements)**

The paper demonstrates proof-of-concept for algorithmic redistricting using graph partitioning. Results are credible and the implementation appears sound.

To strengthen for computational venues (KDD, IPDPS, SC applications tracks):

1. **M1**: Explore hypergraph formulation (captures multi-way relationships)
2. **M2**: Empirically demonstrate block-level feasibility (not just claim it)
3. **M3**: Discuss parallel algorithms for scalability (ParMETIS, MT-METIS, GPU)

These additions would elevate the paper from "proof of concept" to "scalable computational framework" suitable for high-performance computing venues.

---

**Note**: The impossibility defense and Huntington-Hill framing are interesting for political science venues but less relevant for algorithms research. If targeting CS venues (KDD, SIAM, ACM conferences), emphasize computational contributions (scalability, hypergraphs, parallel algorithms) over philosophical arguments. If targeting political science (APSR), current balance is appropriate.
