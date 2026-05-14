---
wave: m1-lodes-economic-evidence
date_open: 2026-05-13
status: complete
source_goal: docs/specs/2026-05-13-m1-lodes-economic-evidence-goal.md
---

# M.1 LODES Economic Evidence

## Mission

Add package-backed evidence for M.1/M.9 economic-character formula and
edge-weight mechanics.

## Claim Boundary

This wave validates synthetic LODES WAC formula replay and edge-weight blending.
It does not claim full 50-state empirical results or within-district JPR
variance completion.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - LODES economic smoke package | DONE | Added reusable derivation, verifier package, tamper test, M.1 paper update, and rebuilt PDF |

## Validation Gate

```powershell
$env:CARGO_INCREMENTAL='0'
$env:CARGO_BUILD_JOBS='1'
cargo fmt
cargo test -p bisect-cli m1_economic_smoke -- --test-threads=1
cargo test -p bisect-cli derive_economic_character -- --test-threads=1
git diff --check
```

## Closeout

Completed. The wave delivered a hash-bound economic-character smoke package and
updated M.1 claim boundaries.
