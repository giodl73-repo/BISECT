---
skill: roles-check
topic: wasm-state-acquisition-recovery
date: 2026-10-23
depth: standard
roles_used: [contour, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 18
p3_open_count: 18
verdict: APPROVED-WITH-CONDITIONS
---

# Paired-source recovery review

Artifact: [contract](../../../docs/specs/wasm-state-acquisition-recovery.md), its
implementation and tests. One agent applied seven installed lenses, not
independent reviewers or human approval. CONTOUR was read for this source-focused
slice; the other profiles and verify questions were read during preceding plan
reviews. This is scoped local implementation approval with conditions, not full
Fetch/Verify parity, source authenticity, certification or release approval.

## Selection

From the [catalog](../../../.roles/ROLE.md): CONTOUR checks Census scope, byte and
derived-source lineage; BENCHMARK checks independent references and regressions;
COVENANT checks custody and identity claims; LEDGER checks versioned checkpoints;
VAULT checks imported authority and hostile input; TRENCH checks stale/partial
failure behavior; SURVEY checks explicit usable recovery. No legal/statistical
or election certification claim was introduced.

## Findings

| ID | Lens | Finding | Severity | Section | Disposition |
|---|---|---|---|---|---|
| CN1 | CONTOUR | A matching archive receipt alone cannot establish that the queued derived sources came from those bytes. | P2 | Saved verification | Closed-scoped: hash/decode original ZIPs and rebuild the exact job; modified archives and rehashed-but-changed derived recipes refuse. |
| CN2 | CONTOUR | Synthetic RI/VT fixtures are not published geography/population evidence. | P3 | Data scope | Keep two-unit/population-18 fixture labels; real-data/provider claims remain open. |
| CN3 | CONTOUR | Other vintages, resolutions and alignment contracts are not established by this 2020 tract fixture. | P3 | WP-05/08 | Expand declared rows rather than extrapolate. |
| B1 | BENCHMARK | Six native adapter comparisons do not execute native CLI Fetch downloading. | P3 | Native reference | Scope them to subsequent source Run; full CLI filesystem/release/delay equivalence remains open. |
| B2 | BENCHMARK | Both native-State and configured YAML/form reconstruction need actual browser coverage. | P3 | Compatibility | Both modes pass; prepared templates, larger profiles and other combinations remain open. |
| B3 | BENCHMARK | Generated target files alone are not reproducible fixtures. | P3 | Fixture custody | Use tracked native fixture source, tracked ZIP/PL text, owned executables and original replies. External browser/toolchain obtainability remains open. |
| C1 | COVENANT | Internally consistent bytes are not authenticated provider data or historical execution. | P3 | Verification meaning | Mark authenticity false and partition/network not performed; keep full custody/replay work open. |
| C2 | COVENANT | Fresh native inspection identity differs from a historical producer identity. | P3 | Runtime identity | Browser verification binds its loaded artifact; no historical binary claim. Broader producer custody remains open. |
| C3 | COVENANT | A local snapshot is not an acceptance-v1/public release bundle. | P3 | Package | Preserve 113 owned files and exact package identities; formal release/notices remain open. |
| L1 | LEDGER | New recovery metadata must not grant authority to legacy v1 results. | P3 | Versioning | New v2 flow; old jobs remain readable/inert. Broader migration remains open. |
| L2 | LEDGER | A generated timestamp could change after Open if it was never saved with the configured form. | P3 | Context retention | Persist the effective form; static configured Save/Open/rebuild passes. Broader generated-date and form variants remain open. |
| L3 | LEDGER | Saved verification markers must not replace a live proof. | P3 | Record semantics | Serialized markers remain data; private proof is created only after fresh verification. Other tools' receipt contracts remain separately reviewed. |
| V1 | VAULT | Imported checkpoints could otherwise enable automatic network continuation. | P2 | Authority | Closed-scoped: private live proofs; Open does nothing; copied/imported markers cannot enable Resume. Browser asserts zero provider/partition work during verification. |
| V2 | VAULT | The same-origin privileged-script threat is outside the imported-data capability boundary. | P3 | Threat model | State the limit; do not describe hashes as malware resistance. |
| V3 | VAULT | Archive allocation/path/expansion bounds and malformed saved metadata remain important. | P3 | Input validation | Reuse bounded decoder/schema/JSON/queue validation; new hostile archive and broader profile coverage remain required. |
| T1 | TRENCH | Repeating an entire failed batch reacquires and can replace already accepted source pairs. | P2 | Resume | Closed-scoped: skip proven queued rows, retry failed/unfinished pairs only, retain exact prior job. Native/configured tests check two new requests only. |
| T2 | TRENCH | Cancellation around staging/hash/commit can publish a late job or proof. | P3 | Cancellation | Check before commit and proof return; real browser cancellation retains prior jobs. More precise fault/interleaving probes remain open. |
| T3 | TRENCH | The full retained-source snapshot can conservatively invalidate recovery on unrelated file edits. | P3 | Context binding | Document the boundary; any future narrowing must trace actual consumers and keep stale-source refusal. |
| S1 | SURVEY | Source checking, unfinished acquisition and engine Run are different user actions. | P3 | UI | Separate controls; actual browser tests assert no Run during checking/acquisition. Larger workflow usability remains open. |
| S2 | SURVEY | Streaming without retained raw archives cannot use this recovery proof. | P3 | Fallback | Keep streaming's existing path and disclose this exclusion; composed disk recovery remains open. |
| S3 | SURVEY | This is not all-source, all-state capacity or full Fetch/Verify support. | P3 | WP-01/05/06/08 | Keep parent records partial and remaining native consumer/provider/matrix work explicit. |

## Synthesis and amendments

Seven lenses: **0 P1, 3 P2 closed-scoped, 18 P3 open**.
Verdict: **APPROVED-WITH-CONDITIONS for the scoped local implementation**.
CONTOUR/COVENANT/LEDGER agree on raw-to-derived lineage without authenticity
overclaim. VAULT/TRENCH/SURVEY agree that only explicit fresh verification may
enable resumed acquisition, and that already queued work must remain exact.

Three amendments implemented and checked:

1. Shared decoded-archive job construction with exact hash/recipe verification.
2. Private checkpoint continuity, remaining-pair-only Resume and separate inert
   Open / explicit source check / explicit Resume / explicit Run actions.
3. Native/configured browser coverage, retained form context, registered build
   gates and an owned source/raw-output snapshot.

## Baseline and limits

HEAD `383db340235e21d752a35610aab28e2948b73e57`, dirty working tree.
Package `dist/wasm-acquisition-recovery-validated`, offline version
`190b4cfba6c5de04bb4799e5e9da53bdac1b6c688a35b93f6f9d63b71490256a`.
WASM SHA-256 `0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`,
25,118,842 bytes. Asset closure: 641 identities, 541 shell assets, 50 state/year
packs, 266 toolkit files and 1,073 dependency edges.

Fresh tests cover native and configured recovery in Node and actual packaged
workers, controlled provider cancellation/Resume, and six subsequent native
adapter replies with both originals retained. Only target_pointer_width
presentation fields are allowed to differ. Existing paired acquisition (50
synthetic states), acquisition/streaming browser, State target, bulk workspace,
input workspace, single/batch recovery and optional C worker regressions pass.
The full standard builds, full legacy C matrix, live providers and real-state
capacity matrix were not rerun here; no Rust WASM was rebuilt.

The stable 113-file snapshot is
`target/acquisition-recovery/package-snapshots/wasm-acquisition-recovery-validated/snapshot.json`.
It retains owned fixture/native executables, original ZIPs, both native replies,
source/checkpoints/projects, browser evidence and package/runtime identities.
The generator checks identities/staged bytes and refuses replacement. Nothing
was committed, published or deployed.
