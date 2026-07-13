# Pulse 01 Certified Split Contract Review

**Date:** 2026-07-10  
**Roles:** MERIDIAN, COVENANT  
**Posture:** Per-node contract; no generalized solver yet

## Implemented

- Versioned certified split instance and certificate schemas.
- Exact `floor(k/2)` / `ceil(k/2)` schedule matching `BisectionTree`.
- Seat-ratio-scaled population objective.
- Equal-seat label-symmetry removal.
- Unequal-seat orientation that preserves valid ratio-correct cuts.
- Parent-certificate and unit-universe identities.
- Objective, connectivity, schedule, orientation, and identity tests.

## Review Disposition

The contract preserves California's `52 -> 26/26 -> 13/13 -> 6/7` schedule.
For equal-seat splits, fixing canonical unit 0 left removes only label
symmetry. For unequal-seat splits, no such restriction is imposed because it
could exclude the ratio-correct orientation.

The two population-deviation fields are algebraically redundant for a binary
split but remain for E0 schema continuity. Connectivity is explicitly a
feasibility predicate, not an objective side effect.

No Pulse 01 blocking defect remains.

## Carry-Forward

Pulse 02 must enumerate both orientations for unequal splits, filter
connectivity before scoring, and verify optimal/infeasible certificates against
this exact contract.
