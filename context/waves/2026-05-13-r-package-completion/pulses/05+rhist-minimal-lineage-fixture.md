---
wave: r-package-completion
pulse: 05
status: done
depends_on: [04]
governing_roles:
  - rhist
  - rcount-core
  - rcount-district
  - docs
---

# Pulse 05 - RHIST Minimal Lineage Fixture

## Mission

Lock the first RHIST lineage package slice so cross-cycle unit history has a
home outside RCOUNT and RPLAN.

## Deliverables

- [x] Positive rename/split/merge lineage fixture.
- [x] Negative missing-unit or bad-weight fixture.
- [x] RCOUNT reference test by package hash.
- [x] RPLAN consumer note or fixture plan.
- [x] Active goal and RHIST implementation docs updated.

## Completion Notes

- Positive fixture: `docs/fixtures/rhist/l2-three-cycle` is locked as the
  rename/split/merge lineage fixture.
- Negative fixtures: `docs/fixtures/rhist/l0-missing-unit` and
  `docs/fixtures/rhist/l1-bad-weights` remain active verifier coverage.
- Verifier hardening: `rhist-core` now verifies the declared manifest package
  hash against the canonical `RHIST_PACKAGE_V1` package projection.
- RCOUNT consumer: `rcount-core` references the locked RHIST fixture by its
  package hash and cycle ids, without importing RHIST lineage semantics.
- RPLAN consumer plan: pulse 06 should add the example-level RPLAN/RHIST note
  or fixture when RCOUNT/RPLAN reference integration is wired together.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```
