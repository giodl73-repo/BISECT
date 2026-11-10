# Native format validation through the browser

Date: 2026-10-27. Status: bounded actual CLI/browser contract implemented;
parent and acceptance-v1 remain open.

The Validate RPLAN action uses the actual `bisect validate --file` parser and
strict-warning policy. Its source remains the selected original file, not a
converted or normalized plan. A format pass does not establish population,
contiguity, complete state coverage, topology, provenance or legal compliance.
Native acceptance of empty legacy assignments is preserved. Modern v0.2 input
retains its own source version even though the native CLI's historical summary
still says v0.1 and reads legacy metadata. That console behavior is compared,
not silently corrected in the browser.

Actual UI testing found that the browser removed a UTF-8 BOM before dispatch.
The same file failed native validation but passed the browser's format check,
and its source SHA described normalized bytes. The original witness is
`target/native-format-cli/browser-hHCVts/progress.json`, with the failed assertion
at `target/native-format-cli/before.log`.

The shared helper now exposes `exactNativeJsonText`: strict UTF-8, at most 3 MiB,
with a BOM preserved. The existing ceremony reader is an alias with the same
behavior. Generic text readers keep their existing behavior. Native format
validation decodes once at the final request-assembly step; an earlier operation
branch cannot be overwritten by the generic normalizer. The worker receives the
original text, and its structured invalid-format result and source hash match
the CLI's refusal and exact file bytes. Projects retain the original base64.

The source-owned `test_native_format_cli.mjs` compares nine fixture variants in
strict and non-strict modes against the actual CLI and both existing actual WASM
binaries. It checks exits, warnings, summaries and raw input SHA, including
legacy/modern, missing-assignment warnings, empty assignments, invalid GEOIDs,
malformed JSON, BOM, CRLF and an opaque hostile-looking label. It also checks
malformed UTF-8 and the selected helper's size bound. Binaries are pinned before
and after each run, without making redundant large copies on the small disk.

The real workbench test opens retained results inertly, then explicitly selects
each original disk fixture through the file input and requires a separate Run.
It compares the final worker request and native outcome, downloads the report,
saves original bytes and confirms that selection/Save do not execute. Imported
results cannot enable report download. Eighteen explicit Runs per profile are
required. An isolated browser context refuses external traffic; this is not an
OS directory-picker permission test.

Both profiles and the final C preview builder now register these checks. At the original component checkpoint, the newly modified whole builders had not
been rerun because available disk space was below their 2 GiB minimum. Registration and parser checks are not whole-pipeline
acceptance. Existing native-format handler, project and source-coverage
regressions also pass. The new local package is
`dist/wasm-native-format-cli-preview`, offline version
`d1274b1f2ce237c6a5c9c76b32148a95bc9de370b301a690511755e4033694f2`:
725 exact asset hashes/sizes, 625 shell files, 50 state/year packs and 1,234
resolved local dependencies. Packaged served-source identities are checked.
Unchanged sealed assets share storage with the previous package; changed files
and the new manifest have independent storage. All previous package asset hashes
were rechecked after staging.

Broader input/capacity/host and file-selection-error coverage, native host I/O,
clean-checkout release reproducibility, formal acceptance and full parent support
remain open. Other native JSON consumers still require their own byte-contract
review. No new whole-build, deployment or election-certification claim is made.


## Current evidence

Default actual CLI/WASM: `target/native-format-cli/run-7u0diS/evidence.json`;
default real file-input/workbench:
`target/native-format-cli/browser-igw6on/evidence.json`.
C-enabled actual CLI/WASM:
`target/native-format-cli/run-p3SFqO/evidence.json`; final packaged C
file-input/workbench: `target/native-format-cli/browser-BY5wOW/evidence.json`. Each has 18 cases/Runs.
The complete project-handler regression is `target/native-format-cli/project.log`.
Current source coverage retains 42 native planner/handoff matches at
`target/fetch-source-coverage/fad2ed4fd27b61a4-1793142335870/evidence.json`;
that suite performs no acquisition. Older attempts retain their own identities.

The existing ceremony reader alias also passes nine actual packaged C workbench
Runs at `target/ceremony-source-start/browser-pQ1SUm/evidence.json`, including raw
BOM refusal and original-profile handling. It uses the retained native reference
from the earlier source-start run; that historical reference was not repinned.


The subsequent [complete preview checkpoint](wasm-report-format-full-preview-checkpoint.md)
removes that disk blocker using rebuildable compiler cache only and passes both
registered profiles plus all final-package gates in run
`47641ab8-cb23-4d47-b138-9c9f4759d1dc`. The source/receipt identities and versioned
ZIP are independently checked. This closes the bounded whole-pipeline condition;
original evidence is unchanged and wider host/capacity/clean-checkout/formal
acceptance remains open.
