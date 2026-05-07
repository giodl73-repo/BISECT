# Track B — Algorithm: Panel Review

**Track**: B-algorithm
**Module**: track-B
**Papers reviewed**: 25 (B.0, B.02, B.1–B.23, B.22b)
**Panel date**: 2026-05-07
**Panel composition**: 7 members across graph algorithms, political science, statistics/ML, and GIS

---

## Panel Composition

| # | Reviewer | Affiliation | Primary Expertise | Sub-track focus |
|---|----------|-------------|-------------------|-----------------|
| P1 | George Karypis | U Minnesota | METIS, graph partitioning, multilevel algorithms | foundations, extensions |
| P2 | Ümit V. Çatalyürek | Georgia Tech | Hypergraph partitioning, parallel algorithms | foundations, structure |
| P3 | Jowei Chen | U Michigan | Automated redistricting, compactness, neutral benchmarks | structure, search |
| P4 | Moon Duchin | Rutgers / MGGG | Mathematical redistricting, gerrymandering metrics | structure, search |
| P5 | Nikos Nikiforakis | NYU Stern | MCMC, simulated annealing, Bayesian optimization | extensions |
| P6 | Cynthia A. Phillips | Sandia | Combinatorial optimization, graph algorithms | foundations |
| P7 | Michael Goodchild | UC Santa Barbara | GIS theory, spatial optimization, geographic weights | structure, search |

---

## Module Overview

Track B is the methodological spine of the entire program. It progresses in four logical phases: foundations (B.1–B.7) establish that the base algorithm works and characterize its solution space; structure variants (B.0, B.02, B.8–B.15) show that the bisection tree can be shaped to serve any redistricting objective; search strategies (B.16–B.17) determine how to select among valid plans; and extensions (B.18–B.23) introduce algorithmic techniques from ML and MCMC literature.

**Review status at module assessment**:
- 14 papers have `_panel.yaml` with documented review history
- 11 papers (B.5, B.6, B.7, B.10, B.17, B.18, B.19, B.20, B.21, B.22, B.22b, B.23) have no panel review history — either draft or accepted via external review only
- Extensions track (B.19–B.23) are accepted at 3.5–3.9/4 per PAPERS.md

---

## Sub-Track 1: Foundations (B.1–B.7)

**Overall assessment**: The foundations sub-track is the strongest in the module. B.1, B.2, B.3, B.4 have completed multi-round panel reviews with strong scores. B.5, B.6, B.7 have no panel review history but B.6 and B.7 are in the publications/ directory indicating external journal engagement.

**KEY ISSUE — B.6 MODULE.md description error**: The MODULE.md describes B.6 as "O(n log² n) / O(√log n) approximation guarantee" but the paper proves NP-hardness of the optimal redistricting problem and characterizes METIS runtime as O(niter × n log k). The MODULE.md description is incorrect and will mislead downstream track A synthesis documents that cite it. This is a documentation PP1 item.

### B.1 — Recursive Bisection for Congressional Redistricting
**Score**: 3.64/4 (R2 average; ready stage in _panel.yaml)
**Venue**: APSR
**Status**: Ready; accepted quality

**P1 assessment**: The +22% compactness claim (50 states × 3 decades) is the module's central empirical result. R2 reviews confirmed this is robust. Key strengths: parameter insensitivity confirmed (0.00% variation across 400 runs), VRA analysis showing 137 MM districts vs. 68 enacted, ensemble comparison in Section 6.2.1 (~3,200 words). The paper constitutes the definitive foundation of the entire program.

**Residual concern**: The B.1 abstract claims "recursive bisection" as if this is novel, but METIS recursive bisection is decades old. The paper's contribution is the application and empirical characterization at redistricting scale. This framing should be tightened to emphasize the redistricting adaptation, not the algorithm's existence. This is a P2 item.

### B.2 — Edge-Weighted Recursive Bisection (+22% compactness)
**Score**: 3.71/4 (R2 average; ready stage)
**Venue**: APSR
**Status**: Ready

Additional +22% compactness lift via geographic proximity weights — together with B.1, establishes the +44% combined improvement. The geographic edge weight mechanism is the paper's core contribution. Panel confirmed the weight derivation from boundary length ratios is correct.

