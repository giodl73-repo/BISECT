---
pulse: 32
title: Block-ensemble v2 Stage 0
status: complete
wave: nrs-2010-national-baseline
validation_level: terminal integration failure retained; v2 gate failed
---

# Pulse 32 - Block-Ensemble v2 Stage 0

## Purpose

Execute the six excluded 25-step preflights and their fresh-process replays
before any governed v2 chain.

## Result

- [x] Reverify pristine v2 readiness and empty custody.
- [x] Confirm 95,795,449,856 free bytes before the Stage 0 decision.
- [x] Admit the first NH Wilson process with 95,793,344,512 free bytes.
- [x] Retain its nonzero exit, resource measurement, admission, and failed
      ledger state.
- [x] Add and pass a terminal-failure verifier over those linked records.
- [x] Stop without retry before any other process.
- [ ] Complete any Stage 0 preflight or replay.
- [ ] Start any governed v2 chain.

The v2 wrapper emitted its frozen execution class
`excluded-expansion-v2-preflight` and base seed `20260811`. The hash-bound Rust
runner accepted only the predecessor identities
`excluded-expansion-preflight`/`governed-stage2` and predecessor base seed
`20260810`, then returned `unsupported execution class` after 1.3138 seconds.

This gap escaped the readiness gate because Python tests asserted the emitted
arguments and custody bound the compiled executable, but no test exercised the
compiled runner's argument-validation contract using the v2 identity.

## Decision

Expansion v2 is terminally failed and closed. No retry, remaining preflight,
replay, or primary is authorized. A successor requires a new identity, package,
and seed plus a compiled runner-contract test before execution.

## Claim Boundary

This pulse supplies negative implementation-integration evidence only. It does
not supply a completed chain, sampler diagnostic, feasibility result, or
paper-facing Wilson-versus-Kruskal comparison.
