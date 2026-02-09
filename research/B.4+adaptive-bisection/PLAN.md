# Paper 3: Adaptive Recursive Bisection - Data-Driven Tree Selection for VRA Compliance

## Research Question
Can data-driven tree selection improve recursive bisection's performance for VRA compliance while preserving its transparency advantages?

## Hypothesis
Adaptive tree selection (locally optimal decisions at each split) should significantly outperform predetermined trees but remain inferior to n-way due to constraint by binary tree structure.

## Key Contributions
1. **Algorithm**: Adaptive bisection that evaluates both split orderings at each step
2. **Theoretical Analysis**: Why adaptive helps but can't match n-way
3. **Empirical Validation**: Quantify improvement over predetermined trees
4. **Practical Guidance**: When is adaptive bisection the right choice?

## Data Requirements

### Existing Data (from Paper 2: N-Way vs Recursive)
- Alabama adaptive results: **46.1% max** (vs 43.0% predetermined, 47.3% n-way)
- Location: `research/gerry-nway-vs-recursive/sections/05_results.tex` line 47

### Data Gaps to Fill
Need adaptive recursive bisection results for 4 more states:

1. **Georgia** (k=14, 42.4% minority, target 5 MM)
   - Predetermined trees: Need results for balanced trees
   - Adaptive: Run with edge-weighted approach
   - N-way baseline: From Paper 2

2. **Mississippi** (k=4, 46.1% minority, target 2 MM)
   - Predetermined trees: [3,1], [2,2], [1,3]
   - Adaptive: Run with edge-weighted approach
   - N-way baseline: From Paper 2

3. **Louisiana** (k=6, 41.6% minority, target 2 MM)
   - Predetermined trees: [5,1], [4,2], [3,3], [2,4], [1,5]
   - Adaptive: Run with edge-weighted approach
   - N-way baseline: From Paper 2

4. **South Carolina** (k=7, 35.1% minority, target 3 MM)
   - Predetermined trees: 6 balanced trees
   - Adaptive: Run with edge-weighted approach
   - N-way baseline: From Paper 2

### Metrics to Collect
- Maximum minority percentage per district
- MM districts created (≥50%)
- Edge cut (compactness)
- Runtime
- Tree structure chosen at each split (for adaptive)

## Experiments to Run

### Experiment 1: Predetermined Trees Baseline
```python
# For each state, test all balanced binary trees
# Example for k=7: [6,1], [5,2], [4,3], [3,4], [2,5], [1,6]
partition = recursive_bisection_with_tree(
    state,
    tree_structure=[3, 4],
    method='edge_weighted',
    weight_factor=5,
    minority_threshold=0.40
)
```

### Experiment 2: Adaptive Tree Selection
```python
# At each split, try both orderings and choose best
def adaptive_bisection(graph, k, targets):
    if k == 1:
        return graph

    best_split = None
    best_score = -inf

    for split_size in range(1, k):
        for ordering in [(split_size, k-split_size), (k-split_size, split_size)]:
            partition = bisect_with_targets(graph, ordering, targets)
            score = evaluate_minority_concentration(partition)
            if score > best_score:
                best_score = score
                best_split = (partition, ordering)

    # Recurse on both subgraphs
    left, right = best_split[0]
    return [
        adaptive_bisection(left, best_split[1][0], targets_left),
        adaptive_bisection(right, best_split[1][1], targets_right)
    ]
```

### Experiment 3: Comparison Analysis
- Paired t-tests: adaptive vs predetermined (by state)
- Effect size: Cohen's d for improvement magnitude
- Correlation: Does improvement correlate with k or demographic asymmetry?

## Paper Outline

### 1. Introduction
- Recursive bisection's transparency advantage from Paper 2
- Problem: Predetermined trees may be suboptimal
- Solution: Data-driven tree selection at each split

### 2. Background
- Binary tree structures for recursive bisection
- Tree selection impact on final districts
- Cite Paper 2 findings on tree structure dependency

### 3. Adaptive Algorithm
- Algorithm description with pseudocode
- Complexity analysis: O(k²) splits vs O(k) for predetermined
- Local optimality vs global optimality (n-way)

### 4. Theoretical Analysis
**Why Adaptive Helps:**
- Makes locally optimal decisions rather than arbitrary predetermined structure
- For Alabama [2,5] likely groups MM districts together initially

**Why Adaptive Insufficient:**
- Still constrained by binary tree structure
- Each split creates two subgraphs that cannot exchange vertices in future steps
- N-way avoids this through global refinement

### 5. Experiments
- 5 states × 3 methods (predetermined, adaptive, n-way)
- Metrics: minority concentration, MM count, compactness, runtime

### 6. Results
- Table showing improvement: predetermined → adaptive → n-way
- Statistical significance testing
- Correlation analysis with k and asymmetry

### 7. Discussion
**When to Use Adaptive:**
- N-way unavailable or k very large (>100)
- Want transparency of recursive bisection with better performance
- Middle ground between predetermined trees and full n-way

**Performance-Complexity Tradeoff:**
- Adaptive adds ~20% runtime (tries multiple splits)
- Still much faster than exhaustive tree search
- Quality improvement (3+ points) justifies cost

### 8. Conclusion
- Adaptive improves recursive bisection significantly
- Remains inferior to n-way for VRA compliance
- Best choice when transparency > performance but predetermined insufficient

## Code Pointers

### Implementation
- Edge-weighted bisection: `scripts/pipeline/test_edge_weighting_comprehensive.py`
- METIS wrapper: `src/apportionment/partition/metis_wrapper.py`
- Recursive bisection: `src/apportionment/partition/recursive_bisection.py`

### Data Sources
- Census tracts: `outputs/data/2020/units/{state}/`
- Demographics: `outputs/data/2020/demographics/{state}/`
- Adjacency: `outputs/data/2020/adjacency/{state}/`

### Results from Other Papers
- Multi-constraint baseline: `research/gerry-vra-compliance/results/multi_constraint_results.csv`
- Edge-weighting results: `research/gerry-vra-compliance/results/edge_weighting_ablation_study.csv`
- N-way vs recursive: `research/gerry-nway-vs-recursive/sections/05_results.tex`

## Timeline
1. **Week 1**: Run experiments for all 5 states (predetermined + adaptive)
2. **Week 2**: Statistical analysis, visualization creation
3. **Week 3**: Draft sections 1-4 (intro, background, algorithm, theory)
4. **Week 4**: Draft sections 5-8 (experiments, results, discussion, conclusion)
5. **Week 5**: Revision, LaTeX compilation, final polish

## Success Criteria
- ✅ Complete data for 5 states × 3 methods (predetermined, adaptive, n-way)
- ✅ Statistical significance tests showing adaptive > predetermined
- ✅ Theoretical explanation of why adaptive helps but can't match n-way
- ✅ Practical guidance on method selection
- ✅ Visualizations: heatmap of tree structures, improvement bar chart, correlation plots
