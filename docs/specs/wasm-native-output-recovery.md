# Reviewed recovery of an interrupted browser installation

WP-03 now includes an explicit recovery action for one fresh native-layout
output package at a time; [batch recovery](wasm-native-batch-output-recovery.md)
now composes that contract with live verification of completed/missing targets.
This is a browser adaptation; it does not assert that
the native CLI offers a resume command. Its ordinary incomplete-target and
collision guards remain unchanged.

Prepare the original output package, select **Review interrupted installation**,
choose the local outputs folder, and review the target and generated paths.
Then select **Resume reviewed installation**. Review writes nothing. Opening
a project never selects a folder, starts recovery or restores installation
authority. A browser without local directory writing can still download the
native-layout ZIP.

The marker must be the bounded, typed
`bisect-browser-native-install-incomplete-v1` journal written by the browser
installer, matching both the exact package hash and target namespace. Existing
three-field journals retain their original meaning and encoding. Other/native
markers, malformed journals, different packages and unrelated files/directories
are refused. Recovery rewrites only the package's generated files and its
browser installation receipt; it does not recursively clear the target. An
already published manifest is handled while the interruption marker remains.
Every generated file is closed and read back before the final manifest, and
the interruption marker is removed last. Sequential browser writes remain
non-atomic and do not lock out another program.

Recovery review authority lives in a session-local WeakMap, bound to the live
root handle, package and observed folder digest. Copied/imported review records
and switched handles cannot authorize recovery. Installation re-observes the
folder and rechecks its journal before writing; a changed target requires a new
review. Source-generation and cancellation checks also run after directory
lookups and file readback, including the final manifest close. Follow-up errors
retain the accepted plan/output in memory. Interrupted recovery keeps the
marker and requires another explicit review.

The shared native guards receive a virtual observation that omits only the
validated browser journal, with explicit collision permission for this recovery
action. The on-disk journal remains present. This differs deliberately from
ordinary force installation, which still refuses an incomplete target.
Hashes bind bytes; this does not authenticate historical execution or certify
an election.

Verification on 2026-10-22:

- `node scripts/web/test_native_state_recovery.mjs`: 11 scenarios, one exact
  native/WASM prepared reply and 31 exact native guard replies. Covers
  interruption before writing, before a plan-file write and before manifest publication, failed marker
  removal, changed targets, unrelated files, wrong package, invalid marker,
  copied review, switched root and stale generation during final publication.
- `node scripts/web/test_native_state_recovery_browser.mjs`: actual browser OPFS
  handles, paused body cancellation, read-only review, explicit resume, exact
  installed files, no new partition, stale-folder rejection, inert Open and
  390px layout. Its fixture regenerates from source rather than retained
  `target/` data.
- Existing `test_native_state_output.mjs --core-only` and
  `test_native_state_batch_output.mjs` pass: two packages/three native replies
  and 100 native replies/50 synthetic state outputs, respectively. The existing
  output browser test also passes.

The real-browser tests substitute an OPFS handle for the picker result. They
exercise actual file APIs, not the operating-system picker dialog. Native
picker permission revocation across browser restarts remains unverified.
Both build scripts include the new module/browser tests at the appropriate
native-reference/browser tiers.

The refreshed local package is `dist/wasm-lab-build`, offline version
`d00bd941b135d0f989405735551101c68d51fccbcd426644913cad156e85b80e`.
Asset checks resolve 256 toolkit files/1,028 local dependencies, and offline
checks validate 620 exact hashes/sizes, 520 shell assets and 50 state/year packs.
This stages local assets without deployment or a new Rust build. The recovery
browser test can use the packaged copy explicitly:

```powershell
node scripts/web/test_native_state_recovery_browser.mjs --package dist/wasm-lab-build/toolkit
```

Remaining WP-03 work includes the unified selected-source/workspace adapter,
discovery, native reset/force lifecycle, full acquisition/execution queue recovery,
broader permission/storage recovery and release acceptance bundles. Recovery
currently requires the exact original output package; it does not migrate a
changed package or roll back an overwritten earlier target. Existing bounded
file/path/byte limits remain in force. This slice does not close WP-03 or full
desktop parity.
