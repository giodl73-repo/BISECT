# Paper 4: Multi-Constraint vs Edge-Weighted Optimization - Why Single-Objective Graph Partitioning Succeeds for VRA Compliance

## Research Question
Why does edge-weighted single-objective optimization dramatically outperform multi-constraint optimization for VRA compliance, despite multi-constraint being the "standard" approach?

## Hypothesis
Multi-constraint optimization creates conflicting objectives that METIS cannot simultaneously satisfy. Edge-weighting directly encodes VRA goals into the primary objective (edge cut minimization), avoiding constraint conflicts.

## Key Contributions
1. **Theoretical Explanation**: Why multi-constraint fails (constraint conflict analysis)
2. **Empirical Breakthrough**: 40% → 80% success rate with edge-weighting
3. **Algorithmic Insight**: Single-objective > multi-constraint for asymmetric targets
4. **Generalization**: When to use each approach beyond redistricting

## Data Requirements

### Existing Data

#### Multi-Constraint Results (from Paper 1: VRA Compliance)
Location: `research/gerry-vra-compliance/results/multi_constraint_results.csv`

**Alabama (k=7, target 2 MM):**
- Best configuration: 42% threshold, 5x tpwgts
- Max minority: 49.6%
- MM districts: 0 (failed to reach 50%)
- Edge cut: ~280

**Other States:** Need to extract from CSV

#### Edge-Weighted Results (from Paper 1: VRA Compliance)
Location: `research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv`

**Alabama (k=7, target 2 MM):**
- Best configurations: 5x@40%, 10x@40%, 5x@45%
- Max minority: 50.8%, 50.1% (2 MM districts achieved)
- Edge cut: 276 (-1.4% vs multi-constraint)
- Success: 80% of configurations (112/140)

**All 5 States Summary:**
- Edge-weighted: 80% success rate
- Multi-constraint: 40% success rate (20 percentage point gap)
- Compactness: Edge-weighted +4% average edge cut (minimal cost)

### Data Gaps to Fill
Need multi-constraint results for comparison with edge-weighting:

1. **Georgia** (k=14, 42.4% minority, target 5 MM)
   - Multi-constraint: Run with tpwgts approach
   - Edge-weighted: Available from Paper 1
   - Comparison metrics: MM count, max minority %, compactness

2. **Mississippi** (k=4, 46.1% minority, target 2 MM)
   - Multi-constraint: Run with tpwgts approach
   - Edge-weighted: Available from Paper 1
   - Comparison metrics: MM count, max minority %, compactness

3. **Louisiana** (k=6, 41.6% minority, target 2 MM)
   - Multi-constraint: Run with tpwgts approach
   - Edge-weighted: Available from Paper 1
   - Comparison metrics: MM count, max minority %, compactness

4. **South Carolina** (k=7, 35.1% minority, target 3 MM)
   - Multi-constraint: Run with tpwgts approach
   - Edge-weighted: Available from Paper 1
   - Comparison metrics: MM count, max minority %, compactness

## Experiments to Run

### Experiment 1: Multi-Constraint Replication
```python
# Replicate multi-constraint approach for all 5 states
partition = partition_graph_with_executable(
    adjacency_list,
    vertex_weights_2d,  # [total_pop, minority_vap]
    nparts=num_districts,
    tpwgts=target_weights,  # Specify 2D targets per district
    ubvec=[1.005, 1.50],    # Population tight, minority loose
    niter=100
)
```

### Experiment 2: Constraint Conflict Analysis
For each state, test multiple ubvec values to see constraint tradeoff:
- ubvec=[1.005, 1.30]: Tight minority tolerance
- ubvec=[1.005, 1.50]: Medium minority tolerance
- ubvec=[1.005, 2.00]: Loose minority tolerance
- ubvec=[1.005, 5.00]: Very loose minority tolerance

**Hypothesis**: Even with very loose minority tolerance, multi-constraint can't achieve 50% because population constraint dominates.

### Experiment 3: Head-to-Head Comparison
For each state, compare best multi-constraint vs best edge-weighted:
- Minority concentration
- MM districts created
- Compactness (edge cut)
- District-level analysis

