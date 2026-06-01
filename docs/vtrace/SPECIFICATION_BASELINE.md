# Specification Baseline

## SB-00 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Baseline type: mixed current/target baseline for an existing repo.

Baseline date: 2026-05-31.

This baseline records the controlled behavior and contracts implied by locked Mission, CONOPS, and Requirements artifacts. It does not claim the current code already satisfies every target item; unknowns and deferred work are called out explicitly.

## SB-01 Specification Sources

| Source | Evidence | Status | Notes |
|---|---|---|---|
| README / docs | `README.md`, `docs/BISECT_CLI.md`, `docs/PAPERS.md`, `research/README.md`, `research/journals/README.md` | current / target | Current public claims and commands exist, but VTRACE review will decide which claims are publication-ready. |
| Workspace manifest | `Cargo.toml` workspace members and dependencies | current | Establishes Rust crate families for BISECT, RPLAN, RCOUNT, RCTX, RHIST, and shared kernels. |
| VTRACE baseline | `docs/vtrace/MISSION.md`, `docs/vtrace/CONOPS.md`, `docs/vtrace/REQUIREMENTS.md` | target | Defines accepted mission, operating scenarios, and locked requirement IDs. |
| Wave/pulse model | `context/waves/PHASES.md` and repo wave materials | current / target | Existing execution model remains the control path for future work. |
| DCRs | `docs/vtrace/DCRS.md` | filed | Release-readiness controls are recorded as DCR-001 through DCR-007 and are not treated as already-closed S4 evidence. |
| CLI/API behavior | `bisect` commands and package CLIs | current / unknown | CLI surfaces are documented, but exact current/target compatibility must be verified during interface stage. |
| Tests / fixtures | Rust and Python tests where present | unknown | This stage does not inventory all test coverage. Verification stage will map coverage. |
| Released package / downstream use | Public docs, committed papers, generated dashboards, package crates | unknown | Existing downstream use is not fully inventoried in this stage. |

## SB-02 Controlled Specification Items

| Spec ID | Parent REQ IDs | Type | Current / Target / Deprecated / Unknown | Specification Statement | Verification Method | Validation Method | Owner | Risk | Status |
|---|---|---|---|---|---|---|---|---|---|
| SPEC-001 | REQ-001, REQ-002, REQ-036 | process / evidence | target | VTRACE artifacts under `docs/vtrace/` SHALL connect mission, CONOPS, requirements, specification, trace, verification, validation, review, and evidence without orphan accepted requirements. | trace review / inspection | maintainer review | Maintainers | medium | accepted |
| SPEC-002 | REQ-003, REQ-031, REQ-037 | interface / compatibility | target | Controlled public interfaces SHALL declare versions, version-unknown status, or compatibility rules and SHALL require change-control when CLI, schema, config, report, package, external-standard mapping, or documentation contracts break. | inspection / review | operator review | Package owners / LEDGER | high | accepted |
| SPEC-003 | REQ-004, REQ-020, REQ-026, REQ-027, REQ-033 | custody / privacy | target | Generated data, large source data, run outputs, protected inputs, and intermediate sensitive artifacts SHALL have custody, publication, redaction, exclusion, or access-control disposition before commit or release. | inspection / review | public-review scenario | Maintainers / VAULT | high | accepted |
| SPEC-004 | REQ-005, REQ-006, REQ-024, REQ-025 | evidence / research | target | Public claims SHALL be classified by posture and evidence status, trace to data/commands/packages/tables/figures/review state, and state assumptions, uncertainty, comparison baseline, political/community-effect scope, or unsupported-gap disposition. | review / analysis | hostile-review scenario | Research owners / DATUM / SCALE / PRECINCT / COMMONS | high | accepted |
| SPEC-005 | REQ-007, REQ-008, REQ-009, REQ-012 | software / reproducibility | target | Evidence-producing redistricting runs SHALL record run configuration, output locations, source-data hashes or custody pointers, executable provenance, external tool/subprocess disclosures, deterministic replay inputs, selected candidate metadata, and inspectable outputs. | demonstration / inspection | analyst rebuild scenario | BISECT owners / MERIDIAN / COVENANT | high | accepted |
| SPEC-006 | REQ-010, REQ-011 | legal-boundary / review | target | Redistricting workflows SHALL keep run completion separate from legal readiness and SHALL record federal, state, chamber, VRA, population, contiguity, subdivision, nesting, and jurisdiction-specific gates independently. | review / inspection | legal-review scenario | BISECT legal-doc owners / BOUNDARY / WARD | high | accepted |
| SPEC-007 | REQ-013, REQ-014, REQ-015 | package / audit | target | RPLAN packages SHALL carry district assignments, geography references, source metadata, manifests, schema versions, audit status, canonicalization/hash controls, and explicit package findings. | test / demonstration | package-audit scenario | RPLAN owners / LEDGER | high | accepted |
| SPEC-008 | REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-023 | package / election-count | target | RCOUNT packages and workflows SHALL preserve election accounting semantics, raw-source custody and parser diagnostics, lifecycle event history, reconciliation/replay evidence, official-certification boundaries, jurisdiction variation, and verified context links for district aggregation. | test / review | count-audit scenario | RCOUNT owners / CANVASS / TALLY | high | accepted |
| SPEC-009 | REQ-021, REQ-022, REQ-023, REQ-033 | context / history | target | RCTX/RHIST workflows SHALL preserve source provenance and report verified, partial, conjectural, blocked, or gap status for crosswalks, geography lineage, and dependent aggregations. | test / inspection | context-verification scenario | RCTX/RHIST owners / CONTOUR | high | accepted |
| SPEC-010 | REQ-026, REQ-027 | public artifact | target | Public verification artifacts SHALL include evidence indexes, manifests or source pointers, limitations, non-claims, uncertainty where applicable, review status, threat-model/non-goal notes where security or privacy is implied, and source-custody/privacy disposition. | inspection / review | public-verification scenario | Report/dashboard owners / VAULT | high | accepted |
| SPEC-011 | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035 | process / wave-pulse | target | Wave/pulse execution SHALL name affected requirements, package boundaries, validation commands, role gates, risk dispositions, verification states, and pitfall status with recurrence coverage. | inspection / trace review | pulse-review scenario | Wave/pulse owners / BENCHMARK / TRENCH | medium | accepted |
| SPEC-012 | REQ-032 | documentation / ops | target | Non-author workflows SHALL document prerequisites, commands, expected outputs, failure modes, and evidence locations for primary user roles. | demonstration / inspection | user-workflow scenario | Documentation owners / COMMONS | medium | accepted |

