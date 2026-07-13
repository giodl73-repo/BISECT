# Pulse 03 Boundary Model Strengthening Review

**Date:** 2026-07-10  
**Roles:** MERIDIAN, SCALE, COVENANT  
**Posture:** Exact decomposition; unresolved proof search

## Review Disposition

At scaled population deviation 1 and odd total population, the right child must
contain exactly 548,689 or 548,690 people. Separate equality-constrained models
therefore cover the entire population-optimal feasible set without overlap.

Both branch models are hash-bound and independently checked. The first probes
returned no result; those attempts are historical because subsequent balanced
exchanges improved the incumbent and regenerated both branches.

No Pulse 03 model-correctness defect remains.

## Carry-Forward

Pulse 04 should tune and extend proof search independently on both exact
population branches and accept boundary optimality only if both are proved
UNSAT.