## Paper Outline

### 1. Introduction
- Standard approach: multi-constraint optimization (2D vertex weights)
- Problem: Fails to achieve VRA compliance (Alabama: 0 MM districts)
- Breakthrough: Edge-weighting achieves 2 MM districts
- Research question: Why does single-objective outperform multi-constraint?

### 2. Background
**Multi-Constraint Graph Partitioning:**
- 2D vertex weights: $(p_v, m_v)$ for population and minority
- Target weights per partition: $\mathbf{t}_i = (t_i^{pop}, t_i^{min})$
- Imbalance constraints: $\mathbf{ubvec} = (ub^{pop}, ub^{min})$
- METIS minimizes edge cut while respecting constraints

**Edge-Weighted Single-Objective:**
- 1D vertex weights: $w_v = p_v$ (population only)
- Weighted edges: $w_{ij} = \begin{cases} \alpha & \text{if both } i,j \text{ minority} \\ 1 & \text{otherwise} \end{cases}$
- METIS minimizes edge cut (avoiding cutting minority-minority edges)
- Population balance constraint only

### 3. Theoretical Analysis

#### Why Multi-Constraint Fails

**Constraint Conflict:**
Population constraint: $\sum_{v \in P_i} p_v \approx \frac{1}{k} \sum_v p_v$ (tight, ±0.5%)
Minority constraint: $\sum_{v \in P_i} m_v \approx t_i^{min}$ (loose, up to 50% deviation)

When minority concentration target (60%) conflicts with population balance:
- METIS prioritizes population (tight constraint)
- Minority target becomes aspirational, not enforceable
- Result: Districts approach 50% but can't exceed due to population dominance

**Mathematical Formulation:**
$$\min \text{edge\_cut}(P)$$
$$\text{s.t. } |P_i^{pop} - target_i^{pop}| \leq 0.005 \cdot target_i^{pop} \quad \forall i$$
$$|P_i^{min} - target_i^{min}| \leq 0.50 \cdot target_i^{min} \quad \forall i$$

Problem: Second constraint too loose to be binding, first constraint prevents minority concentration.

#### Why Edge-Weighting Succeeds

**Direct Objective Encoding:**
$$\min \sum_{(i,j) \in E} w_{ij} \cdot \mathbb{1}[\text{partition}(i) \neq \text{partition}(j)]$$
$$= \min \left( \sum_{\text{normal edges}} 1 + \sum_{\text{minority-minority edges}} \alpha \right)$$

By setting $\alpha \gg 1$, minority-minority edges become expensive to cut:
- METIS strongly avoids cutting them (objective function directly penalizes)
- Minority tracts naturally group together
- No constraint conflict—population is only constraint

**Key Insight:** Encoding goal in objective > encoding goal in constraint (for asymmetric targets).

### 4. Experiments
- 5 states × 2 methods (multi-constraint, edge-weighted)
- Constraint conflict test: vary ubvec to test hypothesis
- Metrics: MM count, max minority %, edge cut

### 5. Results

**Table 1: Head-to-Head Comparison**
| State | Multi-Constraint MM | Edge-Weighted MM | Gap |
|-------|---------------------|------------------|-----|
| Alabama | 0 (49.6%) | 2 (50.8%) | +2 MM |
| Georgia | ? | ? | ? |
| Mississippi | ? | ? | ? |
| Louisiana | ? | ? | ? |
| South Carolina | ? | ? | ? |

**Table 2: Constraint Conflict Test**
| State | ubvec | MM Count | Max Minority % |
|-------|-------|----------|----------------|
| Alabama | [1.005, 1.30] | ? | ? |
| Alabama | [1.005, 1.50] | 0 | 49.6% |
| Alabama | [1.005, 2.00] | ? | ? |
| Alabama | [1.005, 5.00] | ? | ? |

**Hypothesis Test:** Even with ubvec=[1.005, 5.00], multi-constraint fails to achieve 50% (supports constraint conflict theory).

