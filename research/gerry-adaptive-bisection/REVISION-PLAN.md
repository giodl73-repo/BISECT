# Revision Plan: Adaptive Bisection Paper

**Paper**: Edge-Weighting Makes Method Selection Irrelevant
**Date**: 2026-02-08
**Round**: 1
**Review Scores**: 3.0/4 average (all reviewers 3/4)
**Status**: Revision stage - P1 items must be addressed before Round 2

---

## Executive Summary

All five reviewers find the core finding (method equivalence with α=5 edge-weighting) to be novel, surprising, and important. Experimental work praised as rigorous. However, **three P1 blocking issues** must be resolved:

1. **Parameter generalization**: Only α=5 tested; need ablation study
2. **Theory formalization**: Intuitive framework needs theorems/proofs
3. **Computational complexity**: Explain why adaptive is 6-15× slower

With P1 items addressed, paper acceptable for computational science venues (SIAM, INFORMS, ACM).

**Estimated revision time**: 3-4 weeks

---

## P1: Blocking Issues (MUST ADDRESS)

### P1.1: Parameter Generalization - α Ablation Study Required

**Status**: ⏳ **PENDING**

**Reviewers**: All 5 (Karypis M1, Hendrickson M2, Duchin M2, Dwork M3, Teng M2)

**Problem**: All experiments use fixed α=5, τ=0.40. No evidence that method equivalence generalizes to other parameter settings.

**Impact**:
- Limits applicability (practitioners use different parameters)
- Theoretical claim α_crit ∈ [3,5] is unvalidated
- Could appear as "cherry-picked" parameter choice

**Required Work**:

1. **α sweep experiment**:
   - Test α ∈ {1, 2, 3, 4, 5, 7, 10, 20, 50, 100}
   - All 5 states × 3 methods × 10 α values = 150 runs
   - Estimated runtime: ~8 hours (parallelizable to 2-3 hours)

2. **Threshold sensitivity**:
   - Test τ ∈ {0.40, 0.45, 0.50} for α=5
   - 5 states × 3 methods × 3 τ values = 45 runs
   - Estimated runtime: ~2 hours

3. **Analysis and visualization**:
   - Plot variance(max_minority_pct) vs. α
   - Identify α_crit where variance drops to zero
   - Validate theoretical prediction α_crit ∈ [3,5]
   - Create phase transition plot (Figure 7)

**Expected Result**:
- High variance for α ≤ 2 (method matters)
- Sharp transition around α ∈ [3,5]
- Zero variance for α ≥ 5 (method irrelevant)

**Acceptance Criterion**: Must demonstrate method equivalence holds for **range** of α values, minimum α ∈ {3, 5, 7, 10}.

**Files to Update**:
- Create `scripts/test_alpha_ablation.py`
- Add results to `results/alpha_ablation_study.csv`
- Add Figure 7: Variance vs. α phase transition plot
- Update Section 6: Results with α sensitivity analysis
- Update Section 4: Theory with empirically validated α_crit

**Time Estimate**: 1 week (3 days experiments, 2 days analysis, 2 days writing)

---

### P1.2: Theoretical Framework Formalization

**Status**: ⏳ **PENDING**

**Reviewers**: Karypis M2, Hendrickson M1, Teng M1

**Problem**: Section 4 provides intuitive explanation but lacks mathematical rigor. No formal theorems, proofs, or characterization of convergence conditions.

**Specific Gaps**:
1. No formal definition of "strong signal" or convergence conditions
2. Phase transition analysis (α_crit) is hand-wavy
3. No proof that METIS finds same solution for all tree structures
4. Missing characterization of when edge-weighting creates unique global minimum

**Required Work**:

