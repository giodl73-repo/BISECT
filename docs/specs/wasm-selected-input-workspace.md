# Selected native input workspace

WP-03 now connects three explicit input selections to the existing native folder
discovery, source queue, Run, accepted outputs and reviewed installation/recovery
controls. This is a browser adaptation within selected inputs, not automatic
desktop-wide discovery or completion of the whole workspace work item.

Choose **Inspect native input folder**, then use folder upload, **Select live
input folder**, or the input ZIP field. A ZIP may contain V3/V4 at its root or
one enclosing output folder. Configure the Census year, state filters and export
metadata, then Run inspection. **Queue inspected original binary sources** reads
only referenced binary/companions/templates/display geometry and stages their
original bytes. Queueing does not partition. Run the selected state queue
explicitly; its accepted plan/context, report and map remain available through
the existing result actions. Prepare the selected accepted State's native output,
then download its native-layout ZIP or review a selected output folder before
installing. Interrupted writes use the existing explicit recovery review; no
additional partition is required.

## Selection and authority

[native-workspace-source.js](../../web/lab/native-workspace-source.js) is the
common source adapter. Live directory selections read metadata first and check
current read permission, path/size/modified-time observations before queueing.
Referenced files are checked again when read, and the live directory is observed
again before the staged queue is committed. A changed or revoked selection
requires reselection and inspection; a failed/cancelled handoff preserves the
previous job list. Once queued, retained bytes are the explicit input snapshot;
the browser does not watch the directory for later external changes.

Directory and upload selections are session-local. Copied/imported metadata
cannot recreate their private authority. Selecting another root, changing inputs
or opening a project retires affected reviews/exports. A late picker result from
an older project cannot select inputs for the new project. The actual filesystem
API still governs access; metadata/hash observations do not lock other programs
out of a directory or make sequential writes atomic.

Explicit ZIP import runs in
[a dedicated worker](../../web/lab/native-workspace-archive-worker.js). It takes
a private byte snapshot, supports stored/deflated ZIP32 members, and checks local
and central records, descriptors, CRCs, bounds and member extents. It refuses
encrypted/ZIP64/split/unsupported compression, links/special files, traversal,
reserved/prototype names, case aliases, file/directory collisions and hidden or
overlapping member extents. Import caps compressed and expanded bytes at 64 MiB,
entries at 10,000 and path depth at 32. Queue-source and project/archive/output
limits remain separate downstream contracts; an import limit is not a promise
that every corresponding Run or saved project fits.

The original ZIP is retained in the optional `nativeWorkspaceArchive` source
field. Open validates its bounded source encoding but does not inflate it, select
a folder, query permissions, start a Run or install anything. **Load saved input
ZIP** is an explicit action followed by fresh inspection. Old projects without
that field keep their existing meaning. Extracted members stay in memory and
never become filesystem extraction destinations. Pickle markers remain metadata;
the browser never unpickles them. Saved receipts do not authenticate execution.

## Evidence and exact scope

The tracked synthetic fixture is
[native-workspace-case.mjs](../../scripts/web/fixtures/native-workspace-case.mjs):
one four-unit RI 2020 tract graph, physical attributes, exact max-u64 seed,
standard bisection/metis-core, source template and display polygons. Its small
binary/ZIP/project inputs regenerate without retained pipeline outputs.

- `test_native_workspace_source.mjs`: three source modes produce identical
  accepted assignments and native plan bytes; 26 complete native-host replies;
  exact compared fields from the independent native filesystem graph reader;
  maps, original ZIP/project retention, collision refusal, cancellation and exact
  recovered installation; 18 hostile ZIP refusals. Recovery performs no partition.
- The independent native reader's legacy graph JSON omits `unit_kind`; the
  browser explicitly declares `tract`. The named comparison checks that metadata
  difference and all remaining fields exactly, retaining both original JSON texts
  under `target/native-workspace-source/native-graph-reference.json`. It does not
  claim byte-identical graph JSON or full native CLI filesystem Run orchestration.
- `test_native_workspace_browser.mjs`: actual workbench workers exercise a
  portable saved deflated ZIP through Run/download and a live OPFS directory
  through Run/interrupted installation/recovery. Plan/context/assignment bytes
  match the native-compared fixture. It covers metadata-only folder selection,
  denied read permission, changed sources before and during queueing, retained
  downloadable output after cancellation, read-only recovery review, collision
  refusal without writes, inert Open, a late picker and 390px layout.
- The browser test uses genuine OPFS handles with an injected picker and an
  injected read-permission denial. It does not test a native OS dialog or actual
  OS permission revocation. Other browser/host policies remain matrix work.
- Existing folder discovery passes 144 actual native CLI path-selection cases,
  171 native-host comparisons and seven refusals. Existing single-output recovery
  passes 11 scenarios and 31 native guard replies. These retain their own scope.

Both standard WASM build scripts run the new module and optional browser gates.
The local extension command below reuses a validated base engine, builds its host
references, regenerates the small workspace fixtures, refreshes current browser
modules/proof/offline identities, and tests the final package:

```powershell
./scripts/web/build_workspace_preview.ps1 -Base dist/wasm-cffi-gated -Output dist/wasm-workspace-preview -BrowserChecks
```

Choose a new output directory under `dist/`; existing packages are retained.
It retains the base package and does not rebuild the Rust engine, repeat the
base's entire parity matrix or deploy. Source/engine/package identities and
completed checks are recorded under `target/native-workspace-source/` and
`target/native-workspace-browser/`. Optional C browser regressions use a separate
evidence directory so this check does not overwrite the earlier C gate record.

Verified final local package, 2026-10-23: `dist/wasm-workspace-validated` passes
the source-generated module/native checks, actual packaged workspace and optional
C workbench tests, 1,035 local dependency references and 629 exact offline asset
hashes/sizes. Its 529-file shell retains 50 state/year data packs; those packaged
packs are not evidence of all-state Run capacity. Offline version:
`d872c291f500e493396871680160222045d5e2db3ad23be843e2e1151aaa4200`.
The inherited engine remains
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`.
The [seven-role review](../../signals/roles/check/wasm-selected-input-workspace-roles-check-2026-10-23.md)
approves continued scoped implementation with broader conditions open. No
deployment changed.

## Remaining work

Reset/force lifecycle parity, OS-wide discovery adaptations, broader source
formats/vintages/methods and national capacity, acquisition-queue recovery,
composed Verify/replay custody, cross-browser/permission/storage behavior and
formal acceptance-v1 release bundles remain open. Existing single/batch recovery
is reused rather than reclassified as wholly missing. The State workspace
parent remains partial, and no full desktop-parity claim follows from this slice.
