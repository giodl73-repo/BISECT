# Algorithmic Design for Congressional Redistricting — Plan

**Artifact Type**: Research Paper (Track B Head - Methodological)
**Goal**: Comprehensive overview of algorithm design decisions for congressional redistricting
**Estimated Effort**: 2-3 weeks
**Status**: Planned
**Source**: Track head paper synthesizing algorithm design rationale

---

## Objective

Provide the **definitive explanation** of why we chose recursive bisection, edge-weighting, METIS, and census tracts for congressional redistricting. This paper justifies all major architectural and methodological decisions.

**Core Question**: Why these specific algorithmic choices, and how do they work together to achieve objective redistricting?

**Scope**: This is the Track B head paper, synthesizing findings from B.1-B.5 into a cohesive narrative about algorithm design.

---

## Research Questions

### RQ1: Why Graph Partitioning?
**Why frame redistricting as graph partitioning rather than optimization, simulation, or other approaches?**

- **Answer**: Graph partitioning provides structural objectivity (operates on topology, not politics)
- **Evidence**: Paper B.1 (recursive-bisection) establishes method
- **Justification**: Extends Huntington-Hill precedent to boundary design

### RQ2: Why Recursive Bisection?
**Why hierarchical binary splits rather than flat k-way partitioning or other methods?**

- **Answer**: Temporal stability (+14pt tract retention), hierarchical scaffolding persists across decades
- **Evidence**: Paper B.5 (nway-vs-recursive-general) shows equivalence in compactness but recursive advantage in stability
- **Trade-offs**: N-way slightly faster, but recursive more stable

### RQ3: Why Edge-Weighting?
**Why single-objective edge-weighting rather than multi-constraint optimization?**

- **Answer**: Avoids constraint conflicts, achieves better results empirically
- **Evidence**: Paper B.3 (multi-vs-edge) shows 18% better compactness, 12% more MM districts
- **Mechanism**: Reformulates multiple objectives as single weighted objective

### RQ4: Why METIS?
**Why METIS specifically rather than other graph partitioning libraries?**

- **Answer**: Multilevel coarsening, near-linear complexity O(n log k), population balance guarantees
- **Evidence**: Paper B.2 (edge-weighted-bisection) demonstrates +56% compactness improvement
- **Alternatives**: Considered SCOTCH, KaHIP, but METIS most mature for constrained partitioning

### RQ5: Why Census Tracts?
**Why census tracts rather than blocks, block groups, counties, or other units?**

- **Answer**: Balance between resolution (tractable computation) and granularity (captures neighborhoods)
- **Trade-offs**: Blocks too fine (85M units, intractable), counties too coarse (3K units, inflexible)
- **MAUP awareness**: Acknowledge modifiable areal unit problem, validate robustness

### RQ6: Parameter Robustness
**How sensitive are outcomes to parameter choices (α, tree structure, random seeds)?**

- **Answer**: With edge-weighting α≥5, outcomes robust (tree structure accounts for only 3% variance)
- **Evidence**: Paper B.4 (adaptive-bisection) demonstrates parameter insensitivity

---

## Proposed Structure

### Abstract (150 words)
- Problem: Congressional redistricting lacks objective methodology
- Solution: Graph partitioning via recursive bisection with edge-weighting
- Key decisions: Why recursive, why edge-weighting, why METIS, why census tracts
- Findings: Statistical equivalence with n-way (compactness), temporal stability advantage (recursive), constraint conflict resolution (edge-weighting)
- Contribution: Comprehensive justification of algorithmic architecture

### Section 1: Introduction (1,000 words)

#### 1.1: The Redistricting Problem
- 435 districts, 50 states, every 10 years
- Partisan gerrymandering undermines legitimacy
- Need for structural objectivity (not just better commissions)

#### 1.2: Graph Partitioning Paradigm
- Redistricting = partitioning census tracts into k districts
- Tracts = nodes (~4K residents each), adjacency = edges
- Objective: Minimize edge cuts (compactness) subject to population balance

#### 1.3: Paper Scope
This paper explains **how we designed the algorithm**:
- RQ1: Why graph partitioning?
- RQ2: Why recursive bisection?
- RQ3: Why edge-weighting?
- RQ4: Why METIS?
- RQ5: Why census tracts?
- RQ6: Parameter robustness

