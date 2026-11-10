---
skill: roles-check
topic: parity-child-evidence-archive
date: 2026-11-03
roles_used: 5
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Child evidence capsule review

One agent applies five installed role lenses; these are not independent reviewers.
Artifact: the local archive tool, trusted lock, 23 tests, guide and three builder
preflights. BENCHMARK covers measurement, COVENANT provenance, LEDGER interchange,
SURVEY operator workflow, and TRENCH failure boundaries. Browser project formats,
native algorithms and legal/statistical conclusions are outside this delta.

## BENCHMARK

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Initial tests did not cover failed directory publication and diagnostic IO. | P2 | Archive tests | Fixed: three added cases verify retained staging, original exception preservation and successful bytes with unavailable receipts. All 23 pass. |
| B2 | ZIP index construction can allocate before ordinary file-count checks. | P3 | ZIP preflight | Preserve bounded footer/central-record parsing before ZipFile; count corruption, size and malformed entry cases are tested. |
| B3 | Historical evidence restoration cannot replace small fixture regeneration. | P3 | Build gates | Keep original native/WASM/browser gates mandatory. The source projection validates restoration, not a fresh compiler or actual clean Git checkout. |

## COVENANT

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | A hash lock needs an independently trusted source. | P3 | Lock | Document exact-byte integrity without a publisher-authentication claim; retain the original source/authenticity exclusions. |
| C2 | The release asset is prepared locally but unpublished. | P3 | Distribution | Publishing the capsule and checking an actual clean Git checkout remain release conditions. No remote URL or download is invented. |
| C3 | Original reference binaries are data in this transport. | P3 | Restore | Zero archive files execute. All 327 restored files independently match the original transfer manifest; raw historical path text stays unchanged. |

## LEDGER

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | The archive needs a versioned namespace and strict record schema. | P3 | Manifest | The v1 schema binds paths, digests and sizes; duplicate JSON keys, case aliases, reserved names and traversal are refused. |
| L2 | Existing caches must not be overwritten during import. | P3 | Publication | Import requires an absent proof namespace and stages verified bytes first. Existing-directory tests preserve unrelated contents. |
| L3 | Presence alone cannot certify evidence validity. | P3 | Ensure | The existing-directory outcome explicitly delegates to mandatory ledger validation. The full-parent gate still refuses all unresolved parent claims. |

## SURVEY

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | Build bootstrap must not surprise users with network or executable activity. | P3 | Preflight | Use an explicit local archive/environment variable; no archive payload executes and no network request is made. |
| S2 | Missing evidence needs a concrete recovery action. | P3 | CLI and guide | The failure names the local archive variable and manual import command; all three builder registrations retain ledger checks afterward. |
| S3 | Transport scope and size must be operationally clear. | P3 | Guide | Record 26,954,165 encoded bytes and 97,038,379 decoded bytes for 327 files. This is developer evidence transport, not a new browser project format. |

## TRENCH

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Writing a failure receipt could mask the primary filesystem exception. | P2 | Exception handler | Fixed: diagnostics are best effort, preserving the original exception and staging; the dedicated negative test passes. |
| T2 | A receipt-write failure after publication could misreport a successful byte restore. | P2 | Success path | Fixed: return verified restoration with receipt_persisted=false and the diagnostic error. Actual published bytes remain verified. |
| T3 | Parent-link checks do not constitute an adversarial OS transaction. | P3 | Filesystem boundary | The actual Windows junction case passes. State the caller-controlled-directory assumption and retain broader host/permission obligations. |

Roles reviewed: 5. Findings: 15. P1: 0. P2: 3, fixed. P3: 12, including
open release obligations. Verdict: APPROVED-WITH-CONDITIONS for reversible local
integration after current source catalog and disposable patch checks. Original
R20 owners must terminate before Main activation. Complete amended builds,
actual clean checkout, publication, authenticity and all original WP-01–WP-10
acceptance remain open; no parent or additional child scope is promoted.

Three amendments: add publication/diagnostic fault tests (B1), preserve the
primary failure when diagnostics fail (T1), and separate successful restored bytes
from receipt persistence (T2). Cross-role consensus: restored historical bytes
provide prerequisites; current source checks and original runtime gates remain
the evidence for present behavior.

Evidence is retained under `target/parity-evidence-archive-20261103/`:
`projection-audit.json` records the initial evidence-free 13,791-file source
projection, all 327 restored hashes, successful 313-record ledger validation,
and the correctly refused full-parent gate. This is explicitly not a clean Git
checkout or fresh compiler claim. Failed/earlier test attempts remain retained.
