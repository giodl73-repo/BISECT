# Implementation Plan

## Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Implementation baseline: accepted S1 and S2 VTRACE artifacts plus accepted S3 work-package and code-rigor controls.

## Baseline Inputs

| Artifact | Status | Notes |
|---|---|---|
| `MISSION.md` | accepted | Mission IDs `M-*` are locked. |
| `STAGE_EXECUTION.md` | accepted with risk | S3 board; records stage progression and open findings. |
| `CONOPS.md` | accepted | Scenario IDs `CO-*` are locked. |
| `REQUIREMENTS.md` | accepted | Requirement IDs `REQ-001` through `REQ-037` are locked. |
| `SPECIFICATION_BASELINE.md` | accepted | `SPEC-*`, `SPEC-NF-*`, and `IF-*` IDs are locked. |
| `ARCHITECTURE.md` | accepted | `ARCH-*` and package mapping are accepted for S2. |
| `PACKAGE_BOUNDARIES.md` | accepted | `PKG-001` through `PKG-014` are accepted for S2. |
| `INTERFACES.md` | accepted | Expands `IF-001` through `IF-008`. |
| `DESIGN.md` | accepted | `DES-001` through `DES-012` are accepted for S2. |
| `CODE_RIGOR.md` | accepted with risk | `CR-*` constraints accepted for S4 execution, with evidence pending per work package. |
| `VERIFICATION.md` | pending | To be drafted after S3 work packages are accepted. |
| `VALIDATION.md` | pending | To be drafted after verification scope is known. |
| `REVIEW.md` | accepted with risk | Captures S3 role findings and gate decision. |
| `DCRS.md` | filed | Release-readiness DCRs DCR-001 through DCR-007 are filed as S5/S6 follow-on controls. |

## Implementation Strategy

Implement VTRACE adoption through bounded work packages rather than one large unstructured cleanup. S4 execution should proceed through waves/pulses or explicit issue/PR scopes that cite accepted IDs, touch only named package boundaries, and close only after the planned verification or review evidence exists.

This strategy intentionally separates:

- trace/control-plane work from executable product changes;
- compatibility discovery from compatibility claims;
- package audit mechanics from legal/election/public authority claims;
- historical evidence gap disposition from new forward-looking evidence requirements;
- documentation/research claim review from algorithmic correctness.

## Sequencing

| Order | Work Package | Why This Order |
|---:|---|---|
| 1 | WP-001 VTRACE control plane and trace matrix | Establishes trace spine and orphan checks before implementation work closes. |
| 2 | WP-002 Interface and boundary verification | Prevents CLI/schema/adapter compatibility claims before inspection. |
| 3 | WP-003 BISECT provenance and replay evidence | Supports reproducibility claims for algorithmic outputs. |
| 4 | WP-004 Package-family audit fixtures | Covers RPLAN/RCOUNT/RCTX/RHIST integrity and domain semantics. |
| 5 | WP-005 Public claim and research evidence review | Aligns README/docs/papers/dashboards with current evidence. |
| 6 | WP-006 Source custody and artifact publication controls | Blocks accidental publication of protected or generated-transient artifacts. |
| 7 | WP-007 Wave/pulse closure and pitfall controls | Integrates VTRACE into future work execution. |
| 8 | WP-008 Non-author workflow documentation | Publishes usable commands and evidence locations after controls are known. |

## Source-To-Work-Package Mapping

