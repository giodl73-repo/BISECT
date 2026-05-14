---
wave: rstat-core-kernel
pulse: 09
status: done
depends_on: [08]
governing_roles:
  - SCALE
  - BENCHMARK
  - LEDGER
---

# Pulse 09 - Close Validation

## Boundary Review

`rstat-core` is now a reusable deterministic statistics kernel. It owns generic
numeric/statistical helpers only:

- descriptive and weighted summaries;
- R-7 quantiles and percentile intervals;
- seeded bootstrap statistic and interval helpers;
- empirical p-values, ESS beta correction, Bayesian detection score, and
  multiple-testing corrections;
- MCMC diagnostics;
- regularized incomplete beta and standard Normal CDF approximations.

Domain crates retain domain semantics:

- `bisect-analysis` keeps redistricting record shapes, report interpretation,
  and legal/civic meaning.
- `rcount-stats` remains the owner for election-audit method replay semantics.
- ROUTE consumption is deferred until a portable git/local dependency plan is
  executed outside this wave.

## Validation

```powershell
cargo fmt
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p rstat-core -- --ignored --test-threads=1
$env:CARGO_INCREMENTAL='0'; cargo test -p bisect-analysis -- --test-threads=1
git diff --check
```
