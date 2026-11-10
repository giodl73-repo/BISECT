---
skill: roles-check
topic: wasm-native-fetch-folder-install
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, CONTOUR, VAULT, TRENCH]
p1_count: 0
p2_count: 7
p2_open: 1
p3_count: 11
verdict: APPROVED-WITH-CONDITIONS
---

# Reviewed Fetch folder application

One agent applied six installed lenses to the [contract](../../../docs/specs/wasm-native-fetch-folder-install.md),
private grants, source preparation, UI and retained native/browser evidence. These
are perspectives, not independent signatories. Approval covers the local bounded
adapter; native host permission and full release acceptance remain open.

## BENCHMARK

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| B1 | JS folders alone cannot prove native skip/force behavior. | P2 locally closed | Five actual CLI file-tree comparisons per profile, using original RI bytes and controlled gh transport, cover supported success/skip/force cases. |
| B2 | Injected handles and OPFS cannot prove external native-directory grants. | P2 OPEN | Distinct evidence realms; native picker check not-run after two initialization failures. No actual OS permission/revocation or hosted claim. |
| B3 | Loading the C shell is not C partition execution. | P3 bounded | No engine operation runs in installation; unchanged artifacts and separate backend scope disclosed. |

## COVENANT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| C1 | Presence skip and completed write receipts could imply validity. | P2 locally closed | Native presence explicitly unvalidated; receipts record source hashes, effects, marker state and authenticity=false. No graph/producer/certification claim. |
| C2 | Interrupted writes cannot be represented as rollback. | P3 carried | Completed files remain; attempted/unknown effects are reported and new review is required. |
| C3 | Live source mutation must not alter an already reviewed write. | P3 bounded | Private byte copies and source-inventory identity; mutation test preserves reviewed originals. |

## LEDGER

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| L1 | Native incomplete-target overwrite differs from an always-require-force policy. | P2 locally closed | Missing primary or marker permits reviewed clobber without Force; both paths present skip unless Force, matching actual CLI final trees. |
| L2 | Marker invalidation intentionally changes failure side effects. | P3 bounded | Explicit browser safety adaptation, shown in Review and contract; no claim of identical partial-write behavior. |
| L3 | Old outcomes cannot restore folder capability. | P3 bounded | Passive receipt only; Open drops root/review/acquired bytes, resets Force, no picker/network/write. |

## CONTOUR

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| G1 | Installing opaque files could accidentally parse or adopt graph data. | P2 locally closed | Hash/copy/write only; real original pickle/bin/GEOIDs retained, no engine input callback or operation. |
| G2 | Native presence may hold for malformed paths/data. | P3 carried | Presence semantics remain separate from validation; planned directory collisions refuse before writes. |
| G3 | RI write success does not prove other vintages/scales or missing geometry. | P3 carried | Fixed native V3 directory and bounded source/target scope documented; no derivation or graph-readiness claim. |

## VAULT

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| V1 | Serialized or reused reviews could write outside current intent. | P2 locally closed | WeakMap-bound live root, immutable source copies, one-use review, confirmation, current context and permission checks. |
| V2 | Case aliases or foreign paths could be overwritten. | P3 bounded | Fixed native components and safe source basenames; case/directory collisions refuse; unrelated files preserved. |
| V3 | Page settings must not request or silently retain native-folder access on Open. | P3 bounded | Picker is explicit; no handle in project/history. Actual browser permission state is a separate host concern under B2. |

## TRENCH

| ID | Finding | Severity | Resolution |
|---|---|---|---|
| T1 | A cancelled final directory lookup could still create an empty source file. | P2 locally closed | Exact prototype red/final-v2 green; post-await checks stop before getFileHandle(create:true). Directory creation already in flight remains a reported possible effect. |
| T2 | Stale or failed writes could retain an old marker. | P3 bounded | Recheck before changes, invalidate old marker first, readback each file, marker last; stale/failure/cancel tests and real OPFS recovery pass. Native presence remains weaker than validation. |
| T3 | Close/reload can lose an interrupted attempt's receipt. | P3 carried | Current-page outcome download only; durable Fetch history, actual close/reload and cross-process locking remain open. |

Six roles /18 findings: zero P1, seven P2 (six locally closed, B2 OPEN), 11 P3.
BENCHMARK and VAULT agree OPFS is no substitute for native permission evidence.
COVENANT and LEDGER require explicit presence-vs-validity and failure-adaptation
disclosures. Code is implemented/evidence-partial; full parent support stays open.

Amendments applied: (1) separate explicit selection/review/application and private
single-use authority; (2) marker invalidation/readback/last commit with honest partial
outcomes; (3) preserve native success comparisons, post-await red/green and separate
injected/real-OPFS/not-run-native realms. PP-52 records stale asynchronous directory
capability. No deployment or maintainer acceptance is implied.
