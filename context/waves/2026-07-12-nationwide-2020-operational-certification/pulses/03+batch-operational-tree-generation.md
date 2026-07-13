---
pulse: 03
title: Batch operational tree generation
status: in_progress
depends_on: 02
wave: nationwide-2020-operational-certification
validation_level: L2 nationwide recursive execution
---

# Pulse 03 - Batch Operational Tree Generation

Generate deterministic operational recursive trees for every multi-district
State and reuse the completed one-district packages.

## Deliverables

- [x] resumable State/tree batch runner;
- [x] node-specific seed screening and retry policy;
- [ ] 50 operational State packages;
- [ ] 435 one-seat leaves;
- [ ] per-node arithmetic population proof status;
- [x] failure and retry ledger.

## Progress

The ledger contains 24 verified multi-district State packages and five open
failures (Arizona, Colorado, Oklahoma, Utah, and Washington). Virginia is the first
package completed under deterministic two-phase seed screening: METIS-only
screens are ranked before population refinement, each screen has a recorded
180-second operational timeout, completed nodes and screens are reusable, and
timeouts remain distinct from infeasibility.

Virginia produces 11 connected one-seat leaves. Every recursive node reaches
its ratio-arithmetic population floor. At node `10`, eight screens timed out;
seed 14 subsequently reached the arithmetic floor, demonstrating why bounded
screening and retry evidence are both required.

Kansas then completed four connected leaves with arithmetic-floor deviation
zero at all three recursive nodes. The run exposed and repaired a scratch-space
scaling defect: screening now retains only the discovery required for ranking
and deterministic resume rather than duplicating full split instances.

Iowa completed four connected leaves with all three recursive nodes at their
arithmetic floors and no screen timeouts.

Tennessee completed nine connected leaves after bounded root and five-seat-node
screen timeouts. All eight recursive nodes reached their arithmetic floors,
including the resumed final three- and two-seat nodes.

Oklahoma reached arithmetic floor 1 at its root and two-seat child. Its
remaining three-seat child exhausted all 15 completed screens and population
refinements after one screen timeout; best seed 8 reached deviation 20 against
floor 1. This is preserved as an unresolved local-search frontier, not a proof
of infeasibility. Future failed runs now write their full seed-screening and
best-objective evidence before exiting.

Next untouched State: Alabama.
