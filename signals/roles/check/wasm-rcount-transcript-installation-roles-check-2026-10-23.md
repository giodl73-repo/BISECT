---
skill: roles-check
topic: wasm-rcount-transcript-installation
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CANVASS, TALLY, TRENCH]
p1_count: 0
p2_count: 5
p3_count: 19
verdict: APPROVED-WITH-CONDITIONS
---

# Reviewed RCOUNT transcript application

Artifact: [implementation contract](../../../docs/specs/wasm-rcount-transcript-installation.md),
`count-transcript-install.js`, its workbench controls, actual native/browser
tests, package and immutable evidence. Depth: standard. Eight installed role
perspectives were applied by the coding agent, not independent reviewers or
maintainer signatories. No legal or certification determination is made.

SURVEY reviews optional usable delivery; BENCHMARK native and browser evidence;
COVENANT identity and custody; LEDGER exact native format; VAULT private authority;
CANVASS supplied status/lineage; TALLY unaltered accounting evidence; TRENCH
interruption, stale state and storage failures.

## Evidence boundary

Package: `dist/wasm-rcount-transcript-validated`. Execution record:
`target/report-pdf-delivery/build-1792799335204662500/package-evidence.json`,
SHA-256 `682dcc6a2c27c42bb6c52ddb9571c40796d7583cf825c0954b52e5c9d3156ea9`.
All 29 recorded stages pass. Actual default and optional-C engines each match
three native transcript files plus the native write-error outcome. Both source
browsers and the final-package browser exercise real workers and OPFS, with an
injected directory picker. Actual OS picker/permission behavior is not tested.

Native CLI identity is
`341a6d39d5d95bb1a77ff75f35abff7dfdba7ad786e892c87b2ae94086d1b459`.
It was built from the current worktree with recorded Rust/Cargo identities and
65 source/manifests. Engines reuse prior verified artifacts; no fresh WASM build
or complete source-to-binary reproduction is claimed. Existing PDF, preview,
label installation, Verify, workspace, optional-C, project and mutation gates
also pass against the new staged package or source as named in its record.

Immutable snapshot:
`target/report-pdf-delivery/package-snapshots/wasm-rcount-transcript-validated/snapshot.json`,
SHA-256 `0ec7f647938b1c2a2768d196ec7aa19e571db957fd74e9f1196f0d89c0db839f`.
All 1,869 mappings and 903 unique identities verify, including the explicit base
dependency. New storage is 474 blobs / 28,687,850 bytes for 407,974,020 logical
mapped bytes. Final review, support and ledger registration follow freezing;
the snapshot preserves their earlier execution-stage documents and ranking.
Post-freeze planning/source-inventory edits do not change packaged runtime bytes.

## Findings

P1 blocks shipment; P2 must be fixed for this scope; P3 is carried guidance or
limitation. Five P2 findings are closed only within the named tested scope.

### SURVEY

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | Browser verification/download previously omitted native optional on-disk transcript delivery. | P2 | User journey | Closed in this scope: fresh review, explicit add/replace confirmation, write and read-back now operate on the selected matching package. |
| S2 | Both pass and fail transcripts are useful records; failed verification must not disable native-equivalent transcript writing. | P3 | Native contract | Both outcomes are retained and native-byte compared. Keep failure visible and distinct from filesystem failure. |
| S3 | The 390px reviewed controls are usable and the ordinary download survives without directory APIs. | P3 | UI / compatibility | Retain the portable path; do not infer universal host support from injected OPFS. |

### BENCHMARK

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Shared-adapter comparisons alone would not establish native file output and write-error semantics. | P2 | Native tests | Closed: actual CLI add/replace/failing-verification bytes match; destination-directory CLI exits 2 while browser review refuses it before writing. |
| B2 | Actual worker/OPFS, inert Open, no-write review, confirmation, changed-package refusal and cancellation after committed write have browser evidence. | P3 | Browser tests | Retain raw files, receipt and native consumption witness; OS picker remains not tested. |
| B3 | New source checks and final-package regressions pass; standard builder registration is separate from rerunning every complete builder. | P3 | Build gates | Keep the scoped reuse/build mode explicit and do not promote broader parent acceptance. |

### COVENANT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Native semantic package hashes do not independently bind every selected raw source byte. | P2 | Review / receipt | Closed for this write: compare the complete source roster and hashes, and record a separate sorted raw source-inventory identity excluding only the fixed transcript output. |
| C2 | Current native CLI, reused WASM and supplied historical producer claims are different identities. | P3 | Execution record | Preserve explicit build/source/runtime records and original raw results; hashes alone establish no source authenticity. |
| C3 | The inherited snapshot is verifiable but depends on its digest-pinned base. | P3 | Snapshot | Preserve the base and resolve every blob; no standalone-copy or formal release-custody claim. |

