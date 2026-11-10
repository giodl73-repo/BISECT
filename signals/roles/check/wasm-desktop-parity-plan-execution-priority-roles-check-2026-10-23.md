---
skill: roles-check
topic: wasm-desktop-parity-plan-execution-priority
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CANVASS, TALLY, TRENCH]
p1_count: 0
p2_count: 5
p3_count: 19
verdict: APPROVED-WITH-CONDITIONS
---

# Execution-priority review of the WASM parity plan

Artifact: [desktop-to-WASM parity plan](../../../docs/specs/wasm-desktop-parity-plan.md).
Type: implementation and acceptance plan. Depth: standard.

The coding agent applied eight installed repository role perspectives. These are
not independent reviewers or maintainer signatories. This review inspects the
plan, practitioner ranking, native source anchors and recorded package outcomes;
it does not rerun runtime tests, accept implementation, establish legal validity
or authorize deployment. Earlier implementation reviews retain their own scopes.

## Selection and evidence

SURVEY examines the next usable workflow; BENCHMARK compares claimed behavior
with test authority; COVENANT examines artifact identity and retained evidence;
LEDGER examines native formats and inventory consistency; VAULT examines imported
authority and disclosure; CANVASS examines supplied election status and lineage;
TALLY examines transcript/accounting semantics; TRENCH examines interruption,
storage and regression closure. Algorithm and jurisdictional rules are unchanged;
this review makes no new algorithmic or legal assessment.

Inspected the eight corresponding `.roles/*.md` profiles and `.roles/ROLE.md`.
Supporting sources include the practitioner inventory and validator,
`crates/bisect-cli/src/analyze_label.rs::generate_label_report`,
`crates/rcount-cli/src/main.rs`'s `write_verification_transcript` call,
the label installation contract, and the pending PDF delivery contract.

The recorded PDF candidate is `dist/wasm-report-pdf-validated-v2`, with build
record `target/report-pdf-delivery/build-1792797720145821400/package-evidence.json`
(SHA-256 `c0880fefd7662a6405ed613391c8ee84e3df7a3ed9bdb4464b571b27b8a6e3c7`).
Its recorded stage exits pass; immutable snapshot closure and implementation
review remain pending. That observation does not turn this plan review into
runtime acceptance. Verified reuse of engines is not a fresh engine build.

## Findings

P1 blocks shipment, P2 must be addressed for the applicable milestone, and P3
should be addressed or explicitly carried. “Planning addressed” means an explicit
obligation or corrected handoff, not completed implementation or evidence.

### SURVEY — operational priorities

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | The handoff still schedules delivered label installation and describes PDF work without its newer package-check state. | P2 | Current execution handoff | Planning addressed: distinguish completed reviewed label delivery, tested PDF candidate and pending PDF evidence closure; close that bundle before another feature. |
| S2 | Prioritize functional parity and preserve accepted results after follow-up failure. These are appropriate operational requirements. | P3 | Objective / completion | Retain the source-to-result user journey and downloadable accepted artifacts in each new slice. |
| S3 | The plan properly separates portable downloads from selected-folder installation; injected pickers do not establish actual OS permission behavior. | P3 | Compatibility / handoff | Publish separate observed outcomes; retain unsupported/not-run host cases. |

### BENCHMARK — behavior and verification

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Browser per-state commission reports differ from the actual native LabelReport label/year HTML/JSON summary. Native consumption of installed files does not prove native report generation. | P2 | WP-07 / handoff | Planning addressed: add the exact native summary as ranked slice 2 with actual CLI generation/failure comparisons; preserve commission reporting as an extension. Implementation remains missing. |
| B2 | Exact assignments, narrowly named comparison allowances and explicit matching native failures are sound acceptance rules. | P3 | Evidence rules / WP-08 | Preserve the contract before runs; matching failure must never count as supported successful capacity. |
| B3 | PDF automation and GUI print-call checks observe different behavior, and the plan must retain that distinction. | P3 | Immediate closure task | Addressed in planning: retain genuine rendered PDF checks separately from instrumented GUI calls; no OS save-dialog completion inference. |

### COVENANT — evidence and provenance

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Passing package logs alone do not close the PDF candidate's immutable evidence, review and support-registration obligations. | P2 | Current execution handoff / WP-10 | Planning addressed: make evidence closure the immediate task and retain the earlier reviewed package as accepted only within its scope. Implementation acceptance remains pending. |
| C2 | Native executable identity, loaded WASM identity, historical producer claims and source authenticity are correctly separated. | P3 | Evidence rules / WP-06 | Carry those distinct identities into the PDF bundle and transcript application; hashes confer no authenticity by themselves. |
| C3 | The candidate reuses verified engine bytes; a successful browser run cannot establish source-to-binary build reproducibility. | P3 | Current execution handoff | Explicitly disclosed in the amended plan; preserve original engine build records and dirty-source identities. |

