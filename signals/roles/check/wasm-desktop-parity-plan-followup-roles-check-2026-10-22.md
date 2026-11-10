---
skill: roles-check
topic: wasm-desktop-parity-plan-followup
date: 2026-10-22
depth: standard
roles_used: [meridian, benchmark, trench, covenant, ledger, survey, vault]
p1_count: 0
p2_count: 3
p2_open_count: 0
p3_count: 18
p3_open_count: 18
verdict: APPROVED-WITH-CONDITIONS
---

# Current parity plan: follow-up review

Artifact: [desktop-to-WASM parity plan](../../../docs/specs/wasm-desktop-parity-plan.md). This is a technical implementation-plan review using seven installed role lenses in one agent, not seven independent reviewers or human signatories. The [initial ten-role review](wasm-desktop-parity-plan-roles-check-2026-10-22.md) remains historical evidence; its implementation conditions are not closed by this report. Role prose is not current legal authority. No legal, certification or release approval is given.

Baseline: HEAD `383db340235e21d752a35610aab28e2948b73e57`, dirty working tree. Amended plan SHA-256: `9f5fb28b08b46ab2e6091b25f8fc5ac8e93cdaadc2498c464c7ac2857e0527bb`.

## Selection and method

Considered all sixteen installed profiles and their verify questions; ranking follows [ROLE.md](../../../.roles/ROLE.md). Selected profiles were read in full. The current plan, prior review, structured-ledger narrative, sidecar runner and local probe evidence were inspected. Findings concern specification adequacy and remaining acceptance obligations, not newly demonstrated runtime defects.

| Lens | Reason selected |
|---|---|
| [MERIDIAN](../../../.roles/meridian.md) | Seeded partitioning, backend equivalence and explicit RNG behavior. |
| [BENCHMARK](../../../.roles/benchmark.md) | Independent references, repeatable fixtures and negative evidence. |
| [TRENCH](../../../.roles/trench.md) | Stale progress claims, interrupted work and structural closure gates. |
| [COVENANT](../../../.roles/covenant.md) | Runtime/source/artifact identities and additional sidecar dependencies. |
| [LEDGER](../../../.roles/ledger.md) | Cross-memory interface and preserved versioned interchange contracts. |
| [SURVEY](../../../.roles/survey.md) | Next executable milestone and usable browser workflows. |
| [VAULT](../../../.roles/vault.md) | Imported evidence authority, explicit execution and local data privacy. |

CONTOUR, SCALE and WARD conditions from the initial review remain applicable; this follow-up changes no population, statistical or policy contract. BOUNDARY, PRECINCT, DATUM, COMMONS, CANVASS and TALLY need targeted review if subsequent work introduces legal, political, research, community, certification or voting-system claims. Their domain-specific tests are not purportedly passed here.

## Findings

P2 means amend the plan before accepting its next milestone. P3 means an implementation or release obligation remains open. Closed-plan dispositions establish requirements only.

