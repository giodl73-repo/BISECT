# Explicit C METIS browser integration

The optional `c-metis-sidecar` build now connects actual BISECT Rust dispatch
to the vendored C library through a synchronous memory bridge. The default
browser build and requests with no backend selection keep `metis-core`.
This is an experimental local extension, not full desktop-parity or release
approval. Nothing has been deployed.

## Complete local preview build — 2026-10-27

The [complete preview checkpoint](wasm-cffi-full-preview-checkpoint.md) now passes
the entire existing C preview builder, both actual build variants' registered
gates and final-package native C/browser/offline checks. Version `58923b00eff2`
has a hash-verified local distribution and explicit support statement. Earlier
sections saying their scoped checkpoint did not rerun the whole builder remain
historical. This closes that local invocation gap, not clean-checkout, national C,
broader host/capacity, formal parent/release or full desktop-parity acceptance.

## Explicit C retry in the workbench — 2026-10-27

The worker now checks retirement through the private loaded C authority. After
a trap it invalidates its loaded-sidecar cache and reports a fixed recovery code.
The workbench explains that the next Run with C enabled loads a fresh runtime.
It keeps the failed request failed; neither displaying the message nor saving
the project executes or fetches anything. A later explicit C Run creates a fresh
C instance and Rust engine. Ordinary allocator refusal retains its live instance.

`test_cffi_worker_recovery_browser.mjs` exercises actual current workbench sources
and their worker over the retained package's exact engine/C bytes. Its first
asset bundle has deliberately hashed test glue injecting genuine partition/free
WASM traps or one allocator refusal. Trap cases fetch a second, unmodified C
bundle only after another Run click; ordinary refusal succeeds with one load.
All three cases retain a failure receipt, disable completed-plan export, save
inertly and then match the retained native assignment fixture on explicit retry.
The previous worker fails the recovery-code assertion. This is a current-source
overlay integration test, not acceptance of the older immutable package.

Evidence: `target/cffi-worker-recovery/run-aVT2cS/evidence.json`. The checker is
registered with `--require-package` in the C preview builder, enforcing source
identity when used for a final package. The full updated builder has not run.
Real exhaustion, forced termination, other host/runtime profiles and full
parent/release acceptance remain open.

## Controlled C failure checkpoint — 2026-10-27

`node scripts/web/test_cffi_fault_recovery.mjs --browser` passes 16 direct ABI
fault cases and four Rust-engine failure/recovery cases in Node and a dedicated
Chrome module worker. The fixture loads the exact pinned vendored C WASM through
the existing verified loader, with explicitly hashed test glue wrapping its real
allocator/partition exports. A tiny separate WASM `unreachable` export causes
genuine RuntimeError traps. This fixture authority is test-only; project inputs
still cannot supply executable assets or recreate loaded authority.

The direct cases refuse each of five allocations, trap each allocator/free call,
and trap partition execution after writing deliberately invalid private outputs.
Every failure leaves the entire caller memory unchanged. Allocation refusal
frees prior allocations and permits retry. Traps retire the instance with no
further allocator/partition entry. The Rust boundary restores metis-core after
each failure, refuses subsequent calls into retired C instances, and succeeds
with a freshly loaded C instance. Backend restoration is checked against a fresh
Rust result. Removing the delayed-publication fix deterministically reproduces
the cleanup-trap output leak (`target/cffi-faults/before-failure.log`).

Results publish after successful cleanup. This changes the JavaScript bridge;
previous packages and their evidence remain frozen. Current evidence is
`target/cffi-faults/run-mjfLlb/evidence.json`; the runner is registered in the
preview builder, whose full invocation has not been rerun. The tests do not
establish actual browser out-of-memory behavior, forced termination, all native
C failures, large-state capacity, other browsers/CRTs, UI recovery messaging or
full parent/release acceptance.

## Current report-source integration checkpoint — 2026-10-27

