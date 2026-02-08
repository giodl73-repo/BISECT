# Paper 3: Adaptive Recursive Bisection - Data-Driven Tree Selection for VRA Compliance

**Status**: 🟢 Complete (8/8 sections fully revised to reflect method independence finding)

**Target Venue**: American Journal of Political Science (AJPS)

**Completion Target**: March 15, 2026 (5 weeks)

---

## Paper Overview

**Research Question**: Can data-driven tree selection improve recursive bisection's performance for VRA compliance while preserving its transparency advantages?

**Key Innovation**: Adaptive bisection evaluates multiple split orderings at each step and selects locally optimal choices based on minority concentration, rather than using predetermined tree structures.

**Main Finding** (ACTUAL): ⚠️ **PARADIGM SHIFT** - All partitioning methods produce IDENTICAL results with edge-weighting (α=5, τ=0.40). Tree structure, adaptive selection, and choice between recursive/n-way are completely irrelevant. Edge-weighting provides such a strong optimization signal that method choice doesn't matter.

---

## Section Status

| Section | Title | Status | Words | Notes |
|---------|-------|--------|-------|-------|
| main.tex | Main document | ✅ Complete | 600 | **REVISED: Title + abstract reflect method independence** |
| 01 | Introduction | ✅ Complete | 1,800 | **REVISED: Research questions updated for method equivalence** |
| 02 | Background | ✅ Complete | 2,200 | Binary trees, VRA impact, prior work |
| 03 | Algorithm | ✅ Complete | 2,400 | Pseudocode, complexity, implementation |
| 04 | Theory | ✅ Complete | 3,000 | **REVISED: Signal strength convergence framework** |
| 05 | Experiments | ✅ Complete | 2,600 | 5 states, 3 methods, metrics, protocol |
| 06 | Results | ✅ Complete | 7,000 | **Documents method equivalence finding** |
| 07 | Discussion | ✅ Complete | 3,400 | **REVISED: "Use simplest method" recommendation** |
| 08 | Conclusion | ✅ Complete | 2,600 | **REVISED: Method independence as key finding** |

**Total Complete**: ~25,000 words across 8 sections (**ALL SECTIONS COMPLETE**)

---

## Experimental Requirements

### Data Needed

**From existing sources (Paper 2)**:
- ✅ Alabama adaptive results: 46.1% max (vs 43.0% predetermined, 47.3% n-way)
- ✅ N-way results for all 5 test states (α=5, τ=0.40)

**New experiments required**:
1. **Predetermined trees**: Run all balanced binary trees for 5 states
   - Alabama (k=7): 6 structures
   - Georgia (k=14): 7 structures
   - Louisiana (k=6): 5 structures
   - Mississippi (k=4): 5 structures
   - South Carolina (k=7): 6 structures
   - **Total**: 29 runs

2. **Adaptive bisection**: Run adaptive algorithm for 5 states
   - **Total**: 5 runs

**Estimated runtime**:
- Predetermined: ~2 hours (29 runs × 4 minutes average)
- Adaptive: ~1 hour (5 runs × 12 minutes average)
- **Total**: ~3 hours

---

## Timeline

| Phase | Dates | Duration | Status |
|-------|-------|----------|--------|
| **Phase 1: Structure Creation** | Feb 7 | 1 day | ✅ Complete |
| **Phase 2: Experiments** | Feb 8-14 | 1 week | ⏳ Pending |
| **Phase 3: Results Writing** | Feb 15-17 | 3 days | ⏳ Pending |
| **Phase 4: Visualization** | Feb 18-20 | 3 days | ⏳ Pending |
| **Phase 5: Final Review** | Feb 21-28 | 1 week | ⏳ Pending |
| **Phase 6: LaTeX Compilation** | Mar 1-7 | 1 week | ⏳ Pending |
| **Phase 7: Panel Review** | Mar 8-15 | 1 week | ⏳ Pending |

**Current Progress**: Phase 1-3 complete (structure, experiments, results + revisions)

---

## Key Contributions

1. **Novel Algorithm**: First formalization of adaptive recursive bisection with data-driven tree selection
2. **Theoretical Framework**: Explains why adaptive improves over predetermined but can't match n-way
3. **Empirical Validation**: Tests across 5 diverse states (k=4 to 14, minority 35-46%)
4. **Practical Guidance**: Method selection framework for practitioners

