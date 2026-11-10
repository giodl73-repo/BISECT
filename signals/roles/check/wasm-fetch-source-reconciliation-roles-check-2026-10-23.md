---
skill: roles-check
topic: wasm-fetch-source-reconciliation
date: 2026-10-23
roles_used: [MERIDIAN, CONTOUR, BENCHMARK, COVENANT, LEDGER, VAULT, SURVEY, TRENCH]
p1_count: 0
p2_count: 3
p3_count: 21
verdict: APPROVED-WITH-CONDITIONS
---

# Fetch source reconciliation and first matrix tier review

Artifact: [scope and evidence contract](../../../docs/specs/wasm-fetch-source-reconciliation.md),
[source table](../../../docs/specs/wasm-fetch-source-coverage.json) and
[predeclared matrix](../../../docs/specs/wasm-real-data-matrix.json).
Type: source reconciliation, integration fixtures and native/browser evidence.
Eight installed role perspectives were applied by the coding agent, not eight
independent reviewers. This is local scoped evidence review, not maintainer,
release, certification or full-parity acceptance.

Selection: MERIDIAN covers algorithm/seed/backend comparisons; CONTOUR input and
vintage integrity; BENCHMARK test independence and negative outcomes; COVENANT
retained evidence; LEDGER source consumers and scope; VAULT project/network
authority; SURVEY usable GUI behavior; TRENCH failure classifications and closure.

## Findings

P1 blocks shipment; P2 must be fixed for this scope; P3 should be fixed or carried.
Closed findings below apply only to the named evidence tier.

### MERIDIAN

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| M1 | Current NC/CA workloads fail native balance limits; treating matches as successful support would misstate capability. | P2 | Matrix outcomes | Closed for this tier: fixed exact error expectations, separate failure counts and disabled accepted-run downloads were checked on both engines/GUI. Successful larger runs remain open. |
| M2 | C-enabled WASM availability does not mean these metis-core workloads used C. | P3 | Backend | Explicit core backend recorded; no C execution claim. |
| M3 | Native host and WASM share adapter kernels. | P3 | Comparison | Exact complete replies are retained, with independent RI population/contiguity recomputation; independent CLI orchestration remains not-run. |

### CONTOUR

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| D1 | Planned school/EIA downloads do not provide native Run zone weights because the native spatial join is unimplemented. | P2 | Source consumers | Closed for reconciliation: source-pinned native warning/fallback and separate prepared zone observations are explicit. Browser acquisition and shared join remain open. |
| D2 | Original source hashes and historical graph receipts do not rederive the current graph. | P3 | Matrix prerequisites | Fresh derivation explicitly remains not-run; no source authenticity claim. |
| D3 | Current rows cover one vintage and one resolution. | P3 | Matrix scope | Preserve 2000/2010 and other-unit obligations; planning comparisons across vintages are not execution coverage. |

### BENCHMARK

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| B1 | A state list without pinned request/input identities cannot be an executable acceptance matrix. | P2 | Declaration / runner | Closed for first tier: raw and graph hashes, tracked recipe, request digests and expected native outcomes were fixed before both runs. |
| B2 | Missing large data must be not-run rather than synthetic replacement or a passing skip. | P3 | Runner | Explicit integration tier, missing-file incomplete status and nonzero outcome; portable source checks need no national corpus. |
| B3 | GUI acceptance requires actual requests/results, not button presence. | P3 | Browser test | Both builds' GUI requests match declared hashes; RI result matches native, NC/CA failures remain visible. Broader lifecycle is not-run. |

### COVENANT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| C1 | Mutable global native/WASM paths could silently change reference bytes during execution. | P3 | Runtime capture | Each matrix run freezes owned runtime bytes and records identities. Source-planning tests record actual native binary digests. |
| C2 | Historical acceptance records must not be rewritten for new source/test changes. | P3 | Evidence | Historical packages/snapshots retained; only the active workspace builder candidate-source pin reconciled. |
| C3 | A source/test-only tier does not justify a new release package claim. | P3 | Support | Existing toolkit bytes plus frozen engines were tested; served-asset hashes retained. New evidence snapshot is distinct from a deployed site/package release. |

