# Detailed Design

## DS-00 Scope

Repo or feature: BISECT / apportionment workspace at `C:\src\apportionment`.

Stage: S2 Design Baseline accepted.

This artifact records design decisions, invariants, edge cases, migration concerns, and code-rigor hooks that connect the accepted specification baseline to implementation and future wave/pulse work. Decision IDs are accepted for S2 trace use; later changes require trace-preserving disposition.

## DS-01 Design Decision Summary

| ID | Decision | Requirement IDs | Rationale | Alternatives | Evidence | Status |
|---|---|---|---|---|---|---|
| DES-001 | Keep VTRACE artifacts as the control plane under `docs/vtrace/`, separate from implementation code. | REQ-001, REQ-002, REQ-036 | Enables trace without embedding process logic in crates. | Put requirements in code comments only; rejected because trace/review needs durable docs. | Existing Mission/CONOPS/Requirements/Spec baseline. | accepted |
| DES-002 | Treat accepted IDs as stable once a stage is locked; changes require supersede/retire/split/merge disposition. | REQ-002, REQ-031, REQ-036 | Prevents downstream trace breakage. | Renumber freely; rejected after stage lock. | SPEC-001, SPEC-011, IF-007. | accepted |
| DES-003 | Use package-family boundaries for RPLAN, RCOUNT, RCTX, and RHIST rather than one generic civic package. | REQ-013..REQ-023 | Integrity mechanics are shared, but plan/count/context/history semantics differ. | Single package schema; rejected because it would flatten domain semantics. | SPEC-007, SPEC-008, SPEC-009. | accepted |
| DES-004 | Keep run completion, legal readiness, election certification, package audit, and public claim readiness as separate states. | REQ-005, REQ-010, REQ-011, REQ-018, REQ-025, REQ-034 | Avoids success-shaped evidence and authority overreach. | Single pass/fail status; rejected as misleading. | CONOPS CO-04..CO-09. | accepted |
| DES-005 | Require provenance and deterministic replay metadata for evidence-producing redistricting runs going forward. | REQ-007, REQ-008, REQ-009, REQ-012 | The SHA chain is insufficient without source/executable/search metadata. | Record only output hashes; rejected because replay would be incomplete. | SPEC-005, label pipeline docs. | accepted |
| DES-006 | Treat version-unknown compatibility as a recorded finding, not an implicit pass. | REQ-003, REQ-031 | Current docs, binaries, package schemas, and external adapters may drift. | Assume current behavior is compatible; rejected. | SPEC-002, IF-001..IF-008. | accepted |
| DES-007 | Preserve election-count accounting concepts explicitly in RCOUNT. | REQ-016, REQ-017, REQ-018, REQ-019, REQ-020 | Counts are lifecycle and jurisdiction artifacts, not just totals. | Normalize immediately to flat totals; rejected. | CO-06, SPEC-008. | accepted |
| DES-008 | Require verified or explicitly partial RCTX/RHIST links before strong count-to-district claims. | REQ-021, REQ-022, REQ-023 | Aggregation depends on trustworthy geography/context joins. | Aggregate through unverified crosswalks silently; rejected. | CO-07, SPEC-009. | accepted |
| DES-009 | Classify public claims by posture and evidence status before publication or lock. | REQ-005, REQ-006, REQ-024, REQ-025, REQ-026, REQ-027 | Public claims can outrun current evidence unless reviewed. | Rely on paper/report existence as proof; rejected. | SPEC-004, SPEC-010. | accepted |
| DES-010 | Maintain Rust production path independence from Python runtime for core redistricting. | REQ-007, REQ-012, REQ-031, REQ-032 | Rust binary is the maintained production tool; Python remains support/research unless promoted. | Reintroduce Python orchestration as required production dependency; rejected. | README, `docs/BISECT_CLI.md`, Cargo workspace. | accepted |
| DES-011 | Use waves/pulses as implementation units only when they name requirements, boundaries, validation commands, role gates, risks, and pitfalls. | REQ-028, REQ-029, REQ-030, REQ-034, REQ-035 | Integrates VTRACE into the existing execution model. | Complete ad hoc changes without trace; rejected. | `context/waves/PHASES.md`, SPEC-011. | accepted |
| DES-012 | Promote generated artifacts to public evidence only through custody, privacy, and claim review. | REQ-004, REQ-020, REQ-026, REQ-027, REQ-033 | Prevents protected-data leaks and unsupported publication. | Commit/generated artifacts by convenience; rejected. | SPEC-003, SPEC-010. | accepted |
| DES-013 | Treat release-readiness residuals as DCRs rather than reopened satisfied work packages. | REQ-028, REQ-034, REQ-035, REQ-036 | S4 closure remains meaningful while S5/S6 readiness work stays controlled. | Reopen all WPs for release work; rejected because it blurs validation levels. | `docs/vtrace/DCRS.md`, `TRACE.md`, `STAGE_EXECUTION.md`. | filed |

