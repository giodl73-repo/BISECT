# Code Rigor

## Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Risk level: high for public evidence, package integrity, election-count semantics, legal-boundary statements, and reproducibility claims; medium for ordinary documentation and local support tooling.

Language/toolchain: Rust workspace crates, Python support scripts, LaTeX/research sources, Markdown documentation, YAML/TOML/JSON configs, generated reports/maps/packages.

## Coding Constraints

| ID | Constraint | Applies To | Verification | Exception Rule |
|---|---|---|---|---|
| CR-001 | Critical hand-authored functions should stay small enough for focused review; default soft cap is 60 logical lines unless tailored. | Rust/Python critical logic | size/complexity inspection and code review | Larger units require rationale and focused tests or review evidence. |
| CR-002 | Complex control flow should be bounded, tested, or justified. | Algorithms, parsers, reconciliation, search, package audit | design inspection and tests | Record why complexity is necessary and how recurrence is caught. |
| CR-003 | Public interfaces should handle invalid inputs and errors explicitly. | CLIs, configs, schemas, file formats, package import/export | interface tests, fixtures, and review | Waive only for impossible states with rationale. |
| CR-004 | Critical invariants should have assertions, checks, property tests, golden fixtures, or inspection evidence. | Algorithms, state transitions, canonicalization, hashes, election accounting | tests and review | Explain if invariant is enforced by type/system boundary elsewhere. |
| CR-005 | Static analysis, compiler warnings, formatters, and linters should be clean or waived with owner and revisit trigger. | Whole implementation scope | tool output or documented inspection | Waivers require owner, reason, and revisit trigger. |
| CR-006 | Evidence-producing runs must record provenance fields before claims are treated as replayable. | BISECT run/build/analyze/report/verify paths | manifest inspection or smoke run | Historical outputs may be marked partial/gap rather than backfilled without evidence. |
| CR-007 | Deterministic search and replay paths must preserve seed, attempt, candidate, convergence, selection, and engine metadata. | Redistricting search and ensemble workflows | fixture/smoke run or manifest inspection | If nondeterministic behavior is intentional, claims must be limited accordingly. |
| CR-008 | Package canonicalization and hashing must use golden fixtures and versioned domain separation. | RPLAN, RCOUNT, RCTX, RHIST packages | golden package tests or audit fixtures | Version-unknown or algorithm-unknown status blocks compatibility/integrity claims. |
| CR-009 | Election-count normalization must preserve raw-source lineage, parser diagnostics, lifecycle status, and disagreement potential. | RCOUNT parsers, IO, reconciliation, district aggregation | count fixtures and role review | Aggregated totals alone are insufficient evidence. |
| CR-010 | Context/history joins must expose verified, partial, conjectural, blocked, or gap status. | RCTX/RHIST crosswalks and dependent aggregations | context fixtures or inspection | Unknown lineage cannot be silently treated as verified. |
| CR-011 | Public claims must be tied to evidence pointers, limitations, posture, status, uncertainty, and non-claims before publication. | README, docs, papers, dashboards, reports | claim-review checklist | Unsupported claims are marked gap, stale, conjectural, partial, or removed. |
| CR-012 | Source custody and generated-artifact disposition must be recorded before commit, publication, or release. | data caches, reports, dashboards, packages, PDFs, maps, intermediate artifacts | custody inspection and release review | Generated/public promotion requires VAULT disposition. |
| CR-013 | Wave/pulse completion must name requirements, package boundaries, verification, validation level, role gates, risks, and pitfall status. | context/waves, pulse files, PR close notes | trace/pulse inspection | Ad hoc completion is allowed only for explicitly non-VTRACE maintenance work. |

## Tailoring

| Area | Rule | Rationale |
|---|---|---|
| Rust production crates | Prefer existing Cargo tests, crate-level fixtures, and package-specific tests; do not require Python runtime for core BISECT production verification. | Preserves the accepted Rust production boundary. |
| Python support scripts | Treat Python scripts as support/research/acquisition/dashboard tooling unless promoted by interface control. | Prevents support scripts from becoming hidden production contracts. |
| Documentation and research | Review high-stakes claims with DATUM/SCALE/PRECINCT/COMMONS and link claims to evidence or gap status. | Keeps public claims aligned with evidence. |
| Generated artifacts | Do not treat generated outputs as source of truth without source, command, custody, and verification pointers. | Protects reproducibility and publication discipline. |
| Package families | Shared audit mechanics may be reused, but RPLAN/RCOUNT/RCTX/RHIST domain semantics remain separate. | Prevents generic package integrity from flattening civic/election context. |

## Exceptions / Waivers