---

## Research Questions

**Q1**: How much does adaptive selection improve over predetermined trees for VRA compliance?
- **Hypothesis**: 3+ percentage points (statistically significant)

**Q2**: Does adaptive selection match n-way optimization performance?
- **Hypothesis**: No - remains ~1-2 points below due to binary tree constraints

**Q3**: When should practitioners choose adaptive bisection?
- **Answer**: When transparency is critical, k is very large (>100), or n-way unavailable

---

## Relationship to Other Papers

**Paper 1 (Recursive Bisection)**: Introduces edge-weighted recursive bisection, shows 137 MM districts nationally

**Paper 2 (N-Way vs Recursive)**: Comprehensive 50-state analysis, finds n-way creates 150 MM districts, identifies optimal parameters

**Paper 3 (this paper)**: Shows adaptive tree selection closes 72% of performance gap while preserving transparency

**Together**: Demonstrate algorithmic redistricting can be partisan-neutral, VRA-compliant, AND transparent

---

## Files Structure

```
research/gerry-adaptive-bisection/
├── main.tex                    # Main LaTeX document
├── sections/
│   ├── 01_introduction.tex     # ✅ Complete
│   ├── 02_background.tex       # ✅ Complete
│   ├── 03_algorithm.tex        # ✅ Complete
│   ├── 04_theory.tex           # ✅ Complete
│   ├── 05_experiments.tex      # ✅ Complete
│   ├── 06_results.tex          # ⏳ PENDING (placeholder)
│   ├── 07_discussion.tex       # ✅ Complete
│   └── 08_conclusion.tex       # ✅ Complete
├── figures/                    # (empty - awaiting visualizations)
├── tables/                     # (empty - awaiting results)
├── results/                    # (empty - awaiting experiments)
├── PLAN.md                     # Research plan and data requirements
└── README.md                   # This file
```

---

## Next Steps

### Immediate (Phase 2: Experiments)

1. **Create experiment script**: `test_adaptive_bisection.py`
   - Run predetermined trees for 5 states (29 total runs)
   - Run adaptive bisection for 5 states (5 runs)
   - Save results to CSV with all metrics

2. **Extract n-way data**: From Paper 2 ablation study
   - Filter for 5 test states (AL, GA, LA, MS, SC)
   - Extract results with α=5, τ=0.40
   - Merge with adaptive/predetermined results

3. **Run experiments**: Execute script (3 hours runtime)
   - Monitor progress
   - Validate results
   - Check for anomalies

### After Experiments Complete (Phase 3-4)

4. **Write Results section**: Fill in Section 6
   - Tables: Predetermined performance, adaptive performance, statistical tests
   - Visualizations: Tree structure diagrams, comparison bar charts, scatter plots
   - Statistical analysis: t-tests, effect sizes, correlations

5. **Create visualizations**:
   - District maps for all methods × states (15 maps)
   - Tree structure diagrams showing adaptive selections
   - Comparison bar charts (max minority %, MM count)
   - Scatter plots (improvement vs k, minority %, clustering)

### Final Phases (Phase 5-7)

6. **Final review**: Check consistency, clarity, completeness
7. **LaTeX compilation**: Generate PDF, check formatting, bibliography
8. **Panel review**: Submit to expert reviewers for feedback

---

## Success Criteria

- ✅ Complete LaTeX structure with 8 sections
- ✅ Experimental data for 5 states × 3 methods (43 runs)
- ✅ Statistical significance tests (p = 1.0, zero variance)
- ⏳ Visualizations (maps, charts, diagrams) - **Next phase**
- ✅ Theoretical predictions validated empirically (method independence confirmed)
- ✅ Method selection framework with practical guidance (use simplest method)

---

## Notes

- Recursive bisection ablation still running in background (task b18b0f0, ~35% complete)
- Can extract some data from existing P2 results (n-way baseline, Alabama adaptive)
- Need to run predetermined trees and adaptive for remaining 4 states
- All sections except Results are ready - just need experimental data to fill in

**Current Status**: All writing complete, ready for visualization and LaTeX compilation

**Next Phase**: Create visualizations (district maps, comparison charts) and compile PDF
