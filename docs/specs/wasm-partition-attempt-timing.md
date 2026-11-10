# Native State partition-attempt timing

The explicit State workflow can map `--time-partition` into real per-attempt
dispatcher observations. This extends position routing and preserved inactive
debug; it does not turn an elapsed-time number into deterministic evidence.

## Measurement contract

Native `runner/mod.rs` starts `partition_t0` after weight preparation, immediately
before the split dispatcher. It prints `[partition-time]` after a successful
dispatcher return and before final population validation, then retries a failed
balance check in native seed order. The new WASM capture uses corresponding
ordinary and multiscale dispatcher boundaries. Browser source/weight preparation,
post-dispatch metrics, final balance checks, export and follow-ups are excluded.
A single-district shortcut is still a dispatcher return and is observed. Native
block-group multiscale may load its fine adjacency inside the timed branch; that
OS file work is absent from this in-memory browser interval. This adaptation does
not promise identical timed instructions or numerically comparable measurements.

Capture is opt-in through `run-prepared-engine-timed`. Ordinary prepared operations
and inspect-only review keep their prior replies and add no measurement-clock
reads. Existing algorithm timeout clocks keep their prior behavior. The
explicit operation wraps the existing prepared runner; graph/options, dispatcher,
population retry policy, accepted seed, exports and replay bytes stay unchanged.
Native reference execution uses `Instant`; WASM uses the existing host monotonic
millisecond import. Measurements are observations on the actual runtime, with no
assertion that native and browser numbers should match.

Each event (`bisect-native-partition-timing-v1`) records state/year, parent-graph
units, districts, zero-based attempt, exact nullable decimal-u64 seed, clock class,
status and elapsed milliseconds. A multiscale event covers the sampler dispatch;
its unit count names the parent graph, including block-group-output cases. Nested
dispatch instrumentation does not double count. Generation errors produce no
completed event for that failed dispatcher; completed returns before a later
population-exhaustion error are streamed normally.

On success, the prepared receipt includes a `bisect-partition-timing-report-v1`
report. It retains at most 64 events plus an explicit omitted count. The ordinary
native retry ceiling is 51 attempts. Capture guards clear state after errors;
a later execution starts at attempt zero. Nonfinite, negative or backward clock
readings yield `clock-unavailable` with null elapsed time, while preserving the
generated result. That case is distinct from a zero-millisecond observation.

## Workflow and UI boundary

The State binding retains original flag text and hashes. Its optional
`time_partition_requested: true` field is reconstructed from the actual native
parser during execution recheck. Changed or removed choices refuse before source
preparation/partition. Older saved bindings containing a previously refused timing
flag require a fresh handoff; Open does not upgrade them automatically.

Only after explicit Run does the checked binding select the timed internal entry
point. The direct engine-only State compiler retains its orchestration refusals.
Timing is not inferred from STATUS events, round snapshots or total job duration.
Native round schemas remain unchanged. The host adapter applies the same explicit
C-sidecar preflight to timed and untimed prepared operations; unavailable C never
becomes a substituted Rust run.

Live progress shows a separate `[partition-time]` observation, including at STATUS
position 999. Successful single-State and queued results show recorded timing
tables, and JSON/project/ZIP receipts preserve the report. Retained/imported tables
are passive, unverified data; malformed timing records show an invalid-record note.
They never reconstruct a timer, source, grant or execution. Population-exhaustion
still returns the normal engine error; no failed plan is accepted merely because
completed dispatcher timings were observed.

## Verification requirements

Native Rust tests cover invalid clocks, nested dispatch, dropped captures and
bounded retention. `test_partition_timing.mjs` uses a controlled host clock to
check two clock reads per dispatch and exact controlled test intervals, including observed zero. These
are boundary tests, not performance benchmarks. It covers a first-attempt State
run with max-u64 seed, a retained success on attempt four, all 51 exhausted attempts
and wrapping seeds, tract/BG fixed/adaptive multiscale, unavailable clocks, untouched
non-timing replies/replay bytes, strict report validation, inactive review/default
measurement clocks in the tested ordinary path and a changed timing-choice refusal before source/Run.

The native adapter comparison ignores only the newly requested timing report and
checks every other returned field. An actual native CLI run at position 999 checks
the native timing line. The C-enabled test loads the verified C sidecar and checks
an actual timed C run against its untimed result, then verifies default Rust
selection still works. This is a bounded C timing case, not all-method C parity.

`test_partition_timing_browser.mjs` exercises the full workbench's explicit flag
handoff, source queue, real worker timing progress at position 999, successful
timing tables, max-u64 seed, mobile rendering and inert project reopening.
Final packages require fresh registered default/C build gates, module/offline
closure and their own exact-runtime timing/browser/regression checks. Broader
source/method/capacity/host and full parent acceptance remain open.

## Final local evidence — 2026-10-26

- Registered dual-build log: `target/partition-timing/registered-legacy.log`.
- Final V2 package checks: `target/partition-timing/package-1793046626258235600/evidence.json`
  (16/16, both source and package identities stable).
- C runtime follow-up: `target/partition-timing/c-runtime-evidence.json` (52 native
  assignment matches, four matching failures, real C batch/workbench workers and
  an actual timed C browser/native assignment case).
- Earlier functional V1/mobile-layout gap: `target/partition-timing/prototype-table/evidence.json`.

The final timing table uses three columns, with graph context above it. V1 and V2
narrow-view images are retained; existing wider State-result tables still need
separate UX work. The [seven-role implementation review](../../signals/roles/check/wasm-partition-attempt-timing-roles-check-2026-10-26.md)
approves this bounded adapter with broader matrix/host conditions. No deployment
or full desktop/parent acceptance is promoted.

## Retained scope

`target/partition-timing/snapshot-scope/snapshot.json`, SHA-256
`2eaa542ff09f74dc5974c035b80b7784476f16619e087f313c8dfda890f84215`,
fully verifies 6,376 mappings /1,529 identities /1,069,256,313 logical bytes. It
retains exact V2 packages, registered gate records and their declared evidence
roots, C runtime/native/browser results, source preimages, V1 table prototype and
pre-closure contract/review. Browser profiles are excluded; 173,977,450 new unique
bytes were needed relative to the verified prior scope. This post-freeze pointer
does not accept the remaining broad matrix/host or full parent conditions.
