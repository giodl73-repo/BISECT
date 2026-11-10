---
skill: roles-check
topic: wasm-desktop-parity-plan-practitioner-handoff
date: 2026-10-23
depth: standard
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CANVASS, TALLY, TRENCH]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 21
verdict: APPROVED-WITH-CONDITIONS
---

# Practitioner-handoff parity plan review

Artifact: [desktop-to-WASM plan](../../../docs/specs/wasm-desktop-parity-plan.md).
Scope: current implementation handoff and acceptance requirements. One agent applied
eight installed role perspectives; these are not independent reviewers or human
signatories. This is planning approval for continued implementation, not runtime,
release, deployment, legal or full-parity approval. Earlier reviews remain historical
records; their conditions close only through their own scoped evidence.

## Selected perspectives

The [role catalog](../../../.roles/ROLE.md) governs selection. Profiles read:

| Role | Reason |
|---|---|
| [SURVEY](../../../.roles/survey.md) | A usable next practitioner journey and portable fallback. |
| [BENCHMARK](../../../.roles/benchmark.md) | Native references, meaningful failure tests and final browser acceptance. |
| [COVENANT](../../../.roles/covenant.md) | Source/build/package identity and evidence custody. |
| [LEDGER](../../../.roles/ledger.md) | Native artifact semantics, interoperability and stable acceptance records. |
| [VAULT](../../../.roles/vault.md) | Imported authority and disclosure risks in saved/exported practitioner records. |
| [CANVASS](../../../.roles/canvass.md) | Official decisions versus computed verification and lawful amendments. |
| [TALLY](../../../.roles/tally.md) | Preservation of count-source and transcript semantics. |
| [TRENCH](../../../.roles/trench.md) | Stale status, interrupted writes and structural closure requirements. |

No new algorithm, legal, statistical or jurisdictional policy assessment is made.
Their existing plan requirements remain applicable when those claims change.

## Findings

P2 must be amended before following this handoff. Closed-plan means wording was
corrected, not that implementation passed. P3 denotes retained acceptance work,
not a newly demonstrated runtime bug. Three findings per role follow.

| ID | Lens | Finding | Severity | Section | Recommendation / status |
|---|---|---|---|---|---|
| S1 | SURVEY | The handoff still requested the practitioner inventory although a checked source inventory now exists, leaving the next implementable action ambiguous. | P2 | Current execution handoff | Closed-plan: link the inventory and rank repository application, PDF delivery and transcript application, with concrete completion boundaries. |
| S2 | SURVEY | A merge ZIP does not complete the selected-repository practitioner journey. Pending writer code has no wired workbench action. | P3 | WP-03/07, output 1 | Open: provide review/apply/cancel/explicit continuation and preserved downloads, then demonstrate the actual packaged UI. |
| S3 | SURVEY | Directory-picker support and portable download support are different usable workflows. | P3 | Compatibility / output 1 | Open: record separate feature-based outcomes; an injected OPFS handle is not evidence of native OS picker behavior. |
| B1 | BENCHMARK | Source inventory validation establishes current mappings, not execution equivalence. | P3 | WP-01/07 | Open: integrate the validator into applicable gates and bind scoped runtime evidence to existing ledger children. |
| B2 | BENCHMARK | A native host calling the merge adapter cannot prove native CLI consumption of installed repository files. | P3 | Output 1 | Open: execute the native label verification CLI on owned installed outputs; include a tampered-chain failure. This witness does not prove the CLI generated the supplied artifacts. |
| B3 | BENCHMARK | Medium/large matching native failures are valuable comparisons but cannot establish successful capacity. | P3 | WP-08 | Open: preserve expected-failure classification; add predeclared successful cases and host/capacity rows before expanding support. |
| C1 | COVENANT | The handoff called the older continuation package the latest despite a later archive package and immutable evidence contract. | P2 | Current execution handoff | Closed-plan: identify the archive v2 package as the latest scoped local baseline and preserve earlier packages as historical. |
| C2 | COVENANT | Pending source edits cannot inherit the last package's runtime acceptance. | P3 | Output 1 / WP-10 | Open: bind a new immutable snapshot to final staged bytes, dirty-source digests, native references and actual loaded runtime identities. |
| C3 | COVENANT | Correct read-back hashes are neither source authenticity nor official approval. | P3 | Completion / provenance | Open: preserve separate producer, source and loaded-runtime claims in summaries and receipts. |
| L1 | LEDGER | The source-type paragraph described a four-family browser allowlist after school/EIA acquisition had shipped locally. | P2 | Current execution handoff | Closed-plan: enumerate six families within their scoped contracts and retain All-selection and unimplemented spatial-join distinctions. |
| L2 | LEDGER | The new action inventory could become a competing status tracker alongside existing capability records. | P3 | WP-01/07 | Open: retain stable parent/child acceptance authority; use the action JSON for source reconciliation and ranking only. |
| L3 | LEDGER | Browser PDF/print can differ materially from wkhtmltopdf/pandoc output. | P3 | Output 2 | Open: predeclare renderer, exact HTML input and observable content/layout contract; printing UI alone is not a saved-PDF result. |
| V1 | VAULT | Imported projects and exported receipts may be mistaken for permission to write or continue. | P3 | Workspace / output 1 | Open: use fresh live selected-root review and private single-use authority; test forged/copied/stale records and inert Open. |
| V2 | VAULT | Evidence/error exports may contain private election records even when execution is local. | P3 | Workspace / output 3 | Open: retain explicit export and source-preserving local boundaries; avoid automatic publication or public telemetry of package contents. |
| V3 | VAULT | Unexpected partial bytes cannot safely be assumed to belong to the interrupted attempt. | P3 | Output 1 | Open: continue only exact re-observed original/intended states; require fresh conflict review for unknown bytes and recheck before writes. |
| A1 | CANVASS | Inventory wording includes certification/recount actions only where native exposes them; there is no general software certification authority. | P3 | WP-07 / output 3 | Open: preserve computed-check versus supplied official-status distinctions in labels, transcripts and summaries. |
| A2 | CANVASS | Lawfully amended records can change without indicating tampering. | P3 | Output 3 | Open: retain supplied status and amendment lineage; do not replace history with a fresh success badge. |
| A3 | CANVASS | Audit-statistic replay cannot establish ballot acceptance, eligibility or adjudication. | P3 | WP-07 | Open: native command boundaries must remain visible; add no invented official decision workflow. |
| Y1 | TALLY | Transcript installation must bind the exact selected count package rather than a similarly named folder. | P3 | Output 3 | Open: re-observe package identities before write and verify resulting bytes through the native consumer. |
| Y2 | TALLY | Successful import cannot justify silently changing contest IDs, reporting units or vote/ballot distinctions. | P3 | WP-07/08 | Open: preserve raw inputs/parser warnings and include relevant accounting edge cases when extending import/transcript scope. |
| Y3 | TALLY | Existing RCOUNT commands already have browser kernels; an inventory of six commands does not mean six missing engines. | P3 | WP-07 | Open: extend missing end-to-end actions and retain existing native-reference regressions rather than rewriting their kernels. |
| T1 | TRENCH | Registry/index commitment can advertise completion while earlier output writes are incomplete. | P3 | Output 1 | Open: stage index before registry, registry last, exact readback and interruption tests; no OS-atomic or native locking claim. |
| T2 | TRENCH | Cancellation after effects start cannot imply rollback, and retry must not overwrite unrecognized edits. | P3 | Workspace / output 1 | Open: retain passive interrupted receipts, distinguish refusal from effects, and explicitly review continuation. |
| T3 | TRENCH | Repeated plan refreshes can obscure open historical review conditions. | P3 | Closure / handoff | Open: carry condition IDs and scope; use PP-25 and PP-27 rather than bulk-closing them when a later package passes. |