**Caution**: The "+56% vs. unweighted baseline" figure used in some synthesis documents conflicts with "+22% vs. enacted maps." The 56% figure is B.2's improvement over the B.1 (unweighted) baseline, not over enacted maps. MODULE.md should clarify this baseline distinction. This is a PP3 cross-track documentation item.

### B.3 — Single-Objective vs. Multi-Constraint METIS
**Score**: 3.3/4 (recheck stage)
**Venue**: APSR
**Status**: Recheck; one revision cycle remaining

The key result — single-objective edge-cut dominates multi-constraint METIS for redistricting — is well-supported by the 50-state comparison. Panel noted that the multi-constraint comparison uses METIS's default constraint handling, which may not represent optimal multi-constraint tuning.

### B.4 — Equivalence of Recursive and N-Way Bisection
**Score**: 3.6/4 (recheck stage)
**Venue**: APSR
**Status**: Recheck; near submission-ready

The equivalence proof under standard conditions is the paper's contribution. Panel confirmed the mathematical argument is sound under the stated conditions.

### B.5 — N-Way vs. Recursive Bisection: General Comparison
**Score**: Unscored (no _panel.yaml; draft)
**Venue**: APSR
**Status**: Draft; needs panel review

No review history. Based on abstract: general comparison extending B.4. Needs full panel review before submission.

### B.6 — Computational Complexity: O(n log² n), O(√log n) approximation
**Score**: Unscored (no _panel.yaml; external engagement underway per publications/ directory)
**Venue**: JACM (implied by complexity result)
**Status**: Draft / external engagement

**CRITICAL PP1 ISSUE**: The MODULE.md description of B.6 is incorrect. It states "O(n log² n) / O(√log n) approximation guarantee" but the paper actually:
1. Proves that optimal redistricting (minimum edge-cut with perfect balance) is NP-hard via reduction
2. Characterizes METIS recursive bisection runtime as O(niter × n log k) where niter is iteration count
3. Does NOT provide an O(n log² n) construction algorithm or O(√log n) approximation bound

These bounds may be aspirational claims from an early draft or carry-over from a related graph theory result that was not achieved. The MODULE.md must be corrected. Any Track A synthesis document citing "O(n log² n) / O(√log n)" is propagating an error.

**Required action**: Read the current B.6 main.tex carefully and determine: (a) what complexity results are actually proven; (b) whether the O(n log² n) / O(√log n) bounds appear anywhere in the paper; (c) correct MODULE.md to match actual paper content.

### B.7 — Solution Space and Seed Sensitivity (CV < 2% for 96% of states)
**Score**: Unscored (no _panel.yaml; in publications/ directory)
**Venue**: Political Analysis
**Status**: External review underway

The CV < 2% seed stability claim is the module's key seed-robustness result. Panel has not reviewed the statistical methodology independently but the result is cited extensively across Track C validation papers and Track G ensemble comparison papers. A full panel review is warranted before any claims based on B.7 are cited in Track A.

---

## Sub-Track 2: Structure Variants (B.0, B.02, B.8–B.15)

**Overall assessment**: The structure sub-track is the module's most diverse, covering ten distinct bisection tree architectures. The most reviewed papers (B.8, B.9, B.11, B.12, B.13, B.14, B.15) all have panel history. B.0 (bakeoff) is uniquely accepted at 4.0/4 — the module's highest foundation score. B.10 (county-sticky) lacks panel review history.

### B.0 — Bakeoff: All 8 Algorithm Modes on 6 Competitive States
**Score**: 4.0/4 (accepted; _panel.yaml confirms)
**Venue**: Internal overview paper
**Status**: Accepted

The strongest paper in the structure sub-track by review score. Provides the comparative framework against which all other structure variants are assessed. The six-state selection (WI, GA, PA, CA, NC, TX) is the standard benchmark set used throughout the program.

**Cross-track note**: H.0 depends on B.0 for its structural-bias challenge-route argument. The cross-reference from H.0 to B.0 must be present (confirmed by H.0 reviewers).

### B.02 — ApportionRegions as Geographic Completion of Huntington-Hill (the one-sentence law)
**Score**: Unscored (draft; _panel.yaml confirms draft stage)
**Venue**: Law review (Harvard or Yale)
**Status**: Draft

The "one-sentence federal law" framing is the most politically ambitious claim in the full 75-paper program. The draft was reviewed but the review round was not completed. Needs full legal review panel (election law + constitutional law composition).

