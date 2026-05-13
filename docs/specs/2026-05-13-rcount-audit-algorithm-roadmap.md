# RCOUNT Audit Algorithm Roadmap

**Status:** Draft implementation roadmap  
**Date:** 2026-05-13  
**Track:** `research/tracks/V-election-audit`, plus W-series forensic analytics  
**Related docs:** [`2026-05-12-v-election-audit-paper-track.md`](2026-05-12-v-election-audit-paper-track.md),
[`2026-05-13-rcount-validation-data-landscape.md`](2026-05-13-rcount-validation-data-landscape.md),
[`../algorithm-atlas/README.md`](../algorithm-atlas/README.md)

## Purpose

The first RCOUNT implementation proved the package/verifier substrate and landed
the Rhode Island Rep. 28 source adapter. The next layer is algorithmic: every
audit method RCOUNT names needs an atlas page, a paper slot, fixtures, transcript
fields, and eventually a verifier implementation.

This roadmap separates two kinds of algorithms:

- **V-series audit algorithms:** methods that can support certification-style
  claims when their evidence assumptions are met.
- **W-series forensic analytics:** exploratory anomaly tools that can guide
  investigation but must not be treated as certification evidence.

## Sequence

| Code | Algorithm | First deliverable | Implementation target |
|---|---|---|---|
| V.12 | BRAVO ballot-polling | synthetic two-candidate fixture landed in crate tests | `rcount-audit` ballot-polling math |
| V.13 | Minerva/Athena | synthetic multi-round Minerva replay plus Athena boundary | external RI/Arlo/Athena report replay |
| V.14 | Kaplan-Markov/MACRO | overstatement-error transcript fixture | ballot/batch comparison risk math |
| V.15 | ALPHA betting martingales | fixed-bet martingale fixture landed in crate tests | shared sequential test core |
| V.16 | SHANGRLA assorters | assertion/assorter schema fixture | assertion language for V.12-V.15/V.19 |
| V.17 | Stratified/hybrid RLAs | two-stratum coordinator fixture | combined-risk math after allocation fields |
| V.18 | Batch comparison | batch hand-tally fixture | batch comparison audit replay |
| V.19 | RAIRE/AWAIRE | ranked-choice boundary fixtures | RCV/IRV audit support |
| V.20 | Bayesian audits | analytic posterior boundary fixture | non-certifying comparison transcript unless calibrated |
| V.21 | SOBA observable audits | commitment/opening boundary fixture | privacy-preserving ballot/CVR linkage |
| W.01 | Forensic anomaly analytics | injected-outlier fixture | investigation report, not verifier pass/fail |

## Shared Crate Shape

The current crates can host the first pass:

```text
rcount-core
  audit method ids
  assertion ids
  sample-unit ids
  transcript record structs
  optional BRAVO reported votes and ALPHA per-step bets

rcount-audit
  statistical replay functions
  verifier transcript integration
  positive/negative package checks

rcount-stats
  deterministic rational arithmetic
  ppm probability/risk-limit helpers
  BRAVO likelihood-ratio replay
  Minerva exact binomial tail-ratio replay with optional round boundaries
  fixed-bet bounded-mean martingale replay
  shared sequential-test primitives for Minerva and comparison audits

rcount-io
  adapter parsing
  source-summary transcripts
  algorithm-specific import sidecars

rcount-cli
  verify surfaces
  replay-audit-algorithms surface for package-level algorithm runs
  future: rcount audit <method> explain
```

`rcount-stats` should stay IO-free and package-free. It owns numerical
primitives and reusable test machinery only; RCOUNT package records remain in
`rcount-core`, and verifier decisions remain in `rcount-audit`.

## Transcript Contract

Every algorithm implementation should emit:

- `method_id`;
- `assertion_id` where applicable;
- `sample_unit_id` or `batch_id`;
- source ids backing the observation;
- declared risk limit or analytic threshold;
- per-step statistic values;
- final decision;
- unsupported/boundary fields when replay is incomplete.

The transcript must support three honest outcomes:

- `pass`: the method was replayed and supports the claim;
- `fail`: the method was replayed and contradicts the claim;
- `boundary`: source evidence was preserved, but RCOUNT cannot yet replay the
  statistical claim.

## Fixture Ladder

Each V algorithm needs the same ladder:

1. **Toy positive:** tiny enough to inspect by hand.
2. **Toy negative:** one row changed so the verifier fails for the intended
   reason.
3. **Boundary:** valid sources but missing a required assumption.
4. **Real adapter:** public artifact, when available.
5. **Tamper case:** source hash or transcript drift caught after package read.

