---
skill: roles-check
topic: wasm-cffi-worker-recovery
date: 2026-10-27
roles_used: [benchmark, covenant, ledger, vault, survey, trench]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Explicit workbench C retry

One agent applies six installed lenses, not independent signatories. BENCHMARK
reviews recovery evidence, COVENANT its provenance, LEDGER error semantics,
VAULT private runtime authority, SURVEY the practitioner retry path, and TRENCH
cache lifecycle failures. Artifact: private retirement getter, worker cache
invalidation, UI guidance and actual browser integration checker.

| ID | Lens | Finding | Severity | Resolution or condition |
|---|---|---|---|---|
| B1 | BENCHMARK | A direct bridge test misses a stale worker cache. | P2 | Actual workbench/worker trap cases require two asset loads on two explicit runs. |
| B2 | BENCHMARK | A passing recovery oracle must detect the prior behavior. | P2 | Retained previous worker fails the recovery-code assertion. |
| B3 | BENCHMARK | Controlled traps are not actual browser exhaustion. | P3 | OOM, termination, other browsers and capacity remain open. |
| C1 | COVENANT | Overlay testing could be mistaken for immutable package acceptance. | P2 | Evidence explicitly identifies current source overlay; final-package mode requires matching served source hashes. |
| C2 | COVENANT | Fixture code may be confused with the production C bundle. | P2 | First bundle is explicitly hashed fault glue around real C; trap retry receives exact pinned unmodified assets. |
| C3 | COVENANT | Retained native assignments are not a newly built native oracle. | P3 | Claim is equality with retained fixture; wider native matrix stays separate. |
| L1 | LEDGER | Failed execution could be promoted to success by automatic replay. | P2 | Exactly two user Run messages; first remains error with retained request. |
| L2 | LEDGER | Ordinary errors could unnecessarily discard a healthy runtime. | P2 | Allocator refusal succeeds on explicit retry with one bundle load. |
| L3 | LEDGER | Worker recovery metadata must not alter imported receipt schemas. | P3 | Fixed live-message enum only; failure receipt remains its existing strict schema. |
| V1 | VAULT | Imported metadata must not reconstruct retirement authority. | P2 | Getter reads existing private WeakMap; project input cannot set it. |
| V2 | VAULT | Retry might call the poisoned module again. | P2 | Bridge blocks reentry; worker invalidates loaded cache and creates a fresh engine/module on next explicit C Run. |
| V3 | VAULT | Recovery guidance could itself fetch or execute assets. | P3 | Error display and project save are inert; counters checked before second click. |
| S1 | SURVEY | A generic METIS error leaves practitioners without a recovery path. | P2 | Fixed status explains fresh runtime on next Run with C enabled. |
| S2 | SURVEY | Failed plans might remain exportable. | P2 | Completed-plan export disabled; saved result null and failed request retained. |
| S3 | SURVEY | Recovery may lose previous completed batch checkpoints. | P3 | Existing batch handling retained; wider interrupted-batch recovery acceptance remains separate. |
| T1 | TRENCH | Loaded flags can outlive a permanently retired instance. | P2 | PP-65 records private status/cache invalidation with actual worker regression. |
| T2 | TRENCH | Clearing only the flag could still reuse the poisoned engine. | P2 | C enable builds a fresh module and Rust engine; recovered runtime identity and assignments checked. |
| T3 | TRENCH | Source-wiring a checker is not a full builder run. | P3 | Final-package gate registered; entire updated builder explicitly not run here. |

Roles reviewed: 6. P1: 0; P2: 12; P3: 6.
Verdict: APPROVED-WITH-CONDITIONS for explicit current-source workbench recovery.
Top finding: worker cache state must follow private module retirement without
replaying the failed request. BENCHMARK and TRENCH agree the old-worker failure
and actual UI counters establish this boundary.

Three amendments: retain explicit-only retry/cache invalidation; run the full
updated final-package builder before release acceptance; separately cover real
exhaustion/termination, batch checkpoints and other practitioner host profiles.
