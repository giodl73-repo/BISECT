---
wave: rgraph-core-expansion
pulse: 01
status: done
governing_roles:
  - BOUNDARY
  - BENCHMARK
  - LEDGER
---

# Pulse 01 - Connected Components Kernel

## Completion Notes

- Added deterministic connected-component helpers to `rgraph-core`.
- Supported all-node components, filtered components, restricted node subsets,
  and restricted subsets with edge filters.
- Wired `bisect-analysis::contiguity::bfs_component_count` through the restricted
  component helper while preserving its public return type.
- Added L0 inline tests, L1 integration coverage, and an ignored L2 graph stress
  check.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis contiguity -- --test-threads=1
git diff --check
```
