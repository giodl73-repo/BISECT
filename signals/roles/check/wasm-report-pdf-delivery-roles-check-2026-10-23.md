---
skill: roles-check
topic: wasm-report-pdf-delivery
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH]
p1_count: 0
p2_count: 3
p3_count: 15
verdict: APPROVED-WITH-CONDITIONS
---

# Scoped browser PDF implementation review

Artifact: [PDF delivery contract](../../../docs/specs/wasm-report-pdf-delivery.md),
`web/lab/report-print.js`, workbench integration, PDF/browser tests and evidence
storage reader/writer. Six installed repository role perspectives were applied
by the coding agent, not independent human signatories. Depth: standard.

SURVEY covers usable report delivery; BENCHMARK native/browser comparison;
COVENANT provenance; LEDGER format boundaries; VAULT imported authority and safe
preview; TRENCH freshness, partial failure and storage exhaustion. No new legal,
algorithmic or certification assessment is made.

The inspected execution record is
`target/report-pdf-delivery/build-1792797720145821400/package-evidence.json`,
SHA-256 `c0880fefd7662a6405ed613391c8ee84e3df7a3ed9bdb4464b571b27b8a6e3c7`.
Every recorded stage passes. Source/default, source/C and final-package suites
each produce five actual PDF cases, five native-host comparisons and two actual
native CLI missing-renderer cases. This slice reuses verified engine/native
executables; it makes no new Rust-build reproduction claim. The immutable
execution snapshot and this review are registered separately to avoid circular
hashes. The archived contract can precede those final registration references.

Snapshot verification completed for
`target/report-pdf-delivery/package-snapshots/wasm-report-pdf-validated-v2/snapshot.json`
(SHA-256 `d38aa313bef7e14f8d5f0996ce52c75dafc7db3534b48930d36e83a3d0d06000`):
1,461 mappings / 788 unique identities, with all inherited blobs checked.
359 new blobs require 24,406,517 bytes. The eight storage-reader adversarial
tests pass. Registration changes documentation and ledger metadata only.

## Findings

P1 blocks shipment; P2 must be fixed for the scoped delivery; P3 is a limitation
or improvement carried forward. Closed P2 findings below require the retained
execution evidence and verified snapshot, not this prose alone.

### SURVEY

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | Print preview alone did not establish a delivered PDF. | P2 | Delivery / browser test | Closed for the scoped renderer: actual PDFs are produced from a DOM asserted identical to the restricted preview, independently inspected and retained. OS dialog/save completion remains unobserved. |
| S2 | Letter/A4 and both orientations have explicit controls; original HTML remains downloadable. | P3 | User journey | Retain selection freshness and preserve JSON-only/unavailable outputs without inventing HTML. |
| S3 | Contact sheet, synthetic map page and comparison table are readable in inspected final outputs; audit FAIL and DRAFT labels remain visible. | P3 | Visual QA | This does not establish other fonts/drivers/browsers or publication/legal readiness. |

### BENCHMARK

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Native-host agreement alone cannot prove external native PDF subprocess behavior. | P3 | Native boundary | Two actual CLI missing-tool outcomes are separately retained; successful wkhtmltopdf/pandoc rendering is not claimed. |
| B2 | Exact source text, native PNG decoded pixels, paper/page/footer checks and PDF action inspection test meaningful content preservation. | P3 | Independent inspector | Preserve predeclared normalization: NFKC and whitespace only after explicit list markers and verified footer removal. No value or district-label normalization. |
| B3 | Existing preview/security, label application, TUI/composed Verify, workspace and optional-C packaged regressions pass in the new record. | P3 | Package gates | These scoped results do not close all-state capacity, OS picker behavior or parent acceptance. |

