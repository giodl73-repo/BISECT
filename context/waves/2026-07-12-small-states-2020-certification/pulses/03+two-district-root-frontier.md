---
pulse: 03
title: Two-district root frontier
status: done
depends_on: 02
wave: small-states-2020-certification
validation_level: L2 real block data and external proof
---

# Pulse 03 - Two-District Root Frontier

Select the smallest 2020 two-district States, build complete root packages, and
measure exact population/boundary/canonical proof coverage.

## Deliverables

- [x] rank two-district States by block and graph size;
- [x] build connected block RCTX for Hawaii and New Hampshire;
- [x] generate deterministic connected incumbents with shared contiguity repair;
- [x] prove population optimum for both States;
- [x] attempt boundary proof stages with fixed 120-second budgets;
- [x] publish complete operational RPLAN/RCTX/audit packages; and
- [x] preserve solver timeouts and proof gaps.

## Result

Hawaii and New Hampshire both reach scaled population deviation 1 and have
RoundingSat population proofs accepted by VeriPB. Their operational plans pass
contiguity audit. Boundary decisions time out at 120 seconds; canonical proof is
therefore not attempted.

## Claim Boundary

Operational packages prove wall-to-wall assignment and connectivity. Exact
objective claims advance only with independently checked proofs.