## DS-02 Algorithms / Logic

### Redistricting algorithm control

BISECT redistricting is controlled by a three-layer compositor:

1. Structure: bisection tree or construction family.
2. Weights: METIS/graph objective signal.
3. Search: seed and candidate-selection strategy.

The design requires that evidence-producing runs record the resolved layer values, population tolerance, METIS engine, seed/search replay data, selected candidate metadata, executable provenance, and source custody. A run may complete without legal readiness; legal gates are separate design states.

### Package audit logic

RPLAN, RCOUNT, RCTX, and RHIST packages use a common audit posture:

```text
source/package input
  -> parse and schema/version check
  -> canonicalization/hash or semantic validation
  -> package-family completeness checks
  -> status: pass / pass with risk / blocked / deferred / partial / gap
  -> evidence and limitations
```

The common status vocabulary does not erase domain-specific semantics. RCOUNT lifecycle status, RCTX/RHIST lineage status, and RPLAN legal-boundary caveats remain separate.

### Public claim logic

Public claims in README, docs, dashboards, papers, or reports require:

1. claim posture: descriptive, empirical, statistical, legal, operational, or advocacy;
2. evidence status: proven, partial, conjectural, stale, blocked, or gap;
3. data/command/package/table/figure/review pointer;
4. assumptions, uncertainty, comparison baseline, and political/community-effect scope where relevant;
5. explicit non-claims when a reader could infer too much.

### Wave/pulse logic

Future implementation pulses must start from accepted VTRACE IDs and name:

- affected requirements/specs/interfaces/design decisions;
- package boundaries;
- validation commands or review methods;
- governing roles;
- risk and pitfall disposition;
- verification state.

## DS-03 Invariants

- Accepted VTRACE IDs are stable after lock.
- No accepted requirement may remain orphaned from specification, interface/design, verification, validation, or an explicit deferral.
- Version unknown is never equivalent to compatible.
- Source custody gaps must be visible in evidence status.
- A successful CLI run does not establish legal readiness.
- A successful package audit does not establish legal validity, election certification, or public claim readiness.
- RCOUNT must not flatten distinct election accounting concepts into totals without preserving source semantics and lifecycle.
- RCOUNT normalization must retain enough source lineage, parser diagnostics, and raw-source disagreement evidence for an independent parser or reviewer to dispute or reproduce the normalization.
- Unknown RCTX/RHIST lineage cannot be treated as verified.
- Public artifacts must not expose ballot-choice receipts, reidentification risk, coercion channels, protected inputs, or unnecessary sensitive intermediates.
- Research and README claims must not outrun evidence status.
- Rust production redistricting must not require the Python support pipeline.
- Archived code remains forensic reference unless explicitly promoted through boundary review.

## DS-04 Edge Cases

