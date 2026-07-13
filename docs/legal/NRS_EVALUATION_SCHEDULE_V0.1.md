# Statutory Technical Schedule B: National Evaluation Profile v0.1

**Status:** Candidate schedule incorporated by the model statute

## 1. Plan checks

Report for benchmark and final plan:

- district count, complete assignment, and contiguity;
- ideal population, maximum deviation, total deviation, and district values;
- Polsby--Popper, Reock, convex-hull ratio, and Schwartzberg, with versioned
  projection and geometry rules;
- county and municipality splits;
- every submitted community split and disposition;
- majority-minority and opportunity diagnostics labeled as non-legal metrics;
- disparate-impact review; and
- baseline-to-final moved blocks, population, and districts touched.

## 2. Elections

The national profile uses:

- the two most recent presidential general elections; and
- the two most recent U.S. House general elections

available before profile lock with public precinct returns and a published
precinct-to-block crosswalk. Every election, crosswalk, aggregation rule,
uncontested-race treatment, and missing-data rule is hash-bound.

If an election lacks a defensible crosswalk, the report identifies the gap and
does not substitute zeros or a different election after viewing outcomes.

## 3. Partisan diagnostics

For each named election report:

- district Democratic and Republican two-party share;
- seat count;
- efficiency gap;
- mean-median difference;
- partisan bias under a uniform swing;
- seats-votes curve inputs and method; and
- proportionality gap.

No threshold in this schedule determines legal fairness.

## 4. Sensitivity

Report:

- the statutory benchmark seed;
- 100 diagnostic seeds derived from
  `SHA-256("NRS_SENSITIVITY_V1" || input_manifest || seed_index)`;
- assignment similarity, cut objective, population, and partisan metrics across
  those seeds; and
- block versus any coarser research resolution when available.

Diagnostic seeds do not replace the statutory benchmark.

## 5. Ensembles

Ensemble reporting is optional unless required by controlling law. If reported:

- at least four chains;
- precommitted proposal, tolerance, initial plans, seeds, burn-in, thinning,
  and stopping rule;
- R-hat below 1.05;
- ESS at least 100 for descriptive inference;
- ESS at least 1,000 for a percentile below 1% or above 99%;
- partition-space autocorrelation;
- archived traces and uncertainty; and
- cross-tool comparison when a legal claim depends on implementation
  invariance.

Failure of a diagnostic is reported; it does not authorize replacement of the
sample after viewing results.

## 6. Profile lock

This schedule, named elections, metric versions, source hierarchy, and
jurisdiction additions are locked before benchmark or candidate-plan
generation.
