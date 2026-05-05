# Research Papers

28 papers across five tracks. All PDFs are pre-built and open directly. LaTeX sources live under [`research/`](../research/).

To recompile: `cd research && make docs`

---

## Track A — Synthesis

| # | Title | PDF | Status |
|---|-------|-----|--------|
| A.0 | Algorithmic Objectivity for Congressional Redistricting: A National-Scale Demonstration | [PDF](papers/A.0+synthesis-metapaper.pdf) | Published |
| A.1 | Research Portfolio Guide | — | Draft |
| A.2 | Portfolio Summary | — | Draft |

---

## Track B — Algorithm

| # | Title | PDF | Status |
|---|-------|-----|--------|
| B.0 | Algorithm Design Overview — Bakeoff | [PDF](papers/B.0+algorithm-design-overview.pdf) | Published |
| B.02 | ApportionRegions: Redistricting as Geographic Completion of Huntington-Hill | [PDF](papers/B.02+one-federal-law.pdf) | Published |
| B.1 | Recursive Bisection for Congressional Redistricting | [PDF](papers/B.1+recursive-bisection.pdf) | Published |
| B.2 | Edge-Weighted Recursive Bisection for Compact Congressional Districts | [PDF](papers/B.2+edge-weighted-bisection.pdf) | Published |
| B.3 | Single-Objective vs. Multi-Constraint METIS | [PDF](papers/B.3+multi-vs-edge.pdf) | Published |
| B.4 | Equivalence of Recursive and N-Way Bisection | [PDF](papers/B.4+adaptive-bisection.pdf) | Published |
| B.5 | N-Way vs. Recursive Bisection (General) | — | Draft |
| B.6 | Computational Complexity | — | Draft |
| B.7 | Solution Space and Seed Sensitivity | — | Draft |
| B.8 | GeoSection — Ratio-Optimal First-Level Bisection | [PDF](papers/B.8+geosection-ratio-optimal-bisection.pdf) | Published |
| B.9 | AreaSection — Dual Population-Area Constraint | [PDF](papers/B.9+areasection-dual-population-area-constraint.pdf) | Published |
| B.10 | Subdivision-Respecting Redistricting (county-sticky weights) | — | Draft |
| B.11 | ApportionRegions — Geographic Completion of Huntington-Hill | — | Draft |
| B.12 | ProportionalSection — Partisan Proportionality via Dual Constraint | — | Draft |
| B.13 | NestSection — Nested Multi-Chamber Redistricting | [PDF](papers/B.13+nestsection-nested-multi-chamber.pdf) | Published |
| B.14 | VRASection — Minority Geographic Alignment | [PDF](papers/B.14+vrasection-minority-opportunity-bisection.pdf) | Published |
| B.15 | StabilitySection — Cross-Census Stability | [PDF](papers/B.15+stabilitysection-cross-census-stability.pdf) | Published |
| B.16 | ConvergenceSweep — Empirical Seed Sufficiency (T=600) | [PDF](papers/B.16+convergence-sweep.pdf) | Published |

---

## Track C — Validation

| # | Title | PDF | Status |
|---|-------|-----|--------|
| C.1 | Spatial Resolution and Algorithmic Redistricting: MAUP Sensitivity Analysis | [PDF](papers/C.1+maup-sensitivity.pdf) | Published |
| C.2 | Cross-Census Validation | [PDF](papers/C.2+cross-census-validation.pdf) | Published |
| C.3 | Cross-Census Temporal Stability | [PDF](papers/C.3+temporal-stability.pdf) | Published |
| C.4 | Twenty Years of Congressional Redistricting: A Longitudinal Analysis | [PDF](papers/C.4+longitudinal-analysis.pdf) | Published |
| C.5 | Measuring Partisan Fairness: Efficiency Gap Analysis | [PDF](papers/C.5+efficiency-gap-analysis.pdf) | Published |

---

## Track D — Voting Rights Act

| # | Title | PDF | Status |
|---|-------|-----|--------|
| D.0 | VRA Compliance Through Edge-Weighted Graph Partitioning | [PDF](papers/D.0+vra-compliance.pdf) | Published |
| D.1 | The 42% Threshold: Geographic Limits of VRA Compliance | [PDF](papers/D.1+threshold-analysis.pdf) | Published |
| D.2 | N-Way vs. Recursive Bisection for VRA-Compliant Redistricting | [PDF](papers/D.2+nway-vs-recursive-vra.pdf) | Published |
| D.3 | Quantifying the VRA–Compactness Tradeoff | [PDF](papers/D.3+compactness-tradeoff.pdf) | Published |
| D.4 | Legal Implementation Framework | — | Draft |

---

## Track E — Experimental Extensions

| # | Title | PDF | Status |
|---|-------|-----|--------|
| E.1 | Multi-Member Districts and Proportional Representation | [PDF](papers/E.1+multi-member-districts.pdf) | Published |
| E.2 | Direct County Representation | [PDF](papers/E.2+county-representation.pdf) | Published |
| E.3 | National Redistricting Without State Boundaries | [PDF](papers/E.3+national-redistricting.pdf) | Published |
| E.4 | Partisan Similarity Districts: Algorithmic Safe Seats | [PDF](papers/E.4+partisan-similarity-districts.pdf) | Published |
| E.5 | Partisan Fairness Through Algorithmic Districting | [PDF](papers/E.5+party-based-allocation.pdf) | Published |
| E.6 | International Applications | [PDF](papers/E.6+international-applications.pdf) | Published |
| E.7 | Lessons Learned | — | Draft |

---

*To add a compiled PDF: place `main.tex` in `research/PAPER-DIR/`, run `cd research && make docs`, commit `docs/papers/PAPER-DIR.pdf`.*
