---
wave: ropt-core-pareto-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# ROPT Core Pareto Kernel

## Mission

Extract generic deterministic Pareto dominance, non-dominated sorting, and
crowding-distance utilities from `bisect-pareto` into `ropt-core`.

## Claim Boundary

This wave may add optimizer kernels over abstract minimization objective vectors.
It must not own redistricting objective definitions, plan validity, mutation,
crossover, audit packages, or NSGA-II orchestration.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Pareto dominance helpers | DONE | `ropt-core` dominance/sort/crowding helpers; `bisect-pareto` consumer; L0/L1/L2 tests |

## Close Summary

`ropt-core` now owns generic minimization-objective Pareto dominance,
fast non-dominated sorting, and crowding distance with typed validation errors.
`bisect-pareto` keeps redistricting `Objectives`, NSGA-II orchestration,
mutation, crossover, and audit semantics, while its dominance module delegates to
the reusable optimizer kernel.

## Carry-forwards

- Deterministic seed streams remain in `bisect-pareto` until another optimizer
  consumer needs the same derivation shape.
- Simulated annealing schedules, local-search transcripts, and repair/reject
  records remain future `ropt-core` candidates if duplication appears.
