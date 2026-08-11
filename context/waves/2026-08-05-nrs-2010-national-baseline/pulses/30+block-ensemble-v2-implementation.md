---
pulse: 30
title: Block-ensemble v2 implementation gate
status: complete
wave: nrs-2010-national-baseline
validation_level: implementation contract passed; no process launched
---

# Pulse 30 - Block-Ensemble v2 Implementation Gate

## Purpose

Implement the dedicated v2 runner, ledger, capacity boundary, and verifier
required by Pulse 29 without starting Stage 0.

## Result

- [x] Add a strict v2 ledger that rejects v1 identity and completions.
- [x] Freeze seed, parameters, order, phase prerequisites, and package path.
- [x] Place the resource monitor inside mandatory capacity-admitted launch.
- [x] Preserve rejected admission attempts while allowing a fresh recheck.
- [x] Bind successful processes to admission, wrapper, runner, executable, and
      protocol hashes.
- [x] Add a verifier for empty and future populated v2 custody.
- [x] Pass 24 focused old/new/admission tests and the empty-package verifier.
- [x] Pass the `simulate-contract` implementation audit after remediating one
      blocking admission-custody mismatch.

## Next Gate

Reverify the three input audits, build and bind the release executable, review
the implementation package, and only then decide whether Stage 0 preflight is
authorized.

## Claim Boundary

No v2 process ran. This pulse proves only implementation-contract conformance
for an empty active package. It adds no ensemble evidence and does not authorize
preflight or governed execution.
