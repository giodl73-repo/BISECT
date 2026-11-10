# Recover paired state-source acquisition

The paired state acquisition UI now retains a recoverable checkpoint. If a
state fails or acquisition is cancelled, completed pairs remain queued.
**Resume unfinished source acquisition** retries only failed or unfinished
pairs, using the current network/cache settings. It preserves the already
queued recipes, original archives and exact seeds. Running those jobs remains
a separate action.

After Open, Resume is disabled. **Check saved source archives for recovery**
explicitly checks the saved checkpoint against fresh native Fetch planning,
hashes and decodes its retained ZIPs, rebuilds its queued jobs and compares them
with the retained recipes. That check performs no provider request, graph
preparation or partition. A successful check enables Resume only when unfinished
pairs remain. A copied verification marker cannot enable it.

This is source consistency and acquisition recovery. It does not authenticate
the provider bytes, establish historical engine execution, install outputs or
complete the broader Verify workflow.

## Contract and boundaries

[The recovery module](../../web/lab/state-source-acquisition-recovery.js) adds
`bisect-state-source-batch-acquisition-v2`. It retains the native source plan,
queue, per-state job digests, request identity and summary. Private live proofs
bind completed checkpoints to those bytes. Resume checks both the unchanged
checkpoint and the current queue/context before fetching anything. The context
binds the retained project-source files and execution controls, excluding the
mutable queue/state-selection fields. Even unrelated retained source-file edits
can therefore require a fresh check; this is a conservative snapshot boundary.

Verification regenerates the source plan from the project's Fetch selection
through the loaded native inspector. It compares the complete plan, including
manifest identity and pinned endpoints. Every retained archive must match its
receipt's byte hash, scope and planned URL. Bounded archive decoding feeds
[the same job constructor](../../web/lab/state-source-acquisition.js) used by new
acquisition. The rebuilt job must match the retained job digest, including its
derived source bytes, configuration, metadata and raw archive evidence. Checking
only a saved hash or receipt would not establish that relationship.

The verifier returns an explicit source-consistency record, with no network or
partition performed and source authenticity marked unverified. Browser runs bind
the current native inspection artifact. Imported records grant no capability;
only completion of the current verification creates a private live proof.
Cancellation before verification completes retires that proof. A new archive pair
is committed only after its complete job is validated and cancellation is checked.
One failed pair cannot erase earlier accepted jobs.

The source constructor is shared rather than reimplemented. Native State and
configured YAML/form paths retain their existing behavior. The UI persists a
generated configured-run timestamp so Save/Open does not accidentally create a
different execution context during verification. Existing v1 acquisition results
remain inert and usable as queued jobs; they do not acquire v2 Resume authority.

Existing pinned URL, redirect, credential, archive CRC/path/schema/expansion,
retry, cache, provider-response and cancellation boundaries remain in force.
Recovery does not restore credentials or start requests on Open. The checkpoint
and queue retain their existing JSON/archive/source byte limits. Large streaming
jobs without retained raw archives are outside this recovery contract.

## Native relationship and evidence

Desktop [Fetch dispatch](../../crates/bisect-cli/src/main.rs) validates the vintage,
expands all years, builds native item lists and selects check-only or download.
[Its downloader](../../crates/bisect-cli/src/fetch.rs) also has filesystem skip,
force, release and delay behavior. This browser recovery adapts retained selected
sources and explicit requests; it does not claim full equivalence to those CLI
filesystem/process branches. Existing Fetch/FLETCH paths remain separate.

`test_acquisition_recovery.mjs` regenerates synthetic RI/VT ZIPs from the tracked
native TIGER fixture builder and tracked PL94 fixture text. It tests partial and
cancelled acquisition, same-session Resume, imported-proof refusal, no-network
saved verification, archive/hash/derived-recipe tampering, changed settings and
forged verification markers. Only the unfinished pair is fetched on Resume, and
the prior job remains exact. A subsequent explicit two-unit, population-18 Run
retains the maximum-u64 seed and matches six native adapter replies, keeping both
raw replies and allowing only named pointer-width presentation differences.
`test_acquisition_recovery_configured.mjs` separately checks exact YAML/form/seed
reconstruction and recovery. These synthetic fixtures establish no real-state
capacity or geographic validity claim.

`test_acquisition_recovery_browser.mjs` runs against the staged package in both
native-State and configured modes. Actual archive/native workers check saved
sources without provider requests or partitions. Controlled provider responses
then supply only the unfinished VT pair. Cancelling a resumed request preserves
the prior RI job; a later explicit Resume completes it. Save/Open retires live
authority, changed settings refuse, and the panel fits a 390px viewport. These
are controlled bodies at pinned URLs, not live provider availability evidence.

The existing paired-source tests also pass: 50 synthetic state acquisitions and
subsequent native-host comparisons, plus the browser's queueing, 404/cancel,
streaming and inert source-project workflows. The new package passes the existing
standalone reset, selected-State target-action, bulk workspace, input workspace,
single/batch installation recovery and optional C-worker browser regressions.

## Local package and remaining scope

The local package is `dist/wasm-acquisition-recovery-validated`. It has 641 exact
asset identities, 541 shell assets, 50 state/year packs, 266 toolkit files and
1,073 dependency edges. Offline version:
`190b4cfba6c5de04bb4799e5e9da53bdac1b6c688a35b93f6f9d63b71490256a`.
The reused WASM is 25,118,842 bytes, SHA-256
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`.

```powershell
./scripts/web/build_workspace_preview.ps1 -Base dist/wasm-bulk-workspace -Output dist/wasm-acquisition-recovery-validated -BrowserChecks
python scripts/web/snapshot_state_target_run.py --package wasm-acquisition-recovery-validated
```

The stable 113-file local snapshot is
`target/acquisition-recovery/package-snapshots/wasm-acquisition-recovery-validated/snapshot.json`.
It retains source, owned fixture/native reference executables, original synthetic
ZIPs, both native comparison replies, checkpoints/projects, browser evidence and
package/runtime identities. Nothing was deployed. This is not an acceptance-v1
bundle or a reproducible public release.

WP-05 still needs complete Fetch/FLETCH source-type/consumer reconciliation,
live-provider evidence, other vintages/profiles and larger-data recovery coverage,
and full workspace/custody integration. WP-06 still needs composed manifest,
sidecar, original-source, producer-identity and explicit engine replay checks.
Saved bulk-workspace replay must not repeat reset. See the
[full parity plan](wasm-desktop-parity-plan.md).
