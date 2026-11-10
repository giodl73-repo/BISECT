---
skill: roles-check
topic: wasm-fetch-outcome-history
date: 2026-10-26
roles_used: [BENCHMARK, COVENANT, LEDGER, SURVEY, VAULT, TRENCH]
p1_count: 0
p2_count: 6
p2_open: 1
p3_count: 12
verdict: APPROVED-WITH-CONDITIONS
---

# Passive Fetch history review

One agent applied six installed `.roles` lenses to the contract, store, UI and
browser tests. These are review perspectives, not independent signatories.
BENCHMARK covers test strength; COVENANT covers evidence limits; LEDGER covers
schema compatibility; SURVEY covers recovery UX; VAULT covers stored authority
and hostile data; TRENCH covers asynchronous failure and concurrency.

| Role / ID | Finding | Severity | Resolution / section |
|---|---|---|---|
| BENCHMARK B1 | A simulated store cannot prove close/reload persistence. | P2 locally closed | Real IndexedDB and actual OPFS file close before page close; reload asserts an in-progress record with no receipt. Browser gate. |
| BENCHMARK B2 | OPFS and intercepted picker do not prove native grants. | P2 OPEN | Native directory grant/revocation and browser-process crash remain acceptance gaps. Validation scope. |
| BENCHMARK B3 | Both shells must include the newly imported module. | P3 bounded | Staged module closure and offline manifest checks in both profiles; no partition backend execution claimed. |
| COVENANT C1 | A stored status could be mistaken for a verified folder. | P2 locally closed | Every loaded observation is unverified; no receipt after close means unknown effects. Contract and UI. |
| COVENANT C2 | Filesystem and storage are not atomically committed. | P3 carried | Initial/final checkpoints only, no per-file custody or power-loss guarantee. |
| COVENANT C3 | Hashes and local records do not prove producer authenticity. | P3 bounded | Raw receipt retains authenticity=false; no certification promotion. |
| LEDGER L1 | Fetch history could overwrite Run history with a different schema. | P2 locally closed | Fixed separate database names, unit override test and actual browser coexistence. |
| LEDGER L2 | History wrapper could break session receipt consumers. | P3 bounded | Existing session download remains raw terminal receipt; stored download uses documented versioned wrapper. |
| LEDGER L3 | Unknown receipt paths/status flags could acquire meaning later. | P3 bounded | Strict field/schema/scope/path/effect checks refuse malformed history. No import-to-execution route. |
| SURVEY S1 | A reload might imply resume or keep Force armed. | P3 bounded | No root/source/review/Force restored; explicit selection, acquisition, review and Apply. |
| SURVEY S2 | Storage failure could hide an otherwise successful folder Apply. | P3 bounded | Warning remains separate from operation status; session receipt download still works in unavailable-storage test. |
| SURVEY S3 | A single latest entry is not an archive. | P3 carried | UI and contract disclose eviction and downloaded backup; one new Apply replaces prior observation. |
| VAULT V1 | Untrusted history might carry folder grants or invoke accessors. | P2 locally closed | No handle/grant fields; bounded JSON validation before clone/serialization; getter/toJSON tests. |
| VAULT V2 | Root display name can disclose local naming conventions. | P3 bounded | Name only, no absolute path; contract discloses local and downloaded copy. |
| VAULT V3 | Attempt IDs could be presented as trusted authentication. | P3 bounded | CAS coordination only; same-origin/local edits outside provenance claim. |
| TRENCH T1 | A stale tab can overwrite or delete a newer record. | P2 locally closed | Expected-ID save and Forget in one readwrite transaction; actual two-tab rejection tests. |
| TRENCH T2 | An old asynchronous startup read can replace newer UI state. | P3 bounded | Epoch guards startup/Refresh/persist/Forget; storage operations themselves retain CAS. |
| TRENCH T3 | Browser close after a write can erase progress knowledge. | P3 carried | Conservative initial checkpoint survives tested close; effect count remains unknown. PP-53. |

Six roles, 18 findings: P1=0; P2=6 (five locally closed, B2 open); P3=12.
Verdict: APPROVED-WITH-CONDITIONS for the bounded local adapter. COVENANT and
BENCHMARK agree page-close evidence must not become crash/native-host durability.
VAULT and LEDGER agree stored history never restores write authority.

Amendments applied: (1) separate initial and terminal checkpoints with unknown
effects after close; (2) separate bounded schema/database with atomic stale-tab
refusals; (3) preserve session receipt export and storage warnings while keeping
native permission/crash/host gates open. No deployment or full parity approval.
