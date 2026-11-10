# Inspect and replay a selected native output

Choose **Verify native output and original sources** in the workbench. Select one
native output folder with the package controls, select its manifest and files,
or import its ZIP. This operates on selected bytes and needs no server or output
directory permission. It does not acquire sources, reset data or install files.

The scoped local package is `dist/wasm-native-output-verification-validated`.
Both actual engine variants and its final worker/browser checks passed. This
does not close full WP-06, formal release acceptance or deployment.

## Two explicit stages

**Inspect** checks manifest sidecars, original assignments, plan/context/run
correspondence and, when retained, regenerates the original source graph without
partitioning. **Replay** additionally runs the retained portable engine request
and compares its assignments. The default threshold is exact agreement; an
explicit different threshold is recorded. District labels are not permuted.

The retained browser accepted Run is used when available. An optional portable
replay-artifact override supplies the exact replay request for other selected
packages. Selecting it does not execute it. The workflow has one threshold
control; older standalone replay controls remain in their own operations.

The summary exposes the recorded exact seed, original-source stage,
manifest/run binding, partition completion, comparison outcome and recorded
historical producer claim. Download the current verification record explicitly.
Save/Open retains sources and results, but imported results remain unverified;
rechecking requires an explicit action. Changed inputs retire fresh downloads.

## Byte, graph and assignment contracts

The bounded ZIP reader retains its traversal/alias/duplicate/link/CRC and
compressed/expanded limits. The output layout accepts exactly one target
manifest; all members must belong to that target. Existing native-input ZIP
layout behavior remains unchanged. Package file count/byte limits also apply;
ZIP support does not imply arbitrary-size real-state output support.

`data/final_assignments.json` may contain native zero-based unit indices with
one-based district labels. A retained unit-index sidecar or actual replay graph
supplies the unit order. A by-unit companion must agree exactly. Numeric custom
unit IDs overlapping the index universe with a different order are ambiguous
and refused; the verifier must not guess. Native manifest binding checks the
exact district range before replay.

Original manifest bytes go directly to the native typed parser, preserving
integers and seeds outside JavaScript's exact numeric range. Source regeneration
checks its recorded recipe and graph digests, then the Rust verifier compares
the typed graph to the actual selected replay graph. Rehashing a modified source
receipt cannot replace that graph. Raw-byte and canonical graph hashes remain
separately named. Optional graph binding and inspection fields default off,
preserving the existing standalone manifest-replay behavior.

Every verification stage retains its own result. Missing source or producer
evidence stays missing even if assignments replay identically. A loaded WASM
identity identifies the current verifier; a saved producer identity remains a
historical claim. Neither authenticates a publisher or historical execution,
establishes legal validity, certifies an election or authorizes workspace Resume.
Installed-byte checking and fresh live filesystem authority are separate paths.

## Current evidence and remaining gates

- [Node/native dry-run checks](../../scripts/web/test_native_output_verification.mjs)
  regenerate a four-unit RI fixture, test exact maximum-u64 seed retention,
  native graph binding, changed/rehashed sources, missing sidecars/context/source,
  ambiguous indices, explicit replay and a differing threshold outcome. Two
  actual native CLI `verify --dry-run` cases prove filesystem prechecks only;
  they do not prove full CLI replay orchestration.
- [Real-browser checks](../../scripts/web/test_native_output_verification_browser.mjs)
  exercise the worker, folder/ZIP selection, inspection, replay, record download,
  saved-result Open, missing/failed stages, CRC refusal, late ZIP retirement and
  a 390px viewport. The test uses its own browser/server and upload directory.
- Existing manifest replay, TUI Verify, project handlers and selected-workspace
  Node/browser regressions are retained. The project VM fixture imports current
  validators and scopes its minimal-DOM adapters to import handlers; it is not
  evidence for the practitioner/target-action installers.

The dual-build runner adds 16 composed Verify cases, four exact native-host
inspection replies and two actual CLI dry-run cases to the seven existing legacy
gates. Both actual default and C-enabled builds passed. Standard laboratory/toolkit
and workspace-preview builders register the new Node/browser checks.

The final local package passed the new worker/browser test, legacy TUI Verify,
selected workspace, reset/target/bulk actions, acquisition recovery/streaming,
single/batch output recovery and optional loaded-C browser checks. Its C engine
also passed 52 native assignment matches and four matching failures. Asset/offline
checks cover 643 identities, 543 shell assets, 50 state/year packs and 1,084 local
module/worker/HTML dependencies; those asset counts are not execution coverage.

- Default WASM: `60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`, 25,116,980 bytes.
- Packaged C-enabled WASM: `56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`, 25,123,741 bytes. The new Verify cases use `metis-core`; loaded-C execution is a separate regression, not evidence for every C Verify journey.
- Offline version: `66e71bbb14b03acb7b0efe54adc151f6d67170e246955e3f0b0d196d515d0f08`.
- [145-file immutable local snapshot](../../target/native-output-verification/package-snapshots/wasm-native-output-verification-validated/snapshot.json), including original requests/replies, CLI references, both engine variants and final-browser evidence.
- [Seven-role implementation review](../../signals/roles/check/wasm-native-output-verification-roles-check-2026-10-23.md).

The project-handler regression also passes with current sources. The ordinary
ledger check passes; the full-parity gate still rejects the unverified Run parent.
The broader plan still requires other Verify branches, real-data/vintage/unit
coverage, producer custody, saved-workspace continuation and formal release
evidence. [PP-31](../../design/pitfalls/pitfalls-pipeline.md#pp-31-aggregate-verification-success-conceals-missing-stages)
is mitigated for this selected-output workflow, not solved for every consumer.