1. **Add Theorem 1: Method Convergence Conditions**
   ```
   For graph G=(V,E) with edge weights w(e) = α for e ∈ E_minority, w(e) = 1 otherwise,
   and balance constraint |W_i - W_total/k| ≤ ε:

   If α ≥ α_crit(G, ε, k), then all partitioning algorithms A satisfying constraints
   produce partitions with Obj(P_A) = Obj* ± δ, where δ = O(1/α).

   Furthermore, if α ≫ α_crit and optimal partition is unique, then P_A1 = P_A2
   for all algorithms A1, A2.
   ```

   **Proof sketch**: For α ≫ 1, cutting any minority edge costs ≥ α while cutting regular edges costs 1. Any partition violating minority concentration must cut Ω(√k) minority edges, incurring cost Ω(α√k) ≫ k. Thus all algorithms avoid minority cuts, leading to unique partition.

2. **Add Theorem 2: Phase Transition Characterization**
   ```
   Define variance V(α) = Var_{algorithms A} [Obj(P_A)].

   There exists sharp transition: V(α) = Θ(1) for α < α_crit and V(α) = O(1/α²)
   for α > α_crit, with transition at α_crit ≈ k / m_state.
   ```

   **Derivation**: α_crit must be large enough that cost of cutting minority edges (α) exceeds benefit of improved population balance (O(k)). Thus α_crit ~ k / m_state where m_state is minority fraction.

3. **Add spectral analysis** (optional but recommended):
   - Analyze Fiedler vector of weighted Laplacian L_w
   - Show second eigenvalue λ_2(L_w) scales as O(α)
   - Prove optimal partition determined by sign pattern of Fiedler vector

**Acceptance Criterion**: Must have at least **one formal theorem** with proof sketch. Full proofs can be in appendix.

**Files to Update**:
- Section 4.1: Add Theorem 1 with proof sketch
- Section 4.2: Add Theorem 2 with derivation
- Section 4.3: Add spectral analysis (optional)
- Update abstract and introduction with formalized claims

**Time Estimate**: 1 week (3 days theory development, 2 days writing, 2 days review/polish)

---

### P1.3: Computational Complexity Analysis

**Status**: ⏳ **PENDING**

**Reviewers**: Karypis M3, Hendrickson M3, Teng m1

**Problem**: Adaptive bisection takes 6-15× longer but produces identical results. Paper reports this but doesn't explain why or analyze complexity implications.

**Required Work**:

1. **Add Section 4.4: Computational Implications**

   **Time complexity analysis**:
   - Predetermined: O(T_METIS(n, k)) - single METIS call
   - Adaptive: O(k × T_METIS(n, k)) - evaluates O(k) candidate first splits
   - N-way: O(T_METIS(n, k)) - single call with larger constant

   Where T_METIS(n, k) is METIS runtime, typically O(n log k) for well-separated clusters.

2. **Early termination criterion**:
   ```
   If first split achieves target minority concentration within tolerance ε,
   skip evaluating remaining k-1 splits. This reduces adaptive to predetermined runtime.
   ```

   **Algorithm modification**:
   ```python
   # After evaluating first split:
   if minority_concentration_achieved(split_0, target, tolerance):
       return split_0  # Skip exploring remaining k-1 splits
   else:
       # Continue evaluating all k splits
       ...
   ```

3. **Complexity-theoretic interpretation**:
   - Graph partitioning is NP-hard in general
   - For α ≫ α_crit, instances appear easy (all algorithms find optimal quickly)
   - Suggests tractable subclass: "strongly weighted graphs"
   - Connect to "easy instances of hard problems" literature

**Acceptance Criterion**: Must add subsection explaining time complexity and proposing early termination to eliminate adaptive overhead.

**Files to Update**:
- Add Section 4.4: Computational Implications
- Update Algorithm 1 pseudocode with early termination option
- Update Discussion Section 7.2 with complexity analysis
- Add complexity notation to all algorithm descriptions

**Time Estimate**: 3-4 days (2 days analysis, 1-2 days writing)

---

## P2: Important Issues (SHOULD ADDRESS)

### P2.1: Missing Spatial Structure Analysis

