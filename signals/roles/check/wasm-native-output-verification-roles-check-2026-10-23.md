---
skill: roles-check
topic: wasm-native-output-verification
date: 2026-10-23
roles_used: [MERIDIAN, BENCHMARK, COVENANT, LEDGER, VAULT, TRENCH, SURVEY]
p1_count: 0
verdict: APPROVED-WITH-CONDITIONS
---

# Selected native-output Verify: implementation review

Artifact: [scoped implementation and evidence](../../../docs/specs/wasm-native-output-verification.md).
Type: browser/Rust workflow implementation. The coding agent applies seven
repository role lenses; this is not independent reviewers, maintainer acceptance,
legal advice, release sign-off or a full-parity verdict.

Selected lenses: MERIDIAN for actual graph/assignment identity, BENCHMARK for
native/browser proof, COVENANT for recorded versus authenticated producer claims,
LEDGER for schemas/integer/index conventions, VAULT for imports and authority,
TRENCH for partial-stage failure, and SURVEY for usable explicit actions.
Other installed roles' legal, partisan, election-certification and ballot-data
domains receive no new support claim from this implementation.

## Findings

P1 blocks shipment, P2 must be fixed for this scoped milestone, P3 should be fixed
or carried as a documented broader limitation. Closed means the named bounded
guard and its regression passed; it never closes an entire work item by inference.

### MERIDIAN

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| M1 | Rehashed recipe/preparation receipts must not substitute a different actual replay graph. | P2 | `verify_replay.rs`; source stage | Closed for the selected-target scope: typed native graph equality, altered/rehashed source refusals, four exact native-host inspection replies and actual packaged worker checks. |
| M2 | JavaScript numeric normalization can erase a one-unit difference above 2^53 or a maximum-u64 seed. | P3 | Native binding / raw manifest | Raw manifest/replay bytes reach Rust; the native unit witness distinguishes 9007199254740993 from 9007199254740992, and Node/browser receipts retain the exact maximum seed. Preserve these guards. |
| M3 | C-enabled engine packaging does not prove all C-backed Verify journeys or method/data combinations. | P3 | WP-02/06/08 | Open beyond scope. New composed cases use metis-core on both builds; loaded-C/native regression is separate. No all-method or all-state claim. |

### BENCHMARK

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| B1 | A source-tree test cannot accept different final staged modules or an old engine. | P2 | Builders / snapshot | Closed for this package: final worker/browser checks, loaded-WASM hash assertion, asset/offline closure and snapshot checks bind Node/browser outcomes to the staged engine. |
| B2 | Small acceptance fixtures must not depend on an old generated source corpus. | P3 | Node fixture | Four-unit RI adjacency regenerates from tracked JS; current CLI/host binaries are explicit build prerequisites and owned copies are retained. Wider legacy/real-data integration tiers retain their own prerequisites. |
| B3 | Synthetic RI success cannot accept medium/large real-state capacity, vintages or every unit kind. | P3 | WP-08 | Open. Preserve predeclared real-data matrix work and not-run outcomes rather than extrapolating from fixture size. |

### COVENANT

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| C1 | Current WASM identity must not authenticate saved historical producer claims or missing upstream evidence. | P2 | Result / summary | Closed for scoped presentation: loaded identity, unverified historical claim, source stage and replay outcome are separate; the packaged missing-source/matching-replay case keeps the missing stage visible. |
| C2 | Mutable target results and dirty HEAD alone are insufficient to identify the tested delivery. | P3 | Evidence snapshot | A 145-file non-replacing snapshot preserves sources, Cargo inputs, owned references, raw outcomes, both builds, package manifest and browser evidence. Formal acceptance-v1/build custody remains open. |
| C3 | CLI dry-run checks and shared-adapter inspection cannot substantiate complete native CLI replay orchestration. | P3 | Reference scope | Explicitly limited: two actual CLI filesystem prechecks plus four exact native-host inspections. Broader native Verify branch/orchestration evidence remains open. |

### LEDGER

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| L1 | Native index keys can overlap numeric unit IDs with a different ordering. | P3 | Assignment mapping | Refuse ambiguity, incomplete order and by-unit disagreement before partitioning. Native binding checks district range; tests retain raw assignment bytes and chosen convention. |
| L2 | Adding inspection/source binding must not change legacy replay replies when those fields are absent. | P3 | Rust serde defaults | New fields default off; existing manifest replay and TUI constituent comparisons pass against both actual engine variants. Final packaged legacy TUI browser test also passes. |
| L3 | A selected single-target ZIP/file-set workflow is not general native output-tree discovery or arbitrary interchange compatibility. | P3 | Input scope | Keep the exact package/layout scope in the UI and docs. Wider discovery, external schemas and alternate output layouts remain separate requirements. |

