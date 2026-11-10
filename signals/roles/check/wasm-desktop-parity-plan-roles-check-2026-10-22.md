---
skill: roles-check
topic: wasm-desktop-parity-plan
date: 2026-10-22
depth: standard
roles_used: [meridian, contour, benchmark, trench, covenant, ledger, survey, vault, scale, ward]
p1_count: 0
p2_count: 20
p2_open_count: 0
p3_count: 10
p3_open_count: 10
verdict: APPROVED-WITH-CONDITIONS
---

# Desktop-to-WASM parity plan: repository role review

Artifact: [desktop-to-WASM parity plan](../../../docs/specs/wasm-desktop-parity-plan.md). Type: implementation specification/proposal. Domains: graph/backend correctness, data identity, browser filesystem and archive safety, interoperability, evidence/provenance, operational workflow and support-claim scope.

Method: one-agent review using the installed role lenses, not ten independent agents or human approvals. No engine code, external legal authority or production deployment is certified by this review. Repository role prose is a perspective, not an authoritative statement of current law. Applicable ranking follows [`.roles/ROLE.md`](../../../.roles/ROLE.md) rather than conflicting historical rank numbers in individual profiles.

Baseline: repository HEAD `383db340235e21d752a35610aab28e2948b73e57` with a dirty working tree. Original plan SHA-256: `a759a7a957f72adb099c056ca60b93dfbf0060a052d7e5e12c1c2f6c3b8d3be0`. The amended file's digest is recorded below after writing; HEAD alone does not identify the reviewed file.

## Role selection

| Role | Selected lens and reason |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Native backend, graph/geometry, algorithm and seeded-output contracts. |
| [CONTOUR](../../../.roles/contour.md) | Original source bytes, vintage, unit IDs, population basis and derived graph identity. |
| [BENCHMARK](../../../.roles/benchmark.md) | Independent reference, clean-checkout reproducibility and meaningful failure tests. |
| [TRENCH](../../../.roles/trench.md) | Structural prevention, interrupted writes, stale evidence and pitfall closure. |
| [COVENANT](../../../.roles/covenant.md) | Reproducible custody, build identity and the limits of hash-based evidence. |
| [LEDGER](../../../.roles/ledger.md) | Versioned RPLAN/RCTX/archive/interchange contracts and numeric/geometry encodings. |
| [SURVEY](../../../.roles/survey.md) | First-use practicality, ownership, browser fallbacks and achievable work sequencing. |
| [VAULT](../../../.roles/vault.md) | Untrusted imports, selected-root authority, private failure receipts and remote boundaries. |
| [SCALE](../../../.roles/scale.md) | Scope of coverage claims, sampling, denominators and expected-failure accounting. |
| [WARD](../../../.roles/ward.md) | Explicit custom subdivision identity and separation of operational metrics from legal findings. |

All sixteen installed profiles were considered. BOUNDARY is not used to interpret or change federal legal standards in this technical plan; its stop rule remains applicable if implementation later introduces such claims. PRECINCT and COMMONS are not assessing new partisan/representation results; DATUM's publication-methodology remit is covered here only by the selected evidence/testing lenses. CANVASS and TALLY are not reviewing new certification, ballot or vendor-count semantics. Those six need targeted review if the ledger reveals changes in their domains.

## Finding disposition

P1 blocks shipping, P2 must be fixed in the plan before implementation sign-off, and P3 is an implementation/release condition. “Closed—plan” means the requirement is now explicit; it does not mean the planned code or acceptance evidence exists. Each role has three findings at standard depth.

### MERIDIAN

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| M1 | Fixed-seed comparisons do not define whether equality means assignments, constraints, bytes or merely a successful call. | P2 | WP-02/WP-08 | Closed—plan: Evidence/comparison rules require exact contracts and predeclared, narrowly scoped differences; no blanket normalization or silent backend substitution. |
| M2 | An adapter comparison can miss a native CLI graph/dispatch/default mismatch. | P2 | Completion contract / WP-02 | Closed—plan: orchestration claims require native CLI/independent references plus original/derived graph identities, index widths, seed, build and backend assumptions. |
| M3 | C-FFI feasibility and realistic resource behavior remain unknown until an actual port attempt runs. | P3 | WP-02 | Open implementation condition: retain the build/run decision artifact and failure details. Keep backend parity open if unsuccessful; do not stall unrelated workspace work. |