**Status**: ⏳ **PENDING**

**Reviewers**: Duchin M1, Teng questions

**Problem**: Test states have high minority clustering (Moran's I ~ 0.6-0.8) but paper doesn't analyze spatial autocorrelation.

**Required Work**:
1. Compute Moran's I for all 5 test states
2. Add table showing spatial autocorrelation metrics
3. Discuss relationship between clustering and method equivalence
4. Predict when equivalence breaks down (dispersed minorities)

**Files to Update**:
- Add Section 5.2: Spatial Structure Analysis
- Create Table 2: Spatial Autocorrelation Metrics
- Update Discussion with generalization predictions

**Time Estimate**: 2-3 days

---

### P2.2: Fairness Theory Connections

**Status**: ⏳ **PENDING**

**Reviewers**: Dwork M1, M2

**Problem**: Paper treats as pure partitioning result but overlooks algorithmic fairness connections.

**Required Work**:
1. Add Section 2.4: Fairness Theory Background
   - Connect to individual fairness (Dwork et al. 2012)
   - Define algorithmic determinism as fairness property
2. Add Section 7.4: Fairness Guarantees
   - Property 1: Algorithmic Determinism
   - Property 2: Gaming Resistance
   - Property 3: Transparency-Fairness Equivalence

**Files to Update**:
- Add Section 2.4 in Background
- Add Section 7.4 in Discussion
- Update related work with fairness citations
- Reframe abstract/intro with fairness angle

**Time Estimate**: 2-3 days

---

### P2.3: Compactness and Additional Metrics

**Status**: ⏳ **PENDING**

**Reviewers**: Duchin m1, m2

**Problem**: Missing compactness metrics (Polsby-Popper) and comparison to enacted 2020 plans.

**Required Work**:
1. Compute Polsby-Popper scores for all methods×states
2. Download enacted 2020 plans, compute MM counts
3. Create comparison table: Algorithmic vs. Enacted

**Files to Update**:
- Add Table 3: Compactness Metrics
- Add Table 4: Comparison to Enacted Plans
- Update Results section with new metrics

**Time Estimate**: 2-3 days

---

### P2.4: Smoothed Analysis

**Status**: ⏳ **PENDING**

**Reviewers**: Teng M3

**Problem**: Doesn't analyze robustness to measurement error or perturbations.

**Required Work**:
1. Add Section 4.5: Smoothed Analysis
2. Analyze whether equivalence survives under small random perturbations
3. Test robustness to O(1%) noise in vertex weights

**Files to Update**:
- Add Section 4.5 in Theory
- Run perturbation experiments (optional empirical validation)

**Time Estimate**: 2-3 days

---

## P3: Nice-to-Have Issues (COULD ADDRESS)

### P3.1: Experimental Protocol Details

**Reviewers**: Karypis m1, Hendrickson m1, Teng m3

**Improvements**:
- Report METIS version, command-line flags, random seed
- Clarify single run vs. multiple seeds
- Confirm edge weight symmetry
- Add hardware specifications

**Time Estimate**: 1 day

---

### P3.2: Related Work Gaps

**Reviewers**: Karypis m2, Hendrickson m3

**Missing Citations**:
- Fiduccia-Mattheyses (1982)
- Bui & Jones (1993)
- Pothen, Simon, Liou (1990)
- Hendrickson & Rothberg (1998)
- Mézard & Montanari (2009)

**Time Estimate**: 1 day

---

### P3.3: Writing and Notation

**Reviewers**: Karypis m3, Hendrickson m4, Teng m4

**Improvements**:
- Define Catalan numbers C_k
- Consistent notation (α vs. weight factor)
- Add time complexity to Algorithm 1
- Specify α, τ in table captions

**Time Estimate**: 1 day

---

### P3.4: Additional Visualizations

**Reviewers**: Karypis m4, Duchin, Teng

**Suggestions**:
- Alabama district map
- All k=7 tree structures diagram
- Variance vs. α phase transition plot (P1.1)
- α_crit vs. k scaling plot
- Spatial distribution with edge weights

**Time Estimate**: 2 days

---

## Revision Timeline

### Phase 1: P1 Blocking Issues (2 weeks)

**Week 1: Experiments**
- Days 1-3: Run α ablation study (150 runs)
- Days 4-5: Run τ sensitivity test (45 runs)
- Days 6-7: Generate plots, analyze results

**Week 2: Theory + Complexity**
- Days 1-3: Write Theorem 1 & 2 with proofs
- Days 4-5: Write Section 4.4 (complexity analysis)
- Days 6-7: Integrate into paper, update abstract/intro

### Phase 2: P2 Important Issues (1 week)

**Week 3: Enhancements**
- Days 1-2: Spatial structure analysis (Moran's I)
- Days 3-4: Fairness theory sections (2.4, 7.4)
- Days 5-6: Compactness metrics + enacted plan comparison
- Day 7: Smoothed analysis (optional)

### Phase 3: P3 Polish (3 days)

**Days 1-2**: Protocol details, related work, writing improvements
**Day 3**: Additional visualizations, final polish

### Total: 3-4 weeks

---

## Success Criteria

**Minimum for acceptance** (must complete all P1):
- ✅ P1.1: α ablation showing equivalence for range α ∈ {3,5,7,10}
- ✅ P1.2: At least one formal theorem with proof sketch
- ✅ P1.3: Complexity analysis subsection added

**Strong revision** (P1 + P2):
- ✅ All P1 items
- ✅ Spatial structure analysis
- ✅ Fairness theory connections
- ✅ Compactness metrics + enacted comparison

**Excellent revision** (P1 + P2 + P3):
- ✅ All P1 and P2 items
- ✅ Complete protocol details
- ✅ All related work gaps filled
- ✅ Polished writing and notation
- ✅ Additional visualizations

---

## Round 2 Expectations

After addressing P1 items:
- **Expected scores**: 3.5-4.0/4 (significant improvement from 3.0/4)
- **Reviewers**: Same 5 reviewers re-evaluate
- **Timeline**: 2 weeks for reviews + 1 week for synthesis
- **Gate**: Need avg ≥ 2.5/4, no score < 2/4 to advance to ready

After P1 + P2:
- **Expected scores**: 3.5-4.0/4 (strong paper)
- **Suitable for**: SIAM JSC, INFORMS JOC, ACM TALG
- **Potentially competitive for**: SODA/ESA if theory is very strong

---

## Publication Strategy

**Target venues after revision**:

1. **SIAM Journal on Scientific Computing** (primary target after P1)
   - Excellent fit for algorithmic + empirical work
   - Accepts thorough experimental validation
   - Theory formalization sufficient at proof sketch level

2. **INFORMS Journal on Computing** (backup)
   - Strong fit for optimization + practical application
   - Values comprehensive experiments
   - Less demanding on theory formalization

3. **ACM Transactions on Algorithms** (stretch goal if P1+P2 completed)
   - Requires rigorous theory (formal proofs, not sketches)
   - Values algorithmic insights and complexity analysis
   - Fairness angle strengthens submission

4. **American Journal of Political Science** (if fairness/legal added)
   - Requires P2.2 (fairness theory) and P2.3 (enacted comparison)
   - Emphasize legal defensibility and transparency
   - Downplay technical algorithmic details

---

## Next Steps

1. **Author completes P1 revisions** (2 weeks)
2. **Self-review**: Check all P1 items addressed completely
3. **Submit to panel:paper for Round 2 review**
4. **Round 2 reviews** (2 weeks)
5. **Round 2 synthesis** (1 week)
6. **If gate passes**: Advance to ready stage
7. **If gate fails**: Address remaining issues, Round 3

**Current stage**: REVISION
**Next stage gate**: All P1 items marked `addressed: true`
