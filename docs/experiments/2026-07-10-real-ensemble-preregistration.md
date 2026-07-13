# Real Ensemble Evidence Preregistration

**Date frozen:** 2026-07-10, before production chains  
**Wave:** National Standard Evidence And Specification, Pulse 04  
**Status:** Internal preregistration; not external registration or peer review

## Research Questions

1. Where does the NRS v0.1 geographic benchmark fall in the tract-graph
   cut-fraction distributions produced by two ReCom implementations?
2. Does the benchmark occupy an extreme partisan-seat position under the 2016
   and 2020 presidential tract estimates?
3. Do Rust ReCom and independently maintained GerryChain produce materially
   similar distributions under the same graph, initial plan, tolerance, seeds,
   and chain length?

## States

| State | Districts | Reason |
|---|---:|---|
| Rhode Island | 2 | Small coastal state; simplest nontrivial partition |
| Wisconsin | 8 | Medium district count and litigated partisan geography |
| North Carolina | 14 | Larger, high-salience state with urban/rural sorting |

These states provide different graph sizes, district counts, and political
geographies. They are not a nationally representative sample.

## Inputs

- 2020 tract adjacency binaries and GEOID maps already used by BISECT.
- NRS reference profile: `standard-bisect`, geographic weights, single fixed
  seed `424242`, `c-ffi`.
- Presidential tract estimates from the Redistricting Data Hub files
  `tracts-2016-RLCR.csv` and `tracts-2020-RLCR.csv`.
- Two-party vote totals use the named Democratic and Republican presidential
  columns only. Other-party votes are excluded from the seat calculation.

The two elections are selected because matching tract-level national estimates
are locally available under one documented methodology. They measure
sensitivity across two presidential cycles, not all electoral environments.

## Samplers

1. `bisect-ensemble` Rust ReCom.
2. GerryChain ReCom installed from PyPI and recorded by version.

Both begin from the same BISECT benchmark assignment.

## Chain Plan

- Four independent chains per state per implementation.
- 2,000 recorded steps per chain after the initial state.
- Base seeds: `20260710`, with implementation-specific deterministic
  per-chain derivation recorded in each trace.
- Population tolerance: begin with `0.005` (0.5%). A state may use the first
  preregistered fallback in `[0.01, 0.02, 0.03]` only if the 100-step pilot
  acceptance rate is below 1%. The selected tolerance and pilot result must be
  reported. Cross-tool comparisons use the same selected state tolerance.
- Partition snapshots every 10 steps for Hamming diagnostics.
- No cherry-picking or replacement of chains after metrics are observed.
  Failed chains remain archived and reported.

## Metrics

- Graph cut fraction.
- Accepted-step rate.
- Maximum district population deviation.
- Democratic seats under 2016 presidential estimates.
- Democratic seats under 2020 presidential estimates.
- Benchmark percentile for cut fraction and each election's Democratic seats.
- R-hat on cut fraction and Democratic seats.
- ESS on each scalar metric.
- Partition-space Hamming autocorrelation and integrated autocorrelation time.
- Cross-tool distribution differences: mean, standard deviation, quantiles,
  and two-sample Kolmogorov-Smirnov statistic.

Graph cut fraction is a topology proxy. It is not Polsby--Popper and must not be
published as geometric compactness without polygon-based recomputation.

## Burn-in And Stopping

- Primary analysis discards the first 500 steps of each 2,000-step chain.
- No adaptive extension is allowed for the primary tables.
- If any post-burn-in cut-fraction R-hat is at least 1.05 or pooled ESS is below
  100, the result is labeled non-converged. A longer follow-up may be archived
  separately but does not replace the preregistered primary run.

## Percentiles And Uncertainty

The benchmark percentile is the empirical mid-rank:

```text
(count(sample < benchmark) + 0.5 * count(sample == benchmark)) / N
```

Report a 95% bootstrap interval using whole-chain block resampling. Tail claims
below 1% or above 99% are prohibited unless effective sample size is at least
1,000 for the relevant metric.

## Negative Results

The package must retain and report:

- low acceptance;
- non-convergence;
- implementation disagreement;
- benchmark median rather than extremum placement;
- election-to-election sign or percentile changes; and
- unavailable or unmatched tract election records.

## Claim Boundary

This experiment evaluates three tract-level state examples under one profile.
It cannot establish a universal neutral distribution, national partisan
fairness, legal validity, VRA compliance, or block-level NRS conformance.
