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
| 2 | T.14-T.17 | Construction-family golden vertical slices | Queued |
| 3 | U.16-U.20 | Exact/search/audit certificate family, including U.20 fixed-point language | Queued |
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
