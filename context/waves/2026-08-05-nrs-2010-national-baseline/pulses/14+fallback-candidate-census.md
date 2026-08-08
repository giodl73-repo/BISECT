---
pulse: 14
title: Fallback candidate census
status: active
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
- [ ] Add behavior-preserving candidate diagnostics.
- [ ] Add synthetic selection and emitted-method tests.
- [ ] Replay the seven containing State packages with one worker.
- [ ] Verify governed assignments/objectives and publish the stop/advance gate.

## Governing Boundary

The v0.3 metrics cover candidates actually compared by the current
first-feasible traversal, not every constructible bridge-aware candidate.
