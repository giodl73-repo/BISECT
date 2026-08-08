# Pulse 09 Multi-State Sensitivity Review

**Date:** 2026-08-07
**Roles:** MERIDIAN, DATUM, SCALE, BENCHMARK, BOUNDARY
**Posture:** protocol review before diagnostic-seed generation

## Findings

- MERIDIAN: compare the same governed root operation across States; do not
  substitute tract plans or full-plan metrics for root assignments.
- DATUM: freeze State selection from existing metadata, derive seeds
  independently per input manifest, and preserve all 300 pairs.
- SCALE: objective magnitudes are not commensurate across roots. Report each
  root separately and limit top-level aggregation to clearly named assignment
  estimands.
- BENCHMARK: exact replay of the published seed is a prerequisite, not a
  sensitivity result. Bind each State manifest and regenerate all pairs.
- BOUNDARY: even invariance across all three roots would not establish
  national robustness or ensemble convergence.

## Decision

Proceed with New Hampshire, New Mexico, and Georgia. Do not add or replace a
State after outcomes are observed.

## Post-Execution Disposition

All 300 pairs accepted and exactly reproduced their State benchmark. The three
roots exercised population-improvement operations but did not exercise the
seed-dependent moved-population tie-break. The panel accepts the package as a
complete negative sensitivity result and blocks blind national seed expansion
until candidate-tie multiplicity is observable.
