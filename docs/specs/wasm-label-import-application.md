# Reviewed native label import application — next-slice contract

Status: source implementation, dual-profile source checks and registered
regressions and final-package/evidence checks passed; scoped local delivery
registered below. Broader parent and formal release acceptance remain open.
Owner: coding agent; milestone acceptance: BISECT maintainer. Dependencies:
WP-03/WP-07 and completion of the reviewed rename delivery checkpoint.

Native authority: `crates/bisect-cli/src/import_label.rs::run_label_import_at`,
shared `bisect_report::label_import` and `Registry::mark_built`. Native import
writes each grouped state's `assignments.json`, then the year build index, then
marks the label/year built. It overwrites its named output files; there is no
native import force flag and it does not delete unrelated old state files.
An existing label is not by itself an import conflict. Registry changes must
preserve other labels and native built/analyzed/reported semantics.

The browser must recompute the existing prepared import from exact original
source and shapefile companions, selected format, label/year and recorded time.
Bind the prepared registry to its actual original live bytes or observed absence.
If the preparation assumed no registry but the live root has one, refuse and
prepare again with the actual registry; never replace it with a new singleton.
A selected root and explicit reviewed overwrite confirmation adapt native writes
for browser use. Imported receipts/paths/projects are historical data only.

Observe and list the exact per-state assignment/index/registry additions and
replacements, hash existing bytes, refuse file/directory and case aliases, then
recheck before each effect. Unlisted files survive. Assignment files precede
index, registry is last, and every write is read back. Cancellation/IO failure
retains completed effects and a passive receipt; explicit continuation may retain
only re-observed exact intended bytes. Unknown partial bytes need a new conflict
review, not silent repair. No OS advisory-lock/atomic-write guarantee is implied.

Predeclared actual CLI comparisons: CSV/GeoJSON/RPLAN and supported shapefile
companions; multi-state grouping and exact leading-zero GEOIDs; existing output
overwrite while unrelated states survive; existing label/other registry labels;
repeat built year; malformed/empty input and unsupported format; unknown GEOIDs;
missing sidecars; integer-boundary handling and native failures. Record input
and all companion hashes separately. Normalize only named timestamp fields for
actual CLI clock output, retaining both originals; registry JSON ordering may be
compared structurally. No assignment, population, source or algorithm values may
be normalized. Shared-adapter matches alone cannot prove CLI orchestration.

Browser checks must include inert Open, explicit preparation/review/confirmation,
source/target/registry edits after review, absent versus existing registry,
occupied non-file output paths, cancellation/read-back/permission failures,
registry-last behavior, explicit recovery, hostile/consumed authority and the
portable no-directory-API path. Malformed registry/native write failure may occur
after native artifacts exist; browser preflight can refuse earlier, and must name
that failure boundary difference. Import is not generation, validation, analysis,
certification or source authenticity. Imported external build metadata remains
external; no hidden plan execution or altered district basis is introduced.

Acceptance needs source/native/both-engine/browser checks, applicable builder
registration, final staged worker regressions, immutable evidence and a scoped
roles review. Keep the required child and whole parent unresolved until their
own evidence passes. This document grants no runtime or deployment approval.


## Continuation baseline contract

Fresh review captures a bounded `bisect-label-import-baseline-v1` containing the
current target SHA-256/length or absence for the exact native output roster,
original registry expectation, request/result identities and label/year/index.
It retains no handle or write grant. Ordinary projects may store this small
historical object; Open validates its shape without reading a disk or executing.
Continuation recomputes the native intent, binds that baseline to the original
request/result and re-observes all named targets. Only declared original or exact
intended bytes are accepted. Original registry bytes/absence must match the
original source snapshot. An intended registry with incomplete artifacts refuses
unless the registry was already identical before this import.