### COVENANT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Receipts bind original HTML, restricted HTML/DOM/CSS, runtime and selected PDF bytes without claiming source authenticity. | P3 | Receipt | Retain separate original-byte and DOM identities; local time and reported agent are observations. |
| C2 | Print events and selected bytes cannot establish a successful user save or HTML-to-PDF match. | P3 | User journey | Correctly retain `pdf_save_observed: false` and `content_match_verified: false`; independent test inspection is a separate claim. |
| C3 | Inherited evidence depends on its base manifest and verified blobs; it is not a standalone archive. | P3 | Evidence storage | Digest-pin the base and child, resolve every reference, preserve original native/engine build records and disclose reuse. |

### LEDGER

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | Native LabelReport's minimal label/year summary differs from browser per-state commission reports. | P3 | Native boundary | Exact HTML/JSON summary remains a required missing child; PDF on browser commission output is a nonrequired extension for that command. |
| L2 | The new receipt is separately versioned and does not change RPLAN/RCTX/native report bytes or project meaning. | P3 | Project boundary | Keep selected PDF/session/settings out of automatic project serialization; Open requires fresh execution/preview. |
| L3 | Host renderer differences make byte-identical cross-renderer PDF an inappropriate parity claim. | P3 | Comparison contract | Retain renderer/version, paper/content/layout contract and original artifacts; PDF/A/UA/signature support remains unestablished. |

### VAULT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Late selected-PDF reads could overwrite a newer report observation. | P2 | Workbench freshness | Closed in tested scope: abort plus job/result/revision/frame binding guards adoption and error paths; a delayed old read cannot replace the newer selected file/status. |
| V2 | Restricted preview remains script-free, CSP-bound and PNG-only; DOM mutation refuses printing/receipt export. | P3 | Preview | Retain bounded parsing and source validation. `allow-same-origin` with script execution denied is part of this specific boundary, not a universal iframe claim. |
| V3 | Selected PDF bytes are bounded and hashed without rendering or executing them. Header/end checks are not full PDF validity checks. | P3 | Local file observer | Preserve the explicit limited checks and user-declared association; no malware resistance or content-authentication claim. |

### TRENCH

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Duplicate execution snapshots previously exhausted local storage. | P2 | Snapshot writer | Closed for this bounded retention operation only: preflight, verified inherited blobs, fresh destination, hash-checked new copies and independent verification. PP-34 remains mitigated globally. |
| T2 | The initial ordered-list extraction, narrow comparison preview and first package collector attempts failed. | P3 | Failure history | Preserve those diagnostics. Expected list markers were fixed without discarding numbers; provenance layout was fixed; collector now names actual TUI output files. Failed attempts are not accepted packages. |
| T3 | Invalid PNG generation can succeed upstream while printing must refuse undecodable content. | P3 | Partial outcome | Tested print refusal preserves original HTML download. No accepted report or partition is erased by an optional print failure. |

## Synthesis and amendments

Roles reviewed: 6. P1 blockers: 0. P2 findings: 3, closed for the scoped delivery
subject to verified immutable registration. P3 carried notes: 15.

Verdict: **APPROVED-WITH-CONDITIONS** for this local scoped implementation.
Broader Report/LabelReport parents remain evidence-partial; `acceptance_v1` is
false. No release, deployment, successful original-state capacity or whole
WP-07/WP-10 approval is granted.

Top finding: a genuine rendered PDF, a UI print event and a user-selected file
are three different evidence claims. SURVEY, BENCHMARK and COVENANT agree that
their observation boundaries must remain explicit. LEDGER and BENCHMARK agree
that native LabelReport summary generation remains unfinished.

Three amendments applied:

1. Bind print records to fresh source/result/runtime/DOM state and bound selected
   PDF observations; never infer save success or content equivalence.
2. Produce and independently inspect actual PDFs for all four settings and a
   table-bearing draft comparison; retain exact native CLI dependency outcomes,
   native-host replies, packaged regressions and visual QA outputs.
3. Retain execution evidence through a fresh capacity-checked, digest-verified
   inherited snapshot; preserve failures and register partial ledger children
   without promoting broader native summary or release claims.
