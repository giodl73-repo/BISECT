---
skill: roles-check
topic: wasm-desktop-parity-plan-build-boundary
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CONTOUR]
p1_count: 0
p2_count: 3
p3_count: 15
verdict: APPROVED-WITH-CONDITIONS
---

# Parity plan review: current build boundary

Artifact: [desktop-to-WASM parity plan](../../../docs/specs/wasm-desktop-parity-plan.md), especially Current execution handoff, and its [build application contract](../../../docs/specs/wasm-label-build-application.md). Type: implementation and acceptance plan. Depth: standard.

One coding agent applied six installed repository role perspectives. These are not independent reviewers or human signatories. This review changes planning documentation only. It runs source validators and inspects retained evidence; it does not rerun CLI, WASM, browser or package acceptance, certify legal compliance, approve deployment or supersede historical scoped reviews.

## Selected roles

SURVEY checks the practitioner's next usable workflow; BENCHMARK checks observable failure and acceptance tests; COVENANT checks source/package identity and evidence boundaries; LEDGER checks native orchestration and interchange semantics; VAULT checks untrusted imports and authority; CONTOUR checks the successful real-data matrix and source identity. All installed role files were inventoried. Algorithm research, political/statistical outcome and jurisdiction-specific legal review are outside this documentation amendment. Statements in role descriptions are review prompts, not independently verified legal rules.

## Findings

### SURVEY — practical workflow

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | Handoff says 91 source pins; current validator reports 98. | P2 | Current execution handoff | Corrected to 98. Counts remain inventory, not completion. |
| S2 | Core execution is useful progress, but there is no accepted live build UI or project/archive continuation journey. | P3 | Ranked task 1 | Keep build first and partial; explicitly carry UI and inert Save/Open integration. |
| S3 | Medium/large and national success remain the next practical validation tasks. | P3 | WP-08 | Retain requested full chamber counts and executable prerequisites; sampling or matching failure cannot substitute. |

### BENCHMARK — observable tests

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Existing host recovery boundaries do not cover cancellation after worker state completion but before its final reply. The core awaits the reply without engine progress consumption. | P2 | Build acceptance | Added a named actual-worker gate, including retained accepted artifacts and stale-event rejection. Runtime condition remains open. |
| B2 | Seven CLI cases and 22 recovery boundaries could be misread as actual WASM/UI acceptance. | P3 | Build checkpoint | Explicitly label native-host evidence and preserve both actual profiles/final-package gates. |
| B3 | Source-pin validity cannot prove a freshly compiled CLI formatter or output agreement. | P3 | Evidence rules | Disclose retained earlier CLI reference; require fresh native build in the next acceptance bundle. |

### COVENANT — provenance and custody

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Build contract still says predeclared and summary pending, despite prototype progress and accepted summary delivery. This obscures the source/package boundary. | P2 | Build contract / handoff | Corrected status/dependency and added a source-only checkpoint without repinning historical evidence. |
| C2 | The accepted summary package remains the latest accepted runtime; current source changes are not its contents. | P3 | Package checkpoint | State this explicitly; preserve package and inherited snapshot identities. |
| C3 | Local inherited evidence storage is not a publicly reproducible signed release. | P3 | WP-10 | Keep custody dependencies, formal release and maintainer acceptance open. |

### LEDGER — semantics and interchange

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | Native built status can coexist with failed states, including stale built status during interrupted force. | P3 | Build native authority | Preserve per-state failures and ordered invalidation; never label a registry marker algorithm success. Existing contract retained. |
| L2 | Prepared serial jobs and root artifact comparisons do not prove native discovery, parallel scheduling or all nested layouts. | P3 | Build boundary | Added explicit comparison limits; maintain selected-source adaptation classification. |
| L3 | New baseline fields need versioned validation and backward-compatible inert archive reads. | P3 | WP-03/07 | Carry project/build-archive integration as pending; imported baseline never becomes a grant. |

### VAULT — untrusted data and authority

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Saved paths, force flags and receipts cannot authorize writes after Open. | P3 | Completion contract | Preserve fresh private single-use root review and concrete scope confirmation. |
| V2 | Files can change between review and individual effects; neither browser locking nor rollback is proven. | P3 | Host/recovery closure | Retain fresh observations/readback and unknown-byte refusal; do not advertise atomic transactions. |
| V3 | OPFS or injected picker tests do not demonstrate OS-folder revocation or real permission behavior. | P3 | Host/recovery closure | Keep actual selected-folder, worker/OPFS and portable path outcomes separate; unexecuted cases stay not-run. |

### CONTOUR — inputs and coverage

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| D1 | Existing NC/CA matching failures establish behavior, not successful medium/large workloads. | P3 | WP-08 | Preserve original failures and require separate successful full-chamber rows. |
| D2 | Geography vintage, population basis and original unit order are part of the experiment. | P3 | Matrix/evidence rules | Keep immutable source identities and explicit alignment; do not interchange ACS estimates and PL94 counts. |
| D3 | Successful independent states cannot establish a national acquisition/run/output/recovery journey. | P3 | Ranked task 3 | Keep explicit participation, chamber, backend/seed policy and native expected outcomes before national acceptance. |

## Synthesis

Roles reviewed: 6. P1 blockers: 0. P2 issues: 3. P3 notes: 15.

Verdict: **APPROVED-WITH-CONDITIONS for the amended plan**. S1/C1 documentation defects are corrected; B1 is now an explicit acceptance gate and remains unimplemented/unproven at runtime. No implementation or release sign-off follows.

Top finding: passing source/native-host checks must not be promoted to actual WASM/UI/package support. SURVEY, BENCHMARK, COVENANT and LEDGER converge on this boundary. VAULT and BENCHMARK converge on fresh authority and retained results across cancellation.

## Three amendments applied

1. Reconcile the handoff to the current validated inventory (98 pins) and describe the build prototype's exact evidence limits.
2. Require the state-completed-before-final-reply cancellation case in actual workers, with retained accepted output, stale-event refusal and inert Open/fresh-root continuation.
3. Update the build contract's status/dependency while preserving the accepted summary package and all remaining actual-profile, UI, host, matrix and release gates.

## Validation

`python scripts/web/wasm_practitioner_inventory.py` passes: 69 command/screen rows, 17 TUI action groups, 98 source pins.

`python scripts/web/wasm_parity_ledger.py` passes: 309 structured records, 240 engine-facing declarations and 152 native traces; 22 fields have no direct read. Unresolved records are permitted; these are source/structure results, not execution or full parity.

Inspected retained host log: `target/build-application/native-host-application-v4.log`, reporting seven cases and 22 recovery boundaries under `target/build-application/run-1792820651569`. This review does not independently replay those tests. The current code's awaited engine call and missing progress callback support B1; no claim that the potential loss was reproduced in a real WASM worker is made.

Prior review conditions remain scoped by their original IDs. Successful medium/large/national runs, current native/default/C compilation, build UI/project/archive integration, real host/capacity cases, final staged-package acceptance and formal release remain open. Coding agent owns implementation; BISECT maintainer owns milestone acceptance.