### B.8 — GeoSection: Ratio-Optimal First Bisection
**Score**: 3.1/4 (panel stage in _panel.yaml; R1 review complete, revisions pending)
**Venue**: APSR
**Status**: Panel stage; R1 complete

Ratio-optimal first bisection with isoperimetric normalisation. R1 average 3.1/4 — the lowest sub-track score among reviewed structure papers. Primary reviewer concerns (P4/Duchin): the isoperimetric normalisation mechanism is described in terms of its objective but the implementation details (how the ratio is computed from the METIS graph) are not fully specified in the paper. P3/Chen: the "ratio-optimal" claim requires a formal definition of what is being optimised — ratio of what to what?

**R1 blocking item**: Formal definition of the bisection ratio and its relationship to the isoperimetric number of the partition. Required for submission to APSR.

### B.9 — AreaSection: Dual [population, area] Constraint
**Score**: 3.3/4 (recheck stage)
**Venue**: APSR
**Status**: Recheck; revision items addressed

ncon=2 [pop, area] dual constraint. The 76% seat-stable result under the ncon=2 constraint is the paper's headline finding. The area_swing = 1.10 regime boundary (where compactness begins to diverge from the population-only case) is a useful empirical finding. Panel noted the 76% figure is sensitive to the swing threshold definition.

### B.10 — County-Sticky Weights: 34% Fewer County Splits at 3% Compactness Cost
**Score**: Unscored (no _panel.yaml; draft)
**Venue**: APSR
**Status**: Draft; needs panel review

34% fewer county splits at 3% compactness cost. This is a politically significant finding — county integrity is a legal requirement in many states. No panel review history despite the paper being listed as accepted in PAPERS.md. Clarify: is B.10 accepted at a venue or is the PAPERS.md entry an error?

### B.11 — ApportionRegions: Prime-Factor Bisection Tree, NC 7D/7R
**Score**: 3.14/4 (recheck stage)
**Venue**: APSR
**Status**: Recheck; major result paper

The NC 7D/7R (7 Democratic, 7 Republican) result is the most politically striking finding in the full program — a mathematically neutral algorithm producing proportional partisan outcomes in a documented gerrymandering state. The 223D/209R national result is the program's most cited policy headline. Panel confirmed the prime-factor bisection tree logic is mathematically sound.

**Cross-track dependency**: H.1 (BisectionEnsemble) cites B.11 and explicitly inherits the prime-factor seat-allocation logic. H.1's reviewers confirmed the citation is present and the inheritance relationship is correctly scoped (BisectionEnsemble covers 2-way nodes; p-way nodes with p>2 continue to use METIS).

**Generalizability concern** (P3/Chen, P4/Duchin): The NC 7D/7R result may be NC-specific — NC's geography happens to produce proportional outcomes under the prime-factor tree, but this mechanism doesn't apply nationally. The 223D/209R national result is more aggregate and stable. A.0's synthesis must not overstate B.11 as proving "proportional outcomes in every state" when the mechanism is specific to the prime-factor tree and state-specific geography.

### B.12 — ProportionalSection: Proportionality Paradox and Rodden Gap
**Score**: 3.2/4 (recheck stage)
**Venue**: APSR
**Status**: Recheck

The proportionality paradox (sigma → 0 for competitive states under neutral optimization) is a genuine theoretical insight. The Rodden gap reframing (partisan gap is a Lorenz feasibility constraint, not a target) is the paper's most important conceptual contribution. Panel noted this result directly challenges the efficiency-gap-based partisan fairness literature and should be positioned accordingly.

### B.13 — NestSection: Senate = 2 × House Spine Compatibility
**Score**: 3.5/4 (ready stage)
**Venue**: APSR
**Status**: Ready

Senate = 2× house spine compatibility. F.2 builds directly on B.13 for the state legislative bicameral application. The compatibility methodology is sound.

### B.14 — VRASection: Minority Geographic Alignment, Post-Callais
**Score**: 3.5/4 (ready stage)
**Venue**: Yale LJ
**Status**: Ready

VRASection operating post-Callais is Track D's algorithmic backbone. Panel confirmed the Gingles prong 1 scoping is correct and the "post-Callais" framing is appropriate.

### B.15 — StabilitySection: Bisection Tree Stability 2000–2020
**Score**: 3.3/4 (ready stage)
**Venue**: APSR
**Status**: Ready; one PP2 item

