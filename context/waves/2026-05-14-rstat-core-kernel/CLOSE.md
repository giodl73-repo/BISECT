# RSTAT Core Kernel - Close

## Outcome

The wave created `rstat-core` as the first shared deterministic statistics
kernel crate.

## Landed

- `crates/rstat-core`
  - MCMC diagnostics: R-hat, ESS, Hamming autocorrelation, integrated
    autocorrelation time
  - Probability helper: regularized incomplete beta
  - Summary helpers: mean, summary statistics, median, deterministic R-7
    quantiles, percentile intervals
- `bisect-analysis`
  - ensemble diagnostics now delegate reusable math to `rstat-core`
  - permutation report now uses `rstat-core` probability math
  - partisan mean-median and bootstrap percentile intervals now use
    `rstat-core` summary helpers
- Test ladder
  - L0 inline unit tests for summary, MCMC, and probability helpers
  - L1 integration tests for composed summary/probability and MCMC diagnostics
  - L2 ignored numeric stress tests for large samples, long traces, beta grids,
    and larger partition trajectories

## Deferred

- Bootstrap/permutation scaffolds and multiple-testing helpers.
- Any RCOUNT audit-method extraction. `rcount-stats` remains the owner for
  election-audit replay semantics.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis -- --test-threads=1
git diff --check
```

