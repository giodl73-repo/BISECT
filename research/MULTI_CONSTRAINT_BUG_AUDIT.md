# Multi-Constraint Target Weights Bug Audit

**Date**: 2026-02-08
**Bug Description**: Incorrect formula for METIS tpwgts in multi-constraint partitioning

## The Bug

When creating target weights for METIS multi-constraint partitioning with asymmetric minority concentration goals:

**INCORRECT Formula** (produces inverted concentrations):
```python
t_i^{min} = 0.60 * (M_total / k)  # for MM districts
```

**CORRECT Formula** (produces desired concentrations):
```python
M_i = 0.60 * (P_total / k)  # minority VAP per MM district
t_i^{min} = M_i / M_total    # normalize to fraction
```

**Impact**:
- For Alabama: Produces 22% concentration instead of 60% (37.9 percentage point error)
- Concentrations are INVERTED (MM districts get less minority than other districts)

## Papers Affected

### ✅ CONFIRMED BUG: gerry-multi-vs-edge
**Paper**: "Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals"

**Status**: 🔴 **HAS BUG** - Multiple issues
1. `create_target_weights()` defined but **NEVER CALLED** (lines 98-146 in run_multi_constraint_experiments.py)
2. When called, uses **INCORRECT FORMULA** (Equation 3 in sections/02_background.tex)
3. Multi-constraint experiments ran with **NO asymmetric targets** at all

**Files**:
- `run_multi_constraint_experiments.py` - Buggy implementation
- `CORRECT_IMPLEMENTATION.py` - Fixed version (created 2026-02-08)
- `sections/02_background.tex` - Contains Equation 3 with incorrect formula
- `sections/03_theory.tex` - Contains confused mathematical analysis based on bug

**Action Required**:
- ✅ CORRECT_IMPLEMENTATION.py created (2026-02-08)
- ⏳ Someone else is handling fixes (per user)

---

### ✅ NO BUG: gerry-vra-compliance
**Paper**: "Voting Rights Act Compliance Through Edge-Weighted Graph Partitioning"

**Status**: 🟢 **NO BUG** - Uses edge-weighted optimization only

**Reason**: Paper uses edge-weighted single-objective optimization exclusively. Does not implement multi-constraint partitioning, so bug is not present.

**Files**:
- Only mentions multi-constraint in background/comparison sections
- No multi-constraint implementation code

---

### ✅ NO BUG: Main Codebase (vra_targets.py)
**File**: `src/apportionment/partition/vra_targets.py`

**Status**: 🟢 **CORRECT IMPLEMENTATION**

**Reason**:
- Used for **recursive bisection** (not multi-constraint)
- Uses CORRECT formula (line 46):
  ```python
  mm_minority_total = target_mm_districts * pop_per_district * mm_target_pct
  ```
- This matches the correct mathematical derivation

**Files**:
- `src/apportionment/partition/vra_targets.py` - Correct implementation
- `src/apportionment/partition/metis_wrapper.py` - Just passes target_weights through (no calculation)

---

### ⏳ NEEDS AUDIT: gerry-nway-vs-recursive
**Paper**: "N-way vs. Recursive Bisection Multi-Constraint Comparison"

**Status**: 🟡 **UNKNOWN** - Needs investigation

**Reason**: Paper mentions multi-constraint in title and background sections

**Action Required**:
- Check if paper implements multi-constraint with target weights
- Search for `create_target_weights` or `tpwgts` calculations
- If found, verify formula matches CORRECT_IMPLEMENTATION.py

**Files to Check**:
- `scripts/pipeline/test_nway_vs_recursive_v2.py`
- Any other scripts that call multi-constraint partitioning

---

### ✅ NO BUG: gerry-recursive-bisection
**Paper**: "Recursive Bisection for Congressional Redistricting"

**Status**: 🟢 **CORRECT IMPLEMENTATION**

**Reason**: Contains `scripts/vra/vra_constrained_redistricting.py` with correct formula

**Formula** (line 143):
```python
minority_fraction = (mm_target_pct * ideal_pop_per_district) / total_minority_vap
```
This matches the CORRECT formula: `(c_mm * P/k) / M_total`

**Files**:
- `scripts/vra/vra_constrained_redistricting.py` - Correct implementation (lines 101-152)

---

### ✅ NO BUG: gerry-threshold-analysis
**Paper**: "VRA Threshold Analysis"

**Status**: 🟢 **LIKELY NO BUG** - No target weight calculations found

**Reason**: Mentions multi-constraint but no `create_target_weights` function found

**Files**:
- `compute_feasibility_metric.py` - Just feasibility calculations, no METIS calls

---

### ✅ NO BUG: gerry-adaptive-bisection
**Paper**: "Adaptive Bisection for Redistricting"

**Status**: 🟢 **NO BUG** - Uses recursive bisection

**Reason**: Uses recursive bisection (like main codebase), not multi-constraint

---

### ✅ NO BUG: gerry-compactness-tradeoff
**Paper**: "Compactness Tradeoff Analysis"

**Status**: 🟢 **NO BUG** - Analysis only

**Reason**: Analysis of existing results, no partitioning implementation

---

### ✅ NO BUG: gerry-edge-weighted-bisection
**Paper**: "Edge-Weighted Bisection"

**Status**: 🟢 **NO BUG** - Uses edge weights, not multi-constraint

---

### ✅ NO BUG: gerry-temporal-stability
**Paper**: "Temporal Stability Analysis"

**Status**: 🟢 **NO BUG** - Analysis only

---

### ✅ NO BUG: gerry-cross-census-validation
**Paper**: "Cross-Census Validation"

**Status**: 🟢 **NO BUG** - Validation study

---

## Summary

| Paper | Status | Bug Present | Action |
|-------|--------|-------------|--------|
| gerry-multi-vs-edge | 🔴 Confirmed | **YES** | Someone else fixing |
| gerry-vra-compliance | 🟢 Clear | NO | No action needed |
| gerry-nway-vs-recursive | 🟢 Clear | NO | No target_weights calculations found |
| gerry-recursive-bisection | 🟢 Clear | NO | **Correct implementation** |
| Main codebase (vra_targets.py) | 🟢 Clear | NO | Correct implementation |
| All other papers | 🟢 Clear | NO | No action needed |

## Action Items

1. ✅ **DONE**: Document correct formula in CORRECT_IMPLEMENTATION.py
2. ⏳ **IN PROGRESS**: Fix gerry-multi-vs-edge (someone else handling)
3. ✅ **DONE**: Audit gerry-nway-vs-recursive - No bug found
4. ✅ **DONE**: Audit gerry-recursive-bisection/vra_constrained_redistricting.py - Correct implementation confirmed
5. ✅ **DONE**: Complete bug audit - Only gerry-multi-vs-edge affected

## References

- Karypis Review: `research/gerry-multi-vs-edge/reviews/REVIEW-KARYPIS.md`
- Correct Implementation: `research/gerry-multi-vs-edge/CORRECT_IMPLEMENTATION.py`
- Main Codebase (Correct): `src/apportionment/partition/vra_targets.py`
