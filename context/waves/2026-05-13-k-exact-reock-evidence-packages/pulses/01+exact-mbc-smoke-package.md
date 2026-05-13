---
wave: k-exact-reock-evidence-packages
pulse: 01
status: done
depends_on: []
governing_roles:
  - DATUM
  - BENCHMARK
  - LEDGER
---

# Pulse 01 - Exact-MBC Smoke Package

## Mission

Add a deterministic exact polygon-MBC Reock reference package for simple shapes.

## Pre-implementation Scout

```powershell
rg -n "Reock|minimum bounding|MBC|proxy|exact" docs research\tracks crates context\waves
git --no-pager status --short
```

## Deliverables

- [x] Add exact-MBC helper in `bisect-analysis`.
- [x] Keep production `reock()` proxy unchanged.
- [x] Add hash-bound exact-MBC smoke fixtures.
- [x] Add verifier coverage for fixture values and proxy/exact distinction.
- [x] Run validation and commit.

## Result

The exact-MBC reference path now validates square and right-triangle fixtures.
The triangle fixture demonstrates why exact-MBC Reock and the centroid-radius
proxy must be described separately.
