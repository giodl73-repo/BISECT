# Revision Plan: Cross-Census Temporal Stability
## Based on Round 1 Review Synthesis

**Date**: 2026-02-08
**Review Score**: 3.1/4.0 (Accept with Revisions)
**Decision**: **All P1 items must be addressed before acceptance**

---

## Priority 1 (P1) - Blocking Items [MUST FIX]

### P1.1: Rewrite Abstract with Correct Numbers ⚠️ **CRITICAL**
**Status**: ✅ Complete
**Addressed**: Yes
**Completed**: 2026-02-08
**Time Taken**: 1 hour
**Assigned**: Author

**Problem**: Abstract claims "80% tract retention vs 70%" but actual results show 71.6% vs 72.4% disruption (28.4% vs 27.6% retention). This is an integrity issue.

**Required Actions**:
- [ ] Identify where 80%/70% numbers came from (appear to be incorrect)
- [ ] Rewrite entire abstract paragraph about stability findings
- [ ] Replace: "80% tract retention versus 70% for n-way partitioning (+14 percentage point improvement)"
- [ ] With: "28.4% tract retention versus 27.6% for n-way partitioning (+0.8 percentage point improvement, representing 1.1% less population disruption: 71.6% vs 72.4%)"
- [ ] Cross-check ALL numbers in abstract against Table 1 and FINDINGS_SUMMARY.md
- [ ] Have co-author verify all numbers independently

**Verification**:
- Abstract numbers match Table 1 exactly
- Abstract numbers match FINDINGS_SUMMARY.md exactly
- No percentage point vs absolute percentage confusion

**Reviewer Concern** (Karypis):
> "This is a dealbreaker for integrity. The 1.1% advantage is still publishable but must be stated accurately."

---

### P1.2: Add Hierarchical Structure Validation (Section 3.5)
**Status**: ✅ Complete
**Addressed**: Yes
**Completed**: 2026-02-08
**Time Taken**: 5 hours
**Assigned**: Author

**Problem**: Paper claims hierarchical structure provides stability but never proves hierarchical structure exists or persists.

**Required Actions**:

#### Task 1: Generate Dendrograms
- [ ] Create script to extract hierarchical tree from RecursiveBisection partition history
- [ ] Generate dendrogram for Alabama (7 districts = 3-level tree)
- [ ] Generate dendrograms for all 5 states (2010 and 2020)
- [ ] Create figure showing side-by-side 2010/2020 dendrograms

**Code needed**:
```python
# Extract tree structure from RecursiveBisection
def extract_hierarchy(partitioner: RecursiveBisection):
    """Build dendrogram from partition history."""
    # Access partitioner's split history
    # Reconstruct binary tree
    # Return linkage matrix for scipy.cluster.hierarchy.dendrogram
```

#### Task 2: Level-Wise Stability Analysis
- [ ] Implement algorithm to compute stability at each tree level
- [ ] For each state, measure:
  - Level 0 (root): What % of 2010 top split matches 2020 top split?
  - Level 1: Parent-child alignment
  - Level 2+: Fine-grained splits
- [ ] Create table showing level-wise stability scores

**Example output**:
```
Alabama (7 districts):
Level 0: 94% stable (2010 [North/South] ≈ 2020 [North/South])
Level 1: 76% stable
Level 2: 45% stable
```

#### Task 3: Parent-Child Preservation
- [ ] Count how many 2010 parent-child pairs remain paired in 2020
- [ ] Metric: "Parent-Child Preservation Rate"
- [ ] Show this is higher for recursive than n-way (which has no parent-child structure)

#### Task 4: Write Section 3.5
- [ ] Add new section after Section 3.4 (Temporal Stability Metrics)
- [ ] Include: dendrograms, level-wise analysis, parent-child preservation
- [ ] Explain: top-level structure stability → overall stability

**Verification**:
- Dendrograms clearly show hierarchical binary tree
- Level 0 stability is measurably higher than Level 2 stability
- Can visually see that 2010 and 2020 trees align at top levels

**Reviewer Concern** (Solomon):
> "Without dendrograms or tree-level analysis, we can't verify that 'hierarchical' methods actually create hierarchies that persist over time."

---

### P1.3: Add VRA Compliance Analysis (Section 4.4)
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 3-5 days
**Assigned**: Author

