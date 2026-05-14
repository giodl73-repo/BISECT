---
wave: m3-acs-housing-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-m3-acs-housing-evidence-goal.md
---

# M.3 ACS Housing Evidence

## Mission

Turn M.3 housing-character weights from deferred design language into an
implemented, package-backed ACS housing formula and edge-weight slice.

## Claim Boundary

This wave validates formula mechanics and the graph edge-weight blend. It does
not claim North Carolina outcome improvements, all-state empirical results, or
court-ready community preservation metrics.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - ACS housing smoke package | DONE | Added housing loader/weighter, corrected ACS variables, verifier package, M.3 paper update, and rebuilt PDF |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
cargo fmt
cargo test -p bisect-cli m3_housing_smoke -- --test-threads=1
cargo test -p bisect-cli housing_character -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered an implemented `housing-character` weight mode,
hash-bound ACS housing smoke evidence, and updated M.3 documentation with future
empirical boundaries.
