---
skill: roles-check
topic: wasm-label-rename-application
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CONTOUR, TRENCH]
p1_count: 1
p1_open_count: 0
p2_count: 3
p3_count: 19
verdict: APPROVED-WITH-CONDITIONS
---

# Reviewed label rename: scoped implementation review

Artifact: [rename application contract](../../../docs/specs/wasm-label-rename-application.md), native orchestration, browser adapter/UI, native/source/final-package tests and verified archive. Standard depth. Seven installed role perspectives are applied by the coding agent, not independent signatories. This review accepts the named local package scope with conditions; deployment and full parity remain unaccepted.

SURVEY covers practical review/recovery; BENCHMARK compares actual outcomes and test boundaries; COVENANT examines byte identities and retained failures; LEDGER examines native formats and ordering; VAULT examines private root authority and hostile inputs; CONTOUR examines source/data preservation; TRENCH checks structural failure prevention and regressions. No algorithm or legal-standard change is reviewed.

## SURVEY

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| S1 | Source UI evidence alone does not accept a current staged package. | P2 | Delivery | Closed for staged bytes: all 25 package gates pass in `dist/wasm-label-rename-validated`; final support registration follows archive verification. |
| S2 | Browser copying/removal can leave partial changes and differs from native OS rename. | P3 | Application | Explicit review lists force removal and source deletion; receipts and continuation disclose the boundary. |
| S3 | Empty directories are absent from upload file lists. | P3 | Intake | Readonly live intake captures them; partial uploaded subsets refuse live review against complete stage trees. Source checks pass. |
| S4 | A truthful prior completion receipt remained visible after project Open without a historical label. | P3 | Receipt status | Actual Chrome regression fails before the fix and passes afterward. Context-bound historical wording now states that the current project is not verified/installed; the current staged-browser history assertion passes. |

## BENCHMARK

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| B1 | New registrations need actual execution against both current profiles. | P2 | Builders | Closed for scope: both current profiles pass registered gates over verified owned fresh engine bytes. Original failures, build records, source pins and explicit reuse mode are retained. |
| B2 | A Windows case-alias fixture initially addressed an already-existing differently cased path and failed to exercise refusal. | P3 | Test fixture | Corrected fixture uses an uppercase-only directory requested in lowercase; unchanged refusal assertion passes. Retain failure record. |
| B3 | Cancellation may happen after close and before receipt completion. | P3 | Copy/recovery | Actual Chrome pauses after close; cancellation retains source, records attempted effect and explicit continuation re-observes exact bytes. |

## COVENANT

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| C1 | Runtime/package evidence requires a verified immutable snapshot. | P2 | Evidence closure | Closed for scope: verified snapshot `ae32e13bbf9cca1248d29d4426056bcb29c7d58761500be44728c67f1fb95428` retains 9,790 mappings including original failures, witnesses and fixture producers; inherited base dependency stays explicit. |
| C2 | A browser comparison failed copying its native executable because of ENOSPC. | P3 | Evidence | Failure retained, sequential rerun passed; terminal executable/debug symbol compression verified unchanged hashes. No deletion or pass retagging. |
| C3 | Reused default source result is not a new execution. | P3 | Revalidation | Record prior digest and unchanged test/runtime/source pins; later default browser and C filesystem/browser checks are fresh executions. |

## LEDGER

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| L1 | Native registry HashMap serialization order is unspecified. | P3 | Comparison | Predeclared structural registry equality only; every other file and directory roster compares exactly. Original registry stays exact until commit. |
| L2 | Force replaces a destination only for stages with a source. | P3 | Stage orchestration | Actual CLI cases cover retained destination-only stages and forced replacement. Browser iterates runs/analysis/reports in order. |
| L3 | Malformed top index makes native fail after a move, while shared preparation refuses before effects. | P3 | Failure semantics | Explicit adaptation, not full native partial-effect equivalence; keep that broader parent scope open. Year indexes/config references/hashes are preserved. |

