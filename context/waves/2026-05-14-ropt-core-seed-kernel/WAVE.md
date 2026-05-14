---
wave: ropt-core-seed-kernel
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# ROPT Core Seed Kernel

## Mission

Extract deterministic domain-separated seed derivation from `bisect-pareto` into
`ropt-core`.

## Claim Boundary

This wave may add generic SHA-256 seed derivation over explicit domain prefixes
and typed seed parts. It must not own Pareto-specific operation names, NSGA-II
orchestration, random sampling policy, mutation, crossover, or redistricting
plan semantics.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Domain-separated seed derivation | DONE | `ropt-core::derive_seed`; `bisect-pareto::seeds` wrappers; L0/L1/L2 tests |

## Close Summary

`ropt-core` now owns deterministic SHA-256 seed derivation over explicit byte
domains and typed seed parts. `bisect-pareto` keeps its Pareto-specific domain
labels and public seed wrappers, preserving historical seed behavior while moving
the reusable algorithmic kernel into the shared crate.

## Boundary Discipline

The shared kernel crates are reusable algorithm sets whose value comes from
explicit L0/L1/L2 coverage. Other projects can use them, but changes should stay
algorithm-driven: bug fixes, justified reuse, or coverage needed to protect a
domain algorithm.
