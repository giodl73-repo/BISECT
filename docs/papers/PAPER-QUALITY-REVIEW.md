# Paper Quality Review Ledger

**Date opened:** 2026-05-12
**Purpose:** move the paper portfolio from "aligned with the atlas" to
"publication-quality enough to read, audit, and revise paper by paper."

This ledger uses the local Panel process as a quality-improvement tool, not as
real peer review. Any simulated reviewer or role perspective is synthetic
feedback for strengthening drafts. It is not endorsement, acceptance evidence,
or a substitute for submission to real venues.

Portfolio-wide triage scores for algorithm-bearing papers are tracked in
[`ALGORITHM-PAPER-SCORECARD.md`](ALGORITHM-PAPER-SCORECARD.md). Those scores
route work across tracks; this ledger records the slower paper-by-paper
read/patch/rebuild loop.

## Review Standard

Every reviewed paper must pass six paper-quality gates before we mark it done.

| Gate | Question |
|---|---|
| P1. Claim discipline | Do the abstract, introduction, results, discussion, and conclusion make the same claim, with the same scope and caveats? |
| P2. Algorithm clarity | Can a reader identify candidate objects, why they exist, the decision rule, and what artifact is carried forward? |
| P3. Evidence path | Are source code, RPLAN/RCTX packages, transcripts, certificates, benchmarks, or datasets named precisely enough to audit the claim? |
| P4. Legal/statistical boundary | Are legal, statistical, optimality, ensemble, and audit claims separated so one kind of evidence is not oversold as another? |
| P5. Build hygiene | Does the paper rebuild with no undefined citations, undefined references, or BibTeX warnings? Are changed PDFs copied to `docs/papers/`? |
| P6. Reader experience | Does the paper tell a coherent story, with examples/figures/tables where complexity would otherwise stay abstract? |

## Role Passes

Use REDIST roles as focused lenses. A paper does not need a long written
review from every role, but each relevant gate should be checked before edits
are called complete.

| Role | Paper lens |
|---|---|
| MERIDIAN | Algorithm, graph, bisection, METIS, exact/search mechanics |
| BOUNDARY | Equal population, VRA, legal posture, nonjusticiability boundaries |
| WARD | State constitutional and subdivision-preservation claims |
| COVENANT | Audit trail, package provenance, certificate scope, chain of custody |
| CONTOUR | Census/TIGER/data provenance, geographic units, resolution |
| SCALE | Statistical claims, percentiles, confidence, ensemble scope |
| DATUM | Reproducibility, baselines, falsifiability, comparison fairness |
| BENCHMARK | Tests, fixtures, examples, stale assertions, ground truth |
| LEDGER | File formats, RPLAN/RCTX interoperability, external ecosystem claims |
| SURVEY | Practitioner usability, operational feasibility, court/special-master use |

## Review Loop

1. Read the atlas page, paper sources, existing revision plan, and current PDF.
2. Write a short issue list using P1/P2/P3 priority:
   - **P1:** claim, correctness, legal/audit boundary, or reproducibility risk.
   - **P2:** important clarity, evidence, figure/table, or reader-flow gap.
   - **P3:** polish, local prose, citation cleanup, minor organization.
3. Patch only issues that can be fixed from local evidence.
4. Rebuild the paper and verify logs/BibTeX.
5. Copy the PDF to `docs/papers/`.
6. Update this ledger and the local `REVISION-PLAN.md` if the paper has one.
7. Commit and push each coherent batch.

## Batch Plan

