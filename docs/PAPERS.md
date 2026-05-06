# Research Papers

66 papers across eight tracks (A–H). PDFs open directly from the links below. LaTeX sources in [`research/`](../research/).

Papers are organised by the question they answer. Each paper's series code (A–H) is shown for cross-reference. To recompile: `cd research && make docs`

---

## What is the algorithm and why does it work?

The mathematical and computational foundations. Start here if you are new to the project.

| Code | Title | PDF |
|------|-------|-----|
| B.1 | Recursive Bisection for Congressional Redistricting | [PDF](papers/B.1+recursive-bisection.pdf) |
| B.2 | Edge-Weighted Recursive Bisection (+22% compactness) | [PDF](papers/B.2+edge-weighted-bisection.pdf) |
| B.3 | Single-Objective vs. Multi-Constraint METIS | [PDF](papers/B.3+multi-vs-edge.pdf) |
| B.4 | Equivalence of Recursive and N-Way Bisection | [PDF](papers/B.4+adaptive-bisection.pdf) |
| B.5 | N-Way vs. Recursive Bisection: General Comparison | [PDF](papers/B.5+nway-vs-recursive-general.pdf) |
| B.6 | Computational Complexity: O(n log² n), O(√log n) approximation | [PDF](papers/B.6+computational-complexity.pdf) |
| B.7 | Solution Space and Seed Sensitivity (CV < 2% for 96% of states) | [PDF](papers/B.7+solution-space-and-seed-sensitivity.pdf) |

---

## How should the bisection tree be structured?

The "structure" layer of the compositor. Every paper here answers: given k districts, what shape is the recursion?

| Code | Title | PDF |
|------|-------|-----|
| B.0 | Bakeoff — all 8 algorithm modes on 6 competitive states | [PDF](papers/B.0+algorithm-design-overview.pdf) |
| B.8 | GeoSection — ratio-optimal first bisection, isoperimetric normalisation | [PDF](papers/B.8+geosection-ratio-optimal-bisection.pdf) |
| B.9 | AreaSection — dual [population, area] constraint | [PDF](papers/B.9+areasection-dual-population-area-constraint.pdf) |
| B.10 | County-Sticky Weights — 34% fewer county splits at 3% compactness cost | [PDF](papers/B.10+subdivision-respecting-redistricting.pdf) |
| B.11 | ApportionRegions — prime-factor bisection tree, NC 7D/7R | [PDF](papers/B.11+apportion-regions.pdf) |
| B.12 | ProportionalSection — proportionality paradox and Rodden gap | [PDF](papers/B.12+proportional-section.pdf) |
| B.13 | NestSection — senate = 2 × house spine compatibility | [PDF](papers/B.13+nestsection-nested-multi-chamber.pdf) |
| B.14 | VRASection — minority geographic alignment, post-Callais | [PDF](papers/B.14+vrasection-minority-opportunity-bisection.pdf) |
| B.15 | StabilitySection — bisection tree stability 2000–2020 | [PDF](papers/B.15+stabilitysection-cross-census-stability.pdf) |

---

## Which plan do you pick?

The "search" layer. Once you have a structure, how do you select among valid plans?

| Code | Title | PDF |
|------|-------|-----|
| B.16 | ConvergenceSweep — T=600 statutory seed formula | [PDF](papers/B.16+convergence-sweep.pdf) |
| B.17 | Parameter Sensitivity — partisanship insensitive to tuning | [PDF](papers/B.17+parameter-sensitivity.pdf) |

---

## What does the feasible space look like?

Ensemble methods: GerryChain/ReCom comparison, diagnostics, mixing time. How does the bisection plan compare to all valid plans?

