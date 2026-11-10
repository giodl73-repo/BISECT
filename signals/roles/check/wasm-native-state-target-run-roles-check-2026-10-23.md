---
skill: roles-check
topic: wasm-native-state-target-run
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

# Selected-State target-action implementation review

Artifact: [scoped implementation contract](../../../docs/specs/wasm-native-state-target-run.md)
and its linked modules/tests. One agent applied seven installed role lenses, not
independent reviewers. Profiles and their verify questions were read during the
[plan handoff review](wasm-desktop-parity-plan-handoff-roles-check-2026-10-23.md).
This is scoped local implementation approval with remaining conditions, not full
parity, public release, certification or legal assessment.

## Selection

MERIDIAN checks actual backend consumption; BENCHMARK checks native references
and packaged regressions; COVENANT checks stable raw evidence and identities;
LEDGER checks project/receipt meanings; VAULT checks live authority and replay;
TRENCH checks failure ordering and stale closure; SURVEY checks the explicit
practitioner journey and fallbacks. The [role catalog](../../../.roles/ROLE.md)
and each selected profile remain the authority for these perspectives.

## Findings

Three findings per lens, 21 total. Closed-scoped P2 findings have structural
changes and passing regressions. P3 obligations remain open outside the exact
tested scope.

| ID | Lens | Finding | Severity | Section | Disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | Initial preflight treated every C/gpmetis declaration as consumed, even for one district or non-METIS methods. | P2 | Backend preflight | Closed-scoped: mirror native method/count usage, ten actual successful CLI comparisons with exact assignments, and required-backend refusal before action. |
| M2 | MERIDIAN | JS mirrors the native compiler's four-method exemption and may drift if native dispatch changes. | P3 | WP-02/04 | Keep source pins and the native comparison matrix; expand it when dispatch semantics change. |
| M3 | MERIDIAN | These four-unit fixtures do not prove real-state candidate acceptance or all method combinations. | P3 | WP-08 | Keep real-data/method matrix and capacity claims open. |
| B1 | BENCHMARK | New tests were absent from builders and source-tree checks did not identify a final package. | P2 | Build gates | Closed-scoped: both standard builders register checks; workspace builder ran module/native tests and all six relevant packaged browser suites. Full standard-build suites were not rerun. |
| B2 | BENCHMARK | Selected root observation cannot lock other programs out during removal. | P3 | Target action | Retain before/after snapshots and documented concurrency limits; no atomicity claim. |
| B3 | BENCHMARK | OPFS with injected picker does not prove OS dialog/permission handling across browsers. | P3 | Browser support | Retain exact host scope; add advertised host tests before expanding claims. |
| C1 | COVENANT | Mutable latest target records are insufficient for stable evidence custody. | P3 | Evidence | A 63-file owned snapshot now preserves source/raw outcomes/native executable/engine and package identities; obtainability and formal release acceptance remain open. |
| C2 | COVENANT | A completed target receipt is not successful generation or installed output evidence. | P3 | Receipt semantics | Preserve separate outcomes; malformed source after reset retains action outcome without a partition. Full Verify remains open. |
| C3 | COVENANT | Named C profile and retained binary hashes do not establish source authenticity or legal admissibility. | P3 | WP-06/10 | Preserve explicit non-claims and remaining notice/release obligations. |
| L1 | LEDGER | Original and derived argument texts must not overwrite each other. | P3 | Project contract | Both retained and tested; broader schema migration/old-read coverage remains open. |
| L2 | LEDGER | Failure/cancel action receipts are available in-session, but their automatic cross-session custody is unfinished. | P3 | Receipt export | Keep explicit download usable and this limitation visible. |
| L3 | LEDGER | Manual output installation differs from native automatic post-Run writing. | P3 | Workspace lifecycle | Keep automatic installation outside the supported slice. |
| V1 | VAULT | Imported receipts/derived bindings must not reconstruct execution authority. | P3 | Private grant | Source/request-bound one-use grants and inert reopen pass; broader authority workflows remain separately reviewed. |
| V2 | VAULT | A private JS grant does not defend against privileged same-origin script. | P3 | Trust boundary | Document the imported-data threat boundary and avoid cryptographic-authenticity claims. |
| V3 | VAULT | New context, worker or nonce must retire old reviews even when an asynchronous action is pending. | P3 | UI/worker lifecycle | Current stale nonce/target/source and cancel checks pass; broader root/host-policy combinations remain open. |
| T1 | TRENCH | Cancellation after actual removal did not retire the UI action through the Cancel update path; passive receipts also vanished on autosave revision changes. | P2 | UI lifecycle | Closed-scoped: update authority on controls changes; carry only passive records across unchanged-result autosave. Browser pauses after real removal, cancels and downloads an interrupted receipt. |
| T2 | TRENCH | Removing engine-only guards would let stored flags bypass explicit target handling. | P3 | Compiler boundary | Guards remain; ordinary State workflow/adjacency regressions pass. Keep broader branches explicit. |
| T3 | TRENCH | A completed local slice can be mistaken for resolved full reset/force or parent parity. | P3 | Ledger/PP-25/28 | Child records remain evidence-partial; parent and full-parity gate remain unresolved. |
| S1 | SURVEY | Bulk completion/reprocess and acquisition/recovery still lack a composed full lifecycle. | P3 | WP-01/03/05 | Continue source-backed ranking and implementation; this slice does not close the plan handoff's broader ranking obligation. |
| S2 | SURVEY | A portable browser without a writable directory picker cannot perform this local removal. | P3 | Fallback | Explain unsupported action; compute-only Run without filesystem flags and existing downloads remain available. |
| S3 | SURVEY | Unlabeled custom adjacency naming and broader practitioner/real-state acceptance remain incomplete. | P3 | WP-04/07/08 | Require explicit labels for that scope and keep broader work open. |

