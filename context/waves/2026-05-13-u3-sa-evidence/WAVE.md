---
wave: u3-sa-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-u3-sa-evidence-goal.md
---

# U.3 Simulated Annealing Evidence

## Mission

Add package-backed evidence for U.3 simulated-annealing seed derivation and
synthetic split mechanics.

## Claim Boundary

This wave validates a deterministic seed/grid smoke fixture. It does not claim
the NC/WI/TX empirical table, 50-state performance, global optimality, or
distributional sampling.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - SA seed/grid smoke package | DONE | Added hash-bound fixture, verifier package, positive replay, tamper rejection, U.3 paper update, and rebuilt PDF |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_BUILD_JOBS='1'
cargo fmt
cargo test -p bisect-cli u3_sa_smoke -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered a minimal U.3 SA evidence package and updated the
paper/docs boundary so the smoke fixture is not overstated as empirical proof.
