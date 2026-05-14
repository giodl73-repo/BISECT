---
wave: rgraph-core-expansion
pulse: 03
status: done
depends_on: [02]
governing_roles:
  - BOUNDARY
  - BENCHMARK
  - LEDGER
---

# Pulse 03 - Articulation Point Kernel

## Completion Notes

- Added `rgraph-core::articulation_points` and
  `rgraph-core::articulation_points_with_filter`.
- Uses the same undirected projection of the directed graph adapter as bridge
  detection.
- Kept the helper kernel-only; domain crates own any interpretation as route
  redundancy, district fragility, or civic evidence.
- Added L0 inline tests for cycle, root, and filtered cases.
- Added L1 integration coverage and an ignored L2 graph stress check.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rgraph-core -- --ignored --test-threads=1
git diff --check
```
