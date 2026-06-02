---
pulse: 05
title: Package spec register
status: done
validation_level: L1_control
parent_ids:
  - DREQ-004
  - REQ-003
  - REQ-013
  - REQ-014
  - REQ-016
  - REQ-021
  - REQ-022
  - IF-003
  - IF-004
  - IF-005
  - PKG-004
  - PKG-005
  - PKG-006
  - CR-008
  - CR-009
  - CR-010
  - CR-013
governing_roles:
  - LEDGER
  - CANVASS
  - CONTOUR
  - BENCHMARK
  - VAULT
claim_boundary: package spec routing only; no schema restatement or public package promotion
custody_disposition: no package fixtures, generated artifacts, or public bundles promoted
---

# Pulse 05: Package Spec Register

## Objective

Resolve DREQ-004 at the control level by adding an operator-facing package
specification register for RPLAN, RCOUNT, RCTX, and RHIST schema sources,
version identities, hash shapes, verifier paths, and change-control triggers.

## Pre-implementation Scout

Executed before editing:

```powershell
git --no-pager status --short
git --no-pager log -1 --oneline
Get-ChildItem -Name crates | Where-Object { $_ -match '^(rplan|rcount|rctx|rhist)' } | Sort-Object
```

Inspected:

- `docs/vtrace/REQUIREMENTS.md`
- `docs/vtrace/PACKAGE_BOUNDARIES.md`
- `docs/vtrace/INTERFACES.md`
- `docs/vtrace/WORK_PACKAGES.md`
- `docs/vtrace/IMPORT_COMPATIBILITY.md`
- `crates/rplan-*`
- `crates/rcount-*`
- `crates/rctx-core`
- `crates/rhist-*`
- `docs/specs/`

## Deliverables

- [x] Add `docs/vtrace/PACKAGE_SPEC_REGISTER.md`.
- [x] Route DREQ-004 to package-owned schema and canonicalization controls.
- [x] Record version identities, hash shapes, verifier paths, and change-control
      triggers for RPLAN, RCOUNT, RCTX, and RHIST.
- [x] Update the active wave pulse table.
- [x] Update VTRACE index, requirements, trace, stage, rigor, and review records.
- [x] Preserve public-package, interoperability, custody, and S6 readiness
      boundaries.

## Closure Notes

Decision: `package_spec_register_active`.

This pulse adds L1 control evidence only. It does not restate full package
schemas, create a package version, promote package fixtures, expand public
interoperability claims, close DCR-004 at L2, or upgrade S6 readiness.

## Validation

```powershell
git --no-pager diff --check
$stale = git grep -n -E "Future readiness record|not_started|in_progress_l1_control|S6 remains blocked until|no transition target" -- docs/vtrace context/waves | Select-String -NotMatch "git grep -n -E"
if ($stale) { $stale; exit 1 } else { "stale-status-search: pass" }
git grep -n "PACKAGE_SPEC_REGISTER.md" -- docs/vtrace context/waves
```
