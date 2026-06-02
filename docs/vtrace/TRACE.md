# BISECT VTRACE Trace Control Spine

**Status**: S6 internal engineering baseline only, release readiness blocked
**Stage**: S6 Readiness / Transition
**Last updated**: 2026-06-01

This file is the trace-control spine for the BISECT VTRACE adaptation. It maps accepted mission, CONOPS, requirements, specification, S2 design, S3 code-rigor, S4 work-package, and filed DCR IDs to verification/validation obligations or explicit dispositions.

TRACE is control-plane evidence. It proves that the accepted baseline is mapped and reviewable; it does not prove that implementation behavior already satisfies every mapped requirement. S4 work packages must close each row by validating, fixing, recording a gap, or accepting a bounded risk.

## Trace Rules

| Rule | Meaning |
|---|---|
| Accepted IDs must be visible | Every accepted ID from S0-S3 must appear in this file or in a supersede/retire/split/merge disposition. |
| Unknown is not pass | Version-unknown, fixture-unknown, replay-unknown, and evidence-unknown states are findings or risks, not compatibility claims. |
| Specs are local source of truth | Accepted `docs\vtrace\*` specs govern implementation until a controlled baseline change updates them. |
| Mission governs specs | Mission and CONOPS can force a spec change, but only through a recorded baseline update. |
| Work packages close rows | S4 rows close as validated, fixed, gap, risk, or not_applicable. |

## Accepted Baseline Inventory

| Source | IDs |
|---|---|
| Mission | M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09 |
| CONOPS | CO-01, CO-02, CO-03, CO-04, CO-05, CO-06, CO-07, CO-08, CO-09, CO-10, CO-11, CO-12 |
| Requirements | REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023, REQ-024, REQ-025, REQ-026, REQ-027, REQ-028, REQ-029, REQ-030, REQ-031, REQ-032, REQ-033, REQ-034, REQ-035, REQ-036, REQ-037 |
| Specification | SPEC-001, SPEC-002, SPEC-003, SPEC-004, SPEC-005, SPEC-006, SPEC-007, SPEC-008, SPEC-009, SPEC-010, SPEC-011, SPEC-012 |
| Nonfunctional specification | SPEC-NF-001, SPEC-NF-002, SPEC-NF-003, SPEC-NF-004, SPEC-NF-005, SPEC-NF-006, SPEC-NF-007 |
| Architecture | ARCH-001, ARCH-002, ARCH-003, ARCH-004, ARCH-005 |
| Package boundaries | PKG-001, PKG-002, PKG-003, PKG-004, PKG-005, PKG-006, PKG-007, PKG-008, PKG-009, PKG-010, PKG-011, PKG-012, PKG-013, PKG-014 |
| Interfaces | IF-001, IF-002, IF-003, IF-004, IF-005, IF-006, IF-007, IF-008 |
| Design | DES-001, DES-002, DES-003, DES-004, DES-005, DES-006, DES-007, DES-008, DES-009, DES-010, DES-011, DES-012, DES-013 |
| Code rigor | CR-001, CR-002, CR-003, CR-004, CR-005, CR-006, CR-007, CR-008, CR-009, CR-010, CR-011, CR-012, CR-013 |
| Filed DCRs | DCR-001, DCR-002, DCR-003, DCR-004, DCR-005, DCR-006, DCR-007 |

## Work Package Trace

