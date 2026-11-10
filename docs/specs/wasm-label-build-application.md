# Reviewed label build application — next-slice contract

Status: scoped local delivery; broader host/lifecycle/matrix and formal acceptance remain pending.
Owner: coding agent; milestone acceptance: BISECT maintainer. WP-03/WP-07.
Preserve delivered import/rename/summary application gates.

## Native authority

`crates/bisect-cli/src/build_cmd.rs::run_build`, its CLI dispatcher, configured
runner and `run_registry::Registry` define behavior. Validate label, read/validate
YAML and hash the original config before resolving years or checking completion.
Already-built years without force print a diagnostic and continue; they are not
rerun and are not a command-wide error. Preserve original year order.

For an already-built year with force and without dry-run, native invalidates
analyzed/reported registry stages, then removes that year's analysis and report
directories before preparing/running states. `invalidate_year` deliberately keeps
the old built declaration. Interruption after invalidation must not be described
as successful revalidation of the old build. Force on an unbuilt year does not
invent the already-built branch. Dry-run retains config validation and completion
filtering but does not invalidate, remove, execute or create output directories.

Native selects states through `load_all_states` plus filters, invokes
`run_states_parallel`, promotes artifacts for successful states, writes the year
build index and marks the year built. A completed command/year can include failed
states; preserve each state outcome rather than turning built into algorithm
success. Successful promoted `final_assignments.json` and optional provenance
must match their actual accepted runner outputs. Source or output failure can
leave earlier state/year effects intact. Native does not copy the input YAML
into a new source location merely because a portable browser package contains it.

## Browser work and boundary

Connect fresh selected registry/source snapshots to existing prepared build and
configured/state workflows. Apply live completion/force/dry-run decisions before
engine dispatch where native ordering requires them. Installing a final ZIP after
computation does not prove pre-run destructive preparation. Review exact affected
analysis/report trees and explicit force scope before any deletion. Preserve
unlisted labels/years and state files according to native behavior; never replace
native force with a generic whole-target overwrite policy.

Bind original YAML bytes/name/hash, parsed and resolved arguments, source graph/
unit order, requested year/state policy, backend/seed and accepted artifact
identities. Keep selected-source adaptation separate from automatic host-wide
discovery. Unsupported native source/layout/backend paths remain explicit; do not
run a substitute method. Apply state artifacts, native index and registry in the
proved native order with exact readback and failure receipts. Any config-source
copy is an explicit browser export action, not a claimed native filesystem effect.

Open is inert. Imported force flags, paths, completed outputs, baselines and
receipts confer neither authenticity nor removal/write authority. Fresh root
review and concrete confirmation are required. Continuation re-observes current
bytes, does not repeat accepted removals and retains completed accepted outputs.
Unknown partial/concurrent bytes need an explicit conflict decision. Do not
claim OS locking, an atomic directory transaction or rollback.

## Required acceptance

Predeclare actual CLI/reference cases for bad config before completion filtering,
already-built skip, mixed built/unbuilt years, force/dry-run combinations,
stale stage declarations, state filtering/order, partial state failure, artifact
promotion failure, index/registry write failure and interruption after destructive
preparation. Name any source-path/clock/command-line normalization and retain raw
originals; never normalize accepted assignment, upstream or source hashes.

Use tracked small source fixtures and declare retained data as integration-tier.
Run both actual applicable profiles and real worker/OPFS UI for review, Run,
partial outcomes, source edits, cancel, Save/Open and explicit continuation.
Record OS-folder/permission coverage separately and keep portable exports usable.
Register all builders, preserve prior toolkit/Verify/report/import/rename/summary
checks, freeze exact source/native/runtime/package evidence and finish scoped
roles review/support registration. Broad native orchestration, capacity/matrix,
parent and formal release acceptance remain open until their own evidence passes.

## Source checkpoint and remaining acceptance

The native-host core suite records seven actual CLI comparisons and 22 recovery
boundaries in `target/build-application/native-host-application-v4.log`. Its root
is `target/build-application/run-1792820651569`. This is source-prototype evidence,
not actual WASM/browser/UI or packaged acceptance. The retained earlier native CLI
is the comparison baseline; current CLI formatter extraction still needs a fresh
native build. Root assignments, index and registry comparisons do not establish
all nested native output layouts or native parallel scheduling equivalence.

Source UI and passive project/build-archive baseline integration now have
DOM-controller/native-host checks; actual browser acceptance remains pending. Test cancellation after an authenticated state-completed worker event
but before the final one-job reply: accepted output must remain recoverable even
when the reply is aborted. The core now consumes and validates that live engine progress
boundary; four synthetic-transport cases pass, including dropped-reply recovery.
That transport fixture is not actual worker evidence. Also prove stale-event rejection and fresh-root continuation after inert
Open. Run both actual WASM profiles, affected regressions and final staged package
checks; retain failed attempts and scoped evidence. Native-host recovery passes
alone do not close these conditions or the broader host/matrix/release gates.

