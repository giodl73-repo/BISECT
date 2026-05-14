---
wave: rmath-core-matrix-kernel
pulse: 01
status: done
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 01 - Dense Matrix Kernel

## Completion Notes

- Added `rmath-core` to the workspace.
- Added `DenseMatrix`, `mat_mul`, `mat_mul_vec`, `transpose`, and `invert`.
- Added typed errors for dimension mismatch, non-square inversion, non-finite
  inputs, and singular matrices.
- Wired `bisect-analysis::bloc_voting` WLS/HC3 matrix calculations through
  `rmath-core`.
- Preserved bloc-voting public error semantics by mapping `rmath-core` singular
  matrices to `BlocVotingError::Singular`.
- Added L0 inline tests, L1 integration tests, and an ignored L2 numeric stress
  test.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rmath-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis bloc_voting -- --test-threads=1
git diff --check
```