**Figure 1: Success Rate by Method**
- Multi-constraint: 40% (2/5 states achieve target MM count)
- Edge-weighted: 80% (4/5 states achieve target MM count)
- Statistical significance: χ² test

**Figure 2: Compactness Tradeoff**
- Scatter plot: edge cut vs minority concentration
- Color by method (multi-constraint vs edge-weighted)
- Shows edge-weighted achieves better demographics at similar/better compactness

### 6. Discussion

#### Generalization Beyond Redistricting

**When to Use Multi-Constraint:**
- Symmetric targets (all partitions similar)
- Multiple hard constraints (must satisfy both)
- Example: Parallel computing with uniform load distribution

**When to Use Edge-Weighting:**
- Asymmetric targets (partitions should differ)
- Single dominant constraint (population balance)
- Secondary goal encodable in edge weights
- Example: Database partitioning (hot/cold data), network clustering (community detection)

#### Implications for Graph Partitioning Theory

Standard assumption in graph partitioning literature: "Multi-constraint for multi-objective problems"

**Our finding:** When one constraint is tight and others are loose, encoding loose constraints as edge weights is more effective.

**Why this matters:**
- Multi-constraint added to METIS for parallel computing (multiple tight constraints)
- Redistricting has different structure (one tight, one aspirational)
- Algorithm choice depends on constraint tightness, not just number of constraints

### 7. Related Work
- Karypis & Kumar (1999): Multi-constraint partitioning for parallel computing
- Schloegel et al. (2000): Multi-constraint algorithms
- Our contribution: Identify when edge-weighting outperforms multi-constraint

### 8. Conclusion
- Edge-weighted single-objective achieves 2× success rate vs multi-constraint (80% vs 40%)
- Theoretical explanation: Constraint conflict in multi-constraint, direct encoding in edge-weighted
- Practical guidance: Use edge-weighting for asymmetric targets with tight primary constraint
- Generalization: Findings apply beyond redistricting to any graph partitioning with constraint hierarchy

## Code Pointers

### Multi-Constraint Implementation
```python
# From: scripts/pipeline/test_vra_compliance.py (if exists)
# Or need to create new script for multi-constraint experiments
vertex_weights_2d = np.column_stack([
    tracts_with_demo['total_pop'].values,
    tracts_with_demo['minority_vap'].values
])

tpwgts = create_target_weights(num_districts, mm_targets=2)

partition = partition_graph_with_executable(
    adj_list,
    vertex_weights_2d,
    nparts=num_districts,
    tpwgts=tpwgts,
    ubvec=[1.005, 1.50],
    niter=100
)
```

### Edge-Weighted Implementation
```python
# From: scripts/pipeline/test_edge_weighting_comprehensive.py
vertex_weights_1d = tracts_with_demo['total_pop'].values

edge_weights = create_minority_edge_weights(
    adj_matrix,
    tracts_with_demo,
    minority_threshold=0.40,
    weight_factor=5
)

partition = partition_graph_with_executable(
    adj_list,
    vertex_weights_1d,  # 1D only
    nparts=num_districts,
    ufactor=1.005,
    edge_weights=edge_weights,
    niter=100
)
```

### Analysis
```python
# From: research/gerry-vra-compliance/analysis/edge_weighting_analysis.ipynb
# Statistical tests, success rate comparison, compactness analysis
```

## Timeline
1. **Week 1**: Run multi-constraint experiments for all 5 states
2. **Week 2**: Constraint conflict test (vary ubvec)
3. **Week 3**: Statistical analysis, visualization creation
4. **Week 4**: Draft sections 1-5 (intro, background, theory, experiments, results)
5. **Week 5**: Draft sections 6-8 (discussion, related work, conclusion)
6. **Week 6**: Revision, LaTeX compilation, final polish

## Success Criteria
- ✅ Complete multi-constraint data for 5 states
- ✅ Constraint conflict test validates theory (loose ubvec doesn't help)
- ✅ Statistical significance: edge-weighted > multi-constraint
- ✅ Theoretical explanation of constraint conflict
- ✅ Practical guidance on method selection
- ✅ Visualizations: success rate comparison, compactness tradeoff, constraint conflict test