### CONTOUR

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | “Source identities” alone does not require hashes of original downloads, source vintage or population basis. | P2 | WP-05 / Evidence rules | Closed—plan: require raw input hashes/source identifiers plus derived graph/context identities, geography vintage, units and population basis. |
| C2 | A multi-vintage matrix could compare assignments across changed tract universes without an alignment contract. | P2 | WP-08 | Closed—plan: IDs/order and explicit cross-vintage alignment are recorded; ACS estimates cannot substitute for PL94 counts. |
| C3 | Retained fixture bodies do not prove live provider availability or original all-state input capacity. | P3 | WP-05/WP-08 | Open implementation condition: distinguish deterministic acquisition tests, live provider observations and real-data capacity rows. |

### BENCHMARK

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | Existing references to local `target/` evidence can hide a clean-checkout dependency on untracked pipeline outputs. | P2 | WP-01/WP-10 | Closed—plan: small fixtures regenerate from tracked inputs; large external fixtures use an immutable manifest and explicit integration tier. |
| B2 | Skipped data/browser cases or self-referential adapter tests could be counted as completed support. | P2 | WP-08 / Release gates | Closed—plan: independent/native references and not-run outcomes are explicit; required unsupported/skipped rows block the full-parity claim. |
| B3 | New workspace adapters need tests that fail when native ordering or collision guards are removed. | P3 | WP-03/WP-04 | Open implementation condition: run real negative/cancellation/collision cases and inspect written bytes, not mocked handler-call counts. |

### TRENCH

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Read-back hashes alone do not prevent a completion marker after partial writes, revocation or stale worker results. | P2 | WP-03 | Closed—plan: explicit staging/recovery transitions, generation invalidation and marker-last rules; do not promise OS atomicity from browser APIs. |
| T2 | The plan does not structurally prevent declaration coverage being promoted to executable parity. | P2 | WP-01/WP-10 | Closed—plan: acceptance/evidence states are separate; preview/full-parity gates and PP-25 mutation requirements are specified. |
| T3 | Documenting the new failure pattern is not a proved structural fix. | P3 | PP-25 | Open implementation condition: pitfall remains OPEN until an automated closure gate rejects inspector-only, stale, skipped and substituted-backend evidence. |

### COVENANT

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| CV1 | A Git revision cannot identify binaries built from a dirty checkout or different build settings. | P2 | WP-06 / Evidence rules | Closed—plan: native/WASM artifacts, relevant dirty sources, toolchains/dependencies/build settings and exact arguments are bound to acceptance evidence. |
| CV2 | “Verified” could conflate byte identity, successful replay, source authenticity and official certification. | P2 | Completion / compatibility contracts | Closed—plan: distinguish those authorities and retain missing producer evidence as partial; imported receipts grant no authority. |
| CV3 | Release evidence still needs an obtainable, exact tested package and reproducible build/custody record. | P3 | WP-10 | Open implementation condition: check package/runtime/asset identities and publish the supported scope with the exact artifact. No authenticity or legal-admissibility assurance is inferred. |

### LEDGER

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | A common workspace adapter could redefine archive/RPLAN fields or hash projections while preserving superficial UI behavior. | P2 | WP-03/WP-07 | Closed—plan: preserve versioned contracts and old read fixtures; require migration and round-trip cases for schema changes. |
| L2 | Integer/index widths and raw-byte, canonical and semantic hashes were not explicitly separated. | P2 | WP-02 / interoperability | Closed—plan: record numeric encodings and separately name identities; no unversioned replacement of upstream hashes or silent u64 conversion. |
| L3 | “Compatible export” remains unproven for an external tool without a pinned schema and fixture round trip. | P3 | WP-07 | Open implementation condition: name versions and validate actual produced/consumed files for each advertised integration. |

### SURVEY

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | The ten items have no explicit implementation assignment/acceptance ownership, and an all-ledger-first reading can delay a risky backend investigation. | P2 | WP-01 / Recommended order | Closed—plan: maintainer acceptance ownership and per-row assignee required; the pinned backend row can start its feasibility work before unrelated reconciliation finishes. |
| S2 | Folder-picker assumptions could exclude users whose browser only supports portable file import/download. | P2 | WP-03/WP-08 | Closed—plan: feature-based full/fallback paths and recovery requirements are now explicit; no unsupported-browser equivalence is claimed. |
| S3 | A technically correct suite may still require the original developer to operate it. | P3 | WP-07/WP-10 | Open implementation condition: demonstrate a first-use source-to-run-to-review-to-save/reopen/replay journey with documented dependencies and useful error recovery. |

