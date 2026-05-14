---
wave: rstat-core-kernel
pulse: 04
status: done
depends_on: [03]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 04 - Bootstrap Resampling Kernel

## Completion Notes

- Added `rstat-core::resampling`.
- Implemented seeded bootstrap statistic generation and percentile intervals.
- Added typed errors for empty samples, zero replicates, and non-finite
  statistics.
- Wired `bisect-analysis::partisan::bootstrap_ci` through the shared resampling
  helper while preserving the domain-specific metric functions.
- Extended the L0/L1/L2 test ladder with bootstrap coverage.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis partisan -- --test-threads=1
git diff --check
```

