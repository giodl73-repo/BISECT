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
| EVID-CR-009 | CR-003, CR-004, CR-006, CR-008, CR-011, CR-012 | DCR execution artifacts and fixture-backed parser tests. | partial_pass | `docs/fixtures/import-label/`, `docs/vtrace/IMPORT_COMPATIBILITY.md`, `docs/vtrace/RELEASE_SMOKE_BUNDLE.md`, `docs/vtrace/EXTERNAL_WALKTHROUGH.md`, `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md`, `docs/legal/COURT_PACKAGING_BOUNDARY.md`, and `docs/vtrace/REPRODUCIBILITY_RUN.md` close DCR-001 at L2 for named label-import fixtures and DCR-005 at L1, while creating L1/baseline records for DCR-002, DCR-003, DCR-004, DCR-006, and DCR-007. Full release-health, external-user, legal-filing, and full-scale reproducibility closures remain bounded where external user, concrete public bundle, or data-backed replay evidence is unavailable. |

## S3 Gate

Decision: accepted with risk after `.roles` review.

Before S3 lock:

- [x] Code-rigor IDs cover critical implementation and evidence risks from S2.
- [x] Each work package names applicable `CR-*` IDs or records why code rigor is not applicable.
- [x] Verification evidence can be executed, inspected, or explicitly deferred in S4.
- [x] Waivers are bounded, owned, and do not create success-shaped evidence.
