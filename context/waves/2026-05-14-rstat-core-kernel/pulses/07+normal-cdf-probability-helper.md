---
wave: rstat-core-kernel
pulse: 07
status: done
depends_on: [06]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 07 - Normal CDF Probability Helper

## Completion Notes

- Added `rstat-core::probability::standard_normal_cdf` using the existing
  Abramowitz-Stegun approximation.
- Removed the duplicate private Normal CDF and `erf` approximation from
  `bisect-analysis::bloc_voting`.
- Preserved the bloc-voting HC3 two-sided p-value formula and approximation
  boundary.
- Extended L0/L1/L2 coverage for known Normal quantiles, p-value composition,
  and grid monotonicity.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis bloc_voting -- --test-threads=1
git diff --check
```
