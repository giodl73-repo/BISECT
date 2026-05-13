---
wave: r-package-completion
pulse: 06
status: done
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

- [x] RCOUNT fixture references RCTX and RHIST fixture hashes.
- [x] RPLAN note or fixture uses the same base context where possible.
- [x] Consumer tests prove references are preserved and verified at the claim
  boundary.
- [x] Active goal updated with final acceptance deltas.

## Completion Notes

- RCOUNT: `synthetic_summary_basic_package_with_base_references` binds the
  summary fixture to the RCTX L0 context/crosswalk hashes and RHIST L2 package
  hash.
- RPLAN: the regenerated district-aggregation example carries a
  `civic_evidence_base_references` extension documenting the shared RCTX/RHIST
  package hashes and the claim boundary that RPLAN owns assignments only.
- Consumer coverage: `rcount-core`, `rcount-io`, and `rcount-district` tests
  preserve and verify the references without copying RCTX graph/source or RHIST
  lineage semantics into RCOUNT/RPLAN.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rctx-core -p rcount-core -p rcount-district -p rcount-io -p rcount-audit -p rcount-cli
git diff --check
```
