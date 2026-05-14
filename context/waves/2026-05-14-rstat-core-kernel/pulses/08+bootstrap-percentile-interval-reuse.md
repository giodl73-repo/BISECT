---
wave: rstat-core-kernel
pulse: 08
status: done
depends_on: [07]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 08 - Bootstrap Percentile Interval Reuse

## Completion Notes

- Removed the local cluster-bootstrap percentile closure from
  `bisect-analysis::bloc_voting`.
- Reused `rstat-core::summary::percentile_interval_sorted_copy` for cluster and
  naive bootstrap coefficient intervals.
- Preserved the previous empty-vector behavior by mapping interval errors to
  `(NaN, NaN)` at the BISECT consumer boundary.
- Kept percentile semantics on the tested deterministic R-7 interpolation path.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis bloc_voting -- --test-threads=1
git diff --check
```
