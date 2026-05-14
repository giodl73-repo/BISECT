# Research Papers

190+ papers across active tracks (A–G, I–V). PDFs open directly from the links below. LaTeX sources in [`research/tracks/`](../research/tracks/) — each series in its own track subdirectory. Some later-track drafts remain source-only and do not yet have PDFs in this index.

Papers are organised by the question they answer. Each paper's series code is shown for cross-reference. To recompile: `cd research && make docs`

Current atlas-to-paper alignment work is tracked in
[`docs/papers/ATLAS-ALIGNMENT.md`](papers/ATLAS-ALIGNMENT.md).
The slower publication-quality review ledger is
[`docs/papers/PAPER-QUALITY-REVIEW.md`](papers/PAPER-QUALITY-REVIEW.md).
The portfolio-wide algorithm-paper triage scorecard is
[`docs/papers/ALGORITHM-PAPER-SCORECARD.md`](papers/ALGORITHM-PAPER-SCORECARD.md).

---

## Track B — Algorithm Foundations

The mathematical and computational foundations. Start here if you are new to the project.

| Code | Title | PDF |
|------|-------|-----|
| B.0 | Bakeoff — all 8 algorithm modes on 6 competitive states | [PDF](papers/B.0+algorithm-design-overview.pdf) |
| B.1 | Recursive Bisection for Congressional Redistricting | [PDF](papers/B.1+recursive-bisection.pdf) |
| B.2 | Edge-Weighted Recursive Bisection (+22% compactness) | [PDF](papers/B.2+edge-weighted-bisection.pdf) |
| B.3 | Single-Objective vs. Multi-Constraint METIS | [PDF](papers/B.3+multi-vs-edge.pdf) |
| B.4 | Equivalence of Recursive and N-Way Bisection | [PDF](papers/B.4+adaptive-bisection.pdf) |
| B.5 | N-Way vs. Recursive Bisection: General Comparison | [PDF](papers/B.5+nway-vs-recursive-general.pdf) |
| B.6 | Computational Complexity: NP-hard (planar bisection), O(n log k) runtime, O(log n) approximation lower bound | [PDF](papers/B.6+computational-complexity.pdf) |
| B.7 | Solution Space and Seed Sensitivity (CV < 2% for 96% of states) | [PDF](papers/B.7+solution-space-and-seed-sensitivity.pdf) |

---

## Track T — Plan Construction

The `--structure` layer of the compositor. Every paper here answers: given k districts, how is the map or bisection tree constructed?

| Code | Title | PDF |
|------|-------|-----|
| T.1 | GeoSection — ratio-optimal first bisection, isoperimetric normalisation | [PDF](papers/T.1+geosection-ratio-optimal-bisection.pdf) |
| T.2 | AreaSection — dual [population, area] constraint | [PDF](papers/T.2+areasection-dual-population-area-constraint.pdf) |
| T.3 | County-Sticky Weights — 34% fewer county splits at 3% compactness cost | [PDF](papers/T.3+subdivision-respecting-redistricting.pdf) |
| T.4 | ApportionRegions — prime-factor bisection tree, NC 7D/7R | [PDF](papers/T.4+apportion-regions.pdf) |
| T.5 | ProportionalSection — proportionality paradox and Rodden gap | [PDF](papers/T.5+proportional-section.pdf) |
| T.6 | NestSection — senate = 2 × house spine compatibility | [PDF](papers/T.6+nestsection-nested-multi-chamber.pdf) |
| T.7 | VRASection — minority geographic alignment, post-Callais | [PDF](papers/T.7+vrasection-minority-opportunity-bisection.pdf) |
| T.8 | StabilitySection — bisection tree stability 2000–2020 | [PDF](papers/T.8+stabilitysection-cross-census-stability.pdf) |
| T.9 | Multi-Reapportionment Stability — what happens when states gain/lose seats | [PDF](papers/T.9+multi-reapportionment-stability.pdf) |
| T.10 | Centroidal Voronoi Districts (Phase 1 + Phase 2) — graph-distance and geographic Euclidean CVD | [PDF](papers/T.10+centroidal-voronoi.pdf) |
| T.11 | CVD Geographic — projected-centroid district construction | [PDF](papers/T.11+cvd-geographic.pdf) |
| T.12 | BFS Region-Growing — greedy geographic district packing | [PDF](papers/T.12+bfs-growth.pdf) |
| T.13 | Moving-Knife Redistricting — Reock-oriented fair-division construction | [PDF](papers/T.13+moving-knife.pdf) |
| T.14 | Spectral Partitioning — deterministic Fiedler-vector construction baseline | [PDF](papers/T.14+spectral-partitioning.pdf) |
| T.15 | Capacity-Constrained Clustering — population-capacity clusters with audit sidecars | [PDF](papers/T.15+capacity-constrained-clustering.pdf) |
| T.16 | Hierarchical Regionalization — agglomerative connected-region construction | [PDF](papers/T.16+hierarchical-regionalization.pdf) |
| T.17 | Flow-Based Construction — capacity/cost flow assignment with infeasibility witnesses | [PDF](papers/T.17+flow-based-construction.pdf) |

