# Browser report PDF delivery

Date: 2026-10-23. Status: implementation, actual default/C-enabled PDF checks
and final-package browser/regression gates pass. Immutable snapshot and scoped
review registration follow the execution-byte freeze; this archived document
may therefore precede their final references. Whole WP-07/WP-10 and desktop
parity remain open.

Tested package: `dist/wasm-report-pdf-validated-v2`. Execution record:
`target/report-pdf-delivery/build-1792797720145821400/package-evidence.json`,
SHA-256 `c0880fefd7662a6405ed613391c8ee84e3df7a3ed9bdb4464b571b27b8a6e3c7`.
The record binds the final offline manifest, both reused engines, native
executables, current source pins and raw gate logs. Source/default, source/C
and final-package runs each pass five genuine PDF cases, five native-host
comparisons and two actual native CLI missing-PDF-tool cases. Chrome version
155.0.8059.85 is recorded; pypdf 6.11.0 and PyMuPDF 1.28.2 independently inspect
the PDFs. These are synthetic grid fixtures, not successful all-state runs.

The retained example is `output/pdf/synthetic-browser-report-example.pdf`,
SHA-256 `b20eaa1c936f14eab639fd6fbb8ce7752c7be22aa4a99f47bdd6988ba25e868e`.
It deliberately preserves the native population-audit FAIL and missing-analysis
labels. Visual inspection of the final contact sheet, map page and comparison
table found readable content and no clipping in those inspected outputs.

## User journey

Generate an HTML report, choose a generated report and its paper/orientation,
then select **Preview report** and **Print / Save as PDF**. Use the browser's
Save as PDF destination. The named `browser-report-print-v1` profile provides
Letter/A4, portrait/landscape, 12 mm margins, wrapped audit text, restrained print
sizes, repeated table headers and numbered page footers. Small-screen preview
styles do not change original HTML downloads. Browser/driver settings can
override CSS intent; the application does not observe those settings.

Optionally select the PDF you saved and **Download print / PDF record**. The
record includes original native HTML SHA-256, restricted HTML/DOM and print CSS
hashes, settings, loaded WASM identity, reported browser agent, local times,
print-call/events and selected-file hash/size. The selected PDF is never rendered
or executed. Its bounded byte count and PDF header/end markers are checked;
this is not a full PDF parser or an HTML-to-PDF content verification.