### LEDGER

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| L1 | All/empty selection is only the base three families, with separately requested extras. | P3 | Source table | Forty-two exact native planning/report cases per build retain this behavior and national EIA deduplication. |
| L2 | Raw extraction, derived CSVs, cache records and graph consumers have different contracts. | P3 | Lifecycle | Source rows separate those stages; full Fetch filesystem/release semantics remain open. |
| L3 | Declaration and observed execution must not share a mutable expected-result field. | P3 | Matrix schema | Declaration refuses overwrite and stays predeclared; separate run/browser records hold observations. Revised workloads require reviewed/versioned expectations. |

### VAULT

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| V1 | Loading a real-data project must not execute its flags or restore authority. | P3 | Browser | GUI instrumentation observes no Run message on Open; RI saved result reopens unverified with accepted export disabled. |
| V2 | This tier neither broadens network allowlists nor tests provider behavior. | P3 | Acquisition | No production/network policy changed; browser blocks external traffic. New endpoint contracts remain required. |
| V3 | Hashes provide integrity identities rather than origin authentication or privacy. | P3 | Evidence | Scope states this explicitly; do not publish private project/receipt bodies under a generalized data-sharing claim. |

### SURVEY

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| S1 | Exact failure parity is useful diagnostics but does not meet practitioners' successful NC/CA workload needs. | P3 | Next work | Keep successful medium/large runs as required matrix expansion. |
| S2 | RI GUI seed, saved result and small-screen state need direct checks. | P3 | Browser | Visible seed, exact saved result, inert reopen and 390px no-overflow checked; no map/recovery/install claim from this tier. |
| S3 | Practitioner/TUI inventory remains incomplete despite engine validation progress. | P3 | WP-07 | Carry inventory, including certification/lineage semantics, before expanding controls. |

### TRENCH

| ID | Finding | Severity | Section | Recommendation / disposition |
|---|---|---|---|---|
| T1 | Source declarations can drift independently from consumer claims. | P3 | Coverage gate | All table authority files are pinned; every row references pinned native/browser/consumer files. Changed sources require reconciliation. PP-25 remains open globally. |
| T2 | Native ZIP completion predicates and parsed-unused verify flags can produce misleading apparent readiness. | P3 | Native lifecycle | Record target/marker mismatch and unused `verify_downloads`; no invented browser fix or new download-success claim. Full lifecycle tests remain open. |
| T3 | Registering a test does not prove an entire builder or package passed again. | P3 | Builders | New source gate registered and executed on both engines; amended builders syntax-checked only. Full new builder runs remain unclaimed. |

## Synthesis and amendments

Roles reviewed: 8. P1 blockers: 0. P2 issues: 3 closed for this bounded tier.
P3 notes: 21 carried or scoped safeguards retained.
Verdict: **APPROVED-WITH-CONDITIONS for source reconciliation and first evidence tier**.

Top finding: preserve native failures and missing downstream consumers instead of
promoting acquisition metadata or matching rejection into successful support.
MERIDIAN, BENCHMARK and SURVEY agree on separate success/failure outcomes;
CONTOUR and LEDGER agree on acquisition versus usable consumer semantics;
COVENANT and TRENCH agree on immutable evidence versus current source claims.

Three amendments applied:

1. Reconciled all declared types with pinned native/browser/consumer references,
   separating shared no-ops/fallbacks from missing browser acquisition.
2. Predeclared exact inputs/requests/expected outcomes; froze runtimes and retained
   native/WASM replies/errors plus actual packaged-GUI records for both builds.
3. Updated current support/handoff wording and builder registration while retaining
   not-run derivation, CLI, lifecycle, practitioner and release obligations.

No new runtime failure family is introduced by this source/test/documentation
slice. Existing PP-25 and PP-26 cover declaration/consumer and runtime-evidence
conflation; neither is promoted to solved. This review reruns no whole-release,
live-provider, national-capacity or legal certification gate.
