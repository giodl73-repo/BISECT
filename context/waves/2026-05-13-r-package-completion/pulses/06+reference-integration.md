---
wave: r-package-completion
pulse: 06
status: todo
depends_on: [04, 05]
governing_roles:
  - rctx-core
  - rhist
  - rcount-core
  - rcount-district
  - docs
---

# Pulse 06 - RCOUNT/RPLAN Reference Integration

## Mission

Use the RCTX and RHIST fixtures as shared base dimensions so RCOUNT and RPLAN
reference stable package hashes instead of duplicating context or lineage
semantics.

## Deliverables

- [ ] RCOUNT fixture references RCTX and RHIST fixture hashes.
- [ ] RPLAN note or fixture uses the same base context where possible.
- [ ] Consumer tests prove references are preserved and verified at the claim
  boundary.
- [ ] Active goal updated with final acceptance deltas.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```