Current source checkpoint: `target/build-application/source-checkpoint-1792821542418773000/evidence.json`.
The updated core suite is `target/build-application/run-1792821300459/evidence.json`,
with separate `progress-transport-evidence.json`. DOM-controller evidence is
`target/build-application/ui-1792821451333/evidence.json`. Ordinary project/UI
regressions pass in `target/build-application/project-ui-regression.log`.
These source-tier results leave fresh actual native/default/C/browser/package,
scoped review and full work-item acceptance open.

## Fresh engine and actual browser checkpoint

The fresh native/default/C record is
`target/build-application/engines-1792821652704681900/evidence.json`
(SHA-256 `5a09114f18a63ad5d993cdda4da3c102662e00533b7f6f365b959a6d830a9057`).
Both profiles pass seven actual CLI comparisons and 22 recovery boundaries,
with four separate synthetic-progress cases apiece. Native failure witnesses in
`target/build-application/native-failures-1792821998546/evidence.json` preserve
accepted nested state bytes across promotion, index and registry failures.

Actual worker/OPFS browser records:
- default: `target/build-application/browser-1792822072318/evidence.json`, engine
  `19ab174eaa807da4db877adb7a72c084946b4aed2bdcf46b102767a5a4bcaf92`;
- C-enabled: `target/build-application/browser-1792822303070/evidence.json`, engine
  `6ccdeb1d6b643d81d213001df92bd4d60e678507a16b2dd2e4f2e7a47c3fb3bb`.

Both cover review before partition, source-edit retirement, cancellation after
state completion and before final reply, retained artifacts, inert build-archive
Open, explicit replay/re-observation and no new partition after confirmation for
verified states, force preparation, exact output files, dry-run and portable
fallback. The engine-record validator rejects ten substitutions/mutations in
`target/build-application/engine-validation-1792822302591/evidence.json`.
PP-41 records the cancellation transport pitfall. This is actual source-browser
acceptance, not OS folder/permission, final staged-package or full parity support.

The standard builders and both-profile regression runner now register the new
core/controller/native-failure/browser gates. A fresh registered run is in
progress; inspect `target/build-application/registered-gates-first.log` and its
actual exec handle before claiming a result. The local package builder and
inherited snapshot writer are prepared but unexecuted. Existing accepted summary
package and release/support claims remain unchanged.

## Scoped evidence closure

The local package `dist/wasm-label-build-application-validated` passes both
registered actual profiles and all 28 staged gates. Package record:
`target/build-application/package-1792823061103434000/package-evidence.json`,
SHA-256 `5ca3f531792c616bac44c08006caa2333ceb851b434fcc03bf0e4f12bca3c20e`.
Verified inherited snapshot:
`target/build-application/package-snapshots/wasm-label-build-application-validated/snapshot.json`,
SHA-256 `d91c574a06608c9429e96e32336321c159eef73f1126056ff6ccc415e868a1bb`.
It maps 9,489 files / 1,987 identities / 1,913,336,280 logical bytes, with
1,123 new blobs / 1,129,164,099 new bytes. The accepted summary base and inherited
snapshots remain required. A second full verification after lossless NTFS
compression passed. No source or runtime content was changed by compression.

The [seven-role review](../../signals/roles/check/wasm-label-build-application-roles-check-2026-10-23.md)
records 21 findings and four scoped P2 conditions closed. Current child support
is implemented with partial evidence; parent and formal acceptance-v1 remain open.
Original failed comparisons and native promotion/index/registry failure witnesses
remain retained. PP-40 and PP-41 stay globally mitigated.

Observation limits are 64 MiB / 4,096 files and directories before Run,
128 MiB / 6,144 files and directories for mixed rechecks, and 20,000 entries per
scanned directory. Generated retained artifacts remain bounded to 2,000 files /
64 MiB. Exact known-prefix continuation replays retained states explicitly and
never repeats accepted removals after confirmation; unknown partial bytes refuse.
This is a selected prepared-source/serial adaptation, with portable-path limits,
not native host-wide discovery, OS locking, an atomic transaction or rollback.
Config YAML remains portable source data and is not copied into the selected root.
Per-state failure and completed-year markers remain distinct. OS permissions,
other API/capacity/lifecycle cases and wider source/map/national acceptance remain
open. Final review/support registration follows the execution freeze without
changing tested package bytes. Hosting/deployment is unchanged.
