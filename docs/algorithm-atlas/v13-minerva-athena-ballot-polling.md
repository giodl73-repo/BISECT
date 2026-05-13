# V.13 Minerva And Athena Ballot-Polling RLAs

## Mental Model

Minerva and Athena are ballot-polling RLA methods designed to improve practical
sample sizes and support round-based stopping decisions. They are the family
that shows up in Arlo-style public audit reports, including the Rhode Island
Rep. 28 package RCOUNT now imports.

RCOUNT records `MINERVA` as source metadata and now replays scoped
two-candidate Minerva ballot-polling checks with exact binomial tail ratios.

## How RCOUNT Uses It

```text
round settings -> sampled ballots -> round tallies -> risk measurement -> stop/continue
```

The method belongs in RCOUNT when the public report publishes round sample
sizes, sampled ballots, risk measurements, and enough contest totals to replay
the published stopping decision.

## Step-By-Step Mechanics

1. Parse audit method, risk limit, public seed, contest totals, and round rows.
2. Reconstruct round sample counts from sampled ballots or tally sheets.
3. For each winner-loser assertion, compute the method-specific risk measure.
4. Compare the computed risk measure to the declared risk limit.
5. Record whether the round stopped, continued, or escalated.
6. Preserve the published risk measurement even if RCOUNT cannot replay it.

For the implemented round-one Minerva path, RCOUNT counts winner ballots `k`
in the first-round sample of size `n`, computes:

```text
Pr[X >= k | p = reported_winner_share] / Pr[X >= k | p = 1/2]
```

and converts the resulting likelihood ratio to a conservative ppm p-value.
For explicit multi-round packages, `AuditSampleStep.round_index` partitions the
sample into rounds and RCOUNT recomputes each round from cumulative
observations.

## RCOUNT Transcript Fields

| Field | Meaning |
|---|---|
| `method_id` | `minerva-ballot-polling-v1` or `athena-ballot-polling-v1` |
| `round_index` | optional zero-based audit round on `AuditSampleStep` |
| `declared_sample_size` | published round sample size |
| `observed_sample_size` | rows/tallies actually present |
| `winner_observations` | sampled votes for winner |
| `loser_observations` | sampled votes for loser |
| `published_risk` | risk value from source artifact |
| `computed_risk` | RCOUNT replay result, when implemented |
| `decision` | stop, continue, or boundary-only |

## Fixtures

- RI Rep. 28 boundary fixture: source-summary transcript plus package-level
  `minerva-ballot-polling-v1` algorithm run with `Boundary` decision.
- Synthetic Minerva positive fixture with one assertion and known risk:
  `synthetic_minerva_round_one_package`.
- Synthetic Minerva multi-round fixture where round one continues and round
  two stops: `synthetic_minerva_multi_round_package`.
- Synthetic drift fixture where published risk differs from computed risk.
- Synthetic Athena boundary fixture that preserves method identity and sample
  evidence without claiming replay: `synthetic_athena_boundary_package`.

## Current Implementation

`rcount-stats` owns `replay_minerva_round_one_ballot_polling` and
`replay_minerva_ballot_polling_rounds`, which use exact rational arithmetic
over binomial tails. `rcount-audit` exposes them through
`replay_audit_algorithm_statistics` for `minerva-ballot-polling-v1` when the
run has risk limit, reported winner/loser votes, and winner/loser sample
steps. If no `round_index` values are present, the run is treated as one round;
if any are present, every step must carry a nondecreasing `round_index`.

Because Minerva is round-level while the current transcript step type is
sample-step-level, RCOUNT records each computed round statistic on the final
sample step in that round. Declared statistic or p-value drift on those final
round steps fails replay.

The Rhode Island Rep. 28 adapter still preserves Minerva as a first-class
algorithm surface without claiming to recompute the source risk statistic:

- `audit_algorithm_runs` includes `minerva-ballot-polling-v1`;
- the run records risk limit, reported winner/loser votes, source refs, and the
  top-two plurality assertion;
- `rcount replay-audit-algorithms` emits `status: boundary` until enough
  sampled ballot observations are present for the implemented replay surface;
- `rcount verify` still verifies source hashes, count arithmetic,
  batch/accounting checks, and sampled-ballot/retrieval consistency.

Athena remains boundary-only in RCOUNT. `synthetic_athena_boundary_package`
proves the package, IO, and CLI surfaces can carry `athena-ballot-polling-v1`
without conflating it with the Minerva replay path.

## Claim Boundary

RCOUNT may claim synthetic Minerva replay for two-candidate winner/loser
observations, including explicit multi-round package fixtures. It may claim
Athena method preservation and boundary reporting, but not Athena risk replay.
Athena replay and Rhode Island source-risk replay require published per-round
observation/tally evidence, method-specific Athena parameters, and validated
tolerance against external reports.

## References

- Arlo report guide: <https://docs.voting.works/arlo/resources/audit-report-guide>
- Athena paper: <https://arxiv.org/abs/2008.02315>
- PROVIDENCE / R2B2 context: <https://arxiv.org/abs/2210.08717>
