# Reviewed label rename application

Status: scoped local package, verified inherited archive and implementation review delivered. Whole-parent, OS/API/capacity and formal release acceptance remain open.

Native authority is `crates/bisect-cli/src/label_cmd.rs::run_mv`, shared
`bisect-report::label_rename`, and `Registry::rename_label`. Native order is
runs, analysis, reports; where a source stage exists, force removes that
stage's existing destination, then the directory moves and its top index is
patched. The registry is last. Stages without a source preserve destination
bytes. Year indexes, config references and cross-index hashes stay unchanged.
The native operation is not an atomic transaction across all three stages.

The browser must bind a fresh shared-kernel recomputation to the complete live
source/destination stage trees and original registry. It copies and verifies
one stage before removing its source, then patches its top index. It reports
this adaptation rather than claiming OS-atomic rename or advisory locking.
A prepared package/imported review grants no write authority. Root selection,
concrete review and confirmation are separate explicit actions; Open is inert.
Read-only intake includes empty directories. Unrelated trees are untouched.

Fresh application requires the original live bytes. Explicit continuation
recomputes from original retained inputs, re-observes all relevant live bytes,
and accepts only original states or exact intended intermediate/final bytes.
Unknown partial bytes refuse; no saved receipt is trusted as authority. Every
review is private, root-bound, single-use and retired on edits. Registry write
requires all stages accepted. Cancellation and IO failure retain completed
artifacts plus an attempted-effect receipt, without implying rollback.

Predeclared comparisons: successful registry-only move; all three stages with
binary/empty directories; force replacement only where source exists; different,
missing and non-object top-index labels; preserved year indexes/configs; missing
source label; registry/directory conflicts with and without force; malformed
index. Successful file bytes and directory roster compare against an actual CLI,
registry JSON compares structurally because native HashMap order is unspecified.
Original registry bytes must remain exact until registry writing begins.

Predeclared browser-adaptation tests: changed source/destination/registry or
extra files after review; case aliases and file-directory mismatches; forged,
consumed or imported authority; cancellation before effects and after each
remove/copy/patch/registry boundary; interrupted copy and read-back corruption;
permission loss; explicit continuation and unknown-byte refusal. Preserve
original source until destination copy read-back succeeds. Native malformed
index failure may occur after a move; browser shared-kernel preparation refuses
before writes. Record that different failure boundary rather than claiming
native partial-effect equivalence. Absolute paths, locks, concurrent OS writers,
other host/API/capacity and full parent/release acceptance remain open.

Final acceptance still requires both actual engine variants, actual browser
OPFS/UI with inert Save/Open and portable/API-absence outcomes, applicable
builder registration, final-package regressions, immutable evidence and scoped
roles review. Source tests alone do not deliver the slice.

## Historical source evidence checkpoint

The source-test record is
`target/label-rename-application/dual-source-1792806190177877600/revalidation-1792806333429716000/evidence.json`.
Both retained actual default/C binaries pass eleven CLI cases (seven successful
exact filesystem comparisons and four matching failures), forty-one interruption
boundaries each, corrupt-copy/source-preservation, permission loss and removal
failure. Actual Chrome worker/OPFS tests cover readonly intake with empty
directories, confirmation, edit retirement, interrupted copy and explicit
continuation, exact native files, inert Open, API absence and 390px UI.
These are existing engine binaries, not fresh build evidence.

The initial Windows case-alias fixture failure and later browser executable-copy
ENOSPC failure remain retained. The latter was rerun sequentially after owned
terminal executable copies were transparently compressed with unchanged hashes;
its successful unchanged default filesystem result was reused explicitly.

Regression record
`target/label-rename-application/regression-1792806233438161000/evidence.json`
records passing project, label repository, native summary, count transcript and
project-handoff checks against current source UI. Ordinary ledger/source and
8+4 mutation checks pass. All three modified PowerShell builders parse. Their
new registrations were pending at that checkpoint. The later engine/package and archive records close current scoped execution requirements; public deployment and full parity remain unaccepted.


The first complete registered default/C run passed in
`target/label-rename-application/dual-build-legacy-v2.log`, after the original
Fetch source-map failure was reconciled against the exact rename-only UI diff.
The prior coverage map, failed run and subsequent 42 native planner/handoff
comparisons remain retained. Original pre-history-fix source and build records
are frozen under `target/label-rename-application/before-receipt-history-fix/`.

Visual QA then found an older in-session completion receipt still displayed
after a project was opened. It granted no authority, but was ambiguous. The new
Chrome assertion fails before the presentation fix and passes afterward; Open
still makes no request, picker selection or write. The controller now labels
that receipt historical for the changed preparation. See PP-37 and the before/
after logs. The new complete registered run is
`target/label-rename-application/dual-build-legacy-v3.log`; it failed from ENOSPC during filesystem tests. After verified inactive-symbol compression and an explicit two-GiB preflight, `dual-build-legacy-v4.log` passed both profiles with the history assertion. These records predate the later native safety change below and do not accept it.


## Shared native safety correction