| Batch | Papers | Purpose | Status |
|---|---|---|---|
| 1 | T.1-T.4, U.1, U.8, U.9, U.12 | Re-review the atlas-aligned seed/compositor/legacy papers for paper-quality gaps after the alignment pass | Completed 2026-05-12 |
| 2 | T.14-T.17 | Construction-family golden vertical slices | Completed 2026-05-12 |
| 3 | U.16-U.20 | Exact/search/audit certificate family, including U.20 fixed-point language | Completed 2026-05-12 |
| 4 | U.10, G.4, G.7, G.10, G.11 | Sampling and ensemble methods | Completed 2026-05-12 |
| 5 | T.5-T.13, U.0-U.7, U.11, U.13-U.15 | Older algorithm-family papers not yet lifted to the same explanatory standard | In progress: T.5-T.13 complete |
| 6 | B/C/D/F/G/A synthesis tracks | Portfolio, validation, legal, legislative, ensemble, and synthesis papers | Queued |
| 7 | E/I/J/K/L/M/N/O/P/Q/R/S tracks | Remaining indexed papers and source-only drafts | In progress: K.2/K.0/K.7 Reock pass complete |

## Batch 1 Checklist

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| T.1 GeoSection | Reviewed 2026-05-12 | Fixed count and claim-consistency issues | Rebuilt clean; PDF copied |
| T.2 AreaSection | Reviewed 2026-05-12 | Fixed NC/seat-stability claim drift | Rebuilt clean; PDF copied |
| T.3 County-Sticky Weights | Reviewed 2026-05-12 | Fixed alpha grid and split-metric definitions | Rebuilt clean; PDF copied |
| T.4 ApportionRegions | Reviewed 2026-05-12 | Narrowed determinism/constitutional claims | Rebuilt clean; PDF copied |
| U.1 ConvergenceSweep | Reviewed 2026-05-12 | Narrowed optimality/statistical claims; replaced finite-seed theorem with finite-artifact proposition | Rebuilt; no undefined refs/cites; PDF copied |
| U.8 PercentileSweep | Reviewed 2026-05-12 | Narrowed percentile/ReCom/optimality claims; separated four-state evidence from TX/CA extrapolation | Rebuilt; no undefined refs/cites; PDF copied |
| U.9 BisectionEnsemble | Reviewed 2026-05-12 | Scoped node-local ensemble claims; softened large-arity failure and single-run empirical claims; fixed accepted-rank indexing | Rebuilt; no undefined refs/cites; PDF copied |
| U.12 Algorithm-Selection Matrix | Reviewed 2026-05-12 | Expanded matrix with claim-boundary column, worked reading, and audit fixed-point fields | Rebuilt; no undefined refs/cites; PDF copied |

## Batch 2 Checklist

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| T.14 Spectral Partitioning | Reviewed 2026-05-12 | Reframed as staged spectral-style smoothing baseline; added odd-seat split sketch, implementation boundary, evidence ladder, and audit limits | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| T.15 Capacity-Constrained Clustering | Reviewed 2026-05-12 | Aligned status taxonomy with implementation; added worked capacity rejection table, repair/export boundary, and audit fixed-point limits | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| T.16 Hierarchical Regionalization | Reviewed 2026-05-12 | Aligned merge scoring and witness fields with implementation; added worked merge-decision table, merge-log evidence boundaries, and audit limits | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| T.17 Flow-Based Construction | Reviewed 2026-05-12 | Reframed as capacity-gated frontier baseline; aligned summary/status fields with implementation; added worked frontier-decision table and model-relative witness limits | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |

## Batch 2 Paper-Quality Scorecard

**Scoring date:** 2026-05-12

These scores are an internal quality checkpoint using the six paper-quality
gates above. They are not venue acceptance predictions and do not replace a
fresh simulated panel or external review.

| Paper | P1 Claim | P2 Algorithm | P3 Evidence | P4 Boundary | P5 Build | P6 Reader | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| T.14 Spectral Partitioning | 3.7 | 3.5 | 3.5 | 3.8 | 4.0 | 3.6 | 22.1 / 24 |
| T.15 Capacity-Constrained Clustering | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 / 24 |
| T.16 Hierarchical Regionalization | 3.8 | 3.7 | 3.6 | 3.8 | 4.0 | 3.7 | 22.6 / 24 |
| T.17 Flow-Based Construction | 3.8 | 3.6 | 3.6 | 3.9 | 4.0 | 3.6 | 22.5 / 24 |

