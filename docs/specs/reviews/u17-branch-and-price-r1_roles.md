# Role Review: U.17 Branch-And-Price / Column Generation

**Spec:** [`2026-05-11-u17-branch-and-price.md`](../2026-05-11-u17-branch-and-price.md)  
**Status:** Approved for stage-one implementation

## Summary

U.17 is approved as a contract-first exact optimization slice. The initial
implementation must avoid claiming production branch-and-price performance while
still making pricing, master, bounds, gap, and lineage artifacts stable.

## Required Guardrails

- Keep formulation-only status distinct from solved exact status.
- Limit brute-force pricing and master solving to small fixtures.
- Emit `None` bounds and gap for formulation-only reports.
- For solved fixture reports, bounds must agree and gap must be zero.
- Build lineage with producer crate `bisect-column` and method
  `branch-and-price`.

## Stage-One Approval

Approved to add `bisect-column` with column, pricing, master, report, and small
exact fixture coverage before CLI/RPLAN wiring.