W algorithms need:

1. no-anomaly fixture;
2. injected-anomaly fixture;
3. false-positive boundary fixture;
4. report explaining that anomaly is not proof.

## Implementation Order

The most efficient order is not numeric.

1. **V.16 SHANGRLA schema:** define assertion and assorter records without full
   math. This gives the shared language. First slice landed in `rcount-core`
   with package-level `audit_algorithm_runs` and transcript-shape verification.
2. **V.15 ALPHA toy core:** implement fixed-bet martingale replay over explicit
   assorter values. Shared exact-rational and ppm helpers now live in
   `rcount-stats` so this does not turn `rcount-audit` into a numeric utility
   crate. The first slice now replays a one-assertion fixed-bet bounded-mean
   martingale, detects declared statistic/P-value drift, and reports missing
   bets as a boundary rather than a statistical failure.
3. **V.12 BRAVO:** implement as a named ballot-polling special case and compare
   transcript shape against ALPHA. The first slice now replays two-candidate
   winner/loser/other observations from `AuditSampleStep.assorter_value` and
   checks the declared run decision. It covers pass, continue, declared-drift
   fail, and missing-field boundary cases.
4. **V.13 Minerva:** RI Rep. 28 now emits a first-class boundary-only
   `minerva-ballot-polling-v1` algorithm run, and RCOUNT replays synthetic
   one-round and multi-round two-candidate Minerva fixtures with exact
   binomial tails. Athena has a dedicated boundary fixture so method identity
   is preserved without false replay claims. Next validate against RI/Arlo-
   style public sources that expose enough per-round observations or tallies.
5. **V.14/V.18 comparison methods:** V.14 now has exact plurality comparison
   overstatement and taint primitives plus an initial package-level
   Kaplan-Markov taint-product replay transcript. V.18 reuses that arithmetic
   for `batch-comparison-v1` replay and still reports boundaries for incomplete
   batch comparison runs. V.18 also has exact
   batch-level plurality margin overstatement arithmetic, package records for
   reported/hand batch totals, core verification against batch summaries, and
   algorithm-step linkage back to verified batch overstatement taints. A core
   derivation helper now builds `batch-comparison-v1` runs from verified
   `BatchComparisonAudit` records plus sampled-batch order, and the derived
   package path round-trips through IO and CLI replay.
   `rcount-stats` also has an exact rational published MACRO product primitive,
   and package-level V.14 replay uses it when a run declares ballot count,
   reported margin, and gamma. The older taint-product transcript remains as a
   compatibility fallback for packages that do not yet carry those design
   inputs.
   Missing-hand-tally, batch-size-drift, and algorithm-taint-drift negatives are
   covered. Next validate the V.14 replay against full published
   Kaplan-Markov/MACRO fixtures, then call the batch-comparison derivation
   helper from adapters with public sample order.
6. **V.17 stratified/hybrid coordinator:** RCOUNT can now preserve a
   two-stratum coordinator run with component run ids, allocation ppm values,
   a nuisance parameter, and a combining rule, and reports combined-risk replay
   as a boundary. Combined-risk math should wait until the combining rule is
   selected and validated against a public fixture.
7. **V.19 RCV:** RCOUNT can now preserve RAIRE/AWAIRE method identity, reported
   IRV elimination order, and ranked sample choices, while rejecting malformed
   ranked ballots. Assertion generation and risk replay still wait until ranked
   CVR semantics and IRV tabulation fixtures are stable.
8. **V.20 Bayesian analytics:** RCOUNT can now preserve prior and likelihood
   ids, posterior winner probability, posterior risk, simulation seed, and draw
   count on a boundary-only analytic transcript. The verifier rejects malformed
   posterior probabilities and rejects Bayesian fields on non-Bayesian methods.
   Hand-computable posterior replay and calibrated-risk claims remain future
   work.
9. **V.21 SOBA observable audits:** RCOUNT can now preserve a boundary
   observable-ballot run whose sampled units link to anonymized inclusion
   proofs. The verifier rejects missing openings and keeps observable-ballot
   assertions scoped to the SOBA method. CVR/human-observation mismatch
   fixtures and integration with V.14 comparison math remain future work.
10. **W.01:** build outlier reports only after RCOUNT real-data adapters provide
   enough normalized public data.

## Paper/Atlas Contract

Every algorithm paper should point to:

- its atlas page;
- its fixture package directory;
- positive and negative verifier commands;
- transcript schema fields;
- source artifacts or literature references;
- a claim-boundary section.

The atlas page teaches the algorithm. The paper proves the RCOUNT contract. The
fixtures keep both honest.
