---
wave: j-apportionment-evidence-packages
pulse: 01
status: done
depends_on: []
governing_roles:
  - DATUM
  - LEDGER
  - BENCHMARK
---

# Pulse 01 - 2020 Census Table 1 Verifier

## Mission

Add a hash-bound 2020 Census Table 1 apportionment fixture and verifier coverage
that recomputes the official 435-seat Huntington-Hill apportionment.

## Pre-implementation Scout

```powershell
rg -n "J.0|J.1|J.2|J.3|J.4|J.5|J.6|Census|SHA|apportionment fixture|bisect-apportion" docs research\tracks crates\bisect-apportion context\waves
git --no-pager status --short
```

## Deliverables

- [x] Add a package manifest with Census source URL and source XLSX SHA-256.
- [x] Add extracted 2020 Table 1 rows for the 50 states and official seats.
- [x] Add verifier coverage in `bisect-apportion`.
- [x] Update wave checklist and goal doc.
- [x] Run validation and commit.

## Scout Results

- `bisect-apportion` already had Huntington-Hill and general divisor methods,
  but no evidence-package verifier.
- The official 2020 Census Table 1 XLSX was downloaded from Census.gov and
  hashed as
  `93e7e77a222f078c0af32457af2ecc7bcae2bcb9db0cedca4ad93ff3f99b55bf`.
- The extracted fixture records 50 state apportionment populations, official
  representatives, change from 2010, and the 435-seat total.

## Implementation

- Added `bisect_apportion::evidence_manifest`.
- Added `docs/examples/j-apportionment-evidence-packages/2020-census-table01/`.
- Added tests for package hash binding, Huntington-Hill replay, and tampered
  official seat detection.

## Validation

```powershell
cargo fmt
cargo test -p bisect-apportion
git diff --check
```