### Batch 2 Score Notes

**T.14 Spectral Partitioning — 22.1/24.** The paper is now honest about the
implemented spectral-style smoothing baseline rather than a full eigensolver,
and the odd-seat target table makes recursive target propagation inspectable.
The remaining ceiling is visual/mathematical depth: a future version should add
an actual graph/order/sweep figure and a real-data smoke transcript before it
should claim more than deterministic baseline behavior.

**T.15 Capacity-Constrained Clustering — 22.6/24.** The status taxonomy now
matches implementation vocabulary, and the worked capacity rejection table
shows the decision rule instead of merely naming it. The remaining gap is
comparative evidence: path100 and fixture packages support auditability, but
not real-data quality or repair robustness.

**T.16 Hierarchical Regionalization — 22.6/24.** The merge scoring and witness
fields are tied to the crate behavior, and the merge-decision table gives the
paper a clear local mechanism. The next lift is a richer merge-tree visual and
real-data comparison against capacity clustering, flow construction, METIS, and
spectral construction.

**T.17 Flow-Based Construction — 22.5/24.** The paper now sharply separates the
capacity-gated frontier baseline from future exact flow/branch-price solvers.
The model-relative witness language is strong. The remaining ceiling is solver
depth: the paper should not score higher until it has either a fuller flow model
or a recorded comparison showing why this baseline is useful despite being
intentionally modest.

## Batch 3 Checklist

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| U.16 Branch-and-Cut Redistricting | Reviewed 2026-05-12 | Added worked disconnected-incumbent separation example, concrete solve-report fields, manifest lineage boundary, U.13/U.20 positioning, and path8 command evidence | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| U.17 Branch-and-Price Redistricting | Reviewed 2026-05-12 | Added path4 column/master example, concrete branch-price report fields, CLI/test/package evidence, and fixed-point lineage boundary | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| U.18 Large-Neighborhood Search | Reviewed 2026-05-12 | Added six-unit one-move parent/child example, concrete local-search summary fields, CLI/test/package evidence, and no-improvement benchmark boundary | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| U.19 Evolutionary Search Comparison | Reviewed 2026-05-12 | Added selected-frontier row-to-package example, concrete frontier/metadata/lineage fields, U.14/U.15/U.20 boundary, and seed/objective sensitivity caveats | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| U.20 Plan Audit Certificates | Reviewed 2026-05-12 | Added fixed-point endpoint framing, public package-family table, verifier command surfaces, negative fixture signals, and updated evidence ceiling | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |

## Batch 3 Paper-Quality Scorecard

**Scoring date:** 2026-05-12

| Paper | P1 Claim | P2 Algorithm | P3 Evidence | P4 Boundary | P5 Build | P6 Reader | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| U.16 Branch-and-Cut Redistricting | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.7 | 22.7 / 24 |
| U.17 Branch-and-Price Redistricting | 3.8 | 3.8 | 3.6 | 3.9 | 4.0 | 3.7 | 22.8 / 24 |
| U.18 Large-Neighborhood Search | 3.8 | 3.8 | 3.7 | 3.9 | 4.0 | 3.8 | 23.0 / 24 |
| U.19 Evolutionary Search Comparison | 3.8 | 3.7 | 3.7 | 3.9 | 4.0 | 3.7 | 22.8 / 24 |
| U.20 Plan Audit Certificates | 3.9 | 3.7 | 3.8 | 4.0 | 4.0 | 3.7 | 23.1 / 24 |

### Batch 3 Score Notes

**U.16 Branch-and-Cut Redistricting — 22.7/24.** The paper now makes the
exact-search boundary concrete: the five-vertex path example shows why a
connectivity cut is emitted, the solve-report table names the proof and fallback
fields, and the audit section cleanly separates model-relative exactness from
RPLAN/RCTX validity. The remaining ceiling is empirical depth: the path8 package
is the right smoke-test artifact, but real-data exact-performance claims still
need solver versions, hardware, transcripts, and archived solve directories.

