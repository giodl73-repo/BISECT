---
skill: roles-check
topic: wasm-native-state-data-reset
date: 2026-10-23
depth: standard
roles_used: [meridian, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 18
p3_open_count: 18
verdict: APPROVED-WITH-CONDITIONS
---

# Native data reset: scoped implementation review

Artifact: [data-reset contract](../../../docs/specs/wasm-native-state-data-reset.md),
its linked code/tests and `dist/wasm-native-data-reset`. One agent applied seven
installed role lenses, not independent reviewers or human signatories. This
review does not approve full native flag execution, desktop parity or release.

## Selection

Selection follows [.roles/ROLE.md](../../../.roles/ROLE.md); the selected profiles
and verify questions were read during this review campaign.

| Role | Reason |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Preserve actual flag effects and leave engine/seed/partition behavior untouched. |
| [BENCHMARK](../../../.roles/benchmark.md) | Actual CLI filesystem references and error-order witnesses. |
| [COVENANT](../../../.roles/covenant.md) | Raw outcomes, source/binary identities and action-receipt limits. |
| [LEDGER](../../../.roles/ledger.md) | Distinct native/browser action meanings and preserved outputs. |
| [VAULT](../../../.roles/vault.md) | Destructive scope, live selected-root authority and inert Open. |
| [TRENCH](../../../.roles/trench.md) | Stale reviews, cancellation after unabortable effects and truthful closure. |
| [SURVEY](../../../.roles/survey.md) | Concrete review, clear disposition and practitioner recovery. |

No demographic, statistical, jurisdictional, political, community, certification,
voting-data or publication contract changes. Other installed roles retain their
existing plan conditions and require targeted review if those claims change.
Role prose is not current legal authority; no legal assessment was performed.

## Findings

Closed-scoped denotes implemented prevention plus its regression for this action.
P3 denotes broader acceptance work, including carried-forward conditions.

| ID | Lens | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | Native reset deletes only data/, and native force alone permits a labeled collision; neither means whole-target replacement. | P2 | Native effect contract | Closed-scoped: distinct action/control, exact CLI file-tree comparisons and preservation of non-data files. Existing whole-target installation overwrite keeps its separate meaning. |
| M2 | MERIDIAN | Filesystem effect support does not make --reset/--force engine options or prove complete pre-Run orchestration. | P3 | Flag connection | Open: engine-only guards remain; require explicit runtime action connection before removing them. No flag is silently stripped. |
| M3 | MERIDIAN | Bulk reset still follows completion filtering and the separate reprocess choice. | P3 | Run/States | Open: preserve native scheduling order; this action is single-target, not national/bulk equivalence. |
| B1 | BENCHMARK | All 32 CLI witnesses deliberately fail before partition because their graph is missing. | P3 | Reference scope | Open successful whole-Run coverage: count them as matched filesystem/preflight outcomes, not successful partitions; retain all original stdout/stderr/arguments/trees. |
| B2 | BENCHMARK | Actual CLI recreates empty data/ before graph-load failure; the standalone browser action creates nothing. | P3 | Comparison | Open broader orchestration: name this difference, compare file bytes exactly and do not claim whole-directory/Run equality. |
| B3 | BENCHMARK | Small synthetic WA targets do not demonstrate all target sizes, platforms or filesystem failures. | P3 | Matrix | Open: document observation bounds and add predeclared larger/OS/browser cases before extending support. |
| C1 | COVENANT | The module returns a reset receipt, but the UI does not yet export/save it as project custody. | P3 | WP-06 | Open: add explicit receipt export and composed replay/custody; imported receipts must remain non-authoritative. |
| C2 | COVENANT | The immutable local source/raw-outcome snapshot is not an acceptance-v1 release bundle. | P3 | Evidence | Open: retain native executable, source and package identities; complete the formal bundle groups and release gates separately. |
| C3 | COVENANT | Preserving an old manifest after data removal cannot establish a newly completed or authentic Run. | P3 | Output status | Open ongoing boundary: UI names this fact; preserve missing-completion semantics and explicit subsequent Run/installation. |
| L1 | LEDGER | Whole-target overwrite, data reset and interruption recovery are different contracts. | P3 | Action meanings | Open broader interoperability: keep distinct schema/control names and old installer/recovery meanings; no blanket native-force equivalence. |
| L2 | LEDGER | Preserving RPLAN/RCTX/map/report bytes does not test every external consumer after data is removed. | P3 | Interchange | Open: external format/version and downstream missing-data behavior require their own fixtures. |
| L3 | LEDGER | Named/absolute path intentions do not authorize a browser directory. | P3 | Workspace | Open broader path workflows: resolve only within freshly selected root/target; saved flags remain intent and batch/custom-name paths remain separate work. |
| V1 | VAULT | Copied reviews, changed roots/packages or a saved force setting must not authorize deletion. | P2 | Authority | Closed-scoped: private one-use WeakMap, exact root/package/snapshot/force binding, generation checks and Open clearing; module/browser adversarial cases pass. |
| V2 | VAULT | Read/hash bounds and browser handles differ from unrestricted desktop filesystem traversal. | P3 | Supported inputs | Open: targets beyond 1,000 entries/128 MiB/portable paths refuse before removal; symbolic/special and all OS permission cases are not implied. |
| V3 | VAULT | No browser observation can guarantee exclusion of a concurrent external writer. | P3 | Concurrency | Open: recheck before removal and verify preserved entries afterward; disclose non-atomic scope and keep concurrent-change outcomes unverified. |
| T1 | TRENCH | Cancellation after an OS removal begins can otherwise be mislabeled harmless or complete. | P2 | Failure status | Closed-scoped: attempted/interrupted receipt and UI outcome; actual OPFS test pauses after removal, cancels and requires fresh review while retaining downloads. |
| T2 | TRENCH | Guards must precede data removal even when force is selected or temporary data files exist. | P3 | Native ordering | Open broader cases: native incomplete then labeled-collision order is shared/tested; do not bypass manifest.tmp through reset or installation overwrite. |
| T3 | TRENCH | A new workspace button can be mistaken for closing the parsed flag or parent workflow. | P3 | PP-25 | Open: keep required pre-Run flag connection not-run/missing and parents partial; full-parity gate remains closed. |
| S1 | SURVEY | The native warning names the whole root although the code clears only data/. | P3 | Preview clarity | Open shared warning correction: browser displays the true data-only scope; preserve the observed native warning in reference logs rather than copying it as a removal policy. |
| S2 | SURVEY | A fresh accepted-output package is currently required to choose the reset target. | P3 | Before-Run journey | Open: add a source/command-bound target preflight and action handoff for native flag execution; avoid authorizing deletion from imported path strings. |
| S3 | SURVEY | Reset/force exposure remains advanced and has not undergone practitioner usability testing. | P3 | UI/release | Open: controls are separate, previewed and 390px-tested; broader first-use and release acceptance remain. |

## Synthesis and amendments

Seven lenses, **21 findings: 0 P1, 3 P2 closed for this scope, 18 P3 open**.
Verdict: **APPROVED-WITH-CONDITIONS for scoped local implementation**.

Top finding: command names are not filesystem disposition policies. MERIDIAN,
VAULT and SURVEY agree that exact native effects and explicitly reviewed scope
must govern removal. BENCHMARK/COVENANT require the 32 failing CLI witnesses to
remain distinguishable from successful runs. TRENCH requires truthful outcomes
after irreversible effects have begun.

Three amendments implemented and verified:

1. Separated native data-only reset/force preflight from existing whole-target
   installation overwrite, preserving actual guard order and outside-data bytes.
2. Added private one-use root/package/force/snapshot review authority and Open/
   source-generation retirement, while preserving engine-only flag refusals.
3. Added interrupted/unverified outcome handling after attempted removal, with
   actual CLI filesystem and real OPFS cancellation/stale-review regressions.

## Evidence

- 32 actual CLI filesystem comparisons; 25 native/module refusals; no reset
  partitions. Raw argument/stdout/stderr/file trees and an owned native executable
  copy are retained. Empty-directory lifecycle differences are explicit.
- Actual packaged browser worker/OPFS reset tests pass, including no writes on
  review/Open, only data/ removal, stale refusal, native force/incomplete ordering,
  post-removal cancellation, retained downloads and cleared authority on Open.
- Existing recovery (11 scenarios/31 native guards), workspace (three source
  modes/26 host replies/independent graph/18 ZIP refusals) and final packaged
  workspace/C regressions pass.
- `dist/wasm-native-data-reset` passes 1,035 dependency references and 629 offline
  asset hashes/sizes. Version:
  `1cb21f6fd7bd8d54c30fb0cb3cb71737ccf8ce6614bce4d788ea2d25e2ec6113`.
- Immutable local snapshot:
  `target/native-state-data-reset/package-snapshots/wasm-native-data-reset/`.
  It is not formal acceptance-v1 evidence. Full plan/parent gates remain open.

| Reviewed source | SHA-256 |
|---|---|
| `web/lab/native-state-install.js` | `f59217f6b0b3f6150e3eb31c8b6e77fdc48ad6624cf6f0a36b70f34ab7f37136` |
| `web/lab/native-state-output-ui.js` | `6e3a303467446b784d587e0cc5cdac373316c9711c798ff6e8d942857b837671` |
| `web/lab/workbench.html` | `8ea57289cf59b7d1fd635c9a5e19dc85f8f0aee6603d14d70e6396391c4ddd30` |
| `scripts/web/test_native_state_data_reset.mjs` | `d0045f888f5a4eb0260d7ae0c2d5073de8588994b710d72a1f8dd660c4ce627c` |
| `scripts/web/test_native_state_data_reset_browser.mjs` | `33991044d954b4c2b886bcce107cb8c1634ff613d0e935dafcbe6358af5baecc` |

HEAD: `383db340235e21d752a35610aab28e2948b73e57`, dirty worktree. Hashes identify
reviewed bytes, not authenticity or independent reproduction. Nothing was
committed, published or deployed.

[PP-28](../../../design/pitfalls/pitfalls-pipeline.md#pp-28-flag-names-mistaken-for-filesystem-disposition-scope)
records the distinct disposition-scope failure, MITIGATED with the actual CLI and
OPFS regressions. Existing PP-25 still governs false full-parity closure.
