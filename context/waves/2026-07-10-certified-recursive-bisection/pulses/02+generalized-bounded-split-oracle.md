---
pulse: 02
title: Generalized bounded split oracle
status: done
depends_on: 01
wave: certified-recursive-bisection
validation_level: L1 exact oracle
---

# Pulse 02 - Generalized Bounded Split Oracle

Implement and independently verify bounded exact cuts for arbitrary canonical
seat ratios.

## Deliverables

- [x] Enumerate symmetry-reduced equal-seat assignments.
- [x] Enumerate both orientations for unequal-seat assignments.
- [x] Filter child connectivity before objective comparison.
- [x] Select the unique lexicographic optimum.
- [x] Emit exact infeasibility when no connected split exists.
- [x] Emit a deterministic candidate transcript commitment.
- [x] Bind proof identity into the split certificate.
- [x] Re-enumerate through a submission-independent verifier API.
- [x] Add false-optimum and transcript-tamper rejection tests.
- [x] Preserve the 24-unit bounded-oracle claim boundary.

## Claim Boundary

The verifier does not trust submitted artifacts but shares the Rust
enumeration implementation with the generator. Recursive tree verification,
an independent second implementation, CLI packaging, and scalable solving
remain later pulses.

## Validation

```powershell
cargo test -p bisect-ilp certified_split -- --test-threads=1
cargo test -p bisect-core bisection -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```
