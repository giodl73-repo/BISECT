# Design Change Requests

## Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

These DCRs convert the residual release-readiness risks found during S4 closure into controlled follow-on work. They do not reopen WP-001 through WP-008; those work packages remain satisfied at their recorded validation level. A DCR closes only when its acceptance criteria, validation level, review lane, and custody/public-claim disposition are complete.

## DCR Index

| ID | Title | Status | Priority | Primary Parent IDs | Owner / Lane | Target Level |
|---|---|---|---|---|---|---|
| DCR-001 | Golden Interop Fixtures | closed_l2 | high | REQ-003, REQ-013, REQ-014, REQ-031, SPEC-002, SPEC-007, IF-003, IF-008, WP-002, WP-004 | LEDGER / package owners | L2 |
| DCR-002 | Release Smoke Bundle | partial_l1 | high | REQ-007, REQ-008, REQ-012, REQ-026, REQ-032, SPEC-005, SPEC-010, SPEC-012, WP-003, WP-008 | BENCHMARK / BISECT owners | L1 |
| DCR-003 | External User Walkthrough | partial_l1 | high | REQ-032, REQ-034, REQ-035, SPEC-012, IF-001, IF-006, WP-008 | COMMONS / operator review | L2 |
| DCR-004 | Public Evidence Package Contract | baseline_l1 | high | REQ-005, REQ-006, REQ-024, REQ-026, REQ-027, SPEC-004, SPEC-010, IF-006, WP-005, WP-006 | DATUM / SCALE / VAULT | L2 |
| DCR-005 | Import Compatibility Matrix | closed_l1 | medium | REQ-003, REQ-031, SPEC-002, SPEC-NF-007, IF-008, WP-002 | LEDGER / CLI owners | L1 |
| DCR-006 | Court/Legal Packaging Boundary | baseline_l1 | high | REQ-010, REQ-011, REQ-026, SPEC-006, SPEC-010, IF-006, WP-005, WP-006 | BOUNDARY / WARD / COMMONS | L2 |
| DCR-007 | Full-Scale Reproducibility Run | partial_smoke_only | high | REQ-007, REQ-008, REQ-009, REQ-012, SPEC-005, SPEC-NF-001, IF-001, IF-002, WP-003 | MERIDIAN / COVENANT | L2 |

## DCR-001: Golden Interop Fixtures

Status: closed_l2.

Problem: BISECT now supports CSV, GeoJSON, RPLAN, and shapefile label import paths, and package-family crates have meaningful tests, but public interoperability claims still depend on named external fixtures rather than general assertions.

Requested change: Add committed or otherwise public golden fixtures for each supported external plan interchange path that is intended for public compatibility claims: CSV, GeoJSON, RPLAN, and shapefile/DBF. Each fixture must include expected assignments, expected error cases where useful, source/version notes, and command evidence.

Acceptance criteria:

- Fixture set includes at least one positive and one negative case for each public import format claimed for release.
- RPLAN fixtures name schema version and zero-based-to-one-based assignment behavior.
- Shapefile fixtures document required DBF fields, sidecar requirements, and the fact that geometry transformation is out of scope for label import.
- Test or smoke command proves each fixture imports to expected BISECT assignments.
- Public docs cite only fixture-backed compatibility claims.
- DCR-005 matrix rows may cite public compatibility only for formats with matching DCR-001 fixture evidence; unsupported, unknown, and un-fixtured rows remain explicitly bounded.

Validation and review: L2 interoperability fixture review by LEDGER/package owners; custody review if fixture data is externally sourced.

Residual risk until closed: Public compatibility must remain bounded to the implemented adapter behavior and cannot imply universal support for all external tool exports.

Execution evidence: `docs/fixtures/import-label/` now includes positive and
negative public fixtures for CSV, GeoJSON, RPLAN, and shapefile/DBF, with
expected assignment JSON. `crates/bisect-cli/src/import_label.rs` includes
fixture-backed parser tests for every public fixture. The fixtures are tiny
synthetic records generated in-repo, so no external fixture custody review is
required.

## DCR-002: Release Smoke Bundle

Status: partial_l1.

Problem: S4 closure verified targeted code, docs, and package tests, but no single release-smoke command proves that a small end-to-end BISECT bundle can fetch or use prepared data, build, analyze, report, verify, and produce expected evidence artifacts.

Requested change: Define a release smoke bundle for a small representative state or fixture dataset that exercises the label pipeline from input through report/verification.

