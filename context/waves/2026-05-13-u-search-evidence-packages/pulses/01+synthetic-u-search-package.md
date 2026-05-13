---
wave: u-search-evidence-packages
pulse: 01
status: in-progress
governing_roles:
  - DATUM
  - BENCHMARK
---

# Pulse 01 - Synthetic U.2/U.4 Search Package

## Mission

Add a hash-bound synthetic evidence package for U.2 and U.4 that verifies the
shape of archived parameter sweeps and parallel-tempering audit traces.

## Pre-implementation Scout

```powershell
rg -n "U\.2|U\.4|parameter|tempering|package|manifest" docs research\tracks\U-search-optimization crates\bisect-ensemble
git --no-pager status --short
```

## Deliverables

- [x] Decide scope: package/helper only, not production CLI.
- [x] Add `u-search-evidence-manifest v1` verifier types.
- [x] Add synthetic U.2/U.4 package under `docs/examples/`.
- [x] Add positive and negative verifier coverage.
- [x] Update manifest docs.

## Validation

```powershell
cargo fmt
cargo test -p bisect-ensemble search_evidence
git diff --check
```
