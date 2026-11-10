---
skill: roles-check
topic: wasm-fetch-history-process-crash
date: 2026-10-26
roles_used: [BENCHMARK, DATUM, COVENANT, SURVEY, VAULT, TRENCH]
p1_count: 0
p2_count: 4
p2_open: 1
p3_count: 14
verdict: APPROVED-WITH-CONDITIONS
---

# Fetch history after an abrupt browser-process termination

One agent applies six installed `.roles` lenses to a verification-only follow-up
of the unchanged default/C-enabled Fetch-history packages. These perspectives
are not independent signatories. BENCHMARK assesses the fault injection; DATUM
checks evidence scope; COVENANT checks retained source/package identity; SURVEY
checks the post-crash recovery path; VAULT checks process/profile ownership and
restored authority; TRENCH checks failure and cleanup behavior.

| Role / ID | Finding | Severity | Resolution / reference |
|---|---|---|---|
| BENCHMARK B1 | Closing a page does not test process termination. | P2 locally closed | Owned Windows Chrome tree is force-terminated at three checkpoints and its parent exit is awaited. `crash()` in the new gate. |
| BENCHMARK B2 | OPFS cannot establish external native-directory grants. | P2 OPEN | Native picker/grant/revocation evidence remains not-run; host acceptance stays open. |
| BENCHMARK B3 | A retained terminal receipt alone can conceal missing files. | P3 bounded | Restart reads actual OPFS files, checks original source SHA-256 and exact marker bytes. |
| DATUM D1 | One successful crash experiment is not a universal durability guarantee. | P3 carried | Three named checkpoints in each of two unchanged browser shells; no power-loss, stress, browser/OS matrix or exhaustive fault schedule claim. |
| DATUM D2 | The C shell can be mistaken for C partition execution. | P3 bounded | Browser filesystem/storage code only; no engine operation executes. |
| DATUM D3 | A test with assisted teardown must not become the accepted run. | P3 bounded | Prototype evidence explicitly records external owned-process cleanup and is excluded from final passing gates. |
| COVENANT C1 | Tests might silently execute different candidate bytes. | P2 locally closed | Sealed package manifest verified before/after; exact test-source identity is checked at completion. |
| COVENANT C2 | History still is not atomic with filesystem effects. | P3 carried | Restarted in-progress records retain no final receipt and report unknown effects; actual files are separately observed in test evidence. |
| COVENANT C3 | A crashed attempt's local record is not producer authenticity. | P3 bounded | No signature/certification or source-authenticity promotion; passive-data contract unchanged. |
| SURVEY S1 | Restart might silently continue acquisition or writes. | P3 bounded | Picker/source/effect counters stay zero; Apply/Review disabled until explicit selection and source handoff. |
| SURVEY S2 | Force might persist after a failed forced replacement. | P3 bounded | Stored record keeps historical force=true, but live checkbox resets false; fresh review recovers without Force when marker is absent. |
| SURVEY S3 | A browser crash can leave an old marker advertising incomplete data. | P3 bounded | Forced-case checkpoint follows old-marker removal and first source close; marker remains absent after restart, then is committed last on explicit recovery. |
| VAULT V1 | Killing a browser could affect the user's browser session. | P2 locally closed | Fresh owned profile and owned live child PID only; taskkill targets that process tree, with no inherited login/profile. |
| VAULT V2 | Raw browser profiles could leak secrets or inflate evidence storage. | P3 bounded | Only local test fixtures enter the profile; its resolved owned boundary is checked before removal. No raw profile is retained as evidence. |
| VAULT V3 | Stored history might reconstruct folder authority after restart. | P3 bounded | No root/source/review/Force restored; test OPFS handle is supplied only by its explicit controlled picker. |
| TRENCH T1 | CDP disconnect does not establish browser-process death. | P3 bounded | Prototype exposed a teardown hang; final gate explicitly terminates any remaining owned Chrome tree and awaits exit. PP-54. |
| TRENCH T2 | A crash after terminal receipt may differ from a mid-write crash. | P3 bounded | Both are exercised; completed receipt identity and exact files survive the terminal checkpoint. |
| TRENCH T3 | A browser-process crash does not establish power-loss behavior. | P3 carried | OS remains running throughout. No disk/controller/host power failure is simulated. |

Six roles /18 findings: zero P1, four P2 (three locally closed, B2 open), 14 P3.
Verdict: APPROVED-WITH-CONDITIONS for these process-restart observations only.
DATUM and COVENANT require checkpoint-specific wording; BENCHMARK and VAULT keep
native-directory permission evidence separate. Full lifecycle/parent acceptance
remains open.

Amendments applied: (1) kill the complete owned browser tree and observe exit,
(2) verify actual bytes and marker state across each restart, (3) repair CDP
teardown, retain the assisted prototype separately, and delete only the checked
owned test profile. No implementation, engine, deployment or release promotion.