| Code | Title | PDF | Note |
|------|-------|-----|------|
| G.0 | Ensemble Comparison Methodology | [PDF](papers/G.0+ensemble-methodology.pdf) | Framework |
| G.1 | GerryChain Congressional Comparison — 6 key states | [PDF](papers/G.1+gerrychain-congressional-comparison.pdf) | Real GerryChain runs |
| G.2 | Partisan Outcome Distributions | [PDF](papers/G.2+partisan-outcome-distributions.pdf) | |
| G.3 | Compactness Distribution Position | [PDF](papers/G.3+compactness-distribution-position.pdf) | |
| G.4 | Ensemble Diagnostics — R-hat, ESS, Hamming | [PDF](papers/G.4+ensemble-diagnostics-paper.pdf) | |
| G.5 | Convergence and Mixing Time Analysis | [PDF](papers/G.5+convergence-mixing-analysis.pdf) | |

**Key finding (G.1, real data):** The bisection plan sits at the compactness extremum — 0.1–0.7th percentile in WI/GA/PA/CA. NC is 50th percentile: geographic constraint makes the minimum-cut plan coincide with the ensemble median.

---

## Does it hold up under scrutiny?

Validation: does the algorithm produce robust results across resolutions, census years, parameters, and methods?

| Code | Title | PDF |
|------|-------|-----|
| C.0 | Validation Overview (Track C synthesis) | [PDF](papers/C.0+validation-overview.pdf) |
| C.1 | MAUP Sensitivity — robust across 130× unit-count range | [PDF](papers/C.1+maup-sensitivity.pdf) |
| C.2 | Cross-Census Validation — PP varies only ~10% across decades | [PDF](papers/C.2+cross-census-validation.pdf) |
| C.3 | Cross-Census Temporal Stability | [PDF](papers/C.3+temporal-stability.pdf) |
| C.4 | Twenty Years of Congressional Redistricting | [PDF](papers/C.4+longitudinal-analysis.pdf) |
| C.5 | Efficiency Gap Analysis — near-zero EG as byproduct | [PDF](papers/C.5+efficiency-gap-analysis.pdf) |
| C.6 | User Study — algorithmic maps rated fairer by public | [PDF](papers/C.6+user-study.pdf) |
| C.7 | Uncertainty Quantification — 95% CI for +22% improvement: [+15%, +29%] | [PDF](papers/C.7+uncertainty-quantification.pdf) |
| B.18 | Multi-Reapportionment Stability — what happens when states gain/lose seats | [PDF](papers/B.18+multi-reapportionment-stability.pdf) |

---

## Does it comply with voting rights law?

VRA Section 2, the 42% threshold, Gingles analysis, post-Callais disentanglement.

| Code | Title | PDF |
|------|-------|-----|
| D.0 | VRA Compliance via Edge-Weighted Partitioning | [PDF](papers/D.0+vra-compliance.pdf) |
| D.1 | The 42% Threshold — geographic limits of VRA compliance | [PDF](papers/D.1+threshold-analysis.pdf) |
| D.2 | N-Way vs. Recursive for VRA-Compliant Redistricting | [PDF](papers/D.2+nway-vs-recursive-vra.pdf) |
| D.3 | VRA–Compactness Tradeoff — real but bounded | [PDF](papers/D.3+compactness-tradeoff.pdf) |
| D.5 | Gingles Bloc Voting Methodology — expert witness guide | [PDF](papers/D.5+gingles-bloc-voting-methodology.pdf) |

---

## Does it work for state legislative maps?

State house, state senate, bicameral nesting, high-k chambers (WA 98, TX 150, NH 400).

| Code | Title | PDF |
|------|-------|-----|
| F.0 | State Legislative Redistricting Overview | [PDF](papers/F.0+state-legislative-overview.pdf) |
| F.1 | Single-Chamber State House — All 50 States | [PDF](papers/F.1+single-chamber-all-50.pdf) |
| F.2 | Bicameral Redistricting — NestSection at Scale | [PDF](papers/F.2+bicameral-nesting.pdf) |
| F.3 | Multi-Resolution for High-k Chambers | [PDF](papers/F.3+multi-resolution-high-k.pdf) |
| F.4 | State-by-State Variation in Redistricting Criteria | [PDF](papers/F.4+state-criteria-variation.pdf) |
| F.5 | Compactness: State Legislative vs Congressional | [PDF](papers/F.5+compactness-legislative-vs-congressional.pdf) |
| F.6 | VRA Compliance for State Legislative Chambers | [PDF](papers/F.6+vra-state-legislative.pdf) |

