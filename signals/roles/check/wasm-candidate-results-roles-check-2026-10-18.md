---
skill: roles-check
topic: wasm-candidate-results
date: 2026-10-18
roles_used: [meridian, survey, benchmark, datum, covenant]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Review scope

Code and retained evidence review of WASM State/States candidate presentation, RCTX geometry handoffs, and the distinction between execution and measured candidate constraints. This is a focused review, not certification of the full application or every native command. Role lenses were applied in this session; no independent reviewers were invoked. Installed roles were enumerated from `.roles/`; legal assertions in role descriptions are not treated as established requirements.

Selected roles: Meridian for graph/algorithm constraints; Survey for practitioner interpretation; Benchmark for regression evidence; Datum for negative outcomes and claim scope; Covenant for unchanged provenance. Each role's verify lens was considered; legal admissibility, upstream Census graph reconstruction and comparative algorithm superiority are outside this artifact.

P3 items below are scoped follow-up or maintenance notes, not newly established functional defects. The single P2 presentation defect is resolved by this change, subject to the validation below.

## Meridian

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| 1 | `ok` execution can retain a disconnected, out-of-tolerance CVD candidate; the batch table formerly said Succeeded without these checks. | P2, resolved | `web/lab/count-results.js`; retained tract geographic CVD | Separate completion from tolerance and contiguity; retain native results unchanged. |
| 2 | Multiscale reports per-district `connected` rather than scalar `contiguous`. | P3 | `crates/bisect-wasm/src/multiscale_input.rs:250` | Aggregate only nonempty Boolean arrays; empty or malformed arrays remain unknown. Implemented here. |
| 3 | Neither population acceptance nor contiguity proves an optimal cut. | P3 | Batch state table | Keep explicit optimality boundary in the note; no optimizer or acceptance-policy change. |

## Survey

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| 4 | A practitioner must see candidate checks alongside the execution result. | P3 | State results | Show Yes/No/Not recorded and measured deviation on each state row. Implemented. |
| 5 | Year-level execution totals alone hide the same distinctions in multi-year mode. | P3 | Multi-year results | Include each retained year's state constraint table. Implemented. |
| 6 | Unsupported or failed execution has no candidate measurement. | P3 | Failed state rows | Never infer passing checks from a completed batch or attached stale result. Implemented. |

## Benchmark

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| 7 | Tests relying only on ignored local receipts would not provide a portable regression gate. | P3 | New presentation test | Default gate uses self-contained schema fixtures; retained native receipt mode is explicit. Implemented. |
| 8 | All-true fixtures would miss the observed negative CVD case. | P3 | Retained native mode | Assert the actual CVD No/No and configured Yes/Yes outputs. Implemented. |
| 9 | Missing/empty/malformed values could become false positives through JavaScript truthiness. | P3 | Constraint formatter | Exercise empty arrays, non-Boolean elements, null, NaN, zero deviation and failed rows. Implemented. |

## Datum

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| 10 | Retained CVD deviation is about 59.6%, above its requested tolerance. | P3 | `target/context-geometry-attributes/tract-geographic-cvd-run.json` | Preserve this negative outcome; do not call it a balanced result. Implemented in presentation. |
| 11 | Recent RCTX evidence covers retained RI-derived custom unit fixtures, not all states' original source geometries. | P3 | RCTX acceptance evidence | Keep geographic and synthetic-fixture scope explicit; full national parity remains unfinished. |
| 12 | Native Fetch names include placeholders as well as implemented download families. | P3 | `fetch_plan.rs`, `fetch.rs`, native CLI | Compare implemented behavior before prioritizing adapters; browser school/EIA acquisition remains a follow-up. |

## Covenant

| # | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| 13 | Stored batch `succeeded` counts execution completion; changing its meaning would alter replay/checkpoint contracts. | P3 | `web/lab/state-run-batch.js` | Change visible labels only; keep receipt schemas/counts unchanged. Implemented. |
| 14 | Original RCTX and supplemental attributes are retained and bound to exact graph inputs. | P3 | `context-engine-files.js`, `prepared-state-job.js` | Preserve source bytes and hashes through future UI work; no source-authenticity claim. |
| 15 | Current preview artifacts are local and unpublished. | P3 | Local staging/evidence | Do not imply this UI or artifact is already on Cloudflare; publish status must remain explicit. |

## Synthesis

Roles reviewed: 5. P1 blockers: 0. P2 issues: 1 resolved. P3 notes: 14.
Verdict: APPROVED-WITH-CONDITIONS for the focused UI change, conditional on the presentation and staged asset gates passing. Entire WASM parity goal remains open.
Top finding: execution success must not be presented as plan acceptance.
Cross-role consensus: Meridian, Survey and Datum require visible negative constraint outcomes; Benchmark requires regression evidence and Covenant requires unchanged receipts.

Three amendments: (1) display native tolerance/contiguity separately from execution, including multi-year state rows; (2) add portable negative/unknown/multiscale tests plus explicit retained-native acceptance mode; (3) document the pitfall and retain unpublished artifact and incomplete parity scope.

Validation commands: `node scripts/web/test_state_candidate_results.mjs` and `node scripts/web/test_state_candidate_results.mjs --retained-native`; staged asset gate after preview preparation. Run output is reported in the session; this document does not substitute for it.

Validation completed: both presentation modes PASS; workbench syntax PASS; staged toolkit 225 files / 859 dependencies PASS; source/staged module byte identity verified. Local evidence: `target/state-candidate-results/evidence.json`. No fresh browser interaction test was run for this presentation-only update. The status banner also labels executions explicitly and uses state totals for multi-year runs.
