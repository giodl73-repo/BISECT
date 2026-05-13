---
wave: r-package-completion
pulse: 05
status: todo
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

- [ ] Positive rename/split/merge lineage fixture.
- [ ] Negative missing-unit or bad-weight fixture.
- [ ] RCOUNT reference test by package hash.
- [ ] RPLAN consumer note or fixture plan.
- [ ] Active goal and RHIST implementation docs updated.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```