The current C-enabled engine also passes the original State report-source
workflow with actual C dispatch. An explicit `c-ffi-engine` CLI/oracle build
supplies fresh independent references. All eleven four-unit RI 2020 N-way cases
match native requested metadata, successful assignments/ordered IDs/populations
and report counts, or the two expected post-generation CSV/directory failures.
The maximum-u64 requested seed survives final manifests. Cold runtimes refuse;
prepared/graph jobs and reviewed reset retain the bound sources and C identity.

Actual packaged Chrome workers pass ten portable-file cases and eleven folder API
cases with explicit C selection, plus nine folder failure/recovery scenarios.
Successful browser assignments/IDs are compared directly with native CLI plans.
Open resets runtime selection/fresh exports and replay remains explicit. Folder
handles use owned OPFS fixtures; this is not an OS picker permission grant.

`check_cffi_report_workflow.mjs` makes these checks repeatable and is wired into
`build_cffi_preview.ps1`. The dedicated gate ran on the exact existing package;
the whole preview-builder invocation was not repeated by this checkpoint. It
also reruns the 52 native C kernel matches/four matching failures and the actual
C workbench worker. A separate metis-core control gate passes with C-specific
environment overrides deliberately set to invalid values, proving those overrides
are ignored by the ordinary report gate.

An initial integrated C attempt exposed another import-observer timeout. The
observer now retains exact added text nodes from delivered mutation records,
instead of reading only the final current text. A deterministic DOM test proves
the prior oracle misses a coalesced Open/autosave transition and that the revised
oracle rejects autosave alone and the wrong filename. Failed attempts/preimages
remain retained. This tests the oracle; actual product validation is still
exercised by the separate UI journeys.

Evidence: `target/report-cffi/evidence.json`. This advances the C/State report
intersection only. It does not accept national C execution, other CRT profiles,
all methods/data/capacities, fatal traps/allocation failure, actual OS grants or
full flag/parent/release support. Earlier registered metis-core results remain
bound to their own frozen sources; this does not repin them to newer harness code.

Current native-reference coverage has **52 exact assignment matches** and
**four matching native failures**. It covers ordinary single/multi bisection,
N-way, prime-factor (including k=5 asymmetric fallback), GeoSection and
dual-constraint AreaSection; 64/300-node source-owned grids, two weight choices
and seeds 7/4294967297. Native reference uses the actual State parser/compositor,
shared native dispatch and vendored C. It is an independent kernel/orchestration
reference, not a full filesystem CLI run. Failure witnesses retain the native
3% prime-factor research bound; they do not count as supported successful runs.

Native build features are part of that reference: C comparisons use a
`c-ffi-engine` build, while existing Rust comparisons use the portable build.
The legacy native recursive/N-way primitives still choose their backend from
build features; a parsed backend declaration alone does not prove that one
native binary switches those primitives. Reconcile that native limitation
before claiming equivalent per-request engine selection in every desktop build.

The actual packaged workbench worker also runs a prepared C request, matches
native assignments, exports native plan/context/report artifacts, and retains
the loaded runtime hashes through Save/Open. Opening a project and selecting
the runtime load no C assets or partitions. Run loads three fixed local package
assets and verifies the JavaScript/WASM bytes. Open retires fresh exports and
resets the runtime-load selection; imported metadata grants no module authority.
The 390px layout passes. A race regression completes two real engine
instantiations out of order and proves only the newest can become ready.
Source edits and explicit replay retain existing rules.

## Interface and runtime contract

- [Rust bridge](../../crates/bisect-metis-sidecar/src/lib.rs) implements the
  `metis::Graph` subset used by the three BISECT C consumers. It validates CSR
  dimensions, weight/target/tolerance arrays and returned labels. A scoped guard
  selects C for the request and restores the prior backend on exit. Native C
  builds retain their existing selection; enabling the browser feature does not
  change the default `MetisEngine`.