## SB-03 Public Contracts

| Contract ID | Spec IDs | Surface | Compatibility Rule | Change-Control Trigger | Verification Evidence |
|---|---|---|---|---|---|
| IF-001 | SPEC-005, SPEC-006 | `bisect` CLI commands, flags, config labels, and output paths | Compatibility is not claimed until interface inspection records current binary/help behavior; after a command is accepted as stable, breaking changes require explicit break disposition and migration guidance. | Command/flag rename, default change, output path/schema change, or changed legal-readiness claim. | CLI help/doc inspection, smoke run, report verification. |
| IF-002 | SPEC-005 | BISECT YAML/TOML/JSON configs and generated run manifests | Version or schema identity must be recorded before compatibility claims; version unknown is an explicit temporary status, not compatibility. | Field rename/removal, default change, semantic reinterpretation, or new required field. | Schema/config inspection and replay demonstration. |
| IF-003 | SPEC-007 | RPLAN package manifests, canonical bytes, hashes, geography references, and audit results | Canonicalization and hash algorithm changes require explicit versioning; version unknown is an audit finding until resolved. | Schema, canonicalization, hash domain, or completeness-rule change. | Package audit test/demo. |
| IF-004 | SPEC-008 | RCOUNT package manifests, accounting records, reconciliation ledgers, lifecycle status, and aggregation outputs | Election accounting semantics must not be flattened across versions; raw-source format adapter versions and parser warnings remain part of the contract. | Contest/ballot/batch/CVR/lifecycle field semantics change or source-format adapter change. | Count audit test/review. |
| IF-005 | SPEC-009 | RCTX/RHIST source manifests, crosswalks, unit lineage, and validation status | Source lineage and confidence/status vocabulary must remain explicit; version unknown blocks strong compatibility claims. | Source-data contract, crosswalk semantics, lineage model, or status vocabulary change. | Context validation fixture or inspection. |
| IF-006 | SPEC-004, SPEC-010 | Public reports, dashboards, papers, evidence indexes, and claim reviews | Published claims must carry evidence status and limitations; stale claims require correction or gap status. | New headline claim, changed number, changed evidence package, or public release. | Claim review finding and evidence pointer. |
| IF-007 | SPEC-001, SPEC-011 | VTRACE docs, trace rows, review findings, and wave/pulse records | Accepted IDs are stable; retired or superseded IDs require trace-preserving disposition. | Requirement/spec/interface/design/verification/validation change after acceptance. | Trace-matrix review. |
| IF-008 | SPEC-002, SPEC-007, SPEC-008, SPEC-009 | External standard mappings and interchange claims, including GeoJSON, shapefile, Census/TIGER, NIST/CDF where applicable, GerryChain, DRA, PlanScore, and package-family adapters | External compatibility claims must name the external format or schema version, required field/geometry semantics, and known gaps. | New or changed import/export target, external tool version change, coordinate/reference-system assumption, or schema mismatch. | Interoperability inspection or fixture. |