| WP | Primary purpose | Baseline IDs | Verification / validation closure |
|---|---|---|---|
| WP-001 | Trace control spine and orphan checks. | M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, CO-01, CO-02, CO-10, CO-11, CO-12, REQ-001, REQ-002, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036, SPEC-001, SPEC-011, SPEC-NF-003, SPEC-NF-006, ARCH-005, IF-007, DES-001, DES-002, DES-011, DES-013, PKG-001, PKG-013, CR-013 | L0 trace inspection. Closure status: complete for control-plane mapping; implementation compliance and release-readiness DCR closure remain with downstream WPs/DCRs. |
| WP-002 | Interface and boundary verification. | CO-03, REQ-003, REQ-031, REQ-037, SPEC-002, SPEC-NF-004, SPEC-NF-007, IF-001, IF-002, IF-003, IF-004, IF-005, IF-006, IF-007, IF-008, ARCH-001, ARCH-002, ARCH-004, DES-006, PKG-003, PKG-004, PKG-005, PKG-006, PKG-007, PKG-008, PKG-009, PKG-010, PKG-011, PKG-012, PKG-013, PKG-014, CR-001, CR-002, CR-003, CR-004, CR-005 | L0 inspection, L1 targeted tests if behavior changes, L2 interoperability fixture if public adapter is claimed. |
| WP-003 | Redistricting run evidence and replay integrity. | CO-04, CO-08, CO-10, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, SPEC-005, SPEC-006, SPEC-NF-001, IF-001, IF-002, DES-004, DES-005, DES-010, PKG-007, PKG-008, PKG-009, PKG-010, CR-001, CR-002, CR-004, CR-006, CR-007 | L0 manifest/config inspection, L1 single-state replay where data is available, L2 representative full-run replay when release evidence is required. |
| WP-004 | RPLAN/RCOUNT/RCTX/RHIST package family audit. | CO-05, CO-06, CO-07, REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023, SPEC-007, SPEC-008, SPEC-009, SPEC-NF-001, IF-003, IF-004, IF-005, IF-008, DES-003, DES-004, DES-007, DES-008, DES-010, PKG-004, PKG-005, PKG-006, CR-001, CR-002, CR-004, CR-008, CR-009, CR-010 | L0 schema/fixture inspection, L1 package tests where crates exist and are buildable, L2 integrated package audit for downstream/public use. |
| WP-005 | Public claim and research evidence review. | CO-08, CO-09, REQ-005, REQ-006, REQ-024, REQ-025, REQ-026, SPEC-004, SPEC-010, SPEC-NF-005, IF-006, ARCH-003, DES-004, DES-009, PKG-012, CR-011 | L0 claim inventory, L1 table/figure/source checks for quantitative claims, L2 public-release or hostile-review scenario. |
| WP-006 | Custody, privacy, and generated artifact disposition. | CO-05, CO-06, CO-11, REQ-004, REQ-020, REQ-027, REQ-033, SPEC-003, SPEC-010, SPEC-NF-002, IF-002, IF-006, IF-008, DES-012, PKG-002, PKG-011, PKG-014, CR-012 | L0 file/documentation inspection, L1 artifact disposition validation, L2 release-archive audit if artifacts are published. |
| WP-007 | Wave/pulse integration and pitfall updates. | CO-10, CO-11, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036, SPEC-011, IF-007, DES-001, DES-002, DES-011, PKG-013, CR-013 | L0 wave/pulse doc inspection complete at pass_with_risk; L1 process dry-run remains deferred until the first future VTRACE-governed pulse, and L2 full pulse closure scenario remains release-critical only. |
| WP-008 | Non-author operator guide and quickstart validation. | CO-01, CO-02, CO-03, CO-12, REQ-032, SPEC-012, IF-001, IF-006, DES-011, PKG-010, PKG-012, PKG-013, CR-011 | L0 guide inspection, L1 operator dry-run where commands are local and non-destructive, L2 external-user validation if release-critical. |

## Vertical Mission Trace

| Mission/CONOPS need | Requirements/specification/design response | S4 closure |
|---|---|---|
| M-01, M-02, CO-01, CO-02 | REQ-001, REQ-002, SPEC-001, SPEC-011, SPEC-012, DES-001, DES-002, DES-011, ARCH-005 | WP-001, WP-007, WP-008 |
| M-03, CO-03, CO-04 | REQ-003, REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012, SPEC-002, SPEC-005, SPEC-006, IF-001, IF-002, ARCH-001, DES-005, DES-006, DES-010 | WP-002, WP-003 |
| M-04, CO-05, CO-06, CO-07 | REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023, SPEC-007, SPEC-008, SPEC-009, IF-003, IF-004, IF-005, IF-008, ARCH-002, ARCH-004, DES-003, DES-007, DES-008 | WP-004 |
| M-05, M-06, CO-08, CO-09 | REQ-005, REQ-006, REQ-024, REQ-025, REQ-026, SPEC-004, SPEC-010, IF-006, ARCH-003, DES-004, DES-009 | WP-005 |
| M-07, CO-10, CO-11 | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036, SPEC-NF-003, SPEC-NF-006, IF-007, DES-013, PKG-001, PKG-013, CR-013 | WP-001, WP-007, DCR-001..DCR-007 |
| M-08, CO-05, CO-11 | REQ-004, REQ-020, REQ-027, REQ-033, SPEC-003, SPEC-NF-002, PKG-002, PKG-011, PKG-014, DES-012, CR-012 | WP-006 |
| M-09, CO-12 | REQ-032, SPEC-012, IF-001, IF-006, PKG-010, PKG-012, PKG-013, DES-011, CR-011 | WP-008 |

## DCR Trace

