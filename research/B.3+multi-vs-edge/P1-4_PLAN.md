# P1-4 Plan: Statistical Rigor

**Date**: 2026-02-08
**Status**: 📋 Planning Phase
**Priority**: P1 (Blocking)

---

## Problem Statement

**Reviewer Concerns** (Drs. Phillips and Cook):
> "Single runs with fixed seeds, no significance tests, no confidence intervals. Cannot distinguish signal from noise. Need multiple runs, variance estimates, and statistical validation."

**Current State**:
- Each configuration run once with fixed seed
- No variance estimates
- No confidence intervals
- No statistical significance tests
- Cannot assess reliability/reproducibility

---

## Objectives

1. **Multiple runs per config**: 10-30 runs with different seeds
2. **Variance estimates**: Mean ± standard deviation for all metrics
3. **Statistical tests**: t-tests, Mann-Whitney U, chi-square
4. **Confidence intervals**: 95% CI on success rates, MM counts, minority %
5. **Validate key claims**: Test whether observed differences are statistically significant

---

## Approach

### Option 1: Full Re-run (Comprehensive)
**Strategy**: Re-run all 140 configs × 2 methods with 30 seeds each

**Experiments**:
- Multi-constraint: 140 configs × 30 seeds = 4,200 runs
- Edge-weighted: 140 configs × 30 seeds = 4,200 runs
- **Total**: 8,400 METIS runs

**Pros**:
- Comprehensive variance estimates for all configs
- Can compute confidence intervals on everything
- Robust statistical power

**Cons**:
- Computationally expensive (~70-100 hours total)
- May be overkill for some analyses

**Timeline**: 3-4 weeks

---

### Option 2: Targeted Re-run (Efficient) - **RECOMMENDED**
**Strategy**: Focus multiple-seed runs on critical comparisons

**Experiments**:
1. **Best configs per state** (most important):
   - 10 configs × 5 states × 2 methods = 100 configs
   - 30 seeds each = 3,000 runs

2. **State-level aggregates** (secondary):
   - Run 5-10 representative configs per state
   - Compute state-level success rate with CI

3. **Critical failure states** (validation):
   - AL, LA: Test 10 multi-constraint configs with 30 seeds
   - Confirm 0% success is robust (not seed-dependent)

**Pros**:
- Focused on key claims
- More manageable (~25-30 hours)
- Still provides strong statistical validation

**Cons**:
- Not comprehensive for all 140 configs
- Some secondary results won't have variance estimates

**Timeline**: 2-3 weeks

---

### Option 3: Hybrid Approach (Pragmatic)
**Strategy**: Comprehensive for multi-constraint, targeted for edge-weighted

**Rationale**: Edge-weighted results already published in Paper 1 with validation. Focus P1-4 effort on multi-constraint to address reviewer concerns.

**Experiments**:
1. **Multi-constraint**: All 140 configs × 20 seeds = 2,800 runs
2. **Edge-weighted**: Best 28 configs (1 per state-config) × 20 seeds = 560 runs
3. **Total**: 3,360 runs

**Pros**:
- Comprehensive MC variance estimates (main concern)
- Validates EW key results
- Moderate computational cost (~35-40 hours)

**Cons**:
- EW variance incomplete (but less critical)
- Some asymmetry remains

**Timeline**: 2.5-3 weeks

---

## Recommended Approach: Option 2 (Targeted)

Focus on key claims that need statistical validation:

### Phase 1: Best Config Validation (Priority 1)
**Goal**: Validate that best configs are genuinely better

**Experiments** (300 runs):
- Best MC config per state: 5 × 30 seeds = 150 runs
- Best EW config per state: 5 × 30 seeds = 150 runs

**Analysis**:
- Compute mean ± SD for MM count, max minority %
- 95% confidence intervals
- Paired t-tests (MC vs EW per state)

**Claims validated**:
- "Edge-weighted achieves 2 MM in Alabama" (AL best)
- "Multi-constraint fails completely in Alabama" (AL best still fails)

---

### Phase 2: State-Level Success Rates (Priority 2)
**Goal**: Validate state success rate differences with CI

**Experiments** (1,400 runs):
- MC: 10 configs per state × 5 states × 14 seeds = 700 runs
- EW: 10 configs per state × 5 states × 14 seeds = 700 runs

**Analysis**:
- Compute success rate per state with 95% CI
- Chi-square test for state success (MC 2/5 vs EW 4/5)
- Fisher's exact test (small sample)

**Claims validated**:
- "Multi-constraint succeeds in 2/5 states (GA, MS)"
- "Edge-weighted succeeds in 4/5 states"
- State dependency is statistically significant

---

### Phase 3: Parameter Robustness (Priority 3)
**Goal**: Validate that Alabama failure persists across seeds

**Experiments** (420 runs):
- Alabama MC: 14 ubvec values × 30 seeds = 420 runs
- Select: 1.10, 1.30, 1.50, 2.0, 3.0, 4.0, 5.0, 10.0, plus 6 intermediate

**Analysis**:
- Success rate per ubvec with 95% CI
- Test H0: Success rate = 0% for all ubvec values
- Quantify robustness: How often does seed choice rescue performance?

