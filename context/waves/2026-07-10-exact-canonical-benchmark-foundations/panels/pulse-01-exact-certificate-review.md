# Pulse 01 Exact Certificate Review

**Date:** 2026-07-10  
**Roles:** MERIDIAN, COVENANT  
**Posture:** Bounded synthetic exact reference; not production national exactness

## Implemented

- Versioned exact instance and certificate schemas.
- Four-level lexicographic objective:
  1. maximum scaled population deviation;
  2. total scaled population deviation;
  3. weighted boundary cut;
  4. canonical assignment vector.
- Label-symmetry removal by fixing the first canonically ordered unit to
  district 0.
- Exhaustive bounded `k=2`, `n <= 24` reference solver.
- Optimal feasible-assignment and exact infeasibility certificates.
- Canonical instance/certificate hashes.
- Submission-independent verifier that re-enumerates the instance.
- Numeric-range and canonical-unit-order validation.

## Tests

- Path-4 unique optimum.
- Cycle-4 primary-objective tie and lexicographic selection.
- Three-island infeasibility.
- False optimum rejection.
- False infeasibility rejection.
- Certificate-ID tamper rejection.
- Unsorted unit-order rejection.
- Numeric-overflow rejection.

## Review Disposition

The objective order, enumeration count, label symmetry, contiguity, proof
statistics, and hashes are correct within scope. The verifier shares the
reference enumeration implementation; a genuinely independent implementation
remains Pulse 05.

## Claim Boundary

This is an E0 reference oracle and certificate contract. It does not implement
general `k`, national blocks, branch-and-cut proof transcripts, production
solver integration, legal certification, or two-verifier readiness.

