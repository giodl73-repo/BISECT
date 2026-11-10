# Reviewed RCOUNT transcript installation

Date: 2026-10-23. Status: source implementation, actual default/C-enabled
Node/browser checks and final-package regressions pass. Immutable snapshot and
scoped review references are registered after the execution-byte freeze; the
archived version of this document can precede those closure references. This
does not close WP-03/WP-07 or desktop parity.

Tested package: `dist/wasm-rcount-transcript-validated`. Its execution record is
`target/report-pdf-delivery/build-1792799335204662500/package-evidence.json`.
The package's offline version is
`0fe3bc5107398eb03ada3b6b29332981e150e7bdb605d8970f2a7ddad35df141`.
Actual engines are the existing default and optional-C artifacts; the native
RCOUNT CLI was built from the current worktree and its identity, Rust/Cargo
versions and 65 source/manifests are retained separately. This is not a fresh
WASM compilation or complete source-to-binary reproduction claim.

Native authority: `rcount-cli/src/main.rs::run_verify` and
`rcount-audit/src/lib.rs::write_verification_transcript`. Desktop writes
`transcripts/verify-transcript.json` as pretty JSON without a trailing newline,
creating the directory and replacing the file. Both passing and failing
verification produce a transcript when requested; CLI exit remains 0/1 for
verification, with write errors separate. Browser verification uses the existing
`verify-count-files` operation and shared verifier. No new Rust/WASM ABI is added.

## Browser workflow

Run verification on selected count package files, then select **Review package
transcript destination**. Choose the corresponding writable directory. The
review recomputes verification in a separate identity-bound WASM worker and
requires every selected source file and byte to match the directory, except
the fixed transcript destination. Unexpected files are refused. This bounded
exact-roster policy is stronger than desktop directory discovery and is an
explicit browser adaptation, not unrestricted native filesystem equivalence.

Review shows pass/fail, the exact fixed path, add/replace action, previous hash,
new hash and byte length. Confirm that concrete change, then choose **Write
reviewed transcript**. The private single-use review is not serialized. Imported
projects, paths, results, reviews or receipts cannot grant write permission.
Open remains inert; fresh verification and selected-root review are required.
The existing result download path remains available without directory APIs.

The writer rechecks all source bytes and destination state before writing and
all source bytes plus the written transcript afterward. A separate receipt
records native package/manifest identities, exact source-inventory SHA-256,
transcript and previous-destination hashes, verification status and observed
write/read-back outcome. Source inventory hashes the sorted path/hash/length
roster excluding only the fixed transcript destination. This raw-byte identity
is separate from the native semantic content hashes.
`write_started` conservatively marks entry into the potentially mutating write
phase; it does not prove that any bytes were committed. Only successful
read-back establishes the completed transcript outcome.

Cancellation, permission loss, stale context and changed inputs refuse or
interrupt work. Started writes can remain; no OS-atomic transaction, locking or
rollback is claimed. A new review explicitly observes the current destination
before any replacement. A completed write is not an election certification.
Supplied lawful status, amendment lineage, ballot/contest/batch distinctions
and source evidence are retained by preserving the entire input package; no
new official status or inferred total is created.

## Comparison and evidence

`test_count_transcript_installation.mjs` uses tracked synthetic summary-basic
inputs and an actual freshly built `rcount` CLI. It compares complete on-disk
native transcript bytes for absent destination, replacement and source-tampered
failed verification. Default and C-enabled engines match all three cases.
An actual CLI write-error witness exits 2 when the fixed transcript destination
is a directory; browser review refuses that destination before writing.
The serializer restores native struct field order from the existing value
response; transcript/check fields are typed strings, with no numeric projection.
Original outputs and native exit states are retained separately.

Guards cover imported/consumed reviews, explicit confirmation, changed
destination, changed/extra package files, permission refusal and cancellation
after a real committed write. Browser tests use an actual worker and owned OPFS,
an injected picker, inert Open, no-write review, confirmation, review retirement,
native CLI consumption of exported bytes, interruption and 390px layout.
OS picker/permission behavior and other browser hosts are not established.

The standard, laboratory, dual-build and final-workspace builders register the
new checks. Those complete standard builders were not rerun as a whole. The
scoped builder reused verified native/engine bytes and ran both source engines,
the actual final package, PDF content/rendering, report-preview security,
label-installation, TUI/composed Verify, workspace, explicit-C/loaded-C,
project and ledger mutation gates. All recorded stages pass. Native build
identity is separate from reused WASM identity. Final immutable evidence,
review and support registration follow these runtime checks.

## Scoped closure

Immutable execution snapshot:
`target/report-pdf-delivery/package-snapshots/wasm-rcount-transcript-validated/snapshot.json`,
SHA-256 `0ec7f647938b1c2a2768d196ec7aa19e571db957fd74e9f1196f0d89c0db839f`.
All 1,869 mappings / 903 unique identities verify against the explicit inherited
base; 474 new blobs require 28,687,850 bytes for 407,974,020 logical mapped bytes.
Keep the base manifest and blobs available. The execution record SHA-256 is
`682dcc6a2c27c42bb6c52ddb9571c40796d7583cf825c0954b52e5c9d3156ea9`.

The [eight-role implementation review](../../signals/roles/check/wasm-rcount-transcript-installation-roles-check-2026-10-23.md)
accepts this local scope with carried limitations. The `rcount.Verify` child
remains evidence-partial and `acceptance_v1` remains false. This closure section,
review, support wording and inventory re-ranking follow the immutable freeze;
the snapshot preserves their prior execution-stage versions. These registration
edits change no package runtime bytes. Exact native label summary, reviewed
rename/import and broader host/capacity/release acceptance remain open.
