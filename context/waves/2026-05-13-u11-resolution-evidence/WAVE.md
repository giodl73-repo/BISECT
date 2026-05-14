---
wave: u11-resolution-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-u11-resolution-evidence-goal.md
---

# U.11 Resolution Evidence

## Mission

Add package-backed evidence for U.11 resolution manifest fields, GEOID-prefix
fine-to-coarse mapping, population aggregation, and derived coarse adjacency.

## Claim Boundary

This wave validates a synthetic mapping/topology fixture. It does not claim the
Texas autocorrelation result, BG-level precision gains, production legal
adequacy, or all-state resolution behavior.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - resolution mapping smoke package | DONE | Added hash-bound fixture, verifier package, positive replay, tamper tests, U.11 paper update, and rebuilt PDF |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_BUILD_JOBS='1'
cargo fmt
cargo test -p bisect-multiscale resolution_evidence -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered a minimal U.11 resolution evidence package and
updated the paper/docs boundary so the smoke fixture is not overstated as TX
autocorrelation or legal proof.