| ID | Lens | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|---|
| M1 | MERIDIAN | Same vendored C and seed can yield different assignments across runtime libraries; the comparison contract needs a named runtime/RNG profile. | P2 | WP-02 / Evidence rules | Closed-plan: require native runtime identity, unmodified baseline and explicit hashed compatibility experiments. No label normalization for exact assignment claims. |
| M2 | MERIDIAN | One four-cycle success cannot establish weighted, multi-seed or dispatch equivalence. | P3 | WP-02 | Open: run predeclared weighted/unweighted, seed and invalid-input cases, then actual engine dispatch. |
| M3 | MERIDIAN | Node execution of a standalone C module does not demonstrate browser worker or production backend selection. | P3 | WP-02 | Open: exercise real worker integration while preserving explicit backend choice and current defaults. |
| B1 | BENCHMARK | The current local sidecar mismatch must remain a failing exact comparison even though both executions succeed. | P3 | Evidence rules | Open: retain raw native and sidecar results and a regression witness; do not turn successful module loading into parity. |
| B2 | BENCHMARK | A local SDK/cache and target-only logs do not constitute a portable acceptance prerequisite. | P3 | WP-01/02 | Open: document pinned setup and regenerate small cases from source-owned fixtures in a clean environment. |
| B3 | BENCHMARK | Successful error cleanup on one fixture is insufficient coverage for a production C bridge. | P3 | WP-02 | Open: include success after error, memory/interface bounds and browser failures; keep unavailable cases not-run. |
| T1 | TRENCH | WP-09's future-only wording contradicts the current status and scoped implementation evidence. | P2 | WP-09 table | Closed-plan: name delivered explicit county scope and retain remaining subdivision/policy/full-acceptance gaps. |
| T2 | TRENCH | Historical missing-compiler evidence can remain stale even after a new runtime experiment succeeds in building. | P3 | WP-01/02 | Open: reconcile ledger and feasibility history without deleting the earlier failed attempt. |
| T3 | TRENCH | New requirements in prose do not close PP-25's acceptance-evidence failure mode. | P3 | Release gates | Open: retain PP-25 until rejection guards and actual release-bundle integration pass. This is the existing pitfall; no duplicate entry is needed. |
| C1 | COVENANT | A sidecar adds runtime artifacts beyond the main Rust WASM hash. | P3 | WP-02/06 | Open: bind loaded JavaScript and C WASM, build settings and any shim source to execution evidence. Plan now explicitly requires this. |
| C2 | COVENANT | Toolchain availability and a hash do not establish source authenticity or independent reproduction. | P3 | WP-01/10 | Open: publish obtainable identities/setup and state evidence limits; do not infer authenticity. |
| C3 | COVENANT | Dirty-source artifacts can outlive the source snapshot they are cited against. | P3 | WP-10 | Open: recheck source, runtime and package bytes at acceptance; this review does not refresh historical runtime evidence. |
| L1 | LEDGER | Separate Rust and C memories introduce an interface contract even when algorithms are unchanged. | P3 | WP-02 | Open: specify widths, array lengths, ownership, allocation failure and error transfer before integration. |
| L2 | LEDGER | A runtime compatibility profile must not silently change the meaning of existing saved backend/seed fields. | P3 | WP-02/07 | Open: record an explicit profile identity and preserve old read/replay behavior through versioned contracts. |
| L3 | LEDGER | Shared analyzer tests do not establish external GIS or practitioner-tool interoperability. | P3 | WP-07/09 | Open: pin external format/version fixtures for each advertised integration; keep broader claims absent. |
| S1 | SURVEY | The broad recommended order does not state the immediate acceptance checkpoint after the delivered slices. | P2 | Recommended order | Closed-plan: assign the coding agent, retain maintainer acceptance and define three concrete WP-01/02 outputs with an independent workspace continuation path. |
| S2 | SURVEY | A difficult C backend can absorb effort while selected-folder and portable save/load journeys remain incomplete. | P3 | WP-03 | Open: continue bounded workspace work independently if backend integration remains unresolved. |
| S3 | SURVEY | Small scoped tests do not demonstrate first-use or all-state practitioner support. | P3 | WP-08/10 | Open: demonstrate the advertised end-to-end journey and predeclared real-data/browser matrix. |
| V1 | VAULT | An imported receipt or backend profile must not authorize module execution, network access or writes on Open. | P3 | Workspace safety | Open: retain explicit execution and fresh selected-root authority in every new integration. |
| V2 | VAULT | A sidecar's generated runtime imports need review in the deployed worker context. | P3 | WP-02/10 | Open: validate required imports, asset-loading policy and error behavior; compilation alone establishes no sandbox/security claim. |
| V3 | VAULT | Reproduction logs and failed requests may retain private custom input details. | P3 | WP-06/10 | Open: retain explicit local receipt export and safe rendering; do not add automatic uploads with the backend. |

## Synthesis

Roles reviewed: **7**. Findings: **21** — **0 P1**, **3 P2**, **18 P3**. All three P2 planning issues were amended; all eighteen P3 acceptance obligations remain open. Some restate existing requirements because this follow-up does not establish their runtime completion. Counts do not measure implementation progress.

Verdict: **APPROVED-WITH-CONDITIONS** for continued implementation. Full desktop parity and release approval remain unproven.

Top finding: backend identity includes runtime/RNG behavior; compiling the same C source is insufficient for a fixed-seed equivalence claim. MERIDIAN, BENCHMARK and COVENANT converge on this. TRENCH and SURVEY also require status wording and next steps to reflect the actual delivered scope.

## Three amendments applied

1. Added runtime/RNG profile, original-baseline, sidecar artifact/interface and expanded backend-test requirements to Evidence rules.
2. Rewrote WP-09 acceptance wording to acknowledge delivered explicit county analysis and retain the broader gaps.
3. Added a bounded next checkpoint: reconcile evidence, investigate an explicit runtime profile and produce a reproducible decision record. Keep workspace work independent and broader acceptance open.

## Evidence and limits

The local `target/metis-c-sidecar-probe/evidence.json` has SHA-256 `35886666f1beb43648d391e039d40a846faa31ffb890fb06f7514b942cbe9e41`. Its recorded native packed cycle value is 521; the sidecar log reports 518. Both report disconnected-input error -4 and their own initial value after the error. That is an exact-output mismatch, not a production port. The runner is [probe_metis_c_sidecar.py](../../../scripts/web/probe_metis_c_sidecar.py), SHA-256 `e52cba214149d2c834afb7357920a347aef5ec8de0fce65235d66cb8042074cd`. Local target evidence is investigative and may not be present in a clean checkout. The precise cause and compatibility remedy require further tests; this review does not assert them proved.

Only the plan and this follow-up report were changed for this review. Links, finding counts, reviewed-plan identity and whitespace were checked. Runtime suites were not rerun for these documentation amendments; earlier runtime assertions remain subject to their original evidence and future acceptance gates. No deployment changed.