**U.17 Branch-and-Price Redistricting — 22.8/24.** The paper now explains the
column-generation lifecycle through the path4 fixture: valid columns are not
automatically a valid exact cover, and the master must select non-overlapping
columns that cover every unit once. The report-field table and CLI evidence
make the implementation contract auditable. The remaining ceiling is still
solver maturity: production branch-and-price claims need larger column pools,
pricing transcripts, solver versions, and archived benchmark outputs.

**U.18 Large-Neighborhood Search — 23.0/24.** The paper now has a concrete
parent-to-child story: the six-unit fixture shows which unit moves, how edge cut
changes, and why validity gates are separate from improvement claims. The
summary table and package evidence make no-op and improvement runs auditable.
The remaining ceiling is empirical: the current packages prove the contract, not
real-data improvement distributions or production LNS/tabu quality.

**U.19 Evolutionary Search Comparison — 22.8/24.** The paper now makes selected
frontier packaging concrete: a frontier row is search evidence, a selected index
is a trade-off record, and the exported RPLAN/RCTX package is the audit object.
The U.14/U.15/U.20 boundaries are explicit. The remaining ceiling is empirical:
frontier robustness still needs seed, objective, and budget sensitivity evidence
before the paper should make broader quality claims.

**U.20 Plan Audit Certificates — 23.1/24.** The paper now reads as the fixed
point for the algorithm roadmap rather than only a schema note. It names the
public construction, exact, local-search, Pareto, and audit package families;
shows the verifier command surfaces; and lists the negative fixture signals
that prevent package tamper, stale context, unit-order mismatch, profile reuse,
and reserved lineage collisions from being treated as valid evidence. The
remaining ceiling is ecosystem breadth: larger real-data packages, more
adversarial variants, solver-version transcripts, and external package
consumers.

## Batch 4 Checklist

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| U.10 bisect-ensemble | Reviewed 2026-05-12 | Renamed old redist-ensemble framing, aligned current crate/CLI boundary, corrected ReCom pseudocode, RNG seed contract, output schema versioning, and diagnostics scope | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| G.4 Ensemble Diagnostics | Reviewed 2026-05-12 | Fixed old binary/crate names, ESS example drift, source-ensemble certification boundary, 99th-percentile ESS caveat, Hamming reference semantics, statutory step-count rule, and Daubert known-error framing | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| G.7 SMC Redistricting | Reviewed 2026-05-12 | Softened calibration claims, refreshed Phase 2 and SmcPercentile status, added weighted-percentile algorithm, diagnostic-output comparison, NDJSON resample audit parity, and external-validation boundary | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| G.10 Merge-Split | Reviewed 2026-05-12 | Reframed exact-uniform claims to the implemented two-tree stochastic correction, aligned pseudocode with pair-reselection, dual seeds, accepted-state percentile selection, named pair-count/determinant boundaries, and updated ReCom/ForestReCom/SMC guidance | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |
| G.11 Multiscale MCMC | Reviewed 2026-05-12 | Aligned claims with current `bisect-cli` multiscale orchestration and `bisect-multiscale` substrate, corrected visited-state percentile selection, step seed schedule, coarse tolerance boundary, Options A/B/C data requirements, and short-run evidence caveat | Rebuilt; no undefined refs/cites/overfull boxes; PDF copied |

## Batch 4 Paper-Quality Scorecard

**Scoring date:** 2026-05-12

| Paper | P1 Claim | P2 Algorithm | P3 Evidence | P4 Boundary | P5 Build | P6 Reader | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| U.10 bisect-ensemble | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.6 | 22.6 / 24 |
| G.4 Ensemble Diagnostics | 3.8 | 3.7 | 3.7 | 3.9 | 4.0 | 3.6 | 22.7 / 24 |
| G.7 SMC Redistricting | 3.8 | 3.8 | 3.7 | 3.8 | 4.0 | 3.7 | 22.8 / 24 |
| G.10 Merge-Split | 3.8 | 3.8 | 3.6 | 3.9 | 4.0 | 3.6 | 22.7 / 24 |
| G.11 Multiscale MCMC | 3.8 | 3.7 | 3.5 | 3.9 | 4.0 | 3.6 | 22.5 / 24 |