| Source IDs | Work Package | Disposition | Notes |
|---|---|---|---|
| M-01, M-02, M-03, M-04, M-05, M-06, M-07, M-08, M-09, CO-01, CO-02, CO-10, CO-11, CO-12 / REQ-001, REQ-002, REQ-028, REQ-029, REQ-030, REQ-034, REQ-035, REQ-036 / SPEC-001, SPEC-011 / ARCH-005 / IF-007 / DES-001, DES-002, DES-011 / PKG-001, PKG-013 / CR-013 | WP-001, WP-007 | implement | Mission, trace, stage gates, wave/pulse controls, pitfall status. |
| CO-03 / REQ-003, REQ-031, REQ-037 / SPEC-002, SPEC-NF-004, SPEC-NF-007 / IF-001..IF-008 / ARCH-001, ARCH-002, ARCH-004 / DES-006 / PKG-003..PKG-014 / CR-001..CR-005 | WP-002 | discovery / implement | Inspect current CLI, config, schema, adapter, and dependency boundaries before compatibility claims. |
| REQ-007, REQ-008, REQ-009, REQ-010, REQ-011, REQ-012 / SPEC-005, SPEC-006, SPEC-NF-001 / IF-001, IF-002 / DES-004, DES-005, DES-010 / PKG-007, PKG-008, PKG-009, PKG-010 / CR-001, CR-002, CR-004, CR-006, CR-007 | WP-003 | implement | Run provenance, engine disclosure, deterministic replay, output inspection. |
| REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-020, REQ-021, REQ-022, REQ-023 / SPEC-007, SPEC-008, SPEC-009 / IF-003, IF-004, IF-005, IF-008 / DES-003, DES-007, DES-008 / PKG-004, PKG-005, PKG-006 / CR-001, CR-002, CR-004, CR-008, CR-009, CR-010 | WP-004 | implement / discovery | Package family schemas, canonicalization, audit fixtures, count/context/history semantics. |
| CO-08 / REQ-005, REQ-006, REQ-024, REQ-025, REQ-026 / SPEC-004, SPEC-010, SPEC-NF-005 / IF-006 / ARCH-003 / DES-004, DES-009 / PKG-012 / CR-011 | WP-005 | implement / discovery | Public claim inventory and evidence status. |
| REQ-004, REQ-020, REQ-027, REQ-033 / SPEC-003, SPEC-010, SPEC-NF-002 / IF-002, IF-006, IF-008 / DES-012 / PKG-002, PKG-011, PKG-014 / CR-012 | WP-006 | implement | Custody, privacy, generated artifact disposition, archive protection. |
| REQ-010, REQ-011, REQ-018 / SPEC-006, SPEC-008 / IF-004, IF-006 / DES-004 / PKG-005, PKG-007, PKG-012 | WP-003, WP-004, WP-005 | implement / review | Keep legal readiness, election certification, and package/public evidence states separate. |
| REQ-032 / SPEC-012 / IF-001, IF-006 / DES-011 / PKG-010, PKG-012, PKG-013 / CR-011 | WP-008 | implement | Non-author user workflows, commands, outputs, failure modes, evidence locations. |

## Release-Readiness DCR Roadmap

S4 WP-001 through WP-008 are closed at their recorded validation levels. The items below are follow-on DCRs for stronger release, public interoperability, and transition claims.

| DCR | Objective | Depends On | Required Before |
|---|---|---|---|
| DCR-001 | Golden interop fixtures for public import/package compatibility. | WP-002, WP-004 | Public interoperability claims. |
| DCR-002 | Release smoke bundle for a small representative build/analyze/report/verify path. | WP-003, WP-008 | Release-health claims. |
| DCR-003 | External-user walkthrough of non-author docs; operator packet now ready. | WP-008 | Public user-readiness claims. |
| DCR-004 | Public evidence package contract. | WP-005, WP-006 | Stable downstream evidence-package use. |
| DCR-005 | Import compatibility matrix. | WP-002 | Central external-adapter documentation. |
| DCR-006 | Court/legal packaging boundary. | WP-005, WP-006 | Court-ready or filing-ready package language. |
| DCR-007 | Full-scale or declared-subset reproducibility run. | WP-003 | Full-release reproducibility claims. |

## Boundary-To-Work-Package Mapping

| Boundary IDs | Work Package | Allowed Touches | Integration Needed |
|---|---|---|---|
| PKG-001, PKG-013 | WP-001, WP-007 | VTRACE docs, trace rows, wave/pulse checklists, pitfall records. | yes |
| PKG-003..PKG-014 | WP-002 | Read/inspect Cargo, scripts, docs, CLI help, schemas, adapters; modify only to record discovered compatibility/boundary status. | yes |
| PKG-007, PKG-008, PKG-009, PKG-010 | WP-003 | Run metadata, manifests, CLI docs/help alignment, report/verify evidence records. | yes |
| PKG-004, PKG-005, PKG-006 | WP-004 | Package schema docs, fixtures, canonicalization/audit tests, domain validation fixtures. | yes |
| PKG-012 | WP-005 | README/docs/papers/dashboards/report evidence indexes and claim-review records. | yes |
| PKG-002, PKG-011, PKG-014 | WP-006 | `.gitignore`, custody docs, release/publication checks, archive read-only disposition. | yes |
| PKG-010, PKG-012, PKG-013 | WP-008 | Quickstarts, CLI references, evidence-location docs, non-author workflow guides. | yes |

## Branch / Change Control

Branch strategy: continue in the current working branch unless a future S4 work package requires isolation.

Worktree strategy: do not modify `archive/python-pipeline-final/` except through an explicit boundary decision.

Change-control trigger: any accepted ID change, interface contract change, generated-artifact publication, package schema/canonicalization change, public headline claim change, or legal/election authority statement change.

