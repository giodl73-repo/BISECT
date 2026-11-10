---
skill: roles-check
topic: wasm-desktop-parity-plan-acceptance
date: 2026-10-23
roles_used: [MERIDIAN, BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH, SURVEY]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# WASM parity plan: acceptance-checkpoint review

Artifact: [desktop-to-WASM parity plan](../../../docs/specs/wasm-desktop-parity-plan.md).
Type: implementation plan. Domains: engine parity, browser lifecycle, provenance,
interchange, hostile imports, test adequacy and practitioner workflow.

The coding agent applied seven repository role lenses. These are not seven
independent reviewers, maintainer sign-off, runtime acceptance or legal advice.
All sixteen installed roles were inventoried. These seven directly cover this
technical checkpoint; no new legal, partisan-effect, certification or ballot-data
claim warrants expanding this pass to the other role domains.

## Roles and findings

P1 blocks shipment; P2 must be addressed for the scoped milestone; P3 should be
addressed within the named work item. Findings concern the plan and its closure
criteria; a missing acceptance requirement is not proof of an exploitable defect.

### MERIDIAN — computation and graph identity

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| M1 | Source-alignment language does not specify binding the regenerated graph to the graph actually replayed; self-consistent receipts alone are insufficient. | P2 | WP-06 | Require typed native graph equality, including exact populations/edges; mutation with recomputed receipts must fail. Added to immediate gate 2; packaged proof remains open. |
| M2 | An assignment similarity threshold can hide non-identical outcomes if the summary says only matched. | P3 | Evidence contract / WP-06 | Default to exact comparison, disclose an explicit different threshold and retain labels/original assignments. Added to gate 1. |
| M3 | Scoped C results cannot generalize to all methods or C runtimes. | P3 | WP-02 / WP-08 | Preserve named runtime, method/count-aware preflight and per-row outcomes. Existing requirements retained; broader matrix remains open. |

### BENCHMARK — executable and reproducible acceptance

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| B1 | The ordinary ledger gate currently fails on stale `web/lab/native-workspace-source.js` evidence. The plan cannot describe its current source as acceptance-ready. | P2 | WP-01 / next checkpoint | Rerun affected checks, reconcile active pins and bind new evidence without altering historical snapshots. Added gate 4; failure remains open. |
| B2 | The pending Verify Node test is not registered in the inspected standard/preview builders or dual-build runner; no final packaged browser acceptance was established in this review. | P2 | WP-06 / WP-10 | Register the workflow, test both actual engine variants and the final worker/package, retain raw failures/successes. Added gates 3–5; implementation obligation open. |
| B3 | Real-data matrix requirements are detailed, but the plan still lacks fixed runnable small/medium/large rows. | P3 | WP-08 | Predeclare immutable inputs, native expectations, method/backend and API/runtime before expanding claims. Carried forward explicitly after Verify. |

### COVENANT — custody and producer evidence

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| C1 | A CLI `--dry-run` precheck record cannot substantiate complete native CLI replay orchestration. | P3 | WP-06 / evidence contract | Name precisely what ran; obtain the corresponding reference for any broader claim. Added to immediate-delivery baseline. |
| C2 | Loaded WASM identity, a saved producer identity and authenticated historical execution are separate facts. | P3 | WP-06 | Report them separately and preserve missing producer evidence even after a replay match. Added gates 1–2. |
| C3 | New mutable target outputs cannot amend a historically tested package. | P3 | WP-10 | Bind new bytes and dirty-source digests to a new immutable snapshot; retain old evidence unchanged. Existing contract reinforced in gates 4–5. |

### LEDGER — formats and exact values

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| L1 | Composing native indexed assignments with portable unit-ID assignments needs an explicit convention, especially for numeric custom IDs; JavaScript parsing can also round native integers. | P2 | WP-06 / interchange | Preserve raw manifest bytes, use exact native integer handling, disclose mapping convention and refuse ambiguity unless resolved by versioned metadata. Added gate 2; acceptance cases required. |
| L2 | A package-level status can obscure which missing artifact prevented source/context/replay verification. | P3 | WP-06 | Export versioned per-stage outcomes, keeping missing, failed and not-run separate. Added gate 1. |
| L3 | General selected-root discovery and a single-target ZIP/file-set workflow are different interoperability scopes. | P3 | WP-03 / WP-06 | Advertise the exact selected-target formats tested; keep broader discovery and old readers under their separate contracts. Added gates 1 and 4. |

