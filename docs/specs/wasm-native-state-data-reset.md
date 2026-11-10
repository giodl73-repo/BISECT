# Reviewed native State data reset

The native-output panel now has a separate **Native State data reset** action
for one freshly prepared output target. Choose a folder and review the exact
`data/` paths, then select **Reset reviewed data only**. Review is read-only.
Open, saved `--reset`/`--force` flags and imported receipts do not select a folder
or authorize removal. Other target files, including the old manifest, RPLAN,
context, maps and notes, remain unchanged. The action never starts a partition,
installs an output package or declares a new completed Run.

This closes a scoped filesystem-effect gap. It does **not** enable `--reset` or
`--force` in engine-only State preparation. The later
[selected-State before-Run connection](wasm-native-state-target-run.md) provides
that separate reviewed workflow for a bounded scope; engine-only flags remain
refused. No flags are silently treated as algorithm options.

## Actual native behavior

The authority is `run_single_state` in
[the desktop State runner](../../crates/bisect-cli/src/runner/mod.rs):

1. Reject an incomplete `manifest.tmp` target.
2. For a labeled target, reject an existing manifest unless `force` is set.
3. If reset is requested, remove only the existing `data/` directory.
4. Create the needed directories, then load adjacency and continue Run.

The native warning currently names the whole plan root, but its removal call
operates on `data_dir`. The browser preview shows the actual data-only scope.
Native `force` permits the labeled collision; it does not itself remove files.
The browser's older **Replace existing contents** installation option is a
different explicit action and retains its existing meaning. A separate force
checkbox applies only to native data-reset review. Incomplete manifests block
reset even when this force selection is checked.

Bulk Run also applies completion filtering before dispatch. Reset does not
bypass that filter: `reprocess` is a separate native choice. This standalone
single-target action does not claim bulk scheduling or flag-order equivalence.

## Browser boundary

[The installer module](../../web/lab/native-state-install.js) reuses the shared
native incomplete/collision guards, observes the live target, binds the review
to its private session-local authority, package, root, force value and target
digest, and rechecks before removal. A changed target, copied review, changed
root/package or stale project requires another review. Only the literal `data`
child of the reviewed target can be removed; a file at that path refuses reset.
An absent directory is a no-op without creating any folders.

After removal, all non-data entries are verified unchanged. A filesystem removal
already in progress cannot be undone by cancellation. Cancellation, permission
loss or a concurrent change after the call begins produces an interrupted,
unverified action receipt rather than a completion claim. Accepted in-memory
outputs/downloads remain available. These checks do not lock out another program
or provide atomic directory snapshots. Current observation bounds remain 1,000
entries, 300-character paths and 128 MiB of target file bytes; larger targets
remain unsupported by this action.

[UI controls](../../web/lab/native-state-output-ui.js) consume reviews once and
retire them when inputs/results change. Open also clears the reset force
selection. The module returns a structured completed/interrupted receipt; the
current UI displays its outcome but does not yet save that receipt in projects
or offer a dedicated receipt export. Full custody/replay is still WP-06 work.

## Evidence and scope

- `test_native_state_data_reset.mjs` compares **32 actual desktop CLI filesystem
  cases**, covering labeled/unlabeled targets, manifest presence, incomplete
  markers, reset and force. The CLI is deliberately given a missing adjacency
  file: all calls exit with failure, proving the observed preflight/reset effect
  ordering rather than successful end-to-end partitions. Original arguments,
  stdout/stderr, exit codes and before/after file bytes are retained per case.
- There are **25 refusals** across native preflights and copied/changed/stale
  authority, cancellation, removal failure and non-directory cases. Exact files
  outside `data/` remain unchanged. Engine-only `--reset` is still refused.
- Native proceeds to recreate an empty `data/` directory before its deliberately
  failing graph load. The separate browser reset action creates nothing. This
  declared directory-lifecycle difference is retained alongside the exact file
  comparisons; it is not normalized into a whole-Run equivalence claim.
- `test_native_state_data_reset_browser.mjs` uses actual workers and genuine
  OPFS handles with an injected picker. It exercises read-only review, stale
  targets, data-only removal, preserved files, labeled force/incomplete ordering,
  cancellation after removal, no-op reset, retained downloads, inert Open with
  saved reset/force flags, cleared force/review state and 390px layout. It performs
  no partition. OS picker/permission UI and other host policies remain untested.
- Existing single recovery, selected-input workspace and optional C browser
  regressions pass. Both standard build scripts include the new native CLI and
  optional browser gates.

The verified local package is `dist/wasm-native-data-reset`, built by:

```powershell
./scripts/web/build_workspace_preview.ps1 -Base dist/wasm-workspace-validated -Output dist/wasm-native-data-reset -BrowserChecks
```

Use a new output name when rebuilding; existing packages are retained. The
extension reuses the base WASM and does not repeat the full parity matrix.
Final packaged reset/workspace/C browser checks, 1,035 dependency references and
629 offline asset hashes/sizes pass. Offline version:
`1cb21f6fd7bd8d54c30fb0cb3cb71737ccf8ce6614bce4d788ea2d25e2ec6113`.

An immutable local source/raw-outcome snapshot is retained under
`target/native-state-data-reset/package-snapshots/wasm-native-data-reset/`.
It is investigative integration evidence, not an acceptance-v1 bundle or source
authenticity proof. Pre-Run flag connection, reset receipt export/project custody,
broader state/browser/capacity coverage, batch reset and full workspace/release
acceptance remain open. Nothing was deployed.

The [seven-role review](../../signals/roles/check/wasm-native-state-data-reset-roles-check-2026-10-23.md)
approves continued scoped implementation, with before-Run connection and the
other broader acceptance conditions still open.
