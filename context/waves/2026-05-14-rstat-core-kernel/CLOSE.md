# RSTAT Core Kernel - Close

## Outcome

The wave created `rstat-core` as the first shared deterministic statistics
kernel crate.

## Landed

- `crates/rstat-core`
  - MCMC diagnostics: R-hat, ESS, Hamming autocorrelation, integrated
    autocorrelation time
  - Probability helper: regularized incomplete beta
- `bisect-analysis`
  - ensemble diagnostics now delegate reusable math to `rstat-core`
  - permutation report now uses `rstat-core` probability math

## Deferred

- Descriptive summaries, quantiles, bootstrap/permutation scaffolds, and
  multiple-testing helpers.
- Any RCOUNT audit-method extraction. `rcount-stats` remains the owner for
  election-audit replay semantics.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis -- --test-threads=1
git diff --check
```

