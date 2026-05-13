# V.15 ALPHA And Betting-Martingale Audits

## Mental Model

ALPHA treats an audit as a sequential martingale test. Instead of hard-coding a
single ballot-polling or comparison formula, it updates evidence using an
adaptive test martingale. This makes it a strong general-purpose target for
RCOUNT because the same machinery can support polling, comparison, stratified,
weighted, and SHANGRLA-style assertions.

## How RCOUNT Uses It

```text
assertion assorter -> sample stream -> betting strategy -> martingale value -> risk decision
```

ALPHA is the preferred implementation core for modern RCOUNT RLA replay because
it can host multiple audit designs without a new transcript shape for each one.

## Step-By-Step Mechanics

1. Define the assertion as a mean test over an assorter value.
2. Choose sampling mode: with replacement, without replacement, Bernoulli, or
   weighted.
3. For each sampled unit, compute the observed assorter value.
4. Select or replay the betting parameter for that step.
5. Update the martingale/test statistic.
6. Convert the running statistic to a P-value or risk measure.
7. Stop when the risk limit is met.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `alpha-martingale-v1` |
| `assertion_id` | assertion under test |
| `assorter_id` | function mapping ballots/CVRs to values |
| `sampling_mode` | with-replacement, without-replacement, Bernoulli, weighted |
| `draw_index` | sample index |
| `assorter_value` | observed value for the unit |
| `bet` | betting parameter used at this step |
| `martingale` | running test value |
| `p_value` | derived risk measure |

## Fixtures

- Fixed-bet toy crate fixture with hand-computable values.
- Package-level CLI fixture for fixed-bet pass.
- Package-level CLI fixture for declared P-value drift.
- Package-level CLI boundary fixture for missing per-step bets.
- Adaptive-bet fixture with deterministic bet transcript.
- Without-replacement fixture against a finite population.
- Stratified fixture that demonstrates why ALPHA is the shared core.

## Current Implementation

`rcount-stats` owns the first deterministic martingale primitive:

- `replay_fixed_bet_bounded_mean_martingale`;
- exact rational observations, bets, update factors, and martingale values;
- P-value ppm computed from `1 / martingale`;
- risk-limit stop threshold computed exactly.

`rcount-core::AuditSampleStep` now has an optional `bet` field. `rcount-audit`
adds `replay_audit_algorithm_statistics` for `alpha-martingale-v1`; the first
slice supports one assertion, uses half the declared assorter upper bound as the
null mean, and reports a `boundary` transcript when required bets are missing.

## Claim Boundary

ALPHA can unify many audit methods, but only if RCOUNT records the assorter,
sampling mode, betting rule, and draw order exactly. Without those fields, a
package may preserve evidence but cannot replay the statistical claim.

## References

- ALPHA paper: <https://arxiv.org/abs/2201.02707>
- SHANGRLA paper: <https://arxiv.org/abs/1911.10035>
