# RSTAT Core Kernel - Close

## Outcome

The wave created `rstat-core` as the first shared deterministic statistics
kernel crate.

## Landed

- `crates/rstat-core`
  - MCMC diagnostics: R-hat, ESS, Hamming autocorrelation, integrated
    autocorrelation time
  - Probability helpers: regularized incomplete beta, standard Normal CDF
  - Summary helpers: mean, summary statistics, median, deterministic R-7
    quantiles, percentile intervals, weighted descriptive summaries
  - Resampling helpers: seeded bootstrap statistics and percentile intervals
  - Hypothesis helpers: empirical p-values, ESS beta correction, Bayesian
    detection score, Holm-Bonferroni, Benjamini-Hochberg
- `bisect-analysis`
  - ensemble diagnostics now delegate reusable math to `rstat-core`
  - permutation report now uses `rstat-core` hypothesis and probability math
  - partisan mean-median and bootstrap percentile intervals now use
    `rstat-core` summary helpers
  - partisan bootstrap CI now uses `rstat-core` resampling helpers
  - bloc-voting Holm correction now uses `rstat-core` multiple-testing helpers
  - bloc-voting weighted standardization now uses `rstat-core` weighted summaries
  - population-weighted compactness now uses `rstat-core` weighted mean
  - bloc-voting HC3 p-values now use `rstat-core` probability helpers
- Test ladder
  - L0 inline unit tests for summary, MCMC, and probability helpers
  - L1 integration tests for composed summary/probability and MCMC diagnostics
  - L2 ignored numeric stress tests for large samples, long traces, beta grids,
    and larger partition trajectories

## Deferred

- Additional method-specific hypothesis reports beyond current BISECT consumers.
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

