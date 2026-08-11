---
pulse: 29
title: Block-ensemble successor protocol
status: complete
wave: nrs-2010-national-baseline
validation_level: frozen protocol only; no execution authorized
---

# Pulse 29 - Block-Ensemble Successor Protocol

## Purpose

Define a scientifically separate path back to the incomplete NH/NM/GA matrix
without reopening, retrying, or counting evidence from failed Pulse 25.

## Result

- [x] Freeze `nrs-v0.3-block-ensemble-expansion-v2` before v2 processes.
- [x] Preserve v1 as an immutable failed package.
- [x] Select a fresh date-derived seed before v2 execution.
- [x] Require a fresh package and all 24 primary chains plus six exact replays.
- [x] Require actual-volume admission before every process.
- [x] Preserve the 21-hour, 2.25 GiB/process, and 3 GiB storage ceilings.
- [x] Require dedicated runner, verifier, custody, and Stage 0 review before
      execution.

## Next Gate

Implement and test the dedicated v2 runner and verifier. No preflight or
governed chain is authorized by this pulse alone.

## Claim Boundary

This pulse adds no ensemble samples or empirical result. It freezes a successor
design that cannot reuse v1 completions and cannot start until its implementation
and Stage 0 gates pass.
