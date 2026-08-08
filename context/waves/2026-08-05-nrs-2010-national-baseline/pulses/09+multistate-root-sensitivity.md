---
pulse: 09
title: Multi-State NRS v0.3 root sensitivity
status: active
wave: nrs-2010-national-baseline
validation_level: L2 governed empirical evidence
---

# Pulse 09 - Multi-State NRS v0.3 Root Sensitivity

## Purpose

Extend the exact sensitivity machinery beyond Rhode Island to a frozen
cross-State root sample capable of exercising more of the NRS refinement path.

## Deliverables

- [x] Audit all governed 2020 State roots.
- [x] Select an outcome-blind structural sample.
- [x] Replay each published benchmark seed exactly.
- [x] Precommit derivation, metrics, aggregation, failures, and claims.
- [ ] Execute 300 State/index pairs.
- [ ] Preserve compact assignments and every failure.
- [ ] Exactly regenerate every State package.
- [ ] Publish the result and remaining national-node gate.

## Frozen Sample

- New Hampshire: equal `1:1`, 31,948 blocks.
- New Mexico: unequal `1:2`, 107,215 blocks.
- Georgia: equal `7:7`, 232,717 blocks.

All three roots recorded connected-subtree population-improvement operations
in the governed benchmark. Selection used existing metadata only.

## Governing Boundary

Results remain root-specific and State-specific. The package may report
State-weighted and block-weighted assignment agreement separately, but no
single national robustness score.