| ID | Constraint | Exception | Rationale | Owner | Revisit Trigger |
|---|---|---|---|---|---|
| CR-WAIVER-001 | CR-006, CR-007 | Historical outputs that cannot be replayed with full provenance may be marked partial/gap instead of reconstructed. | Backfilling provenance without evidence would create false certainty. | Evidence owners / MERIDIAN / COVENANT | When historical artifact is cited by a public claim. |
| CR-WAIVER-002 | CR-005 | Documentation-only S3 planning artifacts do not require code build/test commands. | No executable behavior is changed by S3 planning artifacts. | Maintainers | Before code-changing S4 work begins. |

## Verification Evidence

| Evidence ID | Constraint IDs | Command / Review | Result | Evidence Pointer |
|---|---|---|---|---|
| EVID-CR-001 | CR-001..CR-005 | Existing test/build commands will be selected per work package, not globally in S3. | pending S4 | `WORK_PACKAGES.md` per-package commands. |
| EVID-CR-002 | CR-006, CR-007 | BISECT provenance/replay manifest inspection plus targeted provenance patch. | pass_with_risk | WP-003 review; L1/L2 replay remains blocked until local data or selected complete artifacts are available. |
| EVID-CR-003 | CR-008..CR-010 | Package-family schema/audit inspection, library tests, and package CLI tests. | pass_with_risk | WP-004 review; RPLAN has public fixtures, while RCOUNT/RCTX/RHIST fixture promotion remains deferred for public interoperability claims. |
| EVID-CR-004 | CR-011, CR-012 | Public claim review and repository custody review completed at L0. | pass_with_risk | WP-005 bounds README and paper-index public claims; WP-006 records custody defaults and fixes the embedded data manifest repository pointer. L1/L2 quantitative/public-release and release-bundle custody checks remain pending. |
| EVID-CR-005 | CR-013 | Wave/pulse close checklist review. | pass_with_risk | WP-007 updates `context/waves/PHASES.md` and records that future VTRACE-governed pulses must include IDs, boundaries, validation level, role gates, risk/pitfall disposition, and public/custody effects. First future governed pulse remains L1 evidence. |
| EVID-CR-006 | CR-003, CR-004, CR-008 | RPLAN and shapefile label-import implementation plus targeted CLI unit tests. | pass_with_risk | `crates/bisect-cli/src/import_label.rs` parses RPLAN through `rplan-io`, reads shapefile DBF attributes through the `shapefile` crate, normalizes assignments to BISECT one-based district ids, and keeps invalid input explicit. Public interoperability still needs named external fixtures. |
| EVID-CR-007 | CR-011, CR-013 | WP-008 non-author documentation repair and stale-command inspection. | pass_with_risk | `docs/BISECT_CLI.md` and `docs/quickstart/*.md` route users through current lowercase label-pipeline commands and describe evidence packages without legal/official certification claims. External-user walkthrough remains deferred. |
| EVID-CR-008 | CR-003, CR-006, CR-007, CR-008, CR-011, CR-012, CR-013 | Release-readiness DCR filing and trace linkage. | filed | `docs/vtrace/DCRS.md` files DCR-001 through DCR-007 for public fixtures, release smoke, user walkthrough, public evidence contract, compatibility matrix, legal packaging boundary, and full-scale reproducibility. These are follow-on gates, not completed release evidence. |
| EVID-CR-009 | CR-003, CR-004, CR-006, CR-008, CR-011, CR-012 | DCR execution artifacts, fixture-backed parser tests, and L1 package/boundary checklists. | partial_pass | `docs/fixtures/import-label/`, `docs/vtrace/IMPORT_COMPATIBILITY.md`, `docs/vtrace/EXTERNAL_WALKTHROUGH.md`, `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md`, and `docs/legal/COURT_PACKAGING_BOUNDARY.md` close DCR-001 at L2 for named label-import fixtures and close DCR-004, DCR-005, and DCR-006 at L1, while keeping DCR-003 partial. External-user, public-bundle, legal-filing, and full-scale reproducibility closures remain bounded where external user, concrete public bundle/legal review, or L2 replay evidence is unavailable. |
| EVID-CR-010 | CR-006, CR-007, CR-013 | VT real-state release smoke and label artifact contract fix. | pass_l1 | `docs/vtrace/RELEASE_SMOKE_BUNDLE.md` records passing `official_proposal/2020` VT build/analyze/report/verify commands and `label-verify` verdict `VERIFIED`; `crates/bisect-cli/src/build_cmd.rs` now promotes successful state runner artifacts into `runs/{label}/{year}/{state}/final_assignments.json`, with `build_cmd::tests::test_promote_label_state_artifacts_copies_runner_outputs_to_label_root` covering the contract. `docs/vtrace/REPRODUCIBILITY_RUN.md` records this as release-subset smoke, not L2 clean replay. |
| EVID-CR-011 | CR-006, CR-007, CR-012, CR-013 | DCR-007 release-subset replay capture harness and candidate replay. | candidate_data_dirty | `scripts/maintenance/dcr007_release_subset_replay.py` records environment/tool versions, source-clean policy, config/data hashes, algorithm/search parameters, resolved METIS engine, command outputs, and artifact hashes for a declared release-subset replay, with separate `candidate_command_allowed` and `clean_for_l2_replay` fields. A 2026-06-02 local candidate replay passed for `official_proposal/2020` VT and hashed 9 artifacts, but used `--allow-dirty-data`; it remains non-L2 until run from a clean data-backed checkout and reviewed as DCR-007 L2 evidence. |
| EVID-CR-012 | CR-009, CR-011, CR-012 | DCR-003 external-operator walkthrough packet and record helper. | ready_for_external_run | `docs/vtrace/EXTERNAL_WALKTHROUGH.md` now defines the L2 external-operator packet, declared-path fields, observer task script, observation-record template, friction taxonomy, and promotion rule; `scripts/maintenance/dcr003_walkthrough_record.py` generates a prefilled observation record for the selected quickstart and workflow. It is not L2 user evidence until a real non-author completes the walkthrough and COMMONS/operator-review dispositions friction. |
| EVID-CR-013 | CR-006, CR-007, CR-011, CR-012, CR-013 | DCR-007 clean replay packet and strict launcher. | ready_for_clean_run | `docs/vtrace/REPRODUCIBILITY_RUN.md` now defines the L2 clean replay packet, declared replay fields, operator tasks, reviewer checklist, custody requirements, and promotion rule; `scripts/maintenance/dcr007_clean_replay.py` refuses dirty checkouts and rejects `--allow-dirty-data` before invoking the replay harness. It is not L2 replay evidence until MERIDIAN/COVENANT review accepts a clean replay record and VAULT dispositions any public artifact promotion. |
| EVID-CR-014 | CR-011, CR-012, CR-013 | S5 integration control record. | complete_l1_control_for_internal_baseline | `docs/vtrace/INTEGRATION.md` integrates S4 work-package evidence, DCR posture, validation classes, claim boundaries, custody gates, and blocked S6 claims. It supports only the selected internal engineering baseline and is not public release-readiness evidence. |
| EVID-CR-015 | CR-011, CR-012, CR-013 | S6 readiness/transition decision. | internal_engineering_baseline_only | `docs/vtrace/READINESS_DECISION.md` approves only an internal engineering baseline and blocks public release, legal/court, external-user, clean reproducibility, and public evidence-package readiness claims until the named DCR and custody gates pass. |
| EVID-CR-016 | CR-011, CR-012, CR-013 | Internal baseline handoff controls. | internal_handoff_control | `docs/vtrace/BASELINE_HANDOFF.md` operationalizes the S6 internal baseline by naming allowed maintainer actions, stop gates, and a continuation checklist. It is not a release artifact and does not upgrade any DCR to L2. |
| EVID-CR-017 | CR-011, CR-012, CR-013 | VTRACE baseline index. | internal_navigation_control | `docs/vtrace/INDEX.md` provides the reading order, evidence/control artifact map, current stage summary, and blocked-claim list for maintainers. It is an entry point only and does not upgrade readiness posture. |
| EVID-CR-018 | CR-011, CR-012, CR-013 | Public documentation pointers to VTRACE baseline. | bounded_discoverability_control | `README.md` and `docs/BISECT_CLI.md` now point maintainers to `docs/vtrace/INDEX.md` and state the `internal_engineering_baseline_only` posture. The pointers improve discoverability without claiming public release, legal/court, external-user, or clean reproducibility readiness. |
| EVID-CR-019 | CR-011, CR-012, CR-013 | First live VTRACE-governed wave selection. | active_internal_maintenance_wave | `context/waves/2026-06-01-vtrace-baseline-maintenance/WAVE.md` and Pulse 01 select the first live VTRACE-governed wave after S6. This closes DREQ-003 selection only; it does not close release-readiness DCRs or upgrade S6 posture. |
| EVID-CR-020 | CR-011, CR-012, CR-013 | Release gate register. | release_gate_register_active | `docs/vtrace/RELEASE_GATE_REGISTER.md` and Pulse 02 route remaining release-grade work through named DCR evidence and review lanes. This is L1 control evidence only; it does not close DCR-003, DCR-004, DCR-006, DCR-007, public custody review, or S6 release readiness. |

## S3 Gate

Decision: accepted with risk after `.roles` review.

Before S3 lock:

- [x] Code-rigor IDs cover critical implementation and evidence risks from S2.
- [x] Each work package names applicable `CR-*` IDs or records why code rigor is not applicable.
- [x] Verification evidence can be executed, inspected, or explicitly deferred in S4.
- [x] Waivers are bounded, owned, and do not create success-shaped evidence.