## SB-04 Package / Language Allocation

| Spec IDs | Package / Crate / Module / Language | Responsibility | Forbidden Responsibility | Validation Profile |
|---|---|---|---|---|
| SPEC-005, SPEC-006 | `crates/bisect-*`, configs, reports, maps, Rust | Build, analyze, report, verify, and explain algorithmic plans. | Claim legal readiness without separate gates; hide provenance or seed/search metadata. | L0/L1/L2 |
| SPEC-007 | `crates/rplan-*`, Rust | Plan package schema, IO, audit, CLI, canonicalization, and integrity evidence. | Certify legal plan validity by package integrity alone. | L0/L1 |
| SPEC-008 | `crates/rcount-*`, Rust | Count package schema, IO, audit, statistics, district aggregation, reconciliation, lifecycle status. | Replace official certification authority; flatten election accounting concepts. | L0/L1/L2 |
| SPEC-009 | `crates/rctx-*`, `crates/rhist-*`, Rust | Context packages, geography provenance, crosswalk validation, and historical unit lineage. | Treat unknown lineage as verified. | L0/L1/L2 |
| SPEC-004, SPEC-010, SPEC-012 | `docs/`, `research/`, `reports/`, dashboard/report generators, LaTeX/Python/Rust | Public explanation, papers, reports, evidence indexes, and non-author workflows. | Publish protected data or unsupported high-stakes claims. | inspection / review |
| SPEC-011 | `context/waves/`, `docs/vtrace/`, pulse files | Controlled execution, work packages, pitfalls, and review gates. | Complete scoped work without verification or risk disposition. | review / trace |
| SPEC-003 | `.gitignore`, data/output conventions, release/publication process | Protect raw/generated/protected data and define public-evidence promotion. | Commit protected or unintended generated artifacts. | inspection / review |
| SPEC-002 | interface inventories, package specs, external-standard adapters | Name controlled versions, unknown-version findings, compatibility rules, and external format assumptions. | Claim compatibility without a named contract, version, or explicit unknown status. | inspection / fixture |

## SB-05 Nonfunctional Constraints

| Constraint ID | Parent Spec IDs | Constraint | Threshold / Rule | Verification Method | Status |
|---|---|---|---|---|---|
| SPEC-NF-001 | SPEC-005, SPEC-007, SPEC-008, SPEC-009 | Reproducibility | Evidence-producing workflows record enough provenance, source custody, deterministic inputs, and external-tool disclosures for independent replay or clearly mark the gap. | demonstration / inspection | accepted |
| SPEC-NF-002 | SPEC-003, SPEC-008, SPEC-010 | Privacy and custody | Public artifacts do not expose protected inputs, ballot-choice receipts, reidentification risks, or coercion channels. | review / inspection | accepted |
| SPEC-NF-003 | SPEC-001, SPEC-011 | Traceability | Accepted requirements map to specs, verification method, validation scenario, and review/evidence status or a recorded deferral. | trace review | accepted |
| SPEC-NF-004 | SPEC-002, SPEC-007, SPEC-008, SPEC-009 | Compatibility | Breaking interface/package/schema changes require versioning, migration notes, or explicit retirement. | review / test | accepted |
| SPEC-NF-005 | SPEC-004, SPEC-010 | Claim discipline | High-stakes public claims carry posture, evidence status, uncertainty/assumptions, and non-claims. | review / analysis | accepted |
| SPEC-NF-006 | SPEC-011 | Pitfall control | Known recurring failures use OPEN, MITIGATED, or SOLVED status and have prevention/validation coverage. | inspection / trace review | accepted |
| SPEC-NF-007 | SPEC-002, SPEC-007, SPEC-008, SPEC-009 | Interoperability honesty | External compatibility claims name the standard/tool/schema version and do not imply universal compatibility from a partial adapter. | inspection / fixture | accepted |

