# Report PDF read ownership — 2026-10-29

The PDF chooser now clears its selection only when the completing read still
owns `reportPdfReadController`. Retiring an old preview already aborts its read;
its asynchronous `finally` must not clear a newer preview's pending file.
The change is confined to cleanup in `web/lab/workbench.js`. PDF bounds, byte
validation, source retention and unverified association semantics are unchanged.

## Reproduction and correction

The downstream standard diagnostic stopped in `test_report_pdf_browser.mjs`
with an intermittent local-file read refusal. Instrumented and unchanged reruns
both passed. The cause of that initial refusal is not established, and this
correction is not claimed to explain it.

A separate deterministic regression delays two actual generated PDF byte reads.
It retires the first preview and starts a read for the next preview. With the
prior handler, finishing the old read first clears the newer chooser: the
assertion expected `commission-letter-native.pdf` and observed no selected file.
The old source bytes and failing log are retained under
`target/report-pdf-debug-20261029/before` and `cleanup-regression-before.log`.

The corrected handler passes both completion orders for all five PDF cases:
old read before the newer read finishes, and old read after it finishes.
The race fixtures are real Chrome-generated PDF bytes supplied as browser File
payloads, so this scheduling test is independent of OS file-token lifetime.
The suite separately retains normal local-path PDF selection checks.

## Validation and limits

`target/report-pdf-debug-20261029/cleanup-regression-after.json` points to
`target/report-pdf-delivery/run-1793297863144`. The current-module suite passes
five native host reply comparisons, two actual CLI PDF dependency cases, five
PDF rendering cases and independent inspection of 28 pages. Existing malformed
PDF refusal, DOM mutation refusal, inert Open, preview retirement, original
HTML retention and 390-pixel viewport checks remain. The project/UI suite and
report-print byte-observation checks also pass.

The initial intermittent refusal, instrumented run and unchanged rerun remain
separate retained attempts. Current source pins were refreshed in allowed
authority/consumer/browser/candidate fields, including the practitioner source
byte length. Historical runtime and implementation evidence was not repinned.
Ordinary ledger/practitioner checks pass; they do not establish full parity.

This records browser read ownership and generated PDF comparisons. It does not
prove an OS print dialog opened or a user saved a PDF, authenticate the supplied
report, or constitute a complete standard/C package pass. The later registered
gates continue under `target/parity-full-build-20261028-r3/remaining-gates-5`.

The [.roles review](../../signals/roles/check/wasm-build-gate-reconciliation-roles-check-2026-10-29.md)
retains complete-build, final-package and immutable evidence closure conditions.

Focused evidence retention is now independently read back in
`target/report-pdf-read-ownership-20261029/closure.json` (SHA-256
`24f45aba8a228ea41e391422b1c222c54e1b82b316fabfd8981d5c0b523012d6`).
Its snapshot has 516 path mappings and 509 unique byte blobs, including original
and corrected cleanup sources, served assets, actual native/module binaries,
generated reports/PDFs, failed/recheck attempts and the prior 151-source scoped
WASM receipt. Independent readback verifies the old/new guard difference and
five PDF cases/28 pages from the retained copy. This is post-run focused
retention, not complete dependency authenticity or whole-package acceptance.