### LEDGER

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | The WASM value response has map ordering different from native struct serialization; naive JSON export would change native transcript bytes. | P2 | Serializer | Closed for v1: restore native declaration order and omitted optional fields, retain typed strings and pretty formatting without a trailing newline; compare whole actual CLI files. |
| L2 | The fixed native destination, directory creation and replacement behavior remain intact behind explicit browser review. | P3 | Native / browser boundary | Preserve existing-source bytes; extra-file refusal and bounded safe paths are named browser adaptations. |
| L3 | Receipt and review use separate versioned schemas and do not alter native transcript or project format meanings. | P3 | Project / receipt | Keep reviews, handles and write grants out of serialization. Historical receipts remain nonauthoritative. |

### VAULT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Imported or consumed reviews cannot grant write authority through their visible hashes or paths. | P3 | Private review | A private WeakMap owns the current root and one-use grant; clone/reuse refusal is tested. |
| V2 | Source/result/revision changes and Open retire reviews; pending reads and writes recheck current context. | P3 | UI / writer | Retain cancellation and context checks; no OS locking or race-free filesystem claim. |
| V3 | Source and transcript commitments can still disclose information; hashing is not anonymization. | P3 | Receipt / privacy | Only explicit local receipts are exported; synthetic examples contain no private voter data. No new candidate-choice proof or public upload is introduced. |

### CANVASS

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| A1 | Software verification pass/fail must not be promoted into legal canvass or certification status. | P3 | UI / receipt | Explicitly labeled verification, not certification; no status event or legal action is generated. |
| A2 | Lawfully amended records must preserve lineage rather than become an invented tampering determination. | P3 | Source preservation | Input package bytes are unchanged; changed inputs require fresh verification and review. This feature does not adjudicate amendments. |
| A3 | Native supplied jurisdiction/status meanings remain authoritative for computation, not evidence of current law. | P3 | Native boundary | No jurisdictional legal threshold or national certification workflow is added or claimed. |

### TALLY

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| Y1 | Transcript delivery must preserve native checks rather than infer new totals from a display projection. | P3 | Serializer | Serialize only the typed native verification transcript; no count arithmetic or numeric coercion is introduced. |
| Y2 | Batch, ballot, contest, source and status records must survive the operation unchanged. | P3 | Source roster | Every selected input other than the fixed output is byte-bound before and after installation; test source preservation separately from result status. |
| Y3 | Source-tampered failure and write failure are distinct outcomes. | P3 | Native cases | Retain CLI verification exit 1 with written fail transcript versus write exit 2, and explicit browser refusal/interruption status. |

### TRENCH

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Low disk headroom could prevent retaining a new package and its acceptance evidence. | P2 | Capacity / retention | Closed for this operation: byte-verified NTFS compression of owned generated artifacts, fresh package preflight and inherited evidence keep historical bytes intact. PP-34 remains mitigated globally. |
| T2 | Cancellation after a real write can leave the intended file committed without completed read-back. | P3 | Interrupted receipt | Tested; report cancelled/interrupted without rollback or completion. Fresh review reobserves current bytes before another replacement. |
| T3 | The first package attempt failed on stale source pins after a Windows text-decoding error. | P3 | Failure history | Correct UTF-8 pin reconciliation passed; retain failed build `build-1792799318293670500` and the later separate successful record. No failed attempt is accepted. |

## Synthesis and amendments

Roles reviewed: 8. P1 blockers: 0. P2 findings: 5, closed for the scoped local
delivery. P3 notes: 19. Verdict: **APPROVED-WITH-CONDITIONS**.

Top finding: preserve the native transcript and its failure semantics while
keeping browser write authority private and fresh. SURVEY, BENCHMARK and LEDGER
agree that exact native file behavior matters; COVENANT and VAULT agree that a
valid hash or transcript does not grant authority or establish authenticity.

Three amendments applied:

1. Deliver the optional selected-package write through fresh review, explicit
   destination confirmation and exact native v1 bytes for both pass and fail.
2. Bind raw sources and live destination before/after writes, preserve inert
   Open and single-use private authority, and retain interrupted outcomes.
3. Complete both-engine/native and final-package checks, freeze verified
   execution bytes without uncontrolled duplication, and register only partial
   scoped evidence. Parent, formal acceptance-v1, host/capacity, release and
   full WP-03/WP-07 acceptance remain open. No deployment approval is granted.
