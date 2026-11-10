# Reviewed batch output recovery

WP-03 now extends **Review interrupted installation** to native-layout output
batches. Prepare the original batch package, explicitly select its local outputs
folder, review the per-target actions, then choose **Resume reviewed installation**.
Open and review perform no installation. Recovery never partitions a graph.

Each target is classified from current selected-folder bytes:

| Action | Required evidence | Writes |
|---|---|---|
| Retained | Every generated package file is present with its expected byte length and hash; native guards accept the observed target. | None inside that target. Portable extra files are preserved. |
| Resumed | A valid browser interruption journal matches the exact original package and namespace. | Only that package's generated files/receipt, using the single-target recovery contract. Unrelated paths refuse recovery. |
| Installed | The target is absent or empty, and ordinary native target guards accept it. | The reviewed package through the ordinary verified installer. |

A batch receipt claiming completion cannot cause a target to be retained. Old
checkpoints are neither read for authorization nor overwritten. A new unique
checkpoint records current actions and outcomes. A private session-local
WeakMap binds the review to its live root, original package, checkpoint path,
and every observed target/action. Imported or copied review objects, changed
roots/packages and a newly occupied checkpoint path are rejected.

All targets are rechecked before writing even the checkpoint. Each retained
target is checked again when reached, and all final target files are checked
before publishing batch completion. A late file mutation changes the batch to
interrupted; the affected final-verification row is failed rather than left as
completed. Summary fields distinguish retained, resumed and newly installed
targets from failed outcomes. Cancellation keeps previous accepted targets and
earlier checkpoints, and leaves interruption markers where publication has not
completed. Resuming again requires a new explicit review.

The UI shows the three action counts and lists generated/existing paths. It
does not describe a retained target as newly written. The original single-target
flow remains available. Existing RPLAN/RCTX, project and interruption-marker
formats are unchanged; the browser batch receipt gains optional recovery/action
metadata. Ordinary installation receipts retain their existing behavior.

Verification on 2026-10-22:

- `node scripts/web/test_native_state_batch_recovery.mjs`: nine scenarios and
  122 native-host/WASM replies. Source-owned fixtures run actual native State
  graph compilation/execution for three synthetic four-unit state/year inputs
  (RI 2020, VT 2010, WA 2000). The test retains both original replies; only the
  inspector's named `target_pointer_width` difference is permitted. Three fixture
  partitions occur; zero additional partitions occur during recovery.
- Scenarios include a mixed retained/resumed/missing batch, all-retained recovery,
  forged old completion receipt, changed completed target, unrelated partial
  files, copied review/switched root, cancelled recovery and retry, late mutation
  invalidating final completion, and checkpoint-path collision.
- `node scripts/web/test_native_state_batch_recovery_browser.mjs`: real OPFS
  handles, cancellation during the second state's body write, visible live
  classification, read-only review, exact resumed/new files and checkpoint,
  unchanged first-target bytes/receipt, inert Open and 390px layout. The fixture
  regenerates from source; retained target data are not a prerequisite.
- Existing single-target recovery, ordinary two-package output, 50-synthetic-state
  batch output and browser output regressions pass. Both build scripts run the
  new module and optional browser checks.

The native reply checks include shared adapter/kernel behavior; they are not an
independent native CLI proof of the entire browser filesystem workflow. Browser
tests substitute actual OPFS handles for the picker result and do not exercise
the operating-system dialog. The synthetic matrix is not evidence of original
all-state national input capacity.

The browser test also accepts a staged package:

```powershell
node scripts/web/test_native_state_batch_recovery_browser.mjs --package dist/wasm-lab-build/toolkit
```

This implementation does not make directory writes atomic, lock out other
programs, authenticate source/history, migrate a changed package or roll back
an earlier overwritten target. Native-only/incomplete foreign markers remain
unsupported. Unified workspace discovery, native reset/force behavior, acquisition
queue recovery, permission/restart coverage and release acceptance bundles remain
open. WP-03 and full desktop parity are not complete.

The current local package is `dist/wasm-lab-build`, offline version
`d00bd941b135d0f989405735551101c68d51fccbcd426644913cad156e85b80e`. Its packaged browser recovery
test passed, with 256 toolkit files/1,028 resolved dependencies and offline
verification of 620 exact hashes/sizes, 520 shell assets and 50 state/year packs.