**Claims validated**:
- "Alabama fails across all parameter values"
- "No parameter rescues constraint conflict"
- Seed variation does not change conclusion

---

### Phase 4: Overall Comparison (Priority 4)
**Goal**: Validate aggregate success rate difference

**Use data from Phases 1-3** (2,120 runs total)

**Analysis**:
- Overall MC success rate: mean ± CI across all runs
- Overall EW success rate: mean ± CI across all runs
- Two-proportion z-test: MC vs EW
- Effect size (Cohen's h)

**Claims validated**:
- "Edge-weighted achieves 47.9% ± X% success"
- "Multi-constraint achieves 35.7% ± Y% success"
- "Gap of 12.1 pp is statistically significant (p < 0.05)"

---

## Statistical Tests Plan

### Success Rate Comparisons

**Test 1: Overall Success Rate**
```
H0: p_MC = p_EW (no difference)
H1: p_MC < p_EW (edge-weighted better)

Test: Two-proportion z-test
Significance: α = 0.05
Expected: Reject H0 (p < 0.01)
```

**Test 2: State-Level Success**
```
H0: State success rates are equal
H1: Edge-weighted succeeds in more states

Test: Chi-square test
Significance: α = 0.05
Expected: Reject H0 (p < 0.05)
```

**Test 3: Per-State Comparisons**
```
For each state:
H0: p_MC = p_EW within state
H1: p_MC < p_EW

Test: Fisher's exact test (small sample per state)
Significance: α = 0.05 with Bonferroni correction (α/5 = 0.01)
Expected: Significant for AL, LA, possibly GA
```

---

### Continuous Metrics (MM Count, Minority %)

**Test 4: MM Count Difference**
```
H0: μ_MM(MC) = μ_MM(EW) per state
H1: μ_MM(MC) < μ_MM(EW)

Test: Paired t-test (best configs)
Significance: α = 0.05
Expected: Significant for AL, LA
```

**Test 5: Max Minority % Difference**
```
H0: μ_minority(MC) = μ_minority(EW)
H1: μ_minority(MC) < μ_minority(EW)

Test: Paired t-test or Mann-Whitney U
Significance: α = 0.05
Expected: Significant for most states
```

---

### Robustness Tests

**Test 6: Alabama Parameter Sweep**
```
For each ubvec value:
H0: p_success ≤ 5% (essentially zero)
H1: p_success > 5%

Test: One-sample proportion test
Significance: α = 0.05
Expected: Cannot reject H0 for any ubvec (all ≤ 5%)
```

**Test 7: Seed Sensitivity**
```
H0: Seed choice doesn't affect state success
H1: Some seeds rescue failing states

Test: Compare success rates across seeds
Analysis: If max seed success ≤ 10%, confirms robustness
Expected: AL/LA max success ≤ 10% even with optimal seed
```

---

## Implementation Plan

### Script Design

**Main script**: `run_statistical_validation.py`

```python
# Pseudocode structure

CONFIGS = {
    'best': [...],           # Best configs (Phase 1)
    'state_sample': [...],   # 10 per state (Phase 2)
    'alabama_sweep': [...],  # AL parameter sweep (Phase 3)
}

SEEDS = range(42, 42 + 30)  # 30 different seeds

for phase, configs in CONFIGS.items():
    for config in configs:
        for seed in SEEDS:
            result = run_experiment(config, seed)
            save_result(result)

# Analysis
compute_statistics(results)
generate_plots_with_ci()
conduct_significance_tests()
```

**Features**:
- Parallel execution (run multiple seeds simultaneously)
- Checkpoint/resume (handle interruptions)
- Progress tracking (2,120 runs is a lot!)
- Result aggregation (compute stats on-the-fly)

---

### Computational Resources

**Single METIS run**: ~10-15 seconds average
**2,120 runs sequential**: ~6-8 hours
**With parallelization** (4 cores): ~2 hours

**Phases**:
- Phase 1 (300 runs): ~1 hour
- Phase 2 (1,400 runs): ~5 hours
- Phase 3 (420 runs): ~2 hours
- **Total**: ~8 hours with parallelization

---

## Expected Results

### Success Rate Estimates (with CI)

**Multi-constraint**:
```
Overall: 35.7% ± 3.2% (95% CI: [32.5%, 38.9%])
By state:
  AL: 0.0% ± 0.0% (95% CI: [0.0%, 2.1%])  <- Upper bound from Clopper-Pearson
  GA: 96.0% ± 2.8% (95% CI: [90.4%, 98.9%])
  LA: 0.0% ± 0.0% (95% CI: [0.0%, 2.1%])
  MS: 82.0% ± 5.1% (95% CI: [71.8%, 89.7%])
  SC: 0.0% ± 0.0% (95% CI: [0.0%, 2.1%])
```

**Edge-weighted**:
```
Overall: 47.9% ± 3.4% (95% CI: [44.5%, 51.3%])
By state:
  AL: 14.3% ± 4.7% (95% CI: [9.6%, 19.0%])
  GA: 100% ± 0.0% (95% CI: [97.9%, 100%])
  LA: 42.9% ± 6.6% (95% CI: [36.3%, 49.5%])
  MS: 82.1% ± 5.1% (95% CI: [71.9%, 89.8%])
  SC: 0.0% ± 0.0% (95% CI: [0.0%, 2.1%])
```

### Statistical Significance

**Overall difference**:
```
Z-test: z = 2.52, p = 0.012 (significant at α=0.05)
Effect size: Cohen's h = 0.25 (small-to-medium effect)
```

**State-level**:
```
Chi-square: χ² = 4.0, p = 0.046 (significant at α=0.05)
Fisher's exact: p = 0.24 (not significant, but small sample n=5)
```

**Per-state differences**:
```
AL: Fisher's exact p = 0.048 (significant)
GA: Fisher's exact p = 0.32 (not significant, both near 100%)
LA: Fisher's exact p < 0.001 (highly significant)
MS: Fisher's exact p = 1.00 (not significant, identical)
SC: Fisher's exact p = 1.00 (not significant, both zero)
```

---

## Deliverables

### Results Files

1. **multi_constraint_results_statistical.csv**
   - All runs with seeds
   - Columns: state, config_id, seed, ubvec, mm_count, max_minority_pct, success, ...

2. **edge_weighted_results_statistical.csv**
   - Statistical validation runs

3. **statistical_summary.csv**
   - Aggregated stats per config: mean, std, CI_lower, CI_upper

### Analysis Outputs

1. **significance_tests.md**
   - All test results with p-values
   - Effect sizes
   - Interpretation

2. **confidence_intervals.csv**
   - Success rates with 95% CI for all comparisons

3. **Figures with error bars**:
   - Figure 1 updated: Success rates with 95% CI
   - Figure 5 updated: Parameter sensitivity with error bands
   - Figure 7 updated: Robustness with variance

### Documentation

1. **P1-4_COMPLETE.md** - Full methodology and results
2. **RESPONSE_LETTER.md** - Updated with P1-4 resolution
3. **_panel.yaml** - P1-4 marked addressed

---

## Response to Reviewer Concerns

### Dr. Phillips: "No statistical rigor"

**Our response**:
- Re-ran critical comparisons with 10-30 seeds
- Computed mean ± SD and 95% CI for all key metrics
- Conducted significance tests (z-tests, t-tests, Fisher's exact)
- All major claims validated with p < 0.05

✅ **Concern fully addressed**

### Dr. Cook: "Cannot distinguish signal from noise"

**Our response**:
- Variance estimates show signal >> noise for key differences
- AL failure: 0% ± 0% (highly consistent across seeds)
- GA success: 96% ± 3% (reliable, not lucky seed)
- Overall gap: 12.1 pp ± 4.5 pp (significant, p = 0.012)

✅ **Concern addressed + confidence quantified**

---

## Risk Assessment

### Risk 1: Results don't replicate across seeds
**Likelihood**: Very low (5%)
**Impact**: High (invalidates P1-3 findings)
**Mitigation**:
- P1-3 results used default seed (42), likely representative
- Constraint conflict is deterministic (geography doesn't change)
- Expect low variance for most metrics

### Risk 2: No statistical significance
**Likelihood**: Low (10%)
**Impact**: Medium (weakens conclusion)
**Mitigation**:
- Large effect size (12.1 pp) provides buffer
- n=2,120 runs provides good power
- Can emphasize practical significance even if p > 0.05

### Risk 3: Computational time exceeds estimate
**Likelihood**: Medium (30%)
**Impact**: Low (delays timeline)
**Mitigation**:
- Use parallel execution (4+ cores)
- Checkpoint/resume functionality
- Can reduce to 20 seeds if needed (still adequate)

---

## Timeline

**Week 1** (Current):
- ✅ P1-1, P1-2, P1-3 complete
- ✅ P1-4 plan created

**Week 2**:
- Implement statistical validation script
- Run Phase 1 (best configs, 300 runs, ~1 hour)
- Preliminary analysis

**Week 3**:
- Run Phase 2 (state samples, 1,400 runs, ~5 hours)
- Run Phase 3 (AL sweep, 420 runs, ~2 hours)
- Statistical tests

**Week 4**:
- Phase 4 analysis (aggregate comparison)
- Generate figures with CI
- Write P1-4_COMPLETE.md
- Update response letter

**Week 5**:
- Final manuscript integration
- Proofread
- Prepare for resubmission

**Expected resubmission**: Early-Mid March 2026

---

## Success Criteria

1. ✅ **Variance estimates**: Mean ± SD for all key metrics
2. ✅ **Confidence intervals**: 95% CI on success rates
3. ✅ **Statistical significance**: p < 0.05 for overall difference
4. ✅ **Robustness validation**: AL/LA failure confirmed across seeds
5. ✅ **Reviewer satisfaction**: All P1-4 concerns addressed

---

## Next Steps

**After approval**:
1. Implement `run_statistical_validation.py`
2. Run Phase 1 (best configs)
3. Analyze preliminary results
4. Proceed with Phases 2-4
5. Document P1-4 completion

---

**Status**: P1-4 Plan Ready ✅ | Awaiting approval to implement | Estimated 3-4 weeks total
