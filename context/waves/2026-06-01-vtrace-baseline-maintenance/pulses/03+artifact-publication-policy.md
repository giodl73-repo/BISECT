---
pulse: 03
title: Artifact publication policy
status: done
validation_level: L1_control
parent_ids:
  - DREQ-001
  - DCR-004
  - DCR-007
  - CR-011
  - CR-012
  - CR-013
governing_roles:
  - VAULT
  - DATUM
  - SCALE
  - COMMONS
  - COVENANT
claim_boundary: artifact custody/publication routing only; no artifact promotion
custody_disposition: no generated artifacts promoted
---

# Pulse 03: Artifact Publication Policy

## Objective

Resolve DREQ-001 at the control level by documenting when BISECT artifacts may
be committed or published, while preserving the default local-only disposition
for generated data, outputs, maps, dashboards, reports, and release bundles.

## Pre-implementation Scout

Executed before editing:

```powershell
git --no-pager status --short
git --no-pager log -5 --oneline
```

Inspected:

- `context/waves/2026-06-01-vtrace-baseline-maintenance/WAVE.md`
- `docs/vtrace/REQUIREMENTS.md`
- `docs/vtrace/EVIDENCE_PACKAGE_CONTRACT.md`
- `docs/vtrace/BASELINE_HANDOFF.md`
- `docs/vtrace/RELEASE_GATE_REGISTER.md`

## Deliverables

- [x] Add `docs/vtrace/ARTIFACT_PUBLICATION_POLICY.md`.
- [x] Route DREQ-001 to a concrete publication/custody control.
- [x] Update the active wave pulse table.
- [x] Update VTRACE index, trace, stage, rigor, and review records.
- [x] Preserve local-only disposition for generated artifacts and raw data.

## Closure Notes

Decision: `artifact_publication_policy_active`.

This pulse adds L1 control evidence only. It does not publish an evidence
package, promote generated artifacts, close DCR-004 at L2, close DCR-007 at L2,
or upgrade S6 readiness.

## Validation

```powershell
git --no-pager diff --check
$stale = git grep -n -E "Future readiness record|not_started|in_progress_l1_control|S6 remains blocked until|no transition target" -- docs/vtrace context/waves | Select-String -NotMatch "git grep -n -E"
if ($stale) { $stale; exit 1 } else { "stale-status-search: pass" }
git grep -n "ARTIFACT_PUBLICATION_POLICY.md" -- docs/vtrace context/waves
```
