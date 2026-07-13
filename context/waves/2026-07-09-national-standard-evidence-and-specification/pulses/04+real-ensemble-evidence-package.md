---
pulse: 04
title: Real ensemble evidence package
status: done
depends_on: 01, 03
wave: national-standard-evidence-and-specification
validation_level: L2 empirical evidence
---

# Pulse 04 - Real Ensemble Evidence Package

## Purpose

Replace the synthetic-only foundation of the neutrality-percentile argument
with archived, diagnosable, cross-tool real-state evidence.

## Minimum Experiment

- At least three structurally different states.
- Multiple elections or a documented election-selection rationale.
- BISECT baseline plus an independent ensemble implementation.
- At least four chains with R-hat, ESS, and partition-space autocorrelation.
- Pre-registered metrics and stopping rules.
- Negative and heterogeneous results reported.

## Deliverables

- [x] Archived plan traces and immutable input hashes.
- [x] Election, metric, RPLAN, and RCTX evidence packages.
- [x] Cross-tool comparison against GerryChain, `redist`, or another
      independently maintained implementation.
- [x] Recomputed percentile and uncertainty tables.
- [x] Updated G.1-G.3 and A.0 claims.
- [x] DATUM and SCALE panel review.

## Validation

Run package verifiers, convergence diagnostics, claim reproduction scripts, and:

```powershell
git --no-pager diff --check
```

Results:

- Rust and GerryChain traces completed for RI, IA, and NC.
- Package verifier and RPLAN audits passed.
- `bisect-ensemble` tests passed.
- G.1, G.2, G.3, and A.0 rebuilt successfully.
- `git --no-pager diff --check` passed.

## Evidence

- `docs/examples/g-ensemble-evidence-packages/G.1-G.3+real-2020/`
- `context/waves/2026-07-09-national-standard-evidence-and-specification/panels/pulse-04-real-ensemble-review.md`

## Closure Rule

Closed at L2 empirical-evidence level. Every retained published percentile
regenerates from the archived package. Wisconsin's contiguity failure,
cross-tool disagreement, incomplete 2016 coverage, and NC/IA ESS limitations
are reported rather than suppressed.
