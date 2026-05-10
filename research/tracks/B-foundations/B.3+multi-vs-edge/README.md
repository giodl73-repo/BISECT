# Paper 4: Multi-Constraint vs Edge-Weighted Optimization

**Title**: Why Single-Objective Graph Partitioning Outperforms Multi-Constraint Optimization for Asymmetric Redistricting Goals

**Status**: Draft Complete (2026-02-07)

## Summary

This paper demonstrates that edge-weighted single-objective optimization dramatically outperforms multi-constraint methods for congressional redistricting under the Voting Rights Act. Through 160 experiments across 5 states, we show:

- **47.9% vs 35.0%** configuration success rate (12.9 pp gap)
- **80% vs 60%** state success rate
- **7 pp higher** minority concentration
- **13% compactness penalty** (modest and acceptable)

**Key Contribution**: We introduce and validate the concept of *constraint conflict*—where tight population balance constraints dominate loose minority concentration constraints, fundamentally limiting multi-constraint effectiveness.

## Files

### Main Document
- `main.tex` - Main LaTeX file
- `references.bib` - Bibliography

### Sections
- `sections/01_introduction.tex` - Introduction and contributions
- `sections/02_background.tex` - Multi-constraint and edge-weighted methods
- `sections/03_theory.tex` - Constraint conflict theory
- `sections/04_experiments.tex` - Experimental design
- `sections/05_results.tex` - Results and analysis
- `sections/06_discussion.tex` - Discussion and implications
- `sections/07_related_work.tex` - Related work
- `sections/08_conclusion.tex` - Conclusion and future work

### Data and Analysis
- `results/multi_constraint_results.csv` - Multi-constraint experiment data (20 runs)
- `results/figure1_success_rates.{png,pdf}` - Success rate comparison
- `results/figure2_compactness_tradeoff.{png,pdf}` - Compactness vs concentration
- `results/figure3_constraint_conflict.{png,pdf}` - Alabama constraint conflict test
- `results/figure4_heatmap.{png,pdf}` - State-by-state heatmap
- `COMPARISON_ANALYSIS.md` - Comprehensive data analysis
- `TABLES_FOR_PAPER.md` - LaTeX-ready tables
- `FIGURES_SUMMARY.md` - Figure descriptions and integration guide

### Supporting Files
- `PLAN.md` - Original research plan and paper outline
- `experiment_log.txt` - Detailed experiment logs
- `generate_figures.py` - Figure generation script
- `run_multi_constraint_experiments.py` - Experiment runner

## Compilation

### Prerequisites
```bash
# LaTeX distribution (TeX Live, MiKTeX, or MacTeX)
pdflatex --version
bibtex --version
```

### Build Paper
```bash
cd research/gerry-multi-vs-edge

# First pass
pdflatex main.tex

# Generate bibliography
bibtex main

# Two more passes for references
pdflatex main.tex
pdflatex main.tex
```

### Or use convenience script:
```bash
# Windows
compile_paper.bat

# Linux/Mac
chmod +x compile_paper.sh
./compile_paper.sh
```

Output: `main.pdf`

## Regenerate Figures

```bash
python generate_figures.py
```

Outputs to `results/`:
- `figure1_success_rates.{png,pdf}`
- `figure2_compactness_tradeoff.{png,pdf}`
- `figure3_constraint_conflict.{png,pdf}`
- `figure4_heatmap.{png,pdf}`

## Key Results

### Overall Performance (Table 2)

| Metric | Edge-Weighted | Multi-Constraint | Gap |
|--------|--------------|------------------|-----|
| Config Success | 47.9% | 35.0% | +12.9 pp |
| State Success | 80% | 60% | +20 pp |
| Avg Max Minority | 63.7% | 56.7% | +7.0 pp |
| Avg Edge Cut | 343 | 303 | +40 (+13%) |

### State-by-State Results (Table 1)

| State | Winner | Key Finding |
|-------|--------|-------------|
| Alabama | **Edge-Weighted** | 2/2 MM vs 1/2 MM (only method to succeed) |
| Georgia | **Edge-Weighted** | 8/5 MM vs 7/5 MM (100% vs 50% config success) |
| Louisiana | **Edge-Weighted** | 2/2 MM vs 2/2 MM (higher concentration: 61.9% vs 53.2%) |
| Mississippi | **Tie** | Both achieve 2/2 MM reliably |
| South Carolina | **Both Fail** | Insufficient demographics (37.9% < needed for 3 MM) |

### Constraint Conflict Test (Table 3)

Alabama with increasingly loose minority constraints:

| ubvec | Tolerance | MM Count | Max % |
|-------|-----------|----------|-------|
| [1.005, 1.3] | ±30% | 0/2 | 46.8% |
| [1.005, 1.5] | ±50% | 0/2 | 49.7% |
| [1.005, 2.0] | ±100% | 0/2 | 48.7% |
| [1.005, 5.0] | ±400% | 1/2 | 50.4% |

**Conclusion**: Even 400% minority tolerance (80× looser than population) fails to achieve target!

## Paper Structure

1. **Introduction** (2 pages) - Problem, contributions, impact
2. **Background** (2 pages) - Multi-constraint and edge-weighted methods
3. **Theoretical Analysis** (2 pages) - Constraint conflict theory
4. **Experimental Design** (1.5 pages) - 160 experiments, 5 states
5. **Results** (3 pages) - Overall performance, state analysis, constraint conflict validation
6. **Discussion** (2.5 pages) - When to use each method, implications, limitations
7. **Related Work** (1 page) - Graph partitioning, redistricting, multi-objective optimization
8. **Conclusion** (1 page) - Summary, impact, future work

**Total**: ~15 pages (estimated)

## Citations Needed

Key citations to add before submission:
- METIS paper (Karypis & Kumar 1998) ✅
- Multi-constraint partitioning (Karypis & Kumar 1999) ✅
- VRA cases (Thornburg v. Gingles) ✅
- Redistricting surveys (Altman 2011) ✅
- Multi-objective optimization (Deb 2001, Marler 2004) ✅

## Next Steps

1. ✅ Complete all sections (DONE)
2. ✅ Generate figures (DONE)
3. ✅ Create tables (DONE)
4. ⏳ Compile LaTeX and review formatting
5. ⏳ Proofread and edit
6. ⏳ Add acknowledgments and author information
7. ⏳ Submit to venue (TBD: INFORMS, Operations Research, SODA?)

## Venue Targets

**Primary**:
- INFORMS Journal on Computing (optimization + applications)
- SIAM Journal on Scientific Computing (graph partitioning focus)
- Operations Research (broader audience)

**Secondary**:
- ACM Transactions on Mathematical Software
- Conference: SODA, SEA (Symposium on Experimental Algorithms)

## Contact

[Author information to be added upon submission]

## License

Research data and code available upon request for reproducibility.
