---
pulse: 02
title: Compact connectivity encoding
status: done
depends_on: 01
wave: scalable-certified-split-solver
validation_level: proof model
---

# Pulse 02 - Compact Connectivity Encoding

Replace exhaustive connectivity no-goods with a polynomial-size,
proof-loggable formulation.

## Deliverables

- [x] Add one root variable per child.
- [x] Add one parent arc for every assigned non-root unit.
- [x] Restrict parent arcs to same-child graph edges.
- [x] Add binary depth variables and strict acyclicity constraints.
- [x] Preserve odd/even orientation and exact objective semantics.
- [x] Remove the 24-unit/exhaustive classifier from compact compilation.
- [x] Generate a compact path-8 boundary request.
- [x] Generate a RoundingSat proof and verify it with VeriPB.
- [x] Quantify Rhode Island model size.

## Result

The compact encoding is sound and complete for connected child subgraphs.
Rhode Island projects to 1,177,222 Boolean variables and 1,263,773 base
constraints before objective-stage additions.

## Validation

```powershell
cargo test -p bisect-ilp proof_backend -- --test-threads=1
python scripts/research/verify_proof_toolchain_smoke.py
python scripts/research/verify_certified_proof_backend.py
cargo fmt --all -- --check
git --no-pager diff --check
```
