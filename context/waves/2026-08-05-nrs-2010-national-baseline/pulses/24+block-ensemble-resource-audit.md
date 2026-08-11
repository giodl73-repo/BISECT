---
pulse: 24
title: Block-ensemble resource audit
status: complete
wave: nrs-2010-national-baseline
validation_level: hash-bound measured resource replay
---

# Pulse 24 - Block-Ensemble Resource Audit

## Purpose

Close the instrumented peak-memory and explicit-budget prerequisites without
reusing resource replays as statistical evidence or prematurely authorizing a
multi-State run.

## Current State

- [x] Freeze `nrs-v0.3-block-ensemble-resource-audit-v1` before measurement.
- [x] Implement and test the cross-platform resource wrapper.
- [x] Execute full Wilson then Kruskal resource replays sequentially.
- [x] Verify exact normalized trace reproduction and resource evidence.
- [x] Apply the frozen formulas to decide whether an NH/NM/GA protocol may be
  drafted.

## Result

Both resource replays regenerated the committed Stage 1 traces exactly after
runtime normalization. The author-machine OS peak was `175,448,064` bytes and
combined runner wall time was `2535.372` seconds. The frozen formulas authorize
21 runner-hours, 2.25 GiB per process, and 3 GiB each of retained and scratch
storage. All hard ceilings pass, so Pulse 25 may draft—but not yet execute—the
precommitted NH/NM/GA expansion.

## Claim Boundary

Resource evidence supports author-machine planning only. It does not add
governed draws, prove portable performance, or authorize multi-State execution.
