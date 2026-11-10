---
skill: roles-check
topic: wasm-native-label-summary
date: 2026-10-23
roles_used: [SURVEY, BENCHMARK, COVENANT, LEDGER, VAULT, CONTOUR, DATUM, TRENCH]
p1_count: 0
p2_count: 5
p3_count: 19
verdict: APPROVED-WITH-CONDITIONS
---

# Native label/year summary implementation review

Scoped local implementation review; whole-parent and release acceptance remain separate.

Artifact: [native summary contract](../../../docs/specs/wasm-native-label-summary.md),
shared renderer, desktop wrapper, native WASM adapter, worker sequencing,
readonly UI, tests, final package and evidence retention. Depth: standard.
Eight installed repository perspectives are applied by the coding agent,
not independent reviewers or maintainer signatories. No new mathematical,
legal, certification or deployment claim is made.

SURVEY covers usable native delivery; BENCHMARK source/native/browser checks;
COVENANT provenance and immutable evidence; LEDGER native formats and ordering;
VAULT untrusted content and imported authority; CONTOUR source/vintage meaning;
DATUM fair comparison and scope; TRENCH failure, freshness and structural guards.

## Runtime evidence inspected

Fresh engine record:
`target/native-label-summary-delivery/engines-1792801480071643200/evidence.json`,
SHA-256 `e0f00fc2d22a6ac8054558fa201636337798ebb878fcf32202bdadc0333ab127`.
Seven shared renderer/security tests pass, including an escaping regression
that failed before the fix. Six retained original/rebuilt CLI comparisons and
thirteen direct plus six all-year/output cases per engine pass. Native host,
CLI and both actual WASM variants are freshly built; complete reproducible-build
or source-authenticity claims are not inferred.

The full dual-build regression records pass, including original configuration,
State, Verify, continuation, source acquisition and new summary checks. The
default optional-API test extension has an explicit affected-browser rerun with
the original record retained; earlier execution is not silently relabeled.

Final package: `dist/wasm-native-label-summary-validated-v2`. Its 21-stage
successful verification record is
`target/native-label-summary-delivery/package-1792804770898803600/package-evidence.json`,
SHA-256 `2677f39eddf905a69f9770341596b259916fa2c7f5e48fd85ef4fbd793722400`.
It includes native summary/readonly intake, genuine PDF inspection, report
preview security, label installation, TUI/composed Verify, workspace, explicit
loaded C, project and mutation gates. The success is a fresh verification of
unchanged staged bytes after actual fixture regeneration. Earlier catalog and
stale-fixture failures remain retained.

Immutable snapshot verification completed:
`target/native-label-summary-delivery/package-snapshots/wasm-native-label-summary-validated-v2/snapshot.json`,
SHA-256 `eaa99fdca9ca5cce61ce4eb19e4f09dd8d821a1a36e0a21da93b0b4ccb16def6`.
All 3,187 mappings / 1,360 unique identities verify, including the digest-pinned
inherited base. The freeze adds 789 blobs / 211,030,165 bytes for 697,727,821
logical mapped bytes. Final review/support/ledger references follow the freeze;
the archive preserves their earlier execution-stage versions. Post-freeze
ranking/registration changes do not change packaged runtime bytes.

## Findings

P1 blocks shipment, P2 must be fixed for the scoped delivery, P3 is carried
guidance/limitation. All five P2 findings have passing evidence for the named local scope; broader
conditions remain open. No whole-parent acceptance follows.

### SURVEY

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | The existing per-state commission reports omitted the simpler native label/year summary and required sources the desktop summary never reads. | P2 | Native/UI contract | Closed in tested scope: explicit native report type, readonly index intake, native HTML/JSON filenames and portable ZIP; commission reports remain separate and retain defaults. |
| S2 | Native all-year order, repeated years, output override and completed-prefix retention are operational semantics, not cosmetic choices. | P3 | Worker sequencing | Six actual CLI cases cover those effects, including later source-index changes caused by output placement. Preserve them in future changes. |
| S3 | Companion audits and resume/reconciliation controls belong to different workflows. | P3 | UI | Hide/disable those controls for the native summary. Save/Open and no-directory-API execution retain the source snapshot; actual OS picker coverage remains open. |

### BENCHMARK

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | New runtime bytes were compared with stale TUI/C expected identities; changing expected hashes would conceal the missing fixture run. | P2 | Package gates | Closed for this package: regenerate through actual new-engine producers and C/native assignments, retain exact comparisons and earlier failed records. PP-36 remains mitigated broadly. |
| B2 | Current native CLI file/failure checks, retained original binary, source-owned goldens and actual browser workers provide different evidence layers. | P3 | Test design | Retain every layer and its limits; missing original-binary integration prerequisites must be not-run/failure rather than an invented pass. |
| B3 | The final package passes all 21 recorded stages after fixture generation. | P3 | Package | Preserve raw evidence and distinguish staged-byte re-verification from a rebuilt or redeployed package. No full matrix inference. |

