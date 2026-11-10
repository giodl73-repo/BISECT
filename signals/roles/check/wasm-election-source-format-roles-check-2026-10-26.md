---
skill: roles-check
topic: wasm-election-source-format
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 5
p2_open: 1
p3_count: 13
verdict: APPROVED-WITH-CONDITIONS
---

# Election source adaptation review

One agent applied the six installed lenses to the [contract](../../../docs/specs/wasm-election-source-format.md),
converter, project field, analysis/follow-up handoffs and staged tests. These are
review perspectives, not independent signatories. Approval covers the bounded
local adapter; actual publication acquisition and full source parity remain open.

## BENCHMARK

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| B1 | JS fixtures alone could repeat a parser defect. | P2 locally closed | Independent Python csv reader matches exact output bytes; both engines match native consumer results. |
| B2 | Synthetic source columns do not validate the actual publication. | P2 OPEN | Live metadata returns 403; explicit synthetic scope, no real-provider or all-state claim. |
| B3 | Minimal DOM handler tests do not prove the new controls. | P3 bounded | Real Chromium tests run both exact staged profiles; VM remains a separate security/handler regression. |

## COVENANT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| C1 | Conversion could erase the source or imply authenticity. | P2 locally closed | Exact original project bytes and original/canonical package hashes; source_authenticity_verified=false. |
| C2 | Successful analysis is not election certification. | P3 carried | Fractional reallocations/two-party scope disclosed; no legal or statistical validity finding. |
| C3 | A synthetic plan must not acquire native engine provenance. | P3 bounded | Missing accepted-tolerance refusal preserved in native/WASM; standalone/combined analysis remain available. |

## LEDGER

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| L1 | Format choice could disappear from queued or reopened work. | P2 locally closed | Strict optional record survives project Open and both follow-up recipe lists; all analysis request paths share conversion. |
| L2 | New format must not become an automatic import guess. | P3 bounded | Explicit selector; default canonical; unknown format rejects. |
| L3 | Parquet and raw provider archives are different schemas. | P3 carried | Conversion targets canonical CSV only; registry/ZIP/Parquet work remains explicitly open. |

## CONTOUR

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| G1 | Wrong party/year aliases could silently change measurements. | P2 locally closed | Exact 2020 Biden/Trump header mapping; wrong/duplicate columns reject, no automatic year inference. |
| G2 | Padding IDs cannot establish matching Census boundaries. | P3 carried | No vintage conversion; native plan coverage remains required and actual publication/vintage proof remains open. |
| G3 | Fractional vote text could be rounded or other parties silently included. | P3 bounded | Preserve decimal lexemes without numeric reserialization; explicitly exclude other parties from two-party CSV. |

## VAULT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| V1 | Project settings could weaken Open's input boundary. | P3 bounded | Retain record-only file inputs and strict format enum; invalid format rejects before analysis. |
| V2 | CSV fields could execute markup or forge generated filenames. | P3 bounded | Plain text conversion, numeric ID/vote validation, fixed export filename and DOM textContent; no eval/HTML. |
| V3 | A conversion adapter should not acquire credentials. | P3 bounded | No network or credential controls added; actual provider failure separately recorded. |

## TRENCH

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| T1 | Normalized IDs can collapse distinct raw strings. | P3 bounded | Duplicate detection after padding; no implicit aggregation. |
| T2 | CSV quoting, widths or huge input could produce partial output. | P3 bounded | Complete bounded conversion before request/export; malformed/overflow cases refuse. |
| T3 | Format edits or imported results might leave stale export authority. | P3 bounded | Revision invalidation and actual browser fresh-only result/inert Open checks. |

Six roles /18 findings: zero P1, five P2 (four locally closed, B2 OPEN), 13 P3.
CONTOUR and COVENANT agree that schema compatibility cannot prove publication or
vintage authenticity. BENCHMARK and LEDGER require separate actual acquisition proof.

Amendments applied: (1) explicit versioned format with source-preserving projects;
(2) independent parser/native comparisons, duplicate-normalization guards and
fractional text retention; (3) retain the real 403 and native provenance refusal
instead of inventing successful acquisition or accepted execution history.
PP-50 records the source schema/rounding pitfall. Full parent/release acceptance
and maintainer sign-off remain open.