### VAULT

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| V1 | Imported success markers or incomplete records must not execute work or break project Open. | P3 | Project restore / renderer | Packaged tests open saved results and a schema-only incomplete record without execution or fresh downloads. The summary handles absent stages and uses text rendering. |
| V2 | A late ZIP read must not overwrite a newer project or operation. | P3 | UI revision/sequence | Read retirement and revision checks guard commit; the packaged delayed-read case preserves the newer project's source and usable controls. No live authority survives Open. |
| V3 | An output-layout extension must retain archive/path/CRC limits. | P3 | Archive decoder | Existing 18 hostile native-archive refusals remain passing; the new browser workflow refuses corrupt CRC and binds the retained ZIP inventory. Broader output-layout/adversarial matrix remains open. |

### TRENCH

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| T1 | Aggregate replay success can conceal missing source/provenance stages. | P2 | PP-31 / result schema | Closed scoped guard: export per-stage outcomes, preserve missing source after a matching replay, and show it in the actual packaged summary. PP-31 is MITIGATED; other consumers remain open. |
| T2 | Precheck/source/context failures must stop before partition and retain a usable result. | P3 | Compose order | Missing sidecar, changed/rehashed source, missing context, incomplete/ambiguous order and unavailable replay cases record no partition; actual browser failures stay downloadable as current verification records. |
| T3 | Updating current pins must not overwrite historical proof or silently promote partial rows. | P3 | Parity ledger | Only active source/candidate mappings were reconciled. New child remains evidence-partial, historical snapshots retained, ordinary ledger passes, full gate still rejects the Run parent. |

### SURVEY

| ID | Finding | Severity | Section/source | Disposition |
|---|---|---|---|---|
| S1 | Two similarity controls with different meanings would make the workflow difficult to document. | P3 | Workbench inputs | New workflow shows one threshold and an optional replay override. Legacy replay controls stay hidden. Mode/threshold changes retire fresh exports; the requested threshold is in the record. |
| S2 | Practitioners need a visible exact seed, readable stage failure and portable record. | P3 | Summary/download | Seed and stage details are text, record downloads match saved results, and the 390px control screenshot was inspected. Broader accessibility/browser/OS-picker coverage remains open. |
| S3 | A passing verification result must not silently start workspace Resume, reset or installation. | P3 | Follow-on boundary | This operation receives selected bytes only; no target/reset/acquisition/install wrapper executes. Actual browser tests record no picker/effect or remote request. Fresh live continuation is the next separate contract. |

## Synthesis and amendments

**7 roles, 21 findings: 0 P1, 4 P2, 17 P3.** The four P2 scoped guards pass.
Verdict: **APPROVED-WITH-CONDITIONS for the scoped local preview**. Broader
method/data/browser coverage, source authenticity, custody, saved-workspace
continuation, remaining Verify branches and formal release acceptance remain open.

Top finding: an assignment replay match must remain separate from missing
source or producer evidence. MERIDIAN/COVENANT/TRENCH agree on that boundary.
BENCHMARK/COVENANT agree that final staged bytes and retained raw evidence are
necessary; VAULT/SURVEY agree that verification must confer no filesystem grant.

Three amendments implemented:

1. Added no-partition inspection and exact native source-to-replay graph binding,
   preserving raw manifest integers and refusing ambiguous/incomplete indices.
2. Consolidated controls, retired stale reads/results, rendered incomplete
   imported records safely, and preserved partial stages in summary and export.
3. Registered actual dual-build and packaged-worker checks, retained legacy
   regressions, reconciled active ledger pins and froze a new local snapshot.

## Evidence and limits

Baseline HEAD `383db340235e21d752a35610aab28e2948b73e57`; dirty working tree.
Package: `dist/wasm-native-output-verification-validated`.
Offline version: `66e71bbb14b03acb7b0efe54adc151f6d67170e246955e3f0b0d196d515d0f08`.
Default WASM: `60a69fd0ad32af37c3853ba423a6af5487d27c5f80937dceac323a80966b24cd`.
Packaged C-enabled WASM: `56d1898c7743dbed3634e342112d061b9400e37d3d040e76def74ad4bf13704b`.

The [immutable 145-file snapshot](../../../target/native-output-verification/package-snapshots/wasm-native-output-verification-validated/snapshot.json)
retains source/raw-output and engine identities. It is a local investigation
bundle, not acceptance-v1, signed source authenticity or reproducible-build proof.

Executed evidence includes 16 new cases on each actual build, four exact
native-host inspection comparisons per build, two actual CLI dry-run cases,
source graph integer-bit unit witness, existing 11-case manifest replay and
14-constituent TUI comparisons on both variants, current project-handler tests,
final-package new and legacy TUI browsers, workspace/reset/target/bulk actions,
acquisition/recovery/streaming, output recovery and optional loaded-C browser.
The C matrix separately retains 52 assignment matches and four matching failures.
Asset dependency counts establish packaging closure, never execution coverage.

The ordinary ledger and 8 ledger/4 native-consumption mutation tests pass.
`--full-parity` still fails at `bisect.Run: Parent evidence has not been verified`.
No verified parent promotion, commit, push, hosted deployment or legal/election
certification was performed.
