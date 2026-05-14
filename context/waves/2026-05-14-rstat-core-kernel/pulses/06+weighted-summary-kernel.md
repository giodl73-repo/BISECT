---
wave: rstat-core-kernel
pulse: 06
status: done
depends_on: [05]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 06 - Weighted Summary Kernel

## Completion Notes

- Added weighted mean, weighted population variance, weighted population standard
  deviation, min, max, and total-weight reporting to `rstat-core::summary`.
- Preserved typed input errors for length mismatch, non-finite/negative weights,
  and zero total weight.
- Wired `bisect-analysis::bloc_voting` weighted standardization through the
  shared helper.
- Wired `bisect-analysis::compactness::population_weighted_compactness` through
  the shared weighted mean helper while preserving its zero-population behavior.
- Extended L0/L1/L2 coverage for weighted summaries.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis bloc_voting -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis compactness -- --test-threads=1
git diff --check
```
