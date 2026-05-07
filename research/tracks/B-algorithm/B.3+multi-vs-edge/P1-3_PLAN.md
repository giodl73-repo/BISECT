# P1-3 Plan: Balanced Experimental Design

**Date**: 2026-02-08
**Status**: 📋 Planning Phase
**Priority**: P1 (Blocking)

---

## Problem Statement

**Reviewer Concern** (Dr. Cynthia Phillips, primary):
> "The experimental comparison is fundamentally unfair: 140 edge-weighted configurations give 140 chances to find a good solution, while 4 multi-constraint configurations give only 4 chances. More configurations = more chances to succeed."

**Current State**:
- **Edge-weighted**: 28 configs/state × 5 states = 140 total
  - Parameter space: 7 weight_factor × 4 threshold = 28 configs
  - Weight factors: {5, 10, 20, 30, 50, 75, 100}
  - Thresholds: {0.30, 0.40, 0.50, 0.60}

- **Multi-constraint**: 4 configs/state × 5 states = 20 total
  - Parameter space: 4 ubvec values
  - ubvec: {1.3, 1.5, 2.0, 5.0}

**Impact**: 7:1 configuration ratio inflates edge-weighted success rate.

---

## Objectives

1. **Balance configuration counts**: Run 28 multi-constraint configs per state (match edge-weighted)
2. **Expand parameter space**: Test more ubvec values and potentially add second dimension
3. **Fair comparison**: Enable apples-to-apples comparison of success rates
4. **Validate robustness**: Test whether multi-constraint can succeed with different parameters

---

## Approach Options

### Option 1: Single-Dimension Expansion (Recommended)
**Parameter**: `ubvec` (minority tolerance)
**Strategy**: Expand from 4 to 28 ubvec values

**Parameter Grid** (28 values):
```python
ubvec_values = [
    # Very tight (may fail due to over-constraint)
    1.10, 1.15, 1.20, 1.25,

    # Tight-to-moderate (current range)
    1.30, 1.35, 1.40, 1.45, 1.50, 1.55, 1.60, 1.70, 1.80, 1.90, 2.00,

    # Moderate-to-loose
    2.25, 2.50, 2.75, 3.00, 3.25, 3.50, 3.75, 4.00,

    # Very loose (may fail due to insufficient guidance)
    4.50, 5.00, 6.00, 7.00, 10.0
]
```

**Total configs**: 28 ubvec × 5 states = 140 (matches edge-weighted)

**Pros**:
- Simple to implement (1 parameter change)
- Direct comparison to existing results
- Tests constraint conflict theory across full tightness spectrum
- Minimal code changes

**Cons**:
- Only explores 1-dimensional parameter space
- Edge-weighted explores 2 dimensions (weight, threshold)

---

### Option 2: Two-Dimension Expansion
**Parameters**: `ubvec` (minority tolerance) + `niter` (refinement iterations)
**Strategy**: 7 ubvec × 4 niter = 28 configs

**Parameter Grid**:
```python
ubvec_values = [1.3, 1.5, 2.0, 3.0, 4.0, 5.0, 7.0]  # 7 values
niter_values = [10, 50, 100, 200]                    # 4 values
```

**Total configs**: 7 × 4 × 5 states = 140

**Pros**:
- Explores 2 dimensions (like edge-weighted)
- Tests whether more refinement helps multi-constraint
- More thorough parameter exploration

**Cons**:
- More complex to analyze
- niter may not significantly impact multi-constraint performance
- Longer runtime (high niter = slower)

---

### Option 3: Hybrid Approach
**Parameters**: Primary ubvec sweep + selective niter test
**Strategy**: 24 ubvec values + 4 niter tests = 28 configs

**Parameter Grid**:
```python
# Primary sweep (24 configs)
ubvec_sweep = [1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0,
               2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 6.0, 7.0, 8.0, 10.0,
               12.0, 15.0, 20.0, 25.0]

# niter sensitivity test (4 configs, using ubvec=2.0)
niter_test = [(2.0, 10), (2.0, 50), (2.0, 100), (2.0, 200)]
```

**Total configs**: 24 + 4 × 5 states = 140

**Pros**:
- Thorough ubvec exploration (primary concern)
- Tests niter sensitivity without full grid
- Flexible analysis (can report separately)

**Cons**:
- Slightly more complex than Option 1
- Requires careful result interpretation

---

## Recommendation: Option 1 (Single-Dimension)

**Rationale**:
1. **Directly addresses reviewer concern**: Balances config counts (140 vs 140)
2. **Tests constraint conflict theory**: Sweeps full tightness spectrum (1.1 to 10.0)
3. **Simple implementation**: Minimal code changes, easy to analyze
4. **Conservative**: Doesn't introduce confounding variables (niter)

**Expected outcome**: Multi-constraint still struggles even with 28 configs, validating that the issue is fundamental constraint conflict, not just unlucky parameter selection.