Acceptance criteria:

- Smoke command or script is documented and non-destructive.
- The bundle predeclares one canonical smoke scope: label/config, state or synthetic fixture, year, data provisioning rule, expected artifact paths, expected verification result, expected runtime class, and known failure modes.
- The path covers build, label-analyze, label-report, and label-verify or explicitly records why fetch/data is pre-provisioned.
- Evidence includes command output, manifest/report locations, and verification result state.
- Failure states use pass/fail/blocked/partial/deferred rather than success-shaped fallbacks.

Validation and review: L1 release-smoke review by BENCHMARK and BISECT owners; L2 only if used as a public release gate.

Residual risk until closed: Release health remains composed from targeted tests and inspections rather than one reproducible release smoke.

Execution evidence: `docs/vtrace/RELEASE_SMOKE_BUNDLE.md` declares the canonical
smoke scope, fixture smoke command, real-state command sequence, expected
artifacts, verification state, runtime class, and known failure modes. Real-state
execution remains blocked until the selected data/config environment is present;
the current checkout lacks `configs/*.yml` and `data/2020/`.

## DCR-003: External User Walkthrough

Status: partial_l1.

Problem: WP-008 repaired non-author workflow documentation at L0, but no non-author has yet followed the special-master, state-staff, researcher, or public-reviewer paths and recorded friction.

Requested change: Run and record an external-user walkthrough against the current quickstarts and CLI reference.

Acceptance criteria:

- L1 internal usability review may use role simulation, but L2 public-readiness closure requires a real non-author operator or explicitly documented external reviewer.
- At least one non-author path is followed end to end or to a documented blocker.
- Friction is recorded as doc fixes, command fixes, or accepted limitations.
- Output expectations, evidence locations, and failure modes are understandable without maintainer context.
- Any public/legal/certification misunderstanding is corrected before readiness claims.

Validation and review: L2 user-workflow review by COMMONS and operator-review lane.

Residual risk until closed: Docs are internally aligned but not externally proven as usable.

Execution evidence: `docs/vtrace/EXTERNAL_WALKTHROUGH.md` records the L1
role-simulation path, expected user understanding, and L2 blocker. L2 closure
requires a real non-author operator or external reviewer.

## DCR-004: Public Evidence Package Contract

Status: baseline_l1.

Problem: Public evidence outputs are discussed across reports, dashboards, papers, manifests, and VTRACE docs, but the stable public artifact layout and required fields for downstream consumers are not yet locked as a release contract.

Requested change: Define the public evidence package contract for release bundles, including required directories, manifests, report formats, hashes, limitations, non-claims, custody disposition, and review status.

Acceptance criteria:

- Contract is published or referenced as a versioned artifact that names required fields, optional fields, compatibility rules, and change-control triggers.
- Contract names required and optional artifacts for a public BISECT evidence package.
- Required manifests include source pointers, command/config pointers, hash fields, verification status, and limitations.
- Contract distinguishes local/generated artifacts from publishable evidence artifacts.
- Claim posture, uncertainty, non-claims, and legal/certification boundary language are required in public-facing outputs.
- Downstream-breaking changes to the contract require DCR or interface-change disposition.
- Public release bundles define immutability or replacement-notice rules, including hash manifest retention and supersession rules.

Validation and review: L2 public artifact review by DATUM, SCALE, COMMONS, and VAULT.

Residual risk until closed: Public artifacts remain reviewable by current docs but not yet stable as a downstream contract.

Execution evidence: `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md` defines
`BISECT-EVIDENCE-PACKAGE-v1`, required layout, required manifest fields,
optional artifacts, compatibility rules, immutability, supersession, and
non-claims. L2 closure requires review against a concrete public bundle.

## DCR-005: Import Compatibility Matrix

Status: closed_l1.

Problem: The current adapter inventory distinguishes implemented and bounded import/export paths, but users still need a concise compatibility matrix that names exact supported formats, field assumptions, version assumptions, unsupported variants, and fixture status.

Requested change: Publish an import/export compatibility matrix for BISECT label-plan and package adapter surfaces.

Acceptance criteria:

