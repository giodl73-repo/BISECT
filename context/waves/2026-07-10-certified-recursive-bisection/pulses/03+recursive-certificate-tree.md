---
pulse: 03
title: Recursive certificate tree
status: done
depends_on: 01, 02
wave: certified-recursive-bisection
validation_level: L1 tree verifier
---

# Pulse 03 - Recursive Certificate Tree

Bind parent and child unit universes, split objectives, and one-seat leaves into
a complete canonical certificate tree.

## Deliverables

- [x] Use `bisect_core::BisectionTree` as the schedule authority.
- [x] Store every non-leaf split instance, certificate, and proof.
- [x] Reconstruct child populations, unit IDs, and induced edges.
- [x] Bind child nodes and leaves to parent certificate IDs.
- [x] Require each child to contain at least its assigned seat count in units.
- [x] Canonically order split nodes in BFS order.
- [x] Canonically index one-seat leaves by binary path.
- [x] Require leaves to partition the root unit universe exactly once.
- [x] Reject child-universe tampering and missing leaves.

## Claim Boundary

The tree certifies sequential locally optimal cuts. It does not establish that
every locally optimal parent cut admits downstream completion; construction
fails explicitly if a certified child cannot be split.

## Validation

```powershell
cargo test -p bisect-ilp certified_tree -- --test-threads=1
cargo test -p bisect-ilp certified_split -- --test-threads=1
cargo test -p bisect-core bisection -- --test-threads=1
cargo fmt --all -- --check
git --no-pager diff --check
```
