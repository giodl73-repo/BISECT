---
skill: roles-check
topic: wasm-workspace-continuation
date: 2026-10-23
roles_used: [MERIDIAN, BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH, SURVEY]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Saved native-workspace continuation: implementation review

Artifact: [workflow contract](../../../docs/specs/wasm-workspace-continuation.md).
Type: browser workflow, scheduler and evidence-boundary implementation. The
coding agent applied seven repository lenses; these are not independent reviewers,
maintainer sign-off, legal advice or full-parity/release acceptance.

Selection: MERIDIAN covers native computation/backend semantics; BENCHMARK covers
actual execution and fixture adequacy; COVENANT covers custody and historical
claims; LEDGER covers versioned records; VAULT covers imports/live authority;
TRENCH covers repeated effects and cancellation; SURVEY covers operator actions.
Other role domains receive no new legal, political, certification or ballot-data
claim from this work.

## Findings

P1 blocks shipment; P2 must be fixed for this bounded milestone; P3 should be
fixed or carried as an explicit broader limitation. A closed scoped guard does
not close its entire work item or establish authenticity.

### MERIDIAN

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| M1 | Retained results must match original-source computation, not a rehashed success marker. | P3 | Verification worker | Completed rows are recomputed and compared, then the composed inspector binds source/graph/manifest/plan/context. Altered results and inputs refuse. Preserve the exact-row tests. |
| M2 | One-district fixtures alone would miss real partition-path regressions. | P3 | Matrix | Added RI two-district module and actual browser cases on both engine variants. Real-state, larger graph, method and capacity rows remain open. |
| M3 | Runtime availability must reach batch workers without substituting C for native bulk's metis-core choice. | P2 | Runtime plumbing | Closed scoped guard: bulk defaults remain unchanged; explicitly C-backed prepared batches load only on explicit execution, match the native C reference, and reject mixed loaded identities before job dispatch. |

### BENCHMARK

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| B1 | Source-tree success cannot accept a different final package or engine. | P2 | Package gates | Closed for this local package: actual default/C-enabled gates, both default source-tree browser cases, final packaged one/two-district browsers, C batch/native checks, asset closure and snapshot identities agree. |
| B2 | An old generated checkpoint's engine identity must not be refreshed by editing its hash. | P3 | Legacy recovery | Regenerated the old batch/source fixtures through their setup suites after a real identity refusal; legacy saved recovery, session Resume, multi-year Resume and project handlers pass. |
| B3 | A passing small fixture is not the advertised real-data matrix. | P3 | WP-08 | Open. New source fixtures regenerate from tracked code; broader real sources, vintages/units, browsers/OS pickers and capacity still require predeclared runs. |

### COVENANT

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| C1 | Old target actions cannot authenticate a historical deletion or authorize a new one. | P2 | Previous actions | Closed scoped guard: `previous_actions` remain unauthenticated records, current root observations and explicit confirmation supply authority, and reappearing data blocks that target for separate review. |
| C2 | Matching WASM alone is insufficient for explicitly C-backed worker pools. | P3 | Runtime records | Workers report the actual loaded runtime descriptors. One profile is bound within the batch and additional slots preload/compare before dispatch; actual C results retain runtime hashes. Cross-platform/method/large-workflow coverage remains open. |
| C3 | Local snapshots are not formal release custody or source-publication authenticity. | P3 | Evidence scope | A new 180-file immutable local snapshot retains raw cases, native C oracle, both engines and browser outcomes. Acceptance-v1 migration, complete build custody and third-party notice/release work stay open. |

### LEDGER

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| L1 | Replaying filesystem receipts would change accepted rows or repeat effects. | P3 | Comparison contract | Omit only `native_run.target_actions` / `native_states.target_actions` from computational comparison; retain the original accepted row including those fields byte-for-byte. No seed/label normalization. |
| L2 | Workspace continuation must preserve old compute-only Resume and multi-year record meaning. | P3 | Scheduler | Existing paths stay distinct. The new session supplies verified rows/selection privately; year order remains 2020/2010/2000. Old session and saved-recovery suites pass. |
| L3 | Native completion presence does not prove current installed contents. | P3 | Live output bytes | A currently complete retained target must match the generated package files. Other layouts, arbitrary native CLI package rebasing and automatic installation/recovery remain separate scopes. |

### VAULT

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| V1 | Imported handles, action records or verification markers must not restore a grant. | P3 | Authority | A private single-use review/session binds current source/policy/engine and selected root; copied display objects fail. Actual browser Open starts no request, picker or deletion and clears runtime selection. |
| V2 | A changed target/source or an old picker callback can invalidate the visible preview. | P3 | Freshness | Re-observe before scheduling/actions and after each year. Source/runtime choice retires reviews; a cancelled late picker cannot retire a newer one. Node/browser refusal cases pass. |
| V3 | Worker replay should have no filesystem capability or implicit acquisition. | P3 | Worker boundary | Verification receives bounded JSON/source bytes, not the root handle. The main thread reviews the root afterward. No reset/install/acquisition wrapper is called during replay; actual OPFS writes remain zero before confirmation. |