T.14-T.17 are implemented as audited vertical slices with draft PDFs and
paper-level simulated review/revision artifacts. T.14 now also has a
method-produced synthetic spectral RPLAN package, and T.14-T.17 have
benchmark-tier 100-unit synthetic packages with transcripts and verifier
manifests under `docs/examples/rplan-method-packages/` and
`docs/examples/rplan-benchmark-packages/`. Writing goal:
[`2026-05-11-algorithm-family-paper-writing-goal.md`](specs/2026-05-11-algorithm-family-paper-writing-goal.md).
Implementation roadmap:
[`2026-05-10-algorithm-family-roadmap.md`](specs/2026-05-10-algorithm-family-roadmap.md).

---

## Track U — Search and Optimization

The search, optimization, certification, and algorithm-selection layer. Once construction methods exist, how do you choose, improve, certify, or compare candidate plans?

| Code | Title | PDF | Note |
|------|-------|-----|------|
| U.0 | Search and Optimization Overview — taxonomy from construction to certification | [PDF](papers/U.0+search-optimization-overview.pdf) | reviewed draft |
| U.1 | ConvergenceSweep — T=600 statutory seed formula | [PDF](papers/U.1+convergence-sweep.pdf) | `--search convergence` |
| U.2 | Parameter Sensitivity — evidence protocol for tuning robustness | [PDF](papers/U.2+parameter-sensitivity.pdf) | synthetic sweep package added; real 50-state sweep pending |
| U.3 | Simulated Annealing Bisection — cooling the edge-cut objective in the bisection tree | [PDF](papers/U.3+simulated-annealing.pdf) | heuristic/local-search boundary |
| U.4 | Parallel Tempering — multi-chain replica exchange MCMC | [PDF](papers/U.4+parallel-tempering.pdf) | synthetic PT package added; production CLI pending |
| U.5 | Adaptive Multi-scale MCMC — self-tuning alpha via Robbins-Monro | [PDF](papers/U.5+adaptive-multiscale.pdf) | `bisect-multiscale` substrate |
| U.6 | Exact Redistricting via ILP — model-scoped certificates for small instances | [PDF](papers/U.6+ilp-redistricting.pdf) | solver package required |
| U.7 | Pareto-Optimal Redistricting — transparent trade-offs via NSGA-II | [PDF](papers/U.7+pareto-redistricting.pdf) | `bisect-pareto` selected-frontier package |
| U.8 | PercentileSweep — Statutory Choice of Legal Posture | [PDF](papers/U.8+percentile-sweep.pdf) | legal posture selection |
| U.9 | BisectionEnsemble — Local ReCom at Each Bisection Node | [PDF](papers/U.9+bisection-ensemble.pdf) | local ensemble search |
| U.10 | bisect-ensemble — Rust ReCom at 2500× Speed | [PDF](papers/U.10+bisect-ensemble.pdf) | high-performance search implementation |
| U.11 | Resolution-Aware Redistricting — Geographic Granularity as a First-Class Parameter | [PDF](papers/U.11+resolution-aware.pdf) | resolution selection |
| U.12 | Algorithm-Selection Matrix — choosing construction, search, exact, ensemble, and audit paths | [PDF](papers/U.12+algorithm-selection-matrix.pdf) | reviewed draft |
| U.13 | Exact-vs-Heuristic Certification — what proofs, bounds, and audits establish | [PDF](papers/U.13+exact-vs-heuristic-certification.pdf) | reviewed draft |
| U.14 | Multi-Objective Selection — selecting among trade-offs and sampled frontiers | [PDF](papers/U.14+multi-objective-selection.pdf) | reviewed draft |
| U.15 | Legal Postures for Search — claim discipline for search and optimization choices | [PDF](papers/U.15+legal-postures-for-search.pdf) | reviewed draft |
| U.16 | Branch-And-Cut Redistricting — connectivity cuts, separation, and solver reports | [PDF](papers/U.16+branch-and-cut-redistricting.pdf) | reviewed draft; public golden and benchmark packages |
| U.17 | Branch-And-Price Redistricting — column generation and exact fixture packages | [PDF](papers/U.17+branch-and-price-redistricting.pdf) | reviewed draft; public golden package |
| U.18 | Large-Neighborhood Search — local improvement, tabu/LNS scaffolding, and repair hooks | [PDF](papers/U.18+large-neighborhood-search.pdf) | reviewed draft; public lineage and benchmark packages |
| U.19 | Evolutionary Search Comparison — validity-preserving crossover/mutation and selected frontier audits | [PDF](papers/U.19+evolutionary-search-comparison.pdf) | reviewed draft; public selected-frontier package |
| U.20 | Plan Audit Certificates — RPLAN/RCTX audit certificates and lineage fixed point | [PDF](papers/U.20+plan-audit-certificates.pdf) | reviewed draft; public package corpus and audit benchmark |