## Synthesis and amendments

Seven lenses: **0 P1, 3 P2 closed in this tested scope, 18 P3 open**.
Verdict: **APPROVED-WITH-CONDITIONS for the scoped local implementation**.

Top finding: cancellation changes execution authority immediately, but cannot
undo removal already in progress. TRENCH/VAULT/LEDGER agree the UI must preserve
a passive outcome without keeping a live grant. MERIDIAN/BENCHMARK agree that
native consumed behavior, rather than declared backend text, governs preflight.

Three amendments were implemented and checked:

1. Method/count-aware preflight, exact native comparison witnesses and required
   unavailable-backend refusal before target actions.
2. Cancellation retirement in controls updates, passive-only autosave revision
   carry, visible interrupted receipt after real removal and replay refusal.
3. Registered native/browser gates, a newly staged tested package and stable
   owned source/raw-output snapshot, preserving the failed earlier package.

## Evidence and limits

HEAD `383db340235e21d752a35610aab28e2948b73e57`, dirty working tree.
The package is `dist/wasm-state-target-run-validated`; offline version
`c3ef377689a73748b14f8de2fd74eeba1f60f4161db2ea93c138784333e95ef2`.
Its engine is 25,118,842 bytes, SHA-256
`0cb66c3d20fe579c2f32a623d0a11fb7e770c90425f274f533b8a77195fe7246`.
The builder checked 635 asset identities, 535 shell assets, 50 state/year packs,
263 toolkit files and 1,049 local dependency edges.

Fresh checks: target-action module/native CLI tests (one baseline plus ten
unused-backend CLI runs), standalone reset (32 intentional-failure CLI filesystem
comparisons/25 refusals), selected workspace (three modes/26 host replies/18 ZIP
refusals), and final-package data-reset, target-action, workspace, single/batch
recovery and optional-C browser suites. Ordinary State workflow (35 replies) and
adjacency workflow (14 replies plus independent native n-way) regressions pass.
No new Rust WASM was built and the full legacy C matrix was not rerun here.

Stable snapshot:
`target/native-state-target-run/package-snapshots/wasm-state-target-run-validated/snapshot.json`.
Its 63 copied files include raw CLI replies, original project/result/action
receipts, browser evidence, critical source files, owned native executable and
packaged engine/sidecar bytes. The generator verifies source evidence and staged
module equality and refuses replacement. Local hashes identify bytes, not
authenticity, public release obtainability or a formal acceptance-v1 bundle.

The earlier `dist/wasm-state-target-run` failed the new cancellation browser
regression; it is not accepted. The separate validated package includes the fix.
Nothing was committed, deployed or published. All remaining WP-01–WP-10 scope
and the full-parity closure gate remain in force.
