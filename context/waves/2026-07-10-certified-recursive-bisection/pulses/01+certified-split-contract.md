---
pulse: 01
title: Certified split contract
status: done
depends_on: none
wave: certified-recursive-bisection
validation_level: L1 schema
---

# Pulse 01 - Certified Split Contract

Freeze the exact per-node model for a canonical recursive bisection cut.

## Deliverables

- [x] Define split instance and certificate schemas.
- [x] Bind parent certificate, unit universe, and `k_left:k_right`.
- [x] Define seat-ratio-scaled population deviation.
- [x] Define canonical left/right orientation and assignment ordering.
- [x] Specify optimal and infeasible results.
- [x] Add synthetic objective/orientation/connectivity tests.
- [x] Update exact format and North-Star documentation.

## Claim Boundary

This pulse defines one split. Recursive certificate chaining and scalable
solving remain later pulses.

## Validation

```powershell
cargo test -p bisect-ilp certified_split -- --test-threads=1
cargo test -p bisect-core bisection -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```
