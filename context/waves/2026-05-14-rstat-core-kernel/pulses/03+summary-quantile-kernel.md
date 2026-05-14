---
wave: rstat-core-kernel
pulse: 03
status: done
depends_on: [01, 02]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 03 - Summary And Quantile Kernel

## Completion Notes

- Added `rstat-core::summary`.
- Implemented finite-sample validation, mean, population/sample variance,
  population/sample standard deviation, min/max, median, deterministic R-7
  quantiles, and percentile intervals.
- Wired `bisect-analysis::partisan` mean-median and bootstrap percentile CI
  helpers through `rstat-core`.
- Added explicit test ladder:
  - L0 inline unit tests in `rstat-core::summary`;
  - L1 integration tests in `crates/rstat-core/tests/l1_*`;
  - L2 ignored numeric stress tests in
    `crates/rstat-core/tests/l2_numeric_stress.rs`.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis partisan -- --test-threads=1
```

