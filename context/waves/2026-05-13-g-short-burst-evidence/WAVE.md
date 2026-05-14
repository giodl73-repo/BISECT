---
wave: g-short-burst-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-g-short-burst-evidence-goal.md
---

# G Short-Burst Evidence

## Mission

Add package-backed evidence for G.6/G.12 short-burst seed derivation, endpoint
retention, selected-endpoint ordering, and acceptance-rate diagnostics.

## Claim Boundary

This wave validates synthetic mechanics fixtures. It does not claim production
CLI availability, NC/WI/TX empirical compactness gains, stationarity, or
distributional correctness.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - short-burst smoke package | DONE | Added hash-bound fixture, verifier package, positive replay, tamper tests, G.6/G.12 paper updates, and rebuilt PDFs |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_BUILD_JOBS='1'
cargo fmt
cargo test -p bisect-ensemble g_short_burst_smoke -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered a minimal G.6/G.12 short-burst evidence package
and updated the papers/docs boundary so the smoke fixture is not overstated as
production CLI or empirical proof.