### Batch 4 Score Notes

**U.10 bisect-ensemble -- 22.6/24.** The paper now matches the actual
workspace crate and current CLI boundary: `bisect ensemble` is SMC-only today,
while `bisect-ensemble` backs ReCom-family library use and the
`bisection-ensemble` search mode. The ReCom pseudocode now matches the
pair-attempt/tree-resample implementation, the SHA-256 seed contract matches
`SmallRng::seed_from_u64`, and diagnostics are scoped as cut-fraction smoke
evidence unless the G.4 validator is used. The remaining ceiling is Phase 2
benchmark and historical G-track cross-validation packaging.

**G.4 Ensemble Diagnostics — 22.7/24.** The paper now matches the current
BISECT validator surface, keeps the corrected ESS values visible, and separates
multi-chain R-hat certification from partial ESS/Hamming certification for
legacy single-chain source ensembles. The remaining ceiling is empirical
packaging: a future pass should archive the actual six-state chain traces and
diagnostic JSON artifacts rather than relying on paper tables alone.

**G.7 SMC Redistricting -- 22.8/24.** The paper now avoids the old "only SMC"
overclaim, distinguishes asymptotic target consistency from external validation,
formalises the weighted-percentile/SmcPercentile selection rule, and matches the
implemented NDJSON audit stream including resample records. The remaining ceiling
is external replication: archived R `redist` cross-validation traces and TX/CA
Phase 2 run artifacts would turn the current implementation evidence into a more
portable publication package.

**G.10 Merge-Split -- 22.7/24.** The paper now matches the shipped
`MergeSplitChain` and `bisect state --search merge-split` behavior: deterministic
forward/reverse SHA-256 seed streams, pair reselection, reverse zero-cut
rejection, accepted-state collection, edge-cut percentile selection, and
diagnostic `StepRecord` fields are all named. The biggest improvement is the
claim boundary: the paper no longer says the implemented chain exactly samples
uniformly, and instead states the two-tree cut-count correction plus the omitted
pair-count and determinant terms. The remaining ceiling is empirical: the NC/WI
figures are useful smoke evidence, but a publication package still wants archived
multi-seed traces and a direct G.9/G.10/G.7 comparison artifact.

**G.11 Multiscale MCMC -- 22.5/24.** The paper now separates the production
`bisect-cli` multiscale path from the reusable but not-yet-consolidated
`bisect-multiscale` chain API. The algorithm description matches the current
visited-state selection rule, deterministic step seed format, coarse/fine move
schedule, CLI tolerance multiplier, and tract-to-county Option B data path; it
also names the current simple coarse-sync boundary for Options A/C. The
remaining ceiling is empirical and architectural: a standalone `MultiScaleChain`
entrypoint plus archived multi-seed traces would make the evidence more durable
than the current short-run smoke comparison.