---

## Implementation Plan

### Phase 1: Script Modification (1 hour)
1. Create `run_multi_constraint_experiments_v2.py` (copy from FIXED version)
2. Expand ubvec grid from 4 to 28 values
3. Update output CSV format (add config_id column)
4. Add progress tracking for 140 experiments

### Phase 2: Experimental Runs (4-6 hours)
- 28 configs × 5 states = 140 METIS runs
- Estimated: 2-3 min per run × 140 = 4.7-7 hours
- States: Alabama, Georgia, Louisiana, Mississippi, South Carolina
- Parameters: k (districts), target (MM count), ubvec (28 values)

### Phase 3: Analysis (2 hours)
1. Combine with existing edge-weighted results (140 configs)
2. Compute success rates (config-level, state-level)
3. Statistical comparison (now fair: 140 vs 140)
4. Parameter sensitivity analysis (ubvec sweep)

### Phase 4: Results Update (2 hours)
1. Update Section 5 with balanced comparison
2. Regenerate Figure 1 (success rates)
3. Regenerate Figure 5 (parameter sensitivity)
4. Update all text references

### Phase 5: Documentation (1 hour)
1. Update RESPONSE_LETTER.md with P1-3 resolution
2. Update _panel.yaml
3. Create P1-3_COMPLETE.md

**Total estimated time**: 10-12 hours (1.5 days)

---

## Expected Results

### Hypothesis
Multi-constraint will **still underperform** edge-weighted even with 28 configs:
- Current: 30.0% (6/20) vs 47.9% (67/140) → 17.9 pp gap
- Predicted: ~35-40% (49-56/140) vs 47.9% (67/140) → 8-13 pp gap

**Why?** Constraint conflict is fundamental, not a parameter selection artifact. Even with thorough ubvec sweep, multi-constraint faces:
1. Tight population constraint (±0.5%) dominating local search
2. Loose minority constraint (±30-1000%) providing insufficient guidance
3. Geographic dispersion preventing perfect minority packing

### Best Case Scenario (for our paper)
Multi-constraint improves slightly but still trails edge-weighted:
- Gap narrows from 17.9 pp to ~10 pp
- Validates constraint conflict theory (fundamental limitation)
- Addresses reviewer fairness concern

### Worst Case Scenario (for our paper)
Multi-constraint matches or exceeds edge-weighted with optimal parameters:
- Gap disappears or reverses
- Invalidates paper conclusion
- Requires major paper revision or rejection

**Likelihood**: Low (~10-15%). Our corrected P1-1 implementation already tested with proper asymmetric targets, and multi-constraint still struggles (30.0% success).

---

## Success Criteria

1. ✅ **Config balance**: 140 multi-constraint vs 140 edge-weighted
2. ✅ **Fair comparison**: Same number of "chances" for each method
3. ✅ **Parameter exploration**: Thorough ubvec sweep (1.1 to 25.0)
4. ✅ **Statistical rigor**: Large enough sample for significance tests (pairs with P1-4)
5. ✅ **Addresses reviewer concern**: Dr. Phillips's fairness critique resolved

---

## Risks and Mitigations

### Risk 1: Multi-constraint performance improves significantly
**Likelihood**: Low (10-15%)
**Impact**: High (invalidates conclusion)
**Mitigation**:
- Analyze which ubvec values succeed (may reveal patterns)
- Emphasize difficulty of parameter selection (robustness issue)
- Report state-level outcomes (Alabama: edge-weighted only method achieving 2 MM)

### Risk 2: Experiments take longer than expected
**Likelihood**: Medium (30%)
**Impact**: Low (delays timeline)
**Mitigation**:
- Run experiments in parallel (5 states × 28 configs)
- Use batch processing scripts
- Monitor progress, abort if >12 hours

### Risk 3: Results are statistically indistinguishable
**Likelihood**: Low (5%)
**Impact**: Medium (weakens conclusion)
**Mitigation**:
- Pair with P1-4 (statistical rigor with multiple seeds)
- Focus on state-level outcomes (Alabama, Louisiana)
- Emphasize robustness (parameter sensitivity)

---

## Next Steps

**After P1-3 Approval**:
1. Implement Option 1 (28 ubvec values)
2. Run experiments (140 configs)
3. Analyze results and update paper
4. Document resolution in P1-3_COMPLETE.md

**Proceed to P1-4**: Statistical rigor (multiple seeds, significance tests)

---

## Notes

- This plan assumes we use the FIXED implementation (P1-1 corrected)
- Results will differ from original Paper 2 (gerry-nway-vs-recursive) due to corrected tpwgts
- May discover optimal ubvec range (e.g., 1.5-2.5) where multi-constraint succeeds
- If so, emphasize brittleness/parameter sensitivity as a practical concern

---

**Status**: 📋 Plan ready for approval | Awaiting go-ahead to implement Option 1