The baseline is declared historical data, not authenticated proof of an earlier
folder state. Importing or changing it authorizes nothing. Live selection,
concrete overwrite review and explicit confirmation are always required. A fresh
conflict review can replace current named targets as native import does; it must
be distinguished from continuation and must not silently repair unknown bytes.
Unlisted state files survive. Old target snapshots are bounded to 25 MiB total;
rechecks allow the bounded 41 MiB mix of old/new targets during writes. Prepared
sources/outputs retain their existing 16 MiB bounds; broader capacity stays open.


## Source implementation checkpoint

`web/lab/label-import-application.js` implements fresh single-use review authority,
registry/absence binding, exact named-target observation, explicit overwrite,
native assignment/index/registry order, readback and passive interruption receipts.
The companion UI exposes registry intake, review, confirmation, cancellation and
explicit continuation. Projects retain only validated passive baseline data;
Open performs no selection, request or write.

The retained dual-source record is
`target/label-import-application/dual-source-1792813286716675300/evidence.json`.
Each actual default/C profile measures 14 CLI outcomes and four interrupted
continuations, plus worker/OPFS browser checks. Exact file comparisons allow only
the top year index `created` clock field and registry JSON object ordering.
Native success above u32 is a declared browser refusal boundary, not a matching
failure or supported browser success. Native malformed registry can fail after
writing assignments; browser preparation refuses before effects. Existing
analyzed/reported stage declarations survive and may be stale; import does not
validate or recompute them.

No Rust kernel changed in this slice. The engine record from the reviewed rename
build supplies digest-verified retained default/C artifacts; reuse is not a new
compile claim. Source pins and raw outputs must remain bound to those artifacts.
The first registered attempt is retained in `dual-build-legacy.log`; it refuses
stale project.js source evidence. Current-source reconciliation retains the full
accepted-to-current diff and its assessment under `source-pin-reconciliation`.
These records do not substitute for passing registered/final-package gates.


The complete registered rerun passed both profiles in
`target/label-import-application/dual-build-legacy-v2.log` after inspecting the
accepted-to-current project diff and reconciling its current source pin. Each
profile records import CLI/recovery and worker/OPFS checks in its own build record.
The original failing attempt remains historical evidence. Final-package checks
and verified immutable storage are still required before scoped registration.


## Final-package execution checkpoint

All 26 registered package stages passed in `dist/wasm-label-import-validated`,
including actual staged worker/OPFS import and retained legacy workflows,
loaded-C execution, project/security checks and ledger/consumption mutations.
The full-parity negative gate still refuses unresolved parent evidence.
Package record: `target/label-import-application/package-1792814305039054300/package-evidence.json`;
SHA-256 `7c4f7b888683852e45b831f76976aa933c4c577eb6b3b3b65bb72449617c36c4`.
Offline version: `23485edbc5ec8c76e26d3ac2c6e58e06f7b2ac0dcaa49d6418f71ecf5c3daef3`.
Verified inherited storage, scoped review and final ledger/support registration
follow this execution checkpoint; they must not alter the tested runtime.


## Scoped evidence closure

The verified inherited snapshot is
`target/label-import-application/package-snapshots/wasm-label-import-validated/snapshot.json`,
SHA-256 `13069294fae648534be9a6a1bc3f4996509563ea548c3f2ff63439780bff766c`.
It retains 6,153 mappings / 1,538 identities and 1,128,431,236 logical bytes,
adding 677 blobs / 22,624,609 bytes. Its accepted rename base and inherited
snapshots remain required and digest-verified. All 26 package gates passed.
The [seven-role review](../../signals/roles/check/wasm-label-import-application-roles-check-2026-10-23.md)
records 21 findings, three scoped P2 conditions closed and explicit remaining
host/capacity/locking/native-boundary limits. This final registration follows the
frozen execution checkpoint without altering tested runtime bytes. The scoped
child is implemented with partial evidence; its broad parent and acceptance-v1
remain unresolved. Hosting/deployment is unchanged.