### VAULT

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Selecting a root once does not define authority after Open, handle revocation or imported path/receipt restoration. | P2 | WP-03 | Closed—plan: only currently authorized roots confer capabilities; imported intents/receipts never do, and mutation/revocation invalidates pending work. |
| V2 | The workspace/acquisition redesign lacks explicit hostile archive, active-content, credential and failure-receipt boundaries. | P2 | WP-03/WP-05 | Closed—plan: bounded decompression/schema/path rules, safe rendering, explicit URL/provider boundaries and private local receipt exports are required. Existing hash formats are preserved, not redesigned. |
| V3 | A new adapter can regress existing security limits even if the plan restates them. | P3 | WP-03/WP-05/WP-10 | Open implementation condition: run adversarial archive/root/URL/permission tests and prove zero automatic execution/fetch/write on Open. |

### SCALE

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| SC1 | “Representative combinations” permits selecting favorable results after execution unless rows and exclusions are fixed first. | P2 | WP-08 | Closed—plan: predeclare each claimed matrix row, source/method/backend/runtime and expected native outcome; national/all-vintage claims require their corresponding rows. |
| SC2 | A single success count can mix execution completion, accepted candidates, expected failure, partial workflow and not-run cases. | P2 | WP-08/WP-10 | Closed—plan: separate those outcomes and preserve denominators; no extrapolated completion percentage. |
| SC3 | Real-data coverage and runtime bounds remain observations until measured on the advertised matrix. | P3 | WP-08 | Open implementation condition: report the actual supported data/backend/browser envelope rather than extrapolating from small synthetic examples. |

### WARD

| # | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| W1 | Extending custom county identities could turn missing identity into zero splits or apply Census-prefix inference to arbitrary IDs. | P2 | WP-09 | Closed—plan: explicit identity source, missing-vs-zero separation, no custom prefix inference and existing Census regression witnesses are required. |
| W2 | Matching native chamber defaults or split counts could be presented as current jurisdictional legal compliance. | P2 | WP-04/WP-09 | Closed—plan: preserve source/vintage of policy/count/tolerance snapshots and separate operational metrics from legal conclusions. No jurisdiction-specific law is interpreted by this review. |
| W3 | Subdivision semantics and policy provenance must be verified for each eventual advertised jurisdiction/chamber, not inferred from a two-letter label. | P3 | WP-08/WP-09 | Open implementation condition: test the declared identity contract and retain scope boundaries; escalate actual legal-rule changes to BOUNDARY/WARD review. |

## Synthesis

Roles reviewed: **10**. Findings: **30** — **0 P1**, **20 P2**, **10 P3**.

Initial verdict: **NEEDS-WORK** for missing planning gates. After amendment: **APPROVED-WITH-CONDITIONS** for implementation planning. All twenty P2 requirements are now explicit in the plan; zero P2 plan issues remain open. Ten P3 implementation/release conditions remain open and attached to the work items above. No execution or parity completion was established.

Top finding: parity must be bound to executable native behavior and exact artifact/fixture evidence, not inferred from command coverage or a passing adapter test.

Cross-role consensus:

- MERIDIAN, BENCHMARK, COVENANT and SCALE require pinned independent baselines and scoped comparison claims.
- TRENCH, VAULT and SURVEY require recoverable selected-root workflows with explicit browser capability/fallback behavior.
- CONTOUR, LEDGER and WARD require source/vintage/unit/subdivision meaning to survive the port without becoming an authenticity or legal claim.

## Three amendments applied

1. Added **Evidence, ownership and comparison rules**: structured row states, assignees, artifact/source/toolchain binding, exact-vs-tolerant rules, clean fixtures and an early reproducible backend decision.
2. Added **Workspace safety and recovery contract** and **Compatibility, interoperability and policy scope**: explicit authority/state transitions, marker-last recovery, hostile-input boundaries, portable browser paths, schema/encoding rules and subdivision provenance.
3. Added **Release gates and closure rules**: distinct preview/full-parity claims, predeclared matrix and outcome accounting, exact package binding and required negative closure tests. Added PP-25 as OPEN; also indexed the already existing PP-24 without changing its implementation status.

## Review limits and next action

This is a source-backed plan review. Local document links/counts/digests are checked; no runtime suite is rerun for documentation-only amendments. The roles' implementation conditions require future evidence and cannot be closed by this report. Existing engine functionality and deployments were unchanged.

Next: implement WP-01's structured gap/evidence records and rank the first three source-backed slices. Start the WP-02 feasibility attempt when its baseline row is pinned. Keep PP-25 and the ten P3 conditions open until their actual guards and tests pass.

Amended plan SHA-256: `c4d84493ffbbdc4c40545a03a9e7e9bf5cee6daa88c8fb8b032a8ad822135a3c`.
