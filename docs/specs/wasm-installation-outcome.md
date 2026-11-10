# Download an installation outcome

After a reviewed native-layout output installation or recovery attempt, choose
**Download installation outcome**. The browser retains the most recent attempt
in this session and reports completed, cancelled or interrupted. The JSON uses
`bisect-browser-installation-outcome-v1` and contains the package hash, reviewed
target namespace, recovery flag, optional original installer receipt and error.

A batch interruption retains the exact installer checkpoint receipt, including
completed and failed target rows. A completed single installation retains its
returned receipt, including verified-file identities. Some single-target failures
and early refusals produce no final receipt: the wrapper then contains `receipt:
null`, and the UI explicitly says completed writes are unknown. An interrupted
outcome never means no writes occurred or that changes were rolled back.

The wrapper is a browser observation, not native CLI output, an authenticated
execution certificate, source provenance or legal/election certification. Its
nested receipt is copied without normalization. The outer `target_namespace`
is the reviewed display summary (version and target count for a batch); use the
nested receipt for canonical per-state target namespaces. Never treat the outer
summary as an extraction or installation path. Original RPLAN/RCTX/assignments,
installer ordering, guards, on-disk receipts and interruption markers are unchanged.
Downloading triggers no engine request, partition, folder picker or target write.

## Lifetime and authority

The outcome contains no root handle, review object, source package bytes or
restorable permission. Imported JSON cannot authorize installation or recovery.
Explicitly select and review current target bytes for every new recovery action.
**Clear installation outcome**, changing inputs or opening another project retires
this session outcome. A new valid installation attempt replaces it. It is not
included automatically in project Save, autosave or cross-session restoration;
download before changing inputs or closing the page. Existing native-layout folder
checkpoints remain available independently when the installer could write them.

## Local implementation evidence — 2026-10-24

`target/install-outcome/validation-1792857149850507900/evidence.json` passes eight
staged gates: asset closure, offline closure, single recovery and batch recovery
on each of the default and C-enabled shells. Both shells preserve their prior
engine bytes and all accepted base-package bytes. This is UI/OPFS proof with an
injected picker returning a genuine browser OPFS directory; it does not prove
Windows selected-folder permission grants or revocation.

The synthetic fixtures regenerate three state/year targets for the batch case.
Cancellation after the first complete target retains that target, interrupts the
second and leaves the third missing. Downloaded cancellation receipts equal the
actual on-disk checkpoint; downloaded completed receipts equal the recovered batch
checkpoint. Single-target cancellation and changed-folder refusal correctly have
no final receipt. Exact accepted output files, inert Open, no recovery partition,
no writes/requests/pickers during download and 390-pixel layout pass on both shells.

Packages: `dist/wasm-install-outcome-default-preview` and
`dist/wasm-install-outcome-c-sidecar-preview`. The staging record is
`target/install-outcome/stage-1792857100322053800/evidence.json`.
Original sources and the 12 explicitly reconciled active source identities are
retained under `target/install-outcome/source-before-1792856998987793600`.
Historical execution bundles, source pins within those bundles and immutable
packages remain unchanged. This delivery does not repeat or replace the prior
national campaign, close broader parent acceptance, or deploy the preview.


## Scoped evidence closure

The [five-role implementation review](../../signals/roles/check/wasm-installation-outcome-roles-check-2026-10-24.md)
records three locally closed P2 findings and 12 bounded/carried P3 findings. One
coding agent applied the installed lenses; this does not supply independent
maintainer signatories. The fully read-back-verified inherited snapshot is
`target/install-outcome/snapshot/snapshot.json`, SHA-256
`bf33a0a96444acec53ddfe5cf261ff3e2d0a453f45f26ecc1bc910c5b0e748ee`:
11,076 mappings, 3,252 content identities. It retains both exact candidate shells,
all eight staged outcomes, downloaded JSON/checkpoints and original source identities.
This closure and ledger registration are post-freeze documentation; the frozen
contract retains its pre-registration bytes. Existing Run/State/States parent
states stay partial. New child evidence is partial with acceptance-v1 false.
