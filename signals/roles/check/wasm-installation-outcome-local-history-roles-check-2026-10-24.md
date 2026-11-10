---
skill: roles-check
topic: wasm-installation-outcome-local-history
date: 2026-10-24
roles_used: [BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH]
p1_count: 0
p2_count: 3
p3_count: 12
verdict: APPROVED-WITH-CONDITIONS
---

# Local installation history implementation review

Artifact: [local-history contract](../../../docs/specs/wasm-installation-outcome-local-history.md),
the UI/store, strict schema gate and extended browser journeys. One coding agent
applied five installed role lenses, not independent signatories. BENCHMARK covers
regression proof; COVENANT custody; LEDGER schema interoperability; VAULT passive
local data/capability boundaries; TRENCH interruption and concurrent-tab failure.
No geographic, statistical or electoral algorithm changed.

## BENCHMARK

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| B1 | Merely cancelling gracefully does not prove close-time history. | P2 locally closed | Both batch browsers reload during a real paused write and download the exact prior progress marker; no new request/picker. |
| B2 | Storage refusal must preserve usable session output. | P3 bounded | Both single browsers perform actual installation with IndexedDB unavailable, then download the completed receipt. |
| B3 | Synthetic OPFS proof is not OS permission or national capacity proof. | P3 carried | Contract names fixture, injected picker and host limits. |

## COVENANT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| C1 | A persisted progress marker cannot prove final filesystem state. | P3 bounded | Null receipt and unknown-writes wording; actual incomplete marker observed after reload. |
| C2 | Browser eviction/early close can still lose evidence. | P3 carried | UI and contract say storage may be cleared; downloaded backup remains necessary. |
| C3 | One record is not a complete audit history or authentic provenance. | P3 carried | Replacing prior attempt is explicit; original native/project artifacts retain separate custody. |

## LEDGER

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| L1 | Persisted records need a versioned, bounded interchange envelope. | P3 bounded | Versioned database/envelope, one-MiB UTF-8/50k-value guard, strict allowed wrapper fields and existing parser limits. |
| L2 | Original receipts must retain their exact data. | P3 bounded | JSON roundtrip/deep equality against downloaded original and actual checkpoint; no receipt normalization. |
| L3 | Historical execution hashes cannot be refreshed as new proof. | P3 bounded | Original active inventory/source files retained; only current candidate pins reconciled; new staged record is separate. |

## VAULT

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| V1 | Stored completion must never restore write authority. | P2 locally closed | Separate passive display/download; reload/Open starts zero actions and install stays disabled. No request importer consumes history. |
| V2 | Malformed or oversized history must refuse before interpretation. | P3 bounded | Strict parser/schema gate; unknown handles/fields/status, hostile JSON and UTF-8 limits tested. |
| V3 | Stored/downloaded local errors can include paths and are not encrypted. | P3 carried | Local origin storage, explicit download/Forget; no upload, no stronger secrecy/authenticity claim. |

## TRENCH

| ID | Finding | Severity | Resolution / condition |
|---|---|---|---|
| T1 | A stale tab could overwrite completion or erase newer work. | P2 locally closed | Atomic compare-ID final save/Forget; both browser gates verify preserved newer record. |
| T2 | Failed initial storage must not leave a false durable-success message. | P3 bounded | Explicit storage failure plus session fallback; no invented saved record. |
| T3 | Browser death before initial commit and quota exhaustion remain possible. | P3 carried | Pre-effect commit attempted/awaited; no guaranteed durable custody; PP-46 records the pattern. |

## Synthesis and amendments

Five roles, 15 findings: zero P1, three locally closed P2, 12 bounded/carried P3.
APPROVED-WITH-CONDITIONS for the local-history UI only. Top finding: persisted
progress is an observation of an attempt, not a receipt proving filesystem state.
COVENANT and TRENCH agree that progress/null receipt requires unknown-writes wording.

Amendments applied: (1) a bounded passive store separated from project/folder
capabilities; (2) pre-effect progress plus atomic attempt-ID final save/Forget;
(3) real reload-mid-write, stale-tab, storage failure and inert restore tests on
both shells. Real OS host behavior, browser eviction, multi-attempt/cross-device
custody and full parent/acceptance-v1 remain open. No deployment approval is implied.