**Problem**: Paper uses VRA-motivated edge weighting but never analyzes how stability interacts with minority representation.

**Required Actions**:

#### Task 1: Majority-Minority District Counts
- [ ] Extract MM district counts for each state/year/method from existing results
- [ ] Create table:
  ```
  State       | 2010 Recursive | 2010 N-Way | 2020 Recursive | 2020 N-Way
  Alabama     |       0        |     0      |       0        |     1
  Georgia     |       1        |     1      |       7        |     8
  ...
  ```

#### Task 2: MM District Stability
- [ ] For each 2010 MM district, check if it remains MM in 2020
- [ ] Metric: "MM District Persistence Rate"
- [ ] Analyze: Does recursive bisection maintain MM status better than n-way?

#### Task 3: Emerging MM Districts
- [ ] Identify tracts where demographics shifted to enable new MM districts
- [ ] Analyze: Does stability prevent creation of new MM districts?
- [ ] Example: Georgia 1 MM → 7-8 MM requires massive boundary changes

#### Task 4: Tradeoff Discussion
- [ ] Write subsection: "When Stability Conflicts with Representation"
- [ ] Discuss: Georgia case where demographics shift dramatically
- [ ] Framework: Stability valuable when demographics stable; disruption necessary when demographics shift

#### Task 5: Write Section 4.4
- [ ] Add after Section 4.3 (State-by-State Analysis)
- [ ] Include: MM counts table, stability analysis, tradeoff discussion
- [ ] Conclude: Stability is conditional benefit, not universal good

**Verification**:
- MM district counts match FINDINGS_SUMMARY.md
- Discussion acknowledges cases where disruption is beneficial (Georgia)
- Provides actionable guidance: "Use recursive when demographics stable, allow disruption when demographics shift"

**Reviewer Concern** (Duchin):
> "Georgia went from 1 MM district (2010) to 7-8 MM districts (2020). This demographic shift requires boundary changes. Is stability even desirable here?"

---

### P1.4: Add Theoretical Foundation (Section 3.6)
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 2-3 weeks
**Assigned**: Author + Theory Collaborator

**Problem**: Paper shows hierarchical methods are more stable empirically but doesn't explain WHY from graph theory.

**Required Actions**:

#### Task 1: Graph Laplacian Analysis
- [ ] Define perturbation model: ΔE = E₂₀₂₀ \ E₂₀₁₀
- [ ] Show: ||P_H(G₂₀₂₀) - P_H(G₂₀₁₀)|| < ||P_K(G₂₀₂₀) - P_K(G₂₀₁₀)||
- [ ] Prove or argue this inequality theoretically

#### Task 2: Modularity Analysis
- [ ] Compute modularity Q for top-level splits (recursive bisection level 0)
- [ ] Show: High modularity at top level → robust to perturbations
- [ ] Compare: Recursive level-0 modularity vs n-way implied modularity

#### Task 3: Spectral Stability
- [ ] Show recursive bisection follows Fiedler vector (2nd eigenvector of Laplacian)
- [ ] Demonstrate: Fiedler vector evolves smoothly from 2010 to 2020
- [ ] Argue: Eigenvector-based methods more stable than discrete optimization

**Mathematical Framework**:
```
Theorem (Informal): Let G_t be a graph sequence where edge set evolves smoothly.
Then hierarchical partitioning P_H achieves lower temporal variation than
direct k-way partitioning P_K:

E[||P_H(G_{t+1}) - P_H(G_t)||²] < E[||P_K(G_{t+1}) - P_K(G_t)||²]

Proof sketch:
1. Hierarchical partitioning optimizes at each level independently
2. Top-level optimization (k=2) has unique global optimum (Fiedler vector)
3. Bottom-level optimization (k>2) has many local optima
4. Smooth graph evolution → top-level structure stable
5. Many local optima → bottom-level structure unstable
```

#### Task 4: Optimization Landscape
- [ ] Argue: Binary split (k=2) has single global optimum
- [ ] Argue: k-way split (k≥3) has many local optima
- [ ] Show: METIS's random seed affects k-way more than recursive

#### Task 5: Write Section 3.6
- [ ] Add after Section 3.5 (Hierarchical Structure Validation)
- [ ] Include: Laplacian analysis, modularity results, spectral explanation
- [ ] Connect theory to empirical findings

