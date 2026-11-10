# Continue a saved native workspace

In **Run state batch**, open the original project and choose **Replay saved
results and review continuation folder**. Choose the current outputs root.
Completed results are explicitly recomputed from their original source jobs,
then passed through the composed source/graph/manifest inspector. This step does
not reset, install or write files. It can be cancelled independently.

Review the per-target list, check the confirmation and choose **Continue verified
workspace**. Completed rows remain byte-for-byte unchanged. Only unfinished or
failed states are dispatched. Native Run's 2020/2010/2000 order and States' single
year remain distinct; the same live review covers the selected years.

The scoped local package `dist/wasm-workspace-continuation-validated` passes
both-build and final-package checks. This is not full desktop workspace parity,
formal release acceptance or a hosted deployment.

## Current target handling

- A completed saved result is replay-verified, retained and never reset by
  continuation. If the live completion predicate reports it installed, all files
  in its generated native-layout package must match current bytes. A mere marker
  is insufficient. This does not adopt arbitrary third-party layouts.
- A previously skipped native completion must still be skipped under the fresh
  predicate. Changed skip decisions require a fresh Run review.
- An unfinished target with no previous action may use a new, explicitly reviewed
  data-only reset. That removal is shown in the current preview.
- If a previous target action was recorded and `data/` is absent, the new target
  action has reset disabled. Its receipt says `current-data-absent-no-repeat`.
  Recorded original Run flags remain unchanged.
- If data is present after a recorded action, that target is blocked for a
  separate explicit reset/recovery review. Other eligible states can continue.
  Continuation does not guess that repeating deletion is safe.
- A newly complete unfinished target without a verified retained result is
  blocked for separate output verification. Automatic installation is separate.

Review captures canonical and historical target observations. It rechecks every
target before scheduling, each pending action before effects, and the year after
execution. A changed source, project, operation, runtime choice, selected target
or engine retires the applicable authority. An old picker callback cannot retire
a newer review. Directory operations do not lock out other programs or undo a
removal already started.

## Evidence and authority

The saved checkpoint must match the exact original jobs, policy, raw source
identities, embedded congressional allocation, scheduled states and WASM bytes.
Completed row replay compares the complete receipt, omitting only the passive
`native_run.target_actions` or `native_states.target_actions` field. It then uses
the composed no-partition verifier to bind original sources and actual graph,
manifest, plan, context and assignments. This adds one completed-state replay,
not an additional partition during inspection.

Historical action records remain explicitly unauthenticated `previous_actions`;
they cannot prove a historical deletion or restore a folder grant. A private,
single-use review/session binds current inputs, verified result bytes and the
live selected root. Serialized verification markers grant no authority. Open is
inert, runtime selection clears on Open, and legacy compute-only Resume remains
a separate workflow. Interrupted continuation records retain both accepted
results and the actual interrupted action outcome for explicit download.

Long completed-result replay uses a cancellable worker computation with progress
instead of the shorter target-guard request deadline. Ordinary guard deadlines
remain in place. Filesystem review still uses bounded paths, entry/byte limits
and native incomplete/collision guards. Missing directory APIs give an explicit
unsupported continuation action; portable projects and downloads remain usable.

## Runtime selection

Run/States use their actual shared bulk compositor, which selects `metis-core`;
loading C does not change that selection. Multi-district RI exercises this real
bulk path. The workbench's explicit C-runtime choice now also reaches independent
batch workers for prepared/configured requests that actually select `c-ffi` and
`msvcrt-v1`. Selecting the checkbox or opening such a project loads no C assets;
loading occurs on explicit execution, including a saved-result replay. Missing C stays a visible failure, with no backend
substitution. Runtime identity records describe what actually executed.

The pool preloads additional C-enabled worker slots and compares their reported
engine, JavaScript and C-WASM identities before dispatching jobs. A mismatched
runtime interrupts the batch without accepting a state or performing a pending
target action. Runtime availability is a choice for the next execution; changing
it retires a live workspace review, while existing accepted results retain their
original recorded identities.

## Tests and remaining acceptance

[Module tests](../../scripts/web/test_native_workspace_continuation.mjs) regenerate
tracked four-unit source fixtures and cover native Run/States, all three years,
RI's two districts, exact retained rows, fresh and already-attempted target
actions, changed/rehashed results/inputs, private review objects, stale targets,
live installed bytes and worker cleanup.

[Real-browser tests](../../scripts/web/test_native_workspace_continuation_browser.mjs)
use actual workers and genuine OPFS with an injected picker over an owned root.
They exercise both one- and two-district saved results, source changes, cancelled
late pickers, interrupted real removal, inert Save/Open, unfinished-only dispatch,
no repeated reset, readable action records and a 390px viewport. This does not
establish every OS picker/permission implementation.

[Explicit C batch tests](../../scripts/web/test_explicit_c_state_batch.mjs) and
[their browser path](../../scripts/web/test_explicit_c_state_batch_browser.mjs)
compare a two-district RI C request with an owned, hash-checked native C reference,
require explicit loading and retain the actual runtime identities. They do not
claim native bulk defaults use C.

The existing six-case native CLI bulk comparison, compute-only session and saved
recovery, multi-year Resume, project handlers, source acquisition, output
installation/recovery and optional-C regressions remain required. Older generated
fixtures must be regenerated against the tested engine rather than having their
hashes updated to match. Broader real-state/vintage/unit/backend/layout coverage,
provider/source authenticity, full custody, automatic installation and formal
release acceptance remain open under the full parity plan.

## Local delivery record

The two actual engine variants pass the seven legacy configuration/State suites,
composed Verify and new continuation module cases. Default-artifact browsers and
the final C-enabled package each pass one- and two-district continuation. The
final package also passes acquisition/recovery/streaming, live workspace/reset/
target/bulk, TUI/selected-output Verify, single/batch output recovery, the explicit
two-worker C batch and existing C-worker regressions. Current project handlers
and existing compute-only/saved/multi-year Resume tests pass.

- [180-file immutable local snapshot](../../target/native-workspace-continuation/package-snapshots/wasm-workspace-continuation-validated/snapshot.json).
- [Seven-role implementation review](../../signals/roles/check/wasm-workspace-continuation-roles-check-2026-10-23.md).
- Default WASM: `60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`.
- Packaged C-enabled WASM: `56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`.
- Offline version: `c1ebed8a632c42f1f0a8d650414c5651fe497f882aadf7fb313862b0d25062e7`.

Asset closure covers 645 identities, 545 shell assets, 50 state/year packs,
268 toolkit files and 1,098 local dependencies. These are packaging counts,
not algorithm/data coverage. The JS-only workspace builder reuses a verified C
base and owned C oracle; a fresh optional-C builder regenerates the C reference
and runtime before its new batch checks. Formal custody/release prerequisites
remain open. [PP-32](../../design/pitfalls/pitfalls-pipeline.md#pp-32-computational-continuation-repeats-destructive-preparation)
is mitigated for this workflow, not globally solved.