### VAULT — imported authority and security boundaries

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| V1 | A passing Verify record must not restore filesystem or destructive-action authority during later workspace Resume. | P3 | WP-03 / WP-06 | Keep passive evidence separate from fresh live review/grants. Added gate 3 and post-Verify ordering. |
| V2 | Output ZIP support changes an existing decoder consumer and needs the same bounded/hostile-input regression coverage. | P3 | Workspace safety / WP-06 | Preserve traversal, alias, duplicate, CRC and expansion limits; rerun existing archive tests with the new layout. Added gate 4; no newly passing hostile-input run claimed. |
| V3 | The latest selected archive read or worker response can arrive after Open or operation change. | P3 | Workspace safety | Prove retirement/freshness in the real browser rather than relying only on persisted input validation. Added gate 3. |

### TRENCH — failure structure and traceability

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| T1 | Planning criteria do not explicitly prevent a successful aggregate replay from concealing incomplete verification stages. | P2 | WP-06 | Require separate stage status and a regression where replay matches while raw source evidence is absent. Added gate 1; record PP-31 as OPEN until packaged regression acceptance. |
| T2 | Closing evidence drift by refreshing hashes without rerunning affected behavior would defeat the stale-evidence guard. | P3 | WP-01 / release gates | Keep failure visible, inspect active changes and regenerate evidence; do not repin historical packages. Added gate 4. Existing PP-25 remains open. |
| T3 | Verification/recovery composition must preserve accepted results while refusing stale or failed follow-ups. | P3 | Workspace safety / WP-06 | Exercise edits, Save/Open, missing sidecars, failed replay and current-result downloads with the worker. Added gate 3; preserve existing recovery regressions. |

### SURVEY — clear actions and finishable milestones

| ID | Finding | Severity | Section | Recommendation |
|---|---|---|---|---|
| S1 | The checkpoint still calls bulk scheduling unsupported in the target-action exclusions despite the later scoped delivery; it also does not acknowledge pending Verify implementation. | P2 | Next acceptance checkpoint | Correct the scope statement and replace generic next work with five ordered Verify acceptance gates. Amended; implementation gates remain open. |
| S2 | The new operation must avoid multiple similarity controls or controls whose values are ignored by the selected workflow. | P3 | WP-06 UI | One meaningful threshold, explicit Inspect/Replay choices, visible exact seed and clear missing evidence. Added gate 3; browser acceptance open. |
| S3 | Broad inventory work can indefinitely postpone a complete usable journey. | P3 | Recommended order | Finish one selected-output Inspect/Replay package, then verified continuation; continue source reconciliation and matrix definition alongside it. Added bounded milestone and follow-on order. |

## Synthesis

Roles reviewed: **7**. Findings: **21 — 0 P1, 6 P2, 15 P3**.

Verdict: **APPROVED-WITH-CONDITIONS for the amended plan**. Documentation
amendments address the missing planning requirements. B1/B2 and the runtime
obligations behind M1/L1 remain unaccepted; this verdict approves neither the
pending Verify implementation nor a release/full-parity claim.

Top finding: close the changed-source-to-final-package evidence chain before
advancing the next workflow. BENCHMARK/COVENANT/TRENCH agree that new source and
mutable local results cannot inherit historical package acceptance.
MERIDIAN/LEDGER agree that graph, integer and assignment identities must survive
composition. VAULT/SURVEY agree that Inspect, Replay and later filesystem actions
need distinct, understandable authority and results.

Three amendments applied:

1. Acknowledge pending Verify source/local records and current stale evidence;
   remove the obsolete bulk-scheduling exclusion while retaining its open scopes.
2. Specify stage-level inspection/replay, exact graph/integer/index binding,
   inert Open and passive evidence that cannot authorize writes or reset.
3. Define five ordered acceptance gates, with builder/dual-build/browser checks
   and a new package/snapshot; retain reconciliation, real-data and release gaps.

## Evidence inspected and checks performed

Baseline HEAD: `383db340235e21d752a35610aab28e2948b73e57`; dirty working tree.
Read the plan, ledger, prior handoff review, selected role profiles, pending Verify
module/test, local evidence record and relevant builder registrations. Existing
`target/native-output-verification/evidence.json` reports 12 cases, two native CLI
dry-runs and default WASM SHA-256
`60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`.
Those records were inspected, not rerun or independently validated by this review.

Executed during this review:

- `python scripts/web/wasm_parity_ledger.py`: exit 1, `bisect.State: Stale evidence: web/lab/native-workspace-source.js`.
- `python scripts/web/wasm_parity_ledger.py --full-parity`: exit 1, `bisect.Run: Parent evidence has not been verified`.
- Both commands first reported matching 69 capability declarations, 240 engine
  fields and 152 native field traces; these counts establish no executable parity.

The companion [baseline](wasm-desktop-parity-plan-acceptance-baseline-2026-10-23.json)
pins inspected files and this amended plan. Hashes identify bytes, not authorship
or build provenance. Only planning/review/pitfall documentation was changed in
this review. No runtime suite, build, commit, deployment or evidence promotion
was performed. Prior scoped reviews remain historical and retain their conditions.
