# Pulse 02 Exact CLI Package Review

**Date:** 2026-07-10  
**Roles:** MERIDIAN, COVENANT  
**Posture:** Bounded E0 package integration

## Implemented

- `bisect exact --method canonical-exhaustive`.
- RCTX-to-exact-instance conversion with integer edge-weight enforcement.
- Optimal package:
  - exact instance and certificate;
  - RPLAN and RCTX;
  - RPLAN audit certificate; and
  - hash-bound exact package manifest.
- Infeasible package:
  - exact instance and infeasibility certificate; and
  - hash-bound package manifest without a fabricated plan.
- Fixed `--generated-at` for deterministic package artifacts.
- Positive path-4 and negative three-island fixture corpus.

## Review Disposition

Hash coverage, result-specific output shape, RPLAN/RCTX semantics, integer
weights, claim boundary, and certificate verification are correct.

The RPLAN audit uses the independently declared `--tolerance`; it is not derived
from the exact solution. The exact certificate remains the optimality evidence.

## Carry-Forward

- Stable proof transcript/verifier API remains Pulse 03.
- Independent second verifier remains Pulse 05.
- General `k`, production branch-and-cut, and real blocks remain future work.

