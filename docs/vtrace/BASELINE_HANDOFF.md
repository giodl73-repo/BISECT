# Internal Baseline Handoff

## Scope

This handoff operationalizes the S6 decision in `READINESS_DECISION.md`.

Status: `internal_engineering_baseline_only`.

This document is for maintainers and project operators continuing engineering
work from the current VTRACE baseline. It is not a public release note, legal
filing packet, clean replay certificate, or external-user validation record.

## Starting posture

| Control area | Current posture | Operator instruction |
|---|---|---|
| VTRACE stage | S6 internal baseline only | Treat S0 through S6 as a controlled engineering baseline. |
| Work packages | WP-001 through WP-008 closed or closed pass-with-risk | Preserve the recorded residual risks unless a new DCR supersedes them. |
| DCR-001 | closed L2 for named import-label fixtures | Use only fixture-backed interoperability claims. |
| DCR-002 | closed L1 for VT release smoke | Use as internal smoke health, not all-state or public release proof. |
| DCR-003 | ready for external run, not L2 closed | Do not claim non-author usability validation. |
| DCR-004 | closed L1 contract, not concrete public bundle | Do not publish an evidence package without VAULT/public-claim review. |
| DCR-006 | closed L1 boundary, not legal authority | Do not claim court-ready, filing-ready, official, or certified status. |
| DCR-007 | release-subset candidate/data-dirty and clean-run ready | Do not claim clean reproducibility until a clean data-backed replay is accepted. |

## Allowed maintainer actions

Maintainers may:

1. Continue feature, bug-fix, and documentation work under the existing VTRACE
   trace IDs.
2. Run local smoke, fixture, and candidate replay commands for engineering
   evidence, provided the evidence class is named honestly.
3. Create new DCRs when a change affects public claims, package boundaries,
   reproducibility, custody, legal boundary, or external-user workflows.
4. Promote a stronger S6 posture only after the required DCR and review gates are
   updated in the same change.

## Required stop gates

Stop and open or update a DCR before any change that:

| Trigger | Required gate |
|---|---|
| Claims public release readiness | DCR-003, DCR-004, DCR-006, DCR-007, and custody/public-claim review. |
| Publishes generated reports, dashboards, maps, or bundles as official evidence | DCR-004 plus VAULT/DATUM/SCALE/COMMONS review. |
| Claims legal, court-ready, filing-ready, certified, approved, or statutory status | DCR-006 L2 legal-boundary review with named external authority. |
| Claims non-author usability or public operator readiness | DCR-003 L2 external walkthrough with friction disposition. |
| Claims clean replay, release-subset reproducibility, or full-scale reproducibility | DCR-007 L2 clean data-backed replay accepted by MERIDIAN/COVENANT. |
| Changes label artifact, manifest, package, or evidence contract shape | Interface/boundary trace update plus affected package tests or DCR. |

## Baseline continuation checklist

Before closing a future VTRACE-governed change, confirm:

- The changed code or docs cite the applicable requirement, specification,
  design, work-package, or DCR ID.
- Validation level is explicit: L0 inspection, L1 internal test/smoke/checklist,
  or L2 external/golden/clean replay evidence.
- Public-facing language avoids unsupported release, certification, court-ready,
  clean-replay, universal-compatibility, and non-author validation claims.
- Generated artifacts remain ignored/local unless custody review promotes them.
- `STAGE_EXECUTION.md`, `TRACE.md`, `CODE_RIGOR.md`, and `REVIEW.md` are updated
  when the change alters evidence posture.

## Handoff decision

The current baseline is suitable for controlled internal continuation. It is not
suitable for public release, legal filing, clean reproducibility publication, or
external-user readiness without the stop gates above.