U.0 and U.12-U.20 are reviewed drafts with PDFs and paper-level simulated
review/revision artifacts. Search/optimization additions through U.20 are
implemented as audited vertical slices where they touch final plans. U.16-U.20
now have public RPLAN package evidence through the golden, method, and benchmark package corpora and
the `rplan verify-certificate` / `bisect verify --manifest` bridge. Writing goal:
[`2026-05-11-algorithm-family-paper-writing-goal.md`](specs/2026-05-11-algorithm-family-paper-writing-goal.md).
Implementation roadmap:
[`2026-05-10-algorithm-family-roadmap.md`](specs/2026-05-10-algorithm-family-roadmap.md).
RPLAN factoring and U.20 audit contract:
[`2026-05-10-rplan-incubation.md`](specs/2026-05-10-rplan-incubation.md),
[`2026-05-10-rplan-v0.2-schema.md`](specs/2026-05-10-rplan-v0.2-schema.md),
[`2026-05-10-plan-audit-certificates.md`](specs/2026-05-10-plan-audit-certificates.md).

---

## Track V — Vote Counting, Certification, And Public Verification

The election-counting fixed point. RPLAN verifies the plan; RCOUNT verifies the
count ledgers, canvass arithmetic, precinct lineage, tamper-evident hashes,
privacy-safe inclusion proofs, audit replay, and district vote aggregation.
Roadmap:
[`2026-05-12-v-election-audit-paper-track.md`](specs/2026-05-12-v-election-audit-paper-track.md).
RCOUNT specs:
[`2026-05-12-rcount-incubation.md`](specs/2026-05-12-rcount-incubation.md),
[`2026-05-12-rcount-certification-research.md`](specs/2026-05-12-rcount-certification-research.md).

| Code | Title | PDF | Note |
|------|-------|-----|------|
| V.0 | RCOUNT Overview — reproducible election count packages | — | planned |
| V.1 | Canvass Arithmetic — unofficial returns to certified totals | — | planned |
| V.2 | Precinct Lineage Across Elections — splits, merges, renames, movement | — | planned |
| V.3 | Tamper-Evident Precinct And Batch Hashing — public roots and proofs | — | planned |
| V.4 | Privacy-Safe Voter Inclusion Proofs — inclusion without coercion receipts | — | planned |
| V.5 | Ballot Manifest Verification — batch/container accounting | — | planned |
| V.6 | CVR-To-Summary Reconciliation — cast vote records to public summaries | — | planned |
| V.7 | Replayable Risk-Limiting Audits — public seed, sample replay, stopping rules | — | planned |
| V.8 | District Vote Aggregation With RPLAN — count hashes plus plan hashes | — | planned |
| V.9 | Count-System Interoperability — vendor exports to RCOUNT | — | planned |
| V.10 | Certification Evidence Matrix — law, systems, and public data availability | — | planned |
| V.11 | Performance And Parallel Verification In Rust — full-election verification | — | planned |

---

## What does the feasible space look like?

Ensemble methods: GerryChain/ReCom comparison, diagnostics, mixing time. How does the bisection plan compare to all valid plans?