## Batch 5 Checklist

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| T.5 ProportionalSection | Reviewed 2026-05-12 | Reframed guarantee language as a top-level proportional target, added a worked block split, aligned the paper with the current top-split-only `run_proportional_section` implementation, corrected electorate/population notation, narrowed Lorenz feasibility to the non-contiguous relaxation, strengthened legal caveats, and clarified that numeric tables need archived trace packages | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; PDF copied |
| T.6 NestSection | Reviewed 2026-05-12 | Reframed the paper as theory plus an implemented validation substrate, corrected the trivial/weak/substantive strict-state counts, fixed the stale 22-state and `g >= 3` threshold claims, added clean and uneven nesting split examples, softened legal guarantees, aligned crate/CLI names with BISECT, and narrowed Mode 3 guarantees to reportable targets | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; PDF copied |
| T.7 VRASection | Reviewed 2026-05-12 | Reframed legal claims as design and evidence boundaries, aligned the demographic input with the current BISECT proxy implementation, removed first-level-to-final-plan compliance guarantees, clarified VAP/CVAP requirements, narrowed Alabama to a first-level precondition case, and fixed bibliography/build issues | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; focused VRASection/demographics tests passed; PDF copied |
| T.8 StabilitySection | Reviewed 2026-05-12 | Reframed StabilitySection as a cross-census audit layer rather than a map constructor, separated ratio/split-share/Jaccard evidence levels, added block-array split visuals, narrowed legal claims to evidentiary use, replaced computed-CSS overclaims with package-completion boundaries, and cleaned stale bibliography/build issues | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; PDF copied |
| T.9 Multi-Reapportionment Stability | Reviewed 2026-05-12 | Corrected ApportionRegions prime behavior to match the implemented large-prime binary fallback, added visible tree sketches and first-split/depth tables, reframed 2030 values as projection scenarios and audit targets, narrowed ReCom/political-redistricting comparisons to scale heuristics pending same-metric packages, and added election-audit reconciliation principles | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; bisect-apportion prime tests passed; PDF copied |
| T.10 Centroidal Voronoi | Reviewed 2026-05-12 | Reframed CVD as proximity packing with graph-distance and geographic metric modes, aligned the seed derivation and approximate-medoid direction with the Rust implementation, added a visible seed/ownership/split grid, corrected Phase 2 language now that geographic CVD and centroid loading exist, narrowed numerical comparisons to benchmark targets pending packages, and surfaced the current half-split limitation for odd-seat nodes | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; focused CVD tests passed; PDF copied |
| T.11 CVD Geographic | Reviewed 2026-05-12 | Recast the archived companion as an implemented geographic-metric note, added map-vs-hop and seed/ownership/centroid visuals, aligned centroid companion naming and projection scope with the Rust loader, removed unsupported NC/FL/WA improvement claims, surfaced the half-population odd-seat limitation, and reframed empirical tables as benchmark targets | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; focused CVD tests passed; PDF copied |
| T.12 BFS Growth | Reviewed 2026-05-12 | Reframed BFS Growth as the implemented recursive two-way splitter rather than a simultaneous k-way constructor, added seed/frontier/split and heap-pop visuals, aligned seed derivation, heap priority, disconnected leftovers, and local rebalance with Rust, removed unsupported NC/WI/TX and warm-start claims, and surfaced validation/ratio-aware gaps | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; focused BFS Growth tests passed; PDF copied |
| T.13 Moving-Knife | Reviewed 2026-05-12 | Reframed Moving-Knife as the implemented recursive directional sweep splitter rather than a global Reock optimizer, added projection-rank, population-prefix, enclosing-circle, and candidate-ledger visuals, aligned centroid-MEC scoring, Polsby-Popper fallback, seed use, and local rebalance with Rust, and replaced unsupported NC/FL/WA claims with an archive-ready evidence plan | Rebuilt; no undefined refs/cites/BibTeX warnings/overfull boxes; focused MKA tests passed; PDF copied |

## Batch 5 Paper-Quality Scorecard

**Scoring date:** 2026-05-12

| Paper | P1 Claim | P2 Algorithm | P3 Evidence | P4 Boundary | P5 Build | P6 Reader | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| T.5 ProportionalSection | 3.8 | 3.7 | 3.5 | 3.9 | 4.0 | 3.6 | 22.5 / 24 |
| T.6 NestSection | 3.7 | 3.6 | 3.3 | 3.9 | 4.0 | 3.7 | 22.2 / 24 |
| T.7 VRASection | 3.7 | 3.6 | 3.4 | 3.9 | 4.0 | 3.4 | 22.0 / 24 |
| T.8 StabilitySection | 3.7 | 3.6 | 3.2 | 3.8 | 4.0 | 3.5 | 21.8 / 24 |
| T.9 Multi-Reapportionment Stability | 3.7 | 3.7 | 3.1 | 3.9 | 4.0 | 3.5 | 21.9 / 24 |
| T.10 Centroidal Voronoi | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.4 | 21.2 / 24 |
| T.11 CVD Geographic | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.5 | 21.3 / 24 |
| T.12 BFS Growth | 3.6 | 3.5 | 3.0 | 3.8 | 3.9 | 3.5 | 21.3 / 24 |
| T.13 Moving-Knife | 3.6 | 3.6 | 3.0 | 3.8 | 3.9 | 3.6 | 21.5 / 24 |