| Edge Case | Expected Behavior | Verification |
|---|---|---|
| Current CLI help differs from docs. | Mark affected command/flag version unknown or correct interface docs before compatibility claim. | CLI inspection. |
| Existing historical run lacks seed/search/executable provenance. | Treat as historical partial/gap evidence; require full target metadata going forward. | Evidence review. |
| Source data is present locally but has no source URL/date/hash/custody record. | Do not make strong provenance claim; record custody gap. | Source custody inspection. |
| METIS engine or external tool version is unknown. | Record version unknown and block byte-identical replay claims if tool identity matters. | Run manifest/tool inspection. |
| METIS engine path differs by build or runtime (`c-ffi`, `bisect-metis`, `gpmetis`, `metis-core`, `metis`). | Preserve engine family, crate/tool source, version/hash or version-unknown status, and feature flags before making reproducibility claims. | Run manifest/tool inspection. |
| Package hash passes but schema version is unknown. | Audit result is pass with risk or version-unknown finding, not clean pass. | Package audit fixture. |
| RCOUNT source lacks CVRs or batch detail. | Preserve absence explicitly; do not invent missing records; limit claims. | Count audit review. |
| Election totals are amended/recounted after initial package. | Preserve lifecycle event history and distinguish unofficial/canvassed/recounted/amended/certified states. | Lifecycle fixture. |
| Crosswalk is partial or conjectural. | Downstream district aggregation may proceed only with partial/conjectural status and limited claims. | Context validation fixture. |
| Public dashboard number changes after rerun. | Trigger claim review and evidence pointer update. | Claim review. |
| Legal doc cites algorithmic output. | Keep statutory/legal argument distinct from run verification; require legal-boundary review. | BOUNDARY/WARD review. |
| Python script generates a paper figure. | Treat script and inputs as evidence dependencies for that figure. | Paper evidence review. |
| Wave pulse fixes code but omits validation. | Pulse remains incomplete or blocked. | Pulse review. |
| Release-readiness risk remains after S4 closure. | File or update a DCR with acceptance criteria, validation level, owner lane, and trace links; do not claim readiness from filing alone. | DCR review. |

## DS-05 Migration / Rollout

The VTRACE adoption rollout is staged and review-gated:

1. S0/S1 baselines are locked: Mission, CONOPS, Requirements, Specification Baseline.
2. S2 design baseline is accepted in this artifact set: Architecture, Package Boundaries, Interfaces, Detailed Design.
3. `.roles` review resolved S2 blockers before S2 IDs locked.
4. The next stage should create trace, verification, validation, review, and DCR artifacts using locked S2 IDs.
5. The first live VTRACE-governed wave/pulse should select a narrow implementation slice and prove the process end to end.
6. Release-readiness work should close the relevant DCRs before S6 transition claims.

Compatibility posture during rollout:

- Existing docs/code are not claimed compliant merely because they are described here.
- Historical artifacts may receive partial/gap status instead of being regenerated.
- Stable public contracts must be corrected or version-unknown before strong compatibility claims.
- Generated artifacts remain governed by current repo hygiene until explicit publication rules are locked.

## DS-06 Code Rigor Hooks

| Area | Risk | Required Code Rigor Constraint | Status |
|---|---|---|---|
| CLI/interface changes | Silent compatibility break. | Tests or inspections for command/flag/default/output-path changes plus migration notes. | accepted |
| Run manifests/provenance | Incomplete replay metadata. | Fixture or smoke test asserting required provenance fields for evidence-producing runs. | accepted |
| Search/replay logic | Non-reproducible candidate selection. | Deterministic seed/candidate metadata test. | accepted |
| Package canonicalization/hashing | Irreproducible audit results. | Golden package tests with canonical bytes and domain-separated hashes. | accepted |
| RCOUNT parsers | Flattened election accounting semantics. | Fixtures for contests, ballots, ballot cards, batches, CVR absence/presence, over/under votes, lifecycle states, parser diagnostics. | accepted |
| RCTX/RHIST crosswalks | Unknown lineage treated as verified. | Fixtures for verified, partial, conjectural, blocked, and gap status. | accepted |
| Public report generation | Unsupported or stale claims. | Claim-review checklist tied to evidence pointers before release. | accepted |
| Source custody | Protected/generated data leak. | Commit/release inspection and custody disposition record. | accepted |
| Wave/pulse completion | Work closes without validation/risk disposition. | Pulse checklist requiring requirement IDs, boundaries, validation, role gates, and pitfall status. | accepted |
| Release readiness | S4 closure mistaken for public, court, or full reproducibility readiness. | DCR closure evidence for fixtures, smoke, walkthrough, evidence contract, legal boundary, and reproducibility gates. | filed |

## DS-07 Design Gate

Decision: accepted for S2 Design Baseline.

S2 lock disposition:

- [x] Design decisions map to accepted requirements/specs/interfaces.
- [x] Invariants cover package integrity, legal/election boundaries, provenance, privacy, and claim discipline.
- [x] Edge cases are concrete enough for verification/validation planning.
- [x] Code rigor hooks identify tests, fixtures, demonstrations, inspections, or role reviews.
- [x] Role review findings are fixed, deferred, or accepted as risk.