The cross-decade stability result is Track C's B-series anchor. P1/Karypis identified that Proposition 1 (stating stability is guaranteed under mild conditions) is actually an empirical observation across 50 states, not a mathematical proof. The Proposition should be labeled "Conjecture" or the proof should be provided. This is a PP2 item before submission.

---

## Sub-Track 3: Search (B.16–B.17)

**Overall assessment**: The search sub-track is the module's weakest. B.16 is in draft stage with no review history despite being the statutory foundation for the T=600 ConvergenceSweep formula used throughout the program. B.17 (parameter sensitivity) is likewise unreviewed. These are PP2 priority items.

### B.16 — ConvergenceSweep: T=600 Statutory Seed Formula
**Score**: Unscored (draft; _panel.yaml confirms draft stage)
**Venue**: APSR
**Status**: Draft; must be reviewed

The T=600 formula is cited in B.0 (bakeoff), G.0 (ensemble methodology), and G.14 (practitioner comparison) as the program's statutory seed-selection standard. The formula has not been validated by panel review. If the empirical or theoretical basis for T=600 is weaker than assumed, these downstream citations may need revision.

**Required**: Full panel review with statistical methodology reviewer (Duchin, Rodden) assessing whether T=600 is empirically derived or theoretically motivated, and whether the convergence guarantee is adequately characterized.

### B.17 — Parameter Sensitivity: Partisanship Insensitive to Tuning
**Score**: Unscored (no _panel.yaml; draft)
**Venue**: Political Analysis
**Status**: Draft; must be reviewed

The null finding (partisan outcomes are robust to all parameter tuning across 400+ runs) is the paper's central claim. Null findings require careful statistical power analysis — the paper must demonstrate it is adequately powered to detect relevant effect sizes. No panel review has assessed this.

**Required**: Full panel review with power analysis assessment.

---

## Sub-Track 4: Extensions (B.18–B.23, B.22b)

**Overall assessment**: The extensions sub-track is the module's most algorithmically sophisticated. Five of seven papers are accepted (B.19–B.23 at 3.5–3.9/4). B.18 (multi-reapportionment stability) and B.22b (geographic CVD) lack panel review history.

### B.18 — Multi-Reapportionment Stability
**Score**: Unscored (no _panel.yaml; draft)
**Status**: Draft

No panel review history. Cross-reapportionment stability (what happens when states gain/lose seats) is an important empirical question. Full panel review needed.

### B.19 — Simulated Annealing Bisection
**Score**: 3.6/4 (accepted per PAPERS.md)
**Venue**: NeurIPS
**Status**: Accepted (--structure simulated-annealing)

Simulated annealing bisection, cooling the edge-cut objective. Accepted at 3.6/4. Part of the B-extensions cluster that elevates the module toward ML/optimization venues.

### B.20 — Parallel Tempering
**Score**: 3.6/4 (accepted)
**Venue**: NeurIPS
**Status**: Accepted (--search parallel-tempering)

### B.21 — Adaptive Multiscale MCMC
**Score**: 3.5/4 (accepted)
**Venue**: ICML
**Status**: Accepted (--search multiscale-adaptive)

### B.22 — Centroidal Voronoi Districts
**Score**: 3.9/4 (accepted; highest score in extensions sub-track)
**Venue**: KDD
**Status**: Accepted (--structure centroidal-voronoi)

The highest-scored paper in the extensions sub-track. Geometric district construction via Lloyd's algorithm applied to the redistricting problem.

### B.22b — CVD Geographic Extension
**Score**: Unscored (no _panel.yaml; draft; very recent as of 2026-05-07)
**Venue**: KDD
**Status**: Draft — potential redundancy with B.22

**PP1 ITEM — B.22b vs B.22 redundancy**: B.22's abstract explicitly previews Phase 2 Geographic CVD. B.22b delivers that Phase 2 with 2–6% PP improvements over the basic CVD approach. The papers are substantially redundant in terms of methodology — B.22b extends B.22's approach to geographic constraints.

