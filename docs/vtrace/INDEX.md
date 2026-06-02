# BISECT VTRACE Baseline Index

## Scope

This index is the entry point for the BISECT VTRACE baseline under
`docs/vtrace/`.

Current posture: `internal_engineering_baseline_only`.

The VTRACE baseline is accepted for controlled internal engineering continuation.
It is not a public release, legal/court filing package, external-user validation
record, clean reproducibility certificate, or official certification.

## Reading order

| Step | Artifact | Purpose |
|---|---|---|
| 1 | `MISSION.md` | Mission objectives and non-goals. |
| 2 | `CONOPS.md` | Operating concept, actors, success paths, degraded paths, and non-claims. |
| 3 | `REQUIREMENTS.md` | Accepted requirements baseline. |
| 4 | `SPECIFICATION_BASELINE.md` | Specification controls derived from requirements. |
| 5 | `ARCHITECTURE.md`, `PACKAGE_BOUNDARIES.md`, `INTERFACES.md`, `DESIGN.md` | S2 design baseline and package/interface controls. |
| 6 | `CODE_RIGOR.md`, `IMPLEMENTATION_PLAN.md`, `WORK_PACKAGES.md` | S3/S4 implementation constraints, sequencing, evidence, and closure posture. |
| 7 | `DCRS.md` | Release-readiness residuals converted into controlled design change requests. |
| 8 | `INTEGRATION.md` | S5 integration control across work packages, DCRs, validation, claims, and custody. |
| 9 | `READINESS_DECISION.md` | S6 decision: internal baseline only; stronger readiness blocked. |
| 10 | `BASELINE_HANDOFF.md` | Maintainer allowed actions, stop gates, and continuation checklist. |

## Evidence and control artifacts

| Artifact | Use |
|---|---|
| `TRACE.md` | Trace-control spine from mission through DCRs and S6 transition controls. |
| `REVIEW.md` | Review-gate ledger and dispositions. |
| `STAGE_EXECUTION.md` | Stage board and evidence ledger. |
| `IMPORT_COMPATIBILITY.md` | Import/export compatibility matrix and fixture-backed claim limits. |
| `RELEASE_SMOKE_BUNDLE.md` | L1 VT release-smoke scope and evidence. |
| `EXTERNAL_WALKTHROUGH.md` | DCR-003 external-operator packet; not L2 evidence until a real non-author run is accepted. |
| `EVIDENCE_PACKAGE_CONTRACT.md` | Public evidence package contract and L1 checklist; not a concrete public bundle acceptance. |
| `REPRODUCIBILITY_RUN.md` | Release-subset smoke/candidate replay posture and DCR-007 clean replay packet. |
| `RELEASE_GATE_REGISTER.md` | Operator-facing register for remaining release-grade gates; routing control only, not release evidence. |
| `ARTIFACT_PUBLICATION_POLICY.md` | Commit/publication rules for source, generated, package, paper, dashboard, and evidence artifacts. |
| `PAPER_EVIDENCE_INVENTORY.md` | Paper-index evidence posture classifier and declared gap rows for DREQ-002. |
| `PACKAGE_SPEC_REGISTER.md` | Package-family schema/canonicalization routing register for DREQ-004. |
| `COMMUNICATIONS_STRATEGY.md` | Audience, channel, allowed-message, blocked-language, and review-lane strategy for public/operator communications. |
| `COMMUNICATIONS_IMPLEMENTATION_AUDIT.md` | L1 audit applying the communications strategy to current public/operator documentation surfaces. |
| `../../context/waves/2026-06-01-vtrace-baseline-maintenance/CLOSE.md` | Closure record for the first live VTRACE-governed internal maintenance wave. |

## Current stage summary

| Stage | Status | Claim boundary |
|---|---|---|
| S0-S3 | complete or accepted with risk | Planning/control baseline only. |
| S4 | complete L0 pass-with-risk | Work packages closed or pass-with-risk; DCR residuals preserve stronger gates. |
| S5 | complete L1 control for internal baseline | Integrates evidence and blocked claims; not release readiness. |
| S6 | internal engineering baseline only | Maintainers may continue controlled internal work; public/release/legal/external-user/clean-replay readiness remains blocked. |

## Blocked claims

Do not use this baseline to claim:

- Public release readiness.
- Legal, court-ready, filing-ready, official, approved, or certified status.
- Non-author usability validation.
- Clean release-subset or full-scale reproducibility.
- Universal external-tool interoperability beyond named fixtures.
- Public evidence-package publication readiness.

Use `BASELINE_HANDOFF.md` before starting future VTRACE-governed work, and use
`READINESS_DECISION.md` before changing any readiness posture.
