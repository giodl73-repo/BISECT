# Reviewed native Fetch folder installation

Full native release bundles can now be reviewed and applied to a user-selected
native **outputs** folder. The controls appear only when complete Fetch ZIP mode
is selected. Select folder, Find/Download, Review and Apply are separate actions.
ZIP export remains available when direct folder access is unavailable.

Find and Download check native completion before network acquisition when a folder
is selected. The native rule is path existence for both
`V3/data/<year>/adjacency/<state>_adjacency_<year>.pkl` and its `.done` marker.
It does not validate either path's contents. Without Force, that presence skips
acquisition. Force permits a new download and a concrete replacement review.

Review shows each file, size, new SHA-256 and existing-file identity. It validates
all retained source bytes, copies them privately, checks current write permission
and fingerprints only planned paths plus native primary/marker paths. Unrelated
files are preserved, including obsolete local release files and other states.
Case aliases and a directory occupying a planned write path refuse before writes.
Selected-path review is bounded to 128 MiB; directory listings to 10,000 entries.

Apply requires that exact live review object and explicit confirmation. Imported,
copied, retired or consumed reviews confer no authority. Permission, scope/revision,
cancellation and target fingerprints are checked around I/O. The browser removes
an old marker before source writes, reads back every committed file, then writes
and verifies exact native `done` bytes last. Unlike the native CLI's force failure
path, this deliberate safety adaptation invalidates the old marker before partial
replacement. Successful supported file trees match the native CLI exactly.

Failures retain completed files and report attempted paths, effects that may have
started, marker invalidation/commit and verified paths. Download the outcome before
leaving. It is a historical receipt, never a root handle or recovery grant. New
review of an incomplete target permits a deliberate full rewrite from retained
sources; it does not automatically resume or skip individual completed files.
Closing/reloading loses this tool's live root, acquired buffers and review. Fetch
outcomes currently have no durable cross-reload history. Open drops the root and
review, resets Force and performs no picker, network, engine or write operation;
an earlier passive outcome remains downloadable in the current page.

## Native limits and authority

Native presence is not graph validity or source authenticity. A failed final marker
creation/write or an external concurrent change can leave presence different from
the attempt's verified outcome; re-review the actual target and use explicit Force
when replacement is intended. Browser APIs provide no cross-file transaction or
OS-wide lock. Fingerprint checks reject observed changes but cannot exclude a race
between observation and another process's write. Completed files are not rolled
back. No pickle/request/display file is decoded or executed. Engine inputs and
saved projects remain separate from this folder application.

The [File System Access API](https://wicg.github.io/file-system-access/) supplies
picker and permission behavior. The application does not store handles in project
files or request permission on Open. The browser governs its own remembered
permission state; dropping this tool's handle does not claim to revoke that state.

## Verification

`target/native-fetch-install/package-1793035590254475300/evidence.json` passes
18 stages across unchanged default/C-enabled engine profiles. Both use retained,
hash-checked original private RI 2020 release assets. Five fresh actual native CLI
file-tree comparisons per profile cover absent targets, primary-only, marker-only,
presence skip and forced replacement; only gh's external transport is controlled.
Copied/consumed grants, absent confirmation, stale files, revoked permission,
write/corrupt-readback failure, source-buffer mutation and cancelled re-review
recovery are exercised. Opaque files remain byte-exact and unexecuted.

Actual browser UI journeys use injected picker/permission APIs backed by isolated
real Node folders. They prove no network on presence skip, no writes before Review
and Apply, forced stale-target refusal, marker-first invalidation/last commit,
outcome download, inert Open, no engine request and 390px layout. These are not real
Windows picker/permission grants.

Separate checks use genuine Chrome 155.0.8059.85 `FileSystemDirectoryHandle` objects
in origin-private storage, with no injected handle or permission APIs:

- Default: `target/native-fetch-install/opfs-1793035734683/evidence.json`.
- C-enabled shell: `target/native-fetch-install/opfs-1793035738021/evidence.json`.

Both pass real write/readback, presence skip, force cancellation, marker removal,
re-review recovery and stale-target refusal. Origin-private storage automatically
has different permissions from a user-selected native directory. The native picker
API is present, but actual Windows picker/permission testing is **not run**: the
computer-use runtime failed initialization twice with `failed to write kernel
assets (os error 3)` before it could open a window. No permission prompt was acted
on. This remains a host acceptance gap rather than a fabricated passing gate.

A targeted asynchronous fault cancels while the final directory lookup resolves.
The prototype creates an empty source file after cancellation:
`target/native-fetch-install/await-cancel-1793035734029/evidence.json` (expected red).
Final v2 adds post-await guards and creates no file:
`target/native-fetch-install/await-cancel-1793035734326/evidence.json` (green).
The earlier package is retained and is not the accepted final implementation.

Real selected-directory grants/revocation, native browser-close recovery, durable
Fetch outcome history, cross-browser/OS locking, wider source/scale and hosted
deployment remain unverified. Parent/acceptance-v1 remains open.

## Retained scope

`target/native-fetch-install/snapshot-scope/snapshot.json`, SHA-256
`d0d57b605206f575c5b6f0fccfae5d1c3946828353f43391a4cbb21d469a030e`,
fully verifies 1,823 mappings /547 identities /504,358,768 logical bytes. It retains
both final packages, native/injected-browser evidence, real OPFS results, targeted
prototype-red/final-green, preimages and pre-closure contract/review. This
post-freeze pointer does not accept the missing native permission or broader
lifecycle/host gates.