Rollback or revert strategy: use normal git commits/PR review; never erase accepted IDs without supersede/retire/split/merge disposition.

## Commit / Push Policy

Commit scope: one VTRACE stage or coherent work package per commit where practical.

Push condition: S3 review accepted or explicit user instruction.

Merge/readiness condition: required verification evidence and role-gate disposition are present for the relevant stage/work package.

## Wave / Pulse Policy

Active wave: no active wave is currently listed in `context/waves/PHASES.md`; R Package Completion is archived historical context, not an active VTRACE-governed pulse.

Pulse mapping rule: every S4 pulse governed by this baseline must name parent `REQ-*`, `SPEC-*`, `IF-*`, `PKG-*`, `DES-*`, applicable `CR-*`, validation level, role lanes, and close evidence.

Pulse close condition: no pulse closes on "implemented" alone; it must carry verification result, validation status or non-applicability, risk disposition, and public/custody impact.

WP-007 disposition: `context/waves/PHASES.md` is the shared wave/pulse convention and now requires future VTRACE-governed pulses to include parent IDs, affected boundaries, validation level, role gates, verification state, risk/pitfall disposition, and public/custody effects. Existing archived pulses remain historical evidence and are not retroactively rewritten.

## Integration Strategy

S4 work packages should produce evidence in the same artifacts that S5 will integrate: trace rows, verification records, validation scenarios, role findings, package fixtures, run manifests, claim-review records, and custody dispositions. Integration must preserve package-boundary semantics rather than merging RPLAN/RCOUNT/RCTX/RHIST or BISECT public evidence into a single generic status.

## Verification Strategy

Verification is selected per work package. Documentation-only planning changes use inspection and structural checks. Code/package changes use existing repository commands and package-specific fixtures; no new build/test tool should be introduced solely by S3 planning.

```powershell
git --no-pager diff --check -- docs\vtrace
cargo test -p bisect-cli --lib -- --test-threads=1
cargo test -p bisect-ensemble
pytest tests\unit\ -v
```

The command list above is a menu for future S4 selection, not a requirement for documentation-only planning edits.

## Validation Levels

| Level | Scope | Required Commands / Evidence | Required Before |
|---|---|---|---|
| L0 | Artifact structure, trace completeness, local unit/fixture where applicable | Markdown inspection, orphan checks, targeted unit tests, diff checks | S4 work-package close |
| L1 | Repo confidence for affected package families | Existing Cargo/Python tests or package fixtures selected by work package | PR/pulse close where code changes occur |
| L2 | Integration, release, public evidence, or downstream adoption readiness | End-to-end run, replay demonstration, public claim review, custody review, role panel | S5/S6 readiness or public release |

## Risks

| Risk ID | Risk | Mitigation | Owner |
|---|---|---|---|
| S3-RISK-001 | S3 work packages become too broad to execute. | Keep S4 pulses scoped to one package family, interface class, or evidence surface. | Maintainers |
| S3-RISK-002 | Compatibility is inferred from docs rather than current behavior. | WP-002 requires inspection before compatibility claims. | LEDGER / package owners |
| S3-RISK-003 | Historical outputs are treated as replayable without provenance. | WP-003 permits partial/gap status rather than reconstruction. | MERIDIAN / COVENANT |
| S3-RISK-004 | RCOUNT totals flatten election accounting semantics. | WP-004 and CR-009 require source lineage and parser diagnostics. | CANVASS / TALLY |
| S3-RISK-005 | Public claims outrun evidence. | WP-005 requires posture/status/evidence review. | DATUM / SCALE / COMMONS |
| S3-RISK-006 | Generated/protected artifacts are accidentally promoted. | WP-006 requires VAULT custody disposition. | VAULT / maintainers |
| S3-RISK-007 | Future pulses close as implemented without VTRACE evidence. | WP-007 updates wave/pulse closure rules and requires first future governed pulse to prove the checklist in practice. | BENCHMARK / TRENCH |
| S4-RISK-001 | S4 closure is mistaken for public release readiness. | DCR-001 through DCR-007 define the extra fixture, smoke, walkthrough, evidence-contract, legal-boundary, and reproducibility gates required for stronger claims. | Maintainers / role lanes |

## Implementation Readiness Decision

Decision: accepted with risk after `.roles` review.

Rationale: S1 and S2 are accepted, and S3 now provides sequenced work packages, code-rigor constraints, verification levels, role lanes, and risk controls. S4 may begin with WP-001 while preserving the open compatibility, provenance, custody, package-fixture, and public-claim risks as work-package evidence obligations rather than completed claims.