## SB-06 Assumptions And Unknowns

| ID | Item | Impact | Disposition | Owner |
|---|---|---|---|---|
| SPEC-UNK-001 | Exact RPLAN/RCOUNT/RCTX/RHIST schemas and canonicalization algorithms are not restated in this baseline. | Interface and verification stages need package-family detail. | defer to INTERFACES / package specs | Package owners |
| SPEC-UNK-002 | Public artifact commit policy is not fully specified for every PDF, dashboard, report, package, or evidence bundle. | Publication workflow may remain inconsistent. | defer to VERIFICATION/VALIDATION and public-release rules | Maintainers / VAULT |
| SPEC-UNK-003 | Paper-by-paper evidence completeness is not inventoried. | Research claim review may find stale or unsupported claims. | discovery during TRACE / REVIEW | Research owners / DATUM |
| SPEC-UNK-004 | Current CLI docs may not exactly match current binary behavior across all commands. | Interface contracts may need correction before lock. | discovery during INTERFACES and VERIFICATION | BISECT owners |
| SPEC-UNK-005 | Existing generated outputs may lack full executable provenance and deterministic replay metadata. | Historical evidence may need partial/gap status. | accept risk for historical artifacts; require target behavior going forward | Evidence owners |
| SPEC-UNK-006 | First live wave/pulse governed by this VTRACE baseline is not selected. | Adoption remains documentary until a controlled pulse uses it. | decide after specification/trace review | Maintainers |
| SPEC-UNK-007 | Exact external format and tool versions for current import/export compatibility claims are not inventoried. | Compatibility claims could be overbroad or stale. | defer to INTERFACES and interoperability fixtures | Package owners / LEDGER |
| SPEC-UNK-008 | S4 satisfaction does not by itself define release, court, public evidence, or full reproducibility readiness. | Users could mistake pass-with-risk closure for release certification. | controlled by DCR-001 through DCR-007 before S6 readiness claims | Maintainers / role lanes |

## SB-07 Requirement-To-Spec Coverage

| Requirement ID | Spec IDs | Coverage Status | Notes |
|---|---|---|---|
| REQ-001 | SPEC-001 | covered | VTRACE artifact set. |
| REQ-002 | SPEC-001 | covered | Parent trace rule. |
| REQ-003 | SPEC-002 | covered | Versioned controlled interfaces. |
| REQ-004 | SPEC-003 | covered | Data/output custody. |
| REQ-005 | SPEC-004 | covered | Claim posture. |
| REQ-006 | SPEC-004 | covered | Quantitative claim evidence. |
| REQ-007 | SPEC-005 | covered | Run metadata. |
| REQ-008 | SPEC-005 | covered | Executable provenance. |
| REQ-009 | SPEC-005 | covered | Search replay metadata. |
| REQ-010 | SPEC-006 | covered | Run completion vs legal readiness. |
| REQ-011 | SPEC-006 | covered | Separate legal gates. |
| REQ-012 | SPEC-005 | covered | Inspectable outputs. |
| REQ-013 | SPEC-007 | covered | RPLAN contents. |
| REQ-014 | SPEC-007 | covered | Controlled package audit. |
| REQ-015 | SPEC-007 | covered | RPLAN findings. |
| REQ-016 | SPEC-008 | covered | RCOUNT semantics. |
| REQ-017 | SPEC-008 | covered | Lifecycle states. |
| REQ-018 | SPEC-008 | covered | Certification boundary. |
| REQ-019 | SPEC-008 | covered | Reconciliation/replay. |
| REQ-020 | SPEC-003, SPEC-008 | covered | Public count privacy disposition. |
| REQ-021 | SPEC-009 | covered | Source provenance. |
| REQ-022 | SPEC-009 | covered | Context validation status. |
| REQ-023 | SPEC-008, SPEC-009 | covered | Count-to-district aggregation. |
| REQ-024 | SPEC-004 | covered | Research claim trace. |
| REQ-025 | SPEC-004 | covered | Claim classification/status. |
| REQ-026 | SPEC-010 | covered | Public evidence index. |
| REQ-027 | SPEC-003, SPEC-010 | covered | Public artifact custody review. |
| REQ-028 | SPEC-011 | covered | Wave/pulse scope. |
| REQ-029 | SPEC-011 | covered | Requirement/pitfall recurrence coverage. |
| REQ-030 | SPEC-011 | covered | Pitfall status. |
| REQ-031 | SPEC-002 | covered | Compatibility/migration. |
| REQ-032 | SPEC-012 | covered | Non-author workflows. |
| REQ-033 | SPEC-003, SPEC-009 | covered | Source acquisition/transformation provenance. |
| REQ-034 | SPEC-011 | covered | Verification result states. |
| REQ-035 | SPEC-011 | covered | Role finding dispositions. |
| REQ-036 | SPEC-001 | covered | Trace matrix scope. |
| REQ-037 | SPEC-002 | covered | Avoid unnecessary mechanism mandates. |

