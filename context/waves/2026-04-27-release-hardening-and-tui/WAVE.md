---
wave: release-hardening-and-tui
date_open: 2026-04-27
date_close: 2026-04-30
status: archived
backfill: true
confidence: medium-high
---

# Release Hardening And TUI

## Mission

Run scenario/user audits, harden CLI behavior and verification, add a TUI, CI
release scaffolding, PlanContext metadata, and fresh-install UX fixes.

## Evidence

Representative commits:

- `27b5cf87` Fix scenarios 3/10/21/24 from 25-user audit
- `699953c3` Tier 1 critical correctness fixes
- `81e24e98` Reproducibility package and executable verification script
- `bdce6a79` Design spec: redist TUI
- `83ac8ec7` redist TUI v1 polish
- `10c7de03` GitHub Actions test/release binaries
- `c72e9073` PlanContext single source of truth
- `08816dd7` Fix 9 fresh-install UX gaps

## Tracks

- CLI UX and correctness.
- TUI.
- CI/release.
- PlanContext and metadata resolution.
- International/location runs.

## Established

- Scenario-audit-driven hardening.
- Test count and release gate culture.
- Metadata resolution patterns later echoed in package references.

## Carry Forward

Later waves renamed `redist` to `bisect` and shifted from release UX to
algorithm-family and package evidence.

