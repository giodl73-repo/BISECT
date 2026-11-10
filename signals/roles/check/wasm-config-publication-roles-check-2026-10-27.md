---
skill: roles-check
topic: wasm-config-publication
date: 2026-10-27
roles_used: 6
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

Standard single-agent application of installed lenses, not independent reviewers
or maintainer acceptance. Artifacts: config-publication.js, UI integration,
project recipe, native-reference/browser tests, builder registration and spec.

BENCHMARK: independent byte/outcome observations. COVENANT: provenance and scope.
LEDGER: native file interchange. VAULT: private write authority. SURVEY:
practitioner operation/receipt semantics. TRENCH: reproducible failure structure.

| ID | Lens | Finding | Severity | Resolution or remaining condition |
|---|---|---|---|---|
| B1 | BENCHMARK | Shared renderer output alone cannot prove native writer behavior. | P2 | Six actual CLI writer variants anchor exact output; collision/changed-byte force/dry-run retained. |
| B2 | BENCHMARK | A Windows facade does not exercise browser writable semantics. | P2 | Separate real worker/OPFS journeys per profile; injected picker explicitly labeled. |
| B3 | BENCHMARK | Gate registration could be mistaken for a complete builder pass. | P3 | Spec separates component/package checks from pending whole-builder acceptance. |
| C1 | COVENANT | Hash readback does not establish source authenticity or atomicity. | P2 | Review/receipt/UI disclose private authority and external-writer race limits. |
| C2 | COVENANT | New overlay could be attributed to an older sealed ZIP. | P2 | Distinct 7c67474c5785 package; earlier packages and ZIP preserved. |
| C3 | COVENANT | Native permission and Cloudflare support could be inferred from local OPFS. | P3 | Neither claimed; actual OS picker and host remain open. |
| L1 | LEDGER | Default output and missing parents are part of native behavior. | P2 | configs/generated filename and exact six native YAML byte comparisons. |
| L2 | LEDGER | Applying write-path validation in dry-run changes native ignored-out behavior. | P2 | Bounded inert path retained; bomb-root test proves no destination access. |
| L3 | LEDGER | Portable path constraints are stricter than unrestricted native paths. | P3 | Explicit bounds and exclusions documented. |
| V1 | VAULT | Saved paths, force or historical reviews could restore write authority. | P2 | Closed optional recipe; private WeakMap single-use review; Open has zero destination/worker calls. |
| V2 | VAULT | A target can change after review. | P2 | Recheck source/permission/lineage/hash before effects and write; changed bytes refuse. Concurrent race remains disclosed. |
| V3 | VAULT | Unsupported permission APIs could be hidden by a passing facade. | P3 | No actual OS grant accepted; regular download/dry-run fallback retained. |
| S1 | SURVEY | Cancellation can leave parents or an empty/changed file. | P2 | Concrete attempted-path/effect/close/readback flags; no silent cleanup or automatic retry. |
| S2 | SURVEY | Force can overwrite unexpected data unless review shows the old file. | P2 | Old hash shown; explicit confirmation; changed old bytes refuse. |
| S3 | SURVEY | A historical failure receipt could appear to authorize the next write. | P3 | Receipt is passive, labeled historical, downloadable separately. Fresh review always required. |
| T1 | TRENCH | Reentrant handler rejection inside finally can clear an active operation. | P2 | Guards before try/finally; delayed parent browser test invokes both handlers and preserves cancellation. PP-71 recorded. |
| T2 | TRENCH | Failure after close can be misreported as no effects or success. | P2 | Node cancellation after close retains actual bytes and close flag without verified-readback success. |
| T3 | TRENCH | Bounded Config implementation could close a full command parent. | P3 | Required child implementation advances; evidence/parents stay partial and formal acceptance open. |

Roles reviewed: 6. P1: 0; P2: 12; P3: 6.
Verdict: APPROVED-WITH-CONDITIONS for the documented selected-root adaptation.
Top finding: interrupted publication must preserve the distinction between
possible effects, completed close and verified output. VAULT, SURVEY and TRENCH
agree that saved data and failure receipts cannot authorize replay.

Amendments applied: private single-use authority with explicit confirmation and
source/target rechecks; independently observed partial effects and byte readback;
separate injected-OPFS evidence from actual OS picker/host/full acceptance.
Remaining acceptance conditions: actual native folder permission, broader
host/capacity matrices, new whole-builder/clean-checkout and acceptance-v1.
