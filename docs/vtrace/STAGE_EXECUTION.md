# Stage Execution

## Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

This board records VTRACE adoption progress for the existing repo. It is a control artifact, not a substitute for code, tests, wave/pulse records, or package-specific evidence.

## Stage Board

| Stage | Status | Gate Decision | Required Next Action |
|---|---|---|---|
| S0 Intake | complete | accepted | Preserve repo scope and source-path assumptions in later trace artifacts. |
| S1 Specification Baseline | complete | accepted | Keep Mission, CONOPS, Requirements, and Specification IDs stable. |
| S2 Design Baseline | complete | accepted | Use accepted Architecture, Package Boundaries, Interfaces, and Design IDs in work planning. |
| S3 Implementation Planning | complete | accepted with risk | Use accepted S3 work packages to govern S4 execution. |
| S4 Work Package Execution | complete_l0_pass_with_risk | WP-001..WP-008 satisfied; DCRs partially executed | Preserve closed work-package evidence and do not convert L1 DCR smoke/checklist evidence into public release/readiness claims without the named L2 evidence. |
| S5 Integration | complete_l1_control_for_internal_baseline | integrated control record accepted for internal baseline; blocked for L2 release scope | Preserve S5 controls and attach required external-user, clean replay, public-claim, and custody evidence before any stronger S6 transition. |
| S6 Readiness / Transition | internal_engineering_baseline_only | internal baseline allowed; public/release/legal/external-user/clean-replay readiness blocked | Continue from `READINESS_DECISION.md`; do not upgrade readiness posture until the named DCR gates pass. |

## Stage Evidence

| Stage | Required Artifacts | Validation Level | Role Lanes | Evidence Pointer |
|---|---|---|---|---|
| S0 | Repo scope and VTRACE source confirmation | none / L0 | systems engineering | Conversation and accepted repo path: `C:\src\apportionment`; VTRACE source: `C:\src\tracker\repos\standards-protocols\vtrace`. |
| S1 | `MISSION.md`, `CONOPS.md`, `REQUIREMENTS.md`, `SPECIFICATION_BASELINE.md` | L0 | systems engineering / traceability / domain roles | Accepted artifacts under `docs/vtrace/`. |
| S2 | `ARCHITECTURE.md`, `PACKAGE_BOUNDARIES.md`, `INTERFACES.md`, `DESIGN.md` | L0 / L1 planning | BOUNDARY, WARD, CANVASS, VAULT, TALLY, CONTOUR, MERIDIAN, DATUM, SCALE, BENCHMARK, COMMONS, LEDGER, SURVEY, TRENCH, PRECINCT, COVENANT | Accepted artifacts under `docs/vtrace/`. |
| S3 | `CODE_RIGOR.md`, `IMPLEMENTATION_PLAN.md`, `WORK_PACKAGES.md`, this board, `REVIEW.md` | L0 | BENCHMARK, TRENCH, LEDGER, BOUNDARY, VAULT, DATUM, SCALE, COMMONS | Accepted S3 planning artifacts under `docs/vtrace/`; remaining risks assigned to S4 work packages. |
| S4 | Work-package evidence, changed files, verification commands, review findings, filed DCRs | L0 / L1 | Package-specific lanes per work package | WP-001 evidence: `TRACE.md`, `WORK_PACKAGES.md`, `IMPLEMENTATION_PLAN.md`; WP-002 evidence now includes `repo-map.toml`, `tools/repo_map.py`, `docs/REPO_MAP_STANDARD.md`, generated Cargo patches, Cargo metadata pass, CLI help pass, dual CLI-surface inventory, package-family schema version/draft inventory, implemented/stubbed adapter inventory, and Rust-gap follow-ups implementing RPLAN and direct shapefile label import; WP-003 evidence includes label build-index provenance fields, per-state executable hashing, label verification SHA-chain inspection, label build artifact promotion into the analysis contract path, historical replay-gap disposition, and a DCR-007 release-subset replay-capture harness; WP-004 evidence includes RPLAN/RCOUNT/RHIST model inspection, package-family library tests, package CLI tests, RPLAN public audit fixtures, and explicit external-fixture risk for RCOUNT/RHIST; WP-005 evidence includes README and paper-index claim-boundary edits, public-claim pattern inspection, and L1/L2 quantitative/public-release risk disposition; WP-006 evidence includes `.gitignore` inspection, tracked/local artifact inventory, data/output policy review, embedded manifest repo correction, archive boundary review, and release-custody risk disposition; WP-007 evidence includes wave index and pulse sample inspection plus `context/waves/PHASES.md` closure-rule updates; WP-008 evidence includes CLI reference and quickstart repair for current label-pipeline workflows plus a DCR-003 external-operator packet; release-readiness follow-ups are filed as DCR-001 through DCR-007 in `docs/vtrace/DCRS.md`; DCR execution artifacts now include public import fixtures, import matrix, VT release smoke, external-walkthrough record and operator packet, evidence-package contract/checklist, court-packaging boundary/checklist, release-subset smoke reproducibility record, and reusable replay preflight/capture tooling. |
| S5 | Integration trace, cross-package evidence, validation results, claim/custody review | L1 control / L2 where applicable | Integration and public evidence lanes | `docs/vtrace/INTEGRATION.md` integrates S4 work-package evidence and DCR posture into S5 gates, with S6 selected only for internal engineering baseline transition. |
| S6 | Readiness decision, transition notes, release/publication disposition | L1 internal baseline / L2 where applicable | Maintainers and external-user readiness lanes | `docs/vtrace/INDEX.md` is the VTRACE entry point; `docs/vtrace/READINESS_DECISION.md` approves only an internal engineering baseline and blocks public release, legal/court, non-author, and clean reproducibility readiness claims; `docs/vtrace/BASELINE_HANDOFF.md` operationalizes the allowed maintainer actions and stop gates. |

