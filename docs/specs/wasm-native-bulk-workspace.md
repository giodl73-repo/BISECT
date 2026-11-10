# Reviewed native bulk workspace

After applying a native Run or States policy to original tract source jobs,
select **Run state batch**, then **Choose outputs folder and review bulk
workspace**. The review observes the selected output root without changing it.
It lists native completed/skipped targets, scheduled targets, missing sources,
blocked targets and the exact `data/` entries scheduled for reset. Check the
confirmation and select **Run on this device** to use that review once.

Completed targets are filtered before reset. Native reprocess separately makes
completed targets eligible again. Original native flags, source recipes and
per-target action receipts remain in the result. Installation and downloading
accepted outputs remain separate operations. States does not acquire a reset
effect from the Run flag; its preflight preserves existing data.

## Native authority and execution

[The desktop entry branches](../../crates/bisect-cli/src/main.rs) call
`filter_incomplete` before dispatch. [The shared predicate](../../crates/bisect-core/src/state_catalog.rs)
recognizes canonical state-name targets and historical state-code folders,
JSON/PKL completion markers and incomplete-marker precedence. Reprocess is a
separate input to this predicate. A marker's presence establishes the native
skip decision, not valid contents or successful historical generation.

[The browser adapter](../../web/lab/native-bulk-workspace.js) gathers bounded
live observations from both target names and calls the existing native selection
operation. Scheduled targets use the native incomplete-target guard and the
existing reviewed data-only action. No argument path becomes filesystem
authority; the user-selected outputs root supplies that authority. Inputs are
snapshotted before asynchronous inspection. Source and policy digests, selected
year and loaded engine identity bind the private review/session.

[The batch scheduler](../../web/lab/state-run-batch.js) re-observes the reviewed
targets before scheduling and rechecks each scheduled target before its action.
It then dispatches the original native job without changing its command. A
changed snapshot refuses rather than using stale completion evidence. An
incomplete target becomes a failed row before removal/partition while other
eligible states continue. Known missing source jobs do not authorize removal.
This remains a browser preflight adaptation, not equivalence to every native
missing-input failure's directory-creation order.

The same session covers Run's native 2020/2010/2000 order, retaining each year's
selection and action records separately. Successfully accepted states survive a
later failed state or cancellation. Run reset without a live reviewed workspace
now refuses before scheduling instead of being mistaken for a completed native
filesystem effect. Existing compute-only queues retain their prior path.

## Authority, cancellation and saved records

Open, recorded paths, command flags and imported receipts never choose a folder
or authorize writes. Reviews and active sessions are private objects. Copying
the public review or reusing a consumed review/session refuses. Source/policy
changes retire the UI review and clear its confirmation. The live root and
engine/source/policy identities are checked before effects. These boundaries
protect against imported project data, not privileged same-origin JavaScript.

The UI offers **Download workspace action record**. A source failure after reset
retains the completed target-action receipt in its failed row. Cancellation after
removal begins cannot undo the filesystem operation: the UI retains a completed
or interrupted action record without claiming a partition occurred. Passive
records can survive unchanged-result autosave; they never recreate authority.
No global filesystem lock, atomic snapshot or rollback is promised.

[Saved-workspace continuation](wasm-workspace-continuation.md) now provides a
separate explicit replay, fresh folder review and unfinished-only action.
Legacy compute-only replay/Resume controls remain separate; a stored action
record never authorizes another reset. The new scoped package retains accepted
rows exactly and does not repeat removal when previously touched data is absent.
Broader lifecycle/layout/capacity and formal acceptance remain open. Historical
test descriptions below refer to their original package, not the later extension.

Target observation retains the existing per-target limits: 1,000 entries,
300-character paths and 128 MiB of file bytes. Combined completion observations
also have a 10,000-path yearly bound and the existing completion-source byte
limit. OS permission loss is a visible failure, not a completed action. Real OS
picker/permission UX and broader browser policy combinations remain untested.

## Evidence and package

`test_native_bulk_workspace.mjs` regenerates a four-unit synthetic graph for VT
and WY. It performs six actual native CLI invocations: Run baseline, reprocess,
incomplete-target continuation, historical completion, States without reset and
all three vintages. Five invocations exit successfully; the incomplete-target
case exits with the expected failure while the other state succeeds. Accepted
assignments and outside-data preservation match. Raw arguments, stdout/stderr,
exit codes and an owned native executable/hash are retained. These synthetic
states each allocate one district; this evidence establishes orchestration,
not multi-district search quality or real-state geographic validity.

The module tests also cover read-only review, copied/consumed authority, changed
targets, required review for reset, malformed source after removal, retained
action outcomes without a partition, States preserving data and year-scoped
receipts. Existing Run queue tests still pass 178 whole native/WASM replies,
saved replay and live Resume; the existing States queue suite also passes.

`test_native_bulk_workspace_browser.mjs` uses real workers and genuine OPFS
handles with an injected picker. It covers refusal before a live review,
read-only review, skipped completion before reset, changed-target refusal,
native-matching assignments, explicit action download, inert Open and cleared
authority, disabled saved-workspace replay, input-change retirement and a
390px review panel. It pauses after real removal, cancels, downloads the
interrupted receipt and verifies that no source job started afterward.

Both standard builders register the new checks. The local package builder ran
the bulk tests plus existing standalone reset, selected-State target actions,
input workspace, single/batch output recovery and optional-C browser regressions
against `dist/wasm-bulk-workspace`. It checked 639 exact asset identities, 539
shell assets, 50 state/year packs, 265 toolkit files and 1,063 dependency edges.
Offline version:
`d658ef3a3376e1e38fe729cc890a9bcb95e9979f1f18651f6dd63c9cefea80c5`.
The unchanged WASM is 25,118,842 bytes, SHA-256
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`.

```powershell
./scripts/web/build_workspace_preview.ps1 -Base dist/wasm-state-target-run-validated -Output dist/wasm-bulk-workspace -BrowserChecks
python scripts/web/snapshot_state_target_run.py --package wasm-bulk-workspace
```

The 85-file stable local snapshot is
`target/native-bulk-workspace/package-snapshots/wasm-bulk-workspace/snapshot.json`.
It includes source, native/engine binaries, original projects, raw CLI outcomes,
browser receipts and package identities. Existing snapshots are retained.
This package reuses the previously tested WASM; it does not rebuild it or repeat
the full legacy C matrix. No formal acceptance-v1 or public release is claimed.

Full workspace replay/recovery, automatic post-Run installation, custom manifest
workspace policy, remaining State naming/diagnostics, source acquisition custody,
broader Verify, real-data/capacity/browser matrices and release notices remain
open under the [full plan](wasm-desktop-parity-plan.md).
