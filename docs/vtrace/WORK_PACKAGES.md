# Work Packages

## Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

These work packages define S4 execution units for implementing or verifying the accepted VTRACE baseline. Status values are execution states; `proposed` means the package is accepted for S4 scheduling, not completed evidence.

## Work Package Table

| ID | Objective | Parent IDs | Affected Surfaces | Entry Criteria | Exit Criteria | L0 / L1 / L2 | Status |
|---|---|---|---|---|---|---|---|
| WP-001 | Build the trace control spine and orphan checks. | M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, CO-01, CO-02, CO-10, CO-11, CO-12, REQ-001, REQ-002, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036, SPEC-001, SPEC-011, SPEC-NF-003, SPEC-NF-006, ARCH-005, IF-007, DES-001, DES-002, DES-011, PKG-001, PKG-013, CR-013 | `docs/vtrace/TRACE.md`, stage docs, trace inspection scripts if needed | S3 accepted | Every accepted ID maps to work package, verification, validation, or disposition. | L0: trace inspection / L1: n/a / L2: n/a | complete |
| WP-002 | Verify interfaces, compatibility status, and package boundaries against current repo behavior. | CO-03, REQ-003, REQ-031, REQ-037, SPEC-002, SPEC-NF-004, SPEC-NF-007, IF-001..IF-008, ARCH-001, ARCH-002, ARCH-004, PKG-003..PKG-014, DES-006, CR-001..CR-005 | `repo-map.toml`, `tools/repo_map.py`, generated Cargo config, Cargo manifests, CLI help/docs, schemas, adapters, package docs | S3 accepted | Compatibility/boundary findings recorded without unsupported pass claims. | L0: inspection / L1: targeted tests if behavior changes / L2: interoperability fixture if public adapter is claimed | complete_l0_pass_with_risk |
| WP-003 | Add or document BISECT provenance, replay, and METIS engine evidence requirements. | REQ-007..REQ-012, SPEC-005, SPEC-006, SPEC-NF-001, IF-001, IF-002, DES-004, DES-005, DES-010, PKG-007..PKG-010, CR-001, CR-002, CR-004, CR-006, CR-007 | BISECT CLI/config/run manifests/reports/docs | WP-002 interface findings known enough to avoid conflicting claims | Evidence-producing runs record config, output, executable, engine, seed/search, candidate, and environment metadata or explicit gaps. | L0: manifest inspection / L1: smoke run or unit fixture / L2: replay demonstration where release claims require it | complete_l0_pass_with_risk |
| WP-004 | Establish package-family audit fixtures and domain-semantic checks. | REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023, SPEC-007, SPEC-008, SPEC-009, IF-003, IF-004, IF-005, IF-008, DES-003, DES-007, DES-008, PKG-004, PKG-005, PKG-006, CR-001, CR-002, CR-004, CR-008, CR-009, CR-010 | RPLAN/RCOUNT/RHIST crates, schemas, fixtures, docs | Package-family current status inspected | Canonicalization, hash, audit, count lifecycle, source lineage, context/history status, and aggregation semantics are tested or dispositioned. | L0: schema/fixture inspection / L1: package tests / L2: integrated package audit where downstream use requires it | complete_l1_pass_with_risk |
| WP-005 | Inventory public claims and align research/docs/dashboard evidence status. | CO-08, REQ-005, REQ-006, REQ-024, REQ-025, REQ-026, SPEC-004, SPEC-010, SPEC-NF-005, ARCH-003, IF-006, DES-004, DES-009, PKG-012, CR-011 | `README.md`, `docs/`, `research/`, reports, dashboards | Claim surfaces selected | Reviewed claims carry posture, status, evidence pointer, assumptions, uncertainty, limitations, non-claims, or gap/stale disposition. | L0: claim inventory / L1: table/figure/source check / L2: public-release review | complete_l0_pass_with_risk |
| WP-006 | Define source custody and generated-artifact publication controls. | REQ-004, REQ-020, REQ-027, REQ-033, SPEC-003, SPEC-010, SPEC-NF-002, IF-002, IF-006, IF-008, DES-012, PKG-002, PKG-011, PKG-014 | `.gitignore`, data/output conventions, release/publication docs, archive policy | Artifact classes selected | Public, restricted, local-only, generated-transient, and archived-reference dispositions are applied before commit/release. | L0: custody inspection / L1: release artifact check / L2: public release review | complete_l0_pass_with_risk |
| WP-007 | Integrate VTRACE into waves, pulses, pitfalls, and closure gates. | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, SPEC-011, IF-007, DES-011, PKG-013, CR-013 | `context/waves/`, pulse files, review panels, pitfalls/invariants docs | S3 accepted and wave/pulse convention selected | VTRACE-governed pulses close with IDs, validation, role gates, risk/pitfall disposition, and evidence pointers. | L0: pulse checklist / L1: review panel / L2: release readiness if pulse affects public outputs | complete_l0_pass_with_risk |
| WP-008 | Update non-author workflow documentation after controls are verified. | REQ-032, SPEC-012, IF-001, IF-006, DES-011, PKG-010, PKG-012, PKG-013, CR-011 | quickstarts, CLI docs, README, evidence-location docs | WP-002 and relevant package findings available | User-facing docs describe prerequisites, commands, expected outputs, failure modes, and evidence locations without overclaiming readiness. | L0: docs inspection / L1: command smoke where feasible / L2: external-user walkthrough if release readiness is claimed | complete_l0_pass_with_risk |