### Batch 5 Score Notes

**T.5 ProportionalSection -- 22.5/24.** The paper now says what the shipped
algorithm actually does: it applies the partisan dual constraint to the
top-level Huntington-Hill split and then recurses with population-only
GeoSection. The formula remains useful, but it is framed as a target identity
and diagnostic rather than a map-wide proportionality guarantee. The new block
example shows the difference between saying "50/50" and showing a six-unit
R-bloc with exactly three Democratic units. The remaining ceiling is evidence:
the historical six-state tables need archived per-seed traces, selected-plan
hashes, and run metadata before they should carry standalone publication
weight.

**T.6 NestSection -- 22.2/24.** The paper now distinguishes the GCD spine
theory from the current BISECT implementation, which validates nesting and
records audit profile rules but does not yet ship a full three-chamber
constructor. The state census now reports 6 trivial one-seat strict states,
3 weak two-region strict states, and 2 substantive exact cases (Oregon and
Alabama), with the broader `g >= 2` tier corrected to 17 states. The new
Washington and New York examples show the split instead of merely naming it:
clean 2:1 nesting versus an uneven 50-house-across-21-senate container pattern.
The remaining ceiling is empirical and visual: actual case-study maps and an
implemented constructor would lift the evidence path beyond schematic figures.

**T.7 VRASection -- 22.0/24.** The paper now separates engineering evidence
from legal conclusions. VRASection is described as a geography-first split
selector using the current BISECT demographic proxy, with formulas that also
apply to true VAP/CVAP when those denominators are available. The Alabama 2:5
example is now a first-level precondition case rather than proof of final
Section 2 compliance, and the paper names the final-plan evidence still needed:
archived partition packages, district-level demographic analysis, VAP/CVAP
denominators, and bloc-voting review. The remaining ceiling is empirical and
legal: stronger scoring requires final-plan packages and jurisdiction-specific
legal analysis, not only first-level split tables.

**T.8 StabilitySection -- 21.8/24.** The paper now reads as an audit/evaluation
layer over GeoSection rather than a new constructor. The core improvement is
the evidence ladder: L1 ratio stability, L2 first-split tract-share stability,
and L3 district-assignment Jaccard. The new block-array examples show why a
stable ratio is weaker than a stable visible split, and the legal discussion
now treats CSS as evidence rather than a burden-shifting rule. The remaining
ceiling is empirical: full three-census CSS needs archived year-by-year
GeoSection packages, crosswalked assignments, district-level Jaccard, and
same-year outcome diagnostics.

**T.9 Multi-Reapportionment Stability -- 21.9/24.** The paper now matches the
implemented ApportionRegions split prescription: composites use
largest-prime-first splits, while large primes use binary floor/ceiling
fallback rather than flat k-way splits. Texas is now explained as a shift from
19 two-seat containers to a 20/21 first split, and the 2030 values are framed
as projection scenarios and audit targets rather than certified future facts.
The new election-audit analogy usefully names the reconciliation ledger that a
reapportionment package should expose. The remaining ceiling is empirical:
Hamming distances, ReCom-step comparisons, political-redistricting ranges, and
geographic disruption claims need archived same-metric packages before the
paper can score higher.

