---
wave: rgraph-core-expansion
pulse: 02
status: done
depends_on: [01]
governing_roles:
  - BOUNDARY
  - BENCHMARK
  - LEDGER
---

# Pulse 02 - Bridge Detection Kernel

## Completion Notes

- Added `rgraph-core::bridges` and `rgraph-core::bridges_with_filter`.
- Treats the existing directed adapter as an undirected topology for bridge
  detection while returning the directed adapter edge IDs that represent each
  bridge.
- Reciprocal adapter edges for the same undirected edge are both returned; true
  parallel same-direction edges are not reported as bridges.
- Added L0 inline tests for cycle, reciprocal, parallel, and filtered cases.
- Added L1 integration coverage and an ignored L2 graph stress check.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
git diff --check
```