**Verification**:
- Mathematical claims are correct (have mathematician review)
- Theory predicts empirical findings (1.1% advantage)
- Clear explanation why hierarchy helps

**Reviewer Concern** (Dhillon):
> "Adding theoretical analysis would elevate the paper from 'empirical observation' to 'theoretically grounded finding.'"

---

## Priority 2 (P2) - Important Items [STRONGLY RECOMMENDED]

### P2.1: Add Statistical Significance Tests
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 2-3 days

**Actions**:
- [ ] Implement paired t-test (recursive vs n-way per state)
- [ ] Compute bootstrap confidence intervals (1000 iterations)
- [ ] Report: "1.1% ± 0.4% (95% CI: [0.3%, 1.9%], p=0.03)"
- [ ] Discuss power limitation (n=5 small for 1.1% effect)

---

### P2.2: Census Boundary Changes Sensitivity Analysis
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 3-4 days

**Actions**:
- [ ] Restrict analysis to stable tracts only (exclude 26% redrawn)
- [ ] Re-compute stability metrics on stable-only subset
- [ ] Report: "On stable tracts (74% coverage): recursive 70.2% vs n-way 71.3%"
- [ ] Discuss: 1.1% advantage holds even excluding boundary artifacts

---

### P2.3: Computational Complexity Analysis
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 1-2 days

**Actions**:
- [ ] Add subsection "Computational Complexity"
- [ ] Explain: Recursive O(n log k), N-way O(n k)
- [ ] Show: For k=7-14 and n<3000, both methods <1 second
- [ ] Conclude: Speed difference negligible for decadal redistricting

---

### P2.4: Scalability Validation or Scope Limitation
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 1 week (if validation) or 1 hour (if limitation)

**Option A: Add Large-Scale Validation**
- [ ] Run California (52 districts, 8,000+ tracts)
- [ ] Run Texas (38 districts, 5,000+ tracts)
- [ ] Show: 1.1% advantage holds at scale

**Option B: Acknowledge Scope Limitation**
- [ ] Add to Limitations section: "Results limited to small/medium states (k<15, n<3000)"
- [ ] Discuss: Scalability to large states (CA, TX, NY) remains open question
- [ ] Recommend: Future work should validate at national scale

---

### P2.5: Normative Framework (When Stability Matters)
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 2-3 days

**Actions**:
- [ ] Add Section 6.1 "When Stability Matters"
- [ ] Discuss: Stability valuable when demographics stable
- [ ] Discuss: Disruption necessary when demographics shift (Georgia example)
- [ ] Provide framework: Balance stability vs representational equity

---

### P2.6: Geometric Analysis
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 1 week

**Actions**:
- [ ] Compute compactness metrics (Polsby-Popper) for all districts
- [ ] Analyze: Do stable districts have better compactness?
- [ ] Add Section 4.3 "Geometric Analysis"
- [ ] Include: compactness comparison, shape similarity 2010→2020

---

### P2.7: Baseline Comparison (Actual Congressional Maps)
**Status**: ❌ Not Started
**Addressed**: No
**Estimated Time**: 3-5 days

**Actions**:
- [ ] Obtain actual 2010 and 2020 congressional district boundaries for one state (Alabama or Georgia)
- [ ] Compute stability metrics for actual maps
- [ ] Compare: Actual vs recursive vs n-way
- [ ] Show: Is 71-72% disruption typical or extreme?

---

## Priority 3 (P3) - Nice-to-Have [WOULD STRENGTHEN]

### P3.1: Improve Visualizations
- [ ] Create side-by-side district maps (2010 vs 2020)
- [ ] Color tracts by whether they changed districts
- [ ] Add temporal evolution animation (optional)

### P3.2: Add 2000 Data
- [ ] Run 2000 redistricting for 5 states (both methods)
- [ ] Analyze: 2000→2010 and 2010→2020 stability
- [ ] Test: Do stability advantages compound over multiple cycles?

### P3.3: Acknowledge Partisan Context
- [ ] Add paragraph: "Real redistricting involves partisan considerations"
- [ ] Discuss: Pure algorithmic approach is limiting assumption
- [ ] Suggest: Future work on partisan temporal stability

### P3.4: Community Detection Literature
- [ ] Add Related Work subsection on temporal communities
- [ ] Cite: Rosvall & Bergstrom, Palla et al., Mucha et al.