### COVENANT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Replacing engines without updating the current served catalog identity invalidates offline binding. Historical run producers must not be retagged. | P2 | Catalog / offline | Closed: only served-engine SHA changes; original catalog histories/source packs remain unchanged. Offline closure passes against final bytes. |
| C2 | The inherited execution snapshot requires an explicit preserved base dependency and verified sizes/hashes for every mapping. | P3 | Evidence retention | Verify the complete inherited closure, retain original/native/current identities and disclose that the archive is not standalone. Final reference follows freeze to avoid circular hashes. |
| C3 | The added portable API-absence test was not in the original default browser invocation. | P3 | Source/evidence timing | Preserve the original default record/browser root and explicit current-script rerun; never silently update an old invocation's scope. Runtime code was unchanged. |

### LEDGER

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | Native summary input is arbitrary raw JSON; forcing typed AnalysisIndex or requiring state report files would change native behavior. | P3 | Shared renderer | Preserve raw context, absent versus empty/null index, and exact u64 JSON values. Thirteen direct cases cover these boundaries. |
| L2 | Report/index/registry bytes and filenames must match actual CLI, with only named runtime differences. | P3 | Comparison | Retain original outputs; allow only top-level creation UTC fields and the specific relative-directory diagnostic separator. No value, label or general JSON normalization. |
| L3 | Relative output directories are a browser adaptation; unknown existing collisions and OS locking/atomicity remain outside portable preparation. | P3 | Filesystem boundary | Preserve explicit output display, exact generated files and safe path rejection. Do not equate ZIP export with live repository installation. |

### VAULT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | The original shared/native renderer interpolated supplied state/status text as executable HTML; a sanitized preview alone would not protect the downloaded original. | P2 | Shared security | Closed in tested code: escape every dynamic HTML text interpolation for both targets. Original regression fails before fix; native/WASM file and browser checks confirm escaped download and inert preview. PP-35 acceptance remains scoped. |
| V2 | Selected-root intake must remain read-only and pending old reads must not replace a newly opened project. | P3 | Readonly UI | Actual worker/OPFS checks record no writes, read permission mode, inert Open and late-picker protection. Imported paths/results confer no write authority. |
| V3 | JSON data can legitimately retain markup strings; hashing or escaping is not source authentication or anonymization. | P3 | Formats / privacy | Preserve exact JSON/source text, render with text-safe controls and restricted previews, and require explicit local exports. No automatic publication/fetch is introduced. |

### CONTOUR

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| G1 | Summary generation consumes supplied index status; it does not revalidate Census totals, GEOIDs, vintages or population bases. | P3 | Source scope | Keep those claims separate from generated/registry-reported status; do not imply successful analysis from a completed command. |
| G2 | Missing analysis index has a native successful fallback and empty upstream hash, not a verified empty dataset. | P3 | Missing evidence | Expose missing-index status and retain the empty chain explicitly. Empty/malformed/null existing files keep their distinct native outcomes. |
| G3 | CLI label/year come from command/layout while raw index metadata may differ. | P3 | Native authority | Preserve actual native behavior; do not invent geography alignment or source authenticity from filename/hash agreement. |

### DATUM

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| D1 | The original binary/source and comparison allowances were retained before extraction, reducing baseline drift. | P3 | Original comparison | Continue retaining both originals and versioned expectation contracts; goldens are executable evidence for their stated scope only. |
| D2 | This is report/workflow parity plus a shared security repair, not a better partition algorithm or statewide capacity result. | P3 | Claims | Keep support wording scoped; no generalization from synthetic summary/worker cases to all-state successful computation. |
| D3 | Source-browser, injected picker, simulated API absence and real OS/browser-host support are different observation scopes. | P3 | Compatibility | Retain exact browser/runtime identity and mark other hosts/not-run capacity separately. No broad support percentage from declarations. |

### TRENCH

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Snapshot preflight found insufficient unique-byte capacity before creating output. | P2 | Storage closure | Closed for this freeze: verified filesystem compression recovered headroom without deletion, and the complete inherited snapshot verifies. PP-34 remains mitigated globally. |
| T2 | Cancellation/failure can retain completed prepared outputs; no OS transaction or rollback is performed. | P3 | Partial workflow | Preserve the completed prefix, final file/index writer order and explicit partial/failure status. Native-summary live installation remains a separate obligation. |
| T3 | Failed packages, stale fixtures and earlier execution stages must remain distinguishable from final acceptance. | P3 | Recovery / release | Retain all failures and unchanged-runtime recovery records. Ordinary validation may pass while negative full-parity still rejects unresolved parents. |

## Synthesis and amendments

Roles reviewed: 8. P1 blockers: 0. P2 findings: 5, closed for the scoped local
delivery. P3 carried notes: 19. Verdict: **APPROVED-WITH-CONDITIONS**. Parent, acceptance-v1, host,
capacity, filesystem and release obligations remain open.

Top finding: native file equivalence must coexist with safe shared HTML output
and honest missing-evidence labels. SURVEY/BENCHMARK/LEDGER agree on exact
operational behavior; VAULT/COVENANT agree that hashes or imported receipts do
not grant authenticity or authority; DATUM/CONTOUR agree on claim limits.

Three amendments implemented or required:

1. Share the native renderer, preserve original comparison witnesses and expose
   native summary separately with exact source bytes, year/output sequencing and
   restricted source-preserving UI.
2. Escape HTML in the shared kernel, retain strict native/runtime comparisons
   and regenerate identity-bearing fixtures through the actual tested engines.
3. Finish verified immutable retention, preserve failed/re-verification records
   and register only scoped partial evidence without accepting broader parents
   or deploying the package.
