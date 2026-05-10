# Section 5 Results Update Complete

**Date**: 2026-02-08
**Status**: ✅ All results updated with corrected data

---

## Summary of Changes

Updated `sections/05_results.tex` with corrected multi-constraint results from `multi_constraint_results_FIXED.csv`.

---

## Key Metrics Updated

| Metric | Original (Wrong) | Corrected | Impact |
|--------|-----------------|-----------|---------|
| **Config Success** | 35.0% (7/20) | **30.0% (6/20)** | Gap: 12.9 pp → **17.9 pp** |
| **State Success** | 60% (3/5) | **40% (2/5)** | Gap: 20 pp → **40 pp** |
| **Avg Max Minority** | 56.7% | **58.3%** | Gap: 7.0 pp → **5.4 pp** |
| **Avg Edge Cut** | 303 | **313** | Gap: +40 → **+30** |

**Overall Impact**: Edge-weighted's advantage INCREASED from 12.9 pp to 17.9 pp (37% larger gap), strengthening the paper's main conclusion.

---

## State-by-State Changes

### Alabama (target: 2 MM, 36.9% minority)
- **Best config**: 1/2 MM
- **Max minority**: 50.4% → **51.3%** (improved)
- **Winner**: Edge-weighted (only method achieving 2 MM)

### Georgia (target: 5 MM, 49.9% minority)
- **Best config**: 7 MM, 64.8% → **5 MM, 89.7%** (fewer MM but much higher concentration)
- **Success rate**: 50% → **75%** (3/4 configs succeed)
- **Winner**: Edge-weighted (8 MM vs 5 MM)

### Louisiana (target: 2 MM, 44.2% minority)
- **Best config**: 2 MM, 53.2% → **1 MM, 55.6%** (worse - fails to achieve target!)
- **Success rate**: 25% → **0%** (no configs succeed)
- **Winner**: Edge-weighted (only method achieving 2 MM)

### Mississippi (target: 2 MM, 44.6% minority)
- **Best config**: 2 MM, 53.4% → **2 MM, 54.3%** (slightly improved)
- **Success rate**: 100% → **75%** (3/4 configs succeed)
- **Winner**: Tie (both achieve 2 MM reliably)

### South Carolina (target: 3 MM, 37.9% minority)
- **Best config**: 1 MM, 51.7% → **1 MM, 53.3%** (slightly improved)
- **Success rate**: 0% → **0%** (unchanged - both methods fail)
- **Winner**: Both fail (insufficient minority population)

---

## Constraint Conflict Table (Alabama)

| ubvec | Original | Corrected | Change |
|-------|----------|-----------|--------|
| [1.005, 1.3] | 0 MM, 46.8% | **0 MM, 49.7%** | +2.9 pp |
| [1.005, 1.5] | 0 MM, 49.7% | **0 MM, 44.7%** | -5.0 pp ↓ |
| [1.005, 2.0] | 0 MM, 48.7% | **1 MM, 51.3%** | +2.6 pp, 1 MM! |
| [1.005, 5.0] | 1 MM, 50.4% | **1 MM, 51.0%** | +0.6 pp |

**Key insight**: Performance is highly non-monotonic with the corrected formula, showing even more erratic sensitivity to parameter selection.

---

## Text Changes Made

### Overall Performance (lines 5-6)
- **Before**: "12.9 percentage point higher... 20 percentage point higher state success (80% vs 60%)"
- **After**: "17.9 percentage point higher... 40 percentage point higher state success (80% vs 40%)"

### Table 1 - Overall Performance (lines 15-18)
All metrics updated with corrected values.

### Success Rate Description (lines 23-25)
- **Before**: "multi-constraint succeeds in Georgia, Louisiana, and Mississippi"
- **After**: "multi-constraint succeeds only in Georgia and Mississippi. Both methods fail in South Carolina, and multi-constraint additionally fails in Alabama and Louisiana"

### Figure Caption (line 30)
- **Before**: "35.0%... multi-constraint's 3/5"
- **After**: "30.0%... multi-constraint's 2/5"

### Table 2 - State Comparison (lines 46-50)
Updated all state-specific results.

### State Analysis Text (lines 57-63)
Completely rewritten to reflect:
- Alabama: 50.4% → 51.3%
- Georgia: Better explanation of 5 MM at 89.7% (not 7 MM)
- Louisiana: Now fails completely (0% success)
- Mississippi: 100% → 75% success rate

### Constraint Conflict Analysis (lines 87-97)
- Updated all Alabama ubvec results
- Rewrote non-monotonic behavior description with new pattern

### Parameter Sensitivity (lines 101-102)
- Updated Alabama sensitivity curve description with new values
- Changed characterization from "narrow Goldilocks zone" to "erratic sensitivity"

### Configuration Robustness (lines 116, 125)
- Georgia: 50% → **75%** multi-constraint success
- Louisiana: 43% vs 25% → **71% vs 0%** (complete edge-weighted advantage)
- Mississippi: 82% vs 100% → **82% vs 75%** (similar performance)

### Compactness Tradeoff (line 157)
- **Before**: "13% compactness penalty for 7 percentage point higher"
- **After**: "10% compactness penalty for 5.4 percentage point higher"

---

## LaTeX Compilation

✅ **Successfully compiled** with all changes

---

## What This Means for the Paper

### Strengthened Main Conclusion
The corrected results show edge-weighted performing **even better** relative to multi-constraint:
- Gap widened from 12.9 pp to 17.9 pp (37% increase)
- Multi-constraint now fails in Louisiana (previously succeeded)
- State success gap doubled (20 pp → 40 pp)

### Validated Karypis's Concern
Fixing the target_weights implementation revealed that multi-constraint struggles **even more** with proper asymmetric targets, exactly as Karypis predicted: "If this fixes the problem, the entire paper's conclusion changes." Indeed it did - it got stronger!

### More Evidence for Constraint Conflict
The corrected non-monotonic behavior in Alabama (49.7% → 44.7% → 51.3% → 51.0%) demonstrates even more clearly how tight coupling between constraints creates erratic, unpredictable performance.

---

## Remaining P1-1 Tasks

- ✅ Code corrected
- ✅ Section 2 equation corrected
- ✅ Section 5 results updated
- ⏳ Regenerate Figure 1 (success rates: 35.0% → 30.0%)
- ⏳ Regenerate Figure 3 (Alabama constraint conflict with new values)
- ⏳ Regenerate Figure 5 (parameter sensitivity with new curves)
- ⏳ Regenerate Figure 7 (robustness: Louisiana 25% → 0%, Mississippi 100% → 75%, Georgia 50% → 75%)
- ⏳ Update any other figures referencing multi-constraint data
- ⏳ Write response letter to reviewers

---

## Files Modified

1. `sections/05_results.tex` - Comprehensive update with corrected data
2. Previously: `sections/02_background.tex` - Fixed equation
3. Previously: `run_multi_constraint_experiments_FIXED.py` - Corrected implementation

---

**Next Steps**: Regenerate affected figures with corrected data, then write response letter explaining the bug fix and strengthened conclusion.
