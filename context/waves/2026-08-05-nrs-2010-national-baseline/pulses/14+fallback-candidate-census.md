---
pulse: 14
title: Fallback candidate census
status: complete
wave: nrs-2010-national-baseline
validation_level: L2 governed diagnostic evidence
---

# Pulse 14 - Fallback Candidate Census

## Purpose

Measure the candidate tie surface inside the seven activated v0.2 nodes and
the single activated v0.3 stage before any fallback seed execution.

## Deliverables

- [x] Freeze both stage-specific candidate universes.
- [x] Freeze the eight activated stage/node pairs.
- [x] Add behavior-preserving candidate diagnostics.
- [x] Add synthetic selection and emitted-method tests.
- [x] Replay the six containing State packages with one worker.
- [x] Verify governed assignments/objectives and publish the stop/advance gate.

## Result

All six containing State assignments and all eight activated stage/node
assignments and objectives reproduced exactly. The seven v0.2 stages each
evaluated 16 candidates. Six had one minimum-deviation candidate; 2000
HI/root had 16, of which five also tied on cut. The 2000 HI/root v0.3 stage
evaluated three candidates, with two tied on deviation and cut.

Every tied finalist set collapsed to one unlabeled physical partition.
Therefore no governed fallback stage provides a physical-partition
sensitivity opportunity, and fallback seed expansion stops.

## Governing Boundary

The v0.3 metrics cover candidates actually compared by the current
first-feasible traversal, not every constructible bridge-aware candidate.
The stop result is limited to the eight activated governed stage/node pairs
and does not establish seed-invariant labels, plans, robustness, optimality,
partisan performance, or legal quality.
