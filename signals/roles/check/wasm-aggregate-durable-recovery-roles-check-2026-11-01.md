---
skill: roles-check
topic: wasm-aggregate-durable-recovery
date: 2026-11-01
roles_used: 5
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Selected Aggregate journal and explicit recovery

Review scope: unadopted `native-aggregate-application.js` and `native-aggregate-application-journal.js`, with final source/test capture at `target/aggregate-application-planner-20261101/journal-reviewed-De0dF0/`. This continues the prior workspace review. The seventeen installed roles were enumerated; BENCHMARK, COVENANT, DATUM, SURVEY and TRENCH were selected for differential evidence, authority, claim scope, operation and interruption behavior. One agent applies these lenses; they are not independent reviewers.

Execution: `journal-5HQdvr/evidence.json` covers eleven actual-file/WASM cases; `journal-browser-vRdPur/evidence.json` covers actual Chrome OPFS, a real page reload, fresh native CLI counterpart namespaces, metadata-only listing, confirmed recovery, unacknowledged close and injected acknowledgement-write failure. Independent readback verifies raw captured journal bytes, hashes/sequence/transitions, unchanged sources, retained empty entry, reload boundaries, exact native namespaces and source hashes matching browser-served bytes. Native counterpart stdout/stderr/arguments, profiles, earlier revisions and failed fixture remain retained. Test scripts were captured after execution, not precommitted.

| # | Lens | Finding | Severity | Recommendation |
|---|---|---|---|---|
| 1 | BENCHMARK | Four earlier writer cases and the five final journal cases compare native namespaces; journal files are additional browser side effects. | P3 | Keep the separate comparison explicit; never call the complete output roots identical. |
| 2 | BENCHMARK | Actual OPFS acknowledgement failure uses an injected QuotaExceededError, not naturally exhausted device storage. Permission loss remains untested here. | P2 | Add scoped permission-loss and capacity witnesses before their profile claims. |
| 3 | BENCHMARK | Execution still uses the owned earlier matching engine/CLI, and scripts live under target. | P2 | Before adoption, make maintained tests use freshly built images, register builders and reconcile current source catalogs/assets. |
| 4 | COVENANT | A raw hash chain detects changed records relative to a retained identity but is not a signature or source authentication. | P3 | Continue calling journals unauthenticated history and require recomputation plus live source/target review. |
| 5 | COVENANT | Baselines retain selected raw reports locally, bounded to 16 MiB serialized journal input. They can contain private practitioner data. | P2 | Explain local journal contents in the eventual UI and export only through explicit user action. |
| 6 | COVENANT | IDs are constrained, paths fixed, records bounded, and imported/copied reviews confer no root authority. | P3 | Preserve private single-use grants and newly selected roots across project Open and recovery UI. |
| 7 | DATUM | Here durable means create/write/close/readback through browser filesystem APIs, not hardware fsync or multi-file atomicity. | P3 | Retain that limit; do not infer power-loss guarantees from reload tests. |
| 8 | DATUM | Freshly observed expected bytes can support explicit acknowledgement of an unacknowledged close, but cannot prove who wrote them. | P3 | Keep the disposition visible and avoid source-authenticity or exclusive-writer claims. |
| 9 | DATUM | Empty record files remain uncommitted; raw verification proves they survive recovery and do not advance the semantic cursor. Nonempty malformed records refuse. | P3 | Keep raw-file witnesses and distinguish an empty creation artifact from corrupt or unknown JSON. |
| 10 | SURVEY | There is no installed picker/review/apply/recovery UI for this candidate. | P2 | Build source/version selection, metadata-only history list, ordered review, confirm/cancel and interruption messages next. |
| 11 | SURVEY | Identical before/after bytes make an outstanding repeated Force write ambiguous. Automatic continuation correctly refuses it. | P2 | Explain the reason and provide a separately reviewed explicit new-run path; never silently guess or count it as acknowledged. |
| 12 | SURVEY | Journal history is retained without cleanup and listing is bounded to 512 IDs. | P3 | Explain storage management; do not delete evidence or introduce automatic retention without a defined workflow. |
| 13 | TRENCH | Rechecks cannot prevent an external writer acting between validation and write. | P2 | Define conflict/recovery behavior and avoid atomic/exclusive installation claims. |
| 14 | TRENCH | Nonempty corrupt/missing records, absent prepared checkpoints and unknown target bytes remain explicit refusals, not general repair support. | P2 | Surface these states in UI and define any later repair as a separate fresh review; preserve evidence. |
| 15 | TRENCH | Epoch supersession uses JavaScript handle identity; aliases of one filesystem directory and separate tabs are not coordinated. | P2 | Add same-entry coordination and cross-context behavior before claiming that class of concurrency support. |

Roles reviewed: 5. P1 blockers: 0; P2 issues: 8; P3 notes: 7. Verdict: APPROVED-WITH-CONDITIONS for isolated continuation. UI, integration and full workflow/profile acceptance remain blocked by their unmet conditions; this is not a shipping approval or evidence-state promotion.

Amendments: (1) record ambiguous repeated-write and empty-journal-entry pitfalls without declaring maintained closure; (2) build the practitioner UI against explicit fresh-root review and declared dispositions; (3) keep the active R12 capture unchanged, then integrate assets/tests/all four catalogs together and rerun complete affected gates. The earlier in-memory-only limitation now has a scoped tested journal implementation; it does not erase the historical checkpoint or complete the parent contract.

Execution addendum: a journal-only recovery receipt originally reported `journal_effects_may_have_started:false`. A retained red control against the earlier captured source fails after observing its real completion-record write. The candidate sets this field before recovery checkpoints; the same test passes, and full candidate actual-file/OPFS suites were rerun with matching final source capture. `journal-reviewed-De0dF0/independent-audit.json` verifies both controls and final browser receipt flags. PP-85 remains open for adoption; the original `journal-reviewed-Gz7r7G/` proof is preserved with its narrower assertions. No other review condition or parent evidence state changes.
