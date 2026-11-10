# Native State diagnostic workflow flags

An explicit State workflow handoff now accepts `--position` and `--debug` alongside
source/output choices. It retains the original argument JSON and exact seed,
derives versioned diagnostic settings from the real native parser, and removes
these orchestration fields from the compiled engine command. Rechecking rebuilds
the binding from the original flags and compares algorithm/profile results before
source preparation or partition. Changed settings refuse execution.

Native authority is `state_args.rs`, `main.rs`, `runner/mod.rs` and `status.rs`.
The State entry point copies `args.position` and `args.debug` into StateConfig.
Position is used at STATUS call sites. Debug is assigned but has no runtime
consumer in the current State runner; this handoff preserves that limitation.
It introduces no debug log mode, new randomness, algorithm change or timing claim.

## Observable behavior

- Position is an exact signed 32-bit value, default 1. Browser orchestration stage
  messages use `STATUS:<position>:<ASCII message>\n` and a versioned progress event.
  Messages are browser workflow stages, not a byte-identical transcript of native
  source loading, weighting or partition diagnostics.
- Position 999 still emits/retains status events, while the browser coordinator
  hides those STATUS updates, following the native protocol's parent-ignore
  convention. Native round snapshots remain unchanged and separately displayed.
- The UI shows the selected position and explains inactive debug in workflow
  history. Worker status reaches single-State and queued State UI paths. Completed
  receipts retain at most 64 status events with an explicit omitted-event count.
  This is bounded operational history, not a complete execution audit.
- Non-ASCII characters and controls in generated stage messages become `?`, with
  a 2,000-character bound and a single-line protocol frame. Unknown event fields,
  invalid positions and mismatched lines are rejected. Native round-event schemas
  receive no added fields.
- Save/Open retains passive choices and results; applying choices, preparing,
  queueing and execution remain explicit. No filesystem path or folder authority
  is created by these diagnostic settings. Older saved bindings with previously
  refused debug/position flags require a fresh handoff; Open does not upgrade them
  into executable choices.

The direct engine-only State compiler keeps its original orchestration refusals.
Use the explicit workflow handoff to support these flags. `--time-partition` stays
refused there: timing the whole browser job would not measure the native per-attempt
partition interval. `print_only`, target actions and other orchestration fields
continue through their existing separate boundaries.

## Evidence and limits

`test_state_diagnostic_flags.mjs` covers signed-int bounds, default/debug-only and
position-only settings, exact max-u64 seed, stale binding refusal before source or
partition, passive projects, untouched native round events and bounded retention.
Four two-district executions compare exact assignments with the independent native
State parser/n-way kernel. Ordinary selected binary and custom precinct State paths
also retain diagnostic settings. The custom path supplies an explicit county
companion; it does not infer county IDs from arbitrary unit identifiers.

Eight actual native CLI one-district runs combine four positions with debug on/off
and check STATUS framing. They support the diagnostic protocol comparison; they
do not establish every native multi-district status message. The full packaged
browser test covers inspect/apply/queue/run, real worker events, hidden 999 updates,
inactive debug disclosure, exact seed, mobile width and inert project reopening.

Both unchanged default/C-enabled packages pass 16 stages in
`target/state-diagnostic-flags/package-1793041054136557100/evidence.json`.
Module/offline closure, native workflow comparisons and Fetch UI regressions pass.
The legacy worker-pool harness uses its existing default artifact; both candidate
shells independently pass the new actual browser and native-assignment checks.
No C partition execution or broader method/source/capacity claim follows.

Scope remains implemented/evidence-partial. Native partition-attempt timing,
remaining flags and source/host/full parent acceptance remain open.

## Retained scope

`target/state-diagnostic-flags/snapshot-scope/snapshot.json`, SHA-256
`39d39b5ce8f3dca865fee911bda39cd5b6230a6147b425587bdbc5815a3fe83f`,
fully verifies 1,729 mappings /554 identities /514,949,650 logical bytes. Exact
packages, native/browser outputs, source preimages and pre-closure contract/review
are retained. This post-freeze pointer accepts no remaining timing, broad source,
backend or host gate.
