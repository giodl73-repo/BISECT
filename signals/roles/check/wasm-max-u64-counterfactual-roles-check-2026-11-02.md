---
skill: roles-check
topic: wasm-max-u64-counterfactual
date: 2026-11-02
roles_used: [benchmark, covenant, ledger, survey, contour, trench]
p1_count: 3
verdict: NEEDS-WORK
---

# Native CLI counterfactual manifest review

Artifact: test_native_cli_unused.py, native_cli_state_observables.py and retained R15/focused-core-r3/negative-controls evidence. Six verify lenses were selected from the sixteen installed roles plus ROLE.md: test instruments, custody, interchange, operator meaning, physical data and failure prevention. One agent applied these lenses; no independent-reviewer claim. The separately reviewed fine-replay export candidate is unchanged except current source metadata and this test integration.

| Role | Finding | Severity | Resolution / remaining action |
|---|---|---|---|
| benchmark | R15 stopped at a stale expectation that maximum u64 seed wraps to -1. | P2 | Require legacy signed seed null, exact engine_seed integer and exact recipe decimal seed. Focused gate passes all original 25 native invocations, 22 variants, 22 WASM executions and 3 print-only adaptations. |
| benchmark | Projecting away an entire context or provenance object would hide regressions. | P2 | Verify native typed recipe/context binding first; independently recompute the context hash. Only the exact declared cwd and its derived comparison hashes change. Full plan, context, provenance, effective fields and remaining recipe compare. Ten controls include option, population, label and binding drift. |
| benchmark | Focused success does not turn the failed full build into a passing release. | P1 | Preserve R15 failure and unrun C/package/final-proof stages. Fresh applicable adopted-source checks and complete original builders remain required. |
| covenant | Recipe hashing is typed serialization, while context hashing is canonical JSON. | P2 | Native verify --dry-run checks the typed recipe digest, audit certificate and context linkage. Python independently checks the fixture context projection. Do not substitute sorted recipe JSON for its wire digest. |
| covenant | Synthetic four-unit metadata checks do not prove source authenticity or execution history. | P2 | Declaration and source hashes keep this bounded. Native dry-run performs no reproduction; 25 native command executions are separately retained. |
| covenant | Full parent custody/release/acceptance obligations remain open. | P1 | No signature, certification, public release or acceptance-v1 claim. Preserve the complete original evidence ledger. |
| ledger | A valid native manifest stores overflowing unsigned seeds separately from legacy signed seed. | P2 | Both exact representations and recipe seed are checked. engine_seed is included among compared effective fields; no rounding through JavaScript Number. |
| ledger | Context bytes legitimately differ because a recorded working directory is part of provenance. | P2 | Keep originals unchanged; expose the comparison projection in declaration and helper. The normalized hash is only a comparison value and is never exported as an actual native context identity. |
| ledger | Helper serialization is fixture-specific. | P3 | Python canonical projection is asserted against every actual fixture context hash; unsupported numeric/schema drift fails rather than silently normalizing it. This is not a general RCTX implementation. |
| survey | Print-only remains an explicit browser-intent adaptation. | P2 | Keep the original 22 matching executions and 3 declared print-only adaptations; do not count dry-run manifest checks as districting runs. |
| survey | Fixing an assertion must not change user-visible algorithm, seed or tolerances. | P2 | Only test code/helper change. Actual native/WASM binaries, four-unit fixture, all flags, counts and original 60-second command budget remain. |
| survey | Broad practitioner/national/desktop support is not established here. | P1 | Keep original WP-01–WP-10 scope and all partial/not-run records. This gate closes only a stale test instrument. |
| contour | Physical context drift must remain detectable after provenance comparison. | P2 | Rehashed population mutation changes checked observables. Full populations, graph, units, subdivisions and any geometry remain compared. |
| contour | Literal district assignment equality must survive unchanged. | P2 | All native assignments, RPLAN body and exported WASM assignments compare without relabeling; a literal-label mutation is detected. |
| contour | Four synthetic units do not establish geography or demographic coverage. | P3 | Real-source NC/CA evidence is separate; no other vintage, resolution, VAP or VRA conclusion follows. |
| trench | Merely fixing the seed assertion exposed a second stale provenance assumption. | P2 | Preserve seed-only failed run. Record exact cwd derivation, verify actual bindings, and compare all remaining recipe fields. No unconditional hash deletion. |
| trench | A missing helper import and a negative control rejected earlier at certificate validation. | P3 | Preserve failed r2 script/log and control preimages. Corrected focused gate passes. Negative tests name their actual rejection layer; none silently becomes a positive. |
| trench | Full-build failure must not trigger in-place repair or evidence deletion. | P3 | Frozen R15 remains unchanged, both guards failed closed, main/captured stability passed. Adopt only guarded source union; retain all original failures. |

Roles reviewed: 6. P1 blockers: 3; P2 issues: 11; P3 notes: 4. Verdict: NEEDS-WORK for full delivery; focused correction is suitable for guarded integration. Consensus: benchmark/covenant/ledger require checked, explicit provenance comparison; benchmark/covenant/survey reject a full-parity claim.

Amendments: (1) integrate the exact reviewed test/helper with current source pins, retaining raw manifests; (2) run maintained checks on adopted sources and preserve the original full-build failure; (3) keep complete builders, national/lifecycle/source-authentication acceptance and ordinary actual-CLI export-byte parity open.
