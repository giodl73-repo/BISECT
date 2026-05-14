---
wave: ropt-seed-consumer-expansion
date_open: 2026-05-14
status: complete
source_goal: docs/specs/2026-05-13-shared-math-graph-kernels.md
---

# ROPT Seed Consumer Expansion

## Mission

Route the existing SHA-256 domain-separated seed helpers in SMC, multiscale MCMC,
ensemble chains, and parallel tempering through `ropt-core::derive_seed`.

## Claim Boundary

This wave may reuse the generic seed derivation kernel across existing consumers.
Each domain crate keeps its own seed prefixes, version-lock comments, RNG usage,
and algorithm semantics.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Seed consumer expansion | DONE | SMC, multiscale, ensemble-chain, and PT seed helpers use `ropt-core::derive_seed` |

## Close Summary

The generic seed derivation kernel now backs deterministic seed helpers in
`bisect-smc`, `bisect-multiscale`, `bisect-ensemble::chain`, and
`bisect-ensemble::parallel_tempering`. Each consumer keeps its own domain prefix,
version-lock tests, and RNG semantics; `ropt-core` only owns the reusable byte
encoding and SHA-256-to-`u64` derivation.

## Boundary Discipline

This remains an algorithm-coverage asset: the shared helper exists to keep the
same deterministic seed formula tested once and reused consistently. Domain
crates still decide when seeds are drawn and what their prefixes mean.
