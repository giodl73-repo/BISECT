---
pulse: 13
title: Cross-census complete-tree DFS census
status: complete
wave: nrs-2010-national-baseline
validation_level: L2 governed diagnostic evidence
---

# Pulse 13 - Cross-Census Complete-Tree DFS Census

## Purpose

Extend the 2020 complete-tree stop result to 2000 and 2010 while preserving
the known California and Hawaii amendment witnesses.

## Deliverables

- [x] Audit both governed 385-node universes.
- [x] Precommit the 770-node replay and decision rule.
- [x] Rebuild all 86 multi-district State trees.
- [x] Verify every State assignment and node objective.
- [x] Enumerate physical-cut opportunities and fallback activations.
- [x] Publish the stage-specific next gate.

## Result

All 86 State assignments and all 770 node assignments/objectives reproduced
exactly. Every node had one physical initial DFS partition; 525 nodes had
orientation-only ties.

Seven nodes activated the v0.2 alternate-root fallback:

- 2000: AZ/10, CA/11010, GA/100, HI/root, TX/100, and TX/0100.
- 2010: CA/00110.

Only 2000 HI/root proceeded to the v0.3 bridge-aware fallback. These seven
nodes form the complete governed fallback-instrumentation universe for the
next pulse.

## Governing Boundary

This pulse identifies activated stages; it does not yet measure candidate
multiplicity inside fallback search.
