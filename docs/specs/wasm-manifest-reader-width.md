# Read-only manifest count support

The Doctor count-width fix exposed four other wasm32 readers that still parsed
the native pointer-sized manifest: certificate verification, Verify prechecks,
original-assignment discovery and multi-label report summaries. Wide metadata
either failed before useful work or silently made the summary manifest unavailable.

These readers now use `ReadOnlyPlanManifest`, including the nested manifest that
gates explicit-label assignment discovery. The ILP summary binding also has a
read-only `u64` count type. Its native alias retains `usize`; verification compares
the claim against the actual bounded report inventory without truncation. The
native Verify consumer explicitly selects its original binding type.

Execution, allocation, report selection, path validation, selected-file limits and
verification order retain their existing rules. Replay, improvement and other
execution-coupled manifest consumers require separate review.

`scripts/web/test_manifest_reader_width.mjs` compares these operations with an
explicit native 64-bit tool. It exercises all 13 manifest counts at `u64::MAX`, a
valid certificate, nested assignment-context discovery, summary availability,
actual audited report counts and invalid numeric forms. It preserves native
outputs and normalizes only the invalid parser diagnostic spelling `expected
usize` to `expected u64`. Actual native Verify dry runs additionally exercise
certificate and ILP precheck branches. Full Verify/Report orchestration and real
browser acceptance remain separate.

## Validation checkpoint

Both fresh WASM profiles pass 72 actual native-tool comparisons and 18 actual
native Verify dry runs each. Evidence is retained in
`target/manifest-reader-width/run-Hk4a9t/evidence.json` (default) and
`target/manifest-reader-width/run-HxkKWl/evidence.json` (C-enabled). These corrected
gates derive their certificate fixture from tracked, hash-checked repository
examples, without depending on previously generated target files.
The selected native executables are hashed in each receipt; their existing
binaries are references, not freshly rebuilt by this test.

The wasm32 release compile check, native CLI release compile check and six native
ILP manifest-binding unit tests pass. The complete project-handler regression
passes. Each fresh engine also passes 15 native policy comparisons plus 68 strict
cases and 42 Fetch planner/handoff comparisons; these do not perform acquisition.

The build captures 151 source identities before compilation and verifies them
after both builds. The new engines are owned copies under
`target/manifest-reader-width/build`. Earlier sealed packages and historical
runtime receipts are unchanged. Two verified generated debug-symbol caches were
removed to provide build space; their identities and recovered space are recorded
in `symbol-cache-cleanup.json`. Native executables and evidence were retained.

Each profile also passes all 72 cases through the actual workbench Open/explicit
Run/worker/download/Save path. Evidence is in `browser-829MoN` and
`browser-CNDBmw`. Imported results stay inactive; exact original sources survive
Save. The C run binds the staged engine and source files and verifies that the
package manifest stays unchanged throughout the check. These project-file tests
do not prove an OS directory picker grant.

The staged package `dist/wasm-manifest-reader-width` has version
`ceed611200195840220269a20c01cf0b50b2d3877d8dd67d565a31b25e1477de`:
741 assets, 641 shell files, 50 state/year packs, 316 toolkit files and 1,251 local
dependencies. Offline/toolkit checks and C 52-match/four-failure regression pass.
Both builders register the new native/browser gates; PowerShell and embedded
Python syntax checks pass. Registration is not a new whole-builder execution.

The [six-lens roles review](../../signals/roles/check/wasm-manifest-reader-width-roles-check-2026-10-27.md)
conditionally approves this bounded implementation. PP-72 records the distinct
host-width metadata pitfall as mitigated. Ordinary ledger/practitioner checks
pass; the full-parity gate still refuses acceptance as expected.

Full builder/clean-checkout, OS folder grants, broader native workflow and
host/capacity matrices, reproducible release and deployment remain open. No new
release or deployment is accepted.

## Immutable evidence closure

`target/manifest-reader-width/closure.json` has SHA-256
`7613c4aa029234e32771a80a78cc3c6d268426dee9d19a4d82954ffd216d8b74`.
Independent readback verifies all 2,679 mappings / 1,642 content identities and
the unchanged previous release archive. It retains the final corrected native
and browser runs, exact package/build inputs, role review and original source
preimages. Working source copies are retained as blobs; only owned immutable
WASM artifacts are linked. Those linked artifacts must never be overwritten in
place. Use new build/output directories for later work.

This pointer follows the checkpoint; its retained document copy intentionally
precedes this pointer. The review's bounded evidence-retention condition is now
satisfied. Broader acceptance conditions remain open.

The current retained-asset rerun verifies the same 13 count fields with both
desktop binaries and both actual staged browser sites. All 144 browser runs
and selected-byte native replies match exactly; the current binaries already
agree on parser diagnostic spelling. The 36 desktop Verify dry runs preserve
both certificate success and ILP count refusal before certificate verification.
A separate actual desktop Compare check matched the browser's table and CSV
bytes for all 18 summary cases in each profile. This closes the scoped Verify
child only. Evidence: `target/current-manifest-reader-width-20261106-r2/`.

The commission Report paths now use the same read-only count type for supplied
manifest metadata. The label Report path checks that the declared district
count can be represented as a plan size and equals the selected plan's actual
count before proceeding. A wide count whose low 32 bits match the plan is
refused. The [desktop comparison](../../scripts/web/test_native_report_manifest_width.mjs)
checks audit-only, complete JSON and HTML output, and explicit fallback for
invalid count syntax against actual native CLI commands. Both staged profiles
passed 36 browser deliveries and two refusal checks per profile. The independent
readback checked the exact source files, outputs, and archive after Save and
inert Open. Evidence: `target/report-manifest-width-overlay-20261106-r1/`.