| DCR | Release-readiness gap | Parent IDs | Required closure |
|---|---|---|---|
| DCR-001 | Public golden interop fixtures for claimed import/package compatibility. | REQ-003, REQ-013, REQ-014, REQ-031, SPEC-002, SPEC-007, IF-003, IF-008, DES-013, WP-002, WP-004 | Closed L2 for CSV, GeoJSON, RPLAN, and shapefile/DBF assignment-table fixtures under `docs/fixtures/import-label/`; broader tool-specific round-trip claims still require separate fixtures. |
| DCR-002 | One documented release smoke bundle for build/analyze/report/verify health. | REQ-007, REQ-008, REQ-012, REQ-026, REQ-032, SPEC-005, SPEC-010, SPEC-012, DES-013, WP-003, WP-008 | Closed L1 via `RELEASE_SMOKE_BUNDLE.md` after the `official_proposal/2020` VT build/analyze/report/verify smoke passed and `label-verify` returned `VERIFIED`; L2 public release health remains out of scope. |
| DCR-003 | External-user walkthrough for non-author workflows. | REQ-032, REQ-034, REQ-035, SPEC-012, IF-001, IF-006, DES-013, WP-008 | Partial L1 via role simulation plus an L2 external-operator packet and prefilled record helper; L2 user-workflow review remains required before public-readiness claims. |
| DCR-004 | Stable public evidence package contract. | REQ-005, REQ-006, REQ-024, REQ-026, REQ-027, SPEC-004, SPEC-010, IF-006, DES-013, WP-005, WP-006 | Closed L1 via `EVIDENCE_PACKAGE_CONTRACT.md` and its internal package checklist; L2 public artifact and custody review remain required before downstream contract claims. |
| DCR-005 | Central import compatibility matrix. | REQ-003, REQ-031, SPEC-002, SPEC-NF-007, IF-008, DES-013, WP-002 | Closed L1 via `IMPORT_COMPATIBILITY.md`; L2 still required if used as public interoperability evidence. |
| DCR-006 | Court/legal packaging boundary. | REQ-010, REQ-011, REQ-026, SPEC-006, SPEC-010, IF-006, DES-013, WP-005, WP-006 | Closed L1 via `COURT_PACKAGING_BOUNDARY.md` and its internal boundary checklist; L2 legal-boundary review remains required before court-ready or filing-ready language. |
| DCR-007 | Full-scale or declared-subset reproducibility run. | REQ-007, REQ-008, REQ-009, REQ-012, SPEC-005, SPEC-NF-001, IF-001, IF-002, DES-013, WP-003 | Partial L1 release-subset candidate evidence plus an L2 clean replay packet via `REPRODUCIBILITY_RUN.md`; `scripts/maintenance/dcr007_release_subset_replay.py` captured a data-dirty VT candidate replay, and `scripts/maintenance/dcr007_clean_replay.py` now enforces the clean-run operator gate, but L2 clean replay review remains required before full-release or clean release-subset reproducibility claims. |

## Current Orphan And Gap Status

| Check | Result | Disposition |
|---|---|---|
| Accepted ID appears in trace inventory | pass | All accepted S0-S3 ID families are represented. |
| Accepted ID maps to at least one S4 work package | pass | Mapping is explicit in Work Package Trace. |
| Architecture IDs mapped | fixed in WP-001 | ARCH-001, ARCH-002, ARCH-003, ARCH-004, ARCH-005 are now mapped. |
| CR-003 mapped | fixed in WP-001 | CR-003 maps to WP-002 because version-unknown compatibility and explicit error posture are interface/boundary verification obligations. |
| Implementation compliance | not claimed | Downstream WPs must validate, fix, gap, or risk-record implementation behavior. |
| Release-readiness DCR coverage | partially executed | DCR-001 is closed at L2; DCR-002, DCR-004, DCR-005, and DCR-006 are closed at L1; DCR-003 and DCR-007 retain bounded partial evidence without external-user or L2 reproducibility closure claims; DCR-003 now has an external-operator packet and prefilled record helper, while DCR-007 has a reusable replay-capture harness, data-dirty VT candidate replay, clean replay packet, and strict clean-run launcher for future closure evidence. |

## S5 Integration Trace

| Integration control | Source evidence | Current disposition |
|---|---|---|
| Work-package closure integrated | WP-001..WP-008 in `WORK_PACKAGES.md`; S4 evidence in `STAGE_EXECUTION.md` | S4 closure evidence is integrated for control-plane use; release-level claims remain bounded by DCR status. |
| DCR posture integrated | DCR-001..DCR-007 in `DCRS.md`; `INTEGRATION.md` DCR posture matrix | DCR-001 is L2 fixture-closed; DCR-002, DCR-004, DCR-005, and DCR-006 are L1-closed; DCR-003 and DCR-007 remain open for L2. |
| Claim and custody gates integrated | `EVIDENCE_PACKAGE_CONTRACT.md`; `COURT_PACKAGING_BOUNDARY.md`; `REPRODUCIBILITY_RUN.md` | S5 may support internal engineering baseline claims, but public release, legal/court, clean replay, and non-author readiness claims remain blocked until selected evidence exists. |

## S6 Readiness Trace

| Transition control | Source evidence | Current disposition |
|---|---|---|
| Internal engineering baseline | `READINESS_DECISION.md`; `INTEGRATION.md`; S4/S5 ledgers | Approved as `internal_engineering_baseline_only` with L1 residual risks recorded. |
| Public release readiness | DCR-003, DCR-004, DCR-006, DCR-007; VAULT/public-claim review | Blocked until external-user, public-bundle, legal-boundary, clean replay, and custody gates selected for the release scope are satisfied. |
| Claim posture | `READINESS_DECISION.md` allowed and blocked S6 statements | Public, legal, court-ready, non-author validated, and clean reproducibility claims remain explicitly blocked. |

## Change Control

Future baseline changes must update this file in the same change as the source document. If an ID is superseded, retired, split, or merged, the disposition must name the old ID, new ID if any, rationale, affected WPs, and validation impact.
