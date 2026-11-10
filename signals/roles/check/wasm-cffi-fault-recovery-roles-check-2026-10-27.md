---
skill: roles-check
topic: wasm-cffi-fault-recovery
date: 2026-10-27
roles_used: [benchmark, covenant, ledger, vault, survey, trench]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Controlled C failure review

One agent applies six installed lenses; these are not independent signatories.
Artifact: bridge delayed publication, deterministic fault fixture, Node/Chrome
worker runner, preview-builder registration and bounded claims.

| ID | Lens | Finding | Severity | Resolution or condition |
|---|---|---|---|---|
| B1 | BENCHMARK | A failure test must detect the observed bug. | P2 | Exact old bridge fails on cleanup-trap memory publication; log and preimage retained. |
| B2 | BENCHMARK | JavaScript errors alone would not prove WASM trap handling. | P2 | Fixture calls a real WASM unreachable export; Node and Chrome worker agree. |
| B3 | BENCHMARK | Injection does not establish capacity or real exhaustion. | P3 | Explicit exclusions; native equivalence matrix is separate. |
| C1 | COVENANT | Changed code must not inherit old package acceptance. | P2 | Historical packages/receipts untouched; current source checks have their own identities. |
| C2 | COVENANT | Fixture assets may differ from the actual vendored module. | P2 | Real C bytes checked against pinned manifest; modified glue explicitly hashes as a test fixture. |
| C3 | COVENANT | Hashes cannot authenticate release or historical execution. | P3 | No deployment/release/attestation claim. |
| L1 | LEDGER | Failure code may coexist with published output. | P2 | Private result snapshot is copied only after successful cleanup; whole caller memory checked. |
| L2 | LEDGER | Ordinary allocation refusal and fatal traps have different lifetimes. | P2 | Refusal returns -3 and permits retry; traps return -4 and prevent reentry. |
| L3 | LEDGER | A controlled allocator fixture is not a native error matrix. | P3 | Distinction recorded; parent contracts stay partial. |
| V1 | VAULT | A poisoned C module could be reentered for cleanup. | P2 | No calls after trap, including allocator/free; counters asserted at all positions. |
| V2 | VAULT | Testing could weaken fresh-loaded authority. | P2 | Existing verified loader/private authority used unchanged; no project execution path added. |
| V3 | VAULT | Test instrumentation might be mistaken for production input. | P3 | Glue and metric globals live only in test fixture/dedicated worker. |
| S1 | SURVEY | Failure might leave Rust backend selection on C. | P2 | Actual engine failure followed by fresh baseline equality for every fault category. |
| S2 | SURVEY | Retirement without a recovery path may strand work. | P2 | Fresh C instance runs successfully; UI restart guidance remains separate acceptance work. |
| S3 | SURVEY | Headless Chrome does not establish every practitioner host. | P3 | OS permissions, other browsers and real exhaustion remain open. |
| T1 | TRENCH | Cleanup traps can leak a failed operation's output. | P2 | PP-64 records structural delayed publication and failing-before/passing-after regression. |
| T2 | TRENCH | Failure after partial allocation may leak live allocations. | P2 | Each allocation refusal frees exactly its predecessors; trapped instances receive no further calls. |
| T3 | TRENCH | Registration alone is not a completed package builder run. | P3 | Dedicated checker passed; entire updated builder explicitly not rerun. |

Roles reviewed: 6. P1: 0; P2: 12; P3: 6.
Verdict: APPROVED-WITH-CONDITIONS for controlled bridge failure handling only.
Top finding: caller outputs must remain private until fallible cleanup finishes.
BENCHMARK and TRENCH agree the old-source regression establishes this boundary.

Three amendments: retain delayed publication and its old-source oracle; run the
complete updated package/release path before release acceptance; separately
validate actual exhaustion/termination, UI recovery and broader host workloads.
