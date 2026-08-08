---
pulse: 11
title: Initial DFS partition census
status: complete
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
- [x] Add behavior-preserving distinct-partition instrumentation.
- [x] Add synthetic orientation and multi-partition tests.
- [x] Replay and exactly verify all 44 governed roots.
- [x] Publish the corrected sensitivity gate.

## Result

All 44 roots preserved their governed assignments and objectives. Every root
had exactly one distinct unlabeled minimum-deviation/minimum-cut bipartition.
The 29 roots with two oriented candidates were exactly the equal-child-seat
roots, and both candidates collapsed to the same physical cut.

The preregistered physical-cut expansion gate is therefore closed for the
initial root-0 DFS stage. Further sensitivity work requires a separate
label-orientation, child-node, or fallback thesis.

## Governing Boundary

The new count collapses labels only for the initial root-0 DFS tree-edge cut.
It does not establish label, child-node, fallback, or full-plan invariance.