## Open Stage Findings

| Finding | Stage | Owner | Disposition |
|---|---|---|---|
| CLI/help compatibility is not yet verified against the current binary. | S3 / S4 | BISECT owners / LEDGER | WP-002 L0 inspected current help and records both legacy and label-pipeline command surfaces; command-level compatibility still requires command-specific evidence before public claims. |
| Cargo/script dependency directions are accepted as intended boundaries but not fully verified against current code. | S3 / S4 | BENCHMARK / TRENCH | WP-002 verifies the repo-map Cargo boundary and records Python as support/research; WP-004 verifies package-family tests; public script/output claims remain WP-005. |
| Historical generated outputs may lack complete executable and METIS replay metadata. | S3 / S4 | MERIDIAN / COVENANT | WP-003 adds missing metadata for new outputs where safe and records historical artifacts as requiring regeneration or supplemental evidence before release-level replay claims. |
| Public research claims and paper evidence completeness are not yet inventoried claim-by-claim. | S3 / S4 | DATUM / SCALE / PRECINCT / COMMONS | WP-005 completed L0 headline/public-surface inventory and bounded README/paper-index claims; full paper-by-paper figure/table recomputation remains deferred to release review. |
| Exact RPLAN/RCOUNT/RCTX/RHIST package schemas and canonicalization algorithms are controlled by package specs and need fixture-level evidence. | S3 / S4 | Package owners / LEDGER / CANVASS / CONTOUR | WP-004 verifies implemented package-family tests and records external golden fixture/public interoperability as an L2 deferred risk. |
| RCOUNT/RCTX/RHIST have mostly inline synthetic tests rather than promoted public golden fixtures. | S4 / WP-004 | Package owners / LEDGER / CANVASS / CONTOUR | Accept for L1 crate evidence; require fixture promotion or selected downstream package scenario before public interoperability/release claims. |
| Local cross-repo paths were reorganized under `C:\src\tracker\repos`; old sibling paths can break Cargo before CLI verification. | S4 / WP-002 | BISECT owners / TRENCH | `repo-map.toml` records path topology; `tools/repo_map.py` generates Cargo patches; keep future local path dependencies generated from the map rather than hard-coded in manifests. |
| Provenance metadata can imply replay completeness beyond current build/environment evidence. | S4 / WP-003 | MERIDIAN / COVENANT | WP-003 adds build-index/executable hashes and marks release-level replay as pass-with-risk pending environment, build-feature, candidate-list, and data-backed replay evidence. |
| Public headline metrics can imply release-final or externally certified status. | S4 / WP-005 | DATUM / SCALE / PRECINCT / COMMONS | README now identifies headline metrics as empirical research/dashboard claims, and `docs/PAPERS.md` clarifies that review/accepted labels are internal project status markers. |
| Generated/local artifact presence can imply publishability. | S4 / WP-006 | VAULT / COVENANT | WP-006 records generated directories as ignored/local-only by default, docs/research artifacts as public only after evidence/custody review, and archives as read-only forensic references unless explicitly promoted. |
| Embedded data manifest pointed to the pre-rename repository. | S4 / WP-006 | VAULT / COVENANT | `data/manifest.json` now points to `giodl73-repo/BISECT`, matching the active git remote. Release asset names still require release-specific validation. |
| Future pulses could close as implemented without VTRACE evidence. | S4 / WP-007 | BENCHMARK / TRENCH | `context/waves/PHASES.md` now requires VTRACE-governed pulses to name IDs, boundaries, validation level, role gates, risk/pitfall disposition, and public/custody effects. Archived pulses are not rewritten. |
| First live VTRACE-governed wave selection was required after S6. | S4 / WP-007 / DREQ-003 | BENCHMARK / TRENCH | Selected `context/waves/2026-06-01-vtrace-baseline-maintenance` as the active internal-maintenance wave; Pulse 01 is the L1 control slice for exercising VTRACE-governed pulse rules without upgrading release posture. |
| `.rplan` and direct shapefile label import were recorded as stub adapter surfaces. | S4 / WP-002 follow-up | BISECT CLI owners / LEDGER | Fixed for RPLAN and direct shapefile: `label-import` parses RPLAN through `rplan-io`, reads shapefile DBF attributes through the `shapefile` crate, and writes one-based BISECT assignment files. Broader public compatibility claims still require named fixtures. |
| Non-author quickstarts contained stale uppercase/legacy command paths. | S4 / WP-008 | COMMONS / LEDGER | Fixed at L0: quickstarts and CLI reference now direct users through current lowercase label-pipeline commands and evidence-package caveats. External-user dry run remains a release-readiness activity. |
| Release-readiness residuals need controlled follow-up records. | S4 / S5 / S6 | Maintainers / role lanes | Filed DCR-001 through DCR-007 and executed artifacts for public import fixtures, release smoke scope, external-user walkthrough simulation, L2 operator packet and record helper, evidence package contract/checklist, import matrix, court/legal packaging boundary/checklist, release-subset smoke reproducibility, a DCR-007 replay-capture harness, a data-dirty VT candidate replay, a DCR-007 clean replay packet, and a strict clean-run launcher. DCR-001 is closed at L2; DCR-002, DCR-004, DCR-005, and DCR-006 are closed at L1; DCR-003 and DCR-007 still block stronger external-user and L2 reproducibility claims as recorded in `DCRS.md`. |
| Remaining release-grade gates need a single routing surface. | S6 | Maintainers / role lanes | `RELEASE_GATE_REGISTER.md` now routes external-user, clean reproducibility, public evidence bundle, legal/court, and expanded interoperability gates to their required DCR evidence and review lanes without upgrading S6 posture. |

## S3 Gate

Decision: accepted with risk after `.roles` review.

Before S3 lock:

- [x] Work packages cover every accepted `REQ-*`, `SPEC-*`, `IF-*`, `PKG-*`, `DES-*`, and applicable `CR-*` item or record a disposition.
- [x] Work packages include entry criteria, exit criteria, verification commands or inspections, validation level, risk, and role lanes.
- [x] Code rigor constraints are tailored to Rust, Python, docs/research, package schemas, and generated/public artifacts.
- [x] Implementation sequencing avoids public-claim, compatibility, or certification claims before evidence exists.
- [x] S3 role review findings are fixed, explicitly deferred, or accepted as risk.