- [C entry point](../../crates/bisect-metis-sidecar/c/bridge.c) supports k-way
  and recursive calls, with compile-time checks for the pinned 32-bit index/real
  and option layout. The browser bridge copies data into separate C memory,
  frees its allocations and copies validated results back only on success.
  It does not alias immutable Rust arrays into C.
- [JavaScript bridge](../../web/lab/metis-c-sidecar.js) validates memory ranges,
  dimensions and resource bounds before C calls. Output overlap, unknown
  numbering and invalid pointers refuse calls. A loaded authority is private
  and cannot be recreated from an imported receipt or copied object. The loader
  compiles the verified WASM snapshot and imports the verified JavaScript
  snapshot; it does not fetch executable URLs from project data.
  An unexpected C trap permanently retires that module instance. The worker
  drops its loaded-sidecar cache, allowing a later explicit C Run to construct
  a fresh instance. Ordinary C error returns preserve the tested recovery path.
  Controlled allocation refusal and actual WASM traps now pass through the
  verified loader, bridge and Rust engine in Node and a dedicated Chrome worker.
  Results publish only after successful allocator cleanup; a cleanup trap leaves
  Rust output memory unchanged. See the scoped checkpoint below. Real exhaustion,
  browser termination and wider host/capacity behavior remain open.
- The explicit `msvcrt-v1` profile preserves the measured Windows CRT RNG
  sequence. It is serial and module-local. Linux/macOS libc, all compiler/ABI
  combinations, concurrent calls and every METIS option are not implied. See
  the [runtime feasibility matrix](wasm-cffi-sidecar-feasibility.md).
- Bridge limits are 10 million vertices, 32 constraints and a 256 MiB aggregate
  transfer budget per call; requested parts cannot exceed vertices. Worker
  downloads cap metadata at 128 KiB, glue at 256 KiB and C WASM at 4 MiB.
  These browser bounds are distinct from algorithm correctness and actual
  state-scale capacity. Allocation failure and invalid input remain errors;
  no fallback silently switches to `metis-core`.

Direct request options use `metis_engine: "c-ffi"` and
`metis_runtime: "msvcrt-v1"` together. `metis-core` needs no runtime field;
omitting both preserves existing behavior. Native State/YAML preparation retains
original flags/source and maps an available C declaration to that named runtime.
An ordinary build without the feature still refuses C execution. `gpmetis`
remains the shared native stub. Some native queue-generated requests deliberately
retain their existing portable defaults and need separate reconciliation.

The workbench exposes **Load C METIS for this run (Windows CRT profile)** only
when the compiled bridge is available. This loads support for requests that
select C; it does not rewrite requests choosing another engine. Preparation and
inspection are not execution evidence. The optional module remains loaded in
the worker after a completed run, while later ordinary requests still execute
the Rust kernel. A browser cancellation terminates the worker through the
existing mechanism; interruption/permission/storage coverage is not closed here.

Successful C calls add a separately versioned `bisect-engine-runtime-v2`
packet containing the main WASM identity and both C asset identities. The
existing two-field main-WASM identity and configured-runtime-v1 contract retain
their meanings. The packet records loaded bytes, not source authenticity or
certified generation. Formal Verify/replay/custody support remains WP-06 work.
The packaged C runtime manifest contains asset names/hashes and the local build
receipt digest. Detailed compiler/configuration hashes and commands stay in the
local build record, so developer cache paths are not published as runtime data.

## Build and check

Install the pinned Emscripten SDK explicitly using the
[feasibility setup](wasm-cffi-sidecar-feasibility.md); the build never downloads
it automatically. With the normal laboratory base already built:

```powershell
./scripts/web/build_cffi_preview.ps1 -Base dist/wasm-lab-build -Output dist/my-cffi-preview -BrowserChecks
```

