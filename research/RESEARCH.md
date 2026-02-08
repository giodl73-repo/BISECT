# Apportionment Research Papers

**Module**: Congressional Redistricting via Algorithmic Graph Partitioning
**Papers**: 10
**Author**: Giovanni Della-Libera

---

## Paper Inventory

| # | Directory | Title | PDF | Venue Target | Stage | Round |
|---|-----------|-------|-----|-------------|-------|-------|
| 1 | [gerry-threshold-analysis](gerry-threshold-analysis/) | The 42% Threshold: Geographic Limits of VRA Compliance | [PDF](gerry-threshold-analysis/main.pdf) | Election Law Journal | ready | 1 |
| 2 | [gerry-adaptive-bisection](gerry-adaptive-bisection/) | Edge-Weighting Makes Method Selection Irrelevant | [PDF](gerry-adaptive-bisection/main.pdf) | APSR / Operation Research | ready | 1 |
| 3 | [gerry-compactness-tradeoff](gerry-compactness-tradeoff/) | Quantifying the VRA-Compactness Tradeoff | [PDF](gerry-compactness-tradeoff/main.pdf) | Political Analysis | ready | 1 |
| 4 | [gerry-multi-vs-edge](gerry-multi-vs-edge/) | Multi-Criterion vs Edge-Weighted Comparison | [PDF](gerry-multi-vs-edge/main.pdf) | INFORMS Journal | ready | 1 |
| 5 | [gerry-nway-vs-recursive](gerry-nway-vs-recursive/) | N-Way vs Recursive Partitioning | [PDF](gerry-nway-vs-recursive/main.pdf) | Algorithmica | ready | 2 |
| 6 | [gerry-edge-weighted-bisection](gerry-edge-weighted-bisection/) | Edge-Weighted Bisection Methods | [PDF](gerry-edge-weighted-bisection/main.pdf) | SIAM J Computing | ready | 2 |
| 7 | [gerry-recursive-bisection](gerry-recursive-bisection/) | Recursive Bisection for Redistricting | [PDF](gerry-recursive-bisection/main.pdf) | INFORMS Journal | ready | 2 |
| 8 | [gerry-cross-census-validation](gerry-cross-census-validation/) | Cross-Census Validation Study | [PDF](gerry-cross-census-validation/main.pdf) | Political Analysis | ready | 3 |
| 9 | [gerry-temporal-stability](gerry-temporal-stability/) | Temporal Stability Analysis | [PDF](gerry-temporal-stability/main.pdf) | Political Analysis | ready | 1 |
| 10 | [gerry-vra-compliance](gerry-vra-compliance/) | VRA Compliance Patterns | [PDF](gerry-vra-compliance/main.pdf) | Election Law Journal | ready | 2 |

---

## Paper Dependency Graph

```
Core Theory Papers:
  gerry-recursive-bisection (foundations) ← ready
    ├── gerry-edge-weighted-bisection (edge weighting) ← ready
    ├── gerry-nway-vs-recursive (method comparison) ← ready
    └── gerry-adaptive-bisection (method equivalence) ← ready

VRA Application Papers:
  gerry-threshold-analysis (geographic limits) ← ready
    ├── gerry-compactness-tradeoff (VRA vs compactness) ← ready
    └── gerry-vra-compliance (compliance patterns) ← ready

Validation Studies:
  gerry-cross-census-validation (temporal validation) ← ready
  gerry-temporal-stability (stability analysis) ← ready

Experimental Comparisons:
  gerry-multi-vs-edge (multi-criterion comparison) ← ready
```

---

## Review Status Summary

**All 10 papers have completed individual reviews**
- **All 10 papers** at "ready" stage (all P1 items addressed, ready for panel review)

**Ready for Cross-Portfolio Panel Review**: All 10 papers eligible for `panel:convene`

---

## Build

```bash
make all          # Build all papers
make dist         # Copy PDFs to docs/
make clean        # Remove build artifacts

# Single paper
cd gerry-threshold-analysis && pdflatex main.tex
```

---

*Apportionment research module — established February 2026*
*Papers: 10 (all 10 ready for cross-portfolio panel review)*
