---
pulse: 12
title: Complete-tree DFS and fallback census
status: complete
wave: nrs-2010-national-baseline
validation_level: L2 governed diagnostic evidence
---

# Pulse 12 - Complete-Tree DFS And Fallback Census

## Purpose

Extend the root-only physical-cut result to every governed 2020 recursive
split node and directly observe fallback-stage activation.

## Deliverables

- [x] Trace child-node execution and fallback entry points.
- [x] Precommit the 385-node universe and decision rule.
- [x] Add behavior-preserving fallback activation diagnostics.
- [x] Add emitted-method tests.
- [x] Rebuild all 44 multi-district State trees.
- [x] Verify every governed State assignment and node objective.
- [x] Publish the complete-tree census and next gate.

## Result

All 44 State assignments and all 385 node assignments/objectives reproduced
exactly. Every split node had one distinct unlabeled
minimum-deviation/minimum-cut initial DFS partition. Two hundred sixty-six
nodes had complementary orientation ties, but no node had competing physical
initial cuts.

Neither the v0.2 alternate-root fallback nor the v0.3 bridge-aware fallback
activated at any governed 2020 node. The preregistered stop rule therefore
closes broader 2020 DFS/fallback seed expansion absent a new
mechanism-specific thesis.

## Governing Boundary

The census observes initial DFS partition multiplicity and fallback activation.
It does not yet count competing candidates inside an activated fallback stage.