The destination must be new and beneath `dist/`. The base package is retained.
This extension runs the seven affected configuration/State suites against actual
default and C-feature builds, then builds the C sidecar and native C reference.
The portable reference used by the legacy suites is retained separately before
the C reference is rebuilt. It runs the Rust bridge and native comparison checks,
stages current toolkit files and exact runtime/proof/offline identities, reruns
the native matrix using the packaged modules and WASM, and optionally runs real
browser checks. The packaged WASM must match the completed C-feature gate receipt.
It does not rerun the base package's entire desktop parity matrix or deploy.
Missing browser tools are a not-run tier, not passing browser support.

Gate receipts identify each actual build, the test sources and the portable
native oracle. Requested variants start each attempt as not-run, so a failed
retry cannot leave an old passing receipt appearing current. Browser/native
capability differences retain both original replies and permit only named
availability/backend fields (plus validation scope text); algorithm inputs,
seeds, assignments and native YAML bytes retain their exact comparisons.
Compiled C support does not imply a loaded module. Unloaded-runtime refusals
are recorded separately from matching native failures.

Local `target/cffi-integration/package-evidence.json` binds completed package
checks, offline-manifest identity, current test sources and the per-variant
receipts. Browser evidence records the tested toolkit and actual loaded runtime
identities. These remain integration-tier inputs, not formal acceptance-v1
bundles. The legacy suites still use existing retained configuration/State
fixtures; their prerequisites are not a new clean-checkout fixture guarantee.

Verified local run, 2026-10-23: `dist/wasm-cffi-gated` passed all seven legacy
suites on both build variants, three Rust bridge tests, 52 native C assignment
matches/four matching failures against the final packaged engine, toolkit
dependency closure, 625 offline asset hashes/sizes and the actual Chrome
workbench/worker checks. The browser also checked inert Open/runtime selection,
saved runtime identities, retired exports, reverse-order initialization and
390px layout. The engine hash is
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`;
offline version is
`58e56c22b29894f2e60286179f172dd3294ba0ab734bcb129ef10c06f8400d65`.
The bytes match the previous local validated package; this run adds integrated
gate evidence, not new algorithm capacity or a deployment. The integration
review's [B1 follow-up](../../signals/roles/check/wasm-cffi-engine-integration-roles-check-2026-10-22.md#b1-follow-up-2026-10-23)
closes that scoped gate condition; broader acceptance below stays open.

Individual checks:

```powershell
cargo build -p bisect-wasm --release --target wasm32-unknown-unknown --features c-metis-sidecar --locked --offline -j 1
./scripts/web/check_cffi_legacy_gates.ps1 -BothBuilds
python scripts/web/build_metis_sidecar.py
cargo test -p bisect-metis-sidecar --locked --offline
cargo build -p bisect-cli --release --example state_signal_preset_reference --locked --offline -j 1
node scripts/web/test_cffi_engine.mjs
# After staging, compare the exact packaged modules and WASM as well:
node scripts/web/test_cffi_engine.mjs --toolkit dist/my-cffi-preview/toolkit
```

The small graph fixture and frozen legacy core result are source-owned. The
default test requires no old untracked WASM artifact. An optional
`--previous-wasm <file>` checks the frozen core result against its pinned original
artifact as well. The native executable is copied into the evidence directory
so later portable builds cannot silently replace its identity. Raw cases,
native outcomes, C results and matching failures remain under
`target/cffi-integration/`. They are investigative acceptance inputs, not yet
formal `bisect-parity-acceptance-v1` bundles.

## Remaining acceptance

The affected legacy gates now distinguish optional feature capability. Broader
final-artifact core regressions and C method/search/weight coverage remain open.
Exercise actual native CLI file orchestration, remaining TUI/jobs,
multiscale, NRS recursive-mode/error paths, optional analyses and explicit replay.
Expand the predeclared real-data/browser/capacity matrix, cancellation and memory
failure tests. Finish versioned runtime custody in Verify, package gates and
third-party notice retention, and role sign-off before expanding the advertised support statement. The useful
local extension does not close WP-02, WP-06, WP-08 or WP-10 by itself.
