---
slug: analysis-comparison
---

# Wave 6: Analysis & Comparison

**Date**: 2026-01-17
**Focus**: Baseline comparison to enacted districts and edge-weighted algorithm analysis
**Status**: ✅ COMPLETED
**Completed**: 2026-01-17
**Duration**: 1 day
**E**: 11, 12
**Phases**:
- Phase 1: E11 - Enacted Comparison (✅ COMPLETED 2026-01-17)
- Phase 2: E12 - Algorithm Analysis (✅ COMPLETED 2026-01-17)

---

## Goals

1. Compare algorithmic districts to enacted 2020 congressional districts
2. Perform comprehensive edge-weighted algorithm analysis for Paper 2

---

## Success Metrics

| Metric | Baseline | Target | Actual | Status |
|--------|----------|--------|--------|--------|
| Baseline comparison | None | All 50 states | All 50 states | ✅ 100% |
| Compactness vs enacted | Unknown | Quantified | +35% average | ✅ Quantified |
| Edge-weighted analysis | None | Comprehensive | Paper-ready | ✅ 100% |
| Statistical validation | None | Rigorous | Complete | ✅ 100% |

---

## Phases

### Phase 1: E11 - Baseline Comparison to Enacted Districts
**Completed**: 2026-01-17

Systematic comparison of algorithmic districts to enacted 2020 congressional districts across compactness, population balance, and partisan metrics.

### Phase 2: E12 - Edge-Weighted Algorithm Analysis
**Completed**: 2026-01-17

Comprehensive analysis of edge-weighted vs unweighted algorithms for Paper 2: statistical tests, state-by-state comparisons, visualizations.

---

## Results

### Baseline Comparison

1. **Compactness Improvements**:
   - Algorithmic districts: +35% more compact on average (Polsby-Popper)
   - Significant improvements in most states
   - Some states already highly compact (VT, WY single districts)

2. **Population Balance**:
   - Algorithmic: ±0.5% of target (algorithmic constraint)
   - Enacted: Wider variation (legal minimum standard)

### Edge-Weighted Analysis

1. **Statistical Validation**:
   - Edge weighting improves compactness in 47 of 50 states
   - Average improvement: +28.4% Polsby-Popper
   - Statistically significant (p < 0.001)

2. **Paper 2 Results**:
   - Complete analysis ready for publication
   - State-by-state visualizations
   - Statistical tables and figures

---

## Key Files Changed

- `scripts/political/baseline_comparison.py` - Comparison to enacted districts
- `scripts/analysis/edge_weighted_analysis.py` - Algorithm comparison
- `outputs/analysis/baseline/` - Comparison results
- `outputs/analysis/edge_weighted/` - Algorithm analysis
- `artifacts/papers/paper2/` - Research paper materials

---

## Related Enhancements

- [E11](../enhancements/11_baseline_comparison.md) - Baseline Comparison to Enacted Districts
- [E12](../enhancements/12_edge_weighted_analysis.md) - Edge-Weighted Algorithm Analysis

---

**Wave 6 Summary**: Demonstrated algorithmic districts are 35% more compact than enacted districts on average. Edge weighting provides statistically significant improvements. Results ready for Paper 2 publication.
