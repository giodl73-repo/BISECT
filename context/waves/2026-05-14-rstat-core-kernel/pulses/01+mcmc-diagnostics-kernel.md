---
wave: rstat-core-kernel
pulse: 01
status: done
depends_on: []
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 01 - MCMC Diagnostics Kernel

## Completion Notes

- Added `rstat-core::mcmc`.
- Moved reusable R-hat, ESS, Hamming autocorrelation, and integrated
  autocorrelation time logic behind shared functions.
- Preserved `bisect-analysis::ensemble_diagnostics` public wrapper functions and
  JSON-facing record structs.

## Validation

```powershell
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis -- --test-threads=1
```