Print events do not establish that a PDF was saved. The record always keeps
`pdf_save_observed: false`, and selected files retain
`content_match_verified: false`. Their association is user-declared. This follows
from the browser API boundary: [print](https://developer.mozilla.org/en-US/docs/Web/API/Window/print)
provides the print dialog, and [afterprint](https://developer.mozilla.org/en-US/docs/Web/API/Window/afterprint_event)
can also follow a closed preview. The application therefore records observations
without inferring a successful save. Page numbering uses
[CSS page margin counters](https://developer.chrome.com/blog/print-margins?hl=en).

Print settings, the selected PDF bytes and the print session are not embedded
automatically in project files. Download the separate record and retain the
PDF/original HTML alongside the source project. Save/Open keeps native sources
and results; Open remains inert and requires a fresh Run/preview for printing.

## Native and host boundaries

Native authority is `report_cmd.rs::run_report`/`try_generate_pdf`, the shared
`bisect-report` assembly/HTML template and existing generated report descriptors.
Desktop PDF delivery tries installed wkhtmltopdf then pandoc against an HTML
path. With no available tool, HTML+PDF retains HTML and returns success, while
PDF-only exits 1. PDF-only does not itself produce a fresh HTML input. Browser
delivery explicitly requires a freshly generated HTML report and uses the host
print renderer. It does not execute either native subprocess in WASM or promise
PDF byte identity across renderers, versions, fonts or drivers.

Native LabelReport accepts only HTML/JSON and emits a minimal label/year summary;
its current browser operation emits per-state ten-section commission reports.
Printing those browser outputs is an extension, not proof of native LabelReport
command/file equivalence. The missing exact summary variant is ranked separately.

Current generated HTML families keep their existing selection/source contracts:
commission, label/year reports, supplied-analysis draft comparisons, and accepted
state/TUI/batch workflows that actually contain HTML. JSON-only/unavailable
outputs do not invent a printable report. Native court/expert/Typst scaffolding
and parsed-but-unused fields retain their shared native limitations. Printing
adds no official certification, legal determination, PDF/A/PDF/UA conformance,
signature or source-authenticity claim. Supplied failure/missing evidence remains
visible and is not converted into a passing finding.

The preview retains its script-free CSP, bounded DOM and PNG-only embedded
image contract. Source/result/operation/paper edits retire the private preview
and observations. The live DOM hash is checked before print/receipt export and
saved-file association. A late file read cannot overwrite a newer observation.
Failed image decoding disables printing while preserving native HTML downloads.
Unknown PDF contents are only hashed, never inserted into a frame.

## Validation contract and evidence

Cases are declared before execution in `test_report_pdf_browser.mjs`. Small
fixtures regenerate from tracked grid RPLAN/RCTX inputs, native configuration,
analysis/report kernels and a native PNG map. Shared boundary vertices avoid
synthetic floating-point overlaps. No new partition/build is claimed.

- Five full native-host/WASM replies: map, commission report, label report,
  a draft comparison with a metric table, and
  deliberately invalid PNG input. Two actual native CLI dependency cases run
  under a controlled PATH without external PDF tools.
- Four paper/orientation combinations and a draft comparison with tables execute
  the actual worker/UI. The live
  restricted preview DOM is copied to an owned standalone Chrome page and
  asserted identical before genuine `Page.printToPDF` output. GUI print entry
  and events are instrumented separately: no OS-dialog/save completion claim.
  The test protocol is the [Chrome print-to-PDF API](https://chromedevtools.github.io/devtools-protocol/tot/Page/#method-printToPDF).
- Independent pypdf/PyMuPDF inspection compares all visible native/restricted
  PDF text, page counts and media boxes, checks native PNG decoded pixels,
  numbered bottom-margin footers and absence of active/external PDF actions.
  DOM ordered-list numbers are materialized explicitly; exact known page
  footers are verified in the margin before source-text comparison. Remaining
  comparison permits Unicode compatibility glyphs and whitespace only, with no
  numeric, punctuation, district-label or arbitrary field normalization.
- Every PDF page is rasterized. First/map pages and contact sheets are retained
  for visual review. UI checks cover 390px including the iframe's own width,
  inert Save/Open, no PDF/session serialization, source/DOM/settings changes,
  late reads, malformed selected bytes and invalid-map print refusal.

The first text-extraction experiment correctly failed: DOM text omits browser
ordered-list markers that appear in the PDF. Its raw outcomes are retained;
the expected-text builder now derives those explicit markers from the DOM rather
than dropping digits from comparisons. Production report values were unchanged.

Normal/preview builders register the new checks. The scoped
`build_report_pdf_preview.py` acceptance builder reuses prior verified actual
engines and native references after digest/pinned-source checks, because this
slice edits no Rust. It performs no new Rust build or complete source-to-binary
reproduction claim. It requires the existing declared integration-tier legacy
fixtures, Chrome/Playwright and PDF inspection dependencies; missing prerequisites
fail/not-run rather than pass. New small PDF cases use tracked sources.

Local disk pressure is handled without changing archive bytes: redundant
owned test executables have a verified immutable-blob restore map, and NTFS
compression preserves all prior snapshot blob hashes/logical lengths. Fresh
package capacity is checked before writes. The evidence writer and reader use
`verified-inherited-blobs-v1`: inherited manifests and blob sizes/hashes are
verified, and only new unique bytes are copied. The base snapshot is an explicit
dependency, not a standalone-copy claim. Eight storage-reader tests cover inherited
corruption, manifest mutation, false sizes, traversal, cycles, conflicting sizes
and corruption after a prior cached read. The snapshot writer verifies all
referenced bytes again after freezing. Final registration and review references
are excluded from the freeze to avoid circular hashes.

| Host/workflow | Scope |
|---|---|
| Tested Chrome on Windows, owned actual worker and print renderer | Four settings, genuine PDFs, source/native comparisons and independent inspection. |
| Actual OS print dialog, chosen destination/driver or canceled manual save | Not observed/tested; no successful-save assertion. |
| Other browser/font/OS versions, PDF/A/UA/signing and full original-state capacity | Not established by these synthetic cases. |
| Original HTML/JSON and project sources | Existing native bytes/downloads retained; print record is separate. |

## Scoped evidence closure

The final snapshot is
`target/report-pdf-delivery/package-snapshots/wasm-report-pdf-validated-v2/snapshot.json`,
SHA-256 `d38aa313bef7e14f8d5f0996ce52c75dafc7db3534b48930d36e83a3d0d06000`.
All 1,461 file mappings and 788 unique identities verified, including the
inherited base. The snapshot adds 359 new blobs / 24,406,517 bytes for
403,663,869 logical mapped bytes. Preserve its digest-pinned base dependency.
It freezes the execution-stage version of this document; final closure links
and review/ledger registration follow that freeze and change no runtime bytes.

The [six-role implementation review](../../signals/roles/check/wasm-report-pdf-delivery-roles-check-2026-10-23.md)
approves this local scope with carried conditions. Report and LabelReport print
children remain evidence-partial with `acceptance_v1: false`; the LabelReport
print extension is not a required native PDF effect. Full parity, release and
deployment remain separate.