## Synthesis

Roles reviewed: **8**. Findings: **24** — **0 P1**, **3 P2** amended in the
plan, **21 P3** retained implementation/release obligations.

Verdict: **APPROVED-WITH-CONDITIONS for continued implementation**.
The top finding is that next work must be an actual missing practitioner journey,
not repeated inventory work. SURVEY/LEDGER/TRENCH agree on that boundary;
BENCHMARK/COVENANT require a native consumer witness and new final-package evidence.
CANVASS/TALLY/VAULT require that transcript and verification work preserve supplied
status, source semantics and private selected-root authority.

Three amendments applied:

1. Updated the handoff to the checked inventory and its next three ranked actions;
   scoped repository installation is first, with independent/native and browser gates.
2. Corrected package/allowlist drift while keeping historical baselines, shared native
   limitations, expected failures and pending source work explicitly separate.
3. Made interruption/conflict handling, registry-last/readback requirements and
   ledger/builder integration explicit before accepting the next slice.

## Checks and limits

Rerun during this review:

- `python scripts/web/wasm_practitioner_inventory.py`: 69 declarations, 17 action
  groups and 61 source pins validate; explicitly no runtime/full-parity claim.
- `python scripts/web/wasm_parity_ledger.py`: 309 structured records validate,
  including 240 option declarations and 152 native field traces. Unresolved allowed.
- `python scripts/web/wasm_parity_ledger.py --full-parity`: expected rejection at
  `bisect.Run: Parent evidence has not been verified`. Full parity remains open.

Source spot checks: native report PDF subprocesses, RCOUNT optional transcript
write, label-chain verification, browser archive allowlist and absence of a wired
repository-install action. Existing evidence contracts were read; runtime suites,
engine builds, OS permissions and final-package acceptance were not rerun.
No new failure class is asserted: existing PP-25 (false parity closure), PP-27
(stale selected-root authority) and the plan's interrupted-write contract cover
these obligations; no pitfall is marked SOLVED by this review.

Only the plan and this review were edited. No runtime code, inventory pins,
historical reports or evidence snapshots were changed; pre-existing dirty work
remains intact. Nothing was committed, pushed or deployed.

## Reviewed bytes

HEAD: `383db340235e21d752a35610aab28e2948b73e57`, dirty working tree. Hashes identify inspected bytes, not
build reproducibility, authenticity or execution acceptance.

| Artifact | SHA-256 |
|---|---|
| `docs\specs\wasm-desktop-parity-plan.md` | `dd2b2f97dbdb800db7edb083f08eed3f75562ffd4859d4628b086c94fa8e1a1f` |
| `docs/specs/wasm-practitioner-actions.json` | `1d3eebcbc34a0e732a5429cbaf2e84f4864b38d416c241e58d477c9e7b49e19f` |
| `scripts/web/wasm_practitioner_inventory.py` | `fd7590b2ae1b3691952a88a1eab35c535e74c9f08623418cb77a7ad939fa045d` |
| `docs/specs/wasm-fetch-archive-acquisition.md` | `7ea24c62792a0143568d0c66bdc9389f115b8ee1be9d74b02026c8202df63b6e` |
| `web/lab/label-repository-install.js` | `fccca5ea9d662fee30fb3f7ba1f94066c98a7cd99102f89f8b62c4b4a829b5a1` |
