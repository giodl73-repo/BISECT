# Doctor tutorial and empty-JSON parity

Native Doctor parses an empty manifest/checksum file and returns a JSON error.
The shared WASM manifest/tutorial adapters now do the same. Explicit native JSON
file readers support zero bytes and preserve BOM/CRLF through parsing and hashing.
Generic source readers retain their nonempty default; callers must explicitly
request empty-source support. The leading-BOM tutorial counterexample is fixed:
native and corrected WASM return 2 where the old normalized decoder returned 0.

Tutorial controls offer structured and complete Doctor text reports. Users select
checksum JSON and local files or a repository folder, then explicitly Run. Folder
selection keeps referenced relative paths and ignores unrelated files. Rust parses
and validates the document before any file read; the worker hashes
only referenced selected paths, in reference order. Unrelated selections and
invalid documents perform no body reads. The worker streams file bytes through
Rust SHA-256 in 1 MiB chunks. Projects retain original
checksum bytes/results, not live file handles or an automatic rehash instruction.
After Open users reselect files and Run to establish fresh observations.

Missing files retain native comparison code 0 and remain visibly missing. A hash
mismatch or read error is 1; bad JSON/schema is 2. Pins, URLs, build labels and
tract counts are informational. Counts are not recomputed, URLs are not fetched,
and matching supplied hashes does not authenticate pins, sources or execution.
Text names the WASM target and replaces the native checksum path with the selected
source label. IO-host error text may differ; its read-error class/status is kept.
No browser native process or arbitrary filesystem access is inferred.

Checksum reads have an owner token, revision guard and explicit Cancel. Run,
source controls and downloads wait for the read. Cancelled A cannot release B;
reentrant handlers cannot acquire or release another read; late reads cannot
overwrite Open. Cancellation retires callbacks, not physical IO. Local files are
session-only and imported results do not enable fresh exports.

## Current evidence

Fresh default and C-enabled builds are retained in
`target/tutorial-cli-contract/build`, with exact Rust source pins checked after
both builds. The first Windows PowerShell 5 wrapper stopped on a compiler warning;
its failed status/log is retained. No compiler remained active before the terminal
retry under PowerShell 7. Both actual builds then completed successfully. No
current native-unit-test build or whole-builder pass is inferred from them.

| Profile | Manifest actual CLI / browser | Tutorial actual CLI / browser |
|---|---|---|
| Default | `target/manifest-cli-contract/run-z1OUO8`, `browser-NR4eQn` | `target/tutorial-cli-contract/run-xbnX4o`, `browser-7gVHbN` |
| C-enabled | `target/manifest-cli-contract/run-GkcwRl`, `browser-tcctP9` | `target/tutorial-cli-contract/run-Mnydu0`, `browser-sTNBQG` |

Each manifest pair passes 21 cases including zero bytes. Each tutorial pair passes
19 cases: pass/missing/drift/mixed, uppercase and unrecorded pins, zero-byte data,
read error, tutorial mismatch, exact large tract count, duplicate references,
schema/malformed/shape/BOM/empty JSON, ignored extra selections, CRLF and the
512 KiB bound. Invalid schema/JSON fixtures include unreadable selected files
and assert no hashing. Each valid case asserts the exact referenced hash order. Actual CLI text
is compared completely except the explicit target heading, selected checksum path
and platform read-error line. Reference and selected byte hashes are retained.

Browser runs use real file inputs, an actual folder selection and the actual
worker/hash implementation. A same-origin test worker shim injects one file read
error; the original CSP stays intact. Main-thread delayed-read faults exercise
A/cancel/B/reentrant/late-A/Open/late-B. Earlier shim initialization timeouts remain
diagnostics and are not accepted as product failures or passing evidence.

Both new profiles also retain 15 native Policy source-chain cases and 68 strict
backward cases each. The new C module passes 52 native assignment matches/four
matching failures. Project regression passes.

`dist/wasm-tutorial-cli-verified`, version
`4a0b7a3930a89227647fea0867a6879290fd7e1791b95e4239f36ea99ff2d23f`,
contains 737 assets, 637 shell assets and 50 state/year packs, with 1,247 local
dependencies across 314 toolkit files. C browser checks bind staged source,
engine and all workbench aliases; the laboratory landing page is preserved.
The earlier partial stage assumed a nonexistent toolkit catalog and stopped
before updating its entry page. It is explicitly superseded, even though its
asset hashes were internally consistent. Hash consistency alone is not UI proof.

Both builders register tutorial and expanded manifest checks. Wider input/path
capacity, actual OS picker/read-error grants, native location workspace, wide plan
metadata, clean-checkout, new whole-builder, deployment and formal full-parent
acceptance remain open. This checkpoint supersedes the earlier missing-empty
and tutorial-reconciliation notes only for the supported selected-input workflow.

## Retention pointer

`target/tutorial-cli-contract/closure.json`, SHA-256
`10483aef1ab66d0f30720fbf4540049c8ce63c993cee2996b30806cdc05627f3`,
retains 2,346 mappings /1,523 identities. Independent
`target/tutorial-cli-contract/readback.json`, SHA-256
`c864379cd69394d34d35db8b58261c2106ca5012cb1df020511864466cd4a090`,
verifies all mapped blobs and the unchanged earlier release ZIP. The snapshot
retains the pre-pointer text. Its local store has 84,965,831 logical bytes,
including 50,931,229 bytes linked from owned immutable WASM artifacts; this is
not a physical allocation estimate. The closure's allocation counters cover only
the successful resumed invocation after a disk-reserve stop.

One verified, nonrecursive compiler-symbol cache deletion recovered 207,388,672
measured physical bytes; its record is retained. Sources, executable binaries,
test evidence and archives were untouched. New build/output directories must be
used rather than overwriting linked sealed artifacts. Final project, source-ledger,
practitioner, dependency, offline and builder syntax checks pass; 42 Fetch planning/
handoff comparisons per profile pass without acquisition. No new whole-builder,
deployment or full-parent acceptance is claimed.
