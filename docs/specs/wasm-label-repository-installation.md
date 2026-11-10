# Reviewed label repository installation

Date: 2026-10-23. Status: scoped local package, both actual engine builds, native CLI consumption
and actual worker/OPFS browser checks pass. Full WP-03/WP-07/WP-10 remains open.

## User journey

In **Prepare repository stage package**, select the existing stage JSON with
current repository snapshots and explicit prior artifact hashes or absence
expectations. Run the native merge. Download the prepared ZIP, or choose
**Review selected repository** where writable directory handles are supported.
Review the concrete paths, confirm the listed changes, then apply.
Opening a project does not run, choose a directory or authorize installation.

The review recomputes the merge in a separate actual WASM worker bound to the
currently loaded engine identity. The worker receives no directory handle.
Only the UI-side writer retains the selected root through a private single-use
review. Inputs, operation or project changes retire that review. Installation
preserves unrelated files and does not generate a new partition, legal finding
or election certification.

## Observable native boundary

Native authority is `analyze_label.rs` stage output and `run_registry.rs` registry
semantics, plus `label_package_merge.rs` reconciliation and
`label_cmd.rs::run_verify` consumption of the installed index chain.
This implements reviewed application of already prepared analysis/report text
artifacts. It does not port every native label command's filesystem lifecycle,
rename/import/build output installation, automatic stage discovery or native
filesystem locking/atomic registry replacement.

All current supplied files, upstream indexes and intended changed paths are
observed before writing. Explicit missing-file expectations must still be absent.
Paths are normalized bounded portable relative names; case aliases and file/
directory conflicts refuse. The supplied snapshot is limited to 2,000 text
files and 25 MiB, with individual observed files bounded by 25 MiB. Unlisted
files are preserved without reading their contents. Original UTF-8 text bytes,
line endings and per-file native SHA-256 digests are retained.

Writes close and read back each artifact. The generated stage index follows
ordinary artifacts, and `.bisect` is written last. All observed source/target
hashes are rechecked before each commit index and the registry. Completion also
checks every intended result file. Directory permission is rechecked before
effects. Browser APIs do not provide an OS-atomic transaction: concurrent edits,
permission loss, disk exhaustion, closure and cancellation can leave effects.

## Interruption and continuation

Cancel stops further work but does not undo completed writes. An explicit receipt
download records attempted/completed paths, whether effects may have started and
whether registry writing/readback occurred. Receipts are passive unauthenticated
records; importing or copying one grants no root, write or resume authority.
The receipt is not automatically embedded in saved project files; export it
explicitly before closing the tab if it is needed as a historical record.

**Review explicit continuation** recomputes the original merge and freshly
observes the selected root. Only exact original or intended bytes are eligible.
Verified intended files are retained without rewriting; remaining eligible paths
are written with the same commit order and readbacks. A registry already at its
intended bytes while an artifact remains incomplete refuses continuation.
Unknown partial bytes or concurrent upstream/target edits require a new explicit
conflict review/re-preparation; continuation never silently repairs or overwrites
them. Automatic rollback, background recovery and imported write authority are
not provided.

## Evidence tiers

- `test_label_repository_installation.mjs` regenerates a synthetic grid fixture
  from tracked RPLAN/RCTX inputs and native config/analysis/report kernels. It
  compares both merges with the native host, installs analysis then report into
  owned test directories and invokes the actual native `label-verify` CLI.
  A configuration tamper must fail that CLI. The supplied build index explicitly
  states that no build/partition was performed by this fixture.
- Disk-backed adapter cases check no review writes, explicit confirmation,
  private/consumed/retired review refusal, exact unlisted binary preservation,
  registry-last/index ordering, corrupted readback, stale/changed sources,
  permission loss, before/after-effect cancellation and explicit continuation.
- `test_label_repository_installation_browser.mjs` uses an owned headless Chrome,
  actual engine/guard workers and real OPFS with an injected picker. It exercises
  inert Open, explicit Run/review/confirmation, edits retiring authority,
  cancellation after real close, continuation and exact analysis/report files.
  Exported installed bytes are consumed by the actual native CLI. A 390px layout
  check is included. No external network or user directory is used.

| Host feature | Outcome |
|---|---|
| Tested Chrome worker + OPFS / injected directory picker | Source-tree and final-package journeys pass. |
| Actual OS directory picker, OS permissions and disk quota failures | Not tested; no universal host-support claim. |
| Directory picker unavailable | Reviewed installation unavailable; existing explicit repository ZIP remains the portable path. |
| Save/Open | Existing stage source/result retained; fresh Run and live review required. Receipt download is separate. |

The local package is `dist/wasm-label-repository-validated`, with offline identity
`611adb138536c14778b3a41a2492bd7c17360876d771872abaf07a7fdae27bca`.
The unchanged default engine is
`60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`;
the unchanged C-enabled engine is
`56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`.
Asset/offline gates verify 655 exact files, 555 shell files, 50 state/year packs,
273 toolkit files and 1,113 local dependency resolutions. These are asset-closure
counts, not completed capability counts.

The first naive snapshot-copy attempt exhausted local disk space (WinError 112).
Its failure record is retained; only its generated incomplete duplicate copy was
removed. The revised helper checks free space with a reserve before creating
output and stores identical bytes once as content-addressed blobs. A one-byte
budget test refuses without creating a destination. Each original path retains
its SHA-256, size and blob mapping; copied blob hashes are checked before the
snapshot manifest is published. This is local evidence storage, not a change to
hosted assets or user project files.

The [immutable snapshot](../../target/label-repository-delivery/package-snapshots/wasm-label-repository-validated-deduplicated-v2/snapshot.json)
binds the final package, raw receipts/results, source modules, served browser
assets, owned native executables and both actual build gate records. Final
roles review and capability-ledger registration follow this execution snapshot
to avoid circular self-hashes; they do not change packaged runtime bytes.
The [scoped implementation review](../../signals/roles/check/wasm-label-repository-installation-roles-check-2026-10-23.md)
records the resulting acceptance boundary. Both engine-profile gates pass;
the default profile preserves its original gate record and separately records
revalidation of only the affected label Node/browser checks after receipt
identity refinement. Other legacy outcomes are retained without pretending that
those unchanged checks reran during the refinement.

Final-package acquisition/local/cache, reset/target actions, single/two-district
workspace continuation, native bulk scheduling, configured/native acquisition
recovery, TUI/composed Verify, workspace discovery/install/recovery, paired-source
streaming and explicit C batch/runtime regressions pass. Project Save/Open and
the eight ledger/four native-consumption mutation guards also pass.
The ordinary ledger permits the explicitly partial rows; the full-parity gate
continues to reject unresolved required parents.
Matching native label-chain checks are integrity checks, not source authenticity.
No all-state capacity, formal acceptance-v1, full parity or deployment claim follows.
