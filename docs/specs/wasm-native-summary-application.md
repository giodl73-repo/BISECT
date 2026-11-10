# Reviewed native summary application — next-slice contract

Status: scoped local delivery registered after fresh native/default/C checks,
all 27 final-package gates, verified inherited storage and scoped roles review.
Broader parent, host/API/capacity and formal release acceptance remain open.
Owner: coding agent; milestone acceptance: BISECT maintainer. Work items: WP-03/WP-07.
Dependencies: preserve reviewed import and rename acceptance gates; consume existing
native-summary generation rather than reimplementing rendering.

## Native authority and observable contract

`crates/bisect-cli/src/analyze_label.rs::run_label_report` validates the label,
loads the registry, resolves the explicit year or original analyzed-year order,
requires each year analyzed, checks its analysis directory, hashes its index if
present, renders report files, writes a report index and marks that year reported.
It stops at the first failing year and retains effects from completed years.
Missing analysis index records an empty upstream hash; it is not evidence that
analysis succeeded. Explicit relative output redirects report files and index;
`.bisect` stays at the repository root. Repeated years and shared output folders
may overwrite earlier files in native order. Native analyzed/reported declarations
are not validation or certification claims.

Existing browser `executeNativeSummaryBatch` prepares completed-prefix bytes and
records year outcomes. Its final flattened file map is insufficient by itself to
prove intermediate live filesystem order when paths overlap. Application must
retain or recompute per-year intended transitions and compare those transitions
against actual CLI witnesses, including repeated years and output overlapping an
analysis source. Do not install the final map once and describe it as equivalent
to native per-year registry/index transitions.

## Browser adaptation and recovery

A fresh selected-root review binds original registry bytes, analysis-directory
presence, analysis index bytes/absence, original request, generated timestamp,
format, year order and relative output. Observe exact affected destinations and
list additions, replacements and retained bytes. Refuse unsafe paths, aliases,
wrong file kinds, stale source/target bytes and unauthorized roots before effects.
No absolute native path is an imported write grant. Root output `.` and safe
relative paths must follow their declared native mapping; unsupported mappings
remain explicit rather than silently redirecting them.

Explicit confirmation authorizes only the reviewed transitions. Recheck current
sources and targets before each effect, read back exact bytes and apply the
registry transition after each completed year's files/index. Treat deliberate
output/source overlap according to previously accepted transitions, without
mistaking those intended changes for external tampering. Cancellation, storage
or permission failure retains completed prefixes and a passive receipt. No OS
atomic transaction or advisory-lock parity is claimed. Unknown partial bytes
require an explicit conflict decision; continuation can retain only re-observed
original or intended bytes under a fresh root review. Open, saved paths, baselines
and receipts are inert and convey neither authenticity nor permission.

## Required evidence before scoped acceptance

- Actual CLI files and ordered year/registry outcomes for default destinations,
  root and relative outputs, shared-output overwrite, repeated years, source/output
  overlap, absent index, missing directory, stale registry and a failed later year.
- Exact comparisons except narrowly predeclared clock fields and JSON object
  ordering; retain original replies. Do not normalize upstream hashes or order.
- Both actual engine profiles; interruption at each write/close/readback/registry
  boundary, altered sources/targets, forged/consumed reviews and permission refusal.
- Actual staged worker/OPFS UI: explicit review/confirmation, source edit retirement,
  cancellation, Save/Open inertness, explicit continuation and portable ZIP fallback.
  OS picker/permission coverage is separate and stays not-run until observed.
- Applicable standard/legacy builder registration, existing summary/report/PDF/
  import/rename/Verify/workspace regressions, immutable raw/source/package evidence,
  scoped roles review and ledger/support registration. Capacity limits must be
  explicit. Broad parent/release acceptance remains unresolved.


## Source implementation checkpoint

`native-summary-application.js` recomputes native per-year intent, observes exact
source/target bytes and directory presence, grants private single-use review and
applies ordered report/index/registry transitions. Output-created source
directories are explicit transitions. Continuation accepts only an exact observed
prefix of the recomputed transitions; the largest equivalent prefix may be
retained when repeated writes produce identical bytes. This is a statement about
current bytes, not authenticated prior execution. Baselines retain no handles or
grants. A failed later native year remains a completed-prefix receipt rather
than successful command completion. Before snapshots are capped at 25 MiB; mixed
rechecks and unique transition buffers at 50 MiB. Prepared source/output bounds
remain unchanged. Unknown bytes require a fresh conflict decision.