## Review Lane Matrix

| Work Package | Required Lanes | Conditional Lanes | Not-Required Rationale |
|---|---|---|---|
| WP-001 | systems engineering, requirements traceability, V&V, configuration/change control | software assurance if trace checks become executable | Security/privacy and safety/risk are not required for documentation-only trace rows. |
| WP-002 | systems engineering, requirements traceability, V&V, software assurance, configuration/change control | security/privacy and source custody if dependency, external adapter, file-format, or supply-chain posture changes; safety/risk if compatibility affects public/election/legal outputs | None silently skipped; conditional lanes become required when their trigger is present. |
| WP-003 | systems engineering, requirements traceability, V&V, software assurance, safety/mission impact, source custody, configuration/change control | security/privacy if run evidence touches protected data or publication controls | Public redistricting evidence and replay claims affect civic trust and provenance. |
| WP-004 | systems engineering, requirements traceability, V&V, software assurance, safety/mission impact, source custody, configuration/change control | security/privacy if parsers, source files, dependencies, or protected data handling change | Election counts, package hashes, and context lineage carry civic/election risk. |
| WP-005 | requirements traceability, V&V, safety/mission impact, source custody, configuration/change control | software assurance if generated tables/figures/scripts change; security/privacy if claim evidence includes protected or restricted data | Public claims affect civic/legal trust even when edits are documentation-only. |
| WP-006 | requirements traceability, V&V, security/privacy, safety/mission impact, source custody, configuration/change control | software assurance if custody checks become executable | Custody/publication controls directly affect protected inputs and generated artifacts. |
| WP-007 | systems engineering, requirements traceability, V&V, software assurance, configuration/change control | safety/mission impact if pulse affects public/election/legal outputs; security/privacy/source custody if pulse touches those surfaces | Process-only changes do not require security/privacy unless they govern protected surfaces. |
| WP-008 | systems engineering, requirements traceability, V&V, safety/mission impact, configuration/change control | source custody for evidence-location docs; security/privacy if user guidance touches protected inputs; software assurance if command smoke scripts are added | Non-author workflows can affect public/civic interpretation but need security review only for triggered surfaces. |

## Work Package Details

### WP-001: Trace Control Spine

Objective: Create the trace matrix and orphan checks needed to connect mission, CONOPS, requirements, specs, interfaces, package boundaries, design decisions, code-rigor constraints, work packages, verification, validation, and review findings.

Parent mission/CONOPS IDs: M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, CO-01, CO-02, CO-10, CO-11, CO-12.

Parent requirement IDs: REQ-001, REQ-002, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036.

Parent specification IDs: SPEC-001, SPEC-011, SPEC-NF-003, SPEC-NF-006.

Architecture/boundary/package IDs: ARCH-005, PKG-001, PKG-013.

Design/interface/code-rigor IDs: DES-001, DES-002, DES-011, IF-007, CR-013.

Validation scenario IDs: CO-01, CO-02, CO-10, CO-11, CO-12.

Affected files/modules: `docs/vtrace/TRACE.md`, `docs/vtrace/VERIFICATION.md`, `docs/vtrace/VALIDATION.md`, `docs/vtrace/REVIEW.md`, wave/pulse control records.

Entry criteria:

- S3 work-package set is accepted.
- Accepted S1/S2 IDs remain stable.

Exit criteria:

- Every accepted ID is mapped or dispositioned.
- Trace gaps are findings, not ignored rows.
- Future ID changes have supersede/retire/split/merge disposition.

Verification commands:

```powershell
git --no-pager diff --check -- data\manifest.json docs\vtrace
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | Trace inspection and orphan check. | pass |
| L1 | no | Not required unless trace generation becomes executable. | not_applicable |
| L2 | no | Not required until release/readiness integration. | not_applicable |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, CO-01, CO-02, CO-10, CO-11, CO-12 | complete | Mission, operating model, and controlled development trace mapped. |
| Requirements | REQ-001, REQ-002, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036 | complete | Orphan prevention and stage control mapped. |
| Specification | SPEC-001, SPEC-011 | complete | Process/evidence control mapped. |
| Architecture / Interface | ARCH-005, PKG-001, PKG-013, IF-007 | complete | VTRACE and wave/pulse surfaces mapped. |
| Design / Code Rigor | DES-001, DES-002, DES-011, CR-013 | complete | Stable IDs and pulse closure mapped. |
| Implementation | `TRACE.md` and supporting checks | complete | Created in S4 WP-001. |
| Verification | Orphan/trace inspection | pass | Accepted baseline IDs are mapped in TRACE/S3 planning targets. |
| Validation | Maintainer review | pending | L0 validation remains user/stage review. |
| Trace | Trace row completeness | complete | Primary output created. |
| Gate | S4 WP-001 close evidence | pass | Role review not required beyond L0 for documentation-only control-plane row unless user requests another review round. |

Validation impact: Enables all later VTRACE stages.

Risks: Overly broad trace matrix could become stale; mitigate with work-package owner and orphan checks.

Assurance/security classification: process/evidence control.

### WP-002: Interface And Boundary Verification

Objective: Inspect current CLI, docs, Cargo/script dependencies, package schemas, and external adapters so compatibility and boundary status are evidence-backed.

Parent CONOPS/requirement IDs: CO-03, REQ-003, REQ-031, REQ-037.

Parent specification IDs: SPEC-002, SPEC-NF-004, SPEC-NF-007.

Architecture/boundary/package IDs: ARCH-001, ARCH-002, ARCH-004, PKG-003 through PKG-014.

Design/interface/code-rigor IDs: DES-006, IF-001 through IF-008, CR-001 through CR-005.

Validation scenario IDs: CO-03, CO-05, CO-06, CO-07, CO-09, CO-10.

Affected files/modules: `repo-map.toml`, `tools/repo_map.py`, `docs/REPO_MAP_STANDARD.md`, `Cargo.toml`, `crates/*/Cargo.toml`, CLI help/docs, schema docs, external adapter docs, package boundary docs.

Entry criteria:

- S3 accepted.
- Existing interface docs identified.

Exit criteria:

- Current, target, deprecated, and version-unknown interfaces are distinguished.
- Boundary exceptions are recorded instead of silently passing.
- Breaking-change triggers and migration expectations are documented.

Verification commands:

```powershell
cargo metadata --no-deps
python tools\repo_map.py check
cargo run -p bisect-cli -- --help
git --no-pager diff --check -- docs\vtrace
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | Interface/boundary inspection; repo-map path/generated-config check, `cargo metadata --no-deps`, and `cargo run -q -p bisect-cli -- --help` pass. | pass |
| L1 | conditional | Targeted CLI/schema tests if behavior changes. | pending |
| L2 | conditional | Interoperability fixture where public adapter compatibility is claimed. | pending |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-05, CO-03, CO-10 | complete | Safe interface evolution is now bounded by explicit command/schema/adapter status. |
| Requirements | REQ-003, REQ-031, REQ-037 | complete | Compatibility and mechanism flexibility are documented without overclaiming unsupported adapters. |
| Specification | SPEC-002 | complete | Current interfaces, remaining stubs, and version/draft surfaces are recorded; RPLAN and direct shapefile label import were later fixed through Rust-gap follow-ups. |
| Architecture / Interface | IF-001..IF-008, PKG-003..PKG-014 | complete | Dual CLI surface, package-family versions, repo-map boundary, Python support boundary, and external adapter status are recorded. |
| Design / Code Rigor | DES-006, CR-001..CR-005 | complete | Explicit errors, reviewable critical logic, and version/draft findings are recorded rather than silently passing. |
| Implementation | `repo-map.toml`, `tools/repo_map.py`, `docs/REPO_MAP_STANDARD.md`, `Cargo.toml`, `crates/bisect-cli/Cargo.toml`, `Cargo.lock`, `crates/bisect-cli/src/import_label.rs`, interface docs, package-boundary docs, and inspection records | complete | External Cargo dependencies remain git dependencies; local checkouts are selected by generated Cargo patches from `repo-map.toml`; RPLAN and direct shapefile label import are implemented in the label-import path. |
| Verification | Repo-map, CLI/help/schema/dependency/adapter inspection plus RPLAN/shapefile label-import unit tests | pass | Repo-map paths, generated Cargo config, Cargo metadata, CLI help, schema versions, implemented/stubbed adapter surfaces, and RPLAN/shapefile label import behavior were inspected or tested. |
| Validation | Operator/package-owner review | pending | User/stage review remains the validation gate before continuing to the next work package. |
| Trace | Trace rows updated | complete | WP-001 trace control supports WP-002 evidence. |
| Gate | S4 close evidence | pass_with_risk | Fixture-backed interoperability and package semantic compatibility are not WP-002 overclaims; WP-004 closes L1 package tests and leaves public interoperability fixtures as risk. |

Validation impact: Blocks unsupported compatibility claims.

Risks: Current binary may diverge from docs; mark gaps rather than editing around them without evidence.

Assurance/security classification: interface/change-control.

### WP-003: BISECT Provenance And Replay Evidence

Objective: Ensure evidence-producing BISECT runs disclose source, executable, dependency/tool, METIS engine, search, replay, output, and environment metadata or explicitly mark historical gaps.

Parent requirement IDs: REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012.

Parent specification IDs: SPEC-005, SPEC-006, SPEC-NF-001.

Boundary/package IDs: PKG-007, PKG-008, PKG-009, PKG-010.

Design/interface/code-rigor IDs: DES-004, DES-005, DES-010, IF-001, IF-002, CR-001, CR-002, CR-004, CR-006, CR-007.

Validation scenario IDs: CO-04, CO-08, CO-10.

Affected files/modules: BISECT CLI/configs/run manifests/reports, METIS engine disclosures, analysis/report docs.

Entry criteria:

- WP-002 identifies relevant CLI/config/report surfaces.
- Evidence-producing run scope is selected.

Exit criteria:

- New evidence-producing run records contain replay/provenance fields or gap status.
- METIS engine path/source/version/hash status is recorded where relevant.
- Run completion remains separate from legal readiness.

Verification commands:

```powershell
bisect show <label>
bisect label-verify <label> --year 2020
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | Manifest/report inspection plus targeted build-index provenance update. | pass_with_risk |
| L1 | conditional | Smoke run with small or existing label if data is available. | blocked_no_local_data |
| L2 | conditional | Replay demonstration for release/public evidence claims. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-07, M-08, CO-04 | partial | Label pipeline provides a SHA chain and inspectable run/report locations; full replay still depends on source data and environment evidence. |
| Requirements | REQ-007..REQ-012 | partial | Config label/year/scope/output, config hash, command, output directory, METIS engine, executable hash, adjacency hash, seed, and legal-boundary separation are present or improved; build commit/features/environment and full candidate replay remain gaps. |
| Specification | SPEC-005, SPEC-006 | partial | Reproducibility metadata is materially present for new label builds, but not complete enough for release-level replay claims. |
| Architecture / Interface | PKG-007..PKG-010, IF-001, IF-002 | partial | Build index, per-state manifest, analysis/report index, and label verification surfaces are identified. |
| Design / Code Rigor | DES-004, DES-005, DES-010, CR-001, CR-002, CR-004, CR-006, CR-007 | partial | Run completion remains separate from legal readiness; replay gaps are recorded instead of treated as success. |
| Implementation | `crates/bisect-cli/src/build_cmd.rs`, `crates/bisect-cli/src/runner.rs`, label verification and report manifest inspection | partial | New build indexes now record `config_path`, command, output directory, and METIS engine; per-state manifests hash the running executable. |
| Verification | Manifest/report inspection and targeted tests | pass_with_risk | No local census/data smoke run was available for L1 replay. |
| Validation | Analyst rebuild or review scenario | blocked | Requires local data or a selected existing label with complete artifacts. |
| Trace | Trace rows updated | partial | WP-003 L0 evidence recorded; L1/L2 remain future work. |
| Gate | S4 close evidence | pass_with_risk | MERIDIAN/COVENANT review accepts L0 improvement and records replay gaps. |

Validation impact: Supports algorithmic reproducibility and public evidence trust.

Risks: Large data may be unavailable locally; accept blocked/partial status instead of synthetic pass.

Assurance/security classification: reproducibility/legal-boundary.

### WP-004: Package-Family Audit Fixtures

Objective: Verify or implement package-family fixtures and audit semantics for RPLAN, RCOUNT, RCTX, and RHIST without collapsing domain concepts.

Parent requirement IDs: REQ-013 through REQ-023.

Parent specification IDs: SPEC-007, SPEC-008, SPEC-009.

Boundary/package IDs: PKG-004, PKG-005, PKG-006.

Design/interface/code-rigor IDs: DES-003, DES-007, DES-008, IF-003, IF-004, IF-005, IF-008, CR-001, CR-002, CR-004, CR-008, CR-009, CR-010.

Validation scenario IDs: CO-05, CO-06, CO-07.

Affected files/modules: RPLAN/RCOUNT/RCTX/RHIST crates, package schemas, fixtures, audit docs, district aggregation logic.

Entry criteria:

- Package-family current implementation status is inspected.
- Schema/canonicalization ownership is identified.

Exit criteria:

- Package audit status vocabulary and canonicalization/hash evidence are fixture-backed or gap-marked.
- RCOUNT fixtures preserve accounting concepts, lifecycle status, parser diagnostics, and source disagreement potential.
- RCTX/RHIST fixtures preserve source provenance and crosswalk/unit-history status.

Verification commands, selected only when the named crates exist and are buildable in the current workspace:

```powershell
cargo test -p rplan-core
cargo test -p rcount-core
cargo test -p rctx-core
cargo test -p rhist-core
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | Schema/fixture inspection. | pass_with_risk |
| L1 | conditional | Package-family tests where crates exist and are buildable. | pass |
| L2 | conditional | Integrated package audit for downstream/public use. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-04, M-05, CO-05, CO-06, CO-07 | partial | Package integrity and domain semantics are test-backed at crate level; public/downstream audit scenarios remain L2. |
| Requirements | REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023 | partial | RPLAN, RCOUNT, RCTX, and RHIST preserve versioning, canonical hashes, source/status concepts, and claim-boundary records; external golden fixture coverage is uneven. |
| Specification | SPEC-007, SPEC-008, SPEC-009 | partial | Package specs are implemented through crate models and tests; package schemas still need public fixture promotion before release-level interoperability claims. |
| Architecture / Interface | PKG-004..PKG-006, IF-003..IF-005 | pass_with_risk | Implemented crates and CLIs exist for RPLAN/RCOUNT/RCTX/RHIST families; some helper crates have no standalone tests. |
| Design / Code Rigor | DES-003, DES-007, DES-008, CR-001, CR-002, CR-004, CR-008..CR-010 | pass_with_risk | Tests cover package canonicalization/hash/audit/status semantics without collapsing legal/certification authority; full public audit validation remains deferred. |
| Implementation | Package fixtures/tests/docs | pass_with_risk | RPLAN has executable fixtures and public examples; RCOUNT/RCTX/RHIST rely mostly on inline synthetic package tests and CLI verification tests. |
| Verification | Audit fixtures and tests | pass | Package-family library and CLI tests pass. |
| Validation | Package audit, count audit, context verification scenarios | deferred | Requires selected downstream/public package scenario and fixture promotion. |
| Trace | Trace rows updated | partial | WP-004 evidence recorded; L2 remains future work. |
| Gate | S4 close evidence | pass_with_risk | LEDGER/CANVASS/TALLY/CONTOUR review accepts L1 package test evidence with external fixture risk. |

Validation impact: Provides civic package evidence without overclaiming authority.

Risks: Some package crates may be targets rather than implemented crates; record version-unknown or gap status rather than assuming existence.

Assurance/security classification: package/election/context integrity.

### WP-005: Public Claim And Research Evidence Review

Objective: Inventory public-facing claims and align them with evidence, assumptions, limitations, uncertainty, comparison baselines, and non-claims.

Parent CONOPS/requirement IDs: CO-08, REQ-005, REQ-006, REQ-024, REQ-025, REQ-026.

Parent specification IDs: SPEC-004, SPEC-010, SPEC-NF-005.

Architecture/boundary/package IDs: ARCH-003, PKG-012.

Design/interface/code-rigor IDs: DES-004, DES-009, IF-006, CR-011.

Validation scenario IDs: CO-08, CO-09.

Affected files/modules: `README.md`, `docs/`, `docs/papers/`, `research/`, reports, dashboards, public evidence indexes.

Entry criteria:

- Claim surfaces selected by release/research priority.
- Source data and output evidence pointers available or acknowledged as gaps.

Exit criteria:

- Reviewed claims are classified by posture and status.
- Unsupported, stale, conjectural, or partial claims are corrected or marked.
- Public reports include limitations and non-claims.

Verification commands:

```powershell
git --no-pager diff --check -- README.md docs research
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | README, paper index, paper quality review, algorithm scorecard, legal docs, and headline-claim search. | pass_with_risk |
| L1 | conditional | Table/figure/source checks for quantitative claims. | deferred |
| L2 | conditional | Public-release or hostile-review scenario. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-05, CO-08, CO-09 | partial | Public evidence discipline is now applied to the README and paper index; dashboard/legal/paper surfaces remain subject to release review. |
| Requirements | REQ-005, REQ-006, REQ-024..REQ-026 | partial | Claims are classified as empirical research, procedural/legal thesis, internal review state, or replay claim; unsupported absolute phrasing was softened or bounded. |
| Specification | SPEC-004, SPEC-010 | partial | Public/research artifacts now distinguish evidence posture from legal certification, external peer review, and release-final replay. |
| Architecture / Interface | ARCH-003, PKG-012, IF-006 | partial | README, `docs/PAPERS.md`, paper quality review, scorecard, legal docs, and headline claim patterns were inspected as the L0 public evidence corpus. |
| Design / Code Rigor | DES-004, DES-009, CR-011 | pass_with_risk | Run completion, public evidence, legal readiness, and internal review state are separated in the updated public docs. |
| Implementation | `README.md`, `docs/PAPERS.md`, claim-review records | pass_with_risk | README now bounds byte-level replay, geographic/VRA inputs, compactness causality, fairness language, runtime, headline metrics, and legal-thesis wording; paper index clarifies internal review labels. |
| Verification | Evidence pointer and consistency review | pass_with_risk | L0 inventory found and bounded high-risk claims; exact quantitative table/figure recomputation remains L1/L2. |
| Validation | Public/hostile review scenario | deferred | Requires selected public release, dashboard snapshot, and figure/table source checks. |
| Trace | Trace rows updated | partial | WP-005 evidence recorded; later release-level claim validation remains WP-008/S5/S6 work. |
| Gate | S4 close evidence | pass_with_risk | DATUM/SCALE/PRECINCT/COMMONS review accepts L0 claim-control improvements while preserving L1/L2 quantitative/public-release risk. |

Validation impact: Prevents public outputs from overstating evidence.

Risks: Full paper-by-paper inventory is large; prioritize headline/README/dashboard/legal claims first.

Assurance/security classification: public evidence/research.

### WP-006: Source Custody And Artifact Publication Controls

Objective: Apply custody and publication disposition to source data, generated outputs, protected inputs, intermediate artifacts, packages, reports, PDFs, maps, dashboards, and archives.

Parent requirement IDs: REQ-004, REQ-020, REQ-027, REQ-033.

Parent specification IDs: SPEC-003, SPEC-010, SPEC-NF-002.

Boundary/package IDs: PKG-002, PKG-011, PKG-014.

Design/interface/code-rigor IDs: DES-012, IF-002, IF-006, IF-008, CR-012.

Validation scenario IDs: CO-06, CO-07, CO-09.

Affected files/modules: `.gitignore`, data/output conventions, release/publication docs, archive policy, source manifests.

Entry criteria:

- Artifact classes and publication targets selected.
- Existing `.gitignore` and generated-output conventions inspected.

Exit criteria:

- Artifact disposition categories are applied before commit/release.
- Protected/restricted artifacts are excluded, redacted, access-controlled, or explicitly dispositioned.
- Archived forensic references remain read-only unless promoted by boundary decision.

Verification commands:

```powershell
git --no-pager status --short
git --no-pager diff --check -- docs\vtrace
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | `.gitignore`, tracked artifact inventory, shallow local artifact inventory, data/output policy files, archive boundary, and active git remote inspection. | pass_with_risk |
| L1 | conditional | Release artifact inspection. | deferred |
| L2 | conditional | Public-release privacy/source-custody review. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-05, CO-06, CO-09 | partial | Repository-level custody categories are explicit; selected release bundles still require artifact-by-artifact review. |
| Requirements | REQ-004, REQ-020, REQ-027, REQ-033 | partial | Raw/source data, generated run outputs, reports, dashboards, PDFs/maps, package artifacts, local patches, and archives have L0 disposition rules. |
| Specification | SPEC-003, SPEC-010 | partial | Commit/publication defaults are documented; release-specific manifests, redaction, and source-pointer review remain L1/L2. |
| Architecture / Interface | PKG-002, PKG-011, PKG-014, IF-002, IF-006, IF-008 | pass_with_risk | `.gitignore`, `data/` policy files, `outputs/README.md`, docs/research artifact exceptions, `.cargo/config.toml`, and `archive/python-pipeline-final/` were inspected. |
| Design / Code Rigor | DES-012, CR-012 | pass_with_risk | Generated artifacts default to ignored/local-only unless promoted as docs/research/public evidence with claim and custody review. |
| Implementation | Custody docs and checks | pass_with_risk | Updated embedded `data/manifest.json` to the active `giodl73-repo/BISECT` repository and recorded current custody rules in VTRACE docs. |
| Verification | Git/release/custody inspection | pass_with_risk | Tracked artifact inventory shows intentionally tracked data policy files, docs PDFs/images/site outputs, `outputs/README.md`, and research figures; local data/output/report directories remain ignored or generated. |
| Validation | Public verification scenario | deferred | Requires selected release artifact set and privacy/source-custody review before publication. |
| Trace | Trace rows updated | partial | WP-006 evidence recorded; release-level custody validation remains WP-008/S5/S6 work. |
| Gate | S4 close evidence | pass_with_risk | VAULT/COVENANT review accepts L0 repository custody controls while preserving release-specific L1/L2 risk. |

Validation impact: Protects repository hygiene, privacy, and source licensing boundaries.

Risks: Some generated artifacts are intentionally committed under `docs/`; custody rules must distinguish allowed public evidence from transient generated output.

Assurance/security classification: source custody/privacy.

### WP-007: Wave/Pulse Closure And Pitfall Controls

Objective: Integrate VTRACE requirements into wave/pulse execution so future work closes with trace, validation, role gates, risk disposition, and pitfall coverage.

Parent requirement IDs: REQ-028, REQ-029, REQ-030, REQ-034, REQ-035.

Parent specification IDs: SPEC-011, SPEC-NF-006.

Boundary/package IDs: PKG-013.

Design/interface/code-rigor IDs: DES-011, IF-007, CR-013.

Validation scenario IDs: CO-10.

Affected files/modules: `context/waves/`, pulse plans, review panels, pitfalls/invariants documentation.

Entry criteria:

- S3 accepted.
- Wave/pulse convention identified in `context/waves/PHASES.md`.

Exit criteria:

- Pulse close condition includes requirements, boundaries, validation, role gates, risks, and pitfall status.
- Verification states distinguish pass, fail, blocked, partial, deferred, and not applicable.

Verification commands:

```powershell
git --no-pager diff --check -- context docs\vtrace
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | `context/waves/PHASES.md` inspection plus pulse/fork sample inspection. | pass_with_risk |
| L1 | conditional | Review panel for first governed pulse. | deferred |
| L2 | conditional | Release readiness if pulse affects public outputs. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-07, CO-10 | complete | Execution governance carried through wave/pulse rules. |
| Requirements | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035 | complete_l0 | Wave/pulse closure states now require IDs, validation, risk, and pitfall/custody disposition. |
| Specification | SPEC-011 | complete_l0 | Process execution standard updated. |
| Architecture / Interface | PKG-013, IF-007 | complete_l0 | Wave/pulse records remain the execution-control boundary. |
| Design / Code Rigor | DES-011, CR-013 | complete_l0 | Closure discipline recorded in wave rules and VTRACE docs. |
| Implementation | Pulse checklist standard | complete_l0 | `context/waves/PHASES.md` updated rather than retrofitting archived pulses. |
| Verification | Pulse evidence inspection | pass_with_risk | Existing pulses name roles/validation but predate full VTRACE ID closure. Future governed pulses must carry full fields. |
| Validation | Maintainer/reviewer acceptance | deferred | First future VTRACE-governed pulse or release-affecting pulse should receive L1 review. |
| Trace | Trace rows updated | complete | WP-001 trace already maps WP-007; this section records closure. |
| Gate | S4 close evidence | pass_with_risk | L0 process standard complete; L1/L2 dry-run deferred. |

Validation impact: Converts documentary VTRACE adoption into live execution control for future governed pulses.

Risks: Existing archived pulses are not retroactively rewritten, and there is currently no active wave in `context/waves/PHASES.md`; first future VTRACE-governed pulse must prove the checklist in practice before L1 closure.

Assurance/security classification: process/change control.

### WP-008: Non-Author Workflow Documentation

Objective: Update user-facing workflow documentation after controls are verified so non-authors can run, inspect, and interpret outputs without overclaiming readiness.

Parent requirement IDs: REQ-032.

Parent specification IDs: SPEC-012.

Boundary/package IDs: PKG-010, PKG-012, PKG-013.

Design/interface/code-rigor IDs: DES-011, IF-001, IF-006, CR-011.

Validation scenario IDs: CO-03, CO-04, CO-08, CO-09, CO-10.

Affected files/modules: `README.md`, `docs/BISECT_CLI.md`, quickstarts, concept docs, evidence-location docs.

Entry criteria:

- Relevant interface and evidence findings from WP-002, WP-003, WP-005, and WP-006 are available.

Exit criteria:

- Non-author workflows document prerequisites, commands, expected outputs, failure modes, and evidence locations.
- User-facing guidance distinguishes run completion, legal readiness, public evidence, and package audit status.

Verification commands:

```powershell
git --no-pager diff --check -- README.md docs
```

Validation levels:

| Level | Required | Commands / Evidence | Result |
|---|---|---|---|
| L0 | yes | Documentation inspection. | pass |
| L1 | conditional | Command smoke test where feasible. | deferred |
| L2 | conditional | External-user walkthrough for release/readiness. | deferred |

V closure:

| V Area | IDs / Evidence | Status | Notes |
|---|---|---|---|
| Need / CONOPS | M-03, M-07, CO-03, CO-04, CO-08, CO-09 | complete_l0 | User-facing docs now distinguish label-pipeline use, evidence packages, and non-certification. |
| Requirements | REQ-032 | complete_l0 | Non-author workflows are routed through current lowercase label commands. |
| Specification | SPEC-012 | complete_l0 | Docs/ops examples use `bisect build`, `label-analyze`, `label-report`, `label-verify`, and `label-import`. |
| Architecture / Interface | PKG-010, PKG-012, IF-001, IF-006 | complete_l0 | Public docs acknowledge legacy surfaces where documented but direct non-author flows to label-pipeline commands. |
| Design / Code Rigor | DES-011, CR-011 | complete_l0 | Public-claim discipline now states outputs are evidence packages, not official/legal certification. |
| Implementation | README/docs updates | complete_l0 | `docs/BISECT_CLI.md` and quickstarts were repaired for stale uppercase command paths and current import formats. |
| Verification | Docs inspection and stale-command search | pass | Search confirms targeted stale uppercase workflow commands are removed from user-facing quickstarts/CLI reference. |
| Validation | User workflow scenario | deferred | External-user walkthrough remains a release-readiness activity. |
| Trace | Trace rows updated | complete | WP-001 dependency remains valid; no new requirements were added. |
| Gate | S4 close evidence | pass_with_risk | COMMONS risk is limited to future external-user dry run and full release smoke tests. |

Validation impact: Makes the platform usable by special masters, researchers, state staff, auditors, and reviewers.

Risks: External-user walkthrough and release-bundle smoke tests remain deferred; current closure is L0 documentation alignment, not a public-readiness certification.

Assurance/security classification: operational documentation.

## Release-Readiness DCRs

The S4 work packages are satisfied at the validation levels recorded above. The remaining release-readiness work is tracked as DCRs rather than reopened WP items.

| DCR | Follow-On Objective | Source WP / Risk | Exit Gate |
|---|---|---|---|
| DCR-001 | Promote public golden interop fixtures for CSV, GeoJSON, RPLAN, and shapefile label import/package claims. | WP-002, WP-004 | L2 interoperability fixture review. |
| DCR-002 | Define and run a small release smoke bundle covering build/analyze/report/verify. | WP-003, WP-008 | L1 release-smoke evidence. |
| DCR-003 | Perform a non-author walkthrough of current quickstarts and CLI docs. | WP-008 | L2 user-workflow review. |
| DCR-004 | Lock public evidence package layout, manifest, limits, custody, and review-state contract. | WP-005, WP-006 | L2 public artifact/custody review. |
| DCR-005 | Publish a central import/export compatibility matrix with format versions, fields, and fixture status. | WP-002 | L1 interface review. |
| DCR-006 | Define the boundary between generated evidence packages and court/legal filing packages. | WP-005, WP-006 | L2 legal-boundary review. |
| DCR-007 | Run a full-scale or declared release-subset reproducibility scenario. | WP-003 | L2 reproducibility review. |

DCR records are filed in `docs/vtrace/DCRS.md` and traced in `docs/vtrace/TRACE.md`.

## Orphan Check

Before implementation starts, confirm:

- [x] Every accepted `REQ-*` is assigned to a work package or dispositioned.
- [x] Every accepted `SPEC-*` is assigned to a work package, verification item, or dispositioned.
- [x] Every interface-changing work package names `IF-*` IDs.
- [x] Every package/crate/module-changing work package names `PKG-*` boundary IDs.
- [x] Every critical-code work package names `CR-*` IDs.
- [x] Every work package has exit criteria and verification commands or inspection evidence.
- [x] Every work package lists L0/L1/L2 requirements or explicit non-requirement.
- [x] Every work package has V closure rows completed or marked `n/a` with rationale.
- [x] Every required assurance/security review lane is complete or accepted with risk.
- [x] No work package is only "cleanup" without parent IDs or discovery status.

## S3 Gate

Decision: accepted with risk after `.roles` review.

Before S3 lock:

- [x] Review confirms the eight-work-package set is complete enough for S4 execution.
- [x] Review confirms no accepted requirement is orphaned or only implicitly covered.
- [x] Review confirms verification commands are realistic and do not require unavailable data unless marked conditional.
- [x] Review confirms role lanes and risk dispositions are sufficient for high-stakes public/election/legal evidence.