### LEDGER — native formats and consistent contracts

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | The plan's ranked table disagrees with the current practitioner inventory, and its source-pin count is stale. | P2 | Current execution handoff | Planning addressed: use transcript application, native label summary, then label rename; update the pin count to 73. Keep the ledger as acceptance authority. |
| L2 | Native LabelReport accepts HTML/JSON and rejects PDF; browser PDF support must not be described as native LabelReport equivalence. | P3 | Ranked slice 2 | Explicitly stated in the amended table; preserve native filenames, index/registry behavior and report variants. |
| L3 | Versioned formats, old read fixtures and original-byte versus canonical/semantic identities are appropriately required. | P3 | Compatibility | Apply these rules to transcripts and label rename/import; do not normalize away native differences. |

### VAULT — authority and privacy

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Inert Open, live selected-root authority and invalidation of pending work establish the right browser boundaries. | P3 | Workspace safety | Retain those guards in transcript/rename application, including project changes while reads or writes are pending. |
| V2 | A selected PDF hash is only an observation of selected bytes and a user-declared association. | P3 | Immediate closure task | Explicitly retain the inability to observe successful save or verify selected PDF content in the UI receipt. |
| V3 | Public evidence and local project retention have different disclosure consequences; hashing private records does not anonymize them. | P3 | Workspace safety / WP-07 | Inspect actual transcript fields before defining public example fixtures or exports; preserve explicit local export actions. |

### CANVASS — election status and amendment lineage

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| A1 | Optional transcript installation must preserve supplied status rather than promote verification into official certification. | P3 | Ranked slice 1 | Existing rule retained explicitly; native software status remains separate from official authority. |
| A2 | Lawful amendments change records; changed bytes alone do not establish tampering. | P3 | Ranked slice 1 / provenance | Preserve lineage and fresh package binding; exercise changed-package refusal separately from authorized amended inputs. |
| A3 | National availability does not imply a single jurisdictional canvass workflow. | P3 | WP-07 / policy scope | Retain supplied jurisdiction/status meanings; do not invent certification or adjudication actions absent native consumers. |

### TALLY — transcript and accounting semantics

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| Y1 | A verification result is not automatically the exact native on-disk transcript. | P3 | Ranked slice 1 | Predeclare exact native transcript format and CLI comparison cases before implementing installation. |
| Y2 | Transcript installation should not normalize ballot, contest, batch or vote distinctions into new inferred totals. | P3 | Completion / WP-07 | Preserve original input and native output meanings; test any transformation separately if later introduced. |
| Y3 | Existing download behavior must survive adding optional selected-folder writes. | P3 | Ranked slice 1 | Keep download and write paths independently testable; a write failure must not erase the verified result. |

### TRENCH — failures and closure

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Evidence duplication has already exhausted local storage; an unbounded new snapshot risks preventing reproducible closure. | P2 | Immediate closure task | Planning addressed: capacity preflight, verified deduplication and resolvable inherited blobs. Preserve evidence on refusal. Existing PP-34 remains mitigated, not solved by this review. |
| T2 | Registry-last writes, read-back verification and explicit interrupted recovery appropriately avoid an unsupported atomic-transaction claim. | P3 | Workspace safety / ranked slice 3 | Retain interruption and changed-source/destination tests; do not infer rollback after partial effects. |
| T3 | Full-parity rejection, unchanged historical evidence and per-slice gates prevent declaration counts from becoming acceptance. | P3 | Release gates / handoff | Preserve ordinary and negative full-parity checks; carry historical conditions by ID and scope. |

## Synthesis

Roles reviewed: 8. P1 blockers: 0. P2 issues: 5. P3 notes: 19.

Verdict: **APPROVED-WITH-CONDITIONS for planning**. All five P2 findings now have
concrete planning amendments; their implementation/evidence obligations remain
open wherever the corresponding scoped contract is incomplete. This is not
approval to claim full parity or release the PDF candidate.

Top finding: exact native command behavior must govern parity. More elaborate
browser reporting can still omit the native summary variant.

Cross-role consensus: BENCHMARK and LEDGER require actual native summary
generation comparisons; SURVEY and COVENANT require finishing evidence closure
before starting the next feature; COVENANT and TRENCH require retained, verifiable
evidence without uncontrolled duplicate storage.

## Three amendments applied

1. **Correct the active baseline and closure task.** Distinguish reviewed label
   delivery, passing PDF package records and pending immutable acceptance. Name
   engine reuse and print/save observation limits.
2. **Align implementation priorities with native source and inventory.** Rank
   transcript application, actual native label summary, then rename application;
   retain label import and other parent gaps. Correct the source-pin count.
3. **Make acceptance costs and boundaries explicit.** Require bounded verified
   evidence retention, exact native transcript comparisons and predeclared matrix
   rows before broader support claims. Preserve host/API, capacity and release
   conditions, with missing prerequisites recorded as not-run.

Validation for these documentation amendments: local links, required report
metadata/finding counts and whitespace are checked separately. No runtime code
was changed or runtime tests rerun by this planning review.
