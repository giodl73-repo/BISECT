---
wave: rstat-core-kernel
pulse: 05
status: done
depends_on: [04]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 05 - Hypothesis And Multiple-Testing Kernel

## Completion Notes

- Added `rstat-core::hypothesis`.
- Implemented empirical lower/upper/two-sided p-values, ESS beta-median
  correction, Bayesian detection score, Holm-Bonferroni, and
  Benjamini-Hochberg.
- Wired `bisect-analysis::permutation` through the empirical p-value and
  ESS/BDS helpers.
- Wired `bisect-analysis::bloc_voting::holm_bonferroni` through the shared
  Holm correction while preserving the public wrapper.
- Extended L0/L1/L2 coverage for hypothesis helpers and multiple-testing stress.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis permutation -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis holm -- --test-threads=1
git diff --check
```