## VAULT

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| V1 | Serializable review objects must not authorize writing. | P3 | Authority | Private WeakMap grant binds the live root and is single-use; cloned/consumed/retired grants refuse in tests. Open is inert. |
| V2 | Extra files, aliases or concurrent edits could invalidate destructive review. | P3 | Freshness | Complete selected subtree hashes/rosters and registry rechecked before effects; unknown partial bytes and aliases refuse. No OS lock or race-free concurrent-writer claim. |
| V3 | Corrupt destination writes must not destroy the only source copy. | P3 | Copy/remove | Destination read-back precedes source removal; injected corruption, permission loss and remove failure leave truthful receipts. Original bytes survive corrupt-copy test. |
| V4 | Native Windows force rename with ASCII case-only labels removed its own source before failing. | P1 | Shared native safety | Owned destructive baseline retained. Early shared guard, failing-before regression, five kernel tests, 31 serial native tests and four rebuilt CLI cases now preserve all source/registry bytes; both fresh engines and final-package checks pass; broader host/transaction guarantees remain excluded. WASM already refused case-only labels. |

## CONTOUR

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| D1 | Renaming labels does not rewrite geographic data or cross-index provenance. | P3 | Data preservation | Preserve raw binary bytes, year indexes, configs and recorded hashes; prompt separate SHA-chain verification. |
| D2 | Supplied registry stages are declarations, not evidence of valid analysis or certification. | P3 | Result interpretation | No analysis, partition, legal or official election claim arises from rename success. |
| D3 | Bounded synthetic rename tests do not establish large/national workflow capacity. | P3 | WP-08 | Keep input limits and host/API/capacity matrix explicit; real data release obligations remain open. |

## TRENCH

| ID | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|
| T1 | An exact-name guard did not prevent filesystem aliases before forced removal. | P3 | PP-38 | Shared case-only guard and failing-before/native-process regressions solve the tested ASCII alias class; symlinks, locks and other namespace/platform cases are excluded. |
| T2 | Passive in-session receipts can become misleading across project contexts even when authority stays private. | P3 | PP-37 | Current rename context labeling has a failing-before/actual-packaged-browser regression. PP-37 remains mitigated globally because other writer panels need their own review. |
| T3 | A stale source/profile/log reference can turn cached runtimes into misleading test evidence. | P3 | Reuse validator | Validate before owned copy/receipt replacement. Seven actual-record negative cases pass; final package separately exercises loaded C execution. No imported project receives build or filesystem authority. |

## Synthesis and amendments

Roles: 7. P1: 1 historical native safety finding, fixed and verified in the rebuilt CLI and both fresh engines; open P1 blockers: 0. P2: 3 closure findings, all closed for the named local scope. P3: 19 addressed or explicitly scoped conditions. Verdict: **APPROVED-WITH-CONDITIONS** for this scoped local delivery. Whole-parent and formal release acceptance remain open.

Top finding: native Windows case-only force rename could delete its own source. The shared early guard and actual CLI preservation tests close that class. SURVEY, BENCHMARK, COVENANT and TRENCH agree that private authority, native outcomes, final staged bytes and archived evidence must remain separately identified.

1. Added shared case-only rejection before native effects, failing-before regressions, 31 serial native tests and four actual CLI preservation cases.
2. Added current-context historical receipt labeling, actual staged-browser regression and all 25 package gates over verified fresh default/C engines.
3. Verified the inherited immutable snapshot and retained all failures; scoped support/ledger registration follows the freeze without broad-parent or acceptance-v1 promotion.

Current evidence: eleven actual CLI cases and forty-one interruption boundaries per fresh actual engine, actual source Chrome/OPFS with inert Open and API absence; project, label repository, native summary, transcript and project-handoff regressions pass. Ordinary ledger/source checks and 8+4 mutation tests pass. All 25 final-package checks and immutable archive verification pass. Broader host/API/capacity, whole-parent and formal release acceptance remain open.


## Accepted local artifact

Package: `dist/wasm-label-rename-validated`.
Package record SHA-256:
`3498102faf7c98af0ec06175e31e8c36548b79e29bee91665056e5f3d9a91f63`.
Snapshot SHA-256:
`ae32e13bbf9cca1248d29d4426056bcb29c7d58761500be44728c67f1fb95428`.
Offline version:
`e0c10299943f170ddfac0cf3178d952f0c7080091f6b68b6c90adfd39d2e6828`.
The [contract closure](../../../docs/specs/wasm-label-rename-application.md#scoped-evidence-closure)
identifies full paths, immutable inheritance and exact current identities.
Original source-tree/owned OPFS tests and injected pickers are not OS permission,
other-browser, all-state capacity, legal or certification proof. Receipts grant
no future root/write authority; Open is inert. No deployment is accepted.
