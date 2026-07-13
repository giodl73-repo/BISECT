---
pulse: 04
title: Three- and four-district trees
status: done
depends_on: 03
wave: small-states-2020-certification
validation_level: L2 real block data and recursive package verification
---

# Pulse 04 - Three- And Four-District Trees

Build operational recursive trees for the smallest three- and four-district
States and report node-level exact proof coverage.

## Deliverables

- [x] rank three- and four-district States by block count;
- [x] build connected block RCTX for Nevada and New Mexico;
- [x] generate every recursive node and one-seat leaf;
- [x] verify complete wall-to-wall leaf coverage;
- [x] bind arithmetic population proofs at every node;
- [x] publish proof/gap matrix and operational packages.

## Result

Nevada produces four connected leaves with populations 776,153/776,154.
New Mexico produces three connected leaves with populations 705,840/705,841.
Every split node reaches its ratio-scaled arithmetic population floor. Boundary
and canonical objectives remain unproved.

## Claim Boundary

Tree packages prove assignment coverage and connectivity. Boundary/canonical
optimality remains node-specific and requires independent proof.
