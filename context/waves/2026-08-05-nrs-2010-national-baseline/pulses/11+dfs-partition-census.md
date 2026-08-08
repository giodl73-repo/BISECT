---
pulse: 11
title: Initial DFS partition census
status: active
wave: nrs-2010-national-baseline
validation_level: L2 governed diagnostic evidence
---

# Pulse 11 - Initial DFS Partition Census

## Purpose

Separate genuinely different initial DFS cuts from complementary left/right
orientations of the same physical bipartition before spending additional seed
runs.

## Deliverables

- [x] Audit the 44-State oriented tie pattern.
- [x] Precommit the unlabeled partition definition and stop rule.
- [ ] Add behavior-preserving distinct-partition instrumentation.
- [ ] Add synthetic orientation and multi-partition tests.
- [ ] Replay and exactly verify all 44 governed roots.
- [ ] Publish the corrected sensitivity gate.

## Governing Boundary

The new count collapses labels only for the initial root-0 DFS tree-edge cut.
It does not establish label, child-node, fallback, or full-plan invariance.
