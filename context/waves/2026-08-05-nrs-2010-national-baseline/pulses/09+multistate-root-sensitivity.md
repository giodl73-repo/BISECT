---
pulse: 09
title: Multi-State NRS v0.3 root sensitivity
status: complete
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
- [x] Execute 300 State/index pairs.
- [x] Preserve compact assignments and every failure.
- [x] Exactly regenerate every State package.
- [x] Publish the result and remaining national-node gate.

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

## Result

All 300 State/index pairs accepted and passed their frozen population bounds.
Each State produced one unique assignment and reproduced its governed
benchmark 100 times. Assignment and objective distributions were invariant in
New Hampshire, New Mexico, and Georgia.

The result remains mechanism-specific. These roots exercised
connected-subtree population improvement, but none exercised a seed-sensitive
tie among deterministic DFS candidates with equal population deviation and
weighted cut. Blind expansion to thousands of runs is not the cheapest next
test. The next gate is source-level candidate-multiplicity instrumentation,
followed by a precommitted node sample that actually contains such ties if any
exist.

The accepted package is
`docs/experiments/nrs-v0.3-multistate-root-sensitivity-2020/`.

## Validation

```powershell
python -m pytest tests/unit/test_nrs_multistate_root_sensitivity.py -q
python scripts/research/verify_nrs_multistate_root_sensitivity.py `
  docs/experiments/nrs-v0.3-multistate-root-sensitivity-2020
git --no-pager diff --check
```