## SB-08 Spec-To-Verification Coverage

| Spec ID | Verification IDs / Commands | Expected Result | Evidence Pointer | Status |
|---|---|---|---|---|
| SPEC-001 | planned VER-001 trace inspection | Accepted VTRACE artifacts have stable IDs and no orphan accepted requirement. | `docs/vtrace/TRACE.md` planned | planned |
| SPEC-002 | planned VER-002 interface inspection | Controlled interfaces list owner, version or version-unknown status, compatibility rule, external-standard mapping where claimed, and change-control trigger. | `docs/vtrace/INTERFACES.md` planned | planned |
| SPEC-003 | planned VER-003 custody inspection | Public artifacts and protected/generated data have recorded disposition. | `docs/vtrace/VERIFICATION.md` planned | planned |
| SPEC-004 | planned VER-004 claim review | Reviewed claims have posture, evidence status, uncertainty/assumptions, and evidence pointer or gap. | review findings planned | planned |
| SPEC-005 | planned VER-005 BISECT smoke/replay demonstration | A representative run records config, command, source-data hashes or custody pointers, executable provenance, external-tool disclosures, seed/search metadata, outputs, and verification status. | command evidence planned | planned |
| SPEC-006 | planned VER-006 legal-gate inspection | Legal-readiness gates remain separate and do not collapse into run success. | review finding planned | planned |
| SPEC-007 | planned VER-007 RPLAN audit fixture | RPLAN package audit checks integrity, completeness, and finding status. | fixture/test evidence planned | planned |
| SPEC-008 | planned VER-008 RCOUNT audit fixture | RCOUNT package audit preserves accounting semantics, raw-source custody, parser diagnostics, lifecycle event history, reconciliation, jurisdiction variation, and certification boundary. | fixture/test evidence planned | planned |
| SPEC-009 | planned VER-009 context/history fixture | RCTX/RHIST validation records provenance and status for crosswalks/lineage. | fixture/test evidence planned | planned |
| SPEC-010 | planned VER-010 public-artifact inspection | Public artifacts include evidence index, limitations, non-claims, threat-model/non-goal notes where relevant, and custody/privacy disposition. | artifact review planned | planned |
| SPEC-011 | planned VER-011 pulse review | A wave/pulse records parent IDs, validation commands, role gates, risk, verification state, and pitfall coverage. | pulse evidence planned | planned |
| SPEC-012 | planned VER-012 workflow demonstration | Non-author workflow docs can be followed to expected outputs or clear failure modes. | demonstration planned | planned |

## SB-09 Specification Gate

Decision: accepted and locked after `.roles` review.

Required before implementation planning:

- [x] Every accepted `REQ-*` maps to one or more `SPEC-*` IDs or a recorded deferral.
- [x] Every baseline-level implementation surface can name parent `SPEC-*` IDs or discovery status; individual work packages inherit this gate when created.
- [x] Public contracts have owners and change-control triggers at the baseline level.
- [x] Unknowns are resolved, blocked, deferred, accepted as risk, or converted to discovery work.
- [x] Verification and validation methods are credible for the controlled claim.

Rationale: This baseline passed `.roles` review after lock-state cleanup. The `SPEC-*` and `SPEC-NF-*` IDs are stable for downstream trace, interface, verification, validation, and wave/pulse planning.