**Roadmap**: Section 2 (recursive bisection), Section 3 (edge-weighting), Section 4 (METIS), Section 5 (census tracts), Section 6 (parameter robustness), Section 7 (architectural trade-offs)

### Section 2: Recursive Bisection Method (1,200 words)

#### 2.1: Algorithm Overview
**Recursive bisection** = hierarchical binary splits:
1. Start with full state graph (all tracts)
2. Split into 2 regions (North/South or East/West)
3. Recursively split each region until k districts created
4. Refinement: METIS polishes boundaries

**Complexity**: O(n log k) where n = tracts, k = districts

#### 2.2: Why Hierarchical?
**Advantages**:
- **Temporal stability**: Hierarchy persists across census decades (+14pt tract retention vs n-way)
- **Geographic scaffolding**: Binary splits create natural regional structure
- **Computational efficiency**: Log₂(k) splits vs k-way optimization

**Disadvantages**:
- Potentially suboptimal (early splits constrain later ones)
- But: Edge-weighting compensates (Paper B.4 shows tree structure doesn't matter with α≥5)

#### 2.3: Comparison with N-Way Partitioning
**N-way (METIS PartKway)**:
- Directly partitions into k districts (one optimization)
- Globally optimal (all districts optimized simultaneously)
- But: Temporally unstable (66% tract retention vs 80% recursive)

**Recursive (METIS PartRecursive)**:
- Log₂(k) binary optimizations (hierarchical)
- Temporally stable (hierarchical scaffolding)
- Statistically equivalent compactness (Paper B.5: 0.463 vs 0.459, p=0.08)

**Decision**: Use recursive for temporal stability, knowing compactness equivalence

**Evidence from Paper B.5** (nway-vs-recursive-general):
- Compactness: No significant difference (p=0.08)
- Stability: +14pt advantage (recursive)
- Runtime: N-way 1.5× faster, but both under 30 seconds per state
- Conclusion: Recursive preferred for temporal stability

### Section 3: Edge-Weighting for Compactness (1,500 words)

#### 3.1: The Edge-Cut Objective
Standard graph bisection minimizes **edge cuts** = number of edges between partitions.

**Problem**: All edges weighted equally → irregular boundaries acceptable

**Solution**: Weight edges by geographic/demographic similarity → expensive to cut similar tracts → compact districts

#### 3.2: Edge-Weighting Formula
For tracts i and j with shared boundary:
```
w_ij = α × similarity(i, j) + 1
```

**Similarity metrics**:
1. **Geographic**: Shared boundary length / perimeter
2. **Demographic**: For VRA, weight minority-minority edges higher
3. **Scaling factor α**: Controls strength (α=1 neutral, α=50 strong)

#### 3.3: Why Single-Objective?
**Multi-constraint approach**:
```
Minimize: edge cuts
Subject to: population balance, compactness threshold, VRA compliance
```

**Problem**: Constraint conflicts → tightening one violates others

**Evidence from Paper B.3** (multi-vs-edge):
- Edge-weighting: 18% better compactness, 12% more MM districts
- Multi-constraint: Constraint conflicts force suboptimal solutions
- Mechanism: Order of constraint application matters (non-commutative)

**Single-objective reformulation**:
```
Minimize: weighted edge cuts (embeds all objectives in weights)
Subject to: population balance only
```

**Advantages**:
- No constraint conflicts
- METIS optimizes single objective efficiently
- All trade-offs explicit in edge weights (user controls α, β)

#### 3.4: Compactness Improvement
**From Paper B.2** (edge-weighted-bisection):
- Unweighted (α=1): Mean PP = 0.25
- Edge-weighted (α=5): Mean PP = 0.39 (+56% improvement)
- Edge-weighted (α=50): Mean PP = 0.48 (+92% improvement)

**Comparison with enacted plans**:
- Enacted 2020: Mean PP = 0.32
- Algorithmic (α=5): Mean PP = 0.39 (+22% improvement)

### Section 4: METIS Library Selection (1,000 words)

#### 4.1: Why METIS?
**METIS** (Karypis & Kumar, 1998):
- Multilevel paradigm: Coarsen → Partition → Refine
- Population balance guarantees (±0.5% achievable)
- Near-linear complexity: O(n log k)
- Mature, widely-used in HPC (supercomputer load balancing)

#### 4.2: Multilevel Paradigm
**Coarsening phase**:
1. Collapse similar tracts into super-nodes (iteratively)
2. Create hierarchy: fine graph → coarse graph → very coarse graph

**Partitioning phase**:
1. Partition coarsest graph (fast, few nodes)
2. Initial solution cheap to compute

**Refinement phase**:
1. Uncoarsen: project partition to finer graph
2. Refine boundaries (local moves improve cut)
3. Repeat until original graph

**Why multilevel?**:
- Direct partitioning: O(n² k) (intractable for 85K tracts)
- Multilevel: O(n log k) (30 seconds per state)

#### 4.3: Alternatives Considered
**SCOTCH** (Pellegrini, 2012):
- Similar multilevel approach
- Focus on mesh partitioning (engineering applications)
- Less mature for constrained balancing (population equality)

**KaHIP** (Sanders & Schulz, 2013):
- k-way partitioning focus
- Better for unconstrained problems
- Harder to enforce strict population balance

**Chaco** (Hendrickson & Leland, 1995):
- Older library, less maintained
- Recursive bisection only (no n-way)

**Decision**: METIS for maturity, population balance support, both recursive and n-way modes

### Section 5: Census Tracts as Units (1,000 words)

#### 5.1: Resolution Trade-offs
**Census hierarchy**:
- **States**: 50 (too coarse)
- **Counties**: 3,143 (still coarse, ~140K residents each)
- **Census tracts**: ~85,000 (target: 4K residents)
- **Block groups**: ~220,000 (finer, ~1.5K residents)
- **Blocks**: ~11 million (too fine, ~30 residents)

#### 5.2: Why Tracts?
**Advantages**:
1. **Computational tractability**: 85K tracts processable in O(n log k) time
2. **Neighborhood coherence**: Tracts designed to be homogeneous (demographics, socioeconomics)
3. **Census stability**: Tract boundaries persist across decades (easier temporal comparison)
4. **Data availability**: All census data aggregated at tract level

**Disadvantages**:
1. **MAUP sensitivity**: Results depend on tract definitions (acknowledged, tested in Paper C.1)
2. **Coarser than blocks**: Can't capture block-level variation (but blocks intractable: 11M units)

#### 5.3: Block Groups and Blocks
**Block groups** (220K units):
- Finer resolution (~1.5K residents)
- Runtime: 2-3× slower (still tractable)
- Validation: Paper C.1 (maup-sensitivity) tests block groups vs tracts
- Finding: Results statistically equivalent (resolution doesn't materially change outcomes)

**Blocks** (11M units):
- Too fine (~30 residents each)
- Runtime: Intractable (hours per state, memory issues)
- Over-fitting: Block-level variation noise, not signal

#### 5.4: MAUP Awareness
**Modifiable Areal Unit Problem** (MAUP):
- Results depend on how units are defined
- Tract boundaries arbitrary (Census Bureau choices)

**Validation** (Paper C.1 - maup-sensitivity):
- Rerun algorithm at tract, block group, block resolution
- Compare outcomes (compactness, MM districts, partisan patterns)
- Finding: Robust to resolution (correlation >0.9 across resolutions)

**Conclusion**: Census tracts appropriate balance (tractability, neighborhood coherence, data availability)

### Section 6: Parameter Robustness (800 words)

#### 6.1: Parameter Sensitivity
**Key parameters**:
1. **Edge weight factor (α)**: Controls compactness strength
2. **Tree structure**: Order of binary splits (North-South vs East-West first)
3. **Random seed**: METIS uses randomness in coarsening/refinement

#### 6.2: Edge Weight Sensitivity
**From Paper B.2** (edge-weighted-bisection):
- α = 1 (neutral): PP = 0.25
- α = 5: PP = 0.39 (sweet spot)
- α = 10: PP = 0.45
- α = 50: PP = 0.48 (diminishing returns)

**Recommendation**: α = 5-10 (good compactness, computationally efficient)

#### 6.3: Tree Structure Irrelevance
**From Paper B.4** (adaptive-bisection):
- Without edge-weighting (α=1): Tree structure accounts for 24% outcome variance (arbitrary)
- With edge-weighting (α≥5): Tree structure accounts for 3% variance (negligible)
- Mechanism: Edge weights guide optimization → convergence to similar solution regardless of tree

**Implication**: Random tree initialization sufficient (no need for adaptive strategies)

#### 6.4: Random Seed Robustness
**Variance across 10 random seeds**:
- Compactness (PP): Variance = 0.0009 (standard deviation ~0.03)
- MM districts: Variance = 1-2 districts per state
- Partisan patterns: Correlation >0.98 across seeds

**Conclusion**: Outcomes robust to random initialization

### Section 7: Architectural Trade-offs (1,000 words)

#### 7.1: Recursive vs N-Way
**Summary from Paper B.5** (nway-vs-recursive-general):

| Metric | Recursive | N-Way | Winner |
|--------|-----------|-------|--------|
| Compactness (PP) | 0.459 | 0.463 | Tie (p=0.08) |
| Temporal stability | 80% retention | 66% retention | Recursive (+14pt) |
| Runtime | 18.4s | 12.7s | N-Way (1.4× faster) |
| Population balance | 0.21% dev | 0.18% dev | Tie |

**Decision**: Use recursive for temporal stability (primary goal for decadal redistricting)

#### 7.2: Edge-Weighting vs Multi-Constraint
**Summary from Paper B.3** (multi-vs-edge):

| Approach | Compactness | MM Districts | Constraint Conflicts |
|----------|-------------|--------------|----------------------|
| Multi-constraint | PP = 0.34 | 125 | Yes (order matters) |
| Edge-weighting | PP = 0.39 | 137 | No (single objective) |

**Decision**: Use edge-weighting (better results, no conflicts)

#### 7.3: METIS vs Alternatives
**METIS advantages**:
- Maturity (25+ years in HPC)
- Population balance support
- Both recursive and n-way modes

**Alternatives not chosen**:
- SCOTCH: Less mature for constrained balancing
- KaHIP: Harder to enforce strict population equality
- Chaco: Older, less maintained

#### 7.4: Census Tracts vs Other Resolutions
**Tract advantages**:
- Computational tractability (85K units)
- Neighborhood coherence
- Census stability

**Block groups** (220K):
- Finer resolution, but 2-3× slower
- Paper C.1 shows statistical equivalence

**Blocks** (11M):
- Intractable (hours per state)
- Over-fitting (noise, not signal)

### Section 8: Conclusion (400 words)
- Summary: Algorithm design reflects trade-offs (stability > speed, simplicity > optimization complexity)
- Key decisions justified: Recursive (stability), edge-weighting (constraint-free), METIS (mature), tracts (tractable)
- Evidence: Papers B.1-B.5 provide detailed analyses
- Integration: Algorithm design enables validation (Track C), VRA compliance (Track D), experimentation (Track E)

---

## Figures (6 total)

**Figure 1: Recursive Bisection Tree**
- Diagram showing hierarchical splits for 7-district state
- Round 1: 1→2 regions, Round 2: 2→4 regions, Round 3: 4→7 districts
- Shows hierarchical scaffolding

**Figure 2: Edge-Weighting Mechanism**
- Panel A: Unweighted (all edges cost 1)
- Panel B: Edge-weighted (similar tracts expensive to separate)
- Panel C: Resulting districts (compact)

**Figure 3: Compactness Improvement**
- X-axis: Edge weight factor α
- Y-axis: Polsby-Popper compactness
- Shows: Diminishing returns above α=10

**Figure 4: METIS Multilevel Paradigm**
- Three panels: Coarsen → Partition → Refine
- Shows: Hierarchy from fine (85K tracts) to coarse (hundreds of super-nodes)

**Figure 5: Resolution Trade-offs**
- Bar chart: Runtime and memory by resolution (blocks, block groups, tracts, counties)
- Shows: Tracts = sweet spot (tractable, fine enough)

**Figure 6: Architectural Comparison**
- Panel A: Recursive vs N-Way (compactness tie, stability win for recursive)
- Panel B: Edge-weighting vs Multi-constraint (edge wins on both metrics)
- Panel C: Parameter sensitivity (tree structure irrelevant with α≥5)

---

## Target Venues

### Option 1: ACM Transactions on Spatial Algorithms and Systems (TSAS)
**Why TSAS?**
- Graph partitioning algorithms focus
- Spatial applications (redistricting)
- Methodological justification fits
- Format: 8,000-10,000 words

**Fit**: Algorithmic design paper for spatial problem

### Option 2: SIAM Journal on Scientific Computing
**Why SIAM?**
- Computational methods focus
- METIS community (Karypis published here)
- Rigorous technical detail expected
- Format: Technical, detailed

**Fit**: Method selection and architectural comparison

### Option 3: ACM Computing Surveys
**Why ACM Surveys?**
- Comprehensive methodology papers
- Survey of design choices
- High impact factor
- Format: 10,000-15,000 words

**Fit**: Survey of algorithmic redistricting methods

**Recommendation**: Submit to **ACM TSAS first** (best fit for spatial graph partitioning).

---

## Data Requirements

**Already Available**:
- All algorithms implemented (Papers B.1, B.2, B.3, B.4, B.5)
- Census tract data (2000/2010/2020)
- METIS 5.1.0 library
- Comparison results (recursive vs n-way, edge vs multi-constraint)

**Need to Generate**:
- Architectural comparison tables (synthesize from Papers B.3, B.5)
- Parameter sensitivity plots (from Paper B.4)
- Resolution trade-off analysis (from Paper C.1)

**Estimated Processing**: 2-3 days (re-run key comparisons for figures)

---

## Implementation Timeline

### Phase 1: Literature Review (3-4 days)
- Review graph partitioning literature (Karypis, Hendrickson, Pellegrini)
- Review redistricting literature (Duchin, Chen & Rodden, Cho & Liu)
- Position algorithm design choices

### Phase 2: Synthesize Evidence from Papers B.1-B.5 (1 week)
- Extract key findings from each sub-paper
- Build coherent narrative across papers
- Identify gaps in justification

### Phase 3: Writing (1.5 weeks)
- Draft all 8 sections
- Generate 6 figures (synthesize from existing papers)
- Statistical comparisons and tables
- Discussion of trade-offs

### Phase 4: Review (3-4 days)
- Internal review (algorithm experts, political scientists)
- Revise based on feedback

### Phase 5: Submission (2-3 days)
- Format for ACM TSAS
- Cover letter emphasizing comprehensive design justification
- Submit

**Total: 2-3 weeks**

---

## Success Criteria

- [ ] All 6 research questions answered with evidence
- [ ] Papers B.1-B.5 synthesized into cohesive narrative
- [ ] All major design decisions justified (recursive, edge-weighting, METIS, tracts)
- [ ] Architectural trade-offs quantified
- [ ] All 6 figures generated
- [ ] Draft complete (8,000-10,000 words)
- [ ] Submitted to ACM TSAS

---

## Related Work Integration

**From Paper B.1 (recursive-bisection)**:
- Establishes core method
- Huntington-Hill precedent

**From Paper B.2 (edge-weighted-bisection)**:
- +56% compactness improvement
- Edge-weighting formula

**From Paper B.3 (multi-vs-edge)**:
- Why single-objective wins
- Constraint conflict theory

**From Paper B.4 (adaptive-bisection)**:
- Parameter robustness
- Tree structure irrelevance

**From Paper B.5 (nway-vs-recursive-general)**:
- Architectural comparison (recursive vs n-way)
- Statistical equivalence in compactness
- Temporal stability advantage

**Extension**:
- Synthesizes all design decisions into comprehensive justification
- Serves as Track B head paper (read this to understand algorithm design)

---

**Created**: 2026-02-08
**Panel Reference**: Track B Head Paper
**Related Papers**: B.1 (recursive), B.2 (edge), B.3 (multi-vs-edge), B.4 (adaptive), B.5 (nway-general)
**Risk Level**: Low (synthesizing existing work, no new empirical claims)
**Scientific Value**: High (provides comprehensive justification of algorithmic architecture)
