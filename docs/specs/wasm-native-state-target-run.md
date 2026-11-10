# Reviewed State target actions before Run

The single selected-source State workflow can now connect original `--reset`
and `--force` flags to an explicitly reviewed local output target before preparing
sources or running the partition. Run pauses, shows the native target namespace
and original flags, and asks the user to choose the outputs folder. Review is
read-only. The separate confirmation performs the reviewed action and continues
that exact request. Native force alone removes nothing. Reset removes only
`data/`; notes, prior manifests and other target files remain unchanged.

The [standalone data-reset action](wasm-native-state-data-reset.md) retains its
separate scope. Whole-target installation overwrite is a different action.
Engine-only reset/force flags remain refused. The browser derives a separate
engine command only after validating the original command and explicit action
binding; both argument texts remain in the successful Run record.

## Native ordering and backend use

The authority remains `run_single_state` in
[the native runner](../../crates/bisect-cli/src/runner/mod.rs): incomplete target
guard, labeled collision guard (with native force), data-only reset, then source
loading/partition. This browser adaptation selects live filesystem authority
explicitly and retains the selected source bytes rather than reading arbitrary
argument paths. Known unsupported scope/count/backend requests refuse before
asking for a destructive action. It does not claim an identical order for every
native graph, source or runtime failure.

Backend preflight follows the method/count rule in
[the native State compiler](../../crates/bisect-wasm/src/native_state_run.rs).
One district and the moving-knife, spectral, regionalization and flow-construction
methods do not consume a METIS backend. Their declared C/gpmetis values remain
unused; they do not activate C loading or cause new backend errors. A method
requiring unavailable C or the gpmetis stub refuses before target review.
No backend is substituted.

## Authority and records

[Worker orchestration](../../web/lab/native-state-target-run.js) checks recipe
bytes and scope before producing a target intent. The live worker/request/nonce
handshake and [private grant](../../web/lab/native-state-target-authority.js)
bind the reviewed action to original request, derived command and exact selected
source recipe. Execution consumes the grant once. The action re-observes the
target before removing data; changes require another review. Existing bounded
target observation and native incomplete/collision guards still apply.

Open is inert. Imported commands, receipts, path strings and saved bindings do
not select a folder or restore a review/grant. Input changes, cancellation and
worker/request replacement retire pending authority. Autosaving an unchanged
result can preserve a passive action record across bookkeeping revisions; it
does not preserve execution authority. The private JavaScript capability protects
the imported-data boundary, not against privileged same-origin JavaScript.

The [UI](../../web/lab/native-state-before-run-ui.js) offers an explicit action
receipt download. Successful Runs retain the intent, receipt, original arguments
and derived engine arguments in the project. A failure after a completed action
retains a separate action outcome instead of claiming partition success.
Cancellation after an OS removal begins cannot undo it. The UI keeps an
interrupted receipt available when completion cannot be verified and says no
rollback is implied. These observations do not lock the target against other
applications or provide atomic filesystem snapshots. Failure/cancel receipts
are downloadable in-session; automatic cross-session custody of failed work
remains unimplemented.

## Evidence and limits

[Module/CLI tests](../../scripts/web/test_native_state_target_run.mjs) regenerate
the four-unit RI source from tracked bytes. They compare a successful native
State Run with exact WASM assignments and preserved outside-data files. Ten
additional actual CLI runs compare unused C/gpmetis declarations across the four
non-METIS methods and the one-district case. They retain original arguments,
stdout/stderr, assignments and an owned native executable/hash. Tests also check
required-backend refusal before callbacks, mismatched count, stale source bytes,
one-use grant replay, imported bindings, incomplete targets, force-only
preservation and an actual malformed-source failure after completed reset with
retained action receipt and no partition.

[Browser tests](../../scripts/web/test_native_state_target_run_browser.mjs) use
actual workers and genuine OPFS handles with an injected picker. They exercise
inert Open, pause before computation, read-only review, stale nonce/target refusal,
native-matching assignments, preserved files, receipt download, saved original
flags, fresh-review requirements after reopen, cancellation before action and
after real removal, and a 390px layout. OS picker/permission UI is outside this
evidence. No full-browser or all-state capacity claim follows from it.

Both standard builders register the new tests. The local package builder also
runs existing data-reset, selected-workspace, single/batch recovery and optional
C-worker regressions against the staged package. The scoped package is
`dist/wasm-state-target-run-validated`, generated with:

```powershell
./scripts/web/build_workspace_preview.ps1 -Base dist/wasm-native-data-reset -Output dist/wasm-state-target-run-validated -BrowserChecks
```

Its stable local source/raw-outcome snapshot is
`target/native-state-target-run/package-snapshots/wasm-state-target-run-validated/`.
Create that new snapshot after successful checks with
`python scripts/web/snapshot_state_target_run.py`; it refuses stale identities
or replacement of an existing snapshot.
These local packages and snapshots are not published release artifacts or
formal acceptance-v1 bundles. The preview builder reuses the previously tested
WASM and reruns the relevant JavaScript/workspace gates; it does not rebuild the
Rust WASM or repeat the full backend matrix.

Unlabeled custom-adjacency target actions, bulk Run/States completion/reprocess,
automatic post-Run installation, OS-wide discovery, full Verify/custody,
dependency-notice packaging and broader real-data/browser matrices remain open.
Portable compute-only Runs can remove filesystem flags and retain existing
download/manual-install paths. See the [full plan](wasm-desktop-parity-plan.md).
