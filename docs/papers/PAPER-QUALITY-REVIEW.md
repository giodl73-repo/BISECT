# Paper Quality Review Ledger

**Date opened:** 2026-05-12
**Purpose:** move the paper portfolio from "aligned with the atlas" to
"publication-quality enough to read, audit, and revise paper by paper."

This ledger uses the local Panel process as a quality-improvement tool, not as
real peer review. Any simulated reviewer or role perspective is synthetic
feedback for strengthening drafts. It is not endorsement, acceptance evidence,
or a substitute for submission to real venues.

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
| 3 | U.16-U.20 | Exact/search/audit certificate family, including U.20 fixed-point language | In progress |
| 4 | U.10, G.4, G.7, G.10, G.11 | Sampling and ensemble methods | Queued |
| 5 | T.5-T.13, U.0-U.7, U.11, U.13-U.15 | Older algorithm-family papers not yet lifted to the same explanatory standard | Queued |
| 6 | B/C/D/F/G/A synthesis tracks | Portfolio, validation, legal, legislative, ensemble, and synthesis papers | Queued |
| 7 | E/I/J/K/L/M/N/O/P/Q/R/S tracks | Remaining indexed papers and source-only drafts | Queued |

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
| U.17 | Queued | Queued | Queued |
| U.18 | Queued | Queued | Queued |
| U.19 | Queued | Queued | Queued |
| U.20 | Queued | Queued | Queued |

## Batch 3 Paper-Quality Scorecard

**Scoring date:** 2026-05-12

| Paper | P1 Claim | P2 Algorithm | P3 Evidence | P4 Boundary | P5 Build | P6 Reader | Total |
|---|---:|---:|---:|---:|---:|---:|---:|
| U.16 Branch-and-Cut Redistricting | 3.8 | 3.7 | 3.6 | 3.9 | 4.0 | 3.7 | 22.7 / 24 |

### Batch 3 Score Notes

**U.16 Branch-and-Cut Redistricting — 22.7/24.** The paper now makes the
exact-search boundary concrete: the five-vertex path example shows why a
connectivity cut is emitted, the solve-report table names the proof and fallback
fields, and the audit section cleanly separates model-relative exactness from
RPLAN/RCTX validity. The remaining ceiling is empirical depth: the path8 package
is the right smoke-test artifact, but real-data exact-performance claims still
need solver versions, hardware, transcripts, and archived solve directories.
