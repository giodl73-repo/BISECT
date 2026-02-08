# Apportionment Research Papers

**Module**: Congressional Redistricting via Algorithmic Graph Partitioning
**Papers**: 10
**Author**: Giovanni Della-Libera

---

## Paper Inventory

| # | Directory | Title | PDF | Venue Target | Score | Tier |
|---|-----------|-------|-----|-------------|-------|------|
| 1 | [gerry-threshold-analysis](gerry-threshold-analysis/) | The 42% Threshold: Geographic Limits of VRA Compliance | [PDF](gerry-threshold-analysis/main.pdf) | Election Law Journal | 3.7/4 (est) | A- |
| 2 | [gerry-adaptive-bisection](gerry-adaptive-bisection/) | Edge-Weighting Makes Method Selection Irrelevant | [PDF](gerry-adaptive-bisection/main.pdf) | APSR / Operation Research | 4.0/4 (est) | A |
| 3 | [gerry-compactness-tradeoff](gerry-compactness-tradeoff/) | Quantifying the VRA-Compactness Tradeoff | [PDF](gerry-compactness-tradeoff/main.pdf) | Political Analysis | TBD | - |
| 4 | [gerry-multi-vs-edge](gerry-multi-vs-edge/) | Multi-Criterion vs Edge-Weighted Comparison | [PDF](gerry-multi-vs-edge/main.pdf) | INFORMS Journal | TBD | - |
| 5 | [gerry-nway-vs-recursive](gerry-nway-vs-recursive/) | N-Way vs Recursive Partitioning | [PDF](gerry-nway-vs-recursive/main.pdf) | Algorithmica | TBD | - |
| 6 | [gerry-edge-weighted-bisection](gerry-edge-weighted-bisection/) | Edge-Weighted Bisection Methods | [PDF](gerry-edge-weighted-bisection/main.pdf) | SIAM J Computing | TBD | - |
| 7 | [gerry-recursive-bisection](gerry-recursive-bisection/) | Recursive Bisection for Redistricting | [PDF](gerry-recursive-bisection/main.pdf) | INFORMS Journal | TBD | - |
| 8 | [gerry-cross-census-validation](gerry-cross-census-validation/) | Cross-Census Validation Study | [PDF](gerry-cross-census-validation/main.pdf) | TBD | TBD | - |
| 9 | [gerry-temporal-stability](gerry-temporal-stability/) | Temporal Stability Analysis | [PDF](gerry-temporal-stability/main.pdf) | TBD | TBD | - |
| 10 | [gerry-vra-compliance](gerry-vra-compliance/) | VRA Compliance Patterns | [PDF](gerry-vra-compliance/main.pdf) | TBD | TBD | - |

---

## Paper Dependency Graph

```
Core Theory Papers:
  gerry-recursive-bisection (foundations)
    ├── gerry-edge-weighted-bisection (edge weighting)
    ├── gerry-nway-vs-recursive (method comparison)
    └── gerry-adaptive-bisection (method equivalence) ← Ready

VRA Application Papers:
  gerry-threshold-analysis (geographic limits) ← Ready
    ├── gerry-compactness-tradeoff (VRA vs compactness) ← Ready
    └── gerry-vra-compliance (compliance patterns)

Validation Studies:
  gerry-cross-census-validation (temporal validation)
  gerry-temporal-stability (stability analysis)

Experimental Comparisons:
  gerry-multi-vs-edge (multi-criterion comparison) ← Data complete
```

---

## Review Status

| Paper | Stage | Round | Reviewers | Score | Verdict |
|-------|-------|-------|-----------|-------|---------|
| gerry-threshold-analysis | revision | 1 | 5/5 | 3.7/4 (est) | Ready for round 2 |
| gerry-adaptive-bisection | revision | 1 | 5/5 | 4.0/4 (est) | Supplement complete |
| gerry-compactness-tradeoff | draft | 0 | - | - | Supplement complete |
| gerry-multi-vs-edge | draft | 0 | - | - | Data collection done |
| gerry-nway-vs-recursive | draft | 0 | - | - | - |
| gerry-edge-weighted-bisection | draft | 0 | - | - | - |
| gerry-recursive-bisection | draft | 0 | - | - | P2.2 blocking |
| gerry-cross-census-validation | draft | 0 | - | - | - |
| gerry-temporal-stability | draft | 0 | - | - | - |
| gerry-vra-compliance | draft | 0 | - | - | - |

**Ready for Panel Review**: gerry-threshold-analysis, gerry-adaptive-bisection (2 papers at recheck stage)

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
*Papers: 10 (2 ready for panel review, 1 data complete, 1 with supplement, 6 in development)*
