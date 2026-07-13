---
pulse: 01
title: Exact objective and certificate schema
status: done
wave: exact-canonical-benchmark-foundations
validation_level: L1 synthetic exact
---

# Pulse 01 - Exact Objective And Certificate Schema

## Purpose

Implement a complete, inspectable exact slice for bounded `k=2` instances.

## Deliverables

- [x] Canonical exact-instance schema with source/model hash.
- [x] Four-level lexicographic objective.
- [x] Exhaustive bounded solver with canonical label/tie handling.
- [x] Optimal feasible-assignment certificate.
- [x] Exact infeasibility certificate.
- [x] Submission-independent reference verifier that rechecks the bounded proof.
- [x] Positive and negative tests.
- [x] North-Star implementation-status update.

## Claim Boundary

Exhaustive enumeration is limited to small synthetic `k=2` instances. It is a
reference oracle and certificate contract, not the production national solver.

## Validation

```powershell
cargo test -p bisect-ilp canonical -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```

Result: 8 exact-canonical tests passed. Formatting and diff checks passed.

## Evidence

- `crates/bisect-ilp/src/canonical.rs`
- `context/waves/2026-07-10-exact-canonical-benchmark-foundations/panels/pulse-01-exact-certificate-review.md`
- `docs/specs/2026-07-10-exact-canonical-benchmark-north-star.md`