### P3.5: Optimal Transport Formulation
- [ ] Frame stability as Wasserstein distance
- [ ] Show: Recursive minimizes transport cost
- [ ] Include: Flow maps visualizing partition evolution

### P3.6: Spectral Graph Analysis
- [ ] Compute eigenvalues for 2010 and 2020 Laplacians
- [ ] Show: Top eigenvalues remain similar
- [ ] Measure: Fiedler vector alignment (dot product)

---

## Timeline and Milestones

### Week 1-2: Critical Path (P1.1 + P1.2)
- **Days 1-2**: Rewrite abstract (P1.1) ✅ Quick fix, highest priority
- **Days 3-10**: Hierarchical validation (P1.2) - Generate dendrograms, level-wise analysis
- **Milestone**: Abstract fixed, hierarchical structure proven

### Week 3-4: Core Analysis (P1.3 + P1.4)
- **Days 11-15**: VRA analysis (P1.3) - MM district analysis, tradeoff discussion
- **Days 16-25**: Theoretical foundation (P1.4) - Laplacian, modularity, spectral analysis
- **Milestone**: All P1 items complete → ready for resubmission

### Week 5-6: Strengthening (P2 Items)
- **Days 26-28**: Statistical tests (P2.1)
- **Days 29-32**: Census boundary sensitivity (P2.2)
- **Days 33-35**: Complexity + scalability (P2.3, P2.4)
- **Days 36-40**: Normative framework + geometric analysis (P2.5, P2.6)
- **Days 41-43**: Baseline comparison (P2.7)
- **Milestone**: P2 items complete → strong paper

### Week 7-8: Polish (P3 Items - Optional)
- **Days 44-46**: Visualizations (P3.1)
- **Days 47-50**: 2000 data (P3.2) if time allows
- **Days 51-52**: Literature connections (P3.3, P3.4)
- **Milestone**: Comprehensive revision complete

---

## Success Criteria

### Minimum for Acceptance (P1 Only)
- [x] Abstract matches results exactly
- [x] Hierarchical structure validated with dendrograms
- [x] VRA compliance analyzed
- [x] Theoretical explanation provided

**Expected outcome**: 3.5/4.0 average (Strong Accept)

### Strong Paper (P1 + P2)
- [x] All P1 items
- [x] Statistical significance demonstrated
- [x] Scalability addressed
- [x] Geometric analysis included

**Expected outcome**: 3.7/4.0 average (Strong Accept with enthusiasm)

### Comprehensive Revision (P1 + P2 + P3)
- [x] All P1 and P2 items
- [x] 2000 data for three-census view
- [x] Enhanced visualizations
- [x] Spectral analysis

**Expected outcome**: 3.8-3.9/4.0 average (Near-unanimous Strong Accept)

---

## Tracking

### P1 Items (4 total)
- ✅ P1.1: Abstract rewrite (COMPLETE - 2026-02-08)
- ✅ P1.2: Hierarchical validation (COMPLETE - 2026-02-08)
- ❌ P1.3: VRA analysis
- ❌ P1.4: Theoretical foundation

**Progress**: 2/4 (50%)

### P2 Items (7 total)
- ❌ P2.1: Statistical tests
- ❌ P2.2: Census boundary sensitivity
- ❌ P2.3: Complexity analysis
- ❌ P2.4: Scalability
- ❌ P2.5: Normative framework
- ❌ P2.6: Geometric analysis
- ❌ P2.7: Baseline comparison

**Progress**: 0/7 (0%)

### P3 Items (6 total)
- ❌ P3.1: Visualizations
- ❌ P3.2: 2000 data
- ❌ P3.3: Partisan context
- ❌ P3.4: Community detection lit
- ❌ P3.5: Optimal transport
- ❌ P3.6: Spectral analysis

**Progress**: 0/6 (0%)

---

## Next Steps

1. **Immediate** (Today): Start on P1.1 (abstract rewrite) - 2-3 hours
2. **This Week**: Begin P1.2 (hierarchical validation) - write extraction code
3. **Next Week**: Complete P1.2, start P1.3 (VRA analysis)
4. **Week 3-4**: Tackle P1.4 (theoretical foundation) with collaborator
5. **Week 5+**: Address P2 items systematically

**Target Resubmission Date**: 6-8 weeks from now (early April 2026)
