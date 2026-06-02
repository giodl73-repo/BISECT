# S5 Integration Control Record

## Scope

This record starts VTRACE S5 integration for BISECT. It integrates the S4
work-package closures, DCR execution artifacts, validation evidence, public-claim
posture, and custody posture into one release-scope control point.

Current decision: `in_progress_l1_control`.

This is not a release-readiness decision. S6 remains blocked until the selected
release scope has accepted DCR evidence at the required validation level and all
public/custody claims are dispositioned.

## Integrated evidence set

| Area | Evidence pointer | Integration status |
|---|---|---|
| Work-package closure | `docs/vtrace/WORK_PACKAGES.md`; `docs/vtrace/STAGE_EXECUTION.md` | WP-001 through WP-008 are closed or closed pass-with-risk at their recorded validation level. |
| Trace control | `docs/vtrace/TRACE.md` | Accepted S0-S4 IDs are mapped to work packages and DCRs; implementation compliance is not overclaimed. |
| Import/package interoperability | `docs/fixtures/import-label/`; `docs/vtrace/IMPORT_COMPATIBILITY.md` | DCR-001 is closed at L2 for named CSV, GeoJSON, RPLAN, and shapefile/DBF label-import fixtures; broader tool-specific round-trip claims remain out of scope. |
| Release smoke | `docs/vtrace/RELEASE_SMOKE_BUNDLE.md`; `docs/vtrace/REPRODUCIBILITY_RUN.md` | DCR-002 is closed at L1 for the VT release-smoke scope; this is not full-scale or clean release-subset reproducibility. |
| External operator workflow | `docs/vtrace/EXTERNAL_WALKTHROUGH.md`; `scripts/maintenance/dcr003_walkthrough_record.py` | DCR-003 is ready for external run but remains open for L2 until a real non-author completes a walkthrough and friction is dispositioned. |
| Evidence package contract | `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md` | DCR-004 is closed at L1; L2 requires review against a concrete public bundle. |
| Legal/court packaging boundary | `docs/legal/COURT_PACKAGING_BOUNDARY.md` | DCR-006 is closed at L1; court-ready or filing-ready language remains blocked without jurisdiction-specific human/legal review. |
| Reproducibility replay | `scripts/maintenance/dcr007_release_subset_replay.py`; `scripts/maintenance/dcr007_clean_replay.py`; `docs/vtrace/REPRODUCIBILITY_RUN.md` | DCR-007 has candidate data-dirty evidence and clean-run tooling, but L2 clean replay remains open. |
| Custody and publication | `.gitignore`; `data/README.md`; `data/manifest.json`; `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md` | Generated outputs and source data remain local/ignored unless promoted by VAULT/public-claim review. |

## DCR integration posture

| DCR | Integrated status | S5 consequence |
|---|---|---|
| DCR-001 | `closed_l2` | Fixture-backed import compatibility can be cited only for the named fixture set. |
| DCR-002 | `closed_l1` | VT release-smoke health can support internal integration checks, not public release health. |
| DCR-003 | `partial_l1_ready_for_external_run` | Non-author readiness cannot be claimed until external walkthrough evidence exists. |
| DCR-004 | `closed_l1` | Evidence-package contract can guide internal bundle screening; public bundle claims still need L2 review. |
| DCR-005 | `closed_l1` | Compatibility matrix is centralized, but public interoperability claims must stay fixture-backed. |
| DCR-006 | `closed_l1` | Legal/court packaging boundary is controlled; filing/court readiness remains out of software scope. |
| DCR-007 | `partial_l1_release_subset_candidate_data_dirty` | Clean release-subset or full-scale reproducibility remains blocked until clean replay review. |

## Integration gates

| Gate | Required result before S6 transition | Current result |
|---|---|---|
| Trace gate | Every S6 claim must cite a work package, DCR, evidence artifact, validation level, and residual risk. | pass for control-plane mapping; release claims remain bounded. |
| Validation gate | Selected release scope must name smoke, fixture, external-user, reproducibility, and package-audit evidence. | partial; fixture and smoke evidence exist, external-user and clean replay evidence remain open. |
| Claim gate | Public-facing language must use evidence class names: fixture, smoke, candidate, L1 contract, L2 reviewed, or blocked. | pass with risk; future release text still needs review. |
| Custody gate | Any promoted bundle must record source data, generated artifacts, hashes, immutability/supersession, and VAULT disposition. | partial; contract exists, concrete public bundle review remains open. |
| Legal boundary gate | Evidence package, legal review package, and court-ready filing package must remain separate. | pass at L1; L2 legal/public review remains required for any filing-ready language. |

## S5 allowed claims

- BISECT has completed VTRACE S4 work-package closure with recorded residual risks.
- DCR-001 is closed at L2 for the named import-label fixture set.
- DCR-002, DCR-004, DCR-005, and DCR-006 are closed at L1 for their declared smoke, contract, matrix, and boundary scopes.
- DCR-003 and DCR-007 are ready for stronger evidence collection but are not L2 closed.

## S5 blocked claims

- Do not claim public release readiness.
- Do not claim legal, court, official, or statutory certification.
- Do not claim clean release-subset or full-scale reproducibility.
- Do not claim non-author usability validation.
- Do not claim universal external-tool interoperability beyond named fixtures.

## Required next action

Select an S6 transition target before any readiness decision:

1. Internal engineering baseline with L1 residual risks accepted.
2. Public evidence-package candidate requiring VAULT/DATUM/SCALE/COMMONS review.
3. Clean release-subset reproducibility run requiring DCR-007 L2 evidence.
4. External-user readiness run requiring DCR-003 L2 evidence.

Until one target is selected, S5 remains `in_progress_l1_control` and S6 remains
blocked.