An owned Windows fixture confirmed that the previous native `mv a A --force`
removed its source directory before failing to rename the now-missing path.
The browser already refused case-only labels. This is a shared native safety
improvement, not a new browser restriction presented as old native equivalence.
`label_rename::distinct_names` now rejects ASCII case-only labels before native
registry/filesystem access. Portable native behavior intentionally refuses them
on case-sensitive hosts too; use a distinct temporary label when changing case.
Exact same-name refusal remains unchanged.

Original witness:
`target/label-rename-application/native-case-alias-witness-1792808289013254600/witness.json`.
The original source and executable are frozen under
`target/label-rename-application/before-native-case-alias-fix/`.
A shared regression fails before the fix; five shared rename tests and 31 serial
native label tests pass afterward. Four actual rebuilt CLI cases preserve every
source stage, exact registry bytes and the original file roster, including no
new lock file; both force settings and case directions are covered. Their record
is `target/label-rename-application/native-case-alias-cli-1792808908694554800/evidence.json`.
The first CLI fixture attempt had case-colliding test-directory names; its failure
and partial results are retained, and unique indexed fixture roots correct the
namespace without changing preservation assertions.

Fresh default/C engines and native host were rebuilt through
`scripts/web/build_label_rename_engines.py`. A regression runner may use
`-EngineRecord` only after validating current source, exact frozen native/WASM
bytes, build/test logs, declared build profiles and the binaries actually tested.
Its metadata explicitly distinguishes reuse of verified owned binaries from a
compile in that runner. No declared feature flag alone proves a backend build.
The fresh-engine and final-package checkpoints below record those outcomes.


## Fresh engine checkpoint

The successful current engine record is
`target/label-rename-application/engines-1792808906050018700/evidence.json`,
SHA-256 `0b5f8337e0c8215189ad15b4d2005dc3035520832b96d71b4ebfecea7ef1ff05`.
All fifteen recorded build/test stages pass. The frozen native CLI is
`456dc1a6a0d71bd89b1472949acbba33e990ccd17d463c3948d9b450c55c7c48`;
actual default and C modules are respectively
`198d524a835dd7ed8c7744fd116c62ed1653f49be0b8c33227f92692b7a391aa`
and `48e553a6bc08312ff8a07d61df2125797726cfe99c59daae09e73564ecbd0038`.
Each runs thirteen direct/six batch summary CLI comparisons and eleven rename
CLI cases with forty-one interruption boundaries. Native alias CLI and serial
unit tests are separate from these WASM comparisons.

The actual-record reuse validator passes its positive case and seven adversarial
refusals: stale source, wrong profile, failed prerequisite, changed log, missing
native safety, wrong build flags and escaped copy destination. Raw outcomes are
under `target/label-rename-application/engine-reuse-validation-1792809765221878700/`.
The full registered runner now consumes these frozen current bytes via
`-EngineRecord`; `dual-build-legacy-v5.log` records both profiles passing. Later sections record final-package and archive closure; broader acceptance remains open.


## Final-package checkpoint

`dist/wasm-label-rename-validated` passes all 25 recorded package gates.
The record is
`target/label-rename-application/package-1792810553620447800/package-evidence.json`,
SHA-256 `3498102faf7c98af0ec06175e31e8c36548b79e29bee91665056e5f3d9a91f63`.
Offline version:
`e0c10299943f170ddfac0cf3178d952f0c7080091f6b68b6c90adfd39d2e6828`.
Both registered profiles passed over the verified owned fresh engine bytes.
The actual staged worker/UI, native summary, PDF, transcript, repository writer,
TUI, composed Verify, workspace, loaded-C/native and mutation checks pass.
Current-runtime fixture producers ran before their browser consumers; no identity
assertion was loosened. Final source pins equal the exact staged files tested.
No public deployment, all-state capacity or whole-parent acceptance is claimed.

Archive preflight checks 9,790 file mappings and 1,433 unique identities;
707 new blobs require 159,101,573 bytes against over 1.4 GB available. The
immutable inherited snapshot must still be copied and verified before closure.
Final review/support links follow that freeze, avoiding circular hashes.


## Scoped evidence closure

The verified immutable snapshot is
`target/label-rename-application/package-snapshots/wasm-label-rename-validated/snapshot.json`,
SHA-256 `ae32e13bbf9cca1248d29d4426056bcb29c7d58761500be44728c67f1fb95428`.
It verifies 9,790 mappings / 1,434 unique identities for 1,539,723,084 logical
bytes; 708 new blobs require 159,103,223 bytes. The native-summary snapshot is an
explicit inherited dependency and must remain resolvable and digest-verified.

The [seven-role scoped review](../../signals/roles/check/wasm-label-rename-application-roles-check-2026-10-23.md)
closes the package, registered-runner and archive conditions for this local
scope. The original Windows data-loss witness, failed tests, ENOSPC attempts,
source-pin failure, old runtimes and actual current-runtime fixture outputs are
retained. The archive preserves the execution-stage contract; final links,
availability-unit correction and support/ledger registration follow the freeze
without changing any tested runtime bytes.

Both scoped capability children remain evidence-partial and `acceptance_v1`
stays false. Native OS locks/atomicity, unknown partial-byte repair, other host/API
and capacity cases, complete native label lifecycle and formal parent/release
acceptance remain open. No public site was deployed.