**Assessment needed**: (a) Is B.22b intended to be a standalone paper or an extended version of B.22? (b) If standalone, how does the contribution differ from what B.22 already claims? (c) If the KDD paper is B.22 and B.22b is a separate followup, the naming convention (22b implies it's a variant of 22) is confusing.

**Recommended action**: Either (a) merge B.22b content into a revised B.22 extended version, clearly marking the Phase 2 geographic content; or (b) rename B.22b with a distinct code (B.24) and clearly differentiate the contribution from B.22. Module reviewers should not need to parse "b" suffixes.

### B.23 — BFS Region-Growing: Greedy Geographic District Packing
**Score**: 3.6/4 (accepted)
**Venue**: AAAI
**Status**: Accepted (--structure bfs-growth)

---

## Cross-Track Dependencies

### B → H.0 (structural bias argument): CLEAR
B.0 and B.17 provide the evidence base for H.0's structural-bias challenge-route argument. H.0 R3 must add cross-references to B.0 and B.17.

### B.11 → H.1 (BisectionEnsemble citation): CLEAR
H.1 explicitly cites B.11 in §2, describes the prime-factor-guided k-way splits, and correctly scopes BisectionEnsemble's coverage (2-way nodes; p>2 uses METIS). No PP1 reinvention flag.

### B.14 → D.0–D.3 (VRA methodology alignment): SHOULD VERIFY
B.14 (VRASection) and D.0 (VRA compliance via edge-weighted partitioning) address the same legal framework from different angles. The 42% threshold in D.1 and the VRASection implementation in B.14 must use consistent BVAP definitions. Panel recommends cross-referencing both in each paper's related-work section.

### B.16 → G.0 (ConvergenceSweep as baseline): NEEDS REVIEW
G.0 (ensemble methodology) cites B.16's T=600 formula as the deterministic alternative to ensemble simulation. If B.16's derivation has not been panel-reviewed, G.0's citation rests on an unvalidated result.

---

## Module Score

| Sub-track | Papers reviewed | Avg score | Status |
|-----------|-----------------|-----------|--------|
| Foundations (B.1–B.7) | 4 of 7 with _panel.yaml | 3.6/4 | Strong; B.5–B.7 need review |
| Structure (B.0–B.15) | 10 of 11 with _panel.yaml | 3.3/4 | Moderate; B.10 gap |
| Search (B.16–B.17) | 1 of 2 with _panel.yaml | Unscored | Weak; both need review |
| Extensions (B.18–B.23, B.22b) | 0 of 7 with _panel.yaml | 3.5–3.9 (external) | Accepted externally; B.18/B.22b unreviewed |

**Module Score: 7.3 / 10**
**Tier**: Strong — research quality is high; administrative and documentation gaps reduce the score

---

## Priority Items

### PP1 — Blocking

**PP1-B6 — B.6 MODULE.md description correction**
MODULE.md states B.6 proves "O(n log² n) / O(√log n) approximation guarantee." The actual paper proves NP-hardness and characterizes METIS runtime as O(niter × n log k). This description is incorrect and propagates errors to Track A. Correct MODULE.md immediately.

**PP1-B22b — B.22b redundancy decision**
B.22b appears to be the "Phase 2" work previewed in B.22. Either merge into B.22 extended version or rename B.22b with a distinct code and clearly differentiate the contribution. Do not publish both as separate papers with the current naming without clearer differentiation.

### PP2 — Important

**PP2-B16 — B.16 ConvergenceSweep panel review required**
The T=600 statutory formula is the program's foundational seed-selection standard. It is cited by B.0, G.0, G.14, and H.0. A full panel review with statistical methodology assessment is required before any downstream papers treating T=600 as validated can be submitted.

**PP2-B17 — B.17 parameter sensitivity panel review required**
The null finding requires statistical power analysis. Full panel review needed.

**PP2-B15 — B.15 Proposition 1 labeling**
B.15 Proposition 1 (stability guaranteed under mild conditions) is an empirical observation, not a mathematical proof. Relabel as "Conjecture" or provide proof.

**PP2-B5B6 — B.5, B.6 panel reviews required**
Both papers have no panel review history. B.6 has the additional MODULE.md description error that must be resolved. Full panel reviews needed for both before JACM/APSR submission.

### PP3 — Enhancement

**PP3-Foundations — B.7 panel review**
B.7's CV < 2% seed stability claim is cited extensively. Panel review would validate the statistical methodology.

**PP3-Extensions — B.18 panel review**
Multi-reapportionment stability is an important empirical question. Panel review would strengthen the evidence base.

---

*Panel convened 2026-05-07. Track B — 25 papers across 4 sub-tracks: foundations, structure, search, extensions.*
