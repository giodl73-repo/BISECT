---
title: Structure validity remediation
status: complete
date: 2026-08-09
---

# Pulse 19 - Structure Validity Remediation

## Trigger

The frozen Wisconsin proof slice completed all four executions but produced two
invalid plans. Ratio-optimal-area disconnected one district. Prime-factor
disconnected a district and exceeded the advertised 0.5 percent population
tolerance, while its native manifest incorrectly reported population balance
as valid.

## Root causes and changes

1. ApportionRegions used a hidden 3 percent final validation tolerance while
   the manifest and audit advertised 0.5 percent. Final runner validation now
   uses the configured contract for every structure.
2. The manifest population flag was hard-coded true. It is now derived from
   the population check in the native audit certificate, with a regression test
   for an imbalanced plan.
3. AreaSection's asymmetric/multi-constraint METIS path now enables `Contig`
   and `MinConn` on the supported k-way routine.
4. ApportionRegions now uses the contiguity-capable k-way routine for two-way
   intermediate partitions rather than recursive METIS, whose contiguity option
   is not supported.

## Validation

- New population-reporting regression: pass.
- `bisect-apportion` library: 105 tests passed.
- `bisect-runner` library: 276 passed, one ignored.
- Rebuilt release binary.
- Post-remediation Wisconsin package: four of four native audits passed.
- Independent exact regeneration: pass.

The original 2-pass/2-fail package remains at
`docs/experiments/neutral-algorithm-family-bakeoff-wi-2020-pre-remediation/`.
The current four-pass package is
`docs/experiments/neutral-algorithm-family-bakeoff-wi-2020/`.

## Decision

The one-State implementation gate is repaired. This does not authorize a
national ranking: a national state schedule, failure policy, resource envelope,
and claim boundary must be frozen separately before expansion.
