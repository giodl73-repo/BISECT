# Accepted State follow-ups with a large RCTX

Date: 2026-10-24. Status: source correction, actual large-context WASM regression
and staged NC/CA browser journey passed; immutable evidence closure and scoped
review/support registration pending.
Owner: coding agent; BISECT maintainer milestone acceptance. WP-03/WP-07/WP-08.

The shared native `run-accepted-state-workflow` boundary admits a 3 MiB RPLAN and
8 MiB RCTX (`crates/bisect-wasm/src/state_workflow.rs::run_accepted`). The browser
request helper advertised 8 MiB but decoded RCTX with the generic 3 MiB artifact
reader first. The actual accepted CA context is 6,012,060 bytes, so the browser
refused before dispatch despite successful native and direct WASM follow-ups.
The original refusal is retained in
`target/build-application/accepted-context-3mib-refusal.log`.

`web/lab/state-workflow-files.js::acceptedStateWorkflowRequest` now validates the
context's operation-specific 8 MiB bound before strict UTF-8 decoding. It retains
exact decoded context/plan text. The 3 MiB plan and generic artifact readers keep
their existing limits; no Rust engine, accepted partition, map algorithm or CSP
was changed. Open remains passive and Run remains explicit.

`test_accepted_workflow_large_context.mjs` uses the real CA plan/context, preserves
both texts exactly, executes accepted-plan WASM analysis without partitioning,
and rejects an over-8-MiB context, over-3-MiB plan and malformed UTF-8. Evidence:
`target/full-chamber-matrix/context-reader-1792827083427/evidence.json`.

The new local candidate is `dist/wasm-full-chamber-followup-preview`. Unchanged
files are hard-linked from the immutable accepted Build package; the two changed
module copies and offline manifest were detached before writes. Base content is
preserved. Offline identity and dependency checks pass. Actual NC/CA staged
map/report/large archive checks passed, as recorded below. Delivery/support claims
remain with the accepted Build package until this correction's evidence closure
and scoped review/registration finish.
The candidate uses the same exact C-enabled engine and explicitly selected batch
dissolve on complete retained geometry. Geometry simplification, optional missing
evidence, native method-budget failures and wider host/national limits remain
visible in the separate matrix contract.

## Staged and regression checkpoint

The corrected staged package passes the actual NC/CA map/report/archive journey:
`target/full-chamber-matrix/map-report-1792824673295/browser-1792827385108/evidence.json`.
Both maps display, the report remains sandboxed, saved results match an exact
native reference, large archives preserve original sources/results, Open is
inert, and no partition is executed. The ISO-millisecond clock reference is
recomputed natively; no artifact field/hash is normalized. CSP remains unchanged.

The affected legacy workflow and ordinary-project/UI regressions pass in
`target/build-application/large-context-legacy-workflow.log` and
`target/build-application/large-context-project-regression.log`. Offline manifest
and local dependency checks pass for the new candidate. The accepted Build
package remains unchanged. Immutable evidence and scoped review/registration for
this correction still follow; this is not a new public deployment or full-parity
claim.


## Scoped evidence closure — 2026-10-24

The [seven-role implementation review](../../signals/roles/check/wasm-national-browser-capacity-roles-check-2026-10-24.md)
approves this local preview with conditions. One coding agent applied the repository
role perspectives; this is not independent maintainer or release acceptance.
Both registered build profiles and all 29 final-package gate outcomes pass.
The immutable local snapshot is
`target/national-matrix/package-snapshots/wasm-national-queue-preview-v4/snapshot.json`,
SHA-256 `257707f9d3fb4e9533cbd0fd502d6d7b78d68b7eab2db555a4c1fbd9c74648c0`.
Full verification covers 9,586 mappings and 3,201 content identities, including
historical failed attempts and the exact tested default/C packages. Earlier failed
freeze attempts remain retained; the completed snapshot passed full readback.

This post-freeze registration changes documentation and support records only.
The Run, Map and TUI scoped child records are implemented with partial evidence;
parent reconciliation, acceptance-v1, real host/API coverage, fresh acquisition,
other vintages/resolutions/methods and deployment remain open. Tested runtime and
package bytes remain unchanged. The map reference is a shared native renderer
sequence, not native CLI directory discovery; national generation uses actual CLI
Build references for configured jobs, not full Run filesystem equivalence.