---

## What if maps were drawn differently?

Experimental extensions: multi-member districts, county representation, national redistricting, partisan systems.

| Code | Title | PDF |
|------|-------|-----|
| E.0 | Experimental Extensions Overview | [PDF](papers/E.0+experimental-overview.pdf) |
| E.1 | Multi-Member Districts and Proportional Representation | [PDF](papers/E.1+multi-member-districts.pdf) |
| E.2 | Direct County Representation | [PDF](papers/E.2+county-representation.pdf) |
| E.3 | National Redistricting Without State Boundaries | [PDF](papers/E.3+national-redistricting.pdf) |
| E.4 | Partisan Similarity Districts: Algorithmic Safe Seats | [PDF](papers/E.4+partisan-similarity-districts.pdf) |
| E.5 | Partisan Fairness Through Algorithmic Districting | [PDF](papers/E.5+party-based-allocation.pdf) |
| E.6 | International Applications | [PDF](papers/E.6+international-applications.pdf) |
| E.7 | Lessons Learned from Six Alternative Systems | [PDF](papers/E.7+lessons-learned.pdf) |

---

## Can it be enacted, and what does adoption look like?

Federal statute, state adoption pathways, court-ordered remedies, competitive elections evidence.

| Code | Title | PDF |
|------|-------|-----|
| B.02 | ApportionRegions as Geographic Completion of Huntington-Hill (the one-sentence law) | [PDF](papers/B.02+one-federal-law.pdf) |
| D.4 | Legal Implementation Framework — 50-state adoption pathways + model statute | [PDF](papers/D.4+legal-implementation.pdf) |
| C.8 | Competitive Elections — algorithmic maps produce 30% more swing districts | [PDF](papers/C.8+competitive-elections.pdf) |
| C.9 | Adoption Case Studies — Arizona, California, North Carolina | [PDF](papers/C.9+adoption-case-studies.pdf) |

See also: [`docs/legal/`](legal/) for bill text, rationale, and state-court companion.

---

## What does the whole portfolio mean?

Synthesis, guides, and practitioner materials.

| Code | Title | PDF |
|------|-------|-----|
| A.0 | National-Scale Demonstration — 50 states, 3 census decades | [PDF](papers/A.0+synthesis-metapaper.pdf) |
| A.1 | Research Portfolio Guide | [PDF](papers/A.1+portfolio-guide.pdf) |
| A.2 | Portfolio Summary | [PDF](papers/A.2+portfolio-summary.pdf) |
| A.3 | Portfolio Visualization — visual guide for non-technical audiences | [PDF](papers/A.3+portfolio-visualization.pdf) |
| A.4 | Replication Materials — AEA-compliant reproducibility package | [PDF](papers/A.4+replication-materials.pdf) |
| A.5 | Policy Brief — 4 pages for legislative staff and commissioners | [PDF](papers/A.5+policy-brief.pdf) |

---

## Track H — Ensemble Search Strategies (3 papers, panel reviews in progress)

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| H.0 | PercentileSweep — Statutory Choice of Legal Posture | [PDF](papers/H.0+percentile-sweep.pdf) | **Accepted** 3.4/4 |
| H.1 | BisectionEnsemble — Local ReCom at Each Bisection Node | [PDF](papers/H.1+bisection-ensemble.pdf) | **Accepted** 3.0/4 |
| H.2 | redist-ensemble — Rust ReCom at 2500× Speed | [PDF](papers/H.2+redist-ensemble.pdf) | **Accepted** 3.2/4 |

---

*To add a paper: place `main.tex` in `research/CODE+title/`, run `cd research && make docs`, commit `docs/papers/CODE+title.pdf`.*