**T.10 Centroidal Voronoi -- 21.2/24.** The paper now explains CVD as
proximity packing rather than a vague geometry alternative, and distinguishes
the implemented graph-distance and geographic modes. The medoid description now
matches the Rust implementation: all district tracts are candidates and the
sampled probes are the distance targets. The new block-grid visual shows seeds,
Voronoi ownership, and the final split instead of merely saying "nearest
seed." The remaining ceiling is implementation/evidence: CVD currently
rebalances node splits toward one half even when the bisection tree records an
odd-seat ratio such as 3:4, and the NC/WI/TX comparisons need archived
RPLAN/RCTX packages before they carry publication weight.

**T.11 CVD Geographic -- 21.3/24.** The paper now reads as a focused companion
to T.10 rather than a stale standalone Phase 2 claim. It shows graph-hop versus
map-distance divergence, shows seed ownership and snapped centroid updates, and
documents the implemented seed prefix, centroid-loader contract, projection
scope, and missing-centroid error behavior. The remaining ceiling is empirical
and architectural: NC/FL/WA comparisons need archived centroid-backed packages,
projection metadata needs to be carried into run manifests, and odd-seat CVD
nodes still need ratio-aware population targets.

**T.12 BFS Growth -- 21.3/24.** The paper now matches the shipped recursive
two-way BFS splitter rather than describing a simultaneous k-seed constructor.
It documents the two-stage deterministic seed derivation, population-weighted
first seed, farthest-BFS second seed, population-keyed heap expansion,
disconnected-leftover safety net, and bounded boundary rebalance. The new
block arrays show why the algorithm tries particular frontier moves instead of
merely naming a 50/50 target. The remaining ceiling is empirical and
architectural: NC/WI/TX comparisons, warm-start claims, and odd-seat
ratio-aware split targets need archived packages before they can carry
publication weight.

**T.13 Moving-Knife -- 21.5/24.** The paper now explains the implemented
directional sweep: project centroids, sort by angle-specific projection rank,
cut the first half-population prefix, score both sides with centroid-MEC Reock,
keep the best worst-half score, and run the bounded local rebalance. It also
corrects the Polsby-Popper path, which currently falls back to Reock, and
removes unsupported statewide superiority and legal-certification claims. The
remaining ceiling is archival and metric-level: per-node angle ledgers,
polygon-boundary Reock, true perimeter-aware Polsby-Popper, and same-metric
NC/FL/WA packages are needed before the paper can claim more than a strong
auditable structure baseline.

## Batch 7 K-Track Checkpoint

| Paper | Review status | Fix status | Build/PDF status |
|---|---|---|---|
| K.0 Compactness Overview | Reviewed 2026-05-13 | Propagated the K.2 Reock implementation boundary: canonical exact-MBC Reock is distinct from the `bisect-analysis` centroid-radius reported metric and the moving-knife point-cloud proxy | Rebuild target for pulse 01 |
| K.2 Reock | Reviewed 2026-05-13 | Reframed the paper around canonical exact-MBC Reock, the production centroid-radius compactness report, and the moving-knife point-cloud proxy; narrowed MKA and Reock-PP claims to single-seed/proxy evidence; added implementation-path table and court disclosure language | Rebuild target for pulse 01 |
| K.7 Composite Court Guide | Reviewed 2026-05-13 | Propagated Reock disclosure into court survey, `bisect label-analyze` output, template expert language, and composite threshold wording | Rebuild target for pulse 01 |

### Batch 7 K-Track Score Notes

**K.2 Reock -- 20.8/24.** The correctness blocker is removed: the paper no
longer says or implies that the production compactness report uses exact Welzl
MBC. It now names the three relevant objects separately: canonical exact-MBC
Reock, `bisect-analysis::reock` as the centroid-plus-maximum-boundary-radius
proxy, and `bisection_runner::welzl_mec` as a moving-knife tract-centroid helper.
The remaining ceiling is evidentiary rather than correctness: exact polygon-MBC
implementation, archived same-metric compactness packages, and per-node MKA
candidate ledgers would be needed before the K.2 paper should move beyond strong.
