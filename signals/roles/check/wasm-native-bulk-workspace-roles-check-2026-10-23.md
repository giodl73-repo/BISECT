---
skill: roles-check
topic: wasm-native-bulk-workspace
date: 2026-10-23
depth: standard
roles_used: [meridian, benchmark, covenant, ledger, vault, trench, survey]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 18
p3_open_count: 18
verdict: APPROVED-WITH-CONDITIONS
---

# Native bulk workspace implementation review

Artifact: [contract](../../../docs/specs/wasm-native-bulk-workspace.md), linked
implementation and tests. One agent applied seven installed lenses, not
independent reviewers or human acceptance. The profiles and verify questions
were read in the preceding plan review. This is bounded local implementation
approval with conditions, not full parity, certification or release approval.

## Selection

The [role catalog](../../../.roles/ROLE.md) supplies these perspectives:
MERIDIAN (native algorithm/allocation semantics), BENCHMARK (actual CLI/failure
evidence), COVENANT (source/binary custody), LEDGER (project/receipt meanings),
VAULT (live root/review authority), TRENCH (ordering/failure guards), and SURVEY
(an explicit practitioner flow). No new legal or election certification claim
is made; those reviews remain required for such changes.

## Findings

Three findings per lens; closed-scoped P2 findings have passing regressions.

| ID | Lens | Finding | Severity | Section | Disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | A canonical-only completion scan misses historical code folders and incomplete-marker precedence. | P2 | Completion | Closed-scoped: observe both names and call the native predicate; actual CLI historical/incomplete cases match. |
| M2 | MERIDIAN | Both synthetic states allocate one district; assignment agreement does not establish multi-district search/backend equivalence. | P3 | WP-02/08 | Keep limits explicit and real-data/method rows open. |
| M3 | MERIDIAN | States does not inherit Run reset; reprocess is distinct from reset. | P3 | Native configuration | Tests cover these distinctions; other mode combinations need their own rows. |
| B1 | BENCHMARK | Shared-adapter tests alone cannot prove native filesystem orchestration. | P3 | Evidence | Six actual CLI invocations cover this scope; broader failures/scale remain open. |
| B2 | BENCHMARK | Failure/cancel after removal can precede any partition. | P3 | Failure ordering | Completed/interrupted records and no-job-after-cancel checks pass; broader faults remain open. |
| B3 | BENCHMARK | New modules need standard registration and final-package tests. | P3 | Packaging | Both standard builders register checks; staged browser/asset gates pass. Full standard suites/C matrix were not rerun here. |
| C1 | COVENANT | Mutable latest records do not provide stable evidence custody. | P3 | Custody | An 85-file owned snapshot binds raw outputs/source/binaries/package; formal release acceptance remains open. |
| C2 | COVENANT | Marker presence is not valid contents or authentic historical generation. | P3 | Completion | Claim only the native skip decision; composed Verify stays required. |
| C3 | COVENANT | An action receipt proves neither installed new outputs nor rollback/admissibility. | P3 | Receipts | Keep installation separate and broader provenance/release obligations open. |
| L1 | LEDGER | Workspace metadata must not break old compute-only checkpoints. | P3 | Checkpoints | Existing Run/States replay/Resume regressions pass; broader migration remains open. |
| L2 | LEDGER | Saved workspace verification cannot simply repeat reset. | P3 | WP-06 | Disable this new replay/Resume scope until non-destructive verification and fresh authority are composed. |
| L3 | LEDGER | Growing multi-year checkpoints could duplicate prior-year actions. | P3 | Year receipts | Filter by year; test action counts 1/2/2. Cross-year recovery remains open. |
| V1 | VAULT | Imported public reviews/receipts cannot grant live folder authority. | P3 | Authority | Private one-use objects and exact identity checks pass; privileged same-origin JS is outside this boundary. |
| V2 | VAULT | Root contents can change after review or during work. | P3 | Target checks | Re-observe before scheduling, per target and at year end; no lock/atomicity claim. |
| V3 | VAULT | OPFS/injected picker does not prove OS permission UX. | P3 | Compatibility | Keep exact host scope and visible failures; broader host tests remain open. |
| T1 | TRENCH | Run reset could be dispatched as intent without performing its actual filesystem effect. | P2 | Queue boundary | Closed-scoped: require current reviewed workspace before scheduling; skipped targets never reset. |
| T2 | TRENCH | Failure before the first checkpoint previewed undefined and stranded busy controls. | P2 | UI finalization | Closed-scoped: no-result preview is empty. Browser refuses, then successfully reviews/runs; PP-29 retains broader cleanup risk. |
| T3 | TRENCH | Target preflight failures can falsely look like execution. | P3 | Failure rows | Pre-dispatch failures explicitly record execution_performed=false; broader worker/source classifications remain open. |
| S1 | SURVEY | Operators need skipped/scheduled/blocked targets visible before consent. | P3 | UI | Read-only panel and explicit confirmation pass; larger-workspace usability remains open. |
| S2 | SURVEY | Portable browsers cannot necessarily select writable folders. | P3 | Fallback | Explain unsupported action; retain compute-only/download paths without silent reset fallback. |
| S3 | SURVEY | This does not finish acquisition, automatic installation, Verify or national capacity. | P3 | WP-03–10 | Keep remaining contracts partial/open and continue the full plan. |

## Synthesis and amendments

Seven lenses: **0 P1, 3 P2 closed-scoped, 18 P3 open**.
Verdict: **APPROVED-WITH-CONDITIONS for the local scoped implementation**.
MERIDIAN/BENCHMARK/TRENCH agree native completion must precede reset.
VAULT/LEDGER/COVENANT agree a saved record is evidence, not live authority or
proof of installed outputs.

Three amendments implemented and tested:

1. Live canonical/historical completion and separate reset/reprocess semantics
   before dispatch of the original source jobs.
2. Private reviewed authority, stale-target refusal, explicit action export,
   retained cancellation/source-failure outcomes and year-scoped receipts.
3. Safe absent-result UI finalization, registered gates, final-package browser
   regressions and a stable owned source/raw-result snapshot.

## Baseline and evidence limits

HEAD `383db340235e21d752a35610aab28e2948b73e57`, dirty working tree.
Package `dist/wasm-bulk-workspace`, offline version
`d658ef3a3376e1e38fe729cc890a9bcb95e9979f1f18651f6dd63c9cefea80c5`.
Engine SHA-256 `0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`,
25,118,842 bytes. Owned native executable SHA-256:
`ce0d94e967adf043862a64060e841fc7c998e2647b140fa55c24686d2ffee4ff`.

Six actual CLI invocations cover baseline, reprocess, incomplete target,
historical completion, States without reset and all three vintages. Five exit 0;
the incomplete case exits 1 while the other state succeeds. Accepted assignments
match. Synthetic VT/WY each allocate one district: this establishes orchestration,
not real geography, multi-district search or all-state capacity.

Fresh packaged tests cover standalone reset, selected-State target actions, bulk
workspace, input workspace, single/batch recovery and optional C worker. Asset
gates checked 639 identities, 539 shell files, 50 state/year packs, 265 toolkit
files and 1,063 dependencies. Existing Run queue (178 whole replies) and States
queue regressions pass. No new Rust WASM or full legacy C matrix was built/run.

The 85-file stable snapshot is
`target/native-bulk-workspace/package-snapshots/wasm-bulk-workspace/snapshot.json`.
It preserves source, owned binaries, raw CLI outcomes, projects, browser action
receipts and package evidence. The generator verifies identities/staged module
equality and refuses replacement. This is not an acceptance-v1 bundle or source
authenticity proof. Nothing was committed, published or deployed.