| Code | Title | PDF | Note |
|------|-------|-----|------|
| G.0 | Ensemble Comparison Methodology | [PDF](papers/G.0+ensemble-methodology.pdf) | Framework |
| G.1 | GerryChain Congressional Comparison — 6 key states | [PDF](papers/G.1+gerrychain-congressional-comparison.pdf) | active synthetic + missing real-evidence packages |
| G.2 | Partisan Outcome Distributions | [PDF](papers/G.2+partisan-outcome-distributions.pdf) | active synthetic + missing real-evidence packages |
| G.3 | Compactness Distribution Position | [PDF](papers/G.3+compactness-distribution-position.pdf) | active synthetic + missing real-evidence packages |
| G.4 | Ensemble Diagnostics — R-hat, ESS, Hamming | [PDF](papers/G.4+ensemble-diagnostics-paper.pdf) | |
| G.5 | Convergence and Mixing Time Analysis | [PDF](papers/G.5+convergence-mixing-analysis.pdf) | diagnostic, not proof of mixing |
| G.6 | Short-Burst Optimization for Minimum-Edge-Cut Redistricting | [PDF](papers/G.6+short-burst.pdf) | **Accepted** 3.0/4 — `--search short-burst` |
| G.7 | Sequential Monte Carlo for Calibrated Redistricting Ensembles | [PDF](papers/G.7+smc-redistricting.pdf) | First draft — `bisect-smc` crate implemented |
| G.8 | Flip Proposals for Local Sensitivity Analysis | [PDF](papers/G.8+flip-proposals.pdf) | **Accepted** 3.4/4 — `--search flip` |
| G.9 | Forest ReCom: Reversible Recombination via Spanning Forest Sampling | [PDF](papers/G.9+forest-recom.pdf) | **Accepted** 3.4/4 — `--search forest-recom` |
| G.10 | Merge-Split MCMC: Explicit Reversibility via Two-Tree Acceptance Ratio | [PDF](papers/G.10+merge-split.pdf) | **Accepted** 3.2/4 — `--search merge-split` |
| G.11 | Multi-scale MCMC: Hierarchical Mixing for Large-k Redistricting | [PDF](papers/G.11+multiscale-mcmc.pdf) | **Accepted** 3.0/4 — `--search multiscale` |
| G.12 | Short-Burst with Calibrated Chains | [PDF](papers/G.12+short-burst-chains.pdf) | short-burst diagnostic boundary |
| G.13 | VRA-Aware Ensemble — chains preserving majority-minority districts | [PDF](papers/G.13+vra-aware-ensemble.pdf) | **Accepted** 3.8/4 — `--search vra-recom` |
| G.14 | A Practitioner's Comparison of Redistricting Ensemble Algorithms | [PDF](papers/G.14+ensemble-comparison.pdf) | **Accepted** 4.0/4 |

**Evidence package (G.1-G.3):** `docs/examples/g-ensemble-evidence-packages/G.1-G.3+missing-evidence/`
is hash-bound and validates the current gap: headline compactness and partisan
percentiles still require archived external traces, election/metric inputs,
diagnostics, and RPLAN/RCTX plan packages before being cited as final findings.

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
| D.6 | Prison Gerrymandering — population adjustment methodology | [PDF](papers/D.6+prison-gerrymandering.pdf) |
| D.7 | Section 203 Language Minorities — coverage, district design, bilingual access | [PDF](papers/D.7+section-203-language-minorities.pdf) |
| D.8 | Post-Shelby VRA Landscape — algorithmic redistricting as transparency substitute | [PDF](papers/D.8+post-shelby-landscape.pdf) |

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

## Track I — Incumbency (5 papers)

How algorithmically-drawn maps affect incumbents — an algorithm with no incumbency input whose geometric outcomes can be measured against enacted maps.

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| I.0 | Incumbency and Algorithmic Redistricting: Overview | [PDF](papers/I.0+incumbency-overview.pdf) | **Accepted** |
| I.1 | Incumbent-Pairing Probability | [PDF](papers/I.1+incumbent-pairing.pdf) | Minor Revision |
| I.2 | Safe-Seat Creation and Destruction | [PDF](papers/I.2+safe-seat-creation.pdf) | Minor Revision |
| I.3 | Open-Seat Effects and Retirement Incentives | [PDF](papers/I.3+open-seat-effects.pdf) | Minor Revision |
| I.4 | Incumbency Protection as a Legal Redistricting Criterion | [PDF](papers/I.4+incumbency-legal-criterion.pdf) | **Accepted** |

---

## Track J — Apportionment Methods (7 papers)

