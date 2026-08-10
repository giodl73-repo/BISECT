---
title: Neutral algorithm-family proof slice
status: complete
date: 2026-08-09
---

# Pulse 18 - Neutral Algorithm-Family Proof Slice

## Objective

Replace reliance on the legacy incomplete 50-State GeoSection sweep with a
preregistered common-input proof slice before considering a national
algorithm-family comparison.

## Frozen design

The protocol fixes Wisconsin's 2020 congressional tract graph, eight districts,
total population, the `edge-weighted` preset, geographic weights, single outer
search, and requested seed 0. It runs `standard-bisect`, `ratio-optimal`,
`ratio-optimal-area`, and `prime-factor`. The wrapper binds the explicit command
because the native standard-bisection manifest retains the preset label rather
than independently recording the structure override.

## Result

All four executions completed on the same 1,542-tract adjacency universe and
produced eight assignments. Standard bisection passed after the native balance
retry advanced requested seed 0 to final seed 2. Ratio-optimal passed at seed 0.
Ratio-optimal-area failed the native contiguity audit. Prime-factor failed both
the 0.5 percent population check and contiguity; its native manifest nevertheless
set `population_balance_valid` to true, contradicting the audit certificate.

Canonical assignments were byte-identical on regeneration. Native weighted
edge-cut totals varied only below one-billionth because of parallel
floating-point reduction and are normalized to six decimals in deterministic
derived outputs. The independent package verifier passed with experiment status
`FAIL`.

## Decision

Do not expand the structure comparison nationally. Treat the two failing rows
as remediation witnesses, not as eligible performance competitors. The old B.0
estimated/pending cells remain unfilled because their metric and input contracts
differ from this proof slice.

## Artifacts

- `docs/specs/2026-08-09-neutral-algorithm-family-bakeoff-protocol.md`
- `scripts/research/run_neutral_algorithm_family_bakeoff.py`
- `scripts/research/analyze_neutral_algorithm_family_bakeoff.py`
- `scripts/research/verify_neutral_algorithm_family_bakeoff.py`
- `tests/unit/test_neutral_algorithm_family_bakeoff.py`
- `docs/experiments/neutral-algorithm-family-bakeoff-wi-2020/`