The UI and project slot `nativeSummaryApplicationBaseline` implement explicit
review/confirmation, passive Save/Open, cancellation, historical receipts and
portable ZIP fallback. Both retained actual engines pass nine measured ordinary
CLI outcomes, nine exact fixed-clock native outcomes, 31 interruption prefixes
and worker/OPFS browser checks. These are source checks, not new package acceptance.

PP-39 records a newly observed comparison gap: repeated self-overlap makes live
clock differences enter downstream provenance hashes. The actual CLI difference
is retained rather than normalized. `run_label_report_at` supplies the browser's
recorded time to the original native filesystem control flow; the ordinary CLI
continues using separate live clock calls for report and index. The fixed-clock
native helper compares all output bytes and upstream hashes exactly; registry
JSON ordering alone is compared structurally. This supplementary native
orchestration reference is distinguished from the real live-clock CLI witness.
Fresh native clock regressions and rebuilt engine evidence remain required.

Retained four-run source checkpoint: `target/native-summary-application/source-checkpoint-1792815727012507900/evidence.json`. Its retained native binaries match all four measured runs. All 42 native report tests pass, including both clock regressions. Fresh build evidence is running separately and does not inherit a passing status from this checkpoint.


## Fresh engine checkpoint

`target/native-summary-application/engines-1792815690836503100/evidence.json`,
SHA-256 `b68a9361f65d00c7a0cc01ab00c2fda644fed5f21ac6fc538e2882b3ec79c4b0`,
records 20 passing stages: fresh native/default/C builds, 42 native report tests,
preserved rename/summary regressions, nine CLI outcomes and nine exact fixed-clock
outcomes plus 31 recovery boundaries per profile, and each actual worker/OPFS UI.
The rebuilt default/C kernels retain their original byte identities; the native
CLI and fixed-clock helper have separately recorded new identities. No kernel
algorithm was substituted. Engine-reuse validation accepts this real record and
rejects ten mutated/escaped cases, including helper substitution, missing clock
tests and missing summary browser evidence. Its record is
`target/label-rename-application/engine-reuse-validation-1792816287132203800/evidence.json`.
The registered dual-build run is now in `target/native-summary-application/dual-build-legacy.log`;
its terminal result, final package, archive and scoped acceptance are pending.


Registered dual-build checkpoint: `target/native-summary-application/dual-build-legacy.log`
completed with exit 0. Both actual profiles record passing summary-application
CLI/reference and worker/OPFS roots alongside the existing toolkit regressions.
Current final-package execution is separate and does not inherit this pass.


## Final-package execution checkpoint

All 27 package gates passed in `dist/wasm-native-summary-application-validated`:
actual staged summary worker/OPFS, preserved import/rename/generation/report/PDF/
transcript/repository/Verify/workspace regressions, loaded C, project checks and
ledger/consumption mutations. The full-parity negative gate still rejects the
unresolved Run parent. Package record:
`target/native-summary-application/package-1792816756117938800/package-evidence.json`,
SHA-256 `cb30f2e276d8ea697cc7caf689177826617c60d37fa75a0b52407d0c435ac4a7`.
Offline version: `f5a30f736c8f36a07d489f303008bfc05a2203272567a8c13fbff35ffac504ba`.

The selected-root adaptation watches at most 301 files and 100 source directories;
source years remain bounded to 100 and each scanned directory to 20,000 entries.
Original target/source observations are limited to 25 MiB; mixed rechecks and
unique transition buffers to 50 MiB. Portable path/type/case checks and existing
prepared-input limits remain enforced. Broader host/capacity acceptance stays
open. Immutable storage and final review/support registration follow this
execution checkpoint without altering tested runtime bytes.


## Scoped evidence closure

Verified snapshot:
`target/native-summary-application/package-snapshots/wasm-native-summary-application-validated/snapshot.json`,
SHA-256 `03328b51072e4ba4bcc9cd06a758b740589b62e92b2795f41d03507280e16477`.
The manifest maps 8,787 files / 1,784 identities / 957,449,073 logical bytes,
with 952 new blobs / 89,307,516 new bytes. Its accepted import base and inherited
snapshots remain required and digest-verified. Both registered profiles and all
27 final-package gates passed. The [seven-role review](../../signals/roles/check/wasm-native-summary-application-roles-check-2026-10-23.md)
records 21 findings and three scoped P2 conditions closed. Raw clock-difference
and failed comparison witnesses remain retained. PP-39 stays globally mitigated;
this scope proves the named recorded-clock comparisons and ordered installation.

Final registration follows the frozen execution checkpoint and changes no tested
runtime bytes. The child is implemented with partial evidence; broad LabelReport,
OS-folder/permission/lock/atomicity, capacity/matrix and formal acceptance-v1 stay
unresolved. Hosting/deployment is unchanged.
