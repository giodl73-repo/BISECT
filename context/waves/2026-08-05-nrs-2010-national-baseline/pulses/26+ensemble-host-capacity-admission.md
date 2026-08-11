---
pulse: 26
title: Ensemble host-capacity admission
status: complete
wave: nrs-2010-national-baseline
validation_level: tested future-protocol admission control
---

# Pulse 26 - Ensemble Host-Capacity Admission

## Purpose

Convert the Pulse 25 host disk-exhaustion failure into a reusable pre-launch
control without reopening or retrying the closed expansion.

## Result

- [x] Preserve the Pulse 25 terminal failure and no-retry decision.
- [x] Define an admission formula on the actual evidence volume.
- [x] Reserve scratch, remaining retained custody, and 2 GiB safety headroom.
- [x] Implement a standalone checker that never auto-deletes data.
- [x] Test pass, rejection, exact shortfall, custody, and invalid-ledger cases.

Future governed protocols can call
`scripts/research/check_block_ensemble_host_capacity.py` immediately before
each process launch. A rejection precedes runner execution and therefore can
be remediated and checked again without becoming a seed retry.

## Claim Boundary

This pulse hardens resource admission only. It does not complete Pulse 25,
authorize a replacement expansion, or add ensemble evidence.