- Matrix includes CSV, GeoJSON, RPLAN, shapefile, GerryChain, DRA-style CSV, PlanScore, Census/TIGER, and NIST/CDF where named in docs.
- Each row states supported direction, version/schema status, required fields, geometry assumptions, failure modes, and fixture evidence.
- Unimplemented, partial, stubbed, or version-unknown paths are plainly marked.
- Matrix may close at L1 with unsupported, unknown, and un-fixtured rows plainly marked, but public compatibility claims require corresponding DCR-001 fixture evidence.
- CLI docs and interface docs do not make broader claims than the matrix.

Validation and review: L1 interface review by LEDGER and CLI owners; L2 if bundled as public interoperability evidence.

Residual risk until closed: Adapter support is discoverable in VTRACE and CLI docs but not centralized for external users.

Execution evidence: `docs/vtrace/IMPORT_COMPATIBILITY.md` centralizes CSV,
GeoJSON, RPLAN, shapefile, GerryChain, DRA-style CSV, PlanScore, Census/TIGER,
and NIST/CDF status, fields, assumptions, failure modes, and fixture evidence.
L2 public interoperability still depends on DCR-001 fixture promotion where
claimed.

## DCR-006: Court/Legal Packaging Boundary

Status: baseline_l1.

Problem: BISECT produces algorithmic evidence packages and legal-analysis materials, but a court-ready or filing-ready package requires separate jurisdictional gates and human/legal review beyond generated reports.

Requested change: Define the boundary between a generated evidence package and a court/legal filing package, including explicit gates for legal readiness, jurisdiction-specific constraints, expert review, and non-claims.

Acceptance criteria:

- Docs distinguish generated evidence package, legal review package, and court-ready filing package.
- Required legal gates remain separate for federal law, state law, chamber rules, VRA, equal population, contiguity, subdivisions, nesting, and jurisdiction-specific criteria.
- Release or quickstart language cannot imply court readiness from `label-report` or `label-verify` alone.
- Handoff checklist identifies what BISECT supplies and what remains with counsel, experts, courts, commissions, or officials.
- Court-ready or filing-ready status requires jurisdiction-specific human/legal authority review outside the software; BISECT supplies evidence and checklists, not legal authority.
- Review lane includes BOUNDARY/WARD and public-claim review.

Validation and review: L2 legal-boundary review by BOUNDARY, WARD, COMMONS, and maintainers.

Residual risk until closed: Current docs are bounded against certification claims but do not provide a full legal-filing packaging contract.

Execution evidence: `docs/legal/COURT_PACKAGING_BOUNDARY.md` distinguishes
generated evidence packages, legal review packages, and court-ready filing
packages; names required legal gates; and states that BISECT supplies evidence
and checklists, not legal authority. L2 closure requires public-claim/legal
review before any filing-ready language is used.

## DCR-007: Full-Scale Reproducibility Run

Status: partial_smoke_only.

Problem: S4 improved provenance and targeted verification, but release-level reproducibility still requires a selected full-scale or release-subset run with source data, environment, build features, search metadata, reports, and verification chain captured together.

Requested change: Run and record a full-scale reproducibility scenario across all states/years or a declared release subset.

Acceptance criteria:

- Scope is declared before execution: all states/years or a specific release subset.
- Build features, binary hash, METIS engine, source-data hashes/custody, environment assumptions, config hash, command line, seed/search metadata, and output paths are recorded.
- Label build, analysis, report, and verification artifacts are generated or explicitly blocked with reasons.
- Reproducibility evidence compares a clean checkout/environment replay against expected hashes, or records exact divergences and dispositions.
- Any failed or partial state/year is recorded with failure mode and retry/disposition.
- Public claims cite this run only within its declared scope.
- Evidence and downstream references use the declared reproducibility class: full-scale, release-subset, or smoke-only.

Validation and review: L2 reproducibility review by MERIDIAN and COVENANT; VAULT review if artifacts are staged for publication.

Residual risk until closed: The project should not claim full release reproducibility beyond targeted tests and documented L0/L1 evidence.

Execution evidence: `docs/vtrace/REPRODUCIBILITY_RUN.md` declares the current
reproducibility class as `smoke-only`, records fixture replay evidence, and lists
the required release-subset/full-scale fields. Full-scale and release-subset
closure remain open until a clean data-backed replay is executed and reviewed;
the current checkout lacks a release config and source-data cache.

## Change Control

Changing a DCR title, scope, acceptance criteria, validation level, or parent-ID mapping requires updating `TRACE.md`, `STAGE_EXECUTION.md`, and any affected interface, package-boundary, requirement, or specification rows in the same change.