### TRENCH

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| T1 | Computational recovery can repeat a destructive outer preparation. | P2 | PP-32 | Closed scoped guard: accepted rows are skipped, touched data-absent targets get reset=false, and reappearing data blocks. Actual cancellation after removal followed by reopened continuation makes no second removal. PP-32 remains MITIGATED globally. |
| T2 | Cancellation cannot be treated as rollback after a filesystem call starts. | P3 | Interrupted action | Keep the interrupted receipt and completed result. Browser tests cancel after genuine OPFS removal and continue explicitly; no pending source job is dispatched before that cancellation. |
| T3 | A failed target must not erase earlier successes or stop independent eligible work. | P3 | Batch transitions | Failed/blocked rows remain visible while other jobs proceed. Three-year continuation dispatches five unfinished jobs while retaining the first row exactly; source failure and action ordering regressions remain passing. |

### SURVEY

| ID | Finding | Severity | Section | Disposition |
|---|---|---|---|---|
| S1 | Review that recomputes results must be labelled differently from read-only filesystem observation. | P3 | Controls | Explicit button says replay saved results; progress/cancel remain available. Confirmation and Continue are separate, and fresh Run cannot consume a continuation review accidentally. |
| S2 | A three-minute guard deadline is inappropriate for potentially long completed-result replay. | P3 | Worker lifecycle | Replay is explicitly cancellable with progress; ordinary guard deadlines remain. Browser controls and 390px layout are tested; broader accessibility/host/browser coverage remains open. |
| S3 | The operator needs clear distinction between accepted computation and installed output. | P3 | Summary / download | Retained-result status and installed-byte checks remain separate; installation is still explicit. Action records expose prior/current actions and no rollback. Richer incomplete-target diagnostics remain a broader UI improvement. |

## Synthesis

**7 roles, 21 findings: 0 P1, 4 P2, 17 P3.** The four scoped P2 guards pass.
Verdict: **APPROVED-WITH-CONDITIONS for the local preview**. This does not close
the complete workspace contract, C method/runtime coverage, real-data matrix,
formal acceptance bundles, licensing/notices or release/deployment sign-off.

Top finding: a computational replay must never recreate deletion authority.
COVENANT/VAULT/TRENCH agree on the separate live-review boundary.
MERIDIAN/BENCHMARK agree that multi-district computation and actual runtime
selection need real execution evidence, not declarations or checkbox presence.

Three amendments implemented:

1. Added original-source replay plus composed inspection, with byte-exact retained
   rows and private current-root continuation authority.
2. Added per-target no-repeat disposition, reappearing-data refusal, stale/late
   review retirement and retained interrupted outcomes.
3. Propagated explicit C availability to workers, compared loaded identities before
   dispatch, expanded RI/native-C cases, and bound final package evidence.

## Evidence and limits

HEAD `383db340235e21d752a35610aab28e2948b73e57`; dirty working tree.
Package: `dist/wasm-workspace-continuation-validated`.
Offline version: `c1ebed8a632c42f1f0a8d650414c5651fe497f882aadf7fb313862b0d25062e7`.
Default WASM `60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`;
packaged C-enabled WASM `56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`.

The [180-file snapshot](../../../target/native-workspace-continuation/package-snapshots/wasm-workspace-continuation-validated/snapshot.json)
retains new source, raw verification/cases/events, both actual engines and their
gate records, default browser outcomes, final package/browser outcomes and the
owned native C reference/input/reply. It is not acceptance-v1 or signed provenance.

Both actual builds pass the seven legacy configuration/State suites, composed
Verify and new continuation module cases. One- and two-district browsers pass
against the default artifact and final C-enabled package. Final checks also cover
legacy workspace/reset/target/bulk flows, paired acquisition/recovery/streaming,
TUI/selected-output Verify, single/batch output recovery and optional C execution.
The explicit C batch uses two real workers and matches an independently executed
native C assignment; its synthetic mixed-identity case fails before job dispatch.
The existing six actual CLI bulk cases and old compute-only/saved Resume tests
pass. Project-handler, ledger and source-consumption checks are retained.

Packaging closure covers 645 identities, 545 shell assets, 50 state/year packs,
268 toolkit files and 1,098 local dependencies. Counts are not execution coverage.
The JS-only workspace preview reuses a verified C base and owned native oracle;
the optional C builder regenerates the native C setup before its new batch checks.
Large retained external data, SDK/toolchain and full clean-checkout release custody
keep their declared prerequisites and integration tiers.

No parent is promoted to verified. Full parity remains blocked by unresolved
required evidence, starting with the Run parent. No commit, push, public deployment
or legal/election certification was performed.
