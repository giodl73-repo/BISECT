---
pulse: 27
title: Capacity-admitted ensemble launch boundary
status: complete
wave: nrs-2010-national-baseline
validation_level: tested future-protocol launch control
---

# Pulse 27 - Capacity-Admitted Ensemble Launch Boundary

## Purpose

Make the Pulse 26 host-capacity contract executable at the process-creation
boundary for future block-ensemble protocols.

## Result

- [x] Reuse the actual package-volume measurement and retained-custody ledger.
- [x] Write a new admission record before process creation.
- [x] Refuse process creation when capacity is rejected.
- [x] Refuse to overwrite an existing admission record.
- [x] Preserve the child process return code after an authorized launch.
- [x] Test rejection, pre-launch record custody, and fail-closed overwrite cases.

Future protocols can invoke
`scripts/research/launch_block_ensemble_admitted.py` with their separately
frozen runner command. The adapter uses argument-vector process creation, not
a shell, and returns without launching when the contemporaneous admission
record rejects capacity.

## Claim Boundary

This pulse launched no governed chain and does not authorize a replacement for
Pulse 25. Admission establishes only filesystem headroom at the launch
boundary. A future protocol must still freeze and enforce its own inputs,
schedule, wall, memory, scratch, retained-evidence, analysis, replay, and
stopping rules.
