---
wave: u5-adaptive-multiscale-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-u5-adaptive-multiscale-evidence-goal.md
---

# U.5 Adaptive Multiscale Evidence

## Mission

Add package-backed evidence for U.5 Robbins-Monro alpha-trace replay, clipping,
coarse tolerance, and `MSC_STEP_` seed derivation.

## Claim Boundary

This wave validates synthetic alpha-trace mechanics. It does not claim NC/TX/IA
state convergence, autocorrelation improvement, production CLI availability, or
legal adequacy.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - adaptive alpha smoke package | DONE | Added hash-bound fixture, verifier package, positive replay, tamper tests, U.5 paper update, and rebuilt PDF |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_BUILD_JOBS='1'
cargo fmt
cargo test -p bisect-multiscale adaptive_evidence -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered a minimal U.5 adaptive multiscale evidence
package and updated the paper/docs boundary so the smoke fixture is not
overstated as state convergence or CLI proof.
