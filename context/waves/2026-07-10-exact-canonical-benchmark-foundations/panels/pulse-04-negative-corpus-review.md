# Pulse 04 Negative Corpus Review

**Date:** 2026-07-10  
**Roles:** MERIDIAN, COVENANT  
**Posture:** Bounded E0 adversarial artifact verification

## Implemented

- Five committed invalid certificate/proof submissions.
- Machine-readable expected rejection classes.
- Reproducible corpus builder and hash custody.
- Public command-line reference verifier.
- Integration test loading all committed hostile artifacts.

## Review Disposition

The corpus distinguishes syntactically self-consistent false claims from
simple hash corruption. Recomputed certificate IDs prevent false-optimum,
false-infeasibility, tie, and connectivity cases from passing merely because
their outer hash is stale. Each therefore reaches the exhaustive semantic
comparison and is rejected as a result mismatch.

No Pulse 04 blocking defects remain.

## Carry-Forward

Pulse 05 must implement the verifier independently of the Rust enumeration and
accept the positive corpus while rejecting these same five submissions.