Congressional seat apportionment — how seats are allocated to states before districts are drawn. Documents the `bisect-apportion` crate.

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| J.0 | Apportionment Methods: Overview | [PDF](papers/J.0+apportionment-overview.pdf) | 2020 Census package verified |
| J.1 | Huntington-Hill: The Federal Method | [PDF](papers/J.1+huntington-hill.pdf) | 2020 Census package verified |
| J.2 | Webster Method and Sainte-Laguë Equivalence | [PDF](papers/J.2+webster-method.pdf) | Minor Revision |
| J.3 | Adams Method and Smallest-State Bias | [PDF](papers/J.3+adams-method.pdf) | Minor Revision |
| J.4 | Jefferson/D'Hondt and Large-State Bias | [PDF](papers/J.4+jefferson-dhondt.pdf) | Minor Revision |
| J.5 | Apportionment Paradoxes and Balinski-Young Impossibility | [PDF](papers/J.5+apportionment-paradoxes.pdf) | Minor Revision |
| J.6 | bisect-apportion: Current Implementation Boundary | [PDF](papers/J.6+bisect-apportion-implementation.pdf) | 2020 Census package verified |

---

## Track K — Compactness Measures (8 papers)

One paper per compactness metric: mathematical definition, properties, implementation in `bisect-analysis`, empirical comparison across B-series algorithms, legal usage. All metrics accessible via `bisect label-analyze --types compactness`.

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| K.0 | Compactness Taxonomy and Overview | [PDF](papers/K.0+compactness-overview.pdf) | exact-MBC smoke package added |
| K.1 | Polsby-Popper Score | [PDF](papers/K.1+polsby-popper.pdf) | **Accepted** |
| K.2 | Reock Score | [PDF](papers/K.2+reock.pdf) | exact-MBC smoke package added |
| K.3 | Convex Hull Ratio | [PDF](papers/K.3+convex-hull.pdf) | **Accepted** |
| K.4 | Schwartzberg Score | [PDF](papers/K.4+schwartzberg.pdf) | **Accepted** |
| K.5 | Length-Width Ratio | [PDF](papers/K.5+length-width.pdf) | MBR/AABB implementation boundary added |
| K.6 | Population-Weighted Compactness | [PDF](papers/K.6+population-weighted-compactness.pdf) | **Accepted** |
| K.7 | Multi-Metric Composite and Court Usage Guide | [PDF](papers/K.7+composite-court-guide.pdf) | exact/proxy Reock disclosure |

---

## Track L — Partisan Fairness (7 papers)

Partisan fairness metrics — how algorithmically-drawn maps compare to enacted maps across all six major metrics. Post-Rucho legal landscape. All metrics via `bisect label-analyze --types partisan`.

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| L.0 | Partisan Fairness Metrics: Overview and Framework | [PDF](papers/L.0+partisan-fairness-overview.pdf) | **Accepted** |
| L.1 | Efficiency Gap: Dedicated Treatment | [PDF](papers/L.1+efficiency-gap.pdf) | **Accepted** |
| L.2 | Mean-Median Difference | [PDF](papers/L.2+mean-median.pdf) | **Accepted** |
| L.3 | Partisan Bias and Swing Ratio | [PDF](papers/L.3+partisan-bias.pdf) | **Accepted** |
| L.4 | Declination | [PDF](papers/L.4+declination.pdf) | **Accepted** |
| L.5 | Seats-Votes Curve and Responsiveness | [PDF](papers/L.5+seats-votes-curve.pdf) | **Accepted** |
| L.6 | Proportionality vs. Majoritarianism | [PDF](papers/L.6+proportionality-majoritarianism.pdf) | **Accepted** |

---

## Track M — Community Character Weights (9 papers)

Operationalizing communities of interest as METIS edge weights. Each paper defines one community character signal (housing type, land use, commuting shed, topography, etc.), derives a cosine similarity metric from publicly available data, and provides legal grounding for redistricting use. CLI: `--weights-override housing-character` etc.

| Code | Title | PDF | Status |
|------|-------|-----|--------|
| M.0 | Community Character Weighting: Framework for Communities of Interest | [PDF](papers/M.0+community-character-overview.pdf) | **Accepted** |
| M.1 | Economic Character Edge Weights via LODES | [PDF](papers/M.1+economic-character-lodes.pdf) | PDF published; empirical Phase 2 pending |
| M.9 | Economic Character Edge Weights via LODES — community similarity as edge modifier | [PDF](papers/M.9+economic-character-weights.pdf) | **Accepted** |
| M.3 | Housing Character Edge Weights via ACS | [PDF](papers/M.3+housing-character-acs.pdf) | PDF published; empirical design deferred |
| M.6 | Administrative Zone Co-membership (School/Fire/Electric Districts) | [PDF](papers/M.6+administrative-zone-membership.pdf) | **Accepted** |

---

*To add a paper: place `main.tex` in `research/tracks/[track]/CODE+title/`, run pdflatex+bibtex in the paper directory, copy the PDF to `docs/papers/CODE+title.pdf`, and update this file.*
